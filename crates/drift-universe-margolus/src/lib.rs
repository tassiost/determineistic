use drift_protocol::{Chunk, CHUNK_WIDTH, CHUNK_HEIGHT, SpatialSchedule};

/// Margolus CA phases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    A,  // (x even, y even)
    B,  // (x odd, y odd)
}

/// 2x2 block encoding: 16-bit pattern (4 cells × 4 bits)
///
/// Cell layout in block:
/// [0][1]
/// [2][3]
///
/// Each cell uses 4 bits (can represent 16 states)
/// Bit layout: cell3(11-8) | cell2(7-4) | cell1(3-0) | cell0(-3)
pub type Block = u16;

/// Extract cell value from block
pub fn block_get_cell(block: Block, cell_index: usize) -> u8 {
    let shift = cell_index * 4;
    ((block >> shift) & 0xF) as u8
}

/// Set cell value in block
pub fn block_set_cell(block: Block, cell_index: usize, value: u8) -> Block {
    let shift = cell_index * 4;
    let masked_value = ((value as u16) & 0xF) << shift;
    let mask = !(0xFu16 << shift);
    (block & mask) | masked_value
}

/// Pack 4 cells into a block
pub fn pack_block(c0: u8, c1: u8, c2: u8, c3: u8) -> Block {
    ((c3 as u16) << 12) | ((c2 as u16) << 8) | ((c1 as u16) << 4) | (c0 as u16)
}

/// Simple Margolus rule: rotate cells clockwise
/// [0][1]    [2][0]
/// [2][3] -> [3][1]
fn rotate_clockwise(block: Block) -> Block {
    let c0 = block_get_cell(block, 0);
    let c1 = block_get_cell(block, 1);
    let c2 = block_get_cell(block, 2);
    let c3 = block_get_cell(block, 3);
    pack_block(c2, c0, c3, c1)
}

/// Margolus rule table
///
/// Fixed array storage (NOT hash map)
/// Sorted by input bit-pattern ascending
/// All 2^16 input patterns have explicit output
pub struct RuleTable {
    rules: [Block; 65536],  // 2^16 entries
}

impl RuleTable {
    /// Create a new rule table with clockwise rotation rule
    /// Phase 0 uses a simple deterministic rule for conformance testing
    pub fn rotation() -> Self {
        let mut rules = [0u16; 65536];
        for i in 0..65536 {
            rules[i] = rotate_clockwise(i as u16);
        }
        RuleTable { rules }
    }

    /// Apply rule to a block for a given phase
    /// Phase 0: same rule for both phases (simplifies conformance)
    pub fn apply(&self, block: Block, _phase: Phase) -> Block {
        self.rules[block as usize]
    }
}

impl Default for RuleTable {
    fn default() -> Self {
        Self::rotation()
    }
}

/// Simulate one tick of Margolus CA on a single chunk
///
/// This is a pure function - no in-place mutation
/// Uses double buffering internally
pub fn simulate_chunk(
    chunk: &Chunk,
    rule_table: &RuleTable,
    _spatial_schedule: SpatialSchedule,
) -> Chunk {
    let mut next = Chunk::new();

    // Phase A: process blocks at (x even, y even)
    for y in (0..CHUNK_HEIGHT).step_by(2) {
        for x in (0..CHUNK_WIDTH).step_by(2) {
            let block = extract_block(chunk, x, y);
            let updated = rule_table.apply(block, Phase::A);
            write_block(&mut next, x, y, updated);
        }
    }

    // Phase B: process blocks at (x odd, y odd)
    for y in (1..CHUNK_HEIGHT - 1).step_by(2) {
        for x in (1..CHUNK_WIDTH - 1).step_by(2) {
            let block = extract_block(&next, x, y);
            let updated = rule_table.apply(block, Phase::B);
            write_block(&mut next, x, y, updated);
        }
    }

    next
}

/// Extract a 2x2 block from chunk at position (x, y)
fn extract_block(chunk: &Chunk, x: usize, y: usize) -> Block {
    let c0 = chunk.get_cell(x, y);
    let c1 = chunk.get_cell(x + 1, y);
    let c2 = chunk.get_cell(x, y + 1);
    let c3 = chunk.get_cell(x + 1, y + 1);
    pack_block(c0, c1, c2, c3)
}

/// Write a 2x2 block to chunk at position (x, y)
fn write_block(chunk: &mut Chunk, x: usize, y: usize, block: Block) {
    chunk.set_cell(x, y, block_get_cell(block, 0));
    chunk.set_cell(x + 1, y, block_get_cell(block, 1));
    chunk.set_cell(x, y + 1, block_get_cell(block, 2));
    chunk.set_cell(x + 1, y + 1, block_get_cell(block, 3));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_packing() {
        let block = pack_block(1, 2, 3, 4);
        assert_eq!(block_get_cell(block, 0), 1);
        assert_eq!(block_get_cell(block, 1), 2);
        assert_eq!(block_get_cell(block, 2), 3);
        assert_eq!(block_get_cell(block, 3), 4);
    }

    #[test]
    fn test_block_set_cell() {
        let block = pack_block(0, 0, 0, 0);
        let block = block_set_cell(block, 1, 5);
        assert_eq!(block_get_cell(block, 1), 5);
    }

    #[test]
    fn test_rotation() {
        let block = pack_block(1, 2, 3, 4);
        let rotated = rotate_clockwise(block);
        assert_eq!(block_get_cell(rotated, 0), 3);
        assert_eq!(block_get_cell(rotated, 1), 1);
        assert_eq!(block_get_cell(rotated, 2), 4);
        assert_eq!(block_get_cell(rotated, 3), 2);
    }

    #[test]
    fn test_rule_table_rotation() {
        let table = RuleTable::rotation();
        let block = pack_block(1, 2, 3, 4);
        let result = table.apply(block, Phase::A);
        assert_eq!(block_get_cell(result, 0), 3);
        assert_eq!(block_get_cell(result, 1), 1);
        assert_eq!(block_get_cell(result, 2), 4);
        assert_eq!(block_get_cell(result, 3), 2);
    }
}
