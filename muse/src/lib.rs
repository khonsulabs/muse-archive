pub mod envelope;
pub mod instrument;
pub mod note;
pub mod oscillators;

pub mod prelude {
    pub use super::{envelope::*, instrument::*, note::*, oscillators::*};
    pub use kurbo;
    pub use rodio::{self, Source};
}
