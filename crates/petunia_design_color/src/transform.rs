//! Display transform facade. moxcms stays inside [`moxcms_adapter`].
//!
//! Named profiles require bytes; checked transforms are exposed in `icc`.

use petunia_design_foundation::PetuniaError;

use crate::{ColorValue, RenderingIntent, Srgb};

/// Converts a stored value for display. `intent` is accepted for contract
/// stability; the MVP path is intent-independent.
#[must_use]
pub fn convert_for_display(value: &ColorValue, _intent: RenderingIntent) -> Srgb {
    value.to_srgb()
}

/// ICC-profile transform (`POST_V1`): profiles, proofing and LittleCMS
/// differential harnesses arrive in a later wave.
pub fn transform_via_profile(
    _value: &ColorValue,
    _profile_name: &str,
    _intent: RenderingIntent,
) -> Result<Srgb, PetuniaError> {
    Err(PetuniaError::capability_unavailable(
        "ICC transform requires profile bytes; a profile name is insufficient",
    ))
}

/// moxcms adapter: Lab→XYZ reference used for differential cross-checks.
#[cfg(test)]
mod moxcms_adapter {
    use crate::Lab;

    /// Reference XYZ (D50-ish PCS path per moxcms) for one Lab value.
    /// Returns `[x, y, z]` as `f64`.
    pub(super) fn lab_to_xyz_reference(lab: Lab) -> [f64; 3] {
        let adapted = moxcms::Lab {
            l: lab.l,
            a: lab.a,
            b: lab.b,
        };
        let xyz = adapted.to_xyz();
        [f64::from(xyz.x), f64::from(xyz.y), f64::from(xyz.z)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Lab;

    #[test]
    fn profile_name_cannot_stand_in_for_profile_bytes() {
        let err = transform_via_profile(
            &ColorValue::Registration,
            "sRGB",
            RenderingIntent::Perceptual,
        )
        .expect_err("must require profile bytes");
        assert!(err.to_string().contains("profile bytes"));
    }

    #[test]
    fn moxcms_reference_agrees_on_white_xyz() {
        // moxcms `to_xyz` returns ICC PCS-encoded XYZ: D50 white halved by
        // (1 + 32767/32768) ≈ (0.4821, 0.5, 0.4125). Drift means the vendored
        // PCS path changed and the adapter needs review, not silence.
        let xyz = moxcms_adapter::lab_to_xyz_reference(Lab {
            l: 100.0,
            a: 0.0,
            b: 0.0,
        });
        assert!((xyz[0] - 0.482_1).abs() < 0.005, "{xyz:?}");
        assert!((xyz[1] - 0.5).abs() < 0.005, "{xyz:?}");
        assert!((xyz[2] - 0.412_5).abs() < 0.005, "{xyz:?}");
    }
}
