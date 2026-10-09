//! Persistent guides, grids and export slices.
//!
//! Guide and grid *definitions* are document state; visibility, snap
//! settings and evaluated candidates are view/session/derived state
//! and are never persisted here. Export slices persist the parameters
//! needed to reproduce an export on another machine.

use crate::color::ColorSpaceRef;
use crate::error::{CoreError, Result};
use crate::id::{GridId, GuideId, ObjectId, PageId, SliceId};
use crate::math::{Point, Rect, Tolerance, Vec2};
use serde::{Deserialize, Serialize};

/// A persistent straight reference line (v0.1: axis-aligned only).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Guide {
    pub id: GuideId,
    pub axis: GuideAxis,
    pub position: f64,
    pub locked: bool,
    pub scope: GuideScope,
}

/// Guide orientation. Angular guides need their own spec and are not
/// encoded as a hidden axis value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuideAxis {
    Horizontal,
    Vertical,
}

/// Which coordinate space a guide belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuideScope {
    Document,
    Page(PageId),
    Artboard(ObjectId),
}

impl Guide {
    /// Build a guide; the position must be finite.
    pub fn new(axis: GuideAxis, position: f64, locked: bool, scope: GuideScope) -> Result<Self> {
        if !position.is_finite() {
            return Err(CoreError::InvariantViolation(format!(
                "non-finite guide position rejected: {position}"
            )));
        }
        Ok(Self {
            id: GuideId::new_v4(),
            axis,
            position,
            locked,
            scope,
        })
    }
}

/// A persistent grid definition with explicit scope and spec.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GridDefinition {
    pub id: GridId,
    pub scope: GridScope,
    pub origin: Point,
    pub spec: GridSpec,
}

/// Grid family. Cartesian, isometric and axonometric share the affine
/// lattice; `kind` preserves the creation preset while the basis
/// vectors preserve exact geometry.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GridSpec {
    Affine(AffineGridSpec),
    Baseline(BaselineGridSpec),
    Perspective(PerspectiveGridSpec),
}

/// Which coordinate space a grid belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridScope {
    Document,
    Page(PageId),
    Artboard(ObjectId),
}

impl GridDefinition {
    #[must_use]
    pub fn new(scope: GridScope, origin: Point, spec: GridSpec) -> Self {
        Self {
            id: GridId::new_v4(),
            scope,
            origin,
            spec,
        }
    }
}

/// Preset intent behind an affine lattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AffineGridKind {
    Cartesian,
    Isometric,
    Axonometric,
    Custom,
}

/// An affine lattice: `P(i,j) = origin + i·u + j·v`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AffineGridSpec {
    pub kind: AffineGridKind,
    pub basis_u: Vec2,
    pub basis_v: Vec2,
    pub subdivisions_u: u32,
    pub subdivisions_v: u32,
}

impl AffineGridSpec {
    /// Basis vectors must be finite, non-zero and non-collinear
    /// within `coincidence`; subdivisions start at one.
    pub fn new(
        kind: AffineGridKind,
        basis_u: Vec2,
        basis_v: Vec2,
        subdivisions_u: u32,
        subdivisions_v: u32,
        coincidence: Tolerance,
    ) -> Result<Self> {
        for basis in [basis_u, basis_v] {
            if !basis.dx.is_finite() || !basis.dy.is_finite() {
                return Err(CoreError::InvariantViolation(format!(
                    "non-finite grid basis rejected: {basis:?}"
                )));
            }
            if basis.dx == 0.0 && basis.dy == 0.0 {
                return Err(CoreError::InvariantViolation(
                    "zero-length grid basis rejected".to_string(),
                ));
            }
        }
        let cross = basis_u.dx * basis_v.dy - basis_u.dy * basis_v.dx;
        let scale = basis_u.length() * basis_v.length();
        if cross.abs() <= coincidence.0 * scale {
            return Err(CoreError::InvariantViolation(
                "collinear grid basis vectors rejected".to_string(),
            ));
        }
        if subdivisions_u == 0 || subdivisions_v == 0 {
            return Err(CoreError::InvariantViolation(
                "grid subdivisions start at one".to_string(),
            ));
        }
        Ok(Self {
            kind,
            basis_u,
            basis_v,
            subdivisions_u,
            subdivisions_v,
        })
    }

    /// Cartesian preset: `u = (spacing_x, 0)`, `v = (0, spacing_y)`.
    pub fn cartesian(
        spacing_x: f64,
        spacing_y: f64,
        subdivisions: u32,
        coincidence: Tolerance,
    ) -> Result<Self> {
        Self::new(
            AffineGridKind::Cartesian,
            Vec2::new(spacing_x, 0.0),
            Vec2::new(0.0, spacing_y),
            subdivisions,
            subdivisions,
            coincidence,
        )
    }
}

/// Horizontal lines at `origin.y + offset + n·spacing`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BaselineGridSpec {
    pub spacing: f64,
    pub offset: f64,
    pub subdivisions: u32,
}

impl BaselineGridSpec {
    /// Spacing must be finite and positive; offset finite.
    pub fn new(spacing: f64, offset: f64, subdivisions: u32) -> Result<Self> {
        if !spacing.is_finite() || spacing <= 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "invalid baseline spacing rejected: {spacing}"
            )));
        }
        if !offset.is_finite() {
            return Err(CoreError::InvariantViolation(format!(
                "non-finite baseline offset rejected: {offset}"
            )));
        }
        if subdivisions == 0 {
            return Err(CoreError::InvariantViolation(
                "grid subdivisions start at one".to_string(),
            ));
        }
        Ok(Self {
            spacing,
            offset,
            subdivisions,
        })
    }
}

/// A normalized 3×3 homography mapping logical grid plane to page.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ProjectiveGridTransform {
    /// Row-major coefficients h00..h22.
    pub coefficients: [f64; 9],
}

impl ProjectiveGridTransform {
    /// Coefficients must be finite and the matrix non-degenerate
    /// (`|det|` above `degeneracy`).
    pub fn new(coefficients: [f64; 9], degeneracy: Tolerance) -> Result<Self> {
        if coefficients.iter().any(|c| !c.is_finite()) {
            return Err(CoreError::InvariantViolation(
                "non-finite homography coefficient rejected".to_string(),
            ));
        }
        let [h00, h01, h02, h10, h11, h12, h20, h21, h22] = coefficients;
        let det = h00 * (h11 * h22 - h12 * h21) - h01 * (h10 * h22 - h12 * h20)
            + h02 * (h10 * h21 - h11 * h20);
        if det.abs() <= degeneracy.0 {
            return Err(CoreError::InvariantViolation(
                "degenerate homography rejected".to_string(),
            ));
        }
        Ok(Self { coefficients })
    }

    /// Identity plane: logical coordinates map 1:1 to the page.
    #[must_use]
    pub fn identity() -> Self {
        Self {
            coefficients: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        }
    }
}

/// Perspective grid over a projective plane.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PerspectiveGridSpec {
    pub transform: ProjectiveGridTransform,
    pub spacing: Vec2,
    pub subdivisions: u32,
}

impl PerspectiveGridSpec {
    /// Spacing components must be finite and positive.
    pub fn new(
        transform: ProjectiveGridTransform,
        spacing: Vec2,
        subdivisions: u32,
    ) -> Result<Self> {
        if !spacing.dx.is_finite()
            || !spacing.dy.is_finite()
            || spacing.dx <= 0.0
            || spacing.dy <= 0.0
        {
            return Err(CoreError::InvariantViolation(format!(
                "invalid perspective spacing rejected: {spacing:?}"
            )));
        }
        if subdivisions == 0 {
            return Err(CoreError::InvariantViolation(
                "grid subdivisions start at one".to_string(),
            ));
        }
        Ok(Self {
            transform,
            spacing,
            subdivisions,
        })
    }
}

/// A reusable output intent saved in the document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportSlice {
    pub id: SliceId,
    pub name: String,
    pub source: SliceSource,
    pub presets: Vec<SliceExportPreset>,
}

/// What a slice exports. Rect sources are page-local.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SliceSource {
    Page(PageId),
    Artboard(ObjectId),
    Object(ObjectId),
    Rect { page: PageId, rect: Rect },
}

impl ExportSlice {
    /// The name must be non-empty; at least one preset is required so
    /// the slice stays reproducible.
    pub fn new(
        name: impl Into<String>,
        source: SliceSource,
        presets: Vec<SliceExportPreset>,
    ) -> Result<Self> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(CoreError::InvariantViolation(
                "export slice needs a non-empty name".to_string(),
            ));
        }
        if presets.is_empty() {
            return Err(CoreError::InvariantViolation(
                "export slice needs at least one preset".to_string(),
            ));
        }
        Ok(Self {
            id: SliceId::new_v4(),
            name,
            source,
            presets,
        })
    }
}

/// Output parameters traveling with the document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SliceExportPreset {
    pub format: ExportFormat,
    pub scale: f64,
    pub color: ExportColorOptions,
    pub suffix: Option<String>,
}

impl SliceExportPreset {
    /// Scale must be a finite positive factor.
    pub fn new(
        format: ExportFormat,
        scale: f64,
        color: ExportColorOptions,
        suffix: Option<String>,
    ) -> Result<Self> {
        if !scale.is_finite() || scale <= 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "invalid export scale rejected: {scale}"
            )));
        }
        Ok(Self {
            format,
            scale,
            color,
            suffix,
        })
    }
}

/// Export container formats covered by the initial contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Png,
    Jpeg,
    Svg,
    Pdf,
}

/// Color intent of one export preset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportColorOptions {
    pub target: ColorSpaceRef,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::BuiltinColorSpace;

    fn coincidence() -> Tolerance {
        Tolerance::new(1e-9).expect("valid")
    }

    #[test]
    fn guide_rejects_non_finite_position() {
        assert!(Guide::new(GuideAxis::Vertical, 120.0, false, GuideScope::Document).is_ok());
        assert!(Guide::new(GuideAxis::Horizontal, f64::NAN, false, GuideScope::Document).is_err());
    }

    #[test]
    fn affine_grid_validates_basis_and_subdivisions() {
        let cartesian = AffineGridSpec::cartesian(10.0, 10.0, 2, coincidence()).expect("valid");
        assert_eq!(cartesian.kind, AffineGridKind::Cartesian);

        // Collinear vectors cannot span a 2D lattice.
        assert!(AffineGridSpec::new(
            AffineGridKind::Custom,
            Vec2::new(10.0, 0.0),
            Vec2::new(5.0, 0.0),
            1,
            1,
            coincidence(),
        )
        .is_err());
        // Zero-length basis is rejected.
        assert!(AffineGridSpec::new(
            AffineGridKind::Custom,
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 10.0),
            1,
            1,
            coincidence(),
        )
        .is_err());
        // Subdivisions start at one.
        assert!(AffineGridSpec::cartesian(10.0, 10.0, 0, coincidence()).is_err());
    }

    #[test]
    fn baseline_grid_requires_positive_spacing() {
        assert!(BaselineGridSpec::new(12.0, 0.0, 1).is_ok());
        assert!(BaselineGridSpec::new(0.0, 0.0, 1).is_err());
        assert!(BaselineGridSpec::new(-4.0, 0.0, 1).is_err());
    }

    #[test]
    fn homography_rejects_degenerate_matrix() {
        assert!(ProjectiveGridTransform::identity().coefficients[0] == 1.0);
        assert!(ProjectiveGridTransform::new([0.0; 9], coincidence()).is_err());
        assert!(ProjectiveGridTransform::new(
            [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, f64::NAN],
            coincidence()
        )
        .is_err());
    }

    #[test]
    fn slice_needs_name_and_preset() {
        let preset = SliceExportPreset::new(
            ExportFormat::Png,
            2.0,
            ExportColorOptions {
                target: ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
            },
            None,
        )
        .expect("valid");
        let page = PageId::new_v4();
        assert!(ExportSlice::new("hero", SliceSource::Page(page), vec![preset.clone()]).is_ok());
        assert!(ExportSlice::new("", SliceSource::Page(page), vec![preset.clone()]).is_err());
        assert!(ExportSlice::new("hero", SliceSource::Page(page), vec![]).is_err());
        assert!(SliceExportPreset::new(
            ExportFormat::Png,
            0.0,
            ExportColorOptions {
                target: ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
            },
            None
        )
        .is_err());
    }

    #[test]
    fn guide_serialization_round_trip() {
        let guide =
            Guide::new(GuideAxis::Vertical, 120.0, true, GuideScope::Document).expect("valid");
        let json = serde_json::to_string(&guide).expect("serializable");
        let back: Guide = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(back, guide);
    }
}
