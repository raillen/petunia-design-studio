#![forbid(unsafe_code)]

//! Semantic color: owned [`ColorValue`] model with adapter-contained CMM math.
//!
//! The document stores semantic color and never roundtrips CMYK→RGB→CMYK for
//! storage. Display conversion is one-way and lossy by contract.

mod intent;
mod transform;
mod value;

pub use intent::RenderingIntent;
pub use transform::{convert_for_display, transform_via_profile};
pub use value::{Cmyk, ColorValue, Lab, Srgb};
