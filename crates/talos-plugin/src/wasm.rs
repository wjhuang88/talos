//! WASM plugin runtime adapter (T46, ADR-032).
//!
//! Loads and executes a WASM module with deterministic resource limits (fuel)
//! and a wall-clock timeout guard (epoch interruption). No host calls are
//! provided — the module runs in full sandbox isolation. All failures degrade
//! to recoverable errors; none may panic the host process (Hard Constraint #9).

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum WasmError {
    #[error("module compilation failed: {0}")]
    Compile(String),
    #[error("module instantiation failed: {0}")]
    Instantiate(String),
    #[error("exported function 'run' not found or has wrong signature")]
    MissingExport,
    #[error("execution trapped: {0}")]
    Trap(String),
    #[error("execution timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
}

pub struct WasmRuntime {
    engine: Arc<wasmtime::Engine>,
    fuel: u64,
    timeout: Duration,
}

impl WasmRuntime {
    pub fn new(fuel: u64, timeout_ms: u64) -> Result<Self, WasmError> {
        let mut config = wasmtime::Config::new();
        config.consume_fuel(true);
        config.epoch_interruption(true);
        let engine =
            wasmtime::Engine::new(&config).map_err(|e| WasmError::Compile(e.to_string()))?;
        Ok(Self {
            engine: Arc::new(engine),
            fuel,
            timeout: Duration::from_millis(timeout_ms),
        })
    }

    pub fn engine(&self) -> &wasmtime::Engine {
        &self.engine
    }
}

pub struct WasmModule {
    module: wasmtime::Module,
    runtime: Arc<WasmRuntime>,
}

impl WasmModule {
    pub fn from_wat(runtime: Arc<WasmRuntime>, wat: &str) -> Result<Self, WasmError> {
        let module = wasmtime::Module::new(&runtime.engine, wat)
            .map_err(|e| WasmError::Compile(e.to_string()))?;
        Ok(Self { module, runtime })
    }

    pub fn from_bytes(runtime: Arc<WasmRuntime>, bytes: &[u8]) -> Result<Self, WasmError> {
        let module = wasmtime::Module::from_binary(&runtime.engine, bytes)
            .map_err(|e| WasmError::Compile(e.to_string()))?;
        Ok(Self { module, runtime })
    }

    pub fn execute(&self) -> Result<i32, WasmError> {
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.execute_inner()));
        match result {
            Ok(inner) => inner,
            Err(_) => Err(WasmError::Trap("host panic during execution".into())),
        }
    }

    fn execute_inner(&self) -> Result<i32, WasmError> {
        let engine = self.runtime.engine.clone();
        let mut store = wasmtime::Store::new(&engine, ());
        store
            .set_fuel(self.runtime.fuel)
            .map_err(|e| WasmError::Instantiate(e.to_string()))?;

        store.epoch_deadline_trap();
        store.set_epoch_deadline(1);

        let engine_for_timeout = engine.clone();
        let timeout = self.runtime.timeout;
        let timeout_handle = thread::spawn(move || {
            thread::sleep(timeout);
            engine_for_timeout.increment_epoch();
        });

        let instance = wasmtime::Instance::new(&mut store, &self.module, &[])
            .map_err(|e| WasmError::Instantiate(e.to_string()))?;

        let func = instance
            .get_typed_func::<(), i32>(&mut store, "run")
            .map_err(|_| WasmError::MissingExport)?;

        let result = func.call(&mut store, ());
        timeout_handle.thread().unpark();
        let _ = timeout_handle.join();

        match result {
            Ok(val) => Ok(val),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("epoch") || msg.contains("interrupt") || msg.contains("fuel") {
                    Err(WasmError::Timeout {
                        timeout_ms: self.runtime.timeout.as_millis() as u64,
                    })
                } else {
                    Err(WasmError::Trap(msg))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FUEL: u64 = 100_000;
    const TIMEOUT_MS: u64 = 3_000;

    fn runtime() -> Arc<WasmRuntime> {
        Arc::new(WasmRuntime::new(FUEL, TIMEOUT_MS).expect("runtime"))
    }

    #[test]
    fn success_fixture() {
        let wat = r#"
            (module
              (func (export "run") (result i32)
                i32.const 42))
        "#;
        let module = WasmModule::from_wat(runtime(), wat).expect("compile");
        assert_eq!(module.execute().unwrap(), 42);
    }

    #[test]
    fn invalid_module_rejected() {
        let result = WasmModule::from_bytes(runtime(), b"not a wasm module");
        assert!(result.is_err());
        assert!(matches!(result, Err(WasmError::Compile(_))));
    }

    #[test]
    fn trap_handled_gracefully() {
        let wat = r#"
            (module
              (func (export "run") (result i32)
                unreachable))
        "#;
        let module = WasmModule::from_wat(runtime(), wat).expect("compile");
        let result = module.execute();
        assert!(matches!(result, Err(WasmError::Trap(_))));
    }

    #[test]
    fn fuel_exhaustion_handled() {
        let wat = r#"
            (module
              (func (export "run") (result i32)
                (loop $forever
                  (br $forever))
                (i32.const 0)))
        "#;
        let low_fuel_runtime = Arc::new(WasmRuntime::new(100, TIMEOUT_MS).expect("runtime"));
        let module = WasmModule::from_wat(low_fuel_runtime, wat).expect("compile");
        let result = module.execute();
        assert!(result.is_err());
    }

    #[test]
    fn timeout_handled() {
        let wat = r#"
            (module
              (func (export "run") (result i32)
                (loop $forever
                  (br $forever))
                (i32.const 0)))
        "#;
        let fast_timeout_runtime = Arc::new(WasmRuntime::new(1_000_000_000, 200).expect("runtime"));
        let module = WasmModule::from_wat(fast_timeout_runtime, wat).expect("compile");
        let result = module.execute();
        assert!(result.is_err());
    }

    #[test]
    fn memory_access_bounds_enforced() {
        let wat = r#"
            (module
              (memory (export "memory") 1)
              (func (export "run") (result i32)
                (i32.load (i32.const 0xFFFFFF00))))
        "#;
        let module = WasmModule::from_wat(runtime(), wat).expect("compile");
        let result = module.execute();
        assert!(matches!(result, Err(WasmError::Trap(_))));
    }

    #[test]
    fn missing_export_rejected() {
        let wat = r#"
            (module
              (func (export "other") (result i32)
                i32.const 0))
        "#;
        let module = WasmModule::from_wat(runtime(), wat).expect("compile");
        let result = module.execute();
        assert!(matches!(result, Err(WasmError::MissingExport)));
    }

    #[test]
    fn no_host_imports_available() {
        let wat = r#"
            (module
              (import "env" "read_file" (func $read_file (param i32) (result i32)))
              (func (export "run") (result i32)
                (call $read_file (i32.const 0))))
        "#;
        let result = WasmModule::from_wat(runtime(), wat);
        assert!(result.is_err() || result.unwrap().execute().is_err());
    }
}
