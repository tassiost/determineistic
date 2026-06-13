# Implementation Guide

## Overview

This is a Phase 0 implementation of the Quantum Drift deterministic world simulator. The system produces identical WorldRoot hashes across machines for identical inputs.

## Architecture

```
drift-cli
   ↓
drift-runtime-cpu
   ↓
drift-universe-margolus
   ↓
drift-codec
   ↓
drift-protocol (truth layer)
```

## Crates

### drift-protocol
Core types and Phase 0 constants:
- `WorldGenesis`, `UniverseDefinition`, `Event`, `TickOutput`
- `WorldState`, `Chunk` with bit-packed cell representation
- Constants: `EVENT_PAYLOAD_SIZE = 32`, `REGION_SIZE = 16`, `WORLD_SIZE = 64`
- Grid constants: `CHUNK_WIDTH = 32`, `CHUNK_HEIGHT = 32`, `CELL_BITS = 4`

### drift-codec
Binary encoding/decoding:
- Little-endian, fixed-width fields
- Custom `Encode`/`Decode` traits
- No serde, no variable-length encoding

### drift-universe-margolus
Margolus CA implementation:
- 2x2 block encoding (16-bit pattern)
- Clockwise rotation rule table
- Double-buffered state
- Phase A (even,even) then Phase B (odd,odd) execution

### drift-runtime-cpu
Reference CPU runtime:
- Full tick loop execution
- Chunk hashing (RowMajor order)
- Region aggregation (16×16 chunks)
- WorldRoot computation (BLAKE3)

### drift-cli
Command-line interface:
- `drift replay --genesis <file> --events <file> --ticks <n>`
- Binary file loading
- WorldRoot output

### drift-conformance
Conformance testing:
- Test vector structure
- Expected WorldRoot validation

## Building

```bash
# Build all crates
cargo build

# Build CLI binary
cargo build --bin drift

# Build fixture generator
cargo build --bin create_fixtures
```

## Running

### Create test fixtures
```bash
cargo run --bin create_fixtures
```
Creates `fixtures/genesis.bin` (20 bytes) and `fixtures/events.bin` (42 bytes).

### Run simulation
```bash
cargo run --bin drift -- replay --genesis fixtures/genesis.bin --events fixtures/events.bin --ticks 10
```
Output:
```
tick: 9
world_root: ac66bb2036beb69bb281f851ba1eb39f905eee6106f22ae0f28eae1c30d9790d
```

## Testing

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p drift-runtime-cpu
cargo test -p drift-conformance
cargo test -p drift-universe-margolus
```

## Determinism Verification

The system produces identical WorldRoot hashes across multiple runs:

```bash
# Run twice and compare
cargo run --bin drift -- replay --genesis fixtures/genesis.bin --events fixtures/events.bin --ticks 10
cargo run --bin drift -- replay --genesis fixtures/genesis.bin --events fixtures/events.bin --ticks 10
```

Both runs should output: `ac66bb2036beb69bb281f851ba1eb39f905eee6106f22ae0f28eae1c30d9790d`

## Phase 0 Constraints

- No floating point
- No serde
- No async
- No hash maps in hot path
- Single-threaded execution
- Little-endian encoding only
- Fixed-width fields only
- RowMajor spatial ordering only

## CI/CD

GitHub Actions workflow (`.github/workflows/ci.yml`) validates:
- Unit tests across Linux, macOS, Windows
- Code formatting (rustfmt)
- Clippy linting
- Deterministic hash generation across platforms

## Next Steps

1. Add more complex Margolus rules
2. Implement event application logic
3. Add chunk initialization from genesis
4. Scale to larger world sizes
5. Add GPU runtime (Phase 1)
6. Add WASM port (Phase 1)
