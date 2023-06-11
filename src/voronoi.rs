use crate::{
    coord::UCoord2Conversions,
    region::{Rect, Region},
};
use glam::{uvec2, vec2, UVec2, Vec2};
use kd_tree::{KdPoint, KdTree};
use ndarray::Array2;
use typenum;

#[derive(Clone)]
pub struct Voronoi {
    pub size: UVec2,
    // TODO: hide VoronoiCenter
    pub centers: Vec<VoronoiCenter>,
    // TODO: would it be more consistent to use a type param for the tiles
    pub border_marker: usize,
    pub border_coefficient: f32,
    pub min_border_width: f32,
    pub n_lloyd_steps: usize,
}

pub struct VoronoiResult {
    pub input_configuration: Voronoi,
    pub output_configuration: Voronoi,
    pub map: Array2<usize>,
    pub regions: Vec<Region<usize>>,
}

impl Voronoi {
    pub fn generate(&self) -> VoronoiResult {
        let a = Array2::from_elem(self.size.as_index2(), self.border_marker);
        let mut r = VoronoiResult {
            output_configuration: self.clone(),
            input_configuration: self.clone(),
            map: a,
            regions: Default::default(),
        };
        r.recompute();
        for _ in 0..self.n_lloyd_steps {
            r.lloyd_step();
            r.recompute();
        }
        r
    }
}

impl VoronoiResult {
    fn lloyd_step(&mut self) {
        let cfg = self.output_configuration.clone();

        let mut counts = vec![1.0; cfg.centers.len()];
        let mut center_sums: Vec<_> = cfg.centers.iter().map(|x| x.position).collect();

        for ix in 0..cfg.size.x {
            for iy in 0..cfg.size.y {
                let t = self.map[(ix, iy).as_index2()];
                if t == cfg.border_marker {
                    continue;
                }

                //if t == 0 {
                    //continue;
                //}

                counts[t] += 1.0;
                center_sums[t] += vec2(ix as f32 + 0.5, iy as f32 + 0.5);
            }
        }

        self.output_configuration.centers = center_sums
            .iter()
            .zip(counts)
            .enumerate()
            .map(|(i, (s, n))| VoronoiCenter {
                position: *s / n,
                index: i,
            })
            .collect();
    }

    pub fn recompute(&mut self) {
        let cfg = &self.output_configuration;
        let kdtree = KdTree::build_by_ordered_float(cfg.centers.clone());

        // TODO: assert self.map already has correct shape
        self.map.fill(cfg.border_marker);

        let mut regions: Vec<_> = cfg
            .centers
            .iter()
            .map(|c| Region {
                bounding_box: Rect::from_corners(
                    c.position.as_uvec2(),
                    c.position.as_uvec2()
                ),
                reference: c.index,
            })
            .collect();

        for ix in 0..cfg.size.x {
            for iy in 0..cfg.size.y {
                let found = kdtree.nearests(&[ix as f32 + 0.5, iy as f32 + 0.5], 3);
                if found.len() < 3 {
                    continue;
                }

                let index = found[0].item.index;
                let d1 = found[1].squared_distance.sqrt() - found[0].squared_distance.sqrt();
                let d2 = found[2].squared_distance.sqrt() - found[0].squared_distance.sqrt();

                if (d1 * d2 >= cfg.border_coefficient)
                    && d1 >= cfg.min_border_width
                {
                    self.map[[ix as usize, iy as usize]] = index;

                    let region = &mut regions[index];
                    let bbox = &mut region.bounding_box;

                    bbox.grow_to_include((ix, iy).as_uvec2());
                }
            }
        }

        self.regions = regions;
    }
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
