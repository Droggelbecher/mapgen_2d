use ndarray::{Array2, Axis};
use ndrustfft::{ndifft, ndifft_r2c, Complex, FftHandler, R2cFftHandler};
use rand::{
    SeedableRng,
    distributions::{Distribution, Uniform}
};

// TODO: Configure with a struct/builder like voronoi
// TODO: Consider making this generic by using num traits and substituting `as` keyword with
// from/into calls
pub fn colored_noise(size_x: usize, size_y: usize, color: f64) -> Array2<f64> {
    let f_domain = generate_freq_domain_noise(size_x, size_y, color);

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

pub fn generate_freq_domain_noise(size_x: usize, size_y: usize, color: f64) -> Array2<Complex<f64>> {
    let mut f_domain: Array2<Complex<f64>> = Array2::zeros((size_x, size_y / 2 + 1));

    // TODO: Allow providing seed from outside
    let mut rng = rand::rngs::StdRng::seed_from_u64(1234);
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

