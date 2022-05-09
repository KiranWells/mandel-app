use druid::{widget::prelude::*, Data, Point, Vec2, Widget, WidgetPod};

use crate::{backends::GeneratorParameters, types::FractalSettings, AppData};

use super::RenderView;

trait OffsetZoomMovement {
    fn offset(&mut self, offset: Vec2);
    fn get_zoom(&self) -> f64;
    fn offset_zoom(&mut self, offset: f64);
}

impl OffsetZoomMovement for AppData {
    fn offset(&mut self, offset: Vec2) {
        match &mut self.settings {
            FractalSettings::Mandel(inner) => {
                inner.offset_x -= offset.x;
                inner.offset_y -= offset.y;
            }
            FractalSettings::Julia(inner) => {
                inner.offset_x -= offset.x;
                inner.offset_y -= offset.y;
            }
        }
    }
    fn get_zoom(&self) -> f64 {
        match &self.settings {
            FractalSettings::Mandel(inner) => inner.zoom,
            FractalSettings::Julia(inner) => inner.zoom,
        }
    }
    fn offset_zoom(&mut self, offset: f64) {
        match &mut self.settings {
            FractalSettings::Mandel(inner) => {
                inner.zoom -= offset;
                inner.max_iter = (f64::powf(2.0, inner.zoom / 10.0) * 1000.0) as usize;
            }
            FractalSettings::Julia(inner) => {
                inner.zoom -= offset;
                inner.max_iter = (f64::powf(2.0, inner.zoom / 10.0) * 1000.0) as usize;
            }
        }
    }
}

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

impl<'a, GP> Widget<AppData> for ViewDragController<GP>
where
    GP: GeneratorParameters + Data + PartialEq,
    GP: TryFrom<AppData>,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &druid::Event, data: &mut AppData, env: &Env) {
        match event {
            Event::MouseDown(mouse_event) => {
                self.old_mouse_pos = Some(mouse_event.window_pos);
            }
            Event::MouseMove(mouse_event) => {
                if let Some(old_pos) = self.old_mouse_pos {
                    let difference = (mouse_event.window_pos - old_pos)
                        * f64::powf(2.0, -data.get_zoom())
                        * SENSITIVITY;
                    const SENSITIVITY: f64 = 0.0009;
                    data.offset(difference);
                    self.old_mouse_pos = Some(mouse_event.window_pos);
                    self.child.widget_mut().should_render = true;
                }
            }
            Event::MouseUp(_event) => {
                self.old_mouse_pos = None;
            }
            Event::Wheel(event) => {
                const SENSITIVITY: f64 = 0.003;
                data.offset_zoom(event.wheel_delta.y * SENSITIVITY);
                self.child.widget_mut().should_render = true;
            }
            _ => {
                let _ = data
                    .clone()
                    .try_into()
                    .map(|mut data| self.child.event(ctx, event, &mut data, env));
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
