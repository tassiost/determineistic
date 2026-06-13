// Phase 0 Constants
pub const EVENT_PAYLOAD_SIZE: usize = 32;
pub const REGION_SIZE: usize = 16;   // chunks per dimension
pub const WORLD_SIZE: usize = 64;    // chunks per dimension

// Grid constants for Margolus CA
pub const CHUNK_WIDTH: usize = 32;   // cells per chunk dimension
pub const CHUNK_HEIGHT: usize = 32;  // cells per chunk dimension
pub const CHUNK_CELLS: usize = CHUNK_WIDTH * CHUNK_HEIGHT;  // total cells per chunk
pub const CELL_BITS: usize = 4;      // 4 bits per cell (16 states)

// Core types

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldGenesis {
    pub protocol_version: u32,
    pub universe_definition: UniverseDefinition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniverseDefinition {
    pub protocol_version: u32,
    pub universe_type_id: u32,   // MARGOLUS_CA_V1
    pub ruleset_id: u32,
    pub arithmetic_contract: ArithmeticContract,
    pub spatial_schedule: SpatialSchedule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArithmeticContract {
    pub overflow: OverflowMode,
    pub division: DivisionMode,
    pub precision: PrecisionMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowMode {
    Wrap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DivisionMode {
    TruncateTowardZero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrecisionMode {
    IntegerOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialSchedule {
    RowMajor,  // Phase 0: ONLY RowMajor
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub tick: u64,
    pub event_type: u16,
    pub payload: [u8; EVENT_PAYLOAD_SIZE],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickOutput {
    pub tick: u64,
    pub world_root: [u8; 32],
}

// World state representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldState {
    pub chunks: Vec<Chunk>,
}

impl WorldState {
    pub fn new() -> Self {
        WorldState {
            chunks: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        WorldState {
            chunks: Vec::with_capacity(capacity),
        }
    }
}

// Chunk representation - bit-packed cell states
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    // Bit-packed cells: 4 bits per cell
    // For 32x32 chunk = 1024 cells = 4096 bits = 512 bytes
    pub data: Vec<u8>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            data: vec
![0u8; (CHUNK_CELLS * CELL_BITS) / 8],
        }
    }

    pub fn get_cell(&self, x: usize, y: usize) -> u8 {
        let index = y * CHUNK_WIDTH + x;
        let byte_offset = (index * CELL_BITS) / 8;
        let bit_offset = (index * CELL_BITS) % 8;
        
        let byte = self.data[byte_offset];
        let mask = (0xF as u8) << bit_offset;
        ((byte & mask) >> bit_offset) & 0xF
    }

    pub fn set_cell(&mut self, x: usize, y: usize, value: u8) {
        let index = y * CHUNK_WIDTH + x;
        let byte_offset = (index * CELL_BITS) / 8;
        let bit_offset = (index * CELL_BITS) % 8;
        
        let masked_value = (value & 0xF) << bit_offset;
        let mask = !((0xF as u8) << bit_offset);
        self.data[byte_offset] = (self.data[byte_offset] & mask) | masked_value;
    }
}
