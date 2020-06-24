pub mod envelope;
pub mod instrument;
pub mod oscillators;

pub mod prelude {
    pub use super::{envelope::*, instrument::*, oscillators::*};
    pub use kurbo;
    pub use pitch_calc;
    pub use rodio::{self, Source};
}
