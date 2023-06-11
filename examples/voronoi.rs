
use glam::uvec2;
use image::{ImageBuffer, Rgb};
use mapgen_2d::{UCoord2Conversions, Voronoi, VoronoiTile};

use rand::{
    distributions::{Distribution, Uniform},
    rngs::SmallRng, SeedableRng,
};

const N_CELLS: usize = 100;

pub fn main() {
    let result = Voronoi::new(uvec2(600, 600))
        .random_centers(N_CELLS)
        .border_coefficient(10.0)
        .generate();

    // Assign some random colors
    let mut rng = SmallRng::seed_from_u64(0);
    let uni = Uniform::<u8>::from(10 .. 240);
    let colors: Vec<_> = (0..N_CELLS).map(|_| {
        Rgb([
        uni.sample(&mut rng),
        uni.sample(&mut rng),
        uni.sample(&mut rng),
        ])
    }).collect();


    let mut img = ImageBuffer::new(600, 600);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let v = result.map[(x, y).as_index2()];

        if let VoronoiTile::Cell(cell) = v {
            *pixel = colors[cell.0];
        }
    }

    img.save("voronoi.png").unwrap();
}
