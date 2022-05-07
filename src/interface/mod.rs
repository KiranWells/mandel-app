use druid::{
    text::format::ParseFormatter,
    widget::{
        Axis, Button, Checkbox, Flex, Label, LineBreaking, MainAxisAlignment, RadioGroup, Scroll,
        Slider, Tabs, TabsTransition, TextBox, ValueTextBox, ViewSwitcher,
    },
    Color, Env, EventCtx, FontDescriptor, FontFamily, FontWeight, TextAlignment, Widget, WidgetExt,
};

use crate::{
    backends::{ImageGenerator, JuliaParameters, MandelParameters},
    AppData, FractalSettings,
};

mod lenses;
mod renderview;
mod view_controllers;

use lenses::*;
pub use renderview::RenderView;
use view_controllers::ViewDragController;

macro parameters_to_interface {
    ($struct:ty [ $( $option:tt ),+ ] ) => {
        Flex::column()
        $(
            .with_child(
                parameters_to_interface!{_inner; $struct, $option}
            )
        )+
    },
    (_inner; $struct:ty, ($member:ident: [$min:literal to $max:literal] $name:literal)) => {
        Flex::column()
            .with_child(
                parameters_to_interface!{_inner_label; $name}
            )
            .with_child(
                ValueTextBox::new(TextBox::new(), ParseFormatter::new()),
            )
            .with_child(
                Slider::new()
                    .with_range($min, $max)
                    .expand_width()
                    .padding((0.0, 5.0))
                    .env_scope(|env: &mut Env, _data: &_| {
                        env.set(druid::theme::BASIC_WIDGET_HEIGHT, 13.0);
                        env.set(druid::theme::BACKGROUND_DARK, Color::from_rgba32_u32(0x212733FF));
                    }),
            )
            .expand_width()
            .lens(<$struct>::$member)
    },
    (_inner; $struct:ty, ($member:ident: [  ] $name:literal $($attribute:ident),*)) => {
        Flex::column()
            .with_child(
                parameters_to_interface!{_inner_label; $name}
            )
            .with_child(
                ValueTextBox::new(TextBox::new(), ParseFormatter::new())
                    .lens(<$struct>::$member)
                    $(
                        .$attribute()
                    )?
                    .expand_width(),
            )
            .expand_width()
    },
    (_inner; $struct:ty, ($member:ident: [ x ] $name:literal)) => {
        Flex::column()
            .with_child(Checkbox::new($name)
                .lens(<$struct>::$member))
                .padding((0.0, 10.0, 0.0, 0.0))
    },
    (_inner_label; $name:literal) => {
        Label::new($name)
            .with_font(
                FontDescriptor::new(FontFamily::SYSTEM_UI)
                    .with_weight(FontWeight::BOLD)
                    .with_size(15.0),
            )
            .padding((0.0, 5.0, 0.0, 10.0))
            .expand_width()
    },
}

#[derive(Clone, Copy)]
enum AppView {
    Generation,
    Coloring,
    Rendering,
}

pub fn build_ui() -> impl Widget<AppData> {
    Flex::row()
        .with_flex_child(
            fractal_switcher(|fractal_settings| match fractal_settings {
                FractalType::Mandel => Box::new(ViewDragController::<MandelParameters>::new()),
                FractalType::Julia => Box::new(ViewDragController::<JuliaParameters>::new()),
            }),
            0.75,
        )
        .with_flex_child(
            Flex::column()
                .with_child(
                    Label::new("Mandelbrot")
                        .with_text_alignment(TextAlignment::Center)
                        .with_line_break_mode(LineBreaking::WordWrap)
                        .with_font(FontDescriptor::new(FontFamily::SYSTEM_UI).with_size(40.0)),
                )
                .with_flex_child(
                    Tabs::new()
                        .with_axis(Axis::Vertical)
                        .with_tab("Generation", select_view(AppView::Generation))
                        .with_tab("Coloring", select_view(AppView::Coloring))
                        .with_tab("Rendering", select_view(AppView::Rendering))
                        .with_transition(TabsTransition::Instant),
                    1.0,
                )
                .with_child(Label::new(
                    |data: &AppData, _env: &_| format! {"{}", data.log_text},
                )),
            0.25,
        )
}

fn select_view(view: AppView) -> impl Widget<AppData> {
    use AppView::*;
    Scroll::new(
        match view {
            Generation => create_generation_tab().expand_width(),
            Coloring => create_coloring_tab().expand_width(),
            Rendering => create_rendering_tab().expand_width(),
        }
        .padding(10.0),
    )
    .vertical()
}

fn create_rendering_tab() -> impl Widget<AppData> {
    Flex::column()
        .with_child(
            Label::new("Render")
                .with_font(FontDescriptor::new(FontFamily::SYSTEM_UI).with_size(20.0)),
        )
        .with_child(
            Flex::row()
                .border(Color::Rgba32(0xFFFFFFFF), 0.5)
                .expand_width()
                .padding(3.0),
        )
        .with_child(parameters_to_interface! {
            AppData
            [
                (output_width: [ ] "Output Image Width" align_left),
                (output_height: [ ] "Output Image Height" align_left),
                (filename: [ ] "Output Image Filename" align_left)
            ]
        })
        .with_child(
            Button::new("Render")
                .on_click(render_full)
                .padding((0.0, 7.0)),
        )
        // .with_child(RenderView::<MandelParameters>::new(100, 100).lens(AppDataToMandel {}))
        .main_axis_alignment(MainAxisAlignment::Start)
}

fn create_generation_tab() -> impl Widget<AppData> {
    Flex::column()
        .with_child(
            Flex::column()
                .with_child(
                    Label::new("Generation")
                        .with_font(FontDescriptor::new(FontFamily::SYSTEM_UI).with_size(20.0)),
                )
                .with_child(
                    Flex::row()
                        .border(Color::Rgba32(0xFFFFFFFF), 0.5)
                        .expand_width()
                        .padding(3.0),
                )
                .with_child(
                    RadioGroup::new(vec![
                        ("Mandelbrot set", FractalType::Mandel),
                        ("Julia set", FractalType::Julia),
                    ])
                    .env_scope(|env: &mut Env, _data: &_| {
                        env.set(
                            druid::theme::BACKGROUND_DARK,
                            Color::from_rgba32_u32(0x212733FF),
                        );
                    })
                    .lens(RadioLens {}),
                )
                .with_child(fractal_switcher(
                    |fractal_settings| match &fractal_settings {
                        FractalType::Mandel => Box::new(
                            parameters_to_interface! {
                                MandelParameters
                                [
                                    (max_iter: [ ] "Maximum Iterations" align_left),
                                    (zoom: [-10.0 to 50.0] "Zoom"),
                                    (offset_x: [ ] "Real Offset (x)" center),
                                    (offset_y: [ ] "Real Offset (y)" center)
                                ]
                            }
                            .lens(AppDataToMandel {}),
                        ),
                        FractalType::Julia => Box::new(
                            parameters_to_interface! {
                                JuliaParameters
                                [
                                    (max_iter: [ ] "Maximum Iterations" align_left),
                                    (zoom: [-10.0 to 50.0] "Zoom"),
                                    (offset_x: [ ] "Real Offset (x)" center),
                                    (offset_y: [ ] "Real Offset (y)" center),
                                    (constant_real: [-2.0 to 2.0] "Real value (x)"),
                                    (constant_imag: [-2.0 to 2.0] "Imaginary value (y)")
                                ]
                            }
                            .lens(AppDataToJulia {}),
                        ),
                    },
                ))
                .with_child(parameters_to_interface! {
                    AppData
                    [
                        (preview_downscaling: [ x ] "Half-scale preview")
                    ]
                }),
        )
        .main_axis_alignment(MainAxisAlignment::Start)
}

fn create_coloring_tab() -> impl Widget<AppData> {
    Flex::column()
        .with_child(
            Label::new("Coloring")
                .with_font(FontDescriptor::new(FontFamily::SYSTEM_UI).with_size(20.0)),
        )
        .with_child(
            Flex::row()
                .border(Color::Rgba32(0xFFFFFFFF), 0.5)
                .expand_width()
                .padding(3.0),
        )
        .with_child(fractal_switcher(
            |fractal_settings| match &fractal_settings {
                FractalType::Mandel => Box::new(
                    parameters_to_interface! {
                        MandelParameters
                        [
                            (saturation: [0.0 to 2.0] "Saturation"),
                            (color_frequency: [0.01 to 10.0] "Color Frequency"),
                            (color_offset: [0.0 to 1.0] "Color Offset"),
                            (glow_spread: [-10.0 to 10.0] "Glow Spread"),
                            (glow_strength: [0.01 to 10.0] "Glow Strength"),
                            (brightness: [0.01 to 10.0] "Brightness"),
                            (internal_brightness: [0.01 to 100.0] "Internal Brightness")
                        ]
                    }
                    .lens(AppDataToMandel {}),
                ),
                FractalType::Julia => Box::new(
                    parameters_to_interface! {
                        JuliaParameters
                        [
                            (saturation: [0.0 to 2.0] "Saturation"),
                            (color_frequency: [0.01 to 10.0] "Color Frequency"),
                            (color_offset: [0.0 to 1.0] "Color Offset"),
                            (glow_spread: [-10.0 to 10.0] "Glow Spread"),
                            (glow_strength: [0.01 to 10.0] "Glow Strength"),
                            (brightness: [0.01 to 10.0] "Brightness"),
                            (internal_brightness: [0.01 to 100.0] "Internal Brightness")
                        ]
                    }
                    .lens(AppDataToJulia {}),
                ),
            },
        ))
        .main_axis_alignment(MainAxisAlignment::Start)
}

fn fractal_switcher(
    switcher: impl Fn(&FractalType) -> Box<dyn Widget<AppData>> + 'static,
) -> impl Widget<AppData> {
    ViewSwitcher::new(
        |data: &AppData, _env| match data.settings {
            FractalSettings::Mandel(_) => FractalType::Mandel,
            FractalSettings::Julia(_) => FractalType::Julia,
        },
        move |fractal_settings, _data, _env| switcher(fractal_settings),
    )
}

fn render_full(_ctx: &mut EventCtx, data: &mut AppData, _env: &Env) {
    let mut render_image = ImageGenerator::new(data.output_width, data.output_height);
    let passable = data.clone();
    data.log_text = String::from("Render Started (done soon)");
    let _ = std::thread::spawn(move || {
        let filename = passable.filename.clone();
        match passable.settings {
            FractalSettings::Mandel(settings) => {
                render_image.do_compute::<MandelParameters>(settings, num_cpus::get())
            }
            FractalSettings::Julia(settings) => {
                render_image.do_compute::<JuliaParameters>(settings, num_cpus::get())
            }
        }
        image::save_buffer(
            filename.as_str(),
            render_image.image_data(),
            render_image.width() as u32,
            render_image.height() as u32,
            image::ColorType::Rgb8,
        )
        .unwrap()
    });
}
