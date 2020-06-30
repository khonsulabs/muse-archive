pub mod add;
pub mod amplify;
pub mod device;
pub mod envelope;
pub mod instrument;
pub mod manager;
pub mod max;
pub mod multiply;
pub mod note;
pub mod oscillators;
pub mod pan;
pub mod parameter;
pub mod sampler;

pub use cpal;

pub mod prelude {
    pub use super::{
        add::*, amplify::*, cpal, envelope::*, instrument::*, max::*, multiply::*, note::*,
        oscillators::*, pan::*, parameter::*, sampler::*,
    };
}
