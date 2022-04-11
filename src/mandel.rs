// A mandelbrot generation program written in Rust
// designed to run as efficiently as possible for double precision.
//
// Author: Griffith Thomas
//
// Sources for any external functions are annotated in the code

use std::cell::UnsafeCell;
use std::f64::consts::PI;
use std::hint::unreachable_unchecked;
use std::ptr;
use std::simd::{f64x4, Simd, StdFloat};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

use druid::{Data, Lens};

use crate::image_generator::ImageGenerator;

#[derive(Clone)]
pub struct MandelGenerator {
    buffer: Arc<Vec<[f64; 4]>>,
    pixels: Arc<Vec<u8>>,
    width: usize,
    height: usize,
    progress: Arc<AtomicI32>,
}

#[derive(Clone, PartialEq, Data, Lens)]
pub struct ImageDescriptor {
    // image parameters
    pub max_iter: usize,
    pub zoom: f64,
    // pub scale: f64,
    pub offset_x: f64,
    pub offset_y: f64,
    // colors
    pub saturation: f64,
    pub color_frequency: f64,
    pub color_offset: f64,
    pub glow_spread: f64,
    pub glow_strength: f64,
    pub brightness: f64,
    pub internal_brightness: f64,
}

const BYTES_PER_PIXEL: usize = 3;

impl MandelGenerator {
    // pub fn change_progress(&mut self, new_progress: f32) {
    //     self.progress.store((new_progress * 10.0) as i32, Ordering::Relaxed);
    // }

    unsafe fn _write_buffer(&mut self, x: usize, y: usize, values: [f64; 4]) {
        let mut data = UnsafeCell::new(self.buffer.as_ptr() as *mut [f64; 4]);
        ptr::write(data.get_mut().offset((x + y * self.width) as isize), values);
    }

    unsafe fn write_pixel(&mut self, x: usize, rgb: [u8; BYTES_PER_PIXEL]) {
        let mut data = UnsafeCell::new(self.pixels.as_ptr() as *mut [u8; 3]);
        ptr::write(data.get_mut().offset((x) as isize), rgb);
    }
    // ptr::write(data.get_mut().offset(((x + y * self.width) * BYTES_PER_PIXEL) as isize + 2), rgb[0]);
    // ptr::write(data.get_mut().offset(((x + y * self.width) * BYTES_PER_PIXEL) as isize + 1), rgb[1]);
    // ptr::write(data.get_mut().offset(((x + y * self.width) * BYTES_PER_PIXEL) as isize + 0), rgb[2]);

    /// calculates one thread's portion of the image.
    /// also prints progress in the first thread (id=0)
    ///
    /// ### Safety
    /// Assumes it is running in parallel with a unique thread_id,
    /// calling multiple concurrent instances with the same thread_id
    /// may lead to data races
    #[target_feature(enable = "avx2")]
    unsafe fn calc_image_region(
        mut self,
        settings: ImageDescriptor,
        threads: usize,
        thread_id: usize,
    ) {
        for j in (thread_id..self.height).step_by(threads) {
            // progress
            self.progress
                .store((j * 1000 / self.height) as i32, Ordering::Relaxed);
            // actual calculation
            for i in (0..self.width).step_by(MM_ONES.lanes()) {
                let (extracted_step, extracted_r, extracted_dr, extracted_orbit) =
                    settings.calc_pixel_mm(self.width, self.height, i, j);

                for v in 0..extracted_step.len() {
                    // let data = [
                    //     extracted_step[v],
                    //     extracted_r[v],
                    //     extracted_dr[v],
                    //     extracted_orbit[v],
                    // ];
                    // self.write_buffer(i + v, j, data);
                    let rgb = settings.shade(
                        extracted_step[v],
                        extracted_r[v],
                        extracted_dr[v],
                        extracted_orbit[v],
                    );
                    self.write_pixel(i + j * self.width + v, rgb);
                }
            }
        }
    }

    /// calculates one thread's portion of the image.
    /// also prints progress in the first thread (id=0)
    ///
    /// ### Safety
    /// Assumes it is running in parallel with a unique thread_id,
    /// calling multiple concurrent instances with the same thread_id
    /// may lead to data races
    #[target_feature(enable = "avx2")]
    unsafe fn calc_image_colors(
        mut self,
        settings: ImageDescriptor,
        threads: usize,
        thread_id: usize,
    ) {
        for j in (thread_id..self.height).step_by(threads) {
            // progress
            self.progress
                .store((j * 1000 / self.height) as i32, Ordering::Relaxed);
            // actual calculation
            for i in 0..self.buffer.len() {
                let data = self.buffer.get_unchecked(i);
                let rgb = settings.shade(data[0], data[1], data[2], data[3]);
                self.write_pixel(i, rgb);
            }
            // for i in (0..self.width).step_by(MM_ONES.lanes()) {
            //     let (extracted_step, extracted_r, extracted_dr, extracted_orbit) =
            //         buffer.get_unchecked(index);

            //     for v in 0..extracted_step.len() {
            //         let rgb = settings.shade(
            //             extracted_step[v],
            //             extracted_r[v],
            //             extracted_dr[v],
            //             extracted_orbit[v],
            //         );
            //         self.write_pixel(i + v, j, rgb);
            //     }
            // }
        }
    }
}

impl ImageGenerator for MandelGenerator {
    type ImageDescriptor = ImageDescriptor;
    fn new(width: usize, height: usize) -> Self {
        let width = width + (4 - width % 4);
        MandelGenerator {
            // data: if width % 4 != 0 {
            //     let padding = 4 - width % 4;
            //     Arc::new(vec![0; width * height * BYTES_PER_PIXEL + padding])
            // } else {
            //     Arc::new(vec![0; width * height * BYTES_PER_PIXEL])
            // },
            buffer: Arc::new(vec![[0.0; 4]; width * height]),
            pixels: Arc::new(vec![0; width * height * BYTES_PER_PIXEL]),
            width,
            height,
            progress: Arc::new(AtomicI32::new(0)),
            // threads: Arc::new(Vec::new()),
        }
    }

    /// handles the dispatch of all threads
    fn do_compute(&mut self, settings: ImageDescriptor, threads: usize) {
        self.progress.store(0, Ordering::Relaxed);

        (0..threads)
            .into_iter()
            .map(|t| {
                let passable_self = self.clone();
                let passable_settings = settings.clone();
                std::thread::spawn(move || unsafe {
                    passable_self.calc_image_region(passable_settings, threads, t)
                })
            })
            // force the threads to start by consuming the iterator
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|t| t.join().unwrap());
        self.progress.store(1000, Ordering::SeqCst);
    }

    /// handles the dispatch of all threads
    fn do_composite(&mut self, settings: ImageDescriptor, threads: usize) {
        self.progress.store(0, Ordering::Relaxed);

        (0..threads)
            .into_iter()
            .map(|t| {
                let passable_self = self.clone();
                let passable_settings = settings.clone();
                std::thread::spawn(move || unsafe {
                    passable_self.calc_image_colors(passable_settings, threads, t)
                })
            })
            // force the threads to start by consuming the iterator
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|t| t.join().unwrap());
        self.progress.store(1000, Ordering::SeqCst);
    }

    // pub unsafe fn finish_compute(&mut self, handles: Vec<JoinHandle<()>>) {
    //     handles.into_iter().for_each(|t| t.join().unwrap());
    //     self.progress.store(1000, Ordering::SeqCst);
    // }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn get_progress(&self) -> f64 {
        self.progress.load(Ordering::SeqCst) as f64 / 10.0
    }

    fn image_data(&self) -> &[u8] {
        self.pixels.as_ref()
    }

    fn needs_recompute(
        settings: &Self::ImageDescriptor,
        old_settings: &Self::ImageDescriptor,
    ) -> bool {
        return settings.max_iter != old_settings.max_iter
            || settings.zoom != old_settings.zoom
            || settings.offset_y != old_settings.offset_y
            || settings.offset_x != old_settings.offset_x;
    }
}

const MM_ONES: f64x4 = Simd::splat(1.0);
const MM_ZERO: f64x4 = Simd::splat(0.0);

impl ImageDescriptor {
    /// calculates the color for a pixel `[i,j]` of the image
    fn calc_pixel_mm(
        &self,
        width: usize,
        height: usize,
        i: usize,
        j: usize,
    ) -> ([f64; 4], [f64; 4], [f64; 4], [f64; 4]) {
        // TODO: add julia set capabilities

        // initialize values
        let scale = f64::powf(2.0, -self.zoom);

        // c: complex number
        let c_real = Simd::from_array([
            ((i + 0) as f64 / width as f64 - 0.5) * scale + self.offset_x,
            ((i + 1) as f64 / width as f64 - 0.5) * scale + self.offset_x,
            ((i + 2) as f64 / width as f64 - 0.5) * scale + self.offset_x,
            ((i + 3) as f64 / width as f64 - 0.5) * scale + self.offset_x,
        ]);

        let c_imag = Simd::splat(
            (j as f64 / height as f64 - 0.5) * scale * (height as f64 / width as f64)
                + self.offset_y,
        );

        // z: complex number
        let mut z_real = MM_ZERO;
        let mut z_imag = MM_ZERO;

        // z': complex running derivative
        let mut z_prime_r = MM_ONES;
        let mut z_prime_i = MM_ONES;

        // z^2: temporary value for optimized computation
        let mut real_2 = MM_ZERO;
        let mut imag_2 = MM_ZERO;

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
            // z' is calculated according to the standard formula (z' = 2*z*z' + 1):
            //   z'.r = 2 * (z.r * z'.r - z.i * z'.i) + 1
            //   z'.i = 2 * (z.i * z'.r + z.r * z'.i)

            let z_imag_tmp = (z_real + z_real) * z_imag + c_imag;
            let z_real_tmp = real_2 - imag_2 + c_real;

            // intermediate values for z'
            let ac_bd = z_real * z_prime_r - z_imag * z_prime_i;
            let bc_da = z_imag * z_prime_r + z_real * z_prime_i;

            let z_prime_r_tmp = ac_bd + ac_bd + MM_ONES;
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
        let extracted_orbit = orbit_acc.to_array();

        (extracted_step, extracted_r, extracted_dr, extracted_orbit)
    }

    fn shade(&self, step: f64, r: f64, dr: f64, orbit: f64) -> [u8; 3] {
        let scale = f64::powf(2.0, -self.zoom);
        // distance estimation: 0.5 * log(r) * r/dr
        let dist_est = 0.5 * (r).ln() * r / dr;
        // a 'glow' effect based on distance (manually adjusted to taste and to adjust to zoom level)
        let glow = (-(dist_est / scale).ln() + self.glow_spread) * self.glow_strength * 0.1;
        // a smoothed version of the iteration count: i + (1 - ln(ln(r))/ln(2))
        let smoothed_step = step + (1.0 - ((r).ln()).ln() / f64::ln(2.0));

        if step as usize >= self.max_iter {
            // color the inside using orbit trap method
            hsl2rgb(
                0.0,
                0.0,
                ((orbit).sqrt()
                    * self.brightness
                    * self.internal_brightness
                    * self.internal_brightness)
                    .clamp(0.0, 1.0),
            )
        } else {
            // color the outside
            hsl2rgb(
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
}

/// converts hsl to rgb, modified from
/// https://web.archive.org/web/20081227003853/http://mjijackson.com/2008/02/rgb-to-hsl-and-rgb-to-hsv-color-model-conversion-algorithms-in-javascript
fn hsl2rgb(h: f64, s: f64, v: f64) -> [u8; BYTES_PER_PIXEL] {
    let r;
    let g;
    let b;

    let i = (h * 6.).floor();
    let f = h * 6. - i;
    let p = v * (1. - s);
    let q = v * (1. - f * s);
    let t = v * (1. - (1. - f) * s);

    match (i % 6.0) as u8 {
        0 => {
            r = v;
            g = t;
            b = p;
        }
        1 => {
            r = q;
            g = v;
            b = p;
        }
        2 => {
            r = p;
            g = v;
            b = t;
        }
        3 => {
            r = p;
            g = q;
            b = v;
        }
        4 => {
            r = t;
            g = p;
            b = v;
        }
        5 => {
            r = v;
            g = p;
            b = q;
        }
        _ => unsafe { unreachable_unchecked() },
    }

    // This is an external function in the C version
    // this is just simpler
    [(r * 255.) as u8, (g * 255.) as u8, (b * 255.) as u8]
}
