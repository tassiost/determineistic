use drift_protocol::{
    ArithmeticContract, DivisionMode, Event, OverflowMode, PrecisionMode, SpatialSchedule,
    UniverseDefinition, WorldGenesis,
};

/// Custom binary encoding trait for deterministic serialization
///
/// Rules:
/// - Little-endian only
/// - Fixed-width fields only
/// - No variable-length integers
/// - No implicit padding
pub trait Encode {
    fn encode(&self, buf: &mut [u8]);
}

/// Custom binary decoding trait for deterministic deserialization
///
/// Rules:
/// - Little-endian only
/// - Fixed-width fields only
/// - No variable-length integers
/// - No implicit padding
pub trait Decode: Sized {
    fn decode(buf: &[u8]) -> Self;
}

// Helper functions for little-endian encoding

pub fn encode_u64(val: u64, buf: &mut [u8]) {
    buf[0] = (val >> 0) as u8;
    buf[1] = (val >> 8) as u8;
    buf[2] = (val >> 16) as u8;
    buf[3] = (val >> 24) as u8;
    buf[4] = (val >> 32) as u8;
    buf[5] = (val >> 40) as u8;
    buf[6] = (val >> 48) as u8;
    buf[7] = (val >> 56) as u8;
}

pub fn decode_u64(buf: &[u8]) -> u64 {
    (buf[0] as u64) << 0
        | (buf[1] as u64) << 8
        | (buf[2] as u64) << 16
        | (buf[3] as u64) << 24
        | (buf[4] as u64) << 32
        | (buf[5] as u64) << 40
        | (buf[6] as u64) << 48
        | (buf[7] as u64) << 56
}

pub fn encode_u32(val: u32, buf: &mut [u8]) {
    buf[0] = val as u8;
    buf[1] = (val >> 8) as u8;
    buf[2] = (val >> 16) as u8;
    buf[3] = (val >> 24) as u8;
}

pub fn decode_u32(buf: &[u8]) -> u32 {
    (buf[0] as u32) | (buf[1] as u32) << 8 | (buf[2] as u32) << 16 | (buf[3] as u32) << 24
}

pub fn encode_u16(val: u16, buf: &mut [u8]) {
    buf[0] = val as u8;
    buf[1] = (val >> 8) as u8;
}

pub fn decode_u16(buf: &[u8]) -> u16 {
    (buf[0] as u16) | (buf[1] as u16) << 8
}

// Event encoding (fixed size: 8 + 2 + 32 = 42 bytes)
impl Encode for Event {
    fn encode(&self, buf: &mut [u8]) {
        encode_u64(self.tick, &mut buf[0..8]);
        encode_u16(self.event_type, &mut buf[8..10]);
        buf[10..10 + drift_protocol::EVENT_PAYLOAD_SIZE].copy_from_slice(&self.payload);
    }
}

impl Decode for Event {
    fn decode(buf: &[u8]) -> Self {
        Event {
            tick: decode_u64(&buf[0..8]),
            event_type: decode_u16(&buf[8..10]),
            payload: {
                let mut payload = [0u8; drift_protocol::EVENT_PAYLOAD_SIZE];
                payload.copy_from_slice(&buf[10..10 + drift_protocol::EVENT_PAYLOAD_SIZE]);
                payload
            },
        }
    }
}

// WorldGenesis encoding (fixed size: 4 + UniverseDefinition = 20 bytes)
impl Encode for WorldGenesis {
    fn encode(&self, buf: &mut [u8]) {
        encode_u32(self.protocol_version, &mut buf[0..4]);
        self.universe_definition.encode(&mut buf[4..20]);
    }
}

impl Decode for WorldGenesis {
    fn decode(buf: &[u8]) -> Self {
        WorldGenesis {
            protocol_version: decode_u32(&buf[0..4]),
            universe_definition: UniverseDefinition::decode(&buf[4..]),
        }
    }
}

// UniverseDefinition encoding (fixed size: 4 + 4 + 4 + 3 + 1 = 16 bytes)
impl Encode for UniverseDefinition {
    fn encode(&self, buf: &mut [u8]) {
        encode_u32(self.protocol_version, &mut buf[0..4]);
        encode_u32(self.universe_type_id, &mut buf[4..8]);
        encode_u32(self.ruleset_id, &mut buf[8..12]);
        self.arithmetic_contract.encode(&mut buf[12..15]);
        self.spatial_schedule.encode(&mut buf[15..16]);
    }
}

impl Decode for UniverseDefinition {
    fn decode(buf: &[u8]) -> Self {
        UniverseDefinition {
            protocol_version: decode_u32(&buf[0..4]),
            universe_type_id: decode_u32(&buf[4..8]),
            ruleset_id: decode_u32(&buf[8..12]),
            arithmetic_contract: ArithmeticContract::decode(&buf[12..15]),
            spatial_schedule: SpatialSchedule::decode(&buf[15..16]),
        }
    }
}

// ArithmeticContract encoding (fixed size: 1 + 1 + 1 = 3 bytes)
impl Encode for ArithmeticContract {
    fn encode(&self, buf: &mut [u8]) {
        buf[0] = self.overflow as u8;
        buf[1] = self.division as u8;
        buf[2] = self.precision as u8;
    }
}

impl Decode for ArithmeticContract {
    fn decode(buf: &[u8]) -> Self {
        ArithmeticContract {
            overflow: match buf[0] {
                0 => OverflowMode::Wrap,
                _ => panic!("Invalid overflow mode"),
            },
            division: match buf[1] {
                0 => DivisionMode::TruncateTowardZero,
                _ => panic!("Invalid division mode"),
            },
            precision: match buf[2] {
                0 => PrecisionMode::IntegerOnly,
                _ => panic!("Invalid precision mode"),
            },
        }
    }
}

// SpatialSchedule encoding (fixed size: 1 byte)
impl Encode for SpatialSchedule {
    fn encode(&self, buf: &mut [u8]) {
        buf[0] = match self {
            SpatialSchedule::RowMajor => 0,
        };
    }
}

impl Decode for SpatialSchedule {
    fn decode(buf: &[u8]) -> Self {
        match buf[0] {
            0 => SpatialSchedule::RowMajor,
            _ => panic!("Invalid spatial schedule"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use drift_protocol::Event;

    #[test]
    fn test_event_roundtrip() {
        let original = Event {
            tick: 12345,
            event_type: 42,
            payload: [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ],
        };

        let mut buf = [0u8; 42];
        original.encode(&mut buf);

        let decoded = Event::decode(&buf);
        assert_eq!(original, decoded);
    }
}
