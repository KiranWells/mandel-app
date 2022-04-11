use std::thread::JoinHandle;

use druid::{
    piet::{ImageFormat, InterpolationMode},
    widget::prelude::*,
    Rect,
};

use crate::image_generator::{ImageGenerator, GeneratorParameters};

pub struct RenderView {
    image: ImageGenerator,
    handle: Option<JoinHandle<()>>,
    old_progress: Option<f64>,
    pub scaling: f64,
    pub should_render: bool,
    pub should_resize: bool,
}

impl RenderView {
    pub fn new(width: usize, height: usize) -> Self {
        RenderView {
            image: ImageGenerator::new(width, height),
            handle: None,
            old_progress: None,
            should_render: true,
            should_resize: false,
            scaling: 0.5,
        }
    }

    fn render_new<GP: GeneratorParameters>(&mut self, settings: &GP) {
        if let Some(handle) = std::mem::replace(&mut self.handle, None) {
            handle.join().unwrap();
        }
        self.old_progress = Some(0.0);
        let sent = settings.clone();
        let mut sent_image = self.image.clone();
        self.handle = Some(std::thread::spawn(move || sent_image.do_compute(sent, 8)));
        self.should_render = false;
    }

    fn resize(&mut self, new_size: &Size) {
        if let Some(handle) = std::mem::replace(&mut self.handle, None) {
            handle.join().unwrap();
            self.old_progress = None;
        }
        let &Size { width, height } = new_size;
        println! {"Size: {:?}", new_size};
        self.image = ImageGenerator::new(
            (width * self.scaling) as usize,
            (height * self.scaling) as usize,
        );
        self.should_resize = false;
        self.should_render = true;
    }
}

impl<GP> Widget<GP> for RenderView
where GP: GeneratorParameters + Data + PartialEq {
    fn event(
        &mut self,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut GP,
        _env: &Env,
    ) {
        match event {
            Event::AnimFrame(x) => {
                if self.old_progress == Some(100.0) {
                    if let Some(handle) = std::mem::replace(&mut self.handle, None) {
                        // unsafe { image.finish_compute(handles) };
                        handle.join().unwrap();
                    }
                    self.old_progress = None;
                }
                // println! {"Anim frame: should_render: {}, old_progress: {:?}", self.should_render, self.old_progress};
                if self.should_render && self.old_progress == None {
                    self.render_new::<GP>(&data.clone().into());
                }
                // if self.should_recolor && self.old_progress == None {
                //     self.render_color(&data.clone().into());
                // }
                if x % 2 == 0 {
                    // println! {"Got 1/2 anim frame"};
                    let progress = self.image.get_progress();
                    if Some(progress) != self.old_progress {
                        self.old_progress = Some(progress);
                        // println! {"Progress: {progress}"};
                        ctx.request_paint()
                    }
                }
                ctx.request_anim_frame();
            }
            _ => {}
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        _data: &GP,
        _env: &Env,
    ) {
        match event {
            LifeCycle::WidgetAdded => {
                ctx.request_anim_frame();
            }
            LifeCycle::Size(new_size) => self.resize(new_size),
            _ => {}
        }
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx,
        old_data: &GP,
        data: &GP,
        _env: &Env,
    ) {
        if data != old_data {
            self.should_render = true;
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &GP,
        _env: &Env,
    ) -> Size {
        let max_size = bc.max();
        let window_size = ctx.window().get_size();
        let new_size = Size::new(
            max_size.width.min(window_size.width),
            max_size.height.min(window_size.height),
        );
        if self.should_resize {
            self.resize(&new_size);
        }
        new_size
        // if self.constrain_to_output {
        //     let aspect = data.output_height as f64 / data.output_width as f64;
        //     if expanded_size.height / aspect < max_size.width {
        //         Size::new(expanded_size.height / aspect, expanded_size.height)
        //     } else {
        //         Size::new(expanded_size.width, expanded_size.width * aspect)
        //     }
        // } else {
        //     expanded_size
        // }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &GP, _env: &Env) {
        let drawable_image = ctx
            .make_image(
                self.image.width(),
                self.image.height(),
                self.image.image_data(),
                ImageFormat::Rgb,
            )
            .unwrap();
        let size = ctx.size();
        ctx.draw_image(
            &drawable_image,
            Rect::from_origin_size((0.0, 0.0), size),
            InterpolationMode::Bilinear,
        );
    }
}
