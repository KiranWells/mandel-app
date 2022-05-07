use druid::{Data, Lens};

use crate::{
    backends::{JuliaParameters, MandelParameters},
    AppData, FractalSettings,
};

#[derive(Clone, Copy, PartialEq, Eq, Data)]
pub enum FractalType {
    Mandel,
    Julia,
}

pub struct RadioLens {}
pub struct AppDataToJulia {}
pub struct AppDataToMandel {}

impl Lens<AppData, FractalType> for RadioLens {
    fn with<V, F: FnOnce(&FractalType) -> V>(&self, data: &AppData, f: F) -> V {
        match &data.settings {
            FractalSettings::Mandel(_) => f(&FractalType::Mandel),
            FractalSettings::Julia(_) => f(&FractalType::Julia),
        }
    }

    fn with_mut<V, F: FnOnce(&mut FractalType) -> V>(&self, data: &mut AppData, f: F) -> V {
        match &data.settings {
            FractalSettings::Mandel(mandel_settings) => {
                let mut frac = FractalType::Mandel;
                let out = f(&mut frac);
                if frac != FractalType::Mandel {
                    let mut new_julia = JuliaParameters::default();
                    new_julia.constant_real = mandel_settings.offset_x;
                    new_julia.constant_imag = mandel_settings.offset_y;
                    new_julia.zoom = mandel_settings.zoom / 2.0;
                    new_julia.max_iter = (f64::powf(2.0, new_julia.zoom / 10.0) * 1000.0) as usize;
                    data.settings = FractalSettings::Julia(new_julia);
                }
                out
            }
            FractalSettings::Julia(julia_settings) => {
                let mut frac = FractalType::Julia;
                let out = f(&mut frac);
                if frac != FractalType::Julia {
                    let mut new_mandel = MandelParameters::default();
                    new_mandel.offset_x = julia_settings.constant_real;
                    new_mandel.offset_y = julia_settings.constant_imag;
                    new_mandel.zoom = julia_settings.zoom * 2.0;
                    new_mandel.max_iter =
                        (f64::powf(2.0, new_mandel.zoom / 10.0) * 1000.0) as usize;
                    data.settings = FractalSettings::Mandel(new_mandel);
                }
                out
            }
        }
    }
}

impl Lens<AppData, MandelParameters> for AppDataToMandel {
    fn with<V, F: FnOnce(&MandelParameters) -> V>(&self, data: &AppData, f: F) -> V {
        if let FractalSettings::Mandel(settings) = &data.settings {
            f(settings)
        } else {
            panic! {};
        }
    }

    fn with_mut<V, F: FnOnce(&mut MandelParameters) -> V>(&self, data: &mut AppData, f: F) -> V {
        if let FractalSettings::Mandel(settings) = &mut data.settings {
            f(settings)
        } else {
            panic! {};
        }
    }
}

impl Lens<AppData, JuliaParameters> for AppDataToJulia {
    fn with<V, F: FnOnce(&JuliaParameters) -> V>(&self, data: &AppData, f: F) -> V {
        if let FractalSettings::Julia(settings) = &data.settings {
            f(settings)
        } else {
            panic! {};
        }
    }

    fn with_mut<V, F: FnOnce(&mut JuliaParameters) -> V>(&self, data: &mut AppData, f: F) -> V {
        if let FractalSettings::Julia(settings) = &mut data.settings {
            f(settings)
        } else {
            panic! {};
        }
    }
}
