use std::assert_matches::debug_assert_matches;

use druid::{
    piet::{ImageFormat, InterpolationMode},
    widget::prelude::*,
    Rect,
};

use crate::backends::{GeneratorParameters, ImageGenerator};

#[derive(Debug, PartialEq)]
enum RenderState {
    NotStarted,
    InProgress(f64),
    Canceled,
    Finished,
}

pub struct RenderView {
    image: ImageGenerator,
    state: RenderState,
    pub should_render: bool,
    pub should_resize: bool,
}

use RenderState::*;

impl RenderView {
    pub fn new(width: usize, height: usize) -> Self {
        RenderView {
            image: ImageGenerator::new(width, height),
            state: NotStarted,
            should_render: true,
            should_resize: false,
        }
    }

    fn finish(&mut self) {
        self.image.cancel_compute();
        self.state = Canceled;
    }

    /// Precondition: Requires self state to not be InProgress or Canceled
    fn render_new<GP: GeneratorParameters>(&mut self, settings: &GP) {
        debug_assert_matches!(self.state, NotStarted | Finished);
        let sent_settings = settings.clone();
        let mut sent_image = self.image.clone();
        let _ = std::thread::spawn(move || sent_image.do_compute(sent_settings, 8));

        self.state = InProgress(0.0);
        self.should_render = false;
    }

    /// Precondition: Requires self state to not be InProgress or Canceled
    fn resize(&mut self, new_size: &Size) {
        debug_assert_matches!(self.state, NotStarted | Finished);
        let &Size { width, height } = new_size;
        self.image = ImageGenerator::new(width as usize, height as usize);

        self.should_resize = false;
        self.should_render = true;
    }
}

impl<GP> Widget<GP> for RenderView
where
    GP: GeneratorParameters + Data + PartialEq,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut GP, _env: &Env) {
        match event {
            Event::AnimFrame(_) => {
                match self.state {
                    NotStarted | Finished => {
                        if self.should_resize {
                            self.resize(&ctx.size());
                        }
                        if self.should_render {
                            self.render_new(data);
                        }
                    }
                    InProgress(old_progress) => {
                        if self.should_render || self.should_resize {
                            self.finish();
                        } else {
                            let progress = self.image.get_progress();
                            if progress == 100.0 {
                                self.state = Finished;
                                ctx.request_paint();
                            } else if old_progress != progress {
                                self.state = InProgress(progress);
                                ctx.request_paint();
                            }
                        }
                    }
                    Canceled => {
                        let progress = self.image.get_progress();
                        if progress == 100.0 {
                            self.state = Finished;
                            ctx.request_paint();
                        }
                    }
                }

                ctx.request_anim_frame();
            }
            _ => {}
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _data: &GP, _env: &Env) {
        match event {
            LifeCycle::WidgetAdded => {
                ctx.request_anim_frame();
            }
            LifeCycle::Size(_) => self.should_resize = true,
            _ => {}
        }
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, old_data: &GP, data: &GP, _env: &Env) {
        if data != old_data {
            self.should_render = true;
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &GP, _env: &Env) -> Size {
        let max_size = bc.max();
        let window_size = ctx.window().get_size();
        let new_size = Size::new(
            max_size.width.min(window_size.width),
            max_size.height.min(window_size.height),
        );
        new_size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &GP, _env: &Env) {
        let image_ref = self.image.image_ref().lock();
        if let Ok(image) = image_ref {
            if image.width == 0 || image.height == 0 {
                return;
            }
            let drawable_image = ctx
                .make_image(
                    image.width,
                    image.height,
                    image.data.as_ref(),
                    ImageFormat::Rgb,
                )
                .unwrap();
            let size = ctx.size();
            ctx.draw_image(
                &drawable_image,
                Rect::from_origin_size((0.0, 0.0), size),
                InterpolationMode::NearestNeighbor,
            );
        }
    }
}
