use glam::uvec2;
use image::{ImageBuffer, Rgb};
use mapgen_2d::{Neighborhood, UCoord2Conversions, WaveFunctionCollapse};
use num_derive::FromPrimitive;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, FromPrimitive, PartialOrd, Ord, Hash)]
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

    if neighbors.count(Red) > 0 {
        ps[3] = 0.01;
        ps[4] = 0.01;
        ps[5] = 0.01;
    }
    if neighbors.count(Orange) > 0 {
        ps[4] = 0.01;
        ps[5] = 0.01;
    }
    if neighbors.count(Yellow) > 0 {
        ps[1] = 0.01;
        ps[5] = 0.01;
    }
    if neighbors.count(Green) > 0 {
        ps[1] = 0.01;
        ps[2] = 0.01;
    }
    if neighbors.count(Blue) > 0 {
        ps[1] = 0.01;
        ps[2] = 0.01;
        ps[3] = 0.01;
    }

    if let Some(c) = neighbors.most_common() {
        ps[c as usize] *= 10.0;
    }


    ps
}

pub fn main() {
    let result = WaveFunctionCollapse::new(uvec2(100, 100), 1234, probability)
        .neighborhood_size(1)
        .generate();

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
