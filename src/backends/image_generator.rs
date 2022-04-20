use std::{
    cell::UnsafeCell,
    ptr,
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc,
    },
};

pub const LANES: usize = 4_usize;
pub const BYTES_PER_PIXEL: usize = 3_usize;

pub type Pixel = [u8; BYTES_PER_PIXEL];
pub type PixelCoord = (usize, usize);

pub trait GeneratorParameters: Clone + Send + 'static {
    type Intermediate;

    fn calc_pixel_row(
        &self,
        width: usize,
        height: usize,
        location: PixelCoord,
    ) -> [Self::Intermediate; 4];

    fn shade_pixel_row(&self, parameters: [Self::Intermediate; 4]) -> [Pixel; 4];

    fn needs_recompute(settings: &Self, old_settings: &Self) -> bool;
}

#[derive(Clone)]
pub struct ImageGenerator {
    pixels: Arc<Vec<u8>>,
    width: usize,
    height: usize,
    progress: Arc<AtomicI32>,
    canceled: Arc<AtomicBool>,
}

impl ImageGenerator {
    pub fn new(width: usize, height: usize) -> Self {
        let width = width + (LANES - width % LANES);
        ImageGenerator {
            pixels: Arc::new(vec![0; width * height * BYTES_PER_PIXEL]),
            width,
            height,
            progress: Arc::new(AtomicI32::new(1000)),
            canceled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// handles the dispatch of all threads
    pub fn do_compute<D: GeneratorParameters>(&mut self, settings: D, threads: usize) {
        self.progress.store(0, Ordering::Relaxed);
        self.canceled.store(false, Ordering::Release);

        (0..threads)
            .into_iter()
            .map(|t| {
                let passable_self = self.clone();
                let passable_settings = settings.clone();
                std::thread::spawn(move || unsafe {
                    if is_x86_feature_detected!("avx2") {
                        passable_self.calc_image_region_avx(passable_settings, threads, t);
                    } else {
                        passable_self.calc_image_region(passable_settings, threads, t);
                    }
                })
            })
            // force the threads to start by consuming the iterator
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|t| t.join().unwrap());
        self.progress.store(1000, Ordering::Release);
    }

    pub fn cancel_compute(&mut self) {
        self.canceled.store(true, Ordering::Release);
    }

    // getters
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get_progress(&self) -> f64 {
        self.progress.load(Ordering::SeqCst) as f64 / 10.0
    }

    pub fn image_data(&self) -> &[u8] {
        self.pixels.as_ref()
    }

    /// calculates one thread's portion of the image.
    /// also prints progress in the first thread (id=0)
    ///
    /// ### Safety
    /// Assumes it is running in parallel with a unique thread_id,
    /// calling multiple concurrent instances with the same thread_id
    /// may lead to data races
    unsafe fn calc_image_region<D: GeneratorParameters>(
        mut self,
        settings: D,
        threads: usize,
        thread_id: usize,
    ) {
        for j in (thread_id..self.height).step_by(threads) {
            // progress
            self.progress
                .store((j * 1000 / self.height) as i32, Ordering::Relaxed);
            // actual calculation
            for i in (0..self.width).step_by(LANES) {
                let intermediate = settings.calc_pixel_row(self.width, self.height, (i, j));
                let pixel = settings.shade_pixel_row(intermediate);
                self.write_pixel(i + j * self.width, pixel);
            }
            if self.canceled.load(Ordering::Acquire) {
                return;
            }
        }
    }

    #[target_feature(enable = "avx2")]
    unsafe fn calc_image_region_avx<D: GeneratorParameters>(
        self,
        settings: D,
        threads: usize,
        thread_id: usize,
    ) {
        self.calc_image_region(settings, threads, thread_id)
    }

    unsafe fn write_pixel(&mut self, x: usize, pixel: [Pixel; LANES]) {
        let mut data = UnsafeCell::new(self.pixels.as_ptr() as *mut Pixel); // as *mut [RgbaPixel; 4]);
        ptr::write(data.get_mut().offset(x as isize), pixel[0]);
        ptr::write(data.get_mut().offset((x + 1) as isize), pixel[1]);
        ptr::write(data.get_mut().offset((x + 2) as isize), pixel[2]);
        ptr::write(data.get_mut().offset((x + 3) as isize), pixel[3]);
    }
}
