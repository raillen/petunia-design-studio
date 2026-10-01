//! Minimal canonical object: every node has a stable [`ObjectId`].

use petunia_design_foundation::ObjectId;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Failure raised when an object cannot be projected into an explicit frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GeometryFrameError {
    /// A stored placement frame cannot be projected safely.
    MissingBounds(ObjectId),
    /// The placement frame is finite but has invalid dimensions.
    InvalidBounds(ObjectId),
    /// A coordinate or transform contains a non-finite value.
    NonFinite(ObjectId),
    /// A v1 explicit path has no persisted frame marker and must be migrated.
    AmbiguousPath(ObjectId),
    /// A v1 modifier has document-space parameters without an explicit frame.
    AmbiguousModifier(ObjectId, &'static str),
    /// An object or parent reference does not exist.
    MissingObject(ObjectId),
    /// The parent chain contains a cycle.
    HierarchyCycle(ObjectId),
}

impl fmt::Display for GeometryFrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingBounds(id) => write!(formatter, "object `{id}` has no placement bounds"),
            Self::InvalidBounds(id) => {
                write!(formatter, "object `{id}` has invalid placement bounds")
            }
            Self::NonFinite(id) => write!(formatter, "object `{id}` has non-finite geometry data"),
            Self::AmbiguousPath(id) => {
                write!(
                    formatter,
                    "object `{id}` has an unversioned path frame; migrate it explicitly"
                )
            }
            Self::AmbiguousModifier(id, kind) => {
                write!(
                    formatter,
                    "object `{id}` has unversioned `{kind}` modifier coordinates"
                )
            }
            Self::MissingObject(id) => write!(formatter, "object `{id}` does not exist"),
            Self::HierarchyCycle(id) => {
                write!(formatter, "object hierarchy contains a cycle at `{id}`")
            }
        }
    }
}

impl std::error::Error for GeometryFrameError {}

fn default_true() -> bool {
    true
}

fn default_one() -> f64 {
    1.0
}

/// Single node in the document tree.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentObject {
    /// Stable identity. Never a `Vec` index.
    pub id: ObjectId,
    /// Human label for panels and tests. Not identity.
    pub name: String,
    /// Fill color as semantic token reference (e.g. `ptnd.red/500`).
    pub fill: Option<String>,
    /// Whether this object is visible in viewports and render passes.
    #[serde(default = "default_true")]
    pub visible: bool,
    /// Whether this object is locked against interactive transformation.
    #[serde(default)]
    pub locked: bool,
    /// Object opacity factor in `[0.0, 1.0]`.
    #[serde(default = "default_one")]
    pub opacity: f64,
    /// Stroke color token reference.
    #[serde(default)]
    pub stroke: Option<String>,
    /// Stroke line width in document points.
    #[serde(default = "default_one")]
    pub stroke_width: f64,
    /// Axis-aligned bounds `[x, y, width, height]` in document points.
    #[serde(default)]
    pub bounds: Option<[f64; 4]>,
    /// In-plane rotation angle in radians.
    #[serde(default)]
    pub rotation: f64,
    /// Canonical V1 Appearance Stack with multiple fills/strokes/effects.
    #[serde(default)]
    pub appearance: Option<crate::appearance::AppearanceStack>,
    /// Parent container object in the canonical tree, if any.
    #[serde(default)]
    pub parent: Option<ObjectId>,
    /// Child object identities in ordered z-index (back to front).
    #[serde(default)]
    pub children: Vec<ObjectId>,
    /// Semantic container role (Group, Layer, ClipGroup).
    #[serde(default)]
    pub role: Option<crate::hierarchy::ContainerRole>,
    /// Whether this object functions as a clipping mask boundary for its siblings in a ClipGroup.
    #[serde(default)]
    pub is_clip_mask: bool,
    /// Target clipping mask object identity, if clipped directly.
    #[serde(default)]
    pub clip_mask_id: Option<ObjectId>,
    /// Mask compositing mode when acting as or attached to a mask.
    #[serde(default)]
    pub mask_mode: crate::hierarchy::MaskMode,
    /// Canonical vector shape or text content of this object, if not a container.
    #[serde(default)]
    pub shape: Option<ShapeKind>,
    /// Ordered live modifier chain (the non-destructive EffectChain, 09.31).
    /// Empty by default; evaluated on read, never stored as geometry.
    #[serde(default)]
    pub modifiers: Vec<crate::modifiers::ModifierItem>,
}

/// Text-on-path attachment (10.6): flows a text object along another
/// object's evaluated outline between normalized fractions. The source
/// path object is never consumed or hidden by the attachment.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextOnPathAttachment {
    /// Path object the text follows.
    pub target: ObjectId,
    /// Span start as a fraction of outline length in `[0.0, 1.0]`.
    pub start: f64,
    /// Span end as a fraction of outline length in `[0.0, 1.0]`.
    pub end: f64,
}

impl TextOnPathAttachment {
    /// Creates a clamped attachment, ordering start before end.
    #[must_use]
    pub fn new(target: ObjectId, start: f64, end: f64) -> Self {
        let (mut a, mut b) = (start.clamp(0.0, 1.0), end.clamp(0.0, 1.0));
        if b < a {
            std::mem::swap(&mut a, &mut b);
        }
        // Degenerate spans keep a minimal readable length.
        if (b - a).abs() < 1e-6 {
            b = (a + 0.01).min(1.0);
            if (b - a).abs() < 1e-6 {
                a = (b - 0.01).max(0.0);
            }
        }
        Self {
            target,
            start: a,
            end: b,
        }
    }
}

/// Canonical geometric shape or text content representation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ShapeKind {
    /// Rectangular shape with optional corner radii.
    Rectangle { corner_radii: [f64; 4] },
    /// Elliptical shape.
    Ellipse,
    /// Legacy/input path in parent coordinates. Writers normalize it to `LocalPath`.
    Path(petunia_design_geometry::GPath),
    /// Editable local geometry; placement and resizing never rewrite these points.
    LocalPath {
        path: std::sync::Arc<petunia_design_geometry::GPath>,
        reference_size: [f64; 2],
    },
    /// Regular polygon with N sides.
    Polygon { sides: u32 },
    /// Star polygon with N points and inner radius ratio.
    Star { points: u32, inner_ratio: f64 },
    /// Text frame / artistic text object.
    Text {
        content: String,
        font_family: String,
        font_size: f64,
        line_height: f64,
        letter_spacing: f64,
        /// Text-on-path attachment; `None` flows inside bounds as before.
        #[serde(default)]
        on_path: Option<TextOnPathAttachment>,
    },
    /// Placed raster image object with path or encoded data.
    Image {
        path: String,
        /// Immutable encoded source shared by snapshots, history and duplication.
        /// The serialized byte-array representation is unchanged.
        #[serde(default)]
        data: Option<std::sync::Arc<Vec<u8>>>,
    },
}

impl ShapeKind {
    /// Whether this descriptor holds editable Bézier geometry.
    #[must_use]
    pub fn is_path(&self) -> bool {
        matches!(self, Self::Path(_) | Self::LocalPath { .. })
    }

    pub(crate) fn into_local(
        self,
        bounds: Option<[f64; 4]>,
    ) -> Result<Self, petunia_design_foundation::PetuniaError> {
        if let Self::Path(path) = self {
            let b = bounds.ok_or_else(|| {
                petunia_design_foundation::PetuniaError::invalid_input("path requires bounds")
            })?;
            if !b.iter().all(|v| v.is_finite()) || b[2] <= 0.0 || b[3] <= 0.0 || !path.is_finite() {
                return Err(petunia_design_foundation::PetuniaError::invalid_input(
                    "path requires finite geometry and positive bounds",
                ));
            }
            Ok(Self::LocalPath {
                path: std::sync::Arc::new(
                    path.transformed(petunia_design_geometry::GAffine::translate(-b[0], -b[1])),
                ),
                reference_size: [b[2], b[3]],
            })
        } else {
            Ok(self)
        }
    }
}

impl DocumentObject {
    /// Creates an object with an explicit stable ID.
    #[must_use]
    pub fn new(id: ObjectId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            fill: None,
            visible: true,
            locked: false,
            opacity: 1.0,
            stroke: None,
            stroke_width: 1.0,
            bounds: None,
            rotation: 0.0,
            appearance: None,
            parent: None,
            children: Vec::new(),
            role: None,
            is_clip_mask: false,
            clip_mask_id: None,
            mask_mode: crate::hierarchy::MaskMode::Vector,
            shape: None,
            modifiers: Vec::new(),
        }
    }

    /// Returns the canonical local outline for this object without applying placement.
    ///
    /// Parametric shapes are generated in a frame whose origin is `[0, 0]`;
    /// `bounds` and `rotation` are placement data and are not embedded here.
    /// Explicit paths must already be local. Text and containers have no local
    /// vector outline and return an empty path.
    pub fn base_path_local(
        &self,
    ) -> Result<petunia_design_geometry::GPath, crate::GeometryFrameError> {
        if self.shape.is_none() && self.modifiers.is_empty() && self.bounds.is_none() {
            return Ok(petunia_design_geometry::GPath::new());
        }
        let b = self
            .bounds
            .ok_or(crate::GeometryFrameError::MissingBounds(self.id))?;
        if !b.iter().all(|value| value.is_finite()) {
            return Err(crate::GeometryFrameError::NonFinite(self.id));
        }
        if b[2] <= 0.0 || b[3] <= 0.0 {
            return Err(crate::GeometryFrameError::InvalidBounds(self.id));
        }
        let rect = petunia_design_geometry::GRect::new(0.0, 0.0, b[2], b[3]);
        match &self.shape {
            Some(ShapeKind::Rectangle { corner_radii }) => Ok(
                petunia_design_geometry::GPath::rect_corners(rect, *corner_radii),
            ),
            Some(ShapeKind::Ellipse) => {
                let rx = b[2] / 2.0;
                let ry = b[3] / 2.0;
                let center = petunia_design_geometry::GPoint::new(rx, ry);
                Ok(petunia_design_geometry::GPath::ellipse(center, rx, ry))
            }
            Some(ShapeKind::Path(_)) => Err(crate::GeometryFrameError::AmbiguousPath(self.id)),
            Some(ShapeKind::LocalPath {
                path,
                reference_size,
            }) => {
                if !path.is_finite() || !reference_size.iter().all(|v| v.is_finite() && *v > 0.0) {
                    return Err(crate::GeometryFrameError::NonFinite(self.id));
                }
                let result = path.transformed(petunia_design_geometry::GAffine::scale(
                    b[2] / reference_size[0],
                    b[3] / reference_size[1],
                ));
                if !result.is_finite() {
                    return Err(crate::GeometryFrameError::NonFinite(self.id));
                }
                Ok(result)
            }
            Some(ShapeKind::Polygon { sides }) => {
                let radius = b[2].min(b[3]) / 2.0;
                let center = petunia_design_geometry::GPoint::new(b[2] / 2.0, b[3] / 2.0);
                Ok(petunia_design_geometry::GPath::regular_polygon(
                    center,
                    radius,
                    *sides as usize,
                ))
            }
            Some(ShapeKind::Star {
                points,
                inner_ratio,
            }) => {
                let outer_r = b[2].min(b[3]) / 2.0;
                let inner_r = outer_r * inner_ratio.clamp(0.1, 0.9);
                let center = petunia_design_geometry::GPoint::new(b[2] / 2.0, b[3] / 2.0);
                Ok(petunia_design_geometry::GPath::star(
                    center,
                    outer_r,
                    inner_r,
                    *points as usize,
                ))
            }
            Some(ShapeKind::Text { .. }) | Some(ShapeKind::Image { .. }) | None => {
                Ok(petunia_design_geometry::GPath::new())
            }
        }
    }

    fn validate_local_modifiers(&self) -> Result<(), crate::GeometryFrameError> {
        for item in self.modifiers.iter().filter(|item| item.enabled) {
            let crate::ModifierSpace::Local { reference_size } = item.space else {
                return Err(crate::GeometryFrameError::AmbiguousModifier(
                    self.id,
                    "parent-frame",
                ));
            };
            if !reference_size.iter().all(|v| v.is_finite() && *v > 0.0) {
                return Err(crate::GeometryFrameError::NonFinite(self.id));
            }
            match &item.kind {
                crate::modifiers::ModifierKind::ContourOffset { distance, .. } => {
                    if !distance.is_finite() {
                        return Err(crate::GeometryFrameError::NonFinite(self.id));
                    }
                }
                crate::modifiers::ModifierKind::TransparentGradient { start, end, stops } => {
                    if !start.iter().chain(end).all(|v| v.is_finite())
                        || stops
                            .iter()
                            .any(|s| !s.offset.is_finite() || !s.opacity.is_finite())
                    {
                        return Err(crate::GeometryFrameError::NonFinite(self.id));
                    }
                }
                crate::modifiers::ModifierKind::Perspective { quad } => {
                    if !quad.iter().flatten().all(|v| v.is_finite()) {
                        return Err(crate::GeometryFrameError::NonFinite(self.id));
                    }
                }
                crate::modifiers::ModifierKind::CropRect { rect } => {
                    if !rect.iter().all(|v| v.is_finite()) || rect[2] <= 0.0 || rect[3] <= 0.0 {
                        return Err(crate::GeometryFrameError::InvalidBounds(self.id));
                    }
                }
            }
        }
        Ok(())
    }

    /// Returns the live local outline after applying the modifier chain.
    pub fn evaluated_path_local(
        &self,
    ) -> Result<petunia_design_geometry::GPath, crate::GeometryFrameError> {
        self.validate_local_modifiers()?;
        let base = self.base_path_local()?;
        let [_, _, w, h] = self.bounds.unwrap_or([0.0; 4]);
        crate::modifiers::evaluate_modifiers_local(&base, &self.modifiers, [w, h])
            .ok_or(crate::GeometryFrameError::NonFinite(self.id))
    }

    /// Returns evaluated local bounds when the local outline has geometry.
    pub fn evaluated_bounds_local(&self) -> Result<Option<[f64; 4]>, crate::GeometryFrameError> {
        let path = self.evaluated_path_local()?;
        Ok(path
            .bounding_box()
            .map(|r| [r.x0, r.y0, r.width().max(1.0), r.height().max(1.0)]))
    }

    /// Returns the legacy outline behavior for compatibility callers.
    ///
    /// New code must choose [`Self::base_path_local`] or the world resolver
    /// explicitly; this method remains while v1 consumers are migrated.
    #[must_use]
    pub fn to_path(&self) -> petunia_design_geometry::GPath {
        let b = self.bounds.unwrap_or([0.0, 0.0, 100.0, 100.0]);
        let rect = petunia_design_geometry::GRect::new(b[0], b[1], b[0] + b[2], b[1] + b[3]);
        match &self.shape {
            Some(ShapeKind::Rectangle { corner_radii }) => {
                petunia_design_geometry::GPath::rect_corners(rect, *corner_radii)
            }
            Some(ShapeKind::Ellipse) => {
                let rx = b[2] / 2.0;
                let ry = b[3] / 2.0;
                let center = petunia_design_geometry::GPoint::new(b[0] + rx, b[1] + ry);
                petunia_design_geometry::GPath::ellipse(center, rx, ry)
            }
            Some(ShapeKind::Path(path)) => path.clone(),
            Some(ShapeKind::LocalPath { .. }) => self
                .base_path_local()
                .map(|path| {
                    path.transformed(petunia_design_geometry::GAffine::translate(b[0], b[1]))
                })
                .unwrap_or_default(),
            Some(ShapeKind::Polygon { sides }) => {
                let radius = b[2].min(b[3]) / 2.0;
                let center =
                    petunia_design_geometry::GPoint::new(b[0] + b[2] / 2.0, b[1] + b[3] / 2.0);
                petunia_design_geometry::GPath::regular_polygon(center, radius, *sides as usize)
            }
            Some(ShapeKind::Star {
                points,
                inner_ratio,
            }) => {
                let outer_r = b[2].min(b[3]) / 2.0;
                let inner_r = outer_r * inner_ratio.clamp(0.1, 0.9);
                let center =
                    petunia_design_geometry::GPoint::new(b[0] + b[2] / 2.0, b[1] + b[3] / 2.0);
                petunia_design_geometry::GPath::star(center, outer_r, inner_r, *points as usize)
            }
            Some(ShapeKind::Text { .. }) | Some(ShapeKind::Image { .. }) => {
                petunia_design_geometry::GPath::new()
            }
            _ => petunia_design_geometry::GPath::rect(rect, 0.0, 0.0),
        }
    }

    /// Folds the live modifier chain over the legacy path while old callers
    /// migrate. New world-scoped readers must use the explicit resolver.
    #[must_use]
    pub fn evaluated_path(&self) -> petunia_design_geometry::GPath {
        if self.modifiers.is_empty()
            || self
                .modifiers
                .iter()
                .any(|m| m.space == crate::ModifierSpace::Parent)
        {
            return crate::modifiers::evaluate_modifiers(&self.to_path(), &self.modifiers);
        }
        let [x, y, _, _] = self.bounds.unwrap_or([0.0; 4]);
        self.evaluated_path_local()
            .map(|path| path.transformed(petunia_design_geometry::GAffine::translate(x, y)))
            .unwrap_or_default()
    }

    /// Bounds of the evaluated outline, falling back to stored base bounds.
    #[must_use]
    pub fn evaluated_bounds(&self) -> Option<[f64; 4]> {
        if self.modifiers.iter().any(|m| m.enabled) {
            self.evaluated_path()
                .bounding_box()
                .map(|r| [r.x0, r.y0, r.width().max(1.0), r.height().max(1.0)])
        } else {
            self.bounds
        }
    }

    /// Samples the editable mask in current local coordinates.
    pub fn opacity_at_local(
        &self,
        point: petunia_design_geometry::GPoint,
    ) -> Result<f64, crate::GeometryFrameError> {
        if !point.x.is_finite() || !point.y.is_finite() {
            return Err(crate::GeometryFrameError::NonFinite(self.id));
        }
        if self.modifiers.is_empty() {
            return Ok(self.opacity);
        }
        self.validate_local_modifiers()?;
        let [_, _, w, h] = self
            .bounds
            .ok_or(crate::GeometryFrameError::MissingBounds(self.id))?;
        crate::modifiers::evaluate_opacity_local(&self.modifiers, point, [w, h])
            .map(|mask| self.opacity * mask)
            .ok_or(crate::GeometryFrameError::NonFinite(self.id))
    }

    /// Effective opacity for export/preview: base opacity times the live
    /// transparency mask sampled at the bounds center. Full mask rendering
    /// stays future work; this documented approximation keeps export honest.
    #[must_use]
    pub fn sampled_opacity(&self) -> f64 {
        let [x, y, w, h] = self.bounds.unwrap_or([0.0; 4]);
        let mask = if self
            .modifiers
            .iter()
            .any(|m| m.space == crate::ModifierSpace::Parent)
        {
            crate::modifiers::evaluate_opacity_at(
                &self.modifiers,
                petunia_design_geometry::GPoint::new(x + w / 2.0, y + h / 2.0),
            )
        } else {
            crate::modifiers::evaluate_opacity_local(
                &self.modifiers,
                petunia_design_geometry::GPoint::new(w / 2.0, h / 2.0),
                [w, h],
            )
            .unwrap_or(0.0)
        };
        (self.opacity * mask).clamp(0.0, 1.0)
    }

    /// Hit-tests whether a document point lies within this object's shape or bounds.
    #[must_use]
    pub fn hit_test(&self, point: petunia_design_geometry::GPoint) -> bool {
        if !self.visible {
            return false;
        }
        if self.modifiers.iter().any(|m| m.enabled) {
            // Live modifiers move the outline: test the evaluated geometry.
            let evaluated = self.evaluated_path();
            if evaluated.is_empty() {
                return false;
            }
            return evaluated.contains_point(point, 0.5);
        }
        if let Some(b) = self.bounds {
            if point.x < b[0] || point.x > b[0] + b[2] || point.y < b[1] || point.y > b[1] + b[3] {
                return false;
            }
            if matches!(self.shape, Some(ShapeKind::Ellipse)) {
                let rx = b[2] / 2.0;
                let ry = b[3] / 2.0;
                let cx = b[0] + rx;
                let cy = b[1] + ry;
                let dx = (point.x - cx) / rx.max(1e-6);
                let dy = (point.y - cy) / ry.max(1e-6);
                return dx * dx + dy * dy <= 1.0;
            }
            if self.shape.as_ref().is_some_and(ShapeKind::is_path) {
                return self.to_path().contains_point(point, 0.5);
            }
            true
        } else {
            false
        }
    }

    /// Returns the effective AppearanceStack for this object.
    /// If an explicit appearance stack is present, it is returned.
    /// Otherwise, a synthesized stack matching legacy fill/stroke/opacity is constructed.
    #[must_use]
    pub fn effective_appearance(&self) -> crate::appearance::AppearanceStack {
        if let Some(app) = &self.appearance {
            app.clone()
        } else {
            let mut stack = crate::appearance::AppearanceStack::new();
            stack.opacity = self.opacity;
            if let Some(f) = &self.fill {
                stack.fills.push(crate::appearance::FillItem::solid(1, f));
            }
            if let Some(s) = &self.stroke {
                stack.strokes.push(crate::appearance::StrokeItem::solid(
                    1,
                    s,
                    self.stroke_width,
                ));
            }
            stack
        }
    }

    /// Returns true if this object is a container (has children or an explicit container role).
    #[must_use]
    pub fn is_container(&self) -> bool {
        !self.children.is_empty() || self.role.is_some()
    }

    /// Computes the local affine transformation for this object.
    /// Model: `T(bounds_origin) * R(rotation)` (F-07). Translation carries
    /// the bounds origin so `ORIGIN` maps to the object position and
    /// `world_transform` composes by accumulation; rotation pivots about
    /// the bounds top-left in the local frame. Exporters must use the same
    /// pivot (top-left) for exact fidelity. When there are no bounds, falls
    /// back to pure rotation about the origin.
    #[must_use]
    pub fn local_transform(&self) -> petunia_design_geometry::GAffine {
        let rot = petunia_design_geometry::GAffine::rotate(self.rotation);
        if let Some(b) = self.bounds {
            petunia_design_geometry::GAffine::translate(b[0], b[1]).after(rot)
        } else {
            rot
        }
    }

    /// Extracts the pure rotation angle (radians) from an affine whose linear
    /// part is rotation-only. Used when preserving world transform on reparent.
    #[must_use]
    pub fn rotation_from_affine(affine: petunia_design_geometry::GAffine) -> f64 {
        affine.coeffs[1].atan2(affine.coeffs[0])
    }

    /// Updates the local translation origin (x, y) while preserving dimensions.
    pub fn set_local_origin(&mut self, x: f64, y: f64) {
        if let Some(b) = &mut self.bounds {
            b[0] = x;
            b[1] = y;
        } else {
            self.bounds = Some([x, y, 0.0, 0.0]);
        }
    }
}

/// Alignment modes for multi-selection layout commands (10.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlignmentMode {
    Left,
    Center,
    Right,
    Top,
    Middle,
    Bottom,
}

/// Distribution axes for multi-selection spacing commands (10.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistributionAxis {
    Horizontal,
    Vertical,
}

/// Z-order arrange positions within a surface (10.1, F-16).
/// Front is the end of the back-to-front `Surface.objects` order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArrangePosition {
    /// Move to the front (top of paint order).
    Front,
    /// Move to the back (bottom of paint order).
    Back,
    /// Move one step toward the front. NoOp when already frontmost.
    Forward,
    /// Move one step toward the back. NoOp when already backmost.
    Backward,
}

#[cfg(test)]
mod text_on_path_tests {
    use super::*;

    #[test]
    fn attachment_orders_and_clamps() {
        let att = TextOnPathAttachment::new(ObjectId::new(1), 0.9, 0.2);
        assert_eq!((att.start, att.end), (0.2, 0.9));
        let att = TextOnPathAttachment::new(ObjectId::new(1), -5.0, 99.0);
        assert_eq!((att.start, att.end), (0.0, 1.0));
    }

    #[test]
    fn legacy_text_without_on_path_loads() {
        // v1 payloads predate the field: serde default keeps them readable.
        let shape: ShapeKind = serde_json::from_value(serde_json::json!({
            "kind": "Text",
            "content": "Hi",
            "font_family": "Inter",
            "font_size": 14.0,
            "line_height": 1.3,
            "letter_spacing": 0.0,
        }))
        .expect("legacy text loads");
        assert!(matches!(shape, ShapeKind::Text { on_path: None, .. }));
    }
}
