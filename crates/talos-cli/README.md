# talos-cli

The command-line binary for [Talos](https://github.com/wjhuang88/talos), a Rust-native local
coding agent.

## Install

### From a release archive

```bash
curl -fsSL https://raw.githubusercontent.com/wjhuang88/talos/main/install/install.sh | sh
```

See the [root README](../../README.md) for Windows instructions and manual archive downloads.

### From source

```bash
cargo install --path crates/talos-cli --bin talos
talos --version
```

### Upgrading

```bash
cargo install --path crates/talos-cli --bin talos --force
```

### Uninstalling

```bash
cargo uninstall talos-cli
```

## Usage

```bash
talos "inspect this repository"       # interactive TUI
talos -p "summarize this repository"  # one-shot print mode
talos --help                          # full CLI reference
```

See the [root README](../../README.md) for configuration, built-in tools, slash commands, skills,
MCP integration, and the safety model.

### Unreleased manual context compaction

Enter `/compact` without arguments in the interactive TUI or inline mode. It
compacts the model's request context while preserving original session history.
An active turn queues the operation until a complete tool batch or turn completion.
Transient status reports queued/started, applied token estimates, below-threshold
skip, failure or cancellation; a measured failure/cancellation retains its prior
estimate. Checkpoint content is not printed as a conversation message.

Manual compaction is eligible at 80% of the resolved model context limit;
automatic request preparation uses 95%. These estimates include prompt and tools,
images, safety margin and configured output/reasoning reservations. A failure
retains prior context; a request that cannot fit after safe recovery is refused.
See [context behavior](../../docs/reference/ARCHITECTURE.md#context-compaction-pipeline)
for raw-history and privacy boundaries. This is part of the next-minor I312 work,
not a claim that an existing release provides this command.

## Support Boundary

This crate provides a **binary only**. The `talos-cli` library API is not a supported SDK surface
and may change without notice between pre-1.0 versions.

Rust projects that need to embed the agent runtime should depend on
[`talos-runtime`](../talos-runtime/) instead, which exposes the stable pre-1.0
`RuntimeBuilder` / `RuntimeHandle` facade.

## License

Apache-2.0. See [LICENSE](../../LICENSE).
