use std::{
    cell::UnsafeCell,
    ptr,
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc, Mutex,
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

pub struct ImageRef {
    pub data: Arc<Vec<u8>>,
    pub width: usize,
    pub height: usize,
}

#[derive(Clone)]
pub struct ImageGenerator {
    pixels: Arc<Vec<u8>>,
    width: usize,
    height: usize,
    scale: usize,
    current_width: usize,
    current_height: usize,
    progress: Arc<AtomicI32>,
    canceled: Arc<AtomicBool>,
    image_ref: Arc<Mutex<ImageRef>>,
}

impl ImageGenerator {
    pub fn new(width: usize, height: usize) -> Self {
        let mut new_ig = ImageGenerator {
            pixels: Arc::new(vec![]),
            width,
            height,
            current_width: 0,
            current_height: 0,
            progress: Arc::new(AtomicI32::new(1000)),
            canceled: Arc::new(AtomicBool::new(false)),
            scale: 16,
            image_ref: Arc::new(Mutex::new(ImageRef {
                data: Arc::new(vec![]),
                width: 0,
                height: 0,
            })),
        };
        new_ig.swap_pixel_buf();
        new_ig
    }

    fn swap_pixel_buf(&mut self) {
        let width = (self.width as f64 / self.scale as f64).ceil() as usize;
        let width = width + (LANES - width % LANES);
        let height = (self.height as f64 / self.scale as f64).ceil() as usize;
        if let Ok(mut im_ref) = self.image_ref.lock() {
            im_ref.data = std::mem::replace(
                &mut self.pixels,
                Arc::new(vec![0; width * height * BYTES_PER_PIXEL]),
            );
            im_ref.width = self.current_width;
            im_ref.height = self.current_height;
        }
        self.current_width = width;
        self.current_height = height;
    }

    /// handles the dispatch of all threads
    pub fn do_compute<D: GeneratorParameters>(&mut self, settings: D, threads: usize) {
        self.progress.store(0, Ordering::Relaxed);
        self.canceled.store(false, Ordering::Release);
        while self.scale >= 1 {
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
            if self.scale == 1 || self.canceled.load(Ordering::Relaxed) {
                break;
            }
            self.scale = self.scale / 2;
            self.swap_pixel_buf();
        }
        if self.scale == 16 || !self.canceled.load(Ordering::Relaxed) {
            if let Ok(mut im_ref) = self.image_ref.lock() {
                im_ref.data = self.pixels.clone();
                im_ref.width = self.current_width;
                im_ref.height = self.current_height;
            }
        }
        self.progress.store(1000, Ordering::Release);
    }

    pub fn cancel_compute(&mut self) {
        self.canceled.store(true, Ordering::Release);
    }

    // getters
    pub fn get_progress(&self) -> f64 {
        self.progress.load(Ordering::SeqCst) as f64 / 10.0
    }

    pub fn image_ref(&self) -> &Arc<Mutex<ImageRef>> {
        &self.image_ref
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
        for j in (thread_id..self.current_height).step_by(threads) {
            // progress
            self.progress
                .store((j * 1000 / self.height) as i32, Ordering::Relaxed);
            // actual calculation
            for i in (0..self.current_width).step_by(LANES) {
                // TODO: calculate only the *new* portion of the pixels,
                // to prevent doubling the render time
                // calculate new pixels
                let intermediate =
                    settings.calc_pixel_row(self.current_width, self.current_height, (i, j));
                let pixel = settings.shade_pixel_row(intermediate);
                self.write_pixel(i + j * self.current_width, pixel);
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
