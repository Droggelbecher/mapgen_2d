use crate::{coord::UCoord2Conversions, neighborhood::{Neighborhood, chebyshev}, region::Rect};
use float_ord::FloatOrd;
use glam::{uvec2, UVec2, IVec2};
use ndarray::{arr1, Array2, Array3, ArrayBase, Ix1, ViewRepr};
use num_traits::FromPrimitive;
use priority_queue::priority_queue::PriorityQueue;
use rand::{
    distributions::{Distribution, Uniform},
    SeedableRng,
};
use std::marker::PhantomData;

type Metric = fn(IVec2) -> u32;

/// Callback returning the probability of each possible tile given its neighborhood.
pub trait ProbabilityCallback<T, const N: usize>: FnMut(&Neighborhood<T, Metric>) -> [f32; N] {}

impl<F, T, const N: usize> ProbabilityCallback<T, N> for F where
    F: FnMut(&Neighborhood<T, Metric>) -> [f32; N],
{
}

/// Concrete type used for default probability callback
type DefaultProbabilityCallback<T, const N: usize> = fn(&Neighborhood<T, Metric>) -> [f32; N];

/// Configuration of a Wave Function Collapse run over a grid with cell type `T`,
/// a probability callback function type `F`, `N` different options for each cell.
#[derive(Clone)]
pub struct WaveFunctionCollapse<T, F, const N: usize>
where
    F: ProbabilityCallback<T, N>,
{
    seed: u64,
    size: UVec2,
    probability: F,
    neighborhood_size: u32,
    bomb_radius: u32,
    max_bombings: u32,

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
            seed,
            size,
            probability,
            bomb_radius: 10,
            max_bombings: 10,
            neighborhood_size: 1,
            _tile: Default::default(),
        }
    }

    pub fn neighborhood_size(mut self, neighborhood_size: u32) -> Self {
        self.neighborhood_size = neighborhood_size;
        self
    }

    pub fn bomb_radius(mut self, bomb_radius: u32) -> Self {
        self.bomb_radius = bomb_radius;
        self
    }

    pub fn max_bombings(mut self, max_bombings: u32) -> Self {
        self.max_bombings = max_bombings;
        self
    }

    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    pub fn size(mut self, size: UVec2) -> Self {
        self.size = size;
        self
    }

    /// Conclude configuration and return an intermediate result in which no actual computation has
    /// been done yet.
    pub fn build(self) -> WaveFunctionCollapseResult<T, F, N> {
        WaveFunctionCollapseResult {
            tiles: Array2::from_elem(self.size.as_index2(), T::default()),
            valid: Array2::from_elem(self.size.as_index2(), false),
            entropy: Default::default(),
            probabilities: Array3::from_elem(self.size.as_index3(N), NO_PROBABILITY),
            configuration: self,
            bombings_done: 0,
        }
    }

    /// Conclude configuration and do the actual computation
    pub fn generate(self) -> WaveFunctionCollapseResult<T, F, N> {
        self.build().regenerate()
    }
}

impl<T, const N: usize> Default for WaveFunctionCollapse<T, DefaultProbabilityCallback<T, N>, N>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            seed: 0_u64,
            size: uvec2(100, 100),
            probability: |_| [0.0_f32; N],
            bomb_radius: 10,
            max_bombings: 10,
            neighborhood_size: 1,
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
    bombings_done: u32,
}

pub const NO_PROBABILITY: f32 = -1.0;

impl<T, F, const N: usize> WaveFunctionCollapseResult<T, F, N>
where
    F: ProbabilityCallback<T, N>,
    usize: From<T>,
    T: FromPrimitive + std::fmt::Debug + Clone + Copy + Default,
{
    /// Recompute the current result with the given configuration.
    /// I the configuration (including random seed) has not been changed,
    /// the result should stay the same.
    pub fn regenerate(mut self) -> Self {
        let mut rng = rand::rngs::StdRng::seed_from_u64(self.configuration.seed);
        let all = Rect::from_size(self.configuration.size);
        self.bombings_done = 0;

        // 1. compute all them probabilities
        self.compute_probabilities(all);

        // 2. compute all entropies, find max
        self.compute_entropies(all);

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

                    // unwrap(): We assume that this enum can be constructed from the probability
                    // index, i.e. that your enum is continuous and at least N elements long
                    tile = Some(T::from_usize(i).unwrap());
                    break;
                }
            }

            // 4. Set tile & update surroundings
            match tile {
                None => panic!(),
                Some(t) => {
                    if !self.set_tile(target, t.into()) {
                        self.backtrack(target);
                    }
                }
            }
        }

        self
    }

    fn is_valid(&self, pos: UVec2) -> bool {
        self.valid[pos.as_index2()]
    }

    fn set_tile(&mut self, pos: UVec2, tile: T) -> bool {
        assert!(!self.is_valid(pos));

        self.tiles[pos.as_index2()] = tile;
        self.valid[pos.as_index2()] = true;

        let neighborhood = Neighborhood::<T, Metric>::new(&self.tiles, pos.as_ivec2(), chebyshev, self.configuration.neighborhood_size);

        // We need to recompute probabilities & entropies for all neighbors
        for neigh in neighborhood.iter_positions() {
            if self.is_valid(neigh) {
                // We only care for invalid (== not-yet-determined) tiles
                continue;
            }

            if !Self::update_probability(
                neigh,
                &self.tiles,
                &mut self.configuration.probability,
                &mut self.probabilities,
                self.configuration.neighborhood_size
            ) {
                return false;
            }

            Self::compute_entropy(neigh, &self.probabilities, &mut self.entropy);
        }

        // Probability for this field is 1.0 for the tile we set, 0 for everything else
        let mut ps = self.probabilities.slice_mut(pos.as_slice3d());
        ps.fill(0.0);
        ps[usize::from(tile)] = 1.0;
        true
    }

    fn get_probabilities(&self, pos: UVec2) -> ArrayBase<ViewRepr<&f32>, Ix1> {
        self.probabilities.slice(pos.as_slice3d())
    }

    fn compute_probabilities(&mut self, rect: Rect) -> bool {
        println!("compute_probabilities {rect:?}");

        for idx in rect.iter_indices() {
            self.tiles[idx.as_index2()] = T::default();
            self.valid[idx.as_index2()] = false;
        }

        for idx in rect.iter_indices() {
            // This is called once at the beginning of computation,
            // If update_probability fails here it means some field already
            // has no solution by definition, this should never happen.
            if !Self::update_probability(
                idx,
                &self.tiles,
                &mut self.configuration.probability,
                &mut self.probabilities,
                self.configuration.neighborhood_size
            ) {
                return false;
            }
        }

        let idx = rect.top_left();
        assert!(
            self.probabilities
                .slice(idx.as_slice3d())
                .iter()
                .filter(|x| **x > 0.0)
                .count()
                > 1
        );

        let idx = rect.bottom_right();
        assert!(
            self.probabilities
                .slice(idx.as_slice3d())
                .iter()
                .filter(|x| **x > 0.0)
                .count()
                > 1
        );
        true
    }

    fn update_probability(
        pos: UVec2,
        tiles: &Array2<T>,
        f: &mut F,
        probabilities: &mut Array3<f32>,
        neighborhood_size: u32
    ) -> bool {
        let neighborhood = Neighborhood::new(tiles, pos.as_ivec2(), chebyshev, neighborhood_size);
        let ps = f(&neighborhood);

        let s: f32 = ps.iter().sum();
        if ps[0] == NO_PROBABILITY || s <= 0.0 {
            println!("f({pos:?}) = {ps:?}");
            return false;
        }

        let ps = ps.map(|p| p / s);
        probabilities.slice_mut(pos.as_slice3d()).assign(&arr1(&ps));
        true
    }

    fn backtrack(&mut self, pos: UVec2) {
        let mut radius = self.configuration.bomb_radius as u64 * 2_u64.pow(self.bombings_done);

        loop {
            println!("Backtracking around: {pos} {radius}");
            let bomb_area = Rect::around(pos, radius as u32)
                .intersect(Rect::from_size(self.configuration.size));
            self.bombings_done += 1;
            if self.bombings_done > self.configuration.max_bombings {
                panic!();
            }

            if !self.compute_probabilities(bomb_area) {
                radius = self.configuration.bomb_radius as u64 * 2_u64.pow(self.bombings_done);

                continue;
            }
            self.compute_entropies(bomb_area);
            break;
        }
    }

    fn compute_entropies(&mut self, rect: Rect) {
        for idx in rect.iter_indices() {
            let ps = self.probabilities.slice(idx.as_slice3d());
            let e = -ps.mapv(|p| if p == 0.0 { 0.0 } else { p * p.log2() }).sum();
            self.entropy.push(idx, FloatOrd(e));
        }
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
