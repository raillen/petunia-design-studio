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
}

impl BlendMode {
    /// Blends a source pixel over a destination pixel.
    #[must_use]
    pub fn blend(self, src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
        let sa = src[3];
        let da = dst[3];
        let out_a = sa + da * (1.0 - sa);
        if out_a <= 0.0 {
            return [0.0, 0.0, 0.0, 0.0];
        }

        let blend_channel = |sc: f32, dc: f32| -> f32 {
            let blended = match self {
                Self::Normal => sc,
                Self::Multiply => sc * dc,
                Self::Screen => 1.0 - (1.0 - sc) * (1.0 - dc),
            };
            (blended * sa + dc * da * (1.0 - sa)) / out_a
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

impl BrushDab {
    /// Stamping kernel: rasterizes dab coverage directly onto the tile map.
    pub fn stamp_onto(&self, tile_map: &mut TileMap) {
        if self.radius <= 0.0 || self.opacity <= 0.0 {
            return;
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

                let dab_alpha = self.color[3] * self.opacity * alpha_factor;
                if dab_alpha <= 0.0 {
                    continue;
                }

                let src_color = [self.color[0], self.color[1], self.color[2], dab_alpha];
                let dst_color = tile_map.get_pixel(px, py);
                let out_color = self.blend_mode.blend(src_color, dst_color);

                tile_map.set_pixel(px, py, out_color);
            }
        }
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
        dab.stamp_onto(&mut map);

        let center = map.get_pixel(64, 64);
        assert!((center[0] - 1.0).abs() < 0.01);
        assert!((center[3] - 1.0).abs() < 0.01);

        // Outside radius is untouched
        let outside = map.get_pixel(10, 10);
        assert_eq!(outside[3], 0.0);
    }
}
