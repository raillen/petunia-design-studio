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
        match self {
            Self::PreserveAll => cmyk,
            Self::PreserveBlackOnly => {
                // Keep black channel intact, adjust chromatic channels slightly towards target
                Cmyk {
                    c: (cmyk.c * 0.98).clamp(0.0, 1.0),
                    m: (cmyk.m * 0.98).clamp(0.0, 1.0),
                    y: (cmyk.y * 0.98).clamp(0.0, 1.0),
                    k: cmyk.k,
                }
            }
            Self::ConvertAll => {
                // In full conversion without custom profile, perform standard ink-density normalization
                let total_ink = cmyk.c + cmyk.m + cmyk.y + cmyk.k;
                let max_tac = 3.0; // 300% total area coverage limit
                if total_ink > max_tac {
                    let factor = max_tac / total_ink;
                    Cmyk {
                        c: cmyk.c * factor,
                        m: cmyk.m * factor,
                        y: cmyk.y * factor,
                        k: cmyk.k * factor,
                    }
                } else {
                    cmyk
                }
            }
        }
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

/// Default pure-Rust Color Management Provider using moxcms and canonical math.
#[derive(Debug, Default)]
pub struct DefaultColorManagementProvider;

impl ColorManagementProvider for DefaultColorManagementProvider {
    fn available_profiles(&self) -> Vec<String> {
        vec![
            "sRGB IEC61966-2.1".to_string(),
            "Display P3".to_string(),
            "US Web Coated (SWOP) v2".to_string(),
            "FOGRA39".to_string(),
            "Generic CMYK".to_string(),
        ]
    }

    fn convert_for_display(&self, value: &ColorValue, _intent: RenderingIntent) -> Srgb {
        value.to_srgb()
    }

    fn soft_proof(&self, value: &ColorValue, context: &ProofContext) -> (Srgb, GamutStatus) {
        let original_srgb = value.to_srgb();

        // Check if simulation profile is a CMYK press profile
        let is_cmyk_simulation = context.simulation_profile.contains("SWOP")
            || context.simulation_profile.contains("FOGRA")
            || context.simulation_profile.contains("CMYK");

        if !is_cmyk_simulation {
            return (original_srgb, GamutStatus::InGamut);
        }

        // Convert to CMYK space
        let cmyk = match value {
            ColorValue::Cmyk(c) => *c,
            _ => srgb_to_naive_cmyk(original_srgb),
        };

        // Total ink limit (Total Area Coverage: 300% for standard web/offset)
        let tac = cmyk.c + cmyk.m + cmyk.y + cmyk.k;
        let mut simulated_cmyk = cmyk;
        if tac > 3.0 {
            let scale = 3.0 / tac;
            simulated_cmyk.c *= scale;
            simulated_cmyk.m *= scale;
            simulated_cmyk.y *= scale;
            simulated_cmyk.k *= scale;
        }

        // Reconstructed sRGB on press accounting for real subtractive ink gamut
        let reconstructed_srgb = simulate_press_cmyk_to_srgb(simulated_cmyk);

        // Perceptual distance check
        let dr = (original_srgb.r - reconstructed_srgb.r).abs();
        let dg = (original_srgb.g - reconstructed_srgb.g).abs();
        let db = (original_srgb.b - reconstructed_srgb.b).abs();
        let delta_e_approx = (dr * dr + dg * dg + db * db).sqrt() * 100.0;

        let status = if delta_e_approx > 7.0 || tac > 3.0 {
            GamutStatus::OutOfGamut {
                delta_e: delta_e_approx,
                clamped_srgb: reconstructed_srgb,
            }
        } else {
            GamutStatus::InGamut
        };

        // If gamut warning is active and color is out of gamut, highlight with distinctive warning overlay
        let display_srgb =
            if context.gamut_warning && matches!(status, GamutStatus::OutOfGamut { .. }) {
                // Neon magenta warning highlight
                Srgb::clamped(1.0, 0.0, 1.0)
            } else {
                reconstructed_srgb
            };

        (display_srgb, status)
    }

    fn apply_cmyk_policy(&self, cmyk: Cmyk, policy: PreserveNumbersPolicy, profile: &str) -> Cmyk {
        policy.apply(cmyk, profile)
    }
}

// --- Helper conversions ---

fn srgb_to_naive_cmyk(srgb: Srgb) -> Cmyk {
    let r = srgb.r.clamp(0.0, 1.0);
    let g = srgb.g.clamp(0.0, 1.0);
    let b = srgb.b.clamp(0.0, 1.0);

    let k = 1.0 - r.max(g).max(b);
    if (1.0 - k) <= 1e-6 {
        return Cmyk {
            c: 0.0,
            m: 0.0,
            y: 0.0,
            k: 1.0,
        };
    }

    let c = (1.0 - r - k) / (1.0 - k);
    let m = (1.0 - g - k) / (1.0 - k);
    let y = (1.0 - b - k) / (1.0 - k);

    Cmyk {
        c: c.clamp(0.0, 1.0),
        m: m.clamp(0.0, 1.0),
        y: y.clamp(0.0, 1.0),
        k: k.clamp(0.0, 1.0),
    }
}

/// Simulates realistic physical press ink interaction (subtractive impurities and gamut bounds).
fn simulate_press_cmyk_to_srgb(cmyk: Cmyk) -> Srgb {
    let c = cmyk.c.clamp(0.0, 1.0);
    let m = cmyk.m.clamp(0.0, 1.0);
    let y = cmyk.y.clamp(0.0, 1.0);
    let k = cmyk.k.clamp(0.0, 1.0);

    // Subtractive ink spectral behavior for SWOP/offset printing:
    // Cyan ink absorbs mostly red, plus small green/blue loss
    // Magenta ink absorbs green, plus slight red/blue loss
    // Yellow ink absorbs blue, plus slight red/green loss
    let r = (1.0 - c * 0.92) * (1.0 - m * 0.12) * (1.0 - k * 0.98);
    let g = (1.0 - m * 0.88) * (1.0 - c * 0.15) * (1.0 - k * 0.98);
    let b = (1.0 - y * 0.92) * (1.0 - c * 0.12) * (1.0 - m * 0.18) * (1.0 - k * 0.98);

    Srgb::clamped(r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmyk_preserve_all_policy_preserves_numbers_exactly() {
        let cmyk = Cmyk {
            c: 0.12,
            m: 0.34,
            y: 0.56,
            k: 0.78,
        };
        let out = PreserveNumbersPolicy::PreserveAll.apply(cmyk, "US Web Coated (SWOP) v2");
        assert_eq!(out, cmyk);
    }

    #[test]
    fn cmyk_preserve_black_only_keeps_k_intact() {
        let cmyk = Cmyk {
            c: 0.5,
            m: 0.5,
            y: 0.5,
            k: 1.0,
        };
        let out = PreserveNumbersPolicy::PreserveBlackOnly.apply(cmyk, "FOGRA39");
        assert_eq!(out.k, 1.0);
        assert!(out.c < 0.5);
    }

    #[test]
    fn extreme_saturated_rgb_is_detected_out_of_gamut_on_swop() {
        let provider = DefaultColorManagementProvider;
        let ctx = ProofContext::for_profile("US Web Coated (SWOP) v2");

        // Highly saturated neon green cannot be printed by standard CMYK press
        let neon_green = ColorValue::Rgb(Srgb::clamped(0.0, 1.0, 0.0));
        let (_, status) = provider.soft_proof(&neon_green, &ctx);

        match status {
            GamutStatus::OutOfGamut {
                delta_e,
                clamped_srgb,
            } => {
                assert!(delta_e > 0.0);
                assert!(clamped_srgb.g <= 1.0);
            }
            GamutStatus::InGamut => panic!("Neon green should be out of CMYK press gamut"),
        }
    }

    #[test]
    fn neutral_color_is_in_gamut() {
        let provider = DefaultColorManagementProvider;
        let ctx = ProofContext::for_profile("US Web Coated (SWOP) v2");

        let mid_gray = ColorValue::Rgb(Srgb::clamped(0.5, 0.5, 0.5));
        let (_, status) = provider.soft_proof(&mid_gray, &ctx);
        assert_eq!(status, GamutStatus::InGamut);
    }

    #[test]
    fn gamut_warning_highlights_out_of_gamut_pixels() {
        let provider = DefaultColorManagementProvider;
        let ctx = ProofContext::for_profile("US Web Coated (SWOP) v2").with_gamut_warning(true);

        let neon_cyan = ColorValue::Rgb(Srgb::clamped(0.0, 1.0, 1.0));
        let (display, status) = provider.soft_proof(&neon_cyan, &ctx);

        assert!(matches!(status, GamutStatus::OutOfGamut { .. }));
        // Should be highlighted magenta
        assert_eq!(display, Srgb::clamped(1.0, 0.0, 1.0));
    }
}
