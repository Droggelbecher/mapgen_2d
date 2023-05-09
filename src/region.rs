
use glam::{UVec2, uvec2};
use ndarray::Array2;

pub struct Region<T>
    where T: Eq+Copy
{
    pub(crate) anchor: UVec2,
    pub(crate) size: UVec2,
    pub(crate) reference: T,
    //pub(crate) a: &'a Array2<T>,
}

