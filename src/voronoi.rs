use crate::{
    coord::UCoord2Conversions,
    region::{Rect, Region},
};
use glam::{vec2, UVec2, Vec2};
use kd_tree::{KdPoint, KdTree};
use ndarray::Array2;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::SmallRng,
    SeedableRng,
};
use typenum;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct VoronoiCell(pub usize);

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum VoronoiTile {
    Border,
    Cell(VoronoiCell),
}

#[derive(Clone)]
pub struct Voronoi {
    pub size: UVec2,
    pub centers: Vec<VoronoiCenter>,
    pub border_coefficient: f32,
    pub min_border_width: f32,
    pub n_lloyd_steps: usize,
}

pub struct VoronoiResult {
    pub input_configuration: Voronoi,
    pub output_configuration: Voronoi,
    pub map: Array2<VoronoiTile>,
    pub regions: Vec<Region<VoronoiTile>>,
}

impl Voronoi {
    pub fn new(size: UVec2) -> Self {
        Self {
            size,
            centers: Vec::new(),
            border_coefficient: 0.0,
            min_border_width: 0.0,
            n_lloyd_steps: 0,
        }
    }

    pub fn random_centers(mut self, n: usize) -> Self {
        let mut rng = SmallRng::seed_from_u64(0);

        let uniform_x = Uniform::<f32>::from(0.0..self.size.x as f32);
        let uniform_y = Uniform::<f32>::from(0.0..self.size.y as f32);
        self.centers = (0..n)
            .map(|i| VoronoiCenter {
                position: vec2(uniform_x.sample(&mut rng), uniform_y.sample(&mut rng)),
                cell: VoronoiCell(i),
            })
            .collect();
        self
    }

    pub fn border_coefficient(mut self, c: f32) -> Self {
        self.border_coefficient = c;
        self
    }

    pub fn generate(&self) -> VoronoiResult {
        let a = Array2::from_elem(self.size.as_index2(), VoronoiTile::Border);
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
                let VoronoiTile::Cell(cell) = t else { continue; };

                counts[cell.0] += 1.0;
                center_sums[cell.0] += vec2(ix as f32 + 0.5, iy as f32 + 0.5);
            }
        }

        self.output_configuration.centers = center_sums
            .iter()
            .zip(counts)
            .enumerate()
            .map(|(i, (s, n))| VoronoiCenter {
                position: *s / n,
                cell: VoronoiCell(i),
            })
            .collect();
    }

    pub fn recompute(&mut self) {
        let cfg = &self.output_configuration;
        let kdtree = KdTree::build_by_ordered_float(cfg.centers.clone());

        // TODO: assert self.map already has correct shape
        self.map.fill(VoronoiTile::Border);

        let mut regions: Vec<_> = cfg
            .centers
            .iter()
            .map(|c| Region {
                bounding_box: Rect::from_corners(c.position.as_uvec2(), c.position.as_uvec2()),
                reference: VoronoiTile::Cell(c.cell),
            })
            .collect();

        for ix in 0..cfg.size.x {
            for iy in 0..cfg.size.y {
                let found = kdtree.nearests(&[ix as f32 + 0.5, iy as f32 + 0.5], 3);
                if found.len() < 3 {
                    continue;
                }

                let cell = found[0].item.cell;
                let d1 = found[1].squared_distance.sqrt() - found[0].squared_distance.sqrt();
                //let d2 = found[2].squared_distance.sqrt() - found[0].squared_distance.sqrt();

                //if (d1 * d2 >= cfg.border_coefficient) && d1 >= cfg.min_border_width {
                if d1 < 100.0 {
                    self.map[[ix as usize, iy as usize]] = VoronoiTile::Cell(cell);

                    let region = &mut regions[cell.0];
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
    pub cell: VoronoiCell,
}

impl KdPoint for VoronoiCenter {
    type Scalar = f32;
    type Dim = typenum::U2;

    fn at(&self, k: usize) -> f32 {
        self.position[k]
    }
}
