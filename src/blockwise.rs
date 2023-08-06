use std::{collections::HashMap, hash::Hash};

use crate::{manhattan, NeighborPositions, Rect};
use glam::{ivec2, uvec2, IVec2, UVec2};
use ndarray::{s, Array2};
use num_traits::Zero;
//use core::ops::Add;
use crate::coord::UCoord2Conversions;
//use slotmap::SlotMap;

#[derive(Clone, PartialEq, Eq, Default)]
struct BlockId(usize);

// TODO: Rename module to match this
pub struct Blocks<T> {
    //map: Array2<usize>,
    block_size: UVec2,
    blocks: HashMap<Array2<T>, BlockId>,

    // [center, top, right, bottom, left]
    neighborhood_configurations: Vec<[BlockId; 5]>,
    next_block_id: usize,
}

// Input Img -> AnalyzeBlocks -> blocks + neigh probabilities
// neigh probabilities -> WFC -> map(block_indices)
// map(block_indices) + blocks -> ApplyBlocks -> map

impl<T> Blocks<T>
where
    T: Hash + Eq + Clone,
{
    // TODO
    fn analyze_blocks(&mut self, source: &Array2<T>) {
        let mut block_views = Vec::new();

        // Assumes block size is >= (1, 1)
        for offset in Rect::from_size(self.block_size - uvec2(1, 1)).iter_indices() {
            // TODO: Iterate over the 4 possible 2d rotations, perhaps with a 2d rotation matrix
            //for _ in
            //{
            block_views.push(self.compute_block_view(source, offset));
            //}
        }

        for block_view in block_views {
            for position in
                Rect::from_corners(uvec2(1, 1), block_view.dim().as_uvec2() - uvec2(2, 2))
                    .iter_indices()
            {
                let config = [
                    // TODO: Get rid of these clones somehow?
                    block_view[position.as_index2()].clone(),
                    // top
                    block_view[(position.as_ivec2() + ivec2(0, -1)).as_uvec2().as_index2()].clone(),
                    // right
                    block_view[(position.as_ivec2() + ivec2(1, 0)).as_uvec2().as_index2()].clone(),
                    // bottom
                    block_view[(position.as_ivec2() + ivec2(0, 1)).as_uvec2().as_index2()].clone(),
                    // left
                    block_view[(position.as_ivec2() + ivec2(-1, 0)).as_uvec2().as_index2()].clone(),
                ];
                self.neighborhood_configurations.push(config);
            }
        }

        //
    }

    fn compute_block_view(&mut self, source: &Array2<T>, offset: UVec2) -> Array2<BlockId> {
        let mut block_view =
            Array2::default(((source.dim().as_uvec2() - offset) / self.block_size).as_index2());

        for position in Rect::from_raw_dim(block_view.raw_dim()).iter_indices() {
            let source_pos = position * self.block_size + offset;
            let block = source
                .slice(s![
                    source_pos.x as usize..(source_pos.x + self.block_size.x) as usize,
                    source_pos.y as usize..(source_pos.y + self.block_size.y) as usize,
                ])
                .into_owned();
            let block_id = self.blocks.entry(block).or_insert_with(|| {
                let r = self.next_block_id;
                self.next_block_id += 1;
                BlockId(r)
            });
            block_view[position.as_index2()] = block_id.clone();
        }

        block_view
    }

}
