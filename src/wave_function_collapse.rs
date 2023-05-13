use crate::{coord::UCoord2Conversions, neighborhood::Neighborhood, tile::Tile};
use float_ord::FloatOrd;
use glam::{uvec2, UVec2};
use ndarray::{arr1, Array2, Array3, ArrayBase, Ix1, ViewRepr};
use priority_queue::priority_queue::PriorityQueue;
use rand::{
    distributions::{Distribution, Uniform},
    SeedableRng,
};
use std::marker::PhantomData;

pub trait ProbabilityCallback<T, const N: usize>: FnMut(&Neighborhood<T>) -> [f32; N] {}

impl<F, T, const N: usize> ProbabilityCallback<T, N> for F where
    F: FnMut(&Neighborhood<T>) -> [f32; N]
{
}

type DefaultProbabilityCallback<T, const N: usize> = fn(&Neighborhood<T>) -> [f32; N];

// TODO: Consistent Lingo, over in map.rs we call these builders "settings"
pub struct WaveFunctionCollapseConfiguration<T, F, const N: usize>
where
    F: ProbabilityCallback<T, N>,
{
    // TODO: Consider builder pattern here rather than making these pub
    pub seed: u64,
    pub size: UVec2,
    pub probability: F,

    // TODO: Hide this again
    pub _tile: PhantomData<T>,
}

pub struct WaveFunctionCollapse<T, F, const N: usize>
where
    F: ProbabilityCallback<T, N>,
    T: Tile,
{
    pub configuration: WaveFunctionCollapseConfiguration<T, F, N>,
    pub tiles: Array2<T>,
    probabilities: Array3<f32>,
    entropy: PriorityQueue<UVec2, FloatOrd<f32>>,
}

pub const NO_PROBABILITY: f32 = -1.0;

impl<T, F, const N: usize> WaveFunctionCollapse<T, F, N>
where
    F: ProbabilityCallback<T, N>,
    T: Tile,
{
    pub fn generate(&mut self) {
        let mut rng = rand::rngs::StdRng::seed_from_u64(self.configuration.seed);

        // 1. compute all them probabilities
        self.compute_probabilities();

        // 2. compute all entropies, find max
        self.compute_entropies();

        loop {
            // 5. Find max entropy
            let (target, _) = match self.entropy.pop() {
                None => break, // done :)
                Some(x) => x,
            };

            // 3. Choose tile for target location
            let mut p_sum = 0.0;
            let roll = Uniform::<f32>::from(0.0..1.0).sample(&mut rng);
            let mut tile = None;
            for (i, p) in self.get_probabilities(target).iter().enumerate() {
                p_sum += p;
                if roll <= p_sum {
                    // We shouldnt select a tile with zero probability, ever.
                    assert!(*p != 0.0);

                    tile = Some(i);
                    break;
                }
            }

            // 4. Set tile & update surroundings
            match tile {
                Some(t) => self.set_tile(target, t.into()),
                None => {
                    println!("p {:?}", self.get_probabilities(target));
                    panic!();
                }
            }
        }
    }

    fn set_tile(&mut self, pos: UVec2, tile: T) {
        assert!(tile.is_valid());
        assert!(!self.tiles[pos.as_index2()].is_valid());

        self.tiles[pos.as_index2()] = tile;

        let neighborhood = Neighborhood::<T>::new(&self.tiles, pos.as_ivec2());

        // We need to recompute probabilities & entropies for all neighbors
        for neigh in neighborhood.iter_positions() {
            if self.tiles[neigh.as_index2()].is_valid() {
                // We only care for invalid (== not-yet-determined) tiles
                continue;
            }

            Self::compute_probability(
                neigh,
                &self.tiles,
                &mut self.configuration.probability,
                &mut self.probabilities,
            );
            Self::compute_entropy(neigh, &self.probabilities, &mut self.entropy);
        }

        // Probability for this field is 1.0 for the tile we set, 0 for everything else
        let mut ps = self.probabilities.slice_mut(pos.as_slice3d());
        ps.fill(0.0);
        ps[tile.as_usize()] = 1.0;
    }

    fn get_probabilities(&self, pos: UVec2) -> ArrayBase<ViewRepr<&f32>, Ix1> {
        self.probabilities.slice(pos.as_slice3d())
    }

    fn compute_probabilities(&mut self) {
        for ix in 0..self.configuration.size.x {
            for iy in 0..self.configuration.size.y {
                Self::compute_probability(
                    (ix, iy).as_uvec2(),
                    &self.tiles,
                    &mut self.configuration.probability,
                    &mut self.probabilities,
                );
            }
        }
    }

    fn compute_probability(
        pos: UVec2,
        tiles: &Array2<T>,
        f: &mut F,
        probabilities: &mut Array3<f32>,
    ) {
        let neighborhood = Neighborhood::new(tiles, pos.as_ivec2());
        let ps = f(&neighborhood);

        let s: f32 = ps.iter().sum();
        if ps[0] == NO_PROBABILITY || s <= 0.0 {
            // TODO: if any(ps == NO_PROBABILITY) or all(ps == 0.0), backtrack!
            // XXX
            println!("ps={:?}", ps);
            println!(
                "neigh: {:?}",
                neighborhood.iter_with_positions().collect::<Vec<_>>()
            );
            todo!("Backtrack!");
        }

        let ps = ps.map(|p| p / s);
        probabilities.slice_mut(pos.as_slice3d()).assign(&arr1(&ps));
    }

    fn compute_entropies(&mut self) {
        for ix in 0..self.configuration.size.x {
            for iy in 0..self.configuration.size.y {
                let idx = (ix, iy).as_index2();
                let ps = self.probabilities.slice(idx.as_slice3d());
                let e = -ps.mapv(|p| if p == 0.0 { 0.0 } else { p * p.log2() }).sum();
                self.entropy.push((ix, iy).as_uvec2(), FloatOrd(e));
            } // for iy
        } // for ix
    }

    fn compute_entropy(
        pos: UVec2,
        probabilities: &Array3<f32>,
        entropy: &mut PriorityQueue<UVec2, FloatOrd<f32>>,
    ) {
        let ps = probabilities.slice(pos.as_slice3d());
        let e = -ps.mapv(|p| if p == 0.0 { 0.0 } else { p * p.log2() }).sum();
        entropy.change_priority(&pos, FloatOrd(e));
    }
}

impl<T, F, const N: usize> WaveFunctionCollapseConfiguration<T, F, N>
where
    F: ProbabilityCallback<T, N>,
    T: Tile,
{
    pub fn build(self) -> WaveFunctionCollapse<T, F, N> {
        WaveFunctionCollapse {
            tiles: Array2::from_elem(self.size.as_index2(), T::invalid()),
            entropy: Default::default(),
            probabilities: Array3::from_elem(self.size.as_index3(N), NO_PROBABILITY),
            configuration: self,
        }
    }
}

impl<T, const N: usize> Default
    for WaveFunctionCollapseConfiguration<T, DefaultProbabilityCallback<T, N>, N>
where
    T: Tile,
{
    fn default() -> Self {
        Self {
            seed: 0_u64,
            size: uvec2(100, 100),
            probability: |_| [0.0_f32; N],
            _tile: Default::default(),
        }
    }
}
