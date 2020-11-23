// This warning doesn't apply to the usage in this project, as we're never using make_mut. https://github.com/rust-lang/rust-clippy/issues/6359
#![allow(clippy::rc_buffer)]

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
