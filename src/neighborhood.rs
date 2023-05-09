use crate::coord::{UCoord2, UCoord2Conversions};
use glam::{ivec2, uvec2, IVec2, UVec2};
use ndarray::{Array2, s};
use std::cmp::Ord;
use crate::tile::Tile;

/// Represents the 2d neighborhood around a tile located
/// at a certain positon in a given array.
/// Generally, methods here will refer to the tiles around the given
/// position, not including that tile itself.
pub struct Neighborhood<'a, T>
where
    T: Tile,
{
    a: &'a Array2<T::Numeric>,
    position: IVec2,
    size: UVec2,
}

impl<'a, T> Neighborhood<'a, T>
where
    T: Tile,
{
    /// Constructor.
    /// Note that position is signed, ie. it is allowed to be outside the array area.
    pub fn new(a: &'a Array2<T::Numeric>, position: IVec2) -> Self {
        let size = uvec2(a.shape()[0] as u32, a.shape()[1] as u32);

        Self {
            position,
            a,
            size,
        }
    }

    pub fn position(&self) -> IVec2 { self.position }

    pub fn get(&self, offset: IVec2) -> Option<T> {
        assert!(offset.x >= -1 && offset.x <= 1);
        assert!(offset.y >= -1 && offset.y <= 1);

        let p = self.position + offset;
        match self.in_map(p) {
            true => Some(self.a[p.as_uvec2().as_index2()].into()),
            false => None,
        }
    }

    /// min/max tile value in the neighborhood.
    /// Ignore invalid tiles.
    /// If there are no valid tiles in the neighborhood, return `None`.
    pub fn range(&self) -> Option<(T, T)> {
        let mut r = None;
        for neighbor in self.iter() {
            if let Some(n) = neighbor {
                if n.is_valid() {
                    r = match r {
                        None => Some((n, n)),
                        Some((a, b)) => Some((
                            a.as_numeric().min(n.as_numeric()).into(),
                            b.as_numeric().max(n.as_numeric()).into(),
                        )),
                    }
                }
            }
        }

        // Post condition
        for neighbor in self.iter() {
            if let Some(n) = neighbor {
                if n.is_valid() {
                    assert!(n.as_numeric() >= r.unwrap().0.as_numeric());
                    assert!(n.as_numeric() <= r.unwrap().1.as_numeric());
                }
            }
        }

        r
    }

    /// Count the number of tiles of type `x` in the neighborhood.
    pub fn count(&self, x: T) -> usize {
        self.iter()
            .map(|neighbor| match neighbor {
                Some(n) if n == x => 1,
                _ => 0,
            })
            .sum()
    }

    /// Iterate all neighors with their positions.
    /// Yields `None` for positions outside of the array area.
    pub fn iter_with_positions(&self) -> impl Iterator<Item = Option<(UVec2, T)>> + '_ {
        NeighborhoodIterator::new(&self)
    }

    /// Iterate tiles in the neighborhood.
    /// Yields `None` for positions outside of the array area.
    pub fn iter(&self) -> impl Iterator<Item = Option<T>> + '_ {
        self.iter_with_positions().map(|o| o.map(|(_p, v)| v))
    }

    /// All generated positions will be inside the map area and thus >= 0
    pub fn iter_positions(&self) -> impl Iterator<Item = UVec2> + '_ {
        self.iter_with_positions()
            .filter_map(|o| o.map(|(p, _v)| p))
    }

    fn in_map_of_size(p: IVec2, size: UVec2) -> bool {
        p.x >= 0 && p.y >= 0 && p.x < (size.x as i32) && p.y < (size.y as i32)
    }

    fn in_map(&self, p: IVec2) -> bool {
        Self::in_map_of_size(p, self.size)
    }
}

const INVALID_OFFSET: IVec2 = IVec2::new(0, 0);
const FIRST_OFFSET: IVec2 = IVec2::new(0, 1);
const LAST_OFFSET: IVec2 = IVec2::new(-1, 0);

pub struct NeighborhoodIterator<'a, T>
where
    T: Tile,
{
    neighborhood: &'a Neighborhood<'a, T>,
    offset: IVec2,
}

impl<'a, T> NeighborhoodIterator<'a, T>
where
    T: Tile,
{
    pub fn new(neighborhood: &'a Neighborhood<'a, T>) -> Self {
        Self {
            neighborhood,
            offset: INVALID_OFFSET,
        }
    }
}

impl<'a, T> Iterator for NeighborhoodIterator<'a, T>
where
    T: Tile,
{
    /// None means "outside of map"
    type Item = Option<(UVec2, T)>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut o = self.offset;

        if o == LAST_OFFSET {
            return None;
        }

        o = if o == INVALID_OFFSET {
            FIRST_OFFSET
        } else {
            // Rotate by 90 degrees (CW in a RH CS)
            // 0, 1 -> 1, 0 -> 0, -1 -> -1, 0
            ivec2(o.y, -o.x)
            // TODO: Actually want (optional) 45
            // Due to the rescaling this is actually not a rotation, but we need to do a case
            // distinction
        };

        self.offset = o;

        let p = self.neighborhood.position + o;
        Some(self.neighborhood.get(o).map(|t| (p.as_uvec2(), t) ))
    }
}
