pub mod envelope;
pub mod instrument;
pub mod note;
pub mod oscillators;
pub mod pan;
pub mod parameter;

pub mod prelude {
    pub use super::{envelope::*, instrument::*, note::*, oscillators::*, pan::*, parameter::*};
    pub use rodio::{self, Source};
}
