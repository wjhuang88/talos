# Public synthetic TLS fixture

These PEM files and the publicly known leaf private key are disposable test materials, never
production credentials. The CA signing key was discarded. Root validity is2020..2045;
positive/host-only leaves are2026-10-01..2027-10-01 (365days), the expired leaf ends2021.
Valid/expired SANs are search-fixture.invalid and127.0.0.1; host-only.pem deliberately lacks
the IP SAN. Regenerate/review before2027-10-01. No OS trust store is changed.

The original25-year positive leaf failed the Apple lifetime requirement (<=825days), despite
Linux/Windows success. See https://support.apple.com/en-sg/103769. The repaired bundle uses
365-day leaves, serverAuth EKU, SANs, P-256/SHA-256 and SKI/AKI. This fixes a confirmed fixture
incompatibility; successful macOS TLS/negative fixtures remain the acceptance gate, not an
inferred pass or proof that lifetime was the sole cause of all original failures.

Offline maintenance: `python3 tests/fixtures/search-tls/generate.py --check` verifies committed
signatures, CA constraints, key correspondence, EKU/SAN/date/lifetime. Without arguments it
emits a JSON mapping of all four PEMs for one coherent regeneration; apply together. It only
loads the public leaf key and keeps a fresh CA signing key in memory; neither writes nor emits
the CA key. Update fixed leaf dates and tests/docs together when renewing. cryptography50.0.2
is an offline maintenance prerequisite only; ordinary fixture execution still uses stdlib.

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
