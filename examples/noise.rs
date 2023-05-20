use glam::{uvec2, UVec2};
use image::{ImageBuffer, Rgb};
use ndarray::Axis;
use ndrustfft::Complex;

use mapgen_2d::{ColoredNoise, UCoord2Conversions};

fn generate_image(size: UVec2, filename: &str) {
    let generator = ColoredNoise {
        size,
        color: -2.0,
        seed: 1234,
    };

    let fnoise = generator.generate_frequencies();
    let mut img = ImageBuffer::new(fnoise.len_of(Axis(0)) as u32, fnoise.len_of(Axis(1)) as u32);
    let m = fnoise
        .fold(0.0.into(), |x: Complex<f64>, y| {
            x.norm().max(y.norm()).into()
        })
        .norm();
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let v = (fnoise[(x, y).as_index2()].norm() * 100.0 * 255.0 / m) as u8;
        *pixel = Rgb([v, v, v]);
    }
    img.save(format!("{}_f.png", filename)).unwrap();

    let mut img = ImageBuffer::new(size.x, size.y);
    let noise = generator.generate();
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let v = (noise[(x, y).as_index2()] * 255.0) as u8;
        *pixel = Rgb([v, v, v]);
    }
    img.save(filename).unwrap();
}

pub fn main() {
    for sz in [100, 200, 300, 400, 500, 600] {
        generate_image(uvec2(sz, sz), &format!("noise{}.png", sz));
    }
}
