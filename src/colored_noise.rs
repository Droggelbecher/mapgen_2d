//! Utilities for generating "colored" noise.

use ndarray::{Array2, Axis};
use ndrustfft::{ndifft, ndifft_r2c, Complex, FftHandler, R2cFftHandler};
use rand::{
    SeedableRng,
    distributions::{Distribution, Uniform}
};
use glam::UVec2;
use rand::rngs::StdRng;

/// Generate two-dimensional noise with a power spectral density per unit of bandwidth of `f^color`.
/// See e.g. https://en.wikipedia.org/wiki/Colors_of_noise .
/// That is for Brownian or "red" noise, set `color = -2.0`.
pub struct ColoredNoise {
    /// Size of the map to generate
    pub size: UVec2,

    /// "Color" of nise (exponent to frequency)
    pub color: f64,

    /// Random seed to use
    pub seed: u64,
}

impl Default for ColoredNoise {
    fn default() -> Self {
        Self {
            size: UVec2::new(100, 100),
            color: -2.0,
            seed: 1
        }
    }
}

impl ColoredNoise {

    // TODO: Consider making this generic by using num traits and substituting `as` keyword with
    // from/into calls

    /// Generate a 2d array of size `self.size` of noise with color `self.color`.
    pub fn generate(&self) -> Array2<f64> {
        let f_domain = self.generate_frequencies();

        let size_x = self.size.x as usize;
        let size_y = self.size.y as usize;

        let mut handler_ax0 = FftHandler::<f64>::new(size_x);
        let mut handler_ax1 = R2cFftHandler::<f64>::new(size_y);

        // TODO: Allow providing this from outside
        let mut r: Array2<f64> = Array2::zeros((size_x, size_y));
        {
            let mut work: Array2<Complex<f64>> = Array2::zeros((size_x, size_y / 2 + 1));
            ndifft(&f_domain, &mut work, &mut handler_ax0, 0);
            ndifft_r2c(&work, &mut r, &mut handler_ax1, 1);
        }

        r.mapv_inplace(|x| x.abs());

        let max = *r.iter().max_by(|x, y| x.partial_cmp(y).unwrap()).unwrap();
        let min = *r.iter().min_by(|x, y| x.partial_cmp(y).unwrap()).unwrap();
        let d = max - min;

        // Normalize to [0, 1]
        // This will leave exactly one element be 1.0 which is usually undesirable
        r.mapv_inplace(|x| (x - min) / d);
        // Replace the 1.0 element with 1.0-eps so that we have values in [0, 1) now.
        r.mapv_inplace(|x| if x >= 1.0 { 1.0 - f64::EPSILON } else { x });

        r
    }

    /// Generate the frequency domain part of the noise described by `self`.
    /// This will be called by `generate`. Usually you don't need to use this directly,
    /// but it can be useful for debugging and visualization.
    pub fn generate_frequencies(&self) -> Array2<Complex<f64>> {
        let size_x = self.size.x as usize;
        let size_y = self.size.y as usize;
        let color = self.color;
        let mut rng = StdRng::seed_from_u64(self.seed);

        let mut f_domain: Array2<Complex<f64>> = Array2::zeros((size_x, size_y / 2 + 1));

        let uniform = Uniform::<f64>::from(-1. ..1.);
        let cx = (size_x as f64) / 2.;
        let cy = (size_y as f64) / 2.;

        for x in 0..f_domain.len_of(Axis(0)) {
            for y in 0..f_domain.len_of(Axis(1)) {
                let distance = ((x as f64 - cx).powf(2.) + (y as f64 - cy).powf(2.)).sqrt();
                let weight = if distance != 0.0 { distance.powf(color) } else { 0.0 };
                f_domain[[x, y]] =
                    Complex::new(uniform.sample(&mut rng), uniform.sample(&mut rng)) * weight;
            }
        }

        f_domain
    }

} // impl
