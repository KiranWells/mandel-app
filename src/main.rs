#![feature(portable_simd)]
#![feature(try_blocks)]
#![feature(decl_macro)]

mod image_generator;
mod interface;
mod mandel;
mod renderview;

use std::sync::Arc;

use druid::{AppLauncher, Data, Lens, PlatformError, WindowDesc};

use mandel::{MandelParameters};

#[derive(Clone, Data)]
pub enum FractalSettings {
    Mandel(MandelParameters),
    _Julia(MandelParameters),
}

#[derive(Clone, Data, Lens)]
pub struct AppData {
    pub preview_downscaling: bool,
    pub settings: FractalSettings,
    pub output_width: usize,
    pub output_height: usize,
    pub filename: String,
    pub log_text: String,
    pub rendering_image: Option<Arc<MandelParameters>>,
}

impl TryFrom<AppData> for MandelParameters {
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
        settings: FractalSettings::Mandel(MandelParameters {
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
    };
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
}
