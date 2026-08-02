// Temporary I169 test migration wrapper. build.rs copies the preserved test
// source and applies only explicit receipt-protocol migrations. This wrapper
// is removed once the migrated tests are committed as normal Rust source.
include!(concat!(env!("OUT_DIR"), "/tests_impl.rs"));
