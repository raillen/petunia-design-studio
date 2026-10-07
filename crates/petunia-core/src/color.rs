//! Color representations and color space metadata.

use serde::{Deserialize, Serialize};

/// RGBA color with normalized f32 channels [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColorRgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorRgba {
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    #[must_use]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates an RGBA color from 8-bit integers (0-255).
    #[must_use]
    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: f32::from(r) / 255.0,
            g: f32::from(g) / 255.0,
            b: f32::from(b) / 255.0,
            a: f32::from(a) / 255.0,
        }
    }
}

/// Color spaces supported by the creative engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ColorSpace {
    #[default]
    Srgb,
    DisplayP3,
    LinearSrgb,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_conversions() {
        let white = ColorRgba::from_u8(255, 255, 255, 255);
        assert!((white.r - 1.0).abs() < f32::EPSILON);
        assert!((white.g - 1.0).abs() < f32::EPSILON);
        assert!((white.b - 1.0).abs() < f32::EPSILON);
        assert!((white.a - 1.0).abs() < f32::EPSILON);
    }
}
