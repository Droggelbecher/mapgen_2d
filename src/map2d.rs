
/*
use glam::UVec2;
use ndarray::Array2;
use crate::coord::UCoord2Conversions;

pub trait Map2d {
    type Element;

    fn size(&self) -> UVec2;
    fn get(&self, position: UVec2) -> Self::Element;
    fn set(&mut self, position: UVec2, element: Self::Element);
}

impl<T> Map2d for Array2<T> {

    type Element = T;

    fn size(&self) -> UVec2 { self.dim().as_uvec2() }

    fn get(&self, position: UVec2) -> Self::Element {
        self[position.as_index2()]
    }

    fn set(&mut self, position: UVec2, element: Self::Element) {
        self[position.as_index2()] = element;
    }
}
*/

//impl Map2d for Image {


