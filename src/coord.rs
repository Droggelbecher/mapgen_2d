
//! Conversions for translating between (integral) coordinates such as `UVec2` and `IVec2`
//! and 2d array indices.

use num::Num;
use glam::{uvec2, UVec2, IVec2, ivec2};
use ndarray::{s, SliceInfo, SliceInfoElem, Dim};

/// Conversions to and from usizes for unsigned 2d coordinates
pub trait UCoord2 {
    type Ordinate : Num+Clone+Copy;

    fn new(x: Self::Ordinate, y: Self::Ordinate) -> Self;
    fn x(&self) -> Self::Ordinate;
    fn y(&self) -> Self::Ordinate;

    fn from_usizes(x: usize, y: usize) -> Self;
    fn x_usize(&self) -> usize;
    fn y_usize(&self) -> usize;
}

/// Conversions from usizes for unsigned 2d coordinates
pub trait ICoord2 {
    type Ordinate : Num+Clone+Copy;

    fn new(x: Self::Ordinate, y: Self::Ordinate) -> Self;
    fn x(&self) -> Self::Ordinate;
    fn y(&self) -> Self::Ordinate;

    fn from_usizes(x: usize, y: usize) -> Self;
}


impl UCoord2 for UVec2 {
    type Ordinate = u32;

    fn new(x: Self::Ordinate, y: Self::Ordinate) -> Self {
        UVec2::new(x, y)
    }

    fn from_usizes(x: usize, y: usize) -> Self {
        uvec2(x as u32, y as u32)
    }

    fn x(&self) -> Self::Ordinate { self.x }
    fn y(&self) -> Self::Ordinate { self.y }

    fn x_usize(&self) -> usize { self.x as usize }
    fn y_usize(&self) -> usize { self.y as usize }
}

impl UCoord2 for (usize, usize) {
    type Ordinate = usize;

    fn new(x: Self::Ordinate, y: Self::Ordinate) -> Self {
        (x, y)
    }

    fn from_usizes(x: usize, y: usize) -> Self {
        (x, y)
    }

    fn x(&self) -> Self::Ordinate { self.0 }
    fn y(&self) -> Self::Ordinate { self.1 }

    fn x_usize(&self) -> usize { self.0 }
    fn y_usize(&self) -> usize { self.1 }
}

impl UCoord2 for (u32, u32) {
    type Ordinate = u32;

    fn new(x: Self::Ordinate, y: Self::Ordinate) -> Self {
        (x, y)
    }

    fn from_usizes(x: usize, y: usize) -> Self {
        (x as u32, y as u32)
    }

    fn x(&self) -> Self::Ordinate { self.0 }
    fn y(&self) -> Self::Ordinate { self.1 }

    fn x_usize(&self) -> usize { self.0 as usize }
    fn y_usize(&self) -> usize { self.1 as usize }
}

impl UCoord2 for Dim<[usize; 2]> {
    type Ordinate = usize;

    fn new(x: Self::Ordinate, y: Self::Ordinate) -> Self {
        Dim([x as usize, y as usize])
    }

    fn from_usizes(x: usize, y: usize) -> Self {
        Dim([x, y])
    }

    fn x(&self) -> Self::Ordinate { self[0] }
    fn y(&self) -> Self::Ordinate { self[1] }

    fn x_usize(&self) -> usize { self[0] as usize }
    fn y_usize(&self) -> usize { self[1] as usize }
}

/// Conversions to indices, slices and uvec2/ivec2 for all implementers of `UCoord`.
/// Use like this:
///
/// ```
/// use mapgen_2d::UCoord2dConversions;
/// use ndarray::Array2;
/// use glam::uvec2;
///
/// let a = Array2::zeros((10, 10));
/// let pos = uvec2(3, 4);
/// a[pos.as_index2()]
/// ```
pub trait UCoord2Conversions {
    fn as_index2(&self) -> (usize, usize);
    fn as_index3(&self, third: usize) -> (usize, usize, usize);
    fn as_slice2d(&self) -> SliceInfo<[SliceInfoElem; 2], Dim<[usize; 2]>, Dim<[usize; 0]>>;
    fn as_slice3d(&self) -> SliceInfo<[SliceInfoElem; 3], Dim<[usize; 3]>, Dim<[usize; 1]>>;
    fn as_uvec2(&self) -> UVec2;
    fn as_ivec2(&self) -> IVec2;
}

impl<T> UCoord2Conversions for T
    where T: UCoord2
{

    fn as_uvec2(&self) -> UVec2 {
        uvec2(self.x_usize() as u32, self.y_usize() as u32)
    }

    fn as_ivec2(&self) -> IVec2 {
        ivec2(self.x_usize() as i32, self.y_usize() as i32)
    }

    fn as_index2(&self) -> (usize, usize) {
        (self.x_usize(), self.y_usize())
    }

    fn as_index3(&self, third: usize) -> (usize, usize, usize) {
        (self.x_usize(), self.y_usize(), third)
    }

    fn as_slice2d(&self) -> SliceInfo<[SliceInfoElem; 2], Dim<[usize; 2]>, Dim<[usize; 0]>> {
        s![self.x_usize(), self.y_usize()]
    }

    fn as_slice3d(&self) -> SliceInfo<[SliceInfoElem; 3], Dim<[usize; 3]>, Dim<[usize; 1]>> {
        s![self.x_usize(), self.y_usize(), ..]
    }

}

