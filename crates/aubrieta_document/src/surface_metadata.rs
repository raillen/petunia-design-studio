//! Surface layout metadata: dimensions, bleed, margins, guides, and background (10.7).
//!
//! Under the 10.7 contract, a Surface acts as an artboard, page, or export region.
//! Bleed, margins, and guides provide non-destructive authoring layout metadata.

use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

/// Per-side bleed configuration in document points (10.7).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Bleed {
    /// Top bleed extension.
    pub top: f64,
    /// Right bleed extension.
    pub right: f64,
    /// Bottom bleed extension.
    pub bottom: f64,
    /// Left bleed extension.
    pub left: f64,
}

impl Bleed {
    /// Zero bleed on all edges.
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    /// Uniform bleed across all four sides.
    #[must_use]
    pub const fn uniform(amount: f64) -> Self {
        Self {
            top: amount,
            right: amount,
            bottom: amount,
            left: amount,
        }
    }

    /// Explicit per-side bleed.
    #[must_use]
    pub const fn new(top: f64, right: f64, bottom: f64, left: f64) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Returns true if all bleed extensions are zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.top.abs() < 1e-6
            && self.right.abs() < 1e-6
            && self.bottom.abs() < 1e-6
            && self.left.abs() < 1e-6
    }
}

/// Per-side margin configuration in document points (10.7).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Margins {
    /// Top inner margin.
    pub top: f64,
    /// Right inner margin.
    pub right: f64,
    /// Bottom inner margin.
    pub bottom: f64,
    /// Left inner margin.
    pub left: f64,
}

impl Margins {
    /// Zero margins on all edges.
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    /// Uniform margin across all four sides.
    #[must_use]
    pub const fn uniform(amount: f64) -> Self {
        Self {
            top: amount,
            right: amount,
            bottom: amount,
            left: amount,
        }
    }

    /// Explicit per-side margins.
    #[must_use]
    pub const fn new(top: f64, right: f64, bottom: f64, left: f64) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

/// Axis orientation of a layout guide.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GuideOrientation {
    /// Horizontal guide at constant Y.
    Horizontal,
    /// Vertical guide at constant X.
    Vertical,
}

/// Single horizontal or vertical layout guide on a Surface (10.7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Guide {
    /// Local guide identity on the surface.
    pub id: u32,
    /// Horizontal or vertical orientation.
    pub orientation: GuideOrientation,
    /// Coordinate offset along the perpendicular axis in points.
    pub position: f64,
    /// Whether the guide is locked against interactive dragging.
    #[serde(default)]
    pub locked: bool,
    /// Whether the guide is visible in viewports.
    #[serde(default = "default_true")]
    pub visible: bool,
}

impl Guide {
    /// Creates a new unlocked, visible layout guide.
    #[must_use]
    pub fn new(id: u32, orientation: GuideOrientation, position: f64) -> Self {
        Self {
            id,
            orientation,
            position,
            locked: false,
            visible: true,
        }
    }
}
