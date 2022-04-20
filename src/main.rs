#![feature(portable_simd)]
#![feature(try_blocks)]
#![feature(decl_macro)]
#![feature(assert_matches)]

mod backends;
mod interface;
mod types;

use druid::{AppLauncher, PlatformError, WindowDesc};

use backends::MandelParameters;
use types::{AppData, FractalSettings};

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(interface::build_ui);
    let data = AppData {
        settings: FractalSettings::Mandel(MandelParameters::default()),
        output_width: 3840,
        output_height: 2160,
        filename: String::from("fractal.png"),
        log_text: String::new(),
        rendering_image: None,
        preview_downscaling: true,
    };
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
}
