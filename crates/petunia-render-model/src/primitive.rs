//! Render primitives: evaluated vector, text, image and raster nodes.

use crate::composite::RenderGroup;
use crate::image::{ImagePrimitive, RasterPrimitive};
use crate::paint::RenderAppearance;
use crate::text::TextPrimitive;
use petunia_core::{ObjectId, Rect, Transform2D};
use serde::{Deserialize, Serialize};

/// One evaluated drawable, distinguished by semantics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RenderPrimitive {
    Vector(VectorPrimitive),
    Text(TextPrimitive),
    Image(ImagePrimitive),
    Raster(RasterPrimitive),
    Group(RenderGroup),
}

/// Evaluated vector geometry: compact flattened path plus paint.
/// Not the authorial `VectorPath`: curves are already flattened and
/// strokes carry resolved widths.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorPrimitive {
    pub source: ObjectId,
    pub geometry: RenderPath,
    pub appearance: RenderAppearance,
    pub transform: Transform2D,
    pub bounds: Rect,
}

/// Compact flattened contours in document space. Plain pairs keep
/// the contract free of geometry helpers; closure rides alongside.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RenderPath {
    pub contours: Vec<Vec<(f64, f64)>>,
    pub closed: Vec<bool>,
}

impl RenderPath {
    /// Empty path.
    #[must_use]
    pub fn new() -> Self {
        Self {
            contours: Vec::new(),
            closed: Vec::new(),
        }
    }

    /// Push one contour with its closed flag.
    pub fn push_contour(&mut self, points: Vec<(f64, f64)>, closed: bool) {
        self.contours.push(points);
        self.closed.push(closed);
    }
}
