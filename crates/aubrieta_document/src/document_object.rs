//! Minimal canonical object: every node has a stable [`ObjectId`].

use aubrieta_foundation::ObjectId;
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
    /// Fill color as semantic token reference (e.g. `aubrieta.red/500`).
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

    /// Computes the local affine transformation for this object based on bounds origin and rotation.
    #[must_use]
    pub fn local_transform(&self) -> aubrieta_geometry::GAffine {
        let (tx, ty) = self.bounds.map_or((0.0, 0.0), |b| (b[0], b[1]));
        aubrieta_geometry::GAffine::translate(tx, ty)
            .after(aubrieta_geometry::GAffine::rotate(self.rotation))
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
