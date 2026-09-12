(module
  (memory (export "memory") 1)
  (func (export "talos_language_abi_version") (result i32) i32.const 1)
  (func (export "talos_language_run") (param i32 i32) (result i64) i64.const 0))
