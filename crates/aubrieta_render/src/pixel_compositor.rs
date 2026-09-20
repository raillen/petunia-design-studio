//! Deterministic pure CPU pixel rasterizer and compositor backend.
//!
//! Provides 8-bit and 16-bit RGBA pixel buffers and compositing algorithms
//! that execute headlessly without a display server or GPU device.

use crate::blend::BlendMode;
use aubrieta_document::Surface;
use aubrieta_geometry::GRect;

/// A contiguous 8-bit RGBA pixel buffer (4 bytes per pixel).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PixelBufferRgba8 {
    /// Buffer width in pixels.
    pub width: u32,
    /// Buffer height in pixels.
    pub height: u32,
    /// Interleaved RGBA pixel data.
    pub data: Vec<u8>,
}

impl PixelBufferRgba8 {
    /// Creates a transparent buffer with specified dimensions.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width as usize) * (height as usize) * 4;
        Self {
            width,
            height,
            data: vec![0; size],
        }
    }

    /// Creates a buffer filled with an initial RGBA color.
    #[must_use]
    pub fn with_fill(width: u32, height: u32, fill: [u8; 4]) -> Self {
        let count = (width as usize) * (height as usize);
        let mut data = Vec::with_capacity(count * 4);
        for _ in 0..count {
            data.extend_from_slice(&fill);
        }
        Self {
            width,
            height,
            data,
        }
    }

    /// Retrieves the RGBA pixel value at (x, y) if in bounds.
    #[must_use]
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x < self.width && y < self.height {
            let idx = ((y as usize * self.width as usize) + x as usize) * 4;
            Some([
                self.data[idx],
                self.data[idx + 1],
                self.data[idx + 2],
                self.data[idx + 3],
            ])
        } else {
            None
        }
    }

    /// Sets the RGBA pixel value at (x, y) directly.
    pub fn set_pixel(&mut self, x: u32, y: u32, pixel: [u8; 4]) {
        if x < self.width && y < self.height {
            let idx = ((y as usize * self.width as usize) + x as usize) * 4;
            self.data[idx..idx + 4].copy_from_slice(&pixel);
        }
    }

    /// Blends a source pixel into (x, y) using the given blend mode and opacity.
    pub fn composite_pixel(
        &mut self,
        x: u32,
        y: u32,
        mut source: [u8; 4],
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        if x < self.width && y < self.height {
            let op = opacity.clamp(0.0, 1.0);
            source[3] = (source[3] as f32 * op + 0.5) as u8;

            let idx = ((y as usize * self.width as usize) + x as usize) * 4;
            let backdrop = [
                self.data[idx],
                self.data[idx + 1],
                self.data[idx + 2],
                self.data[idx + 3],
            ];
            let out = blend_mode.composite_u8(backdrop, source);
            self.data[idx..idx + 4].copy_from_slice(&out);
        }
    }

    /// Fills an axis-aligned rectangle with a color, blend mode, and optional clipping.
    pub fn fill_rect(
        &mut self,
        rect: GRect,
        color: [u8; 4],
        blend_mode: BlendMode,
        opacity: f32,
        clip: Option<GRect>,
    ) {
        let min_x = rect.x0.max(0.0).floor() as u32;
        let min_y = rect.y0.max(0.0).floor() as u32;
        let max_x = (rect.x1.ceil() as u32).min(self.width);
        let max_y = (rect.y1.ceil() as u32).min(self.height);

        for y in min_y..max_y {
            for x in min_x..max_x {
                if let Some(ref c) = clip {
                    if (x as f64) < c.x0
                        || (x as f64) >= c.x1
                        || (y as f64) < c.y0
                        || (y as f64) >= c.y1
                    {
                        continue;
                    }
                }
                self.composite_pixel(x, y, color, blend_mode, opacity);
            }
        }
    }

    /// Blends another pixel buffer onto this buffer with offset and optional clipping.
    pub fn composite_buffer(
        &mut self,
        src: &PixelBufferRgba8,
        offset_x: i32,
        offset_y: i32,
        blend_mode: BlendMode,
        opacity: f32,
        clip: Option<GRect>,
    ) {
        for sy in 0..src.height {
            let dy = offset_y + sy as i32;
            if dy < 0 || dy >= self.height as i32 {
                continue;
            }
            for sx in 0..src.width {
                let dx = offset_x + sx as i32;
                if dx < 0 || dx >= self.width as i32 {
                    continue;
                }

                if let Some(ref c) = clip {
                    if (dx as f64) < c.x0
                        || (dx as f64) >= c.x1
                        || (dy as f64) < c.y0
                        || (dy as f64) >= c.y1
                    {
                        continue;
                    }
                }

                if let Some(px) = src.get_pixel(sx, sy) {
                    self.composite_pixel(dx as u32, dy as u32, px, blend_mode, opacity);
                }
            }
        }
    }
}

/// A contiguous 16-bit RGBA pixel buffer (8 bytes per pixel) for high-precision deep color.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PixelBufferRgba16 {
    /// Buffer width in pixels.
    pub width: u32,
    /// Buffer height in pixels.
    pub height: u32,
    /// Interleaved RGBA 16-bit pixel data.
    pub data: Vec<u16>,
}

impl PixelBufferRgba16 {
    /// Creates a transparent 16-bit buffer with specified dimensions.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width as usize) * (height as usize) * 4;
        Self {
            width,
            height,
            data: vec![0; size],
        }
    }

    /// Creates a 16-bit buffer filled with an initial RGBA color.
    #[must_use]
    pub fn with_fill(width: u32, height: u32, fill: [u16; 4]) -> Self {
        let count = (width as usize) * (height as usize);
        let mut data = Vec::with_capacity(count * 4);
        for _ in 0..count {
            data.extend_from_slice(&fill);
        }
        Self {
            width,
            height,
            data,
        }
    }

    /// Retrieves the 16-bit RGBA pixel value at (x, y) if in bounds.
    #[must_use]
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<[u16; 4]> {
        if x < self.width && y < self.height {
            let idx = ((y as usize * self.width as usize) + x as usize) * 4;
            Some([
                self.data[idx],
                self.data[idx + 1],
                self.data[idx + 2],
                self.data[idx + 3],
            ])
        } else {
            None
        }
    }

    /// Sets the 16-bit RGBA pixel value at (x, y) directly.
    pub fn set_pixel(&mut self, x: u32, y: u32, pixel: [u16; 4]) {
        if x < self.width && y < self.height {
            let idx = ((y as usize * self.width as usize) + x as usize) * 4;
            self.data[idx..idx + 4].copy_from_slice(&pixel);
        }
    }

    /// Blends a 16-bit source pixel into (x, y) using the given blend mode and opacity.
    pub fn composite_pixel(
        &mut self,
        x: u32,
        y: u32,
        mut source: [u16; 4],
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        if x < self.width && y < self.height {
            let op = opacity.clamp(0.0, 1.0);
            source[3] = (source[3] as f32 * op + 0.5) as u16;

            let idx = ((y as usize * self.width as usize) + x as usize) * 4;
            let backdrop = [
                self.data[idx],
                self.data[idx + 1],
                self.data[idx + 2],
                self.data[idx + 3],
            ];
            let out = blend_mode.composite_u16(backdrop, source);
            self.data[idx..idx + 4].copy_from_slice(&out);
        }
    }
}

/// Headless software compositor rendering documents to pixel buffers.
#[derive(Debug, Default)]
pub struct SoftwarePixelCompositor;

impl SoftwarePixelCompositor {
    /// Renders a surface's objects into an 8-bit RGBA pixel buffer.
    #[must_use]
    pub fn render_surface_rgba8(
        surface: &Surface,
        width: u32,
        height: u32,
        background: [u8; 4],
    ) -> PixelBufferRgba8 {
        let mut buffer = PixelBufferRgba8::with_fill(width, height, background);

        for (i, obj) in surface.objects.iter().enumerate() {
            let fill_color = obj
                .fill
                .as_deref()
                .map(parse_color_token)
                .unwrap_or([180, 180, 180, 255]);

            // For headless demonstration: place objects along an incremental grid
            let offset_x = (i as f64 * 32.0).min(width as f64 - 32.0);
            let offset_y = (i as f64 * 32.0).min(height as f64 - 32.0);
            let rect = GRect::new(offset_x, offset_y, offset_x + 64.0, offset_y + 64.0);

            buffer.fill_rect(rect, fill_color, BlendMode::Normal, 1.0, None);
        }

        buffer
    }
}

/// Basic color parser mapping semantic token names or hex colors to RGBA8.
fn parse_color_token(token: &str) -> [u8; 4] {
    if token.starts_with('#') {
        let hex = token.trim_start_matches('#');
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return [r, g, b, 255];
            }
        }
    }
    match token {
        "aubrieta.red/500" => [239, 68, 68, 255],
        "aubrieta.blue/500" => [59, 130, 246, 255],
        "aubrieta.green/500" => [34, 197, 94, 255],
        "aubrieta.yellow/500" => [234, 179, 8, 255],
        _ => [128, 128, 128, 255],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aubrieta_document::DocumentObject;
    use aubrieta_foundation::IdGenerator;

    #[test]
    fn buffer_rgba8_fill_rect_and_compositing() {
        let mut buf = PixelBufferRgba8::new(10, 10);
        let rect = GRect::new(2.0, 2.0, 6.0, 6.0);
        let red = [255, 0, 0, 255];

        buf.fill_rect(rect, red, BlendMode::Normal, 1.0, None);

        // Outside rectangle: transparent
        assert_eq!(buf.get_pixel(0, 0), Some([0, 0, 0, 0]));
        // Inside rectangle: red
        assert_eq!(buf.get_pixel(2, 2), Some([255, 0, 0, 255]));
        assert_eq!(buf.get_pixel(5, 5), Some([255, 0, 0, 255]));
    }

    #[test]
    fn buffer_composite_another_buffer_with_opacity() {
        let mut dest = PixelBufferRgba8::with_fill(10, 10, [0, 0, 0, 255]); // black
        let src = PixelBufferRgba8::with_fill(10, 10, [255, 255, 255, 255]); // white

        dest.composite_buffer(&src, 0, 0, BlendMode::Normal, 0.5, None);

        let px = dest.get_pixel(5, 5).unwrap();
        // 0.5 * 255 ~ 128
        assert!((px[0] as i32 - 128).abs() <= 1);
        assert!((px[1] as i32 - 128).abs() <= 1);
        assert!((px[2] as i32 - 128).abs() <= 1);
        assert_eq!(px[3], 255);
    }

    #[test]
    fn software_compositor_renders_surface() {
        let mut gen = IdGenerator::new();
        let mut surface = Surface::new(gen.next_surface(), "TestPage");
        let mut obj = DocumentObject::new(gen.next_object(), "Box");
        obj.fill = Some("aubrieta.red/500".to_string());
        surface.objects.push(obj);

        let buf =
            SoftwarePixelCompositor::render_surface_rgba8(&surface, 100, 100, [255, 255, 255, 255]);
        assert_eq!(buf.width, 100);
        assert_eq!(buf.height, 100);
        let px = buf.get_pixel(10, 10).unwrap();
        assert_eq!(px, [239, 68, 68, 255]);
    }
}
