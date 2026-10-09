//! Editor overlays: abstract UI geometry rasterized after output.
//!
//! The engine/UI describes overlays; render decides final pixels.
//! Overlays composite after the output transform so selection colors
//! never pass through the document print profile, and pixel snapping
//! happens in device space without touching document geometry.

/// Abstract overlay primitives from engine/UI descriptions.
#[derive(Debug, Clone, PartialEq)]
pub enum OverlayPrimitive {
    Line {
        from: (f64, f64),
        to: (f64, f64),
    },
    Polyline {
        points: Vec<(f64, f64)>,
    },
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
    Handle {
        at: (f64, f64),
    },
    Anchor {
        at: (f64, f64),
    },
    TextLabel {
        at: (f64, f64),
        text: String,
    },
    CursorShape {
        at: (f64, f64),
        radius: f64,
    },
    HighlightRegion {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
}

/// One 8-bit RGBA device target for overlay drawing.
#[derive(Debug, Clone)]
pub struct OverlayTarget {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl OverlayTarget {
    /// Allocate a target from an existing RGBA8 buffer.
    pub fn from_rgba8(width: u32, height: u32, pixels: Vec<u8>) -> Option<Self> {
        if pixels.len() != (width as usize) * (height as usize) * 4 {
            return None;
        }
        Some(Self {
            width,
            height,
            pixels,
        })
    }

    /// Snap a device coordinate to the pixel grid for crisp 1px UI.
    #[must_use]
    pub fn snap(value: f64) -> f64 {
        value.round()
    }

    fn plot(&mut self, x: i64, y: i64, color: [u8; 4]) {
        if x < 0 || y < 0 || x >= self.width as i64 || y >= self.height as i64 {
            return;
        }
        let index = ((y as u32 * self.width + x as u32) * 4) as usize;
        // Opaque UI colors replace presentation pixels directly.
        self.pixels[index..index + 4].copy_from_slice(&color);
    }

    /// Bresenham line in device space.
    pub fn line(&mut self, from: (f64, f64), to: (f64, f64), color: [u8; 4]) {
        let (mut x0, mut y0) = (Self::snap(from.0) as i64, Self::snap(from.1) as i64);
        let (x1, y1) = (Self::snap(to.0) as i64, Self::snap(to.1) as i64);
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut error = dx + dy;
        loop {
            self.plot(x0, y0, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let doubled = 2 * error;
            if doubled >= dy {
                error += dy;
                x0 += sx;
            }
            if doubled <= dx {
                error += dx;
                y0 += sy;
            }
        }
    }

    /// 1px rectangle outline.
    pub fn rect_outline(&mut self, x: f64, y: f64, width: f64, height: f64, color: [u8; 4]) {
        let (x0, y0) = (Self::snap(x), Self::snap(y));
        let (x1, y1) = (Self::snap(x + width), Self::snap(y + height));
        self.line((x0, y0), (x1, y0), color);
        self.line((x1, y0), (x1, y1), color);
        self.line((x1, y1), (x0, y1), color);
        self.line((x0, y1), (x0, y0), color);
    }

    /// 5×5 centered handle square.
    pub fn handle(&mut self, at: (f64, f64), color: [u8; 4]) {
        let (cx, cy) = (Self::snap(at.0) as i64, Self::snap(at.1) as i64);
        for dy in -2..=2 {
            for dx in -2..=2 {
                self.plot(cx + dx, cy + dy, color);
            }
        }
    }

    /// Draw one abstract primitive in a UI color.
    pub fn draw(&mut self, primitive: &OverlayPrimitive, color: [u8; 4]) {
        match primitive {
            OverlayPrimitive::Line { from, to } => self.line(*from, *to, color),
            OverlayPrimitive::Polyline { points } => {
                for pair in points.windows(2) {
                    self.line(pair[0], pair[1], color);
                }
            }
            OverlayPrimitive::Rect {
                x,
                y,
                width,
                height,
            } => self.rect_outline(*x, *y, *width, *height, color),
            OverlayPrimitive::Handle { at } | OverlayPrimitive::Anchor { at } => {
                self.handle(*at, color)
            }
            OverlayPrimitive::CursorShape { at, radius } => {
                let steps = 24;
                for step in 0..steps {
                    let a0 = step as f64 / steps as f64 * std::f64::consts::TAU;
                    let a1 = (step + 1) as f64 / steps as f64 * std::f64::consts::TAU;
                    self.line(
                        (at.0 + radius * a0.cos(), at.1 + radius * a0.sin()),
                        (at.0 + radius * a1.cos(), at.1 + radius * a1.sin()),
                        color,
                    );
                }
            }
            OverlayPrimitive::HighlightRegion {
                x,
                y,
                width,
                height,
            } => self.rect_outline(*x, *y, *width, *height, color),
            OverlayPrimitive::TextLabel { .. } => {
                // Text rasterization for labels belongs to the text
                // pipeline; the primitive stays reserved here.
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target() -> OverlayTarget {
        OverlayTarget {
            width: 16,
            height: 16,
            pixels: vec![0u8; 16 * 16 * 4],
        }
    }

    fn alpha_at(target: &OverlayTarget, x: u32, y: u32) -> u8 {
        target.pixels[((y * target.width + x) * 4 + 3) as usize]
    }

    #[test]
    fn lines_and_rects_land_on_device_pixels() {
        let mut target = target();
        let white = [255, 255, 255, 255];
        target.draw(
            &OverlayPrimitive::Rect {
                x: 2.0,
                y: 2.0,
                width: 6.0,
                height: 6.0,
            },
            white,
        );
        assert_eq!(alpha_at(&target, 2, 2), 255);
        assert_eq!(alpha_at(&target, 8, 8), 255);
        assert_eq!(alpha_at(&target, 5, 5), 0);
    }

    #[test]
    fn handles_cover_a_five_pixel_square() {
        let mut target = target();
        target.draw(
            &OverlayPrimitive::Handle { at: (8.0, 8.0) },
            [255, 0, 0, 255],
        );
        assert_eq!(alpha_at(&target, 8, 8), 255);
        assert_eq!(alpha_at(&target, 6, 6), 255);
        assert_eq!(alpha_at(&target, 5, 5), 0);
    }
}
