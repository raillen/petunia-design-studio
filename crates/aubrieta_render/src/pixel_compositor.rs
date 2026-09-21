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
    /// Renders a surface's objects into an 8-bit RGBA pixel buffer (F-03/F-04).
    /// Consumes the canonical document state: `visible`, `bounds`,
    /// `effective_appearance()` (fills, opacity, blend), `is_clip_mask` /
    /// `clip_mask_id` clipping, and effect `bounds_inflation`.
    /// Mask boundary objects (`is_clip_mask`) are not painted themselves;
    /// content referencing them via `clip_mask_id` is clipped to the mask
    /// bounds. Objects without bounds are skipped.
    #[must_use]
    pub fn render_surface_rgba8(
        surface: &Surface,
        width: u32,
        height: u32,
        background: [u8; 4],
    ) -> PixelBufferRgba8 {
        let mut buffer = PixelBufferRgba8::with_fill(width, height, background);

        for obj in surface.objects().iter() {
            if !obj.visible {
                continue;
            }
            // Mask boundaries define clips; they are not painted (10.5).
            if obj.is_clip_mask {
                continue;
            }
            let bounds = match obj.bounds {
                Some(b) => b,
                None => continue,
            };
            // Resolve clip from the referenced mask object, if any.
            let mut clip: Option<GRect> = None;
            if let Some(mask_id) = obj.clip_mask_id {
                match surface.objects().iter().find(|o| o.id == mask_id) {
                    Some(mask) => match mask.bounds {
                        Some(mb) => {
                            clip = Some(GRect::new(mb[0], mb[1], mb[0] + mb[2], mb[1] + mb[3]));
                        }
                        None => continue,
                    },
                    None => continue,
                }
            }
            let eff = obj.effective_appearance();
            // Opacity: stack opacity x primary entry opacity (10.4 order).
            let entry_opacity = eff
                .primary_fill()
                .map(|f| f.opacity)
                .or_else(|| eff.primary_stroke().map(|s| s.opacity))
                .unwrap_or(1.0);
            let opacity = (eff.opacity * entry_opacity).clamp(0.0, 1.0) as f32;
            if opacity <= 0.0 {
                continue;
            }
            // Blend: primary entry blend, falling back to stack blend.
            let doc_blend = eff
                .primary_fill()
                .map(|f| f.blend_mode)
                .or_else(|| eff.primary_stroke().map(|s| s.blend_mode))
                .unwrap_or(eff.blend_mode);
            let blend_mode = BlendMode::from(doc_blend);
            // Fill color: primary fill paint sampled at center; when there is
            // no fill but a stroke exists, preview with the stroke color.
            let fill_color: Option<[u8; 4]> = eff
                .primary_fill()
                .and_then(|f| {
                    paint_to_rgba8(&f.paint, 0.5, f.opacity as f32 * eff.opacity as f32)
                })
                .or_else(|| {
                    eff.primary_stroke().and_then(|s| {
                        paint_to_rgba8(&s.paint, 0.5, s.opacity as f32 * eff.opacity as f32)
                    })
                })
                .or_else(|| {
                    obj.fill
                        .as_deref()
                        .map(|t| token_to_rgba8(t, obj.opacity as f32))
                });
            let fill_color = match fill_color {
                Some(c) => c,
                None => continue,
            };
            // Effect inflation expands the painted rect (F-12).
            let inflation = eff.bounds_inflation();
            let rect = GRect::new(
                bounds[0] - inflation,
                bounds[1] - inflation,
                bounds[0] + bounds[2] + inflation,
                bounds[1] + bounds[3] + inflation,
            );

            // Drop shadows paint first (behind the object) as offset fills
            // (F-12). Blur radius is approximated by the inflated footprint:
            // the headless CPU compositor has no kernel-blur pass, so soft
            // edges degrade to solid-offset silhouettes. GaussianBlur and
            // InnerShadow consume `bounds_inflation` for planning but have no
            // pixel pass here by contract (documented approximation).
            for effect in eff.effects.iter().filter(|e| e.visible) {
                if let aubrieta_document::EffectKind::DropShadow {
                    offset,
                    color,
                    opacity: shadow_opacity,
                    ..
                } = &effect.kind
                {
                    let shadow_rect = GRect::new(
                        bounds[0] + offset[0],
                        bounds[1] + offset[1],
                        bounds[0] + bounds[2] + offset[0],
                        bounds[1] + bounds[3] + offset[1],
                    );
                    let shadow_color = token_to_rgba8(
                        color,
                        (*shadow_opacity as f32).clamp(0.0, 1.0) * eff.opacity as f32,
                    );
                    buffer.fill_rect(shadow_rect, shadow_color, blend_mode, opacity, clip);
                }
            }

            buffer.fill_rect(rect, fill_color, blend_mode, opacity, clip);
        }

        buffer
    }
}

/// Converts a `Paint` to premultiplied-by-opacity RGBA8 via center sampling.
fn paint_to_rgba8(
    paint: &aubrieta_document::Paint,
    t: f64,
    opacity: f32,
) -> Option<[u8; 4]> {
    match paint {
        aubrieta_document::Paint::None => None,
        aubrieta_document::Paint::Solid(token) => Some(token_to_rgba8(token, opacity)),
        aubrieta_document::Paint::LinearGradient(g) => {
            let (rgb, a) = g.sample_rgba(t)?;
            Some([
                (rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8,
                (rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8,
                (rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8,
                (a.clamp(0.0, 1.0) * opacity.clamp(0.0, 1.0) * 255.0).round() as u8,
            ])
        }
        aubrieta_document::Paint::RadialGradient(g) => {
            let (rgb, a) = g.sample_rgba(t)?;
            Some([
                (rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8,
                (rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8,
                (rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8,
                (a.clamp(0.0, 1.0) * opacity.clamp(0.0, 1.0) * 255.0).round() as u8,
            ])
        }
    }
}

/// Token/literal to RGBA8 with explicit opacity factor.
/// Resolves via `aubrieta_document::resolve_color_to_rgb` (F-05) so all
/// documented literals work; unknown tokens fall back to mid-gray.
fn token_to_rgba8(token: &str, opacity: f32) -> [u8; 4] {
    let rgb = aubrieta_document::resolve_color_to_rgb(token);
    let a = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
    [
        (rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        a,
    ]
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
        let surface_id = gen.next_surface();
        let mut obj = DocumentObject::new(gen.next_object(), "Box");
        obj.fill = Some("aubrieta.red/500".to_string());
        obj.bounds = Some([0.0, 0.0, 64.0, 64.0]);
        let surface = Surface::with_objects(surface_id, "TestPage", vec![obj]);

        let buf =
            SoftwarePixelCompositor::render_surface_rgba8(&surface, 100, 100, [255, 255, 255, 255]);
        assert_eq!(buf.width, 100);
        assert_eq!(buf.height, 100);
        let px = buf.get_pixel(10, 10).unwrap();
        let expected = token_to_rgba8("aubrieta.red/500", 1.0);
        assert_eq!(px, expected);
    }

    #[test]
    fn compositor_respects_bounds_visibility_opacity_and_clip() {
        let mut gen = IdGenerator::new();
        let surface_id = gen.next_surface();
        // Visible box at (10,10,20x20).
        let mut box_obj = DocumentObject::new(gen.next_object(), "Box");
        box_obj.fill = Some("aubrieta.blue/500".to_string());
        box_obj.bounds = Some([10.0, 10.0, 20.0, 20.0]);
        // Hidden box must not paint.
        let mut hidden = DocumentObject::new(gen.next_object(), "Hidden");
        hidden.fill = Some("aubrieta.red/500".to_string());
        hidden.bounds = Some([10.0, 10.0, 20.0, 20.0]);
        hidden.visible = false;
        // Fully transparent box elsewhere must not paint.
        let mut ghost = DocumentObject::new(gen.next_object(), "Ghost");
        ghost.fill = Some("aubrieta.red/500".to_string());
        ghost.bounds = Some([60.0, 60.0, 20.0, 20.0]);
        ghost.opacity = 0.0;
        let surface = Surface::with_objects(surface_id, "Clip", vec![box_obj, hidden, ghost]);

        let buf =
            SoftwarePixelCompositor::render_surface_rgba8(&surface, 100, 100, [255, 255, 255, 255]);
        let expected_blue = token_to_rgba8("aubrieta.blue/500", 1.0);
        assert_eq!(buf.get_pixel(15, 15).unwrap(), expected_blue);
        // Ghost area stays background.
        assert_eq!(buf.get_pixel(65, 65).unwrap(), [255, 255, 255, 255]);
    }

    #[test]
    fn compositor_skips_mask_boundary_and_clips_content() {
        let mut gen = IdGenerator::new();
        let surface_id = gen.next_surface();
        let mask_id = gen.next_object();
        let mut mask = DocumentObject::new(mask_id, "Mask");
        mask.fill = Some("aubrieta.red/500".to_string());
        mask.bounds = Some([10.0, 10.0, 20.0, 20.0]);
        mask.is_clip_mask = true;
        let mut content = DocumentObject::new(gen.next_object(), "Content");
        content.fill = Some("aubrieta.blue/500".to_string());
        content.bounds = Some([0.0, 0.0, 100.0, 100.0]);
        content.clip_mask_id = Some(mask_id);
        let surface = Surface::with_objects(surface_id, "Mask", vec![mask, content]);

        let buf =
            SoftwarePixelCompositor::render_surface_rgba8(&surface, 100, 100, [255, 255, 255, 255]);
        let expected_blue = token_to_rgba8("aubrieta.blue/500", 1.0);
        // Inside mask bounds: content paints.
        assert_eq!(buf.get_pixel(15, 15).unwrap(), expected_blue);
        // Outside mask bounds: background (content clipped, mask not painted).
        assert_eq!(buf.get_pixel(5, 5).unwrap(), [255, 255, 255, 255]);
    }
}
