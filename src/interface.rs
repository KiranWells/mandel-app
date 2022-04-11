// use bitmap::generate_bitmap_image;
use druid::{
    text::format::ParseFormatter,
    widget::{
        Axis, Button, Checkbox, Controller, Flex, Label, LineBreaking, MainAxisAlignment, Scroll,
        Slider, Tabs, TabsTransition, TextBox, ValueTextBox,
    },
    Color, Data, Env, Event, EventCtx, FontDescriptor, FontFamily, FontWeight, Lens, Point,
    TextAlignment, Widget, WidgetExt,
};
// use image::
// use clipboard::{ClipboardProvider};

use crate::{
    image_generator::ImageGenerator,
    mandel::{ImageDescriptor, MandelGenerator},
    renderview::RenderView,
    AppData, FractalSettings,
};

struct AppDataToMandel {}

impl Lens<AppData, ImageDescriptor> for AppDataToMandel {
    fn with<V, F: FnOnce(&ImageDescriptor) -> V>(&self, data: &AppData, f: F) -> V {
        if let FractalSettings::Mandel(settings) = &data.settings {
            f(settings)
        } else {
            panic! {};
        }
    }

    fn with_mut<V, F: FnOnce(&mut ImageDescriptor) -> V>(&self, data: &mut AppData, f: F) -> V {
        if let FractalSettings::Mandel(settings) = &mut data.settings {
            f(settings)
        } else {
            panic! {};
        }
    }
}

pub fn build_ui() -> impl Widget<AppData> {
    Flex::row()
        .with_flex_child(
            RenderView::new(100, 100)
                .controller(ViewDragController::default())
                .lens(AppDataToMandel {}),
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
                        .with_transition(TabsTransition::Instant), // .lens(AppData::tab_name)
                    1.0,
                )
                .with_child(Label::new(
                    |data: &AppData, _env: &_| format! {"{}", data.log_text},
                )),
            0.25,
        )
}

fn render_full(_ctx: &mut EventCtx, data: &mut AppData, _env: &Env) {
    let mut render_image = MandelGenerator::new(data.output_width, data.output_height);
    let passable = data.clone();
    data.log_text = String::from("Render Started...");
    let _ = std::thread::spawn(move || {
        let filename = passable.filename.clone();
        render_image.do_compute(passable.try_into().unwrap(), num_cpus::get());
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

#[derive(Default)]
struct ViewDragController {
    old_mouse_pos: Option<Point>,
}

impl Controller<<MandelGenerator as ImageGenerator>::ImageDescriptor, RenderView<MandelGenerator>>
    for ViewDragController
{
    fn event(
        &mut self,
        child: &mut RenderView<MandelGenerator>,
        ctx: &mut EventCtx,
        event: &druid::Event,
        data: &mut <MandelGenerator as ImageGenerator>::ImageDescriptor,
        env: &Env,
    ) {
        match event {
            Event::MouseDown(mouse_event) => {
                self.old_mouse_pos = Some(mouse_event.window_pos);
            }
            Event::MouseMove(mouse_event) => {
                if let Some(old_pos) = self.old_mouse_pos {
                    let difference = mouse_event.window_pos - old_pos;
                    const SENSITIVITY: f64 = 0.0009;
                    data.offset_x -= difference.x * f64::powf(2.0, -data.zoom) * SENSITIVITY;
                    data.offset_y -= difference.y * f64::powf(2.0, -data.zoom) * SENSITIVITY;
                    self.old_mouse_pos = Some(mouse_event.window_pos);
                    child.should_render = true;
                }
            }
            Event::MouseUp(_event) => {
                self.old_mouse_pos = None;
            }
            Event::Wheel(event) => {
                const SENSITIVITY: f64 = 0.003;
                data.zoom -= event.wheel_delta.y * SENSITIVITY;
                // data.scale = f64::powf(2.0, -data.zoom);
                data.max_iter = (f64::powf(2.0, data.zoom / 10.0) * 1000.0) as usize;
                child.should_render = true;
            }
            _ => {
                child.event(ctx, event, data, env);
            }
        }
    }

    fn lifecycle(
        &mut self,
        child: &mut RenderView<MandelGenerator>,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &<MandelGenerator as ImageGenerator>::ImageDescriptor,
        env: &Env,
    ) {
        child.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        child: &mut RenderView<MandelGenerator>,
        ctx: &mut druid::UpdateCtx,
        old_data: &<MandelGenerator as ImageGenerator>::ImageDescriptor,
        data: &<MandelGenerator as ImageGenerator>::ImageDescriptor,
        env: &Env,
    ) {
        child.update(ctx, old_data, data, env)
    }
}

// fn save_to_clipboard(ctx: &mut EventCtx, data: &mut AppData, env: &Env) {
//     if let Ok(ctx) = ClipboardProvider::new() {
//         ctx.set_contents(data.to_string()).unwrap();
//     } else {
//         println!{"Failed to set clipboard!"};
//     }
// }

// fn get_from_clipboard(ctx: &mut EventCtx, data: &mut AppData, env: &Env) {
//     if let Ok(ctx) = ClipboardProvider::new() {
//         data = ctx.get_contents().unwrap().parse();
//     } else {
//         println!{"Failed to set clipboard!"};
//     }
// }

// #[derive(Data, Clone)]
// struct SidewaysPolicy {

// }

// impl TabsPolicy for SidewaysPolicy {
//     fn close_tab(&self, key: Self::Key, data: &mut Self::Input) {}

//     fn build(build: Self::Build) -> Self {
//         panic!("TabsPolicy::Build called on a policy that does not support incremental building")
//     }

//     fn default_make_label(info: druid::widget::TabInfo<Self::Input>) -> Label<Self::Input> {
//         Label::new(info.name).transform().with_text_color(druid::theme::FOREGROUND_LIGHT)
//     }

//     type Key;

//     type Input;

//     type BodyWidget;

//     type LabelWidget;

//     type Build;

//     fn tabs_changed(&self, old_data: &Self::Input, data: &Self::Input) -> bool {
//         todo!()
//     }

//     fn tabs(&self, data: &Self::Input) -> Vec<Self::Key> {
//         todo!()
//     }

//     fn tab_info(&self, key: Self::Key, data: &Self::Input) -> druid::widget::TabInfo<Self::Input> {
//         todo!()
//     }

//     fn tab_body(&self, key: Self::Key, data: &Self::Input) -> Self::BodyWidget {
//         todo!()
//     }

//     fn tab_label(
//         &self,
//         key: Self::Key,
//         info: druid::widget::TabInfo<Self::Input>,
//         data: &Self::Input,
//     ) -> Self::LabelWidget {
//         todo!()
//     }
// }

#[derive(Clone, Copy)]
enum AppView {
    Generation,
    Coloring,
    Rendering,
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
    // .expand()
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
        .with_child(create_option_label("Output Image Width"))
        .with_child(
            ValueTextBox::new(TextBox::new(), ParseFormatter::new())
                .lens(AppData::output_width)
                .align_left()
                .expand_width(),
        )
        .with_child(create_option_label("Output Image Height"))
        .with_child(
            ValueTextBox::new(TextBox::new(), ParseFormatter::new())
                .lens(AppData::output_height)
                .align_left()
                .expand_width(),
        )
        .with_child(create_option_label("Output Image Filename"))
        .with_child(
            ValueTextBox::new(TextBox::new(), ParseFormatter::new())
                .lens(AppData::filename)
                .expand_width(),
        )
        .with_child(
            Button::new("Render")
                .on_click(render_full)
                .padding((0.0, 7.0)),
        )
        // .with_child(RenderView::<MandelGenerator>::new(100, 100).lens(AppDataToMandel {}))
        .main_axis_alignment(MainAxisAlignment::Start)
}

fn create_generation_tab() -> impl Widget<AppData> {
    Flex::column()
        .with_child(
            Flex::<ImageDescriptor>::column()
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
                .with_child(create_option_label("Maximum Iterations"))
                .with_child(
                    ValueTextBox::new(TextBox::new(), ParseFormatter::new())
                        .lens(ImageDescriptor::max_iter)
                        .align_left()
                        .expand_width(),
                )
                .with_child(create_option_label("Zoom"))
                .with_child(create_input(-10.0, 50.0).lens(ImageDescriptor::zoom))
                .with_child(create_option_label("Offset"))
                .with_child(
                    Flex::row().with_child(Label::new("x:")).with_flex_child(
                        ValueTextBox::new(TextBox::new(), ParseFormatter::new())
                            .lens(ImageDescriptor::offset_x)
                            .expand_width(),
                        1.0,
                    ),
                )
                .with_spacer(5.0)
                .with_child(
                    Flex::row().with_child(Label::new("y:")).with_flex_child(
                        ValueTextBox::new(TextBox::new(), ParseFormatter::new())
                            .lens(ImageDescriptor::offset_y)
                            .expand_width(),
                        1.0,
                    ),
                )
                .lens(AppDataToMandel {}),
        )
        .with_child(create_option_label("Preview Scaling"))
        .with_child(Checkbox::new("Half scale preview").lens(AppData::preview_downscaling))
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
        .with_child(create_option_label("Saturation"))
        .with_child(create_input(0.0, 2.0).lens(ImageDescriptor::saturation))
        .with_child(create_option_label("Color Frequency"))
        .with_child(create_input(0.01, 10.0).lens(ImageDescriptor::color_frequency))
        .with_child(create_option_label("Color Offset"))
        .with_child(create_input(0.0, 1.0).lens(ImageDescriptor::color_offset))
        .with_child(create_option_label("Glow Spread"))
        .with_child(create_input(-10.0, 10.0).lens(ImageDescriptor::glow_spread))
        .with_child(create_option_label("Glow Strength"))
        .with_child(create_input(0.01, 10.0).lens(ImageDescriptor::glow_strength))
        .with_child(create_option_label("Brightness"))
        .with_child(create_input(0.1, 10.0).lens(ImageDescriptor::brightness))
        .with_child(create_option_label("Internal Brightness"))
        .with_child(create_input(0.1, 100.0).lens(ImageDescriptor::internal_brightness))
        .main_axis_alignment(MainAxisAlignment::Start)
        .lens(AppDataToMandel {})
}

fn create_option_label<T: Data>(text: &str) -> impl Widget<T> {
    Label::new(text)
        .with_font(
            FontDescriptor::new(FontFamily::SYSTEM_UI)
                .with_weight(FontWeight::BOLD)
                .with_size(15.0),
        )
        .padding((0.0, 5.0, 0.0, 10.0))
        .expand_width()
}

fn create_input(min: f64, max: f64) -> impl Widget<f64> {
    Flex::column()
        .with_child(
            ValueTextBox::new(TextBox::new(), ParseFormatter::new()),
            // 0.3,
        )
        .with_child(
            Slider::new()
                .with_range(min, max)
                .expand_width()
                .padding((0.0, 5.0)),
        )
        .expand_width()
}
