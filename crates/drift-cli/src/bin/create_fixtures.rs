use drift_codec::Encode;
use drift_protocol::{
    ArithmeticContract, DivisionMode, Event, OverflowMode, PrecisionMode, SpatialSchedule,
    UniverseDefinition, WorldGenesis,
};
use std::fs;

fn main() {
    // Create genesis
    let genesis = WorldGenesis {
        protocol_version: 0,
        universe_definition: UniverseDefinition {
            protocol_version: 0,
            universe_type_id: 1, // MARGOLUS_CA_V1
            ruleset_id: 0,
            arithmetic_contract: ArithmeticContract {
                overflow: OverflowMode::Wrap,
                division: DivisionMode::TruncateTowardZero,
                precision: PrecisionMode::IntegerOnly,
            },
            spatial_schedule: SpatialSchedule::RowMajor,
        },
    };

    // Encode genesis (20 bytes total)
    let mut genesis_buf = [0u8; 20];
    genesis.encode(&mut genesis_buf);
    fs::write("fixtures/genesis.bin", &genesis_buf[..20]).expect("Failed to write genesis.bin");

    // Create a single event
    let event = Event {
        tick: 0,
        event_type: 0,
        payload: [0u8; 32],
    };

    // Encode event (42 bytes)
    let mut event_buf = [0u8; 42];
    event.encode(&mut event_buf);
    fs::write("fixtures/events.bin", event_buf).expect("Failed to write events.bin");

    println!("Created fixtures/genesis.bin (17 bytes)");
    println!("Created fixtures/events.bin (42 bytes)");
}
