
use ndarray::Array2;
use glam::{UVec2, Vec2, uvec2};
use kd_tree::{KdTree, KdPoint};
use typenum;
use crate::region::Region;
use std::cmp::{min, max};

#[derive(Clone)]
pub struct Voronoi {
    // TODO: turn into a builder, hide VoronoiCenter
    pub size: UVec2,
    pub centers: Vec<VoronoiCenter>,
}

pub struct VoronoiResult {
    pub input_configuration: Voronoi,
    pub output_configuration: Voronoi,
    pub map: Array2<usize>,
    pub regions: Vec<Region<usize>>,
}

impl Voronoi {

    pub fn generate(&self) -> VoronoiResult {
        let kdtree = KdTree::build_by_ordered_float(
            self.centers.clone()
        );

        // TODO: Allow providing this from outside?
        let mut a = Array2::zeros((self.size.x as usize, self.size.y as usize));

        let mut regions: Vec<_> = self.centers.iter().map(|c| {
            Region {
                anchor: c.position.as_uvec2(),
                size: uvec2(1, 1),
                reference: c.index.into(),
                // TODO XXX: we would like to reference this array but that is being moved
                // at the end of the function so the ref lifetime is too short, what can we do?
                //a: &a
            }
        }).collect();

        for ix in 0..self.size.x {
            for iy in 0..self.size.y {
                let found = kdtree.nearests(&[ix as f32, iy as f32], 3);

                let index = found[0].item.index;

                // This is needed for the "smooth" wall.
                // TODO: Make this more configurable
                let d1 = found[1].squared_distance - found[0].squared_distance;
                let d2 = found[2].squared_distance - found[0].squared_distance;

                // TODO: Make configurable / dependent on expected cell size
                if d1 * d2 >= 5000000.0 {
                    a[[ix as usize, iy as usize]] = index;

                    let region = &mut regions[index];
                    assert!(region.reference == index);
                    region.anchor = uvec2(
                        min(region.anchor.x, ix),
                        min(region.anchor.y, iy),
                    );

                    if ix > region.anchor.x {
                        region.size.x = max(region.size.x, (ix - region.anchor.x) as u32);
                    }
                    if iy > region.anchor.x {
                        region.size.y = max(region.size.y, (iy - region.anchor.y) as u32);
                    }
                }
            }
        }

        VoronoiResult {
            output_configuration: self.clone(),
            input_configuration: self.clone(),
            map: a,
            regions
        }

    }

    pub fn lloyd_step(&mut self, _a: &mut Array2<u32>) {
        // TODO: lloyd step
        todo!()
    }

    /*
    pub fn add_walls(&self, a: &mut Array2<u32>) {
        for ix in 0..self.size.x as usize {
            for iy in 0..self.size.y as usize {
                for dx in [-1, 1] {
                    for dy in [-1, 1] {
                    }
                }
            }
        }
    }
    */
}

#[derive(Clone)]
pub struct VoronoiCenter {
    pub position: Vec2,
    pub index: usize,
}

impl KdPoint for VoronoiCenter {
    type Scalar = f32;
    type Dim = typenum::U2;

    fn at(&self, k: usize) -> f32 {
        self.position[k]
    }
}

