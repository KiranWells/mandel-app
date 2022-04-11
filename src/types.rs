use std::sync::Arc;

use druid::{Data, Lens};

use crate::backends::MandelParameters;

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
