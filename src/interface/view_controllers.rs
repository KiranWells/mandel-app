use druid::{widget::prelude::*, Data, Point, Widget, WidgetPod};

use crate::{
    backends::{GeneratorParameters, JuliaParameters, MandelParameters},
    AppData, FractalSettings,
};

use super::RenderView;

pub struct ViewDragController<GP> {
    old_mouse_pos: Option<Point>,
    child: WidgetPod<GP, RenderView>,
}

impl<GP> ViewDragController<GP>
where
    GP: GeneratorParameters + Data + PartialEq,
{
    pub fn new() -> Self {
        ViewDragController {
            old_mouse_pos: None,
            child: WidgetPod::new(RenderView::new(100, 100)),
        }
    }
}

impl Widget<AppData> for ViewDragController<MandelParameters> {
    fn event(&mut self, ctx: &mut EventCtx, event: &druid::Event, data: &mut AppData, env: &Env) {
        if let FractalSettings::Mandel(data) = &mut data.settings {
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
                        self.child.widget_mut().should_render = true;
                    }
                }
                Event::MouseUp(_event) => {
                    self.old_mouse_pos = None;
                }
                Event::Wheel(event) => {
                    const SENSITIVITY: f64 = 0.003;
                    data.zoom -= event.wheel_delta.y * SENSITIVITY;
                    data.max_iter = (f64::powf(2.0, data.zoom / 10.0) * 1000.0) as usize;
                    self.child.widget_mut().should_render = true;
                }
                _ => {
                    self.child.event(ctx, event, data, env);
                }
            }
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &AppData,
        env: &Env,
    ) {
        let _ = data
            .clone()
            .try_into()
            .map(|data| self.child.lifecycle(ctx, event, &data, env));
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        old_data: &AppData,
        data: &AppData,
        env: &Env,
    ) {
        if data.preview_downscaling != old_data.preview_downscaling {
            self.child.widget_mut().scaling = if data.preview_downscaling { 0.6 } else { 1.0 };
            self.child.widget_mut().should_resize = true;
        }
        if data.output_height != old_data.output_height
            || data.output_width != old_data.output_width
        {
            self.child.widget_mut().should_resize = true;
        }
        let _ = data
            .clone()
            .try_into()
            .map(|data| self.child.update(ctx, &data, env));
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &AppData, env: &Env) {
        let _ = data
            .clone()
            .try_into()
            .map(|data| self.child.paint(ctx, &data, env));
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &AppData,
        env: &Env,
    ) -> druid::Size {
        data.clone()
            .try_into()
            .map(|data| self.child.layout(ctx, bc, &data, env))
            .unwrap_or(bc.min())
    }
}

impl Widget<AppData> for ViewDragController<JuliaParameters> {
    fn event(&mut self, ctx: &mut EventCtx, event: &druid::Event, data: &mut AppData, env: &Env) {
        if let FractalSettings::Julia(data) = &mut data.settings {
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
                        self.child.widget_mut().should_render = true;
                    }
                }
                Event::MouseUp(_event) => {
                    self.old_mouse_pos = None;
                }
                Event::Wheel(event) => {
                    const SENSITIVITY: f64 = 0.003;
                    data.zoom -= event.wheel_delta.y * SENSITIVITY;
                    data.max_iter = (f64::powf(2.0, data.zoom / 10.0) * 1000.0) as usize;
                    self.child.widget_mut().should_render = true;
                }
                _ => {
                    self.child.event(ctx, event, data, env);
                }
            }
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &AppData,
        env: &Env,
    ) {
        let _ = data
            .clone()
            .try_into()
            .map(|data| self.child.lifecycle(ctx, event, &data, env));
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        old_data: &AppData,
        data: &AppData,
        env: &Env,
    ) {
        if data.preview_downscaling != old_data.preview_downscaling {
            self.child.widget_mut().scaling = if data.preview_downscaling { 0.6 } else { 1.0 };
            self.child.widget_mut().should_resize = true;
            ctx.request_layout();
        }
        if data.output_height != old_data.output_height
            || data.output_width != old_data.output_width
        {
            self.child.widget_mut().should_resize = true;
            ctx.request_layout();
        }
        let _ = data
            .clone()
            .try_into()
            .map(|data| self.child.update(ctx, &data, env));
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &AppData, env: &Env) {
        let _ = data
            .clone()
            .try_into()
            .map(|data| self.child.paint(ctx, &data, env));
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &AppData,
        env: &Env,
    ) -> druid::Size {
        data.clone()
            .try_into()
            .map(|data| self.child.layout(ctx, bc, &data, env))
            .unwrap_or(bc.min())
    }
}

// TODO: render preview controller

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
