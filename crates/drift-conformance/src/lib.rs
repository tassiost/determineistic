use drift_protocol::WorldGenesis;
use drift_runtime_cpu::{run_simulation, EventLog};

/// Conformance test vector
pub struct TestVector {
    pub name: String,
    pub genesis: WorldGenesis,
    pub events: EventLog,
    pub expected_world_root: [u8; 32],
    pub max_ticks: u64,
}

/// Run conformance test
pub fn run_conformance_test(vector: &TestVector) -> bool {
    let outputs = run_simulation(&vector.genesis, &vector.events, vector.max_ticks);

    if let Some(final_output) = outputs.last() {
        final_output.world_root == vector.expected_world_root
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use drift_protocol::{
        ArithmeticContract, DivisionMode, Event, OverflowMode, PrecisionMode, SpatialSchedule,
        WorldGenesis,
    };
    use drift_runtime_cpu::EventLog;

    #[test]
    fn test_vector_01_basic() {
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

        let mut event_log = EventLog::new();
        event_log.add_event(Event {
            tick: 0,
            event_type: 0,
            payload: [0u8; 32],
        });

        // Expected WorldRoot from reference implementation
        let expected_hash: [u8; 32] = [
            0xac, 0x66, 0xbb, 0x20, 0x36, 0xbe, 0xb6, 0x9b, 0xb2, 0x81, 0xf8, 0x51, 0xba, 0x1e,
            0xb3, 0x9f, 0x90, 0x5e, 0xee, 0x61, 0x06, 0xf2, 0x2a, 0xe0, 0xf2, 0x8e, 0xae, 0x1c,
            0x30, 0xd9, 0x79, 0x0d,
        ];

        let vector = TestVector {
            name: "basic_01".to_string(),
            genesis,
            events: event_log,
            expected_world_root: expected_hash,
            max_ticks: 10,
        };

        assert!(
            run_conformance_test(&vector),
            "Conformance test failed: WorldRoot mismatch"
        );
    }

    #[test]
    fn test_vector_02_scaling_sparse() {
        // Phase 1: Chunk scaling stress test - sparse event distribution
        // Tests determinism under scale with minimal event load
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

        let mut event_log = EventLog::new();
        // Sparse events: only a few events spread across many ticks
        event_log.add_event(Event {
            tick: 0,
            event_type: 0,
            payload: [0u8; 32],
        });
        event_log.add_event(Event {
            tick: 5000,
            event_type: 0,
            payload: [1u8; 32],
        });
        event_log.add_event(Event {
            tick: 10000,
            event_type: 0,
            payload: [2u8; 32],
        });

        // Run to get the golden hash for this configuration
        let outputs = run_simulation(&genesis, &event_log, 10000);
        let golden_hash = outputs.last().unwrap().world_root;

        // Verify determinism by running multiple times
        for _ in 0..3 {
            let test_outputs = run_simulation(&genesis, &event_log, 10000);
            let test_hash = test_outputs.last().unwrap().world_root;
            assert_eq!(
                test_hash, golden_hash,
                "Determinism failed in scaling sparse test"
            );
        }
    }

    #[test]
    fn test_vector_03_scaling_dense() {
        // Phase 1: Chunk scaling stress test - dense event distribution
        // Tests determinism under scale with high event load
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

        let mut event_log = EventLog::new();
        // Dense events: many events across the simulation
        for i in 0..100 {
            event_log.add_event(Event {
                tick: i * 100,
                event_type: 0,
                payload: [(i % 256) as u8; 32],
            });
        }

        // Run to get the golden hash for this configuration
        let outputs = run_simulation(&genesis, &event_log, 10000);
        let golden_hash = outputs.last().unwrap().world_root;

        // Verify determinism by running multiple times
        for _ in 0..3 {
            let test_outputs = run_simulation(&genesis, &event_log, 10000);
            let test_hash = test_outputs.last().unwrap().world_root;
            assert_eq!(
                test_hash, golden_hash,
                "Determinism failed in scaling dense test"
            );
        }
    }
}

#[cfg(test)]
mod heavy_scale_tests {
    use super::*;
    use drift_protocol::{
        ArithmeticContract, DivisionMode, Event, OverflowMode, PrecisionMode, SpatialSchedule,
        WorldGenesis,
    };
    use drift_runtime_cpu::EventLog;

    #[test]
    #[ignore]
    fn test_heavy_scale_100k_ticks() {
        // Heavy scale test: 100k ticks for local/dev validation
        // Run with: cargo test -- --ignored test_heavy_scale_100k_ticks
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

        let mut event_log = EventLog::new();
        // Sparse events spread across 100k ticks
        event_log.add_event(Event {
            tick: 0,
            event_type: 0,
            payload: [0u8; 32],
        });
        event_log.add_event(Event {
            tick: 50000,
            event_type: 0,
            payload: [1u8; 32],
        });
        event_log.add_event(Event {
            tick: 100000,
            event_type: 0,
            payload: [2u8; 32],
        });

        let outputs = run_simulation(&genesis, &event_log, 100000);
        let golden_hash = outputs.last().unwrap().world_root;

        // Verify determinism with fewer reps for heavy test
        for _ in 0..2 {
            let test_outputs = run_simulation(&genesis, &event_log, 100000);
            let test_hash = test_outputs.last().unwrap().world_root;
            assert_eq!(
                test_hash, golden_hash,
                "Determinism failed in heavy scale 100k test"
            );
        }
    }

    #[test]
    #[ignore]
    fn test_heavy_scale_1m_ticks() {
        // Heavy scale test: 1M ticks for local/dev validation
        // Run with: cargo test -- --ignored test_heavy_scale_1m_ticks
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

        let mut event_log = EventLog::new();
        // Sparse events spread across 1M ticks
        event_log.add_event(Event {
            tick: 0,
            event_type: 0,
            payload: [0u8; 32],
        });
        event_log.add_event(Event {
            tick: 500000,
            event_type: 0,
            payload: [1u8; 32],
        });
        event_log.add_event(Event {
            tick: 1000000,
            event_type: 0,
            payload: [2u8; 32],
        });

        let outputs = run_simulation(&genesis, &event_log, 1_000_000);
        let golden_hash = outputs.last().unwrap().world_root;

        // Single verification for very heavy test
        let test_outputs = run_simulation(&genesis, &event_log, 1_000_000);
        let test_hash = test_outputs.last().unwrap().world_root;
        assert_eq!(
            test_hash, golden_hash,
            "Determinism failed in heavy scale 1M test"
        );
    }
}
