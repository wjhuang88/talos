# talos-sandbox

Linux commands use Bubblewrap when available. The host root is read-only;
`/dev`, `/proc` and a private `/tmp` are established before the authorized
workspace is mounted writable. Commands explicitly start in that workspace,
including when it is below `/tmp`. Host temporary siblings remain hidden.
Additional read paths are mounted afterward and remain read-only even within
the workspace. A missing workspace fails rather than running in HOME.
Workspace and extra-read mount paths are resolved before launch; a symbolic-link workspace starts at its
canonical location. Roots equal to or containing `/tmp`,
`/dev` or `/proc` are rejected, including symbolic-link aliases, because they
would cover the protected mounts. Unresolvable extra-read paths also fail closed.
Network access remains disabled unless the caller allows it. This does not
change caller permission checks or unavailable-backend fallback policy.

Run native Linux isolation regressions with a working Bubblewrap installation:

```sh
cargo test -p talos-sandbox --features bwrap_installed --locked
```

This opt-in test feature requires successful namespace creation. An installed
but unusable Bubblewrap fails these tests; it is not counted as a passing skip.
macOS tests and cross-compilation do not establish Linux runtime behavior.
