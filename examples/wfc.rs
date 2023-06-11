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
    YGreen = 4,
    Green = 5,
    GBlue = 6,
    Blue = 7,
}

impl From<Color> for usize {
    fn from(color: Color) -> usize {
        color as usize
    }
}

fn probability(neighbors: &Neighborhood<Color>) -> [f32; 8] {
    use Color::*;

    let mut ps = [0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];


    if neighbors.count(Red) > 1 {
        ps[4] = 0.0;
        ps[5] = 0.0;
        ps[6] = 0.0;
        ps[7] = 0.0;
    }
    if neighbors.count(Orange) > 1 {
        ps[5] = 0.0;
        ps[6] = 0.0;
        ps[7] = 0.0;
    }
    if neighbors.count(Yellow) > 1 {
        ps[6] = 0.0;
        ps[7] = 0.0;
    }
    if neighbors.count(YGreen) > 1 {
        ps[1] = 0.0;
        ps[7] = 0.0;
    }
    if neighbors.count(Green) > 1 {
        ps[1] = 0.0;
        ps[2] = 0.0;
    }
    if neighbors.count(GBlue) > 1 {
        ps[1] = 0.0;
        ps[2] = 0.0;
        ps[3] = 0.0;
    }
    if neighbors.count(Blue) > 1 {
        ps[1] = 0.0;
        ps[2] = 0.0;
        ps[3] = 0.0;
        ps[4] = 0.0;
    }

    //if ps.iter().sum::<f32>() == 0.0 {
    //if neighbors.position().x <= 1
        //&& neighbors.position().y <= 1 {
    if ps.iter().sum::<f32>() < 1.0 {
        //ps[7] = 1.0;
        //println!("neigh {:?} {:?}", neighbors.position(), neighbors.iter().collect::<Vec<_>>());
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
            Color::YGreen => Rgb([128u8, 255, 0]),
            Color::Green => Rgb([0u8, 255, 0]),
            Color::GBlue => Rgb([0u8, 255, 255]),
            Color::Blue => Rgb([0u8, 0, 255]),
        };
    }

    img.save("wfc.png").unwrap();
}
