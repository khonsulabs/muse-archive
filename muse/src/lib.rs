pub mod envelope;
pub mod instrument;
pub mod manager;
pub mod node;
mod note;
pub use note::*;
pub mod parameter;
pub mod sampler;

pub use cpal;

pub mod prelude {
    pub use super::{cpal, envelope::*, instrument::*, note::*, parameter::*, sampler::prelude::*};
}
