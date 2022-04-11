#![feature(portable_simd)]
#![feature(try_blocks)]

mod image_generator;
mod interface;
mod mandel;
mod renderview;

use std::sync::Arc;

use druid::{AppLauncher, Data, Lens, PlatformError, WindowDesc};
// use clipboard::{ClipboardProvider};

use image_generator::ImageGenerator;
use mandel::{ImageDescriptor, MandelGenerator};

#[derive(Clone, Data)]
pub enum FractalSettings {
    Mandel(ImageDescriptor),
    _Julia(ImageDescriptor),
}

#[derive(Clone, Data, Lens)]
pub struct AppData {
    // // image parameters
    // pub max_iter: usize,
    // pub zoom: f64,
    // pub offset_x: f64,
    // pub offset_y: f64,
    // // colors
    // pub saturation: f64,
    // pub color_frequency: f64,
    // pub color_offset: f64,
    // pub glow_spread: f64,
    // pub glow_strength: f64,
    // pub brightness: f64,
    // pub internal_brightness: f64,
    pub preview_downscaling: bool,
    pub settings: FractalSettings,

    // pub julia_constant: Option<(f64, f64)>,
    // app state
    pub output_width: usize,
    pub output_height: usize,
    pub filename: String,
    pub log_text: String,
    pub rendering_image: Option<Arc<MandelGenerator>>,
    // pub rendering_thread: Option<Arc<std::thread::JoinHandle<()>>>
}

impl TryFrom<AppData> for <MandelGenerator as ImageGenerator>::ImageDescriptor {
    type Error = ();
    fn try_from(val: AppData) -> Result<Self, Self::Error> {
        if let FractalSettings::Mandel(settings) = val.settings {
            Ok(settings.clone())
        } else {
            Err(())
        }
    }
}

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(interface::build_ui);
    let data = AppData {
        settings: FractalSettings::Mandel(ImageDescriptor {
            max_iter: 250,
            // scale: 4.0,
            zoom: -2.0,
            offset_x: -0.5,
            offset_y: 0.0,
            saturation: 1.0,
            color_frequency: 1.0,
            color_offset: 0.0,
            glow_spread: 1.0,
            glow_strength: 1.0,
            brightness: 2.0,
            internal_brightness: 1.0,
        }),
        output_width: 3840,
        output_height: 2160,
        filename: String::from("mandel.png"),
        log_text: String::new(),
        rendering_image: None,
        preview_downscaling: true,
        // rendering_thread: None,
    };
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
}
