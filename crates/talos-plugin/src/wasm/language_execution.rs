//! Per-invocation sandbox limits and deadline custody (ADR-079).

use super::{WasmError, WasmModule, classify_execution_error};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

// These are sandbox ceilings, not additional public configuration fields. Keeping the existing
// WasmProviderLimits shape avoids breaking callers that construct it with struct literals.
const MAX_MEMORY_BYTES: usize = 64 * 1024 * 1024;
const MAX_TABLE_ELEMENTS: usize = 16_384;

/// Owns the timer until the store's last use; completion wakes and joins it immediately.
struct DeadlineGuard {
    completed: mpsc::Sender<()>,
    worker: Option<thread::JoinHandle<()>>,
}

impl Drop for DeadlineGuard {
    fn drop(&mut self) {
        let _ = self.completed.send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

pub(super) struct Invocation {
    pub(super) store: wasmtime::Store<wasmtime::StoreLimits>,
    pub(super) instance: wasmtime::Instance,
    deadline: Instant,
    timeout: Duration,
    _guard: DeadlineGuard,
}

impl Invocation {
    pub(super) fn new(
        module: &WasmModule,
        fuel: u64,
        timeout: Duration,
    ) -> Result<Self, WasmError> {
        if module.module.imports().next().is_some() {
            return Err(WasmError::Instantiate(
                "language providers cannot import host functions".into(),
            ));
        }
        let deadline = Instant::now()
            .checked_add(timeout)
            .ok_or_else(|| WasmError::Instantiate("provider timeout exceeds clock range".into()))?;
        let engine = &module.runtime.engine;
        let limits = wasmtime::StoreLimitsBuilder::new()
            .memory_size(MAX_MEMORY_BYTES)
            .table_elements(MAX_TABLE_ELEMENTS)
            .instances(1)
            .memories(1)
            .tables(1)
            .trap_on_grow_failure(true)
            .build();
        let mut store = wasmtime::Store::new(engine, limits);
        store.limiter(|limits| limits);
        store
            .set_fuel(fuel)
            .map_err(|error| WasmError::Instantiate(error.to_string()))?;
        // Epoch counters belong to the shared engine. Another invocation's timer may tick it,
        // but only this store's absolute deadline can time out this invocation.
        store.epoch_deadline_callback(move |_| {
            if Instant::now() >= deadline {
                Err(wasmtime::Error::msg("provider execution deadline exceeded"))
            } else {
                Ok(wasmtime::UpdateDeadline::Continue(1))
            }
        });
        store.set_epoch_deadline(1);
        let (completed, receiver) = mpsc::channel();
        let timer_engine = Arc::clone(engine);
        let worker = thread::Builder::new()
            .name("talos-wasm-deadline".into())
            .spawn(move || {
                if matches!(
                    receiver.recv_timeout(deadline.saturating_duration_since(Instant::now())),
                    Err(mpsc::RecvTimeoutError::Timeout)
                ) {
                    // Continue ticking until completion: another store's tick can enter this
                    // store's callback just before its deadline. A sole final tick could then
                    // be absorbed by Wasmtime applying Continue(1) after that callback returns.
                    loop {
                        timer_engine.increment_epoch();
                        match receiver.recv_timeout(Duration::from_millis(1)) {
                            Err(mpsc::RecvTimeoutError::Timeout) => {}
                            _ => break,
                        }
                    }
                }
            })
            .map_err(|error| WasmError::Instantiate(format!("deadline worker: {error}")))?;
        let guard = DeadlineGuard {
            completed,
            worker: Some(worker),
        };
        // The guard and all resource controls exist before a guest start function can execute.
        let instance = wasmtime::Instance::new(&mut store, &module.module, &[])
            .map_err(|error| classify_execution_error(error, timeout))?;
        let invocation = Self {
            store,
            instance,
            deadline,
            timeout,
            _guard: guard,
        };
        invocation.check_deadline()?;
        Ok(invocation)
    }

    pub(super) fn check_deadline(&self) -> Result<(), WasmError> {
        if Instant::now() >= self.deadline {
            Err(WasmError::Timeout {
                timeout_ms: self.timeout.as_millis().min(u64::MAX as u128) as u64,
            })
        } else {
            Ok(())
        }
    }

    pub(super) fn version(&mut self) -> Result<u32, WasmError> {
        self.check_deadline()?;
        let version = self
            .instance
            .get_typed_func::<(), i32>(&mut self.store, super::WASM_LANGUAGE_ABI_EXPORT)
            .map_err(|_| WasmError::MissingLanguageProviderExport)?
            .call(&mut self.store, ())
            .map_err(|error| classify_execution_error(error, self.timeout))?;
        self.check_deadline()?;
        super::validate_abi_version(version as u32)
            .map_err(|error| WasmError::Instantiate(error.into()))?;
        Ok(version as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wasm::{WasmLanguageProvider, WasmRuntime, validate_language_provider_abi};
    use talos_text::wasm_provider::WasmProviderLimits;

    fn module(wat: &str) -> WasmModule {
        WasmModule::from_wat(
            Arc::new(WasmRuntime::new(100_000, 1000).expect("runtime")),
            wat,
        )
        .expect("module")
    }

    fn v2(allocator: &str, run: &str) -> WasmModule {
        module(&format!(
            r#"(module
            (memory (export "memory") 1)
            (func (export "talos_language_abi_version") (result i32) i32.const 2)
            {allocator}
            (func (export "talos_language_run") (param $ptr i32) (param $len i32) (result i64)
                {run}))"#
        ))
    }

    #[test]
    fn allocated_transport_writes_only_the_owned_buffer() {
        let guest = v2(
            r#"(func (export "talos_language_alloc") (param i32) (result i32)
                i32.const 0 i32.const 73 i32.store8 i32.const 1024)"#,
            r#"i32.const 0 i32.load8_u i32.const 73 i32.ne if unreachable end
                local.get $ptr i32.const 1024 i32.ne if unreachable end
                local.get $ptr i64.extend_i32_u i64.const 32 i64.shl
                local.get $len i64.extend_i32_u i64.or"#,
        );
        let provider = WasmLanguageProvider::new(WasmProviderLimits::default());
        provider.validate_module(&guest).expect("admission");
        assert_eq!(
            provider.execute_payload(&guest, b"first").expect("first"),
            b"first"
        );
        assert_eq!(
            provider
                .execute_payload(&guest, b"different")
                .expect("next"),
            b"different"
        );
    }

    #[test]
    fn highlight_rejects_offsets_inside_unicode_characters() {
        let payload = r#"{"Spans":[[0,1,"keyword"]]}"#;
        let packed = (2048u64 << 32) | payload.len() as u64;
        let guest = module(&format!(
            r#"(module
            (memory (export "memory") 1)
            (data (i32.const 2048) "{}")
            (func (export "talos_language_abi_version") (result i32) i32.const 1)
            (func (export "talos_language_run") (param i32 i32) (result i64) i64.const {packed}))"#,
            payload.replace('"', r#"\22"#)
        ));
        let result = WasmLanguageProvider::new(WasmProviderLimits::default())
            .execute_highlight(
                &guest,
                &talos_text::wasm_provider::ProviderRequest {
                    language: "rust".into(),
                    source: "中文".into(),
                },
            )
            .expect("safe fallback");
        assert!(matches!(result, talos_text::HighlightResult::PlainText));
    }

    #[test]
    fn invalid_allocator_never_retries_legacy_transport() {
        let provider = WasmLanguageProvider::new(WasmProviderLimits::default());
        for allocator in [
            "",
            r#"(func (export "talos_language_alloc") (result i32) i32.const 1024)"#,
            r#"(func (export "talos_language_alloc") (param i32) (result i32) i32.const 0)"#,
            r#"(func (export "talos_language_alloc") (param i32) (result i32) i32.const 65535)"#,
            r#"(func (export "talos_language_alloc") (param i32) (result i32) unreachable)"#,
        ] {
            let guest = v2(allocator, "i64.const 0");
            assert!(
                provider.execute_payload(&guest, b"four").is_err(),
                "{allocator}"
            );
        }
    }

    #[test]
    fn admission_enforces_initial_memory_and_table_limits() {
        for declaration in [
            "(memory 1025)",
            "(table 16385 funcref)",
            "(memory 1) (memory 1)",
        ] {
            let guest = module(&format!(
                r#"(module {declaration}
                (func (export "talos_language_abi_version") (result i32) i32.const 1))"#
            ));
            assert!(
                validate_language_provider_abi(&guest).is_err(),
                "{declaration}"
            );
        }
    }

    #[test]
    fn public_probe_bounds_start_and_version_loops() {
        for body in [
            r#"(func $start (loop $again br $again)) (start $start)
                (func (export "talos_language_abi_version") (result i32) i32.const 1)"#,
            r#"(func (export "talos_language_abi_version") (result i32)
                (loop $again br $again) i32.const 1)"#,
        ] {
            let guest = module(&format!("(module {body})"));
            assert!(validate_language_provider_abi(&guest).is_err());
        }
    }

    #[test]
    fn guest_growth_and_output_are_bounded_before_copy() {
        let provider = WasmLanguageProvider::new(WasmProviderLimits::default());
        let allocator =
            r#"(func (export "talos_language_alloc") (param i32) (result i32) i32.const 1024)"#;
        let grow = v2(allocator, "i32.const 1024 memory.grow drop i64.const 0");
        assert!(provider.execute_payload(&grow, b"x").is_err());
        let oversized = v2(allocator, "i64.const 4294967295");
        let error = provider
            .execute_payload(&oversized, b"x")
            .expect_err("oversized");
        assert!(error.to_string().contains("response exceeds limit"));
        assert_eq!(
            provider
                .execute_payload(&v2(allocator, "i64.const 0"), b"x")
                .expect("healthy"),
            b""
        );
    }

    #[test]
    fn another_store_epoch_tick_does_not_cancel_this_invocation() {
        let guest = module(r#"(module (func (export "run") (result i32) i32.const 7))"#);
        let mut invocation =
            Invocation::new(&guest, 1000, Duration::from_secs(30)).expect("invocation");
        guest.runtime.engine.increment_epoch();
        let function = invocation
            .instance
            .get_typed_func::<(), i32>(&mut invocation.store, "run")
            .expect("run");
        assert_eq!(
            function
                .call(&mut invocation.store, ())
                .expect("not expired"),
            7
        );
        // Drop must wake the timer now, not wait for its thirty-second deadline.
        let before = Instant::now();
        drop(invocation);
        assert!(before.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn language_deadline_stops_run_without_exhausting_fuel() {
        let allocator =
            r#"(func (export "talos_language_alloc") (param i32) (result i32) i32.const 1024)"#;
        let guest = v2(allocator, "(loop $again br $again) i64.const 0");
        let provider = WasmLanguageProvider::new(WasmProviderLimits {
            fuel: u64::MAX,
            timeout: Duration::from_millis(20),
            ..WasmProviderLimits::default()
        });
        assert!(matches!(
            provider.execute_payload(&guest, b"x"),
            Err(WasmError::Timeout { .. })
        ));
    }

    #[test]
    fn deadline_repeats_tick_when_callback_absorbs_the_first_tick() {
        let guest = module(r#"(module (func (export "run") (loop $again br $again)))"#);
        let engine = Arc::clone(&guest.runtime.engine);
        let (completed, result) = mpsc::channel();
        let worker = thread::spawn(move || {
            let mut invocation =
                Invocation::new(&guest, u64::MAX, Duration::from_millis(10)).expect("invocation");
            let mut continued_once = false;
            // Force the real interleaving's effect: the callback applies Continue(1) AFTER
            // our own deadline tick, so that tick cannot satisfy the new epoch deadline.
            invocation.store.epoch_deadline_callback(move |_| {
                if !continued_once {
                    continued_once = true;
                    Ok(wasmtime::UpdateDeadline::Continue(1))
                } else {
                    Err(wasmtime::Error::msg("test deadline reached"))
                }
            });
            let run = invocation
                .instance
                .get_typed_func::<(), ()>(&mut invocation.store, "run")
                .expect("run");
            let failed = run.call(&mut invocation.store, ()).is_err();
            drop(invocation);
            let _ = completed.send(failed);
        });
        let outcome = result.recv_timeout(Duration::from_secs(5));
        // A regression must fail, not leave the test suite hung with an effectively infinite
        // fuel budget. One rescue tick releases the second callback, then join the worker.
        if outcome.is_err() {
            engine.increment_epoch();
        }
        worker.join().expect("worker joined");
        assert_eq!(outcome, Ok(true), "a single final tick loses the deadline");
    }
}
