use crate::{
    coord::UCoord2Conversions,
    region::{Rect, RectIterator},
};
use glam::{ivec2, uvec2, IVec2, UVec2};
use ndarray::Array2;
use std::cmp::Ord;

pub struct NeighborPositions {
    pub size: UVec2,
    pub position: UVec2,
    pub radius: u32,
}

impl NeighborPositions {
    pub fn iter(&self) -> impl Iterator<Item = UVec2> {
        let pos = self.position;
        let radius = self.radius;
        let r = ivec2(radius as i32, radius as i32);
        let top_left = (pos.as_ivec2() - r).clamp(ivec2(0, 0), self.size.as_ivec2() - ivec2(1, 1));
        let bottom_right = (pos.as_ivec2() + r).clamp(ivec2(0, 0), self.size.as_ivec2() - ivec2(1, 1));

        RectIterator::new(Rect::from_corners(
            top_left.as_uvec2(),
            bottom_right.as_uvec2(),
        ))
        .filter(move |x| {
            *x != pos &&
            // Manhattan distance (no diagonal movement)
            manhattan(*x, pos) <= radius
        })
    }
}

fn manhattan(a: UVec2, b: UVec2) -> u32 {
    (a.x as i32 - b.x as i32).abs() as u32 + (a.y as i32 - b.y as i32).abs() as u32
}

// TODO: NeighborPositions is quite useful as it avoids multiply borrowing the array.
// Either get rid of Neighborhood or implement it in terms of NeighborPositions

/// Represents the 2d neighborhood around a tile located
/// at a certain positon in a given array.
/// Generally, methods here will refer to the tiles around the given
/// position, not including that tile itself.
pub struct Neighborhood<'a, T> {
    a: &'a Array2<T>,
    valid: Option<&'a Array2<bool>>,
    position: IVec2,
    size: UVec2,
}

impl<'a, T> Neighborhood<'a, T>
where
    T: Clone + Copy,
{
    /// Constructor.
    /// Note that position is signed, ie. it is allowed to be outside the array area.
    pub fn new(a: &'a Array2<T>, position: IVec2) -> Self {
        let size = uvec2(a.shape()[0] as u32, a.shape()[1] as u32);

        Self {
            position,
            a,
            size,
            valid: None,
        }
    }

    pub fn new_with_mask(a: &'a Array2<T>, mask: &'a Array2<bool>, position: IVec2) -> Self {
        let size = uvec2(a.shape()[0] as u32, a.shape()[1] as u32);

        Self {
            position,
            a,
            size,
            valid: Some(mask),
        }
    }

    pub fn position(&self) -> IVec2 {
        self.position
    }

    pub fn get(&self, offset: IVec2) -> Option<T> {
        assert!(offset.x >= -1 && offset.x <= 1);
        assert!(offset.y >= -1 && offset.y <= 1);

        let p = self.position + offset;
        match self.in_map(p) && self.is_valid(p) {
            true => Some(self.a[p.as_uvec2().as_index2()].into()),
            false => None,
        }
    }

    /*
    pub fn get_mut(&self, offset: IVec2) -> Option<&mut T> {
        assert!(offset.x >= -1 && offset.x <= 1);
        assert!(offset.y >= -1 && offset.y <= 1);

        let p = self.position + offset;
        match self.in_map(p) && self.is_valid(p) {
            true => Some(&mut self.a[p.as_uvec2().as_index2()]),
            false => None,
        }
    }
    */

    /// min/max tile value in the neighborhood.
    /// Ignore invalid tiles.
    /// If there are no valid tiles in the neighborhood, return `None`.
    pub fn min(&self) -> Option<T>
    where
        T: Ord,
    {
        self.iter().filter_map(|x| x).min()
    }

    /// min/max tile value in the neighborhood.
    /// Ignore invalid tiles.
    /// If there are no valid tiles in the neighborhood, return `None`.
    pub fn max(&self) -> Option<T>
    where
        T: Ord,
    {
        self.iter().filter_map(|x| x).max()
    }

    /// Count the number of tiles of type `x` in the neighborhood.
    pub fn count(&self, x: T) -> usize
    where
        T: Eq,
    {
        self.iter()
            .map(|neighbor| match neighbor {
                Some(n) if n == x => 1,
                _ => 0,
            })
            .sum()
    }

    pub fn has_only(&self, x: Vec<T>) -> bool
    where
        T: Eq,
    {
        self.iter()
            .filter_map(|neighbor| match neighbor {
                Some(n) => Some(x.contains(&n)),
                None => None,
            })
            .all(|x| x)
    }

    /// Iterate all neighors with their positions.
    /// Yields `None` for positions outside of the array area.
    pub fn iter_with_positions(&self) -> impl Iterator<Item = Option<(UVec2, T)>> + '_ {
        NeighborhoodIterator::new(&self)
    }

    /*
    pub fn iter_mut_with_positions(&'a mut self) -> impl Iterator<Item = Option<(UVec2, &'a mut T)>> + '_ {
        NeighborhoodIteratorMut::new(self)
    }
    */

    /// Iterate tiles in the neighborhood.
    /// Yields `None` for positions outside of the array area.
    pub fn iter(&self) -> impl Iterator<Item = Option<T>> + '_ {
        self.iter_with_positions().map(|o| o.map(|(_p, v)| v))
    }

    //pub fn iter_mut(&mut self) -> impl Iterator<Item = Option<&mut T>> + '_ {
    //self.iter_mut_with_positions().map(|o| o.map(|(_p, v)| v))
    //}

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

    fn is_valid(&self, p: IVec2) -> bool {
        match self.valid {
            None => true,
            Some(v) => v[p.as_uvec2().as_index2()],
        }
    }
}

const INVALID_OFFSET: IVec2 = IVec2::new(0, 0);
const FIRST_OFFSET: IVec2 = IVec2::new(0, 1);
const LAST_OFFSET: IVec2 = IVec2::new(-1, 0);

pub struct NeighborhoodIterator<'a, T> {
    neighborhood: &'a Neighborhood<'a, T>,
    offset: IVec2,
}

impl<'a, T> NeighborhoodIterator<'a, T> {
    pub fn new(neighborhood: &'a Neighborhood<'a, T>) -> Self {
        Self {
            neighborhood,
            offset: INVALID_OFFSET,
        }
    }
}

impl<'a, T> Iterator for NeighborhoodIterator<'a, T>
where
    T: Clone + Copy,
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
        Some(self.neighborhood.get(o).map(|t| (p.as_uvec2(), t)))
    }
}

/*
pub struct NeighborhoodIteratorMut<'a, T> {
    neighborhood: &'a mut Neighborhood<'a, T>,
    offset: IVec2,
}

impl<'a, T> NeighborhoodIteratorMut<'a, T> {
    pub fn new(neighborhood: &'a mut Neighborhood<'a, T>) -> Self {
        Self {
            neighborhood,
            offset: INVALID_OFFSET,
        }
    }
}

impl<'a, T> Iterator for NeighborhoodIteratorMut<'a, T>
where
    T: Clone + Copy,
{
    /// None means "outside of map"
    type Item = Option<(UVec2, &'a mut T)>;

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
        Some(self.neighborhood.get_mut(o).map(|mut t| (p.as_uvec2(), t)))
    }
}
*/
