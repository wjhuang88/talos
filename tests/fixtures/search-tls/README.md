# Public synthetic TLS fixture

These PEM files and the publicly known leaf private key are disposable test materials, never
production credentials. The CA signing key was discarded. Leaf/root validity is2020..2045;
the expired leaf ends2021. Valid/expired SANs are search-fixture.invalid and127.0.0.1; host-only.pem deliberately lacks
the IP SAN. Regenerate/review before
the positive certificate expires. No OS trust store is changed.

Only cfg(test) SearchTransport accepts this root, via reqwest tls_certs_merge; hostname/IP/date
verification remains enabled. The fixture server uses Python's standard ssl/socket modules,
already available on repository CI runners. It is a test-only host utility, not a production
runtime or Python FFI/dependency. Missing Python fails the test; nothing is silently skipped.
Offline fixture generation used cryptography50.0.2; test execution does not require that package
or openssl CLI. Replacement trigger: an approved Rust-native TLS test server dependency.

The owned child binds loopback only. Its CONNECT witness relays solely to its own fixed TLS
port/IP, records SNI/application/CONNECT receipts, drains pending accepts and joins workers at
shutdown. Rust bounds startup/operation and child polling, and kills a stuck owned child. OS kill/wait
and reader-join cleanup tails are not independently interruptible; receipts require quiescence.
