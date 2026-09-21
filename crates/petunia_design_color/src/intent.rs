//! Rendering intents for display/proofing conversion.

use serde::{Deserialize, Serialize};

/// ICC-style rendering intent selector.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderingIntent {
    /// Preserve visual relationships (default for photo preview).
    #[default]
    Perceptual,
    /// Preserve in-gamut colors exactly, clip the rest.
    RelativeColorimetric,
    /// Map white point, then clip.
    Saturation,
    /// Preserve out-of-gamut relationships at hue cost.
    AbsoluteColorimetric,
}
