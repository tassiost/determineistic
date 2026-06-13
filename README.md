# 🌍 Quantum Drift - Phase 0 Implementation Plan (drift-protocol v0)

## Overview

Build a minimal deterministic world simulator that produces identical WorldRoot hashes across machines, runs a Margolus 2x2 Cellular Automaton universe, uses a fully specified byte-level protocol, and has a conformance suite that validates determinism across implementations.

**This is not a game engine yet. It is a deterministic execution standard prototype.**

## Core Invariant

Given identical WorldGenesis + EventLog, all runtimes must produce identical WorldRoot.

```
simulate(WorldState[t], Events[t]) -> WorldState[t+1]
```

Must be:
- **deterministic**
- **bitwise reproducible**
- **architecture independent**

## Crate Structure

### 2.1 drift-protocol (CORE SPEC)

Defines the canonical types and rules.

**Must include:**
- WorldGenesis
- UniverseDefinition
- Event
- Tick
- WorldRoot
- ChunkHash

#### UniverseDefinition
```rust
struct UniverseDefinition {
    protocol_version: u32,
    universe_type_id: u32,   // MARGOLUS_CA_V1
    ruleset_id: u32,
    arithmetic_contract: ArithmeticContract,
    spatial_schedule: SpatialSchedule,
}
```

#### ArithmeticContract
```rust
struct ArithmeticContract {
    overflow: OverflowMode,   // Wrap
    division: DivisionMode,   // TruncateTowardZero
    precision: PrecisionMode, // Fixed or Integer-only
}
```

#### SpatialSchedule
```rust
// Phase 0: RowMajor ONLY
enum SpatialSchedule {
    RowMajor,
}
```

### 2.2 drift-codec

Custom binary encoding layer.

**Requirements:**
- NO serde
- NO repr(C) assumptions
- NO rkyv/zerocopy as source of truth

**Must define:**
```rust
trait Encode {
    fn encode(&self, buf: &mut [u8]);
}

trait Decode: Sized {
    fn decode(buf: &[u8]) -> Self;
}
```

**Rules:**
- Little-endian only
- Fixed-width fields only
- No variable-length integers
- No implicit padding

### 2.3 drift-universe-margolus

Reference simulation kernel.

**Implements:**
- 2x2 Margolus CA
- Bit-packed grid state
- Deterministic block update rules

**Must:**
- NOT use floating point
- NOT use randomness
- NOT rely on iteration order unless defined by SpatialSchedule

### 2.4 drift-runtime-cpu (REFERENCE IMPLEMENTATION)

- Single-threaded preferred (Phase 0 simplicity)
- Executes full tick pipeline
- Produces canonical WorldRoot

### 2.5 drift-cli

Command-line verification tool.

**Command:**
```bash
drift replay --genesis genesis.bin --events events.bin --out worldroot.bin
```

**Output:**
- final WorldRoot hash
- optional debug logs per tick

### 2.6 drift-conformance

Test suite for determinism.

**Must include:**
- precomputed fixtures:
  - genesis.bin
  - events.bin
  - expected_worldroot.bin

**Test requirement:**
Any valid runtime must produce identical output byte-for-byte

**Run across:**
- Linux x86_64
- macOS ARM64
- Windows x86_64 (optional but ideal)

## Margolus Universe Specification

### Grid
- 2x2 block decomposition
- alternating phase offsets

### Rules
- bitwise lookup table or explicit transition function
- no branching randomness
- deterministic ordering defined by SpatialSchedule

### Rule Table Representation (Phase 0)

**Critical constraint:** The Margolus transition function is a fully enumerated 2×2 → 2×2 lookup table stored in canonical byte order.

**Requirements:**
- Input block encoding: 16-bit fixed pattern (4 cells × 4 bits)
- Output block encoding: same 16-bit format
- Lookup table: sorted by input bit-pattern ascending
- Fixed array storage (NOT hash map, NOT computed dynamically)
- No undefined states (all 2^16 input patterns must have explicit output)

**Forbidden:**
- Hash map-based rule lookup
- Computed/dynamic rule generation
- Compiler-dependent table reordering

### Execution Order (Phase 0)

For each tick:
1. **Phase A**: process blocks at (x even, y even)
2. **Phase B**: process blocks at (x odd, y odd)

Strict order. No fusion. Both phases complete before tick advances.

### State Buffering (Phase 0)

**Critical constraint:** Strict double-buffering required.

**Requirements:**
- Read and write state buffers must be separate
- No in-place mutation during CA step
- Write visibility timing must not affect results
- Buffer swap occurs only after complete phase execution

**Forbidden:**
- In-place updates
- Shared read/write state during phase execution
- Dependency on write ordering within a phase

## Tick System

Each tick is atomic:

```
Tick N:
  1. Load Events[N]
  2. Apply Universe Rules
  3. Update WorldState
  4. Compute ChunkHash
  5. Aggregate into WorldRoot
  6. Freeze state
```

No mid-tick mutation allowed.

## Native Runtime Execution Loop (Phase 0)

This is the canonical tick execution contract for `drift-runtime-cpu`.

### Core Loop

```rust
for tick in 0..MAX_TICKS {
    // 1. Load events for this tick (pre-sorted, deterministic order)
    let events = event_log.get(tick);

    // 2. Apply events to world state (no mid-tick mutation visibility elsewhere)
    apply_events(&mut world_state, &events);

    // 3. Execute simulation kernel (Margolus CA)
    // Double-buffered to prevent in-place drift
    let next_state = simulate_margolus(&world_state, &universe_definition);

    // 4. Swap buffers
    world_state = next_state;

    // 5. Compute chunk hashes (deterministic spatial order)
    let chunk_hashes = compute_chunk_hashes(&world_state, SpatialSchedule::RowMajor);

    // 6. Aggregate region hashes
    let region_hashes = compute_region_hashes(&chunk_hashes);

    // 7. Compute WorldRoot (final canonical state hash for tick)
    let world_root = blake3_concat(&region_hashes);

    // 8. Emit checkpoint (optional debug + conformance anchor)
    emit_checkpoint(tick, world_root, &world_state);
}
```

### Critical Invariants

1. **Event ordering is pre-resolved**
   - `events[tick]` is already sorted deterministically
   - No sorting during runtime

2. **No in-place mutation during CA step**
   - `read(world_state) → write(next_state)`
   - Never mix

3. **Hash order is fixed**
   - RowMajor chunk iteration only
   - Region aggregation order fixed
   - No parallel reduction ambiguity

4. **Tick boundary is atomic**
   - Once `world_root` is computed, state is frozen
   - Cannot be reused for earlier ticks

### Function Decomposition

```rust
run_tick()
├── load_events()
├── apply_events()
├── simulate_margolus()
├── swap_buffers()
├── compute_chunk_hashes()
├── compute_region_hashes()
└── compute_world_root()
```

No hidden side effects between stages.

### Margolus Step Detail

Inside `simulate_margolus`:

```rust
for phase in [A, B] {
    for each 2x2 block in SpatialSchedule {
        apply_lookup_rule(block);
    }
}
```

- Phase A and B are strictly sequential
- Block iteration order is deterministic (RowMajor grid scan)

### Output Contract Per Tick

```rust
struct TickOutput {
    tick: u64,
    world_root: [u8; 32],
    debug_snapshot: Option<DebugSnapshot>,
}
```

Only `world_root` is required for conformance.

## Event System

```rust
struct Event {
    tick: u64,
    event_type: u16,
    payload: [u8; 32],  // FIXED_SIZE = 32 bytes
}
```

**Rules:**
- must be assigned to a tick before execution
- cannot be inserted mid-tick
- must serialize deterministically

## Hashing System

**Use:** BLAKE3 in default, unkeyed, non-domain-separated configuration.

**Critical constraints:**
- Single-instance, default parameters only
- No keyed hashing
- No incremental state reuse across ticks
- Inputs are always fully materialized byte arrays per hash operation
- No domain separation tags

**Hierarchy:**
```
ChunkHash → RegionHash → WorldRoot
```

**Explicit Structure:**
- **Chunk**: smallest simulation unit (Margolus grid block group)
- **Region**: 16 × 16 chunks (256 chunks total)
- **World**: 64 × 64 chunks (4096 chunks total)

**Reduction Rules:**
```
RegionHash = Hash(concat(sorted ChunkHash[256]))
WorldRoot = Hash(concat(sorted RegionHash[16]))
```

**Phase 0 Constants:**
```rust
pub const EVENT_PAYLOAD_SIZE: usize = 32;
pub const REGION_SIZE: usize = 16;   // chunks per dimension
pub const WORLD_SIZE: usize = 64;    // chunks per dimension
```

## Determinism Rules (STRICT)

The following are **forbidden**:
- floating point math
- hash map iteration order dependence
- pointer-based identity assumptions
- platform-dependent serialization
- implicit integer widening
- non-deterministic parallelism (Phase 0 = single-thread recommended)

## Phase 0 Milestones

### Milestone 1 — Minimal Kernel
- Margolus CA runs for 100 ticks
- deterministic single machine output

### Milestone 2 — CLI Replay
- genesis + event log → WorldRoot

### Milestone 3 — Conformance Test
- 2 machines produce identical hash

### Milestone 4 — Cross-platform verification
- x86 + ARM identical output

### Milestone 5 — 10k chunks / long tick run
- stability test

## Success Criteria

Phase 0 is complete when:

```
WorldRoot(machine A) == WorldRoot(machine B)
```

for identical inputs, across at least 2 architectures.

## Non-goals (explicit)

- no GPU acceleration
- no networking
- no multi-universe system
- no agent simulation
- no physics realism beyond Margolus CA
- no performance optimization focus

## Key Philosophy

- The protocol defines the universe.
- The runtime only executes it.
- The codec guarantees identical bytes.
- The conformance suite proves correctness.

## Recommended Stack

### Core Language
**Rust (stable)**
- explicit memory control
- deterministic builds
- strong typing for protocol enforcement
- Avoid nightly unless absolutely necessary

### Workspace Layout

```
drift/
  crates/
    drift-protocol
    drift-codec
    drift-universe-margolus
    drift-runtime-cpu
    drift-cli
    drift-conformance
```

### Determinism Core Choices

❌ **Avoid entirely:**
- serde (too flexible, non-canonical unless heavily constrained)
- f32 / f64 (no floats in Phase 0)
- rand
- async runtimes (tokio, async-std)
- rayon (parallelism breaks determinism unless heavily controlled)
- hashmaps in hot path (iteration order risk)

### Math / Numeric Strategy

**Use:**
- i64, u64, u32 only
- explicit fixed-point where needed:
  - Fixed(i64) with compile-time scale constant
- explicit wrapping rules (wrapping_add, etc.)

### Hashing

**Recommended:** BLAKE3
- fast
- stable
- widely available
- deterministic across platforms

**Config:**
- fixed input formatting only
- no streaming ambiguity in protocol layer

### Binary Codec Layer

**Do NOT use:**
- serde
- bincode (still abstracted)
- rkyv (good but introduces lifetime/representation coupling risk for Phase 0)

**DO:**
- custom byte codec (drift-codec)
- manual encoding rules
- fixed-width fields only
- explicit little-endian encoding

### Testing / Conformance

**Rust native:**
- cargo test (unit tests only)
- golden file tests (critical)

**Optional but strong:**
- insta (snapshot testing) → useful for debugging, but must snapshot bytes only

### CLI Tooling

**clap**
- only for argument parsing in drift-cli
- safe, stable, widely used

### Build / Tooling

**Formatting:**
- rustfmt (fixed config, no custom style drift)

**Linting:**
- clippy
- treat warnings as errors

**CI:**
- GitHub Actions
- matrix build:
  - linux x86_64
  - macOS arm64
  - windows x86_64 (optional but ideal)

### Determinism Validation Tooling

Add a CI step:
- run drift-cli
- compare worldroot.bin
- assert byte-equality

No fuzzy matching allowed.

### Suggested Crate Dependencies

**drift-protocol**
- no_std preferred
- bytemuck (optional, ONLY for safe casting—not serialization)
- static_assertions

**drift-codec**
- byteorder (optional, but even manual encoding is fine)
- or pure manual encoding (preferred)

**drift-cli**
- clap
- blake3
- hex (debug output only)

**drift-conformance**
- serde ONLY for test vector loading (optional exception)
- assert_eq! + byte comparisons
- insta (optional snapshot layer)

## Hard Constraints (Important)

This is your real "stack law":

- no hidden serialization magic
- no runtime reflection
- no dependency that controls memory layout implicitly
- no parallelism in Phase 0 runtime
- no floats anywhere in simulation path
- no platform-dependent behavior in core crates

## Mental Model of the Stack

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

And separately:

```
drift-conformance = validator of everything above
```

## Minimum Correct Stack

If you ignore everything else, this is the "minimum correct stack":

- Rust stable
- no async
- no floats
- no serde
- blake3
- clap
- custom binary codec
- single-threaded CPU reference runtime
- GitHub Actions CI matrix
