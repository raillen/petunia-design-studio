//! Deterministic pure CPU pixel rasterizer and compositor backend.
//!
//! Provides 8-bit and 16-bit RGBA pixel buffers and compositing algorithms
//! that execute headlessly without a display server or GPU device.

use std::collections::HashMap;

use crate::blend::BlendMode;
use petunia_design_document::{Change, ChangeSet, Surface};
use petunia_design_geometry::GRect;

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

    /// Clears an axis-aligned rectangle with a solid color, directly setting pixel bytes.
    pub fn clear_rect(&mut self, rect: GRect, color: [u8; 4]) {
        let min_x = (rect.x0.max(0.0).floor() as u32).min(self.width);
        let min_y = (rect.y0.max(0.0).floor() as u32).min(self.height);
        let max_x = (rect.x1.max(0.0).ceil() as u32).min(self.width);
        let max_y = (rect.y1.max(0.0).ceil() as u32).min(self.height);

        if min_x >= max_x || min_y >= max_y {
            return;
        }

        for y in min_y..max_y {
            let start = ((y as usize * self.width as usize) + min_x as usize) * 4;
            let end = ((y as usize * self.width as usize) + max_x as usize) * 4;
            for chunk in self.data[start..end].as_chunks_mut::<4>().0 {
                chunk.copy_from_slice(&color);
            }
        }
    }

    /// Fills an axis-aligned rectangle with a color, blend mode, and optional clipping.
    /// Clamps bounds before the pixel loop, eliminating per-pixel branching overhead.
    pub fn fill_rect(
        &mut self,
        rect: GRect,
        color: [u8; 4],
        blend_mode: BlendMode,
        opacity: f32,
        clip: Option<GRect>,
    ) {
        let mut min_x = (rect.x0.max(0.0).floor() as u32).min(self.width);
        let mut min_y = (rect.y0.max(0.0).floor() as u32).min(self.height);
        let mut max_x = (rect.x1.max(0.0).ceil() as u32).min(self.width);
        let mut max_y = (rect.y1.max(0.0).ceil() as u32).min(self.height);

        if let Some(ref c) = clip {
            if c.x0 >= c.x1 || c.y0 >= c.y1 {
                return;
            }
            let clip_min_x = (c.x0.max(0.0).ceil() as u32).min(self.width);
            let clip_min_y = (c.y0.max(0.0).ceil() as u32).min(self.height);
            let clip_max_x = (c.x1.max(0.0).ceil() as u32).min(self.width);
            let clip_max_y = (c.y1.max(0.0).ceil() as u32).min(self.height);

            min_x = min_x.max(clip_min_x);
            min_y = min_y.max(clip_min_y);
            max_x = max_x.min(clip_max_x);
            max_y = max_y.min(clip_max_y);
        }

        if min_x >= max_x || min_y >= max_y {
            return;
        }

        let op = opacity.clamp(0.0, 1.0);
        // Fast-path: fully opaque Normal blend directly overwrites row slices
        if blend_mode == BlendMode::Normal && (op - 1.0).abs() < 1e-5 && color[3] == 255 {
            for y in min_y..max_y {
                let start = ((y as usize * self.width as usize) + min_x as usize) * 4;
                let end = ((y as usize * self.width as usize) + max_x as usize) * 4;
                for chunk in self.data[start..end].as_chunks_mut::<4>().0 {
                    chunk.copy_from_slice(&color);
                }
            }
        } else {
            for y in min_y..max_y {
                for x in min_x..max_x {
                    self.composite_pixel(x, y, color, blend_mode, opacity);
                }
            }
        }
    }

    /// Blends another pixel buffer onto this buffer with offset and optional clipping.
    /// Intersects source and destination spans ahead of the pixel loop.
    pub fn composite_buffer(
        &mut self,
        src: &PixelBufferRgba8,
        offset_x: i32,
        offset_y: i32,
        blend_mode: BlendMode,
        opacity: f32,
        clip: Option<GRect>,
    ) {
        let mut min_dx = offset_x.max(0);
        let mut min_dy = offset_y.max(0);
        let mut max_dx = (offset_x + src.width as i32).min(self.width as i32);
        let mut max_dy = (offset_y + src.height as i32).min(self.height as i32);

        if let Some(ref c) = clip {
            if c.x0 >= c.x1 || c.y0 >= c.y1 {
                return;
            }
            let clip_min_x = (c.x0.max(0.0).ceil() as i32).min(self.width as i32);
            let clip_min_y = (c.y0.max(0.0).ceil() as i32).min(self.height as i32);
            let clip_max_x = (c.x1.max(0.0).ceil() as i32).min(self.width as i32);
            let clip_max_y = (c.y1.max(0.0).ceil() as i32).min(self.height as i32);

            min_dx = min_dx.max(clip_min_x);
            min_dy = min_dy.max(clip_min_y);
            max_dx = max_dx.min(clip_max_x);
            max_dy = max_dy.min(clip_max_y);
        }

        if min_dx >= max_dx || min_dy >= max_dy {
            return;
        }

        for dy in min_dy..max_dy {
            let sy = (dy - offset_y) as u32;
            for dx in min_dx..max_dx {
                let sx = (dx - offset_x) as u32;
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
        let viewport = GRect::new(0.0, 0.0, width as f64, height as f64);
        Self::render_surface_dirty_rgba8(surface, &mut buffer, viewport, None);
        buffer
    }

    /// Incrementally renders objects into an existing 8-bit RGBA pixel buffer,
    /// constrained to `dirty_rect`.
    ///
    /// If `background` is provided, pixels inside `dirty_rect` are cleared
    /// before redrawing.
    ///
    /// Visual elements outside `dirty_rect` are culled with zero pixel iterations.
    /// Clip mask relationships are pre-indexed to O(1) per object.
    pub fn render_surface_dirty_rgba8(
        surface: &Surface,
        buffer: &mut PixelBufferRgba8,
        dirty_rect: GRect,
        background: Option<[u8; 4]>,
    ) {
        if !dirty_rect.is_finite()
            || dirty_rect.x0 >= dirty_rect.x1
            || dirty_rect.y0 >= dirty_rect.y1
        {
            return;
        }

        let buffer_rect = GRect::new(0.0, 0.0, buffer.width as f64, buffer.height as f64);
        let dirty_rect = match dirty_rect.intersection(buffer_rect) {
            Some(r) => r,
            None => return,
        };

        if let Some(bg) = background {
            buffer.clear_rect(dirty_rect, bg);
        }

        // Pre-index clip masks for O(1) lookup across the render loop (eliminating O(N^2) scan).
        let mut mask_map = HashMap::new();
        for obj in surface.objects().iter() {
            if obj.is_clip_mask {
                if let Some(mb) = obj.bounds {
                    mask_map.insert(
                        obj.id,
                        GRect::new(mb[0], mb[1], mb[0] + mb[2], mb[1] + mb[3]),
                    );
                }
            }
        }

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
                match mask_map.get(&mask_id) {
                    Some(&mask_rect) => {
                        clip = Some(mask_rect);
                    }
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
                .and_then(|f| paint_to_rgba8(&f.paint, 0.5, 1.0))
                .or_else(|| {
                    eff.primary_stroke()
                        .and_then(|s| paint_to_rgba8(&s.paint, 0.5, 1.0))
                })
                .or_else(|| obj.fill.as_deref().map(|t| token_to_rgba8(t, 1.0)));
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

            // Compute total visual envelope including drop shadows for culling
            let mut visual_envelope = rect;
            for effect in eff.effects.iter().filter(|e| e.visible) {
                if let petunia_design_document::EffectKind::DropShadow { offset, .. } = &effect.kind
                {
                    let shadow_rect = GRect::new(
                        bounds[0] + offset[0] - inflation,
                        bounds[1] + offset[1] - inflation,
                        bounds[0] + bounds[2] + offset[0] + inflation,
                        bounds[1] + bounds[3] + offset[1] + inflation,
                    );
                    if let Some(u) = visual_envelope.union(shadow_rect) {
                        visual_envelope = u;
                    }
                }
            }

            // Viewport & dirty-rect culling: if clip is active, intersect with it
            if let Some(c) = clip {
                visual_envelope = match visual_envelope.intersection(c) {
                    Some(inter) => inter,
                    None => continue,
                };
            }

            // Early-out if the object's visual footprint is disjoint from the dirty rectangle
            if visual_envelope.intersection(dirty_rect).is_none() {
                continue;
            }

            // Constrain painting within the dirty rectangle and any mask clip
            let effective_clip = match clip {
                Some(c) => match c.intersection(dirty_rect) {
                    Some(inter) => inter,
                    None => continue,
                },
                None => dirty_rect,
            };

            // Drop shadows paint first (behind the object) as offset fills
            // (F-12). Blur radius is approximated by the inflated footprint:
            // the headless CPU compositor has no kernel-blur pass, so soft
            // edges degrade to solid-offset silhouettes. GaussianBlur and
            // InnerShadow consume `bounds_inflation` for planning but have no
            // pixel pass here by contract (documented approximation).
            for effect in eff.effects.iter().filter(|e| e.visible) {
                if let petunia_design_document::EffectKind::DropShadow {
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
                    buffer.fill_rect(
                        shadow_rect,
                        shadow_color,
                        blend_mode,
                        opacity,
                        Some(effective_clip),
                    );
                }
            }

            buffer.fill_rect(rect, fill_color, blend_mode, opacity, Some(effective_clip));
        }
    }

    /// Calculates the tightest axis-aligned dirty rectangle affected by a [`ChangeSet`].
    /// Returns `None` if the changeset is empty or had no visible impact on this surface.
    #[must_use]
    pub fn dirty_rect_for_changeset(surface: &Surface, changeset: &ChangeSet) -> Option<GRect> {
        let mut dirty: Option<GRect> = None;

        let mut union_rect = |rect: GRect| {
            if let Some(d) = dirty {
                dirty = d.union(rect);
            } else {
                dirty = Some(rect);
            }
        };

        let mut union_bounds = |bounds: [f64; 4], inflation: f64| {
            union_rect(GRect::new(
                bounds[0] - inflation,
                bounds[1] - inflation,
                bounds[0] + bounds[2] + inflation,
                bounds[1] + bounds[3] + inflation,
            ));
        };

        for change in &changeset.changes {
            match change {
                Change::ObjectAdded {
                    surface: s_id,
                    object,
                    ..
                }
                | Change::ObjectRemoved {
                    surface: s_id,
                    object,
                    ..
                } => {
                    if *s_id == surface.id {
                        if let Some(b) = object.bounds {
                            let inf = object.effective_appearance().bounds_inflation();
                            union_bounds(b, inf);
                        }
                    }
                }
                Change::BoundsChanged {
                    id,
                    previous_bounds,
                    next_bounds,
                    ..
                } => {
                    if let Some(pb) = previous_bounds {
                        union_bounds(*pb, 0.0);
                    }
                    if let Some(nb) = next_bounds {
                        let inf = surface
                            .objects()
                            .iter()
                            .find(|o| o.id == *id)
                            .map(|o| o.effective_appearance().bounds_inflation())
                            .unwrap_or(0.0);
                        union_bounds(*nb, inf);
                    }
                }
                Change::FillChanged { id, .. }
                | Change::StrokeChanged { id, .. }
                | Change::ShapeChanged { id, .. }
                | Change::AppearanceChanged { id, .. }
                | Change::VisibilityChanged { id, .. }
                | Change::OpacityChanged { id, .. }
                | Change::ModifiersChanged { id, .. }
                | Change::ChildrenChanged { id, .. }
                | Change::ClipMaskChanged { id, .. }
                | Change::Reparented { id, .. } => {
                    if let Some(obj) = surface.objects().iter().find(|o| o.id == *id) {
                        if let Some(b) = obj.bounds {
                            let inf = obj.effective_appearance().bounds_inflation();
                            union_bounds(b, inf);
                        }
                    }
                }
                Change::ObjectReordered {
                    surface: s_id, id, ..
                } => {
                    if *s_id == surface.id {
                        if let Some(obj) = surface.objects().iter().find(|o| o.id == *id) {
                            if let Some(b) = obj.bounds {
                                let inf = obj.effective_appearance().bounds_inflation();
                                union_bounds(b, inf);
                            }
                        }
                    }
                }
                Change::SurfaceAdded { id, .. }
                | Change::SurfaceGeometryChanged { id, .. }
                | Change::SurfaceBleedChanged { id, .. }
                | Change::SurfaceMarginsChanged { id, .. }
                | Change::SurfaceBackgroundChanged { id, .. }
                    if *id == surface.id =>
                {
                    let sb = surface.bounds();
                    union_bounds(sb, 0.0);
                }
                _ => {}
            }
        }

        dirty
    }
}

/// Converts a `Paint` to premultiplied-by-opacity RGBA8 via center sampling.
fn paint_to_rgba8(paint: &petunia_design_document::Paint, t: f64, opacity: f32) -> Option<[u8; 4]> {
    match paint {
        petunia_design_document::Paint::None => None,
        petunia_design_document::Paint::Solid(token) => Some(token_to_rgba8(token, opacity)),
        petunia_design_document::Paint::LinearGradient(g) => {
            let (rgb, a) = g.sample_rgba(t)?;
            Some([
                (rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8,
                (rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8,
                (rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8,
                (a.clamp(0.0, 1.0) * opacity.clamp(0.0, 1.0) * 255.0).round() as u8,
            ])
        }
        petunia_design_document::Paint::RadialGradient(g) => {
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
/// Resolves via `petunia_design_document::resolve_color_to_rgb` (F-05) so all
/// documented literals work; unknown tokens fall back to mid-gray.
fn token_to_rgba8(token: &str, opacity: f32) -> [u8; 4] {
    let rgb = petunia_design_document::resolve_color_to_rgb(token);
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
    use petunia_design_document::DocumentObject;
    use petunia_design_foundation::IdGenerator;

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
        obj.fill = Some("ptnd.red/500".to_string());
        obj.bounds = Some([0.0, 0.0, 64.0, 64.0]);
        let surface = Surface::with_objects(surface_id, "TestPage", vec![obj]);

        let buf =
            SoftwarePixelCompositor::render_surface_rgba8(&surface, 100, 100, [255, 255, 255, 255]);
        assert_eq!(buf.width, 100);
        assert_eq!(buf.height, 100);
        let px = buf.get_pixel(10, 10).unwrap();
        let expected = token_to_rgba8("ptnd.red/500", 1.0);
        assert_eq!(px, expected);
    }

    #[test]
    fn compositor_respects_bounds_visibility_opacity_and_clip() {
        let mut gen = IdGenerator::new();
        let surface_id = gen.next_surface();
        // Visible box at (10,10,20x20).
        let mut box_obj = DocumentObject::new(gen.next_object(), "Box");
        box_obj.fill = Some("ptnd.blue/500".to_string());
        box_obj.bounds = Some([10.0, 10.0, 20.0, 20.0]);
        // Hidden box must not paint.
        let mut hidden = DocumentObject::new(gen.next_object(), "Hidden");
        hidden.fill = Some("ptnd.red/500".to_string());
        hidden.bounds = Some([10.0, 10.0, 20.0, 20.0]);
        hidden.visible = false;
        // Fully transparent box elsewhere must not paint.
        let mut ghost = DocumentObject::new(gen.next_object(), "Ghost");
        ghost.fill = Some("ptnd.red/500".to_string());
        ghost.bounds = Some([60.0, 60.0, 20.0, 20.0]);
        ghost.opacity = 0.0;
        let surface = Surface::with_objects(surface_id, "Clip", vec![box_obj, hidden, ghost]);

        let buf =
            SoftwarePixelCompositor::render_surface_rgba8(&surface, 100, 100, [255, 255, 255, 255]);
        let expected_blue = token_to_rgba8("ptnd.blue/500", 1.0);
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
        mask.fill = Some("ptnd.red/500".to_string());
        mask.bounds = Some([10.0, 10.0, 20.0, 20.0]);
        mask.is_clip_mask = true;
        let mut content = DocumentObject::new(gen.next_object(), "Content");
        content.fill = Some("ptnd.blue/500".to_string());
        content.bounds = Some([0.0, 0.0, 100.0, 100.0]);
        content.clip_mask_id = Some(mask_id);
        let surface = Surface::with_objects(surface_id, "Mask", vec![mask, content]);

        let buf =
            SoftwarePixelCompositor::render_surface_rgba8(&surface, 100, 100, [255, 255, 255, 255]);
        let expected_blue = token_to_rgba8("ptnd.blue/500", 1.0);
        // Inside mask bounds: content paints.
        assert_eq!(buf.get_pixel(15, 15).unwrap(), expected_blue);
        // Outside mask bounds: background (content clipped, mask not painted).
        assert_eq!(buf.get_pixel(5, 5).unwrap(), [255, 255, 255, 255]);
    }

    #[test]
    fn compositor_dirty_rect_repaints_only_dirty_region() {
        let mut gen = IdGenerator::new();
        let surface_id = gen.next_surface();
        let mut obj1 = DocumentObject::new(gen.next_object(), "Box1");
        obj1.fill = Some("ptnd.red/500".to_string());
        obj1.bounds = Some([10.0, 10.0, 20.0, 20.0]);

        let mut obj2 = DocumentObject::new(gen.next_object(), "Box2");
        obj2.fill = Some("ptnd.blue/500".to_string());
        obj2.bounds = Some([60.0, 60.0, 20.0, 20.0]);

        let surface = Surface::with_objects(surface_id, "DirtyTest", vec![obj1, obj2]);

        // Start with a green buffer
        let green = [0, 255, 0, 255];
        let mut buf = PixelBufferRgba8::with_fill(100, 100, green);

        // Repaint only dirty rect covering Box1 (0..40, 0..40), clearing to white
        let dirty_rect = GRect::new(0.0, 0.0, 40.0, 40.0);
        let white = [255, 255, 255, 255];
        SoftwarePixelCompositor::render_surface_dirty_rgba8(
            &surface,
            &mut buf,
            dirty_rect,
            Some(white),
        );

        let red = token_to_rgba8("ptnd.red/500", 1.0);
        // Inside dirty rect at Box1: red
        assert_eq!(buf.get_pixel(15, 15).unwrap(), red);
        // Inside dirty rect outside Box1: white (cleared background)
        assert_eq!(buf.get_pixel(5, 5).unwrap(), white);
        // Outside dirty rect at Box2: STILL GREEN! (Box2 was outside dirty rect, so not painted, and background was preserved)
        assert_eq!(buf.get_pixel(65, 65).unwrap(), green);
        assert_eq!(buf.get_pixel(80, 80).unwrap(), green);
    }

    #[test]
    fn compositor_viewport_culling_skips_offscreen() {
        let mut gen = IdGenerator::new();
        let surface_id = gen.next_surface();

        // 100 offscreen objects far away (1000..5000)
        let mut objects = Vec::new();
        for i in 0..100 {
            let mut off = DocumentObject::new(gen.next_object(), format!("Offscreen_{i}"));
            off.fill = Some("ptnd.red/500".to_string());
            off.bounds = Some([1000.0 + (i as f64 * 50.0), 1000.0, 40.0, 40.0]);
            objects.push(off);
        }

        // 1 visible object in viewport [0, 0, 100, 100]
        let mut on = DocumentObject::new(gen.next_object(), "Visible");
        on.fill = Some("ptnd.blue/500".to_string());
        on.bounds = Some([10.0, 10.0, 30.0, 30.0]);
        objects.push(on);

        let surface = Surface::with_objects(surface_id, "CullTest", objects);

        let buf =
            SoftwarePixelCompositor::render_surface_rgba8(&surface, 100, 100, [255, 255, 255, 255]);
        let blue = token_to_rgba8("ptnd.blue/500", 1.0);
        assert_eq!(buf.get_pixel(20, 20).unwrap(), blue);
        assert_eq!(buf.get_pixel(0, 0).unwrap(), [255, 255, 255, 255]);
    }

    #[test]
    fn compositor_dirty_rect_for_changeset() {
        let mut gen = IdGenerator::new();
        let surface_id = gen.next_surface();
        let obj_id = gen.next_object();
        let mut obj = DocumentObject::new(obj_id, "Box");
        obj.bounds = Some([20.0, 20.0, 30.0, 40.0]);
        let surface = Surface::with_objects(surface_id, "ChangeSetTest", vec![obj]);

        // Changeset with bounds change
        let mut cs = ChangeSet::empty();
        cs.push(Change::BoundsChanged {
            id: obj_id,
            previous_bounds: Some([10.0, 10.0, 20.0, 20.0]),
            next_bounds: Some([30.0, 30.0, 50.0, 50.0]),
            previous_rotation: 0.0,
            next_rotation: 0.0,
        });

        let dirty = SoftwarePixelCompositor::dirty_rect_for_changeset(&surface, &cs).unwrap();
        // Envelope must cover previous [10..30, 10..30] and next [30..80, 30..80] -> [10..80, 10..80]
        assert_eq!(dirty.x0, 10.0);
        assert_eq!(dirty.y0, 10.0);
        assert_eq!(dirty.x1, 80.0);
        assert_eq!(dirty.y1, 80.0);

        // Empty changeset yields None
        let empty_cs = ChangeSet::empty();
        assert_eq!(
            SoftwarePixelCompositor::dirty_rect_for_changeset(&surface, &empty_cs),
            None
        );
    }

    #[test]
    fn compositor_many_objects_and_masks_preserves_pixels_and_culling() {
        let mut gen = IdGenerator::new();
        let surface_id = gen.next_surface();

        let mask_id = gen.next_object();
        let mut mask = DocumentObject::new(mask_id, "Mask");
        mask.bounds = Some([10.0, 10.0, 50.0, 50.0]);
        mask.is_clip_mask = true;

        let mut objects = vec![mask];

        // 1000 objects, 500 of which reference mask_id and 500 offscreen
        for i in 0..1000 {
            let mut obj = DocumentObject::new(gen.next_object(), format!("Obj_{i}"));
            obj.fill = Some("ptnd.blue/500".to_string());
            if i % 2 == 0 {
                obj.bounds = Some([10.0, 10.0, 50.0, 50.0]);
                obj.clip_mask_id = Some(mask_id);
            } else {
                // Offscreen
                obj.bounds = Some([2000.0 + (i as f64 * 10.0), 2000.0, 20.0, 20.0]);
            }
            objects.push(obj);
        }

        let surface = Surface::with_objects(surface_id, "PerfTest", objects);

        let buf =
            SoftwarePixelCompositor::render_surface_rgba8(&surface, 200, 200, [255, 255, 255, 255]);
        let blue = token_to_rgba8("ptnd.blue/500", 1.0);
        assert_eq!(buf.get_pixel(25, 25).unwrap(), blue);
        assert_eq!(buf.get_pixel(100, 100).unwrap(), [255, 255, 255, 255]);
    }
}
