# SOP: Local Development

## Purpose

Set up and maintain a local development environment for Talos.

## Prerequisites

- Rust toolchain: repository-pinned `rust-toolchain.toml`, edition 2024
- OS: macOS, Linux, or WSL2

## Setup

### 1. Install Rust Toolchain

```bash
rustup default stable
rustup update
```

### 2. Clone and Build

```bash
git clone <repo-url> talos
cd talos
cargo build --workspace
```

### 3. Verify

```bash
cargo check --locked --workspace
cargo test --locked --workspace
cargo clippy --locked --workspace -- -D warnings
```

All three must exit 0 before starting work.

## Development Commands

| Command | Purpose |
| --- | --- |
| `cargo build --workspace` | Build all crates |
| `cargo check --locked --workspace` | Fast type-check all crates with the committed lockfile |
| `cargo test --locked --workspace` | Run all tests with the committed lockfile |
| `cargo clippy --locked --workspace -- -D warnings` | Lint shipped workspace targets with CI's warning policy |
| `./scripts/release_preflight.sh vX.Y.Z` | Validate a release candidate using the exact CI flow |
| `cargo doc --workspace --no-deps` | Generate documentation |
| `cargo run -p talos-cli` | Run the CLI |
| `cargo test -p talos-{crate}` | Test a specific crate |
| `cargo test -p talos-{crate} test_name` | Run a specific test |

## Project Structure

```
talos/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── talos-core/         # Core types and traits
│   ├── talos-agent/        # Agent loop
│   ├── talos-tools/        # Tool registry and built-ins
│   ├── talos-sandbox/      # Platform sandboxes
│   ├── talos-permission/   # Permission engine
│   ├── talos-provider/     # LLM providers
│   ├── talos-session/      # Session storage
│   ├── talos-skill/        # Skill system
│   ├── talos-plugin/       # Plugin runtime
│   ├── talos-mcp/          # MCP integration
│   ├── talos-config/       # Configuration
│   ├── talos-cli/          # CLI entry point
│   └── talos-rpc/          # JSON-RPC server
├── docs/                   # Governance and reference
├── tests/                  # Integration tests
└── AGENTS.md               # Agent coding guide
```

## Useful Tools

- `cargo-expand` — Inspect macro expansions
- `cargo-tree` — Visualize dependency tree
- `cargo-outdated` — Check for outdated dependencies
- `cargo-audit` — Security vulnerability check
- `rust-analyzer` — IDE support (VS Code, Neovim)
