# I008: "Learning Agent"

**User can**: Agent adapts its behavior across sessions via built-in evolution with cognitive feedback (ADR-001).

## Status: COMPLETE ✅

## Completed Tasks

### S1: Evolution crate structure + data models ✅
- Created `crates/talos-evolution` with Cargo.toml
- Defined core types: `Observation`, `Pattern`, `Conflict`, `EvolutionConfig`
- SQLite schema for observations, patterns, conflicts tables
- Basic CRUD operations with rusqlite

### S2: TurnObserver - Signal capture ✅
- Implemented signal types: Error, Correction, Satisfaction, Inefficiency
- Intensity scoring (0.0 - 1.0)
- TurnObserver struct to capture signals during agent execution

### S3: PatternExtractor - Rule-based extraction ✅
- Extract patterns from observations based on signal types
- Contradiction detection between new and existing patterns
- Confidence scoring with evidence counting

### S4: KnowledgeStore - SQLite persistence ✅
- Extended SQLite schema with evolution tables
- CRUD operations for observations and patterns
- Query patterns by confidence, type, recency

### S5: BehaviorAdapter - System prompt injection ✅
- Read high-confidence patterns (confidence > 0.7)
- Format patterns as natural language instructions
- Inject into system prompt assembly

### S6: TUI evolution insights panel ✅
- New panel in TUI showing learned patterns
- Display confidence scores and evidence counts
- Toggle visibility with Ctrl+E

### S7: `/learned` command ✅
- CLI command to display evolution insights
- Show top patterns by confidence
- Show recent observations

## Verification

```bash
# Build and test
cargo build --release -p talos-cli
cargo test --workspace

# Test evolution engine
cargo test -p talos-evolution

# Test --learned command
./target/release/talos --learned

# Test TUI with evolution panel
./target/release/talos --mock --tui
# Press Ctrl+E to toggle evolution panel
```
