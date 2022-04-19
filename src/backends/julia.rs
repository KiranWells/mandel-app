use std::f64::consts::PI;
use std::simd::{f64x4, Simd, StdFloat};

use druid::{Data, Lens};

use crate::image_generator::{GeneratorParameters, Pixel, BYTES_PER_PIXEL, LANES};

use super::hsl2rgb;

#[derive(Clone, PartialEq, Data, Lens)]
pub struct JuliaParameters {
    // image parameters
    pub max_iter: usize,
    pub zoom: f64,
    pub offset_x: f64,
    pub offset_y: f64,
    pub constant_real: f64,
    pub constant_imag: f64,
    // colors
    pub saturation: f64,
    pub color_frequency: f64,
    pub color_offset: f64,
    pub glow_spread: f64,
    pub glow_strength: f64,
    pub brightness: f64,
    pub internal_brightness: f64,
}

const MM_ONES: f64x4 = Simd::splat(1.0);
const MM_ZERO: f64x4 = Simd::splat(0.0);

impl GeneratorParameters for JuliaParameters {
    type Intermediate = [f64; 4];
    /// calculates the color for LANES number of pixels from `[i,j]` to `[i,j+LANES]` of the image
    fn calc_pixel_row(
        &self,
        width: usize,
        height: usize,
        (i, j): (usize, usize),
    ) -> [Self::Intermediate; LANES] {
        // initialize values
        let scale = f64::powf(2.0, -self.zoom);

        // c: complex number
        let c_real = Simd::splat(self.constant_real);
        let c_imag = Simd::splat(self.constant_imag);

        // z: complex number
        let mut z_real = Simd::from_array([
            ((i + 0) as f64 / width as f64 - 0.5) * scale + self.offset_x,
            ((i + 1) as f64 / width as f64 - 0.5) * scale + self.offset_x,
            ((i + 2) as f64 / width as f64 - 0.5) * scale + self.offset_x,
            ((i + 3) as f64 / width as f64 - 0.5) * scale + self.offset_x,
        ]);

        let mut z_imag = Simd::splat(
            (j as f64 / height as f64 - 0.5) * scale * (height as f64 / width as f64)
                + self.offset_y,
        );

        // z': complex running derivative
        let mut z_prime_r = MM_ONES;
        let mut z_prime_i = MM_ONES;

        // z^2: temporary value for optimized computation
        let mut real_2 = z_real * z_real;
        let mut imag_2 = z_imag * z_imag;

        // value accumulators for coloring
        let mut step_acc = MM_ZERO;
        let mut orbit_acc = MM_ONES;

        for _step in 0..self.max_iter {
            // iterate values, according to z = z^2 + c
            //
            // uses an optimized computation method from wikipedia for z:
            //   z.i := 2 × z.r × z.i + c.i
            //   z.r := r2 - i2 + c.r
            //   r2 := z.r × z.r
            //   i2 := z.i × z.i
            //
            // z' is calculated according to the standard formula (z' = 2*z*z'):
            //   z'.r = 2 * (z.r * z'.r - z.i * z'.i)
            //   z'.i = 2 * (z.i * z'.r + z.r * z'.i)

            let z_imag_tmp = (z_real + z_real) * z_imag + c_imag;
            let z_real_tmp = real_2 - imag_2 + c_real;

            // intermediate values for z'
            let ac_bd = z_real * z_prime_r - z_imag * z_prime_i;
            let bc_da = z_imag * z_prime_r + z_real * z_prime_i;

            let z_prime_r_tmp = ac_bd + ac_bd;
            let z_prime_i_tmp = bc_da + bc_da;

            let radius_2 = real_2 + imag_2;

            // select lanes which have not escaped
            // escape of 1000.0 used to smooth distance estimate
            let mask = radius_2.lanes_lt(Simd::splat(1000.0));

            // conditionally iterate, only if the pixel has not escaped
            z_real = mask.select(z_real_tmp, z_real);
            z_imag = mask.select(z_imag_tmp, z_imag);
            z_prime_i = mask.select(z_prime_i_tmp, z_prime_i);
            z_prime_r = mask.select(z_prime_r_tmp, z_prime_r);

            real_2 = z_real * z_real;
            imag_2 = z_imag * z_imag;

            step_acc = mask.select(MM_ONES, MM_ZERO) + step_acc;
            orbit_acc = orbit_acc.min(real_2 + imag_2);

            // finish if all pixels have escaped
            if !mask.any() {
                break;
            }
        }

        // calculate the absolute value (radius) of z for distance estimation
        let r = (real_2 + imag_2).sqrt();
        let dr = (z_prime_r * z_prime_r + z_prime_i * z_prime_i).sqrt();

        // extract values necessary for coloring
        let extracted_step = step_acc.to_array();
        let extracted_dr = dr.to_array();
        let extracted_r = r.to_array();
        let extracted_orbit = orbit_acc.sqrt().to_array();

        [extracted_step, extracted_r, extracted_dr, extracted_orbit]
    }

    fn shade_pixel_row(&self, parameters: [Self::Intermediate; LANES]) -> [Pixel; LANES] {
        let scale = f64::powf(2.0, -self.zoom);
        let mut row: [Pixel; LANES] = [[0; BYTES_PER_PIXEL]; LANES];
        for v in 0..parameters.len() {
            let step = parameters[0][v];
            let r = parameters[1][v];
            let dr = parameters[2][v];
            let orbit = parameters[3][v];
            // distance estimation: 0.5 * log(r) * r/dr
            let dist_est = 0.5 * (r).ln() * r / dr;
            // a 'glow' effect based on distance (manually adjusted to taste and to adjust to zoom level)
            let glow = (-(dist_est / scale).ln() + self.glow_spread) * self.glow_strength * 0.1;
            // a smoothed version of the iteration count: i + (1 - ln(ln(r))/ln(2))
            let smoothed_step = step + (1.0 - ((r).ln()).ln() / f64::ln(2.0));

            if step as usize >= self.max_iter {
                // color the inside using orbit trap method
                row[v] = hsl2rgb(
                    0.0,
                    0.0,
                    ((orbit)
                        * self.brightness
                        * self.internal_brightness
                        * self.internal_brightness)
                        .clamp(0.0, 1.0),
                )
            } else {
                // color the outside
                row[v] = hsl2rgb(
                    // color hue based on an sinusoidal step counter, offset to a [0,1] range
                    (((smoothed_step.ln() * self.color_frequency - self.color_offset * 2.0 * PI)
                        .sin())
                        * 0.5
                        + 0.5)
                        .clamp(0.0, 1.0),
                    // saturation decreased when glow is high to hide noise when hue oscillates quickly
                    (self.saturation * (1.0 - (glow * glow))).clamp(0.0, 1.0),
                    // use glow around edges for brightness
                    (glow * self.brightness).clamp(0.0, 1.0),
                )
            }
        }
        row
    }

    fn needs_recompute(settings: &Self, old_settings: &Self) -> bool {
        return settings.max_iter != old_settings.max_iter
            || settings.zoom != old_settings.zoom
            || settings.offset_y != old_settings.offset_y
            || settings.offset_x != old_settings.offset_x
            || settings.constant_real != old_settings.constant_real
            || settings.constant_imag != old_settings.constant_imag;
    }
}

impl Default for JuliaParameters {
    fn default() -> Self {
        Self {
            max_iter: 250,
            zoom: -2.0,
            offset_x: 0.0,
            offset_y: 0.0,
            constant_real: 0.15,
            constant_imag: -0.6,
            saturation: 1.0,
            color_frequency: 1.0,
            color_offset: 0.0,
            glow_spread: 1.0,
            glow_strength: 1.0,
            brightness: 2.0,
            internal_brightness: 1.0,
        }
    }
}
