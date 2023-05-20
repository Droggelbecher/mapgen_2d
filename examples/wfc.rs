use glam::uvec2;
use image::{ImageBuffer, Rgb};
use mapgen_2d::{Neighborhood, UCoord2Conversions, WaveFunctionCollapse};
use num_derive::FromPrimitive;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, FromPrimitive)]
enum Color {
    #[default]
    NotSet = 0,
    Red = 1,
    Orange = 2,
    Yellow = 3,
    Green = 4,
    Blue = 5,
}

impl From<Color> for usize {
    fn from(color: Color) -> usize {
        color as usize
    }
}

fn probability(neighbors: &Neighborhood<Color>) -> [f32; 6] {
    use Color::*;

    let mut ps = [0.0, 1.0, 1.0, 1.0, 1.0, 1.0];

    if neighbors.count(Red) > 2 {
        ps[3] = 0.0;
        ps[4] = 0.0;
        ps[5] = 0.0;
    }
    if neighbors.count(Orange) > 2 {
        ps[4] = 0.0;
        ps[5] = 0.0;
    }
    if neighbors.count(Yellow) > 2 {
        ps[1] = 0.0;
        ps[5] = 0.0;
    }
    if neighbors.count(Green) > 2 {
        ps[1] = 0.0;
        ps[2] = 0.0;
    }
    if neighbors.count(Blue) > 2 {
        ps[1] = 0.0;
        ps[2] = 0.0;
        ps[3] = 0.0;
    }

    ps
}

pub fn main() {
    let result = WaveFunctionCollapse::new(uvec2(100, 100), 1234, probability).generate();

    let mut img = ImageBuffer::new(100, 100);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        *pixel = match result.tiles[(x, y).as_index2()] {
            Color::NotSet => Rgb([0u8, 0, 0]),
            Color::Red => Rgb([255u8, 0, 0]),
            Color::Orange => Rgb([255u8, 128, 0]),
            Color::Yellow => Rgb([255u8, 255, 0]),
            Color::Green => Rgb([0u8, 255, 0]),
            Color::Blue => Rgb([0u8, 0, 255]),
        };
    }

    img.save("wfc.png").unwrap();
}
