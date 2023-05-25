use crate::{coord::UCoord2Conversions, neighborhood::Neighborhood};
use float_ord::FloatOrd;
use glam::{uvec2, UVec2};
use ndarray::{arr1, Array2, Array3, ArrayBase, Ix1, ViewRepr};
use num_traits::FromPrimitive;
use priority_queue::priority_queue::PriorityQueue;
use rand::{
    distributions::{Distribution, Uniform},
    SeedableRng,
};
use std::marker::PhantomData;

/// Callback returning the probability of each possible tile given its neighborhood.
pub trait ProbabilityCallback<T, const N: usize>: FnMut(&Neighborhood<T>) -> [f32; N] {}

impl<F, T, const N: usize> ProbabilityCallback<T, N> for F where
    F: FnMut(&Neighborhood<T>) -> [f32; N]
{
}

type DefaultProbabilityCallback<T, const N: usize> = fn(&Neighborhood<T>) -> [f32; N];

pub struct WaveFunctionCollapse<T, F, const N: usize>
where
    F: ProbabilityCallback<T, N>,
{
    // TODO: Consider builder pattern here rather than making these pub
    pub seed: u64,
    pub size: UVec2,
    pub probability: F,

    _tile: PhantomData<T>,
}

impl<T, F, const N: usize> WaveFunctionCollapse<T, F, N>
where
    F: ProbabilityCallback<T, N>,
    usize: From<T>,
    T: FromPrimitive + std::fmt::Debug + Clone + Copy + Default,
{
    pub fn new(size: UVec2, seed: u64, probability: F) -> Self {
        Self {
            seed, size, probability,
            _tile: Default::default(),
        }
    }

    pub fn build(self) -> WaveFunctionCollapseResult<T, F, N> {
        WaveFunctionCollapseResult {
            tiles: Array2::from_elem(self.size.as_index2(), T::default()),
            valid: Array2::from_elem(self.size.as_index2(), false),
            entropy: Default::default(),
            probabilities: Array3::from_elem(self.size.as_index3(N), NO_PROBABILITY),
            configuration: self,
        }
    }

    pub fn generate(self) -> WaveFunctionCollapseResult<T, F, N> {
        self.build().regenerate()
    }
}

impl<T, const N: usize> Default
    for WaveFunctionCollapse<T, DefaultProbabilityCallback<T, N>, N>
where
    T: Default,
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

pub struct WaveFunctionCollapseResult<T, F, const N: usize>
where
    F: ProbabilityCallback<T, N>,
{
    pub configuration: WaveFunctionCollapse<T, F, N>,
    pub tiles: Array2<T>,
    valid: Array2<bool>,
    probabilities: Array3<f32>,
    entropy: PriorityQueue<UVec2, FloatOrd<f32>>,
}

pub const NO_PROBABILITY: f32 = -1.0;

impl<T, F, const N: usize> WaveFunctionCollapseResult<T, F, N>
where
    F: ProbabilityCallback<T, N>,
    usize: From<T>,
    T: FromPrimitive + std::fmt::Debug + Clone + Copy,
{
    pub fn regenerate(mut self) -> Self {
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
                println!("i={:?} p={:?} psum={:?} roll={:?}", i, p, p_sum, roll);
                if roll <= p_sum {
                    // We shouldnt select a tile with zero probability, ever.
                    assert!(*p != 0.0);

                    // unwrap(): We assume that this enum can be constructed from the probability
                    // index, i.e. that your enum is continuous and at least N elements long
                    tile = Some(T::from_usize(i).unwrap());
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

        self
    }

    fn is_valid(&self, pos: UVec2) -> bool {
        self.valid[pos.as_index2()]
    }

    fn set_tile(&mut self, pos: UVec2, tile: T) {
        assert!(!self.is_valid(pos));

        self.tiles[pos.as_index2()] = tile;
        self.valid[pos.as_index2()] = true;

        let neighborhood =
            Neighborhood::<T>::new(&self.tiles, pos.as_ivec2());

        println!("set_tile {:?} = {:?}", pos, tile);

        // We need to recompute probabilities & entropies for all neighbors
        for neigh in neighborhood.iter_positions() {
            println!("   neighbor {:?}", neigh);

            if self.is_valid(neigh) {
                println!("   (neighbor valid {:?})", neigh);
                // We only care for invalid (== not-yet-determined) tiles
                continue;
            }

            println!("   recomputing neighbor {:?}", neigh);

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
        ps[usize::from(tile)] = 1.0;
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
        //println!("-> {:?}", ps);
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
        // We assume the item is already in the queue
        entropy.change_priority(&pos, FloatOrd(e));
    }
}

