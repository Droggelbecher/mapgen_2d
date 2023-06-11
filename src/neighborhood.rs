use crate::{
    coord::UCoord2Conversions,
    region::{Rect, RectIterator},
};
use counter::Counter;
use glam::{ivec2, uvec2, IVec2, UVec2};
use ndarray::Array2;
use num::integer::Roots;
use std::hash::Hash;

/// Representation of a neighborhood in a 2d grid.
pub struct NeighborPositions<M>
where
    M: FnMut(IVec2) -> u32 + Copy,
{
    /// size of the grid
    size: UVec2,
    /// position around which this neighborhood is defined
    /// Note that this may be outside the grid (and thus potentially negative)
    position: IVec2,
    /// neighborhood radius
    radius: u32,
    /// distance metric
    metric: M,
}

impl<M> NeighborPositions<M>
where
    M: FnMut(IVec2) -> u32 + Copy,
{
    pub fn new(size: UVec2, position: IVec2, metric: M, radius: u32) -> Self {
        Self {
            size,
            position,
            radius,
            metric
        }
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn position(&self) -> IVec2 {
        self.position
    }

    pub fn radius(&self) -> u32 {
        self.radius
    }

    /// Iterate over all the positions in the neighborhood.
    /// This will not include the position around which the neighborhood is defined.
    pub fn iter(&self) -> impl Iterator<Item = UVec2> {
        let pos = self.position;
        let mut metric = self.metric;
        let radius = self.radius;
        let r = ivec2(radius as i32, radius as i32);
        let top_left = (pos - r).clamp(ivec2(0, 0), self.size.as_ivec2() - ivec2(1, 1));
        let bottom_right = (pos + r).clamp(ivec2(0, 0), self.size.as_ivec2() - ivec2(1, 1));

        RectIterator::new(Rect::from_corners(
            top_left.as_uvec2(),
            bottom_right.as_uvec2(),
        ))
        .filter(move |x| {
            x.as_ivec2() != pos
            && metric(x.as_ivec2() - pos) <= radius
        })
    }
}

// |x| + |y| <= r (diamond)
pub fn manhattan(a: IVec2) -> u32 {
    a.x.abs() as u32 + a.y.abs() as u32
}

// max(|x|, |y|) <= r (square)
pub fn chebyshev(a: IVec2) -> u32 {
    a.x.abs().max(a.y.abs()) as u32
}

// sqrt(|x|^2 + |y|^2) <= r (disc)
pub fn euclidean(a: IVec2) -> u32 {
    (a.x.abs().pow(2) + a.y.abs().pow(2)).sqrt() as u32
}

#[test]
fn test_neighbor_positions() {
    let np = NeighborPositions::new(uvec2(10, 10), ivec2(-1, 8), chebyshev, 3);
    assert_eq!(
        np.iter().collect::<Vec<_>>(),
        vec![
            uvec2(0, 5),
            uvec2(1, 5),
            uvec2(2, 5),
            uvec2(0, 6),
            uvec2(1, 6),
            uvec2(2, 6),
            uvec2(0, 7),
            uvec2(1, 7),
            uvec2(2, 7),
            uvec2(0, 8),
            uvec2(1, 8),
            uvec2(2, 8),
            uvec2(0, 9),
            uvec2(1, 9),
            uvec2(2, 9)
        ]
    );

    let np = NeighborPositions::new(uvec2(10, 10), ivec2(-1, 8), euclidean, 3);
    assert_eq!(
        np.iter().collect::<Vec<_>>(),
        vec![
            uvec2(0, 5),
            uvec2(1, 5),
            uvec2(0, 6),
            uvec2(1, 6),
            uvec2(2, 6),
            uvec2(0, 7),
            uvec2(1, 7),
            uvec2(2, 7),
            uvec2(0, 8),
            uvec2(1, 8),
            uvec2(2, 8),
            uvec2(0, 9),
            uvec2(1, 9),
            uvec2(2, 9)
        ]
    );

    let np = NeighborPositions::new(uvec2(10, 10), ivec2(-1, 8), manhattan, 3);
    assert_eq!(
        np.iter().collect::<Vec<_>>(),
        vec![
            uvec2(0, 6),
            uvec2(0, 7),
            uvec2(1, 7),
            uvec2(0, 8),
            uvec2(1, 8),
            uvec2(2, 8),
            uvec2(0, 9),
            uvec2(1, 9),
        ]
    );
}

/// Represents the 2d neighborhood around a tile located
/// at a certain positon in a given array.
/// Generally, methods here will refer to the tiles around the given
/// position, not including that tile itself.
pub struct Neighborhood<'a, T, M>
where
    M: FnMut(IVec2) -> u32 + Copy
{
    a: &'a Array2<T>,
    positions: NeighborPositions<M>,
}

impl<'a, T, M> Neighborhood<'a, T, M>
where
    T: Clone + Copy,
    M: FnMut(IVec2) -> u32 + Copy,
{
    /// Constructor.
    /// Note that position is signed, ie. it is allowed to be outside the array area.
    pub fn new(a: &'a Array2<T>, position: IVec2, metric: M, radius: u32) -> Self {
        let size = uvec2(a.shape()[0] as u32, a.shape()[1] as u32);

        Self {
            a,
            positions: NeighborPositions::new(size, position, metric, radius),
        }
    }

    pub fn position(&self) -> IVec2 {
        self.positions.position()
    }

    pub fn get(&self, offset: IVec2) -> Option<T> {
        assert!(offset.x >= -1 && offset.x <= 1);
        assert!(offset.y >= -1 && offset.y <= 1);

        let p = self.position() + offset;
        match self.in_map(p) {
            true => Some(self.a[p.as_uvec2().as_index2()].into()),
            false => None,
        }
    }

    /// Count the number of tiles of type `x` in the neighborhood.
    pub fn count(&self, x: T) -> usize
    where
        T: Eq,
    {
        self.iter().filter(|n| *n == x).count()
    }

    pub fn most_common(&self) -> Option<T>
    where
        T: Hash + Eq + PartialOrd + Ord,
    {
        let counts = self.iter().collect::<Counter<_>>();
        let most_common = counts.k_most_common_ordered(1);
        if most_common.len() > 0 {
            Some(most_common[0].0)
        } else {
            None
        }
    }

    pub fn has_only(&self, x: Vec<T>) -> bool
    where
        T: Eq,
    {
        self.iter().map(|n| x.contains(&n)).all(|x| x)
    }

    /// Iterate all neighors with their positions.
    /// Yields `None` for positions outside of the array area.
    pub fn iter_with_positions(&self) -> impl Iterator<Item = (UVec2, T)> + '_ {
        self.iter_positions().map(|p| (p, self.a[p.as_index2()]))
    }

    /// Iterate tiles in the neighborhood.
    /// Yields `None` for positions outside of the array area.
    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.iter_positions().map(|p| self.a[p.as_index2()])
    }

    /// All generated positions will be inside the map area and thus >= 0
    pub fn iter_positions(&self) -> impl Iterator<Item = UVec2> + '_ {
        self.positions.iter()
    }

    fn in_map_of_size(p: IVec2, size: UVec2) -> bool {
        p.x >= 0 && p.y >= 0 && p.x < (size.x as i32) && p.y < (size.y as i32)
    }

    fn in_map(&self, p: IVec2) -> bool {
        Self::in_map_of_size(p, self.positions.size)
    }
}
