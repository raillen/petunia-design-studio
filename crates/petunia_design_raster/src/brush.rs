//! Brush stroke pipeline, dab generation and blend modes (09.6).

use crate::tile::TileMap;

/// Normalized input sample from a pointer or tablet pen (09.6).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrushInputSample {
    pub x: f64,
    pub y: f64,
    pub pressure: f32,
    pub tilt_x: f32,
    pub tilt_y: f32,
}

impl BrushInputSample {
    #[must_use]
    pub const fn new(x: f64, y: f64, pressure: f32) -> Self {
        Self {
            x,
            y,
            pressure,
            tilt_x: 0.0,
            tilt_y: 0.0,
        }
    }
}

/// Standard Porter-Duff and photo blend modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    /// Erases destination coverage independently of the source color.
    DestinationOut,
}

impl BlendMode {
    /// Blends a source pixel over a destination pixel.
    #[must_use]
    pub fn blend(self, src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
        let sa = src[3];
        let da = dst[3];
        if self == Self::DestinationOut {
            let alpha = da * (1.0 - sa.clamp(0.0, 1.0));
            return if alpha <= 0.0 {
                [0.0; 4]
            } else {
                [dst[0], dst[1], dst[2], alpha]
            };
        }
        let out_a = sa + da * (1.0 - sa);
        if out_a <= 0.0 {
            return [0.0, 0.0, 0.0, 0.0];
        }

        let blend_channel = |sc: f32, dc: f32| -> f32 {
            let blended = match self {
                Self::Normal => sc,
                Self::Multiply => sc * dc,
                Self::Screen => 1.0 - (1.0 - sc) * (1.0 - dc),
                Self::DestinationOut => unreachable!("handled before color blending"),
            };
            ((1.0 - da) * sa * sc + (1.0 - sa) * da * dc + sa * da * blended) / out_a
        };

        [
            blend_channel(src[0], dst[0]),
            blend_channel(src[1], dst[1]),
            blend_channel(src[2], dst[2]),
            out_a.clamp(0.0, 1.0),
        ]
    }
}

/// A single stamp or dab emitted along a brush stroke path.
#[derive(Debug, Clone, PartialEq)]
pub struct BrushDab {
    pub center_x: f64,
    pub center_y: f64,
    pub radius: f64,
    pub hardness: f32,
    pub opacity: f32,
    pub color: [f32; 4],
    pub blend_mode: BlendMode,
}

/// Default dab radius in document points (Table B).
pub const DEFAULT_DAB_RADIUS: f64 = 12.0;
/// Default dab hardness in `[0.0, 1.0]` (Table B).
pub const DEFAULT_DAB_HARDNESS: f32 = 0.8;
/// Default paint color (near-black, opaque).
pub const DEFAULT_PAINT_COLOR: [f32; 4] = [0.1, 0.1, 0.1, 1.0];
/// Eraser color (fully transparent).
pub const ERASER_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

impl BrushDab {
    /// Default paint dab at a center point (Table B).
    #[must_use]
    pub fn paint_dab(center_x: f64, center_y: f64) -> Self {
        Self {
            center_x,
            center_y,
            radius: DEFAULT_DAB_RADIUS,
            hardness: DEFAULT_DAB_HARDNESS,
            opacity: 1.0,
            color: DEFAULT_PAINT_COLOR,
            blend_mode: BlendMode::Normal,
        }
    }

    /// Default eraser dab at a center point (Table B).
    #[must_use]
    pub fn eraser_dab(center_x: f64, center_y: f64) -> Self {
        Self {
            center_x,
            center_y,
            radius: DEFAULT_DAB_RADIUS,
            hardness: DEFAULT_DAB_HARDNESS,
            opacity: 1.0,
            color: ERASER_COLOR,
            blend_mode: BlendMode::DestinationOut,
        }
    }
}

impl BrushDab {
    /// Stamping kernel: rasterizes dab coverage directly onto the tile map.
    pub fn stamp_onto(
        &self,
        tile_map: &mut TileMap,
    ) -> Result<(), petunia_design_foundation::PetuniaError> {
        use petunia_design_foundation::PetuniaError;
        if ![self.center_x, self.center_y, self.radius]
            .iter()
            .all(|v| v.is_finite())
            || !self.opacity.is_finite()
            || !self.hardness.is_finite()
            || !self.color.iter().all(|v| v.is_finite())
        {
            return Err(PetuniaError::invalid_input(
                "brush parameters must be finite",
            ));
        }
        if self.radius <= 0.0 || self.opacity <= 0.0 {
            return Ok(());
        }

        // Bound work before integer conversion or allocation. Large strokes
        // must be tiled by the job scheduler, rather than blocking this kernel.
        const MAX_DAB_PIXELS: f64 = 4_194_304.0;
        let span = (2.0 * self.radius + 3.0).ceil();
        if span * span > MAX_DAB_PIXELS {
            return Err(PetuniaError::invalid_input(
                "brush dab exceeds synchronous work budget",
            ));
        }
        let min_coord = f64::from(i32::MIN) * crate::TILE_SIZE as f64;
        let max_coord = (f64::from(i32::MAX) + 1.0) * crate::TILE_SIZE as f64 - 1.0;
        if self.center_x - self.radius < min_coord
            || self.center_y - self.radius < min_coord
            || self.center_x + self.radius > max_coord
            || self.center_y + self.radius > max_coord
        {
            return Err(PetuniaError::invalid_input(
                "brush coordinates exceed tile address range",
            ));
        }

        let min_x = (self.center_x - self.radius).floor() as i64;
        let max_x = (self.center_x + self.radius).ceil() as i64;
        let min_y = (self.center_y - self.radius).floor() as i64;
        let max_y = (self.center_y + self.radius).ceil() as i64;

        let r2 = self.radius * self.radius;
        let inner_r = self.radius * f64::from(self.hardness.clamp(0.0, 1.0));
        let inner_r2 = inner_r * inner_r;

        for py in min_y..=max_y {
            let dy = py as f64 - self.center_y;
            let dy2 = dy * dy;
            if dy2 > r2 {
                continue;
            }

            for px in min_x..=max_x {
                let dx = px as f64 - self.center_x;
                let dist2 = dx * dx + dy2;
                if dist2 > r2 {
                    continue;
                }

                let alpha_factor = if dist2 <= inner_r2 || inner_r >= self.radius {
                    1.0
                } else {
                    let d = dist2.sqrt();
                    let fade = (self.radius - d) / (self.radius - inner_r);
                    fade as f32
                };

                let coverage = self.opacity * alpha_factor;
                let dab_alpha = if self.blend_mode == BlendMode::DestinationOut {
                    coverage
                } else {
                    self.color[3] * coverage
                };
                if dab_alpha <= 0.0 {
                    continue;
                }

                let src_color = [self.color[0], self.color[1], self.color[2], dab_alpha];
                let dst_color = tile_map.get_pixel(px, py);
                let out_color = self.blend_mode.blend(src_color, dst_color);

                if out_color != dst_color && !tile_map.set_pixel(px, py, out_color) {
                    return Err(PetuniaError::invalid_input(
                        "brush tile allocation budget exceeded",
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pixel::{AlphaMode, PixelFormat};

    #[test]
    fn brush_dab_applies_coverage_and_blend() {
        let mut map = TileMap::new(PixelFormat::Rgba8, AlphaMode::Straight);
        let dab = BrushDab {
            center_x: 64.0,
            center_y: 64.0,
            radius: 5.0,
            hardness: 1.0,
            opacity: 1.0,
            color: [1.0, 0.0, 0.0, 1.0], // solid red
            blend_mode: BlendMode::Normal,
        };
        dab.stamp_onto(&mut map).unwrap();

        let center = map.get_pixel(64, 64);
        assert!((center[0] - 1.0).abs() < 0.01);
        assert!((center[3] - 1.0).abs() < 0.01);

        // Outside radius is untouched
        let outside = map.get_pixel(10, 10);
        assert_eq!(outside[3], 0.0);
    }
}
