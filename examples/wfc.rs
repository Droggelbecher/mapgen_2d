use glam::uvec2;
use image::{ImageBuffer, Rgb};
use mapgen_2d::{Neighborhood, UCoord2Conversions, WaveFunctionCollapse};
use num_derive::FromPrimitive;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, FromPrimitive, PartialOrd, Ord, Hash)]
enum Color {
    #[default]
    NotSet = 0,
    White = 1,
    Black = 2,
}

impl From<Color> for usize {
    fn from(color: Color) -> usize {
        color as usize
    }
}

fn probability(neighbors: &Neighborhood<Color>) -> [f32; 3] {
    use Color::*;

    let mut ps = [0.0, 0.8, 0.2];

    if neighbors.count(Black) >= 2 {
        ps[Black as usize] = 1.0;
        ps[White as usize] = 0.0;
    }
    else if neighbors.count(Black) >= 1 {
        ps[Black as usize] = 0.8;
        ps[White as usize] = 0.2;
    }

    ps
}

pub fn main() {
    let result = WaveFunctionCollapse::new(uvec2(600, 600), 1234, probability)
        .neighborhood_size(1)
        .generate();

    let mut img = ImageBuffer::new(600, 600);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        *pixel = match result.tiles[(x, y).as_index2()] {
            Color::NotSet => Rgb([255u8, 0, 0]),
            Color::Black => Rgb([0u8, 0, 0]),
            Color::White => Rgb([255u8, 255, 255]),
        };
    }

    img.save("wfc.png").unwrap();
}
