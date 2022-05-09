use druid::{Data, Lens};

use crate::backends::{JuliaParameters, MandelParameters};

#[derive(Clone, Data, PartialEq)]
pub enum FractalSettings {
    Mandel(MandelParameters),
    Julia(JuliaParameters),
}

#[derive(Clone, Data, Lens)]
pub struct AppData {
    pub settings: FractalSettings,
    pub output_width: usize,
    pub output_height: usize,
    pub filename: String,
    pub log_text: String,
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
impl TryFrom<AppData> for JuliaParameters {
    type Error = ();
    fn try_from(val: AppData) -> Result<Self, Self::Error> {
        if let FractalSettings::Julia(settings) = val.settings {
            Ok(settings.clone())
        } else {
            Err(())
        }
    }
}
