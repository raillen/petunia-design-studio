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
}
