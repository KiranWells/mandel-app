mod image_generator;
mod julia;
mod mandel;
mod utilities;

pub use julia::JuliaParameters;
pub use mandel::MandelParameters;

pub use self::image_generator::{GeneratorParameters, ImageGenerator};
