use crate::block::Block;

impl Block {
    /// Creates a mock block.
    pub fn create_for_testing() -> Block {
        Block::default()
    }
}
