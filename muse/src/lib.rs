pub mod envelope;
pub mod instrument;
pub mod note;
pub mod oscillators;
pub mod parameter;

pub mod prelude {
    pub use super::{envelope::*, instrument::*, note::*, oscillators::*, parameter::*};
    pub use rodio::{self, Source};
}
