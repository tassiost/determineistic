use clap::{Parser, Subcommand};
use drift_codec::Decode;
use drift_protocol::{Event, WorldGenesis, EVENT_PAYLOAD_SIZE};
use drift_runtime_cpu::{run_simulation, EventLog};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "drift")]
#[command(about = "Deterministic world simulator CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Replay simulation from genesis and events
    Replay {
        /// Path to genesis binary file
        #[arg(short, long)]
        genesis: PathBuf,

        /// Path to events binary file
        #[arg(short, long)]
        events: PathBuf,

        /// Output path for worldroot binary file
        #[arg(short, long)]
        out: Option<PathBuf>,

        /// Number of ticks to simulate
        #[arg(short, long, default_value_t = 100)]
        ticks: u64,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Replay {
            genesis,
            events,
            out,
            ticks,
        } => {
            // Load genesis from binary file
            let genesis = load_genesis(&genesis);

            // Load events from binary file
            let event_log = load_events(&events);

            // Run simulation
            let outputs = run_simulation(&genesis, &event_log, ticks);

            // Output final WorldRoot
            if let Some(final_output) = outputs.last() {
                println!("tick: {}", final_output.tick);
                println!("world_root: {}", hex::encode(final_output.world_root));
            }

            // Optionally write to file
            if let Some(out_path) = out {
                if let Some(final_output) = outputs.last() {
                    fs::write(&out_path, final_output.world_root)
                        .expect("Failed to write worldroot");
                    println!("Wrote worldroot to: {}", out_path.display());
                }
            }
        }
    }
}

fn load_genesis(path: &PathBuf) -> WorldGenesis {
    let bytes = fs::read(path).expect("Failed to read genesis file");
    WorldGenesis::decode(&bytes)
}

fn load_events(path: &PathBuf) -> EventLog {
    let bytes = fs::read(path).expect("Failed to read events file");
    let event_size = 8 + 2 + EVENT_PAYLOAD_SIZE; // 42 bytes per event
    let num_events = bytes.len() / event_size;

    let mut event_log = EventLog::new();
    for i in 0..num_events {
        let start = i * event_size;
        let event = Event::decode(&bytes[start..start + event_size]);
        event_log.add_event(event);
    }
    event_log
}
