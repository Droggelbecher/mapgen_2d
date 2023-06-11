use crate::coord::UCoord2Conversions;
use glam::{UVec2, uvec2};
use ndarray::{Array2, Dim};

/// In a 2d array with elements of type T,
/// this describes a region of the array in which all elements are equal to `reference`.
/// Note that the array is not owned or borrow by this structure but instead needs to be carried
/// along separately.
#[derive(Debug,Clone,Copy)]
pub struct Region<T>
where
    T: Eq + Copy,
{
    pub(crate) bounding_box: Rect,
    pub(crate) reference: T,
}

impl<T> Region<T>
where
    T: Eq + Copy,
{
    pub fn size(&self) -> UVec2 {
        self.bounding_box.size()
    }

    pub fn id(&self) -> T {
        self.reference
    }

    pub fn bounding_box(&self) -> Rect {
        self.bounding_box
    }

    pub fn top_left(&self) -> UVec2 {
        self.bounding_box.top_left()
    }

    pub fn bottom_right(&self) -> UVec2 {
        self.bounding_box.bottom_right()
    }

    pub fn iter_indices<'a>(&self, array: &'a Array2<T>) -> impl Iterator<Item = UVec2> + 'a {
        let r = self.reference;
        RectIterator::new(self.bounding_box).filter(move |p| array[p.as_index2()] == r)
    }

    pub fn iter_relative_indices<'a>(&self, array: &'a Array2<T>) -> impl Iterator<Item = UVec2> + 'a {
        let base = self.top_left();
        self.iter_indices(array)
            .map(move |p| p - base)
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Rect {
    //pub(crate) anchor: UVec2,
    //pub(crate) size: UVec2,
    top_left: UVec2,
    // inclusive
    bottom_right: UVec2,
}

// TODO: Write lots of tests for this
impl Rect {

    pub fn from_shape(shape: Dim<[usize; 2]>) -> Self {
        Self::from_size(uvec2(shape[0] as u32, shape[1] as u32))
    }

    // Includes bottom_right
    // TODO: Test!
    pub fn from_corners(top_left: UVec2, bottom_right: UVec2) -> Self {
        Self {
            top_left, bottom_right
            //anchor: top_left,
            //size: bottom_right - top_left + uvec2(1, 1),
        }
    }

    pub fn from_size(size: UVec2) -> Self {
        assert!(size.x != 0 && size.y != 0);
        Self {
            top_left: uvec2(0, 0),
            bottom_right: size - uvec2(1, 1),
        }
    }

    // Radius 0: Include exactly only center,
    // Radius 1: Include the 3x3 neighborhood around center
    // and so forth
    pub fn around(center: UVec2, radius: u32) -> Self {
        let top_left = uvec2(
                        center.x.saturating_sub(radius),
                        center.y.saturating_sub(radius)
                    );
        let bottom_right = uvec2(
            center.x.saturating_add(radius),
            center.y.saturating_add(radius),
        );
        Self::from_corners(top_left, bottom_right)
    }


    pub fn size(&self) -> UVec2 {
        self.bottom_right - self.top_left + uvec2(1, 1)
    }

    pub fn top_left(&self) -> UVec2 {
        self.top_left
    }

    pub fn bottom_right(&self) -> UVec2 {
        self.bottom_right
    }

    pub fn center(&self) -> UVec2 {
        self.top_left + self.size() / 2
    }

    pub fn grow_to_include(&mut self, pos: UVec2) {
        self.top_left = self.top_left.min(pos);
        self.bottom_right = self.bottom_right.max(pos);

        /*
        if pos.x < self.anchor.x {
            let delta = self.anchor.x - pos.x;
            self.anchor.x -= delta;
            self.size.x += delta;
        }
        if pos.y < self.anchor.y {
            let delta = self.anchor.y - pos.y;
            self.anchor.y -= delta;
            self.size.y += delta;
        }
        if pos.x >= self.anchor.x + self.size.x {
            let delta = pos.x - (self.anchor.x + self.size.x) + 1;
            self.size.x += delta;
        }
        if pos.y >= self.anchor.y + self.size.y {
            let delta = pos.y - (self.anchor.y + self.size.y) + 1;
            self.size.y += delta;
        }
        */
    }

    pub fn intersect(&self, other: Rect) -> Self {
        Self::from_corners(
            self.top_left().min(other.top_left()),
            self.bottom_right().min(other.bottom_right()),
        )
    }

    pub fn iter_indices(&self) -> impl Iterator<Item = UVec2> {
        RectIterator::new(*self)
    }
}

pub struct RectIterator {
    rect: Rect,
    next: UVec2,
}

impl RectIterator {
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            next: rect.top_left(),
        }
    }

    pub fn from_shape(shape: Dim<[usize; 2]>) -> Self {
        Self::new(Rect::from_shape(shape))
    }
}

impl Iterator for RectIterator {
    type Item = UVec2;

    fn next(&mut self) -> Option<Self::Item> {
        let r = self.next;
        if r.y > self.rect.bottom_right().y {
            return None;
        }

        self.next.x += 1;
        if self.next.x > self.rect.bottom_right().x {
            self.next.x = self.rect.top_left().x;
            self.next.y += 1;
        }

        Some(r)
    }
}
