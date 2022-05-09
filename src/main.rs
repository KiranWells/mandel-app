#![feature(portable_simd)]
#![feature(try_blocks)]
#![feature(decl_macro)]
#![feature(assert_matches)]

mod backends;
mod interface;
mod types;

use druid::{
    theme, AppLauncher, Color, Env, FontDescriptor, FontFamily, PlatformError, WindowDesc,
};

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
    };
    AppLauncher::with_window(main_window)
        .configure_env(configure)
        .use_simple_logger()
        .launch(data)
}

fn configure(env: &mut Env, _data: &AppData) {
    env.set(theme::BACKGROUND_DARK, Color::from_rgba32_u32(0x191e2aFF));
    env.set(theme::BACKGROUND_LIGHT, Color::from_rgba32_u32(0x212733FF));
    env.set(
        theme::WINDOW_BACKGROUND_COLOR,
        Color::from_rgba32_u32(0x191e2aFF),
    );
    env.set(theme::SELECTION_COLOR, Color::from_rgba32_u32(0xfad07bAA));
    env.set(
        theme::SELECTION_TEXT_COLOR,
        Color::from_rgba32_u32(0xd9d7ceFF),
    );
    env.set(theme::BORDER_DARK, Color::from_rgba32_u32(0x191e2aFF));
    env.set(theme::BORDER_LIGHT, Color::from_rgba32_u32(0x191e2aFF));
    env.set(theme::LABEL_COLOR, Color::from_rgba32_u32(0xd9d7ceFF));
    env.set(theme::FOREGROUND_DARK, Color::from_rgba32_u32(0xc7c7c7FF));
    env.set(theme::FOREGROUND_LIGHT, Color::from_rgba32_u32(0xd9d7ceFF));
    env.set(theme::PRIMARY_DARK, Color::from_rgba32_u32(0xfad07bFF));
    env.set(theme::PRIMARY_LIGHT, Color::from_rgba32_u32(0xffd580FF));
    env.set(theme::BUTTON_DARK, Color::from_rgba32_u32(0x212733FF));
    env.set(theme::BUTTON_LIGHT, Color::from_rgba32_u32(0x212733FF));
    env.set(theme::BUTTON_BORDER_RADIUS, 5.0);
    env.set(theme::BUTTON_BORDER_WIDTH, 0.0);
    env.set(theme::TEXTBOX_BORDER_RADIUS, 5.0);
    env.set(theme::TEXTBOX_BORDER_WIDTH, 0.0);
    env.set(theme::PROGRESS_BAR_RADIUS, 5.0);
    env.set(theme::SCROLLBAR_PAD, 2.0);
    env.set(
        theme::UI_FONT,
        FontDescriptor::new(FontFamily::SYSTEM_UI).with_size(16.0), // .with_weight(FontWeight::SEMI_BOLD),
    );
}
