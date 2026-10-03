//! Soft-proofing, out-of-gamut detection, and CMYK number preservation policies.
//!
//! Soft-proofing provides a render-time simulation of press output and gamut limitations
//! without altering the canonical color values stored in the document.

use crate::{Cmyk, ColorValue, RenderingIntent, Srgb};
use serde::{Deserialize, Serialize};

/// Context parameters for soft-proofing simulation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProofContext {
    /// Target simulation profile identifier (e.g., "US Web Coated (SWOP) v2", "FOGRA39").
    pub simulation_profile: String,
    /// Rendering intent for gamut mapping.
    pub intent: RenderingIntent,
    /// Whether out-of-gamut pixels should be highlighted.
    pub gamut_warning: bool,
    /// Whether to simulate substrate / paper white tint.
    pub simulate_paper_white: bool,
    /// Whether black-point compensation is enabled.
    pub black_point_compensation: bool,
}

impl Default for ProofContext {
    fn default() -> Self {
        Self {
            simulation_profile: "US Web Coated (SWOP) v2".to_string(),
            intent: RenderingIntent::RelativeColorimetric,
            gamut_warning: false,
            simulate_paper_white: false,
            black_point_compensation: true,
        }
    }
}

impl ProofContext {
    /// Creates a proofing context for a target ICC profile name.
    #[must_use]
    pub fn for_profile(profile_name: impl Into<String>) -> Self {
        Self {
            simulation_profile: profile_name.into(),
            ..Default::default()
        }
    }

    /// Enables the out-of-gamut warning flag.
    #[must_use]
    pub fn with_gamut_warning(mut self, enabled: bool) -> Self {
        self.gamut_warning = enabled;
        self
    }
}

/// Result of evaluating whether a color is within the gamut volume of a target medium.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GamutStatus {
    /// No gamut claim can be made without a loaded profile/checked transform.
    Unavailable { reason: String },
    /// Color can be accurately reproduced on target device.
    InGamut,
    /// Color cannot be physically reproduced; exceeds device gamut boundaries.
    OutOfGamut {
        /// Approximate perceptual color difference (Delta E).
        delta_e: f32,
        /// Nearest printable color approximation in sRGB.
        clamped_srgb: Srgb,
    },
}

/// Policy governing the preservation of authored CMYK channel percentages during conversion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreserveNumbersPolicy {
    /// Preserve all CMYK process channels literally (e.g. 100% C remains 100% C).
    #[default]
    PreserveAll,
    /// Preserve the Black (K) channel only (preventing single-color black text from becoming rich black).
    PreserveBlackOnly,
    /// Full appearance conversion through color transform.
    ConvertAll,
}

impl PreserveNumbersPolicy {
    /// Applies this preservation policy to a CMYK value for a target output profile.
    #[must_use]
    pub fn apply(&self, cmyk: Cmyk, _target_profile: &str) -> Cmyk {
        // The compatibility API accepts a name, not profile bytes. Preserve
        // channels rather than inventing conversions/TAC limits from that name.
        cmyk
    }
}

/// Unified Color Management Provider contract (09.9).
pub trait ColorManagementProvider: Send + Sync {
    /// Lists names of available standard profiles.
    fn available_profiles(&self) -> Vec<String>;

    /// Converts a color value to sRGB for normal display.
    fn convert_for_display(&self, value: &ColorValue, intent: RenderingIntent) -> Srgb;

    /// Evaluates a color under soft-proof simulation, returning simulated preview color and gamut status.
    fn soft_proof(&self, value: &ColorValue, context: &ProofContext) -> (Srgb, GamutStatus);

    /// Applies a CMYK number preservation policy.
    fn apply_cmyk_policy(&self, cmyk: Cmyk, policy: PreserveNumbersPolicy, profile: &str) -> Cmyk;
}

/// Compatibility provider for unprofiled metadata. Actual ICC proof uses `IccProofSettings`.
#[derive(Debug, Default)]
pub struct DefaultColorManagementProvider;

impl ColorManagementProvider for DefaultColorManagementProvider {
    fn available_profiles(&self) -> Vec<String> {
        vec!["sRGB IEC61966-2.1".to_string()]
    }

    fn convert_for_display(&self, value: &ColorValue, _intent: RenderingIntent) -> Srgb {
        value.to_srgb()
    }

    fn soft_proof(&self, value: &ColorValue, _context: &ProofContext) -> (Srgb, GamutStatus) {
        (
            value.to_srgb(),
            GamutStatus::Unavailable {
                reason:
                    "A profile name does not define an ICC proof; load press and monitor profiles"
                        .into(),
            },
        )
    }

    fn apply_cmyk_policy(&self, cmyk: Cmyk, policy: PreserveNumbersPolicy, profile: &str) -> Cmyk {
        policy.apply(cmyk, profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn named_profiles_cannot_modify_authored_inks() {
        let ink = Cmyk {
            c: 0.5,
            m: 0.7,
            y: 0.9,
            k: 1.0,
        };
        for policy in [
            PreserveNumbersPolicy::PreserveAll,
            PreserveNumbersPolicy::PreserveBlackOnly,
            PreserveNumbersPolicy::ConvertAll,
        ] {
            assert_eq!(policy.apply(ink, "FOGRA39"), ink);
        }
    }
    #[test]
    fn profile_names_do_not_invent_proof_or_gamut_measurements() {
        let original = Srgb::clamped(0., 1., 0.);
        let (display, status) = DefaultColorManagementProvider.soft_proof(
            &ColorValue::Rgb(original),
            &ProofContext::for_profile("SWOP").with_gamut_warning(true),
        );
        assert_eq!(display, original);
        assert!(matches!(status, GamutStatus::Unavailable { .. }));
        assert_eq!(
            DefaultColorManagementProvider.available_profiles(),
            vec!["sRGB IEC61966-2.1"]
        );
    }
}
