#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![deny(clippy::unwrap_used)]

mod autogen_schemes;
mod detect;
mod errors;
mod extensible_lipika;
mod lipika;
mod mapping;
mod numerals;
mod reshape;
mod scheme;
mod transliterate;
mod unicode_norm;
pub mod wasm;

#[cfg(feature = "vedic")]
pub mod vedic;

/// Extensions for runtime transliteration scheme modification
pub mod extensions;

pub use detect::detect;
pub use extensible_lipika::{ExtensibleLipika, ExtensibleTransliterationResult, ExtensionStats};
pub use lipika::Lipika;
pub use mapping::Mapping;
pub use scheme::Scheme;
pub use transliterate::transliterate;
