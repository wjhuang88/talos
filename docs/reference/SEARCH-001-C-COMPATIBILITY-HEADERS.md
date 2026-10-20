# SEARCH-001-C: Dormant Compatibility Header Snapshots

Private library prerequisite under I293/#712/ADR-090, based on main
2d7f4b36c17ae17f5f74d34b167987876db8add2 (#738). Fixed compatibility preparation now freezes
the existing upstream UA choice and Bing language inside the owned PreparedTransportRequest.
No official caller, default search activation, model schema, public API, new dependency,
permission or provider admission is introduced.

## Preparation And Ownership

prepare_search validates count1..20 first. Both fixed preparation paths then validate the raw
query, exact encoded request bounds, actual endpoint and manifest membership before selecting
headers. They call the existing pinned public rust_websearch::anti_detection::random_user_agent
helper once, retaining its &'static str. Preparation has no HTTP/DNS/authority/budget effects,
but now includes local randomness/entropy access; it is not a deterministic pure validator.

The selected UA must be nonblank, at most512 bytes and visible ASCII with spaces, then must pass
actual reqwest HeaderValue validation. Helper panic or invalid output returns only the existing
closed InvalidRequest code, with no reroll, fallback or raw diagnostic. catch_unwind cannot
suppress panic hooks or recover abort/OOM. The private supplier supports deterministic validation
and panic tests; exposed preparation/execute methods accept no callback, UA or header map.

CompatibilityHeaders has private fields, a constructor visible only in the compatibility subtree
and redacted Debug. Generic JSON/form constructors retain None. Only the fixed factories attach
this typed snapshot after successful request validation. The same owning non-Clone request must
survive future permission admission; a snapshot is not an approval seal, authority, fingerprint
or global single-use manifest. UA choice is not added to the query-free Session scope.

| Fixed protocol | User-Agent | Accept-Language |
|---|---|---|
| DDG Lite | One retained upstream static-pool choice | Absent |
| DDG HTML | One retained upstream static-pool choice | Absent |
| Instant Answer | One retained upstream static-pool choice | Absent |
| Bing CN | One retained upstream static-pool choice | zh-CN,zh;q=0.9,en;q=0.8 |

## Controlled Sends And Redirects

execute consumes the same payload and never selects again. Each hop applies the saved UA and
optional fixed Bing language after the existing endpoint/authority/DNS/address checks and before
the existing synchronous final authority fence. Scoped GET redirects retain the same public,
non-sensitive profile even across origins. Every target still requires its own manifest endpoint;
an unscoped redirect receives no connection or headers. Existing POST method/cross-origin body
restrictions remain authoritative. No credential/cookie/referer/custom-header facility is added.

Accept-Encoding remains identity, content type remains the existing fixed body format, and
request/body/response/hop/shared-deadline/proxy/TLS controls remain in the same transport path.
The512-byte UA check bounds that value before HeaderValue allocation; it is not a complete
serialized-header allocation bound. Generic transport requests acquire neither compatibility UA
nor language. Snapshot Debug and closed errors contain no query, UA or dependency diagnostic.

## Provenance

The existing dependency and lock remain unchanged. Fresh rust-websearch0.1.2 archive SHA256:
06b90d9a9ad9e7e347c4cd85c3fb37189256703db4fab1d6c6fb963ae1eed0ee.
VCS b30180a03609580e8b28dfb682284cb2ba249fba, path crates/websearch. All three archived files
match freshly retrieved immutable pinned source bytes exactly:

| Source under crates/websearch | SHA256 |
|---|---|
| [src/anti_detection/mod.rs](https://github.com/Rust-Framework/rust-agent-framework/blob/b30180a03609580e8b28dfb682284cb2ba249fba/crates/websearch/src/anti_detection/mod.rs) | 748abdeb85f683e196dda06e45079e9b01d251f6caeb8d3add6ca0e53966e398 |
| [src/anti_detection/user_agent.rs](https://github.com/Rust-Framework/rust-agent-framework/blob/b30180a03609580e8b28dfb682284cb2ba249fba/crates/websearch/src/anti_detection/user_agent.rs) | a3b3b4aae8b57b9216d6ccd4ac98b00cd5325fd0f91914d9f51fc232d4bb4045 |
| [src/bing.rs](https://github.com/Rust-Framework/rust-agent-framework/blob/b30180a03609580e8b28dfb682284cb2ba249fba/crates/websearch/src/bing.rs) | 73fe17e6156599bd0e0e109762f88bacf4b47d18024f5d7a3eb7e78dfb3334d0 |

The tools-package LICENSES/rust-websearch-MIT.txt contains the pinned root's complete MIT text
(Copyright2026 Rust-Framework Contributors), with one final LF added by Talos. The pinned notice
is1083 bytes without a final LF; the retained notice is1084 bytes. They are not byte-identical.
Pinned SHA256: be15d050a36faff5b4ee8ae6c893a3525297bee5dc8bc364dd59a4d191e211cc;
retained SHA256: c3016bd5c845b1065bf11a65dd8c7cdf14e11b56c36b2de0101b79cad7af7f13.
New code records derivation and Talos's snapshot/validation/error changes.
The helper and its12-entry static UA pool are
reused rather than maintaining a second production pool; values do not identify the real user's
device. This is code-license/source evidence, not D service-use/privacy or anti-bot admission.

## Evidence And Remaining Gates

Seven new synthetic tests cover value bounds/control characters/HeaderValue validation,
closed helper panic/injection without retry, redaction/protocol language, zero helper calls for
invalid count/query/encoding/scope and exactly one for valid preparation, generic header absence,
scoped same/cross-origin GET snapshot retention and an unscoped target's zero sends. Existing
four-format controlled execution tests also assert exact captured wire UA, only Bing language,
identity encoding and absent sensitive headers for count1/2. Existing transport guard tests stay
in scope. Test-only relocation preserves the snapshot and revalidates explicit loopback scope.
No official provider is contacted; no statistical assertion requires different random draws.

Upstream selects a UA whenever its client is built, including its own source/retry scheduling.
This stage freezes once per preparation and never introduces retries or source selection.
Reuse of the helper does not prove matching draw distribution, timing, live response charset,
probe/retry/source/proxy behavior or full protocol/service parity. Those migration differences,
D/privacy/result use, official roots, admission/router/health/cancellation/rollback/default
activation and release retain their gates. I293 remains Active/Claimed, Completion Pending;
#643/#624 stay open, and #705/#717/#735 retain independent ownership.
