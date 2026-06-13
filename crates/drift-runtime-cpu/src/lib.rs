use blake3::{Hash, Hasher};
use drift_protocol::{
    Event, SpatialSchedule, TickOutput, WorldGenesis, WorldState, REGION_SIZE, WORLD_SIZE,
};
use drift_universe_margolus::{simulate_chunk, RuleTable};

/// Event log - stores events indexed by tick
pub struct EventLog {
    events: Vec<Vec<Event>>,
}

impl Default for EventLog {
    fn default() -> Self {
        Self::new()
    }
}

impl EventLog {
    pub fn new() -> Self {
        EventLog { events: Vec::new() }
    }

    pub fn add_event(&mut self, event: Event) {
        let tick = event.tick as usize;
        while self.events.len() <= tick {
            self.events.push(Vec::new());
        }
        self.events[tick].push(event);
    }

    pub fn get(&self, tick: u64) -> &[Event] {
        self.events
            .get(tick as usize)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

/// Apply events to world state
fn apply_events(_world_state: &mut WorldState, _events: &[Event]) {
    // Placeholder - Phase 0 will implement actual event application
}

/// Compute chunk hashes in deterministic spatial order
fn compute_chunk_hashes(world_state: &WorldState, _schedule: SpatialSchedule) -> Vec<Hash> {
    world_state
        .chunks
        .iter()
        .map(|chunk| {
            let mut hasher = Hasher::new();
            hasher.update(&chunk.data);
            hasher.finalize()
        })
        .collect()
}

/// Aggregate region hashes from chunk hashes
/// Region = 16x16 chunks (256 chunks per region)
/// World = 64x64 chunks (16 regions total)
fn compute_region_hashes(chunk_hashes: &[Hash]) -> Vec<Hash> {
    let chunks_per_region = REGION_SIZE * REGION_SIZE; // 256
    let num_regions = (WORLD_SIZE / REGION_SIZE) * (WORLD_SIZE / REGION_SIZE); // 16

    let mut region_hashes = Vec::with_capacity(num_regions);

    for region_idx in 0..num_regions {
        let start = region_idx * chunks_per_region;
        let end = (start + chunks_per_region).min(chunk_hashes.len());

        let mut hasher = Hasher::new();
        for chunk_hash in &chunk_hashes[start..end] {
            hasher.update(chunk_hash.as_bytes());
        }
        region_hashes.push(hasher.finalize());
    }

    region_hashes
}

/// Compute WorldRoot from region hashes
fn compute_world_root(region_hashes: &[Hash]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    for hash in region_hashes {
        hasher.update(hash.as_bytes());
    }
    hasher.finalize().into()
}

/// Run a single tick
pub fn run_tick(
    world_state: &WorldState,
    events: &[Event],
    universe_definition: &drift_protocol::UniverseDefinition,
) -> (WorldState, TickOutput) {
    // 1. Apply events (placeholder - no mutation yet)
    let mut world_state = world_state.clone();
    apply_events(&mut world_state, events);

    // 2. Execute simulation kernel with double buffering
    let rule_table = RuleTable::default();
    let mut next_chunks = Vec::with_capacity(world_state.chunks.len());

    for chunk in &world_state.chunks {
        let next_chunk = simulate_chunk(chunk, &rule_table, universe_definition.spatial_schedule);
        next_chunks.push(next_chunk);
    }

    let next_state = WorldState {
        chunks: next_chunks,
    };

    // 3. Compute chunk hashes
    let chunk_hashes = compute_chunk_hashes(&next_state, SpatialSchedule::RowMajor);

    // 4. Aggregate region hashes
    let region_hashes = compute_region_hashes(&chunk_hashes);

    // 5. Compute WorldRoot
    let world_root = compute_world_root(&region_hashes);

    // 6. Emit tick output
    let tick_output = TickOutput {
        tick: 0, // Will be set by caller
        world_root,
    };

    (next_state, tick_output)
}

/// Run full simulation from genesis
pub fn run_simulation(
    genesis: &WorldGenesis,
    event_log: &EventLog,
    max_ticks: u64,
) -> Vec<TickOutput> {
    // Initialize world state with proper chunk grid
    let total_chunks = WORLD_SIZE * WORLD_SIZE; // 64x64 = 4096 chunks
    let mut world_state = WorldState::with_capacity(total_chunks);

    for _ in 0..total_chunks {
        world_state.chunks.push(drift_protocol::Chunk::new());
    }

    let mut outputs = Vec::new();

    for tick in 0..max_ticks {
        let events = event_log.get(tick);
        let (next_state, mut output) = run_tick(&world_state, events, &genesis.universe_definition);
        output.tick = tick;
        outputs.push(output);
        world_state = next_state;
    }

    outputs
}

#[cfg(test)]
mod tests {
    use super::*;
    use drift_protocol::{
        ArithmeticContract, DivisionMode, OverflowMode, PrecisionMode, SpatialSchedule,
    };

    #[test]
    fn test_deterministic_simulation() {
        let genesis = WorldGenesis {
            protocol_version: 0,
            universe_definition: drift_protocol::UniverseDefinition {
                protocol_version: 0,
                universe_type_id: 1,
                ruleset_id: 0,
                arithmetic_contract: ArithmeticContract {
                    overflow: OverflowMode::Wrap,
                    division: DivisionMode::TruncateTowardZero,
                    precision: PrecisionMode::IntegerOnly,
                },
                spatial_schedule: SpatialSchedule::RowMajor,
            },
        };

        let event_log = EventLog::new();
        let outputs1 = run_simulation(&genesis, &event_log, 10);
        let outputs2 = run_simulation(&genesis, &event_log, 10);

        // Verify deterministic output
        assert_eq!(outputs1.len(), outputs2.len());
        for (o1, o2) in outputs1.iter().zip(outputs2.iter()) {
            assert_eq!(o1.world_root, o2.world_root);
        }
    }
}
