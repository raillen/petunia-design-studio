#![forbid(unsafe_code)]

//! Semantic color: owned [`ColorValue`] model with adapter-contained CMM math.
//!
//! The document stores semantic color and never roundtrips CMYK→RGB→CMYK for
//! storage. Display conversion is one-way and lossy by contract.

mod intent;
pub mod proof;
mod transform;
mod value;

pub use intent::RenderingIntent;
pub use proof::{
    ColorManagementProvider, DefaultColorManagementProvider, GamutStatus, PreserveNumbersPolicy,
    ProofContext,
};
pub use transform::{convert_for_display, transform_via_profile};
pub use value::{Cmyk, ColorValue, Lab, Srgb};

pub mod icc;
pub mod rgb_profiles;
pub use icc::{
    cmyk_to_cmyk, cmyk_to_rgb, parse_cmyk_token, rgb_to_cmyk, CmykDisplayTransform, IccColorSpace,
    IccProfile, IccProfileId, IccProofSettings, IccTransformOptions,
};
