use crate::coord::UCoord2Conversions;
use glam::UVec2;
use ndarray::Array2;

#[derive(Debug)]
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
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Rect {
    pub(crate) anchor: UVec2,
    pub(crate) size: UVec2,
}

impl Rect {
    pub fn size(&self) -> UVec2 {
        self.size
    }
    pub fn top_left(&self) -> UVec2 {
        self.anchor
    }
    pub fn bottom_right(&self) -> UVec2 {
        self.anchor + self.size
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
}

impl Iterator for RectIterator {
    type Item = UVec2;

    fn next(&mut self) -> Option<Self::Item> {
        let r = self.next;
        if r.y >= self.rect.bottom_right().y {
            return None;
        }

        self.next.x += 1;
        if self.next.x >= self.rect.bottom_right().x {
            self.next.x = self.rect.top_left().x;
            self.next.y += 1;
        }

        Some(r)
    }
}
