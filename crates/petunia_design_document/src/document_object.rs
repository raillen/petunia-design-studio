//! Minimal canonical object: every node has a stable [`ObjectId`].

use petunia_design_foundation::ObjectId;
use serde::{Deserialize, Serialize};

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

/// Canonical geometric shape or text content representation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ShapeKind {
    /// Rectangular shape with optional corner radii.
    Rectangle { corner_radii: [f64; 4] },
    /// Elliptical shape.
    Ellipse,
    /// Explicit arbitrary vector path with Bézier verbs.
    Path(petunia_design_geometry::GPath),
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
    },
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

    /// Returns the canonical outline `GPath` for this object based on its shape and bounds.
    /// Polygon/Star use `min(w,h)/2` as radius with center at bounds center so
    /// non-square bounds do not distort (F-20). Text has no outline: returns an
    /// empty path so `ConvertToCurves` must reject it explicitly instead of
    /// silently substituting a rectangle.
    ///
    /// This is the editable BASE source. Readers that must see live modifiers
    /// (render, hit-test, selection, booleans, export) use
    /// [`Self::evaluated_path`] instead. Node editing always targets base.
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
            Some(ShapeKind::Polygon { sides }) => {
                let radius = b[2].min(b[3]) / 2.0;
                let center = petunia_design_geometry::GPoint::new(b[0] + b[2] / 2.0, b[1] + b[3] / 2.0);
                petunia_design_geometry::GPath::regular_polygon(center, radius, *sides as usize)
            }
            Some(ShapeKind::Star {
                points,
                inner_ratio,
            }) => {
                let outer_r = b[2].min(b[3]) / 2.0;
                let inner_r = outer_r * inner_ratio.clamp(0.1, 0.9);
                let center = petunia_design_geometry::GPoint::new(b[0] + b[2] / 2.0, b[1] + b[3] / 2.0);
                petunia_design_geometry::GPath::star(center, outer_r, inner_r, *points as usize)
            }
            Some(ShapeKind::Text { .. }) => petunia_design_geometry::GPath::new(),
            _ => petunia_design_geometry::GPath::rect(rect, 0.0, 0.0),
        }
    }

    /// Folds the live modifier chain over the base path (09.31).
    /// Render, hit-testing, selection, booleans, and export read this;
    /// editing tools write base. Empty chain returns base unchanged.
    #[must_use]
    pub fn evaluated_path(&self) -> petunia_design_geometry::GPath {
        crate::modifiers::evaluate_modifiers(&self.to_path(), &self.modifiers)
    }

    /// Bounds of the evaluated outline, falling back to stored base bounds.
    #[must_use]
    pub fn evaluated_bounds(&self) -> Option<[f64; 4]> {
        if self.modifiers.iter().any(|m| m.enabled) {
            self.evaluated_path().bounding_box().map(|r| {
                [
                    r.x0,
                    r.y0,
                    r.width().max(1.0),
                    r.height().max(1.0),
                ]
            })
        } else {
            self.bounds
        }
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
            if let Some(ShapeKind::Path(path)) = &self.shape {
                return path.contains_point(point, 0.5);
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
