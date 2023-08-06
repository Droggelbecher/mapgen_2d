
use mapgen_2d::UCoord2Conversions;
use mapgen_2d::Blocks;
use ndarray::Array2;
use glam::uvec2;
use std::env::args;
use image::{GenericImageView, Rgba};

pub fn main() {
    /* WIP
    let block_size = uvec2(3, 3);

    let args: Vec<_> = args().collect();
    let input_path = args[1].as_str();

    // Copy image data into an Array2
    let im = image::open(input_path).unwrap();
    let mut arr = Array2::from_elem(
        im.dimensions().as_uvec2().as_index2(),
        Rgba([128u8, 128, 128, 255])
    );

    for x in 0..im.dimensions().0 {
        for y in 0..im.dimensions().1 {
            //arr[[x as usize, y as usize]] = if im.get_pixel(x, y)[0] > 100 { 1_u32 } else { 0_u32 };
            arr[[x as usize, y as usize]] = im.get_pixel(x, y); //[0] > 100 { 1_u32 } else { 0_u32 };
        }
    }

    let blockwise = Blocks::new(arr, block_size);
    */
}

