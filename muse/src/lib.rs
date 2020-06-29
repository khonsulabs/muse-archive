pub mod amplify;
pub mod device;
pub mod envelope;
pub mod instrument;
pub mod manager;
pub mod note;
pub mod oscillators;
pub mod pan;
pub mod parameter;
pub mod sampler;

pub use cpal;

pub mod prelude {
    pub use super::{
        amplify::*, cpal, envelope::*, instrument::*, note::*, oscillators::*, pan::*, parameter::*,
    };
}
