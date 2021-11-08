//! Secondary's Block builders and iterators
//!
//! [`Block`] is the minimum managing unit in the storage engine.

mod char_block_builder;
mod primitive_block_builder;
mod primitive_block_iterator;
mod primitive_nullable_block_builder;
mod varchar_block_builder;
pub use char_block_builder::*;
pub use primitive_block_builder::*;
pub use primitive_block_iterator::*;
pub use primitive_nullable_block_builder::*;
pub use varchar_block_builder::*;

use bytes::{Buf, BufMut, Bytes};
use risinglight_proto::rowset::block_checksum::ChecksumType;
use risinglight_proto::rowset::block_index::BlockType;

use crate::array::Array;

/// A block is simply a [`Bytes`] array.
pub type Block = Bytes;

/// Builds a block. All builders should implement the trait, while
/// ensuring that the format follows the block encoding scheme.
///
/// In RisingLight, the block encoding scheme is as follows:
///
/// ```plain
/// | block_type | cksum_type | cksum  |    data     |
/// |    4B      |     4B     |   8B   |  variable   |
/// ```
pub trait BlockBuilder<A: Array> {
    /// Append one data into the block.
    fn append(&mut self, item: Option<&A::Item>);

    /// Get estimated size of block. Will be useful on runlength or compression encoding.
    fn estimated_size(&self) -> usize;

    /// Check if we should finish the current block. If there is no item in the current
    /// builder, this function must return `true`.
    fn should_finish(&self, next_item: &Option<&A::Item>) -> bool;

    /// Finish a block and return encoded data.
    fn finish(self) -> Vec<u8>;
}

/// An iterator on a block. This iterator requires the block being pre-loaded in memory.
pub trait BlockIterator<A: Array> {
    /// Get a batch from the block. A `0` return value means that this batch contains no
    /// element. Some iterators might support exact size output. By using `expected_size`,
    /// developers can get an array of NO MORE THAN the `expected_size`.
    fn next_batch(&mut self, expected_size: Option<usize>, builder: &mut A::Builder) -> usize;

    /// Skip `cnt` items.
    fn skip(&mut self, cnt: usize);

    /// Number of items remaining in this block
    fn remaining_items(&self) -> usize;
}

/// A key in block cache contains `rowset_id`, `column_id`
/// and `block_id`.
#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct BlockCacheKey {
    pub rowset_id: u32,
    pub storage_column_id: u32,
    pub block_id: u32,
}

impl BlockCacheKey {
    pub fn block(mut self, block_id: u32) -> Self {
        self.block_id = block_id;
        self
    }

    pub fn column(mut self, storage_column_id: u32) -> Self {
        self.storage_column_id = storage_column_id;
        self
    }

    pub fn rowset(mut self, rowset_id: u32) -> Self {
        self.rowset_id = rowset_id;
        self
    }
}

#[derive(Default, Debug, Clone)]
pub struct BlockHeader {
    pub block_type: BlockType,
    pub checksum_type: ChecksumType,
    pub checksum: u64,
}

pub const BLOCK_HEADER_SIZE: usize = 4 + 4 + 8;

impl BlockHeader {
    pub fn encode(&self, buf: &mut impl BufMut) {
        buf.put_i32(self.block_type.into());
        buf.put_i32(self.checksum_type.into());
        buf.put_u64(self.checksum);
    }

    pub fn decode(&mut self, buf: &mut impl Buf) {
        self.block_type = BlockType::from_i32(buf.get_i32()).unwrap();
        self.checksum_type = ChecksumType::from_i32(buf.get_i32()).unwrap();
        self.checksum = buf.get_u64();
    }
}
