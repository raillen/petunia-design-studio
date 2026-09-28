//! Single-pass vector painting for the canvas.
//!
//! The previous adapter drew one `rect()` per object from its bounds, so a star,
//! a polygon and a Bézier path all rendered as plain rectangles. This module
//! paints the evaluated world outline instead, inside one Freya `Canvas` paint
//! callback, so the cost is one pass over the scene rather than a widget tree
//! per object.
//!
//! Presentation only: it reads the toolkit-neutral `CanvasSnapshot` and never
//! mutates the document or reaches into domain state.

use freya::prelude::*;
use freya_engine::prelude::{
    AlphaType, Canvas as SkiaCanvas, Color, ColorType, Data, Font, Image, ImageInfo, Paint,
    PaintStyle, Path, PathBuilder, Point, Rect as SkRect,
};
use petunia_design_geometry::{GPath, GPoint, PathVerb};

#[allow(deprecated)]
fn make_skia_image_from_rgba8(width: i32, height: i32, data: &[u8]) -> Option<Image> {
    let info = ImageInfo::new(
        (width, height),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    let bytes = Data::new_copy(data);
    Image::from_raster_data(&info, bytes, (width * 4) as usize)
}

/// Paints a 128x128 sparse CPU raster tile onto the Skia canvas.
#[allow(dead_code, deprecated)]
pub fn paint_raster_tile(
    canvas: &SkiaCanvas,
    tile: &petunia_design_raster::Tile,
    camera: &ViewportCamera,
    adjustments: &[petunia_design_document::adjustments::AdjustmentItem],
) {
    let mut rgba8 = match tile.format {
        petunia_design_raster::PixelFormat::Rgba8 => tile.data.clone(),
        _ => {
            let mut out = Vec::with_capacity(
                petunia_design_raster::TILE_SIZE * petunia_design_raster::TILE_SIZE * 4,
            );
            for y in 0..petunia_design_raster::TILE_SIZE {
                for x in 0..petunia_design_raster::TILE_SIZE {
                    let [r, g, b, a] = tile.get_pixel_normalized(x, y);
                    out.push((r * 255.0).round() as u8);
                    out.push((g * 255.0).round() as u8);
                    out.push((b * 255.0).round() as u8);
                    out.push((a * 255.0).round() as u8);
                }
            }
            out
        }
    };
    if !adjustments.is_empty() {
        for chunk in rgba8.chunks_exact_mut(4) {
            let rgb = [
                chunk[0] as f32 / 255.0,
                chunk[1] as f32 / 255.0,
                chunk[2] as f32 / 255.0,
            ];
            let adjusted = petunia_design_document::adjustments::apply_adjustment_chain(rgb, adjustments);
            chunk[0] = (adjusted[0].clamp(0.0, 1.0) * 255.0).round() as u8;
            chunk[1] = (adjusted[1].clamp(0.0, 1.0) * 255.0).round() as u8;
            chunk[2] = (adjusted[2].clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }
    let Some(image) = make_skia_image_from_rgba8(
        petunia_design_raster::TILE_SIZE as i32,
        petunia_design_raster::TILE_SIZE as i32,
        &rgba8,
    ) else {
        return;
    };
    let bounds = tile.coord.bounds();
    let tl = camera.doc_to_screen(GPoint::new(bounds.x0, bounds.y0));
    let br = camera.doc_to_screen(GPoint::new(bounds.x1, bounds.y1));
    let dst = SkRect::new(tl.x as f32, tl.y as f32, br.x as f32, br.y as f32);
    canvas.draw_image_rect(&image, None, dst, &Paint::default());
}

/// Paints a single raster brush dab stamp onto the Skia canvas.
fn paint_brush_dab(
    canvas: &SkiaCanvas,
    dab: &petunia_design_raster::BrushDab,
    camera: &ViewportCamera,
) {
    let center = camera.doc_to_screen(GPoint::new(dab.center_x, dab.center_y));
    let r = (dab.radius * camera.zoom) as f32;
    if r <= 0.1 {
        return;
    }
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(PaintStyle::Fill);
    let alpha = (dab.color[3].clamp(0.0, 1.0) * dab.opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
    let red = (dab.color[0].clamp(0.0, 1.0) * 255.0).round() as u8;
    let green = (dab.color[1].clamp(0.0, 1.0) * 255.0).round() as u8;
    let blue = (dab.color[2].clamp(0.0, 1.0) * 255.0).round() as u8;
    paint.set_color(Color::from_argb(alpha, red, green, blue));

    let mut circ = PathBuilder::default();
    circ.add_circle((center.x as f32, center.y as f32), r, None);
    canvas.draw_path(&circ.detach(), &paint);
}
use petunia_design_shell::canvas::{
    CanvasObjectProjection, CanvasOverlays, CanvasSnapshot, GradientOverlay, SelectionHandle,
    SelectionHandleKind, SnapOrientation, SurfaceView, ViewportCamera,
};

/// Screen-pixel size of a handle glyph. The hit box is larger and lives in
/// `CanvasOverlays`; `08 23` requires the two to scale independently.
const HANDLE_GLYPH_PX: f32 = 7.0;

/// Accent used for handles and selection chrome.
const ACCENT: Color = Color::from_rgb(0xB7, 0x7A, 0xFF);
/// Marquee and committed-selection colour.
const MARQUEE: Color = Color::from_rgb(0x35, 0xC7, 0xD4);
/// In-flight preview colour.
const PREVIEW: Color = Color::from_rgb(0xB7, 0x7A, 0xFF);
/// Snap guide colour.
const GUIDE: Color = Color::from_rgb(0xF0, 0x6C, 0x8D);
/// Subtractive / Alt-carve colour.
const SUBTRACTIVE: Color = Color::from_rgb(0xF0, 0x6C, 0x8D);

/// Builds the canvas element that paints a whole scene in one pass.
pub fn canvas_view(
    snapshot: CanvasSnapshot,
    in_flight_guide: Option<(petunia_design_document::GuideOrientation, f64)>,
) -> Canvas {
    let on_render = RenderCallback::new(move |context: &mut CanvasContext| {
        paint_scene(&snapshot, context, in_flight_guide);
    });
    canvas(on_render)
        .width(Size::fill())
        .height(Size::fill())
}

/// Paints background, artwork, overlays and handles in a single ordered pass.
fn paint_scene(
    snapshot: &CanvasSnapshot,
    context: &mut CanvasContext,
    in_flight_guide: Option<(petunia_design_document::GuideOrientation, f64)>,
) {
    let canvas = context.canvas;
    paint_surface(canvas, snapshot.surface.as_ref(), &snapshot.camera);
    for object in &snapshot.objects {
        paint_object(canvas, object, &snapshot.camera);
    }
    paint_overlays(canvas, &snapshot.overlays, snapshot.surface.as_ref(), &snapshot.camera);
    if let Some((orient, pos)) = in_flight_guide {
        let guide_color = Color::from_rgb(0x00, 0xE5, 0xFF);
        let guide_paint = outline_paint(guide_color, 1.5);
        let mut text_paint = Paint::default();
        text_paint.set_color(Color::WHITE);
        text_paint.set_anti_alias(true);
        let mut font = Font::default();
        font.set_size(10.0);
        match orient {
            petunia_design_document::GuideOrientation::Horizontal => {
                let sy = snapshot.camera.doc_to_screen(GPoint::new(0.0, pos)).y as f32;
                canvas.draw_line(
                    Point::new(0.0, sy),
                    Point::new(snapshot.camera.viewport_width as f32, sy),
                    &guide_paint,
                );
                let label = format!("Y: {:.1} pt", pos);
                canvas.draw_str(&label, Point::new(30.0, sy - 4.0), &font, &text_paint);
            }
            petunia_design_document::GuideOrientation::Vertical => {
                let sx = snapshot.camera.doc_to_screen(GPoint::new(pos, 0.0)).x as f32;
                canvas.draw_line(
                    Point::new(sx, 0.0),
                    Point::new(sx, snapshot.camera.viewport_height as f32),
                    &guide_paint,
                );
                let label = format!("X: {:.1} pt", pos);
                canvas.draw_str(&label, Point::new(sx + 4.0, 35.0), &font, &text_paint);
            }
        }
    }
    paint_rulers(canvas, &snapshot.camera);
}

/// Fills the pasteboard behind the artwork.
fn paint_surface(canvas: &SkiaCanvas, surface: Option<&SurfaceView>, camera: &ViewportCamera) {
    let Some(surface) = surface else {
        return;
    };
    let [x, y, width, height] = surface.bounds;
    let top_left = camera.doc_to_screen(GPoint::new(x, y));
    let bottom_right = camera.doc_to_screen(GPoint::new(x + width, y + height));
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(PaintStyle::Fill);
    paint.set_color(Color::from_rgb(0xE2, 0xE4, 0xE8));
    canvas.draw_rect(
        SkRect::new(
            top_left.x as f32,
            top_left.y as f32,
            bottom_right.x as f32,
            bottom_right.y as f32,
        ),
        &paint,
    );
}

/// Paints one object from its evaluated world outline.
///
/// Objects with no evaluable geometry are skipped rather than drawn as a proxy
/// rectangle: a bounding box is not the artwork, and painting one reintroduces
/// the very defect this module replaces.
fn paint_object(
    canvas: &SkiaCanvas,
    object: &CanvasObjectProjection,
    camera: &ViewportCamera,
) {
    for tile in &object.raster_tiles {
        paint_raster_tile(canvas, tile, camera, &object.adjustments);
    }
    let opacity = object.opacity.clamp(0.0, 1.0) as f32;
    if let Some(path) = object.outline.as_ref() {
        let sk_path = build_skia_path(path, camera);
        if !sk_path.is_empty() {
            // Render visible drop shadows behind the object
            for effect in &object.effects {
                if !effect.visible {
                    continue;
                }
                if let petunia_design_document::EffectKind::DropShadow {
                    offset,
                    blur: _,
                    color,
                    opacity: shadow_opacity,
                } = &effect.kind {
                    let dx = (offset[0] * camera.zoom) as f32;
                    let dy = (offset[1] * camera.zoom) as f32;
                    let mut shadow_paint = Paint::default();
                    shadow_paint.set_anti_alias(true);
                    shadow_paint.set_style(PaintStyle::Fill);
                    let final_opacity = (opacity * shadow_opacity.clamp(0.0, 1.0) as f32).clamp(0.0, 1.0);
                    shadow_paint.set_color(resolve_color_with_adjustments(
                        Some(color),
                        final_opacity,
                        &object.adjustments,
                    ));
                    canvas.save();
                    canvas.translate((dx, dy));
                    canvas.draw_path(&sk_path, &shadow_paint);
                    canvas.restore();
                }
            }
            if let Some(fill_token) = object.fill.as_deref() {
                let mut paint = Paint::default();
                paint.set_anti_alias(true);
                paint.set_style(PaintStyle::Fill);
                paint.set_color(resolve_color_with_adjustments(
                    Some(fill_token),
                    opacity,
                    &object.adjustments,
                ));
                canvas.draw_path(&sk_path, &paint);
            }
            if let Some(stroke_token) = object.stroke.as_deref() {
                if object.stroke_width > 0.0 {
                    let mut paint = Paint::default();
                    paint.set_anti_alias(true);
                    paint.set_style(PaintStyle::Stroke);
                    let screen_width = (object.stroke_width * camera.zoom).max(1.0) as f32;
                    paint.set_stroke_width(screen_width);
                    paint.set_color(resolve_color_with_adjustments(
                        Some(stroke_token),
                        opacity,
                        &object.adjustments,
                    ));
                    canvas.draw_path(&sk_path, &paint);
                }
            }
            if object.active {
                let selection_paint = outline_paint(ACCENT, 1.5);
                canvas.draw_path(&sk_path, &selection_paint);
            }
        }
    } else if let Some(petunia_design_document::ShapeKind::Text {
        content,
        font_size,
        ..
    }) = object.shape.as_ref()
    {
        paint_text_object(canvas, object, content, *font_size, camera, opacity);
    } else if let Some(petunia_design_document::ShapeKind::Image {
        path,
        data,
    }) = object.shape.as_ref()
    {
        paint_image_object(canvas, object, path, data.as_deref(), camera, opacity);
    }
}

/// Paints a text object with Skia string rendering.
fn paint_text_object(
    canvas: &SkiaCanvas,
    object: &CanvasObjectProjection,
    content: &str,
    font_size: f64,
    camera: &ViewportCamera,
    opacity: f32,
) {
    if content.is_empty() {
        return;
    }
    let screen_origin = camera.doc_to_screen(GPoint::new(
        object.frame_origin[0],
        object.frame_origin[1],
    ));
    canvas.save();
    canvas.translate((screen_origin.x as f32, screen_origin.y as f32));
    if object.rotation.abs() > f64::EPSILON {
        canvas.rotate((object.rotation * 180.0 / std::f64::consts::PI) as f32, None);
    }
    let screen_font_size = (font_size * camera.zoom).max(6.0) as f32;
    let mut font = Font::default();
    font.set_size(screen_font_size);

    let mut text_paint = Paint::default();
    text_paint.set_anti_alias(true);
    text_paint.set_style(PaintStyle::Fill);
    let color_token = object.fill.as_deref().or(Some("ptnd.gray/900"));
    text_paint.set_color(resolve_color(color_token, opacity));

    canvas.draw_str(
        content,
        Point::new(0.0, screen_font_size * 0.8),
        &font,
        &text_paint,
    );

    if object.active {
        let w = (object.size[0] * camera.zoom) as f32;
        let h = (object.size[1] * camera.zoom) as f32;
        canvas.draw_rect(SkRect::new(0.0, 0.0, w, h), &outline_paint(ACCENT, 1.5));
    }
    canvas.restore();
}

/// Builds a Skia path from world geometry projected to screen pixels.
fn build_skia_path(path: &GPath, camera: &ViewportCamera) -> Path {
    let mut builder = PathBuilder::default();
    let to_point = |p: GPoint| {
        let screen = camera.doc_to_screen(p);
        Point::new(screen.x as f32, screen.y as f32)
    };
    for verb in &path.verbs {
        match *verb {
            PathVerb::MoveTo(p) => {
                builder.move_to(to_point(p));
            }
            PathVerb::LineTo(p) => {
                builder.line_to(to_point(p));
            }
            PathVerb::QuadTo(control, p) => {
                builder
                    .quad_to(to_point(control), to_point(p));
            }
            PathVerb::CubicTo(control1, control2, p) => {
                builder
                    .cubic_to(to_point(control1), to_point(control2), to_point(p));
            }
            PathVerb::Close => {
                builder.close();
            }
        }
    }
    builder.detach()
}

/// Resolves a design-token into a Skia color with opacity and optional tonal adjustments (Spec 10.10).
fn resolve_color(token: Option<&str>, opacity: f32) -> Color {
    resolve_color_with_adjustments(token, opacity, &[])
}

/// Resolves a design-token and applies non-destructive tonal adjustments.
fn resolve_color_with_adjustments(
    token: Option<&str>,
    opacity: f32,
    adjustments: &[petunia_design_document::adjustments::AdjustmentItem],
) -> Color {
    let mut rgb = token
        .map_or([0.18, 0.5, 0.97], petunia_design_document::resolve_color_to_rgb);
    if !adjustments.is_empty() {
        rgb = petunia_design_document::adjustments::apply_adjustment_chain(rgb, adjustments);
    }
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color::from_argb(alpha, channel(rgb[0]), channel(rgb[1]), channel(rgb[2]))
}

/// Paints overlays in one ordered pass, above the artwork.
///
/// `snap_guides` are drawn first so the marquee and the handles stay readable
/// on top of them.
fn paint_overlays(
    canvas: &SkiaCanvas,
    overlays: &CanvasOverlays,
    surface: Option<&SurfaceView>,
    camera: &ViewportCamera,
) {
    // 1. Surface layout guides (cyan rules)
    if let Some(surface) = surface {
        let guide_color = Color::from_rgb(0x00, 0xBC, 0xD4);
        let guide_paint = outline_paint(guide_color, 1.0);
        let [sx, sy, sw, sh] = surface.bounds;
        for guide in &surface.guides {
            if !guide.visible {
                continue;
            }
            match guide.orientation {
                petunia_design_document::GuideOrientation::Horizontal => {
                    let y = camera.doc_to_screen(GPoint::new(0.0, guide.position)).y as f32;
                    let left = camera.doc_to_screen(GPoint::new(sx - 2000.0, 0.0)).x as f32;
                    let right = camera.doc_to_screen(GPoint::new(sx + sw + 2000.0, 0.0)).x as f32;
                    canvas.draw_line(Point::new(left, y), Point::new(right, y), &guide_paint);
                }
                petunia_design_document::GuideOrientation::Vertical => {
                    let x = camera.doc_to_screen(GPoint::new(guide.position, 0.0)).x as f32;
                    let top = camera.doc_to_screen(GPoint::new(0.0, sy - 2000.0)).y as f32;
                    let bottom = camera.doc_to_screen(GPoint::new(0.0, sy + sh + 2000.0)).y as f32;
                    canvas.draw_line(Point::new(x, top), Point::new(x, bottom), &guide_paint);
                }
            }
        }
    }

    // 2. Snapping magnetic guides
    for guide in &overlays.snap_guides {
        let (start, end) = match guide.orientation {
            SnapOrientation::Vertical => (
                camera.doc_to_screen(GPoint::new(guide.position, guide.span_start)),
                camera.doc_to_screen(GPoint::new(guide.position, guide.span_end)),
            ),
            SnapOrientation::Horizontal => (
                camera.doc_to_screen(GPoint::new(guide.span_start, guide.position)),
                camera.doc_to_screen(GPoint::new(guide.span_end, guide.position)),
            ),
        };
        stroke_line(canvas, start, end, GUIDE, 1.0);
        if let Some(label) = &guide.label {
            paint_guide_badge(canvas, label, start, end);
        }
    }
    if let Some(preview) = &overlays.transform_preview {
        paint_transform_preview(canvas, preview, camera);
    }
    if let Some(marquee) = overlays.marquee_screen {
        let rect = SkRect::new(
            marquee.x0.min(marquee.x1) as f32,
            marquee.y0.min(marquee.y1) as f32,
            marquee.x0.max(marquee.x1) as f32,
            marquee.y0.max(marquee.y1) as f32,
        );
        paint_marching_ants_rect(canvas, rect);
    }
    if let Some(points) = overlays.pen_preview.as_ref() {
        let color = if overlays.region_subtractive {
            SUBTRACTIVE
        } else {
            PREVIEW
        };
        paint_polyline(canvas, points, camera, color, 1.5);
    }
    if let Some(points) = overlays.lasso_screen.as_ref() {
        paint_polyline(canvas, points, camera, MARQUEE, 1.0);
    }
    if let Some(contours) = overlays.selection_mask.as_ref() {
        for contour in contours {
            paint_marching_ants_polyline(canvas, contour, camera);
        }
    }
    if let Some(region) = overlays.region_preview.as_ref() {
        let (fill_color, outline_color) = if overlays.region_subtractive {
            (
                Color::from_argb(0x4D, 0xF0, 0x6C, 0x8D),
                SUBTRACTIVE,
            )
        } else {
            (
                Color::from_argb(0x4D, 0xB7, 0x7A, 0xFF),
                ACCENT,
            )
        };
        if region.len() >= 3 {
            let mut builder = PathBuilder::default();
            let start = camera.doc_to_screen(region[0]);
            builder.move_to((start.x as f32, start.y as f32));
            for p in &region[1..] {
                let screen = camera.doc_to_screen(*p);
                builder.line_to((screen.x as f32, screen.y as f32));
            }
            builder.close();
            let path = builder.detach();

            let mut fill = Paint::default();
            fill.set_anti_alias(true);
            fill.set_style(PaintStyle::Fill);
            fill.set_color(fill_color);
            canvas.draw_path(&path, &fill);
            canvas.draw_path(&path, &outline_paint(outline_color, 1.5));
        } else {
            paint_polyline(canvas, region, camera, outline_color, 1.0);
        }
    }
    if let Some(gradient) = &overlays.gradient {
        paint_gradient_overlay(canvas, gradient, camera);
    }
    if let Some(preview_path) = &overlays.path_preview {
        let skia_path = build_skia_path(preview_path, camera);
        canvas.draw_path(&skia_path, &outline_paint(ACCENT, 1.5));
    }
    for &(anchor, control) in &overlays.node_control_lines {
        let s_anchor = camera.doc_to_screen(anchor);
        let s_control = camera.doc_to_screen(control);
        stroke_line(canvas, s_anchor, s_control, Color::from_argb(0xCC, 0xB7, 0x7A, 0xFF), 1.0);
    }
    if let Some(dabs) = &overlays.brush_preview {
        for dab in dabs {
            paint_brush_dab(canvas, dab, camera);
        }
    }
    if let Some((pos_doc, label)) = &overlays.measure_badge {
        paint_measure_overlay(canvas, pos_doc, label, overlays.pen_preview.as_deref(), camera);
    }
    paint_selection_bounding_box(canvas, &overlays.handles, camera);
    for handle in &overlays.handles {
        paint_handle(canvas, handle, &overlays.handles, camera);
    }
}

/// Paints the bounding box outline connecting the four corner handles.
fn paint_selection_bounding_box(
    canvas: &SkiaCanvas,
    handles: &[SelectionHandle],
    camera: &ViewportCamera,
) {
    let tl = handles
        .iter()
        .find(|h| h.kind == SelectionHandleKind::TopLeft);
    let tr = handles
        .iter()
        .find(|h| h.kind == SelectionHandleKind::TopRight);
    let br = handles
        .iter()
        .find(|h| h.kind == SelectionHandleKind::BottomRight);
    let bl = handles
        .iter()
        .find(|h| h.kind == SelectionHandleKind::BottomLeft);

    if let (Some(tl), Some(tr), Some(br), Some(bl)) = (tl, tr, br, bl) {
        let p_tl = camera.doc_to_screen(tl.doc_point);
        let p_tr = camera.doc_to_screen(tr.doc_point);
        let p_br = camera.doc_to_screen(br.doc_point);
        let p_bl = camera.doc_to_screen(bl.doc_point);

        let mut builder = PathBuilder::default();
        builder.move_to((p_tl.x as f32, p_tl.y as f32));
        builder.line_to((p_tr.x as f32, p_tr.y as f32));
        builder.line_to((p_br.x as f32, p_br.y as f32));
        builder.line_to((p_bl.x as f32, p_bl.y as f32));
        builder.close();
        canvas.draw_path(&builder.detach(), &outline_paint(ACCENT, 1.0));

        // When only 4 corner handles are present (e.g. Perspective warp quad wireframe),
        // render the internal 3x3 perspective guide mesh lines.
        if handles.len() == 4 {
            let grid_paint = outline_paint(Color::from_argb(0x66, 0xB7, 0x7A, 0xFF), 1.0);
            for i in 1..3 {
                let t = i as f64 / 3.0;
                let left_x = p_tl.x + (p_bl.x - p_tl.x) * t;
                let left_y = p_tl.y + (p_bl.y - p_tl.y) * t;
                let right_x = p_tr.x + (p_br.x - p_tr.x) * t;
                let right_y = p_tr.y + (p_br.y - p_tr.y) * t;
                canvas.draw_line(
                    Point::new(left_x as f32, left_y as f32),
                    Point::new(right_x as f32, right_y as f32),
                    &grid_paint,
                );

                let top_x = p_tl.x + (p_tr.x - p_tl.x) * t;
                let top_y = p_tl.y + (p_tr.y - p_tl.y) * t;
                let bot_x = p_bl.x + (p_br.x - p_bl.x) * t;
                let bot_y = p_bl.y + (p_br.y - p_bl.y) * t;
                canvas.draw_line(
                    Point::new(top_x as f32, top_y as f32),
                    Point::new(bot_x as f32, bot_y as f32),
                    &grid_paint,
                );
            }
        }
    }
}

/// Paints the transient transform preview as translucent proposed frames.
fn paint_transform_preview(
    canvas: &SkiaCanvas,
    preview: &petunia_design_shell::canvas::TransformPreview,
    camera: &ViewportCamera,
) {
    for object in &preview.objects {
        let [x, y, width, height] = object.bounds;
        let mut fill = Paint::default();
        fill.set_anti_alias(true);
        fill.set_style(PaintStyle::Fill);
        fill.set_color(PREVIEW);
        fill.set_alpha_f(0.10);

        if object.rotation.abs() > f64::EPSILON {
            let rot_trans = petunia_design_geometry::GAffine::translate(x, y)
                .after(petunia_design_geometry::GAffine::rotate(object.rotation));
            let c0 = camera.doc_to_screen(rot_trans.apply(GPoint::new(0.0, 0.0)));
            let c1 = camera.doc_to_screen(rot_trans.apply(GPoint::new(width, 0.0)));
            let c2 = camera.doc_to_screen(rot_trans.apply(GPoint::new(width, height)));
            let c3 = camera.doc_to_screen(rot_trans.apply(GPoint::new(0.0, height)));

            let mut builder = PathBuilder::default();
            builder.move_to((c0.x as f32, c0.y as f32));
            builder.line_to((c1.x as f32, c1.y as f32));
            builder.line_to((c2.x as f32, c2.y as f32));
            builder.line_to((c3.x as f32, c3.y as f32));
            builder.close();
            let path = builder.detach();
            canvas.draw_path(&path, &fill);
            canvas.draw_path(&path, &outline_paint(ACCENT, 1.0));
        } else {
            let top_left = camera.doc_to_screen(GPoint::new(x, y));
            let bottom_right = camera.doc_to_screen(GPoint::new(x + width, y + height));
            let rect = SkRect::new(
                top_left.x as f32,
                top_left.y as f32,
                bottom_right.x as f32,
                bottom_right.y as f32,
            );
            canvas.draw_rect(rect, &fill);
            canvas.draw_rect(rect, &outline_paint(ACCENT, 1.0));
        }
    }
}

/// Paints a polyline through document-space points.
fn paint_polyline(
    canvas: &SkiaCanvas,
    points: &[GPoint],
    camera: &ViewportCamera,
    color: Color,
    width: f32,
) {
    let Some(first) = points.first() else {
        return;
    };
    let mut builder = PathBuilder::default();
    let start = camera.doc_to_screen(*first);
    builder.move_to((start.x as f32, start.y as f32));
    for point in &points[1..] {
        let screen = camera.doc_to_screen(*point);
        builder.line_to((screen.x as f32, screen.y as f32));
    }
    let path = builder.detach();
    canvas.draw_path(&path, &outline_paint(color, width));
}

/// Strokes a straight line between two document-space points.
fn stroke_line(
    canvas: &SkiaCanvas,
    start: GPoint,
    end: GPoint,
    color: Color,
    width: f32,
) {
    let mut builder = PathBuilder::default();
    builder.move_to((start.x as f32, start.y as f32));
    builder.line_to((end.x as f32, end.y as f32));
    let path = builder.detach();
    canvas.draw_path(&path, &outline_paint(color, width));
}

/// Paints a dashed line with alternating colors (marching ants effect).
fn paint_dashed_line(
    canvas: &SkiaCanvas,
    p0: Point,
    p1: Point,
    color1: Color,
    color2: Color,
    dash: f32,
) {
    let dx = p1.x - p0.x;
    let dy = p1.y - p0.y;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist <= 0.001 {
        return;
    }
    let ux = dx / dist;
    let uy = dy / dist;
    let mut paint1 = Paint::default();
    paint1.set_color(color1);
    paint1.set_style(PaintStyle::Stroke);
    paint1.set_stroke_width(1.0);
    paint1.set_anti_alias(true);
    let mut paint2 = Paint::default();
    paint2.set_color(color2);
    paint2.set_style(PaintStyle::Stroke);
    paint2.set_stroke_width(1.0);
    paint2.set_anti_alias(true);
    let mut d = 0.0;
    let mut toggle = false;
    while d < dist {
        let d_next = (d + dash).min(dist);
        let start = Point::new(p0.x + ux * d, p0.y + uy * d);
        let end = Point::new(p0.x + ux * d_next, p0.y + uy * d_next);
        let paint = if toggle { &paint1 } else { &paint2 };
        canvas.draw_line(start, end, paint);
        d = d_next;
        toggle = !toggle;
    }
}

/// Paints a marquee rectangle with classic black/white marching ants.
fn paint_marching_ants_rect(canvas: &SkiaCanvas, rect: SkRect) {
    let tl = Point::new(rect.left, rect.top);
    let tr = Point::new(rect.right, rect.top);
    let br = Point::new(rect.right, rect.bottom);
    let bl = Point::new(rect.left, rect.bottom);
    let black = Color::BLACK;
    let white = Color::WHITE;
    paint_dashed_line(canvas, tl, tr, black, white, 4.0);
    paint_dashed_line(canvas, tr, br, black, white, 4.0);
    paint_dashed_line(canvas, br, bl, black, white, 4.0);
    paint_dashed_line(canvas, bl, tl, black, white, 4.0);
}

/// Paints a polyline with marching ants for photo selection masks.
fn paint_marching_ants_polyline(canvas: &SkiaCanvas, points: &[GPoint], camera: &ViewportCamera) {
    if points.len() < 2 {
        return;
    }
    let black = Color::BLACK;
    let white = Color::WHITE;
    for window in points.windows(2) {
        let p0 = camera.doc_to_screen(window[0]);
        let p1 = camera.doc_to_screen(window[1]);
        paint_dashed_line(
            canvas,
            Point::new(p0.x as f32, p0.y as f32),
            Point::new(p1.x as f32, p1.y as f32),
            black,
            white,
            4.0,
        );
    }
}

/// Paints a measurement badge pill at the midpoint of a snap guide.
fn paint_guide_badge(
    canvas: &SkiaCanvas,
    label: &str,
    start: GPoint,
    end: GPoint,
) {
    let mid_x = ((start.x + end.x) / 2.0) as f32;
    let mid_y = ((start.y + end.y) / 2.0) as f32;

    let font_size = 10.0;
    let mut font = Font::default();
    font.set_size(font_size);

    let text_width = (label.len() as f32) * 6.5;
    let pad_x = 5.0;
    let pad_y = 2.5;
    let half_w = text_width / 2.0 + pad_x;
    let half_h = font_size / 2.0 + pad_y;

    let badge_rect = SkRect::new(
        mid_x - half_w,
        mid_y - half_h,
        mid_x + half_w,
        mid_y + half_h,
    );

    let mut bg_paint = Paint::default();
    bg_paint.set_anti_alias(true);
    bg_paint.set_style(PaintStyle::Fill);
    bg_paint.set_color(Color::from_rgb(0x1F, 0x1F, 0x24));

    let border_paint = outline_paint(GUIDE, 1.0);

    let mut text_paint = Paint::default();
    text_paint.set_anti_alias(true);
    text_paint.set_style(PaintStyle::Fill);
    text_paint.set_color(Color::from_rgb(0xFF, 0xFF, 0xFF));

    canvas.draw_rect(badge_rect, &bg_paint);
    canvas.draw_rect(badge_rect, &border_paint);
    canvas.draw_str(
        label,
        Point::new(mid_x - half_w + pad_x, mid_y + half_h - pad_y - 1.0),
        &font,
        &text_paint,
    );
}

/// Paints precision measurement overlays: dimension lines, end ticks, orthogonal guides and badge.
fn paint_measure_overlay(
    canvas: &SkiaCanvas,
    pos_doc: &GPoint,
    label: &str,
    pen_preview: Option<&[GPoint]>,
    camera: &ViewportCamera,
) {
    if let Some(pts) = pen_preview {
        if pts.len() >= 2 {
            let s0 = camera.doc_to_screen(pts[0]);
            let s1 = camera.doc_to_screen(pts[1]);
            let dx = s1.x - s0.x;
            let dy = s1.y - s0.y;
            let len = dx.hypot(dy);

            let tick_paint = outline_paint(Color::from_rgb(0x00, 0xE5, 0xFF), 1.5);

            if len > 2.0 {
                let nx = -dy / len;
                let ny = dx / len;
                let tick_len = 6.0;

                // End cap tick at s0
                canvas.draw_line(
                    Point::new((s0.x - nx * tick_len) as f32, (s0.y - ny * tick_len) as f32),
                    Point::new((s0.x + nx * tick_len) as f32, (s0.y + ny * tick_len) as f32),
                    &tick_paint,
                );
                // End cap tick at s1
                canvas.draw_line(
                    Point::new((s1.x - nx * tick_len) as f32, (s1.y - ny * tick_len) as f32),
                    Point::new((s1.x + nx * tick_len) as f32, (s1.y + ny * tick_len) as f32),
                    &tick_paint,
                );

                // Orthogonal projection lines (when not near-horizontal or near-vertical)
                if dx.abs() > 16.0 && dy.abs() > 16.0 {
                    let corner = GPoint::new(s1.x, s0.y);
                    let dash_paint = outline_paint(Color::from_argb(0x88, 0x00, 0xE5, 0xFF), 1.0);
                    canvas.draw_line(
                        Point::new(s0.x as f32, s0.y as f32),
                        Point::new(corner.x as f32, corner.y as f32),
                        &dash_paint,
                    );
                    canvas.draw_line(
                        Point::new(corner.x as f32, corner.y as f32),
                        Point::new(s1.x as f32, s1.y as f32),
                        &dash_paint,
                    );
                }
            }
        }
    }

    let screen_pt = camera.doc_to_screen(*pos_doc);
    paint_guide_badge(canvas, label, screen_pt, screen_pt);
}

/// Paints the interactive gradient line vector and stops on the canvas.
fn paint_gradient_overlay(
    canvas: &SkiaCanvas,
    gradient: &GradientOverlay,
    _camera: &ViewportCamera,
) {
    let p_start = gradient.start;
    let p_end = gradient.end;

    // 1. If Radial, draw extent guide circle
    if gradient.kind == petunia_design_shell::canvas::GradientOverlayKind::Radial {
        let dx = (p_end.x - p_start.x) as f32;
        let dy = (p_end.y - p_start.y) as f32;
        let radius = (dx * dx + dy * dy).sqrt();
        if radius > 1.0 {
            let mut radial_paint = outline_paint(Color::from_argb(0x88, 0xB7, 0x7A, 0xFF), 1.0);
            radial_paint.set_style(PaintStyle::Stroke);
            let mut circle_builder = PathBuilder::default();
            circle_builder.add_circle((p_start.x as f32, p_start.y as f32), radius, None);
            canvas.draw_path(&circle_builder.detach(), &radial_paint);
        }
    }

    // 2. Vector line with drop shadow for contrast
    stroke_line(canvas, p_start, p_end, Color::from_argb(0x80, 0x00, 0x00, 0x00), 2.5);
    stroke_line(canvas, p_start, p_end, ACCENT, 1.5);

    // 3. Intermediate stops with resolved stop colors
    for (i, &(_offset, stop_screen)) in gradient.stops.iter().enumerate() {
        let (sx, sy) = (stop_screen.x as f32, stop_screen.y as f32);
        let rgb = gradient.stop_colors.get(i).copied().unwrap_or([1.0, 1.0, 1.0]);
        let stop_color = Color::from_rgb(
            (rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8,
            (rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8,
            (rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        );

        // Shadow ring
        let mut shadow_circ = PathBuilder::default();
        shadow_circ.add_circle((sx, sy + 1.0), 6.0, None);
        let mut shadow_paint = Paint::default();
        shadow_paint.set_anti_alias(true);
        shadow_paint.set_color(Color::from_argb(0x80, 0, 0, 0));
        canvas.draw_path(&shadow_circ.detach(), &shadow_paint);

        // White outer ring
        let mut outer_circ = PathBuilder::default();
        outer_circ.add_circle((sx, sy), 5.5, None);
        let mut white_paint = Paint::default();
        white_paint.set_anti_alias(true);
        white_paint.set_color(Color::WHITE);
        canvas.draw_path(&outer_circ.detach(), &white_paint);

        // Colored stop center
        let mut inner_circ = PathBuilder::default();
        inner_circ.add_circle((sx, sy), 4.0, None);
        let mut inner_paint = Paint::default();
        inner_paint.set_anti_alias(true);
        inner_paint.set_color(stop_color);
        canvas.draw_path(&inner_circ.detach(), &inner_paint);

        // Subtle dark rim
        let stroke = outline_paint(Color::from_argb(0x80, 0, 0, 0), 1.0);
        let mut border_circ = PathBuilder::default();
        border_circ.add_circle((sx, sy), 5.5, None);
        canvas.draw_path(&border_circ.detach(), &stroke);
    }

    // 4. Start handle (origin ring)
    let (sx, sy) = (p_start.x as f32, p_start.y as f32);
    let mut start_circ = PathBuilder::default();
    start_circ.add_circle((sx, sy), 6.5, None);
    let path = start_circ.detach();
    let mut fill_paint = Paint::default();
    fill_paint.set_anti_alias(true);
    fill_paint.set_color(Color::WHITE);
    canvas.draw_path(&path, &fill_paint);
    canvas.draw_path(&path, &outline_paint(ACCENT, 2.0));

    // 5. End handle (termination square)
    let (ex, ey) = (p_end.x as f32, p_end.y as f32);
    let half = 5.5;
    let mut end_box = PathBuilder::default();
    end_box.add_rect(
        SkRect::new(ex - half, ey - half, ex + half, ey + half),
        None,
        None,
    );
    let end_path = end_box.detach();
    let mut end_fill = Paint::default();
    end_fill.set_anti_alias(true);
    end_fill.set_color(ACCENT);
    canvas.draw_path(&end_path, &end_fill);
    canvas.draw_path(&end_path, &outline_paint(Color::WHITE, 1.5));
}

/// Builds an antialiased stroke paint.
fn outline_paint(color: Color, width: f32) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(PaintStyle::Stroke);
    paint.set_stroke_width(width);
    paint.set_color(color);
    paint
}

/// Paints one handle, encoding its role by shape.
///
/// `08 23`: role is carried by shape, never by colour alone, so a corner, a
/// side and the rotation handle must be distinguishable at a glance.
fn paint_handle(
    canvas: &SkiaCanvas,
    handle: &SelectionHandle,
    all_handles: &[SelectionHandle],
    camera: &ViewportCamera,
) {
    let screen = camera.doc_to_screen(handle.doc_point);
    let (x, y) = (screen.x as f32, screen.y as f32);
    let half = HANDLE_GLYPH_PX / 2.0;

    let mut stroke_paint = Paint::default();
    stroke_paint.set_anti_alias(true);
    stroke_paint.set_style(PaintStyle::Stroke);
    stroke_paint.set_stroke_width(1.5);
    stroke_paint.set_color(ACCENT);

    let mut fill_paint = Paint::default();
    fill_paint.set_anti_alias(true);
    fill_paint.set_style(PaintStyle::Fill);
    fill_paint.set_color(Color::from_rgb(0xFF, 0xFF, 0xFF));

    match handle.kind {
        SelectionHandleKind::Rotation => {
            // Draw a connecting stem line towards the top-center handle
            if let Some(top_handle) = all_handles
                .iter()
                .find(|h| h.kind == SelectionHandleKind::Top)
            {
                let top_screen = camera.doc_to_screen(top_handle.doc_point);
                let mut stem = PathBuilder::default();
                stem.move_to((x, y));
                stem.line_to((top_screen.x as f32, top_screen.y as f32));
                canvas.draw_path(&stem.detach(), &stroke_paint);
            }

            // Draw circular knob
            let mut circ = PathBuilder::default();
            circ.add_circle((x, y), half + 1.0, None);
            let path = circ.detach();
            canvas.draw_path(&path, &fill_paint);
            canvas.draw_path(&path, &stroke_paint);
        }
        SelectionHandleKind::Top
        | SelectionHandleKind::Bottom
        | SelectionHandleKind::Left
        | SelectionHandleKind::Right => {
            // Diamond / pill handle for midpoints
            let mut builder = PathBuilder::default();
            builder.move_to((x, y - half));
            builder.line_to((x + half, y));
            builder.line_to((x, y + half));
            builder.line_to((x - half, y));
            builder.close();
            let path = builder.detach();
            canvas.draw_path(&path, &fill_paint);
            canvas.draw_path(&path, &stroke_paint);
        }
        SelectionHandleKind::TopLeft
        | SelectionHandleKind::TopRight
        | SelectionHandleKind::BottomLeft
        | SelectionHandleKind::BottomRight => {
            // Crisp square handle for corners
            let mut builder = PathBuilder::default();
            builder.add_rect(
                SkRect::new(x - half, y - half, x + half, y + half),
                None,
                None,
            );
            let path = builder.detach();
            canvas.draw_path(&path, &fill_paint);
            canvas.draw_path(&path, &stroke_paint);
        }
        SelectionHandleKind::NodeCusp => {
            let mut builder = PathBuilder::default();
            builder.add_rect(
                SkRect::new(x - half, y - half, x + half, y + half),
                None,
                None,
            );
            let path = builder.detach();
            canvas.draw_path(&path, &fill_paint);
            canvas.draw_path(&path, &stroke_paint);
        }
        SelectionHandleKind::NodeCuspSelected => {
            let mut builder = PathBuilder::default();
            builder.add_rect(
                SkRect::new(x - half, y - half, x + half, y + half),
                None,
                None,
            );
            let path = builder.detach();
            let mut solid_accent = fill_paint;
            solid_accent.set_color(ACCENT);
            let mut white_border = stroke_paint;
            white_border.set_color(Color::from_rgb(0xFF, 0xFF, 0xFF));
            canvas.draw_path(&path, &solid_accent);
            canvas.draw_path(&path, &white_border);
        }
        SelectionHandleKind::NodeSmooth => {
            let mut circ = PathBuilder::default();
            circ.add_circle((x, y), half, None);
            let path = circ.detach();
            canvas.draw_path(&path, &fill_paint);
            canvas.draw_path(&path, &stroke_paint);
        }
        SelectionHandleKind::NodeSmoothSelected => {
            let mut circ = PathBuilder::default();
            circ.add_circle((x, y), half, None);
            let path = circ.detach();
            let mut solid_accent = fill_paint;
            solid_accent.set_color(ACCENT);
            let mut white_border = stroke_paint;
            white_border.set_color(Color::from_rgb(0xFF, 0xFF, 0xFF));
            canvas.draw_path(&path, &solid_accent);
            canvas.draw_path(&path, &white_border);
        }
        SelectionHandleKind::NodeSymmetric => {
            let mut builder = PathBuilder::default();
            builder.move_to((x, y - half));
            builder.line_to((x + half, y));
            builder.line_to((x, y + half));
            builder.line_to((x - half, y));
            builder.close();
            let path = builder.detach();
            canvas.draw_path(&path, &fill_paint);
            canvas.draw_path(&path, &stroke_paint);
        }
        SelectionHandleKind::NodeSymmetricSelected => {
            let mut builder = PathBuilder::default();
            builder.move_to((x, y - half));
            builder.line_to((x + half, y));
            builder.line_to((x, y + half));
            builder.line_to((x - half, y));
            builder.close();
            let path = builder.detach();
            let mut solid_accent = fill_paint;
            solid_accent.set_color(ACCENT);
            let mut white_border = stroke_paint;
            white_border.set_color(Color::from_rgb(0xFF, 0xFF, 0xFF));
            canvas.draw_path(&path, &solid_accent);
            canvas.draw_path(&path, &white_border);
        }
        SelectionHandleKind::NodeControl => {
            let mut circ = PathBuilder::default();
            circ.add_circle((x, y), half - 0.5, None);
            let path = circ.detach();
            let mut knob_fill = fill_paint;
            knob_fill.set_color(ACCENT);
            let mut knob_border = stroke_paint;
            knob_border.set_stroke_width(1.0);
            knob_border.set_color(Color::from_rgb(0xFF, 0xFF, 0xFF));
            canvas.draw_path(&path, &knob_fill);
            canvas.draw_path(&path, &knob_border);
        }
    }
}

/// Paints a placed raster image object.
fn paint_image_object(
    canvas: &SkiaCanvas,
    object: &CanvasObjectProjection,
    path: &str,
    data: Option<&[u8]>,
    camera: &ViewportCamera,
    _opacity: f32,
) {
    let screen_origin = camera.doc_to_screen(GPoint::new(
        object.frame_origin[0],
        object.frame_origin[1],
    ));
    let w = (object.size[0] * camera.zoom).max(10.0) as f32;
    let h = (object.size[1] * camera.zoom).max(10.0) as f32;
    let dst = SkRect::new(
        screen_origin.x as f32,
        screen_origin.y as f32,
        screen_origin.x as f32 + w,
        screen_origin.y as f32 + h,
    );

    let decoded = if let Some(bytes) = data {
        Image::from_encoded(Data::new_copy(bytes))
    } else if let Ok(bytes) = std::fs::read(path) {
        Image::from_encoded(Data::new_copy(&bytes))
    } else {
        None
    };

    if let Some(img) = decoded {
        canvas.draw_image_rect(&img, None, dst, &Paint::default());
    } else {
        let mut bg = Paint::default();
        bg.set_color(Color::from_rgb(0xEB, 0xEE, 0xF5));
        bg.set_style(PaintStyle::Fill);
        canvas.draw_rect(dst, &bg);

        let border = outline_paint(Color::from_rgb(0x9C, 0xA3, 0xAF), 1.0);
        canvas.draw_rect(dst, &border);

        let mut font = Font::default();
        font.set_size(11.0);
        let mut text_paint = Paint::default();
        text_paint.set_color(Color::from_rgb(0x37, 0x41, 0x51));
        canvas.draw_str(
            &format!("🖼 {}", path),
            Point::new(screen_origin.x as f32 + 8.0, screen_origin.y as f32 + 20.0),
            &font,
            &text_paint,
        );
    }

    if object.active {
        canvas.draw_rect(dst, &outline_paint(ACCENT, 1.5));
    }
}

/// Paints horizontal and vertical graduated rulers (ptnd.surface.canvas.rulers)
fn paint_rulers(canvas: &SkiaCanvas, camera: &ViewportCamera) {
    let ruler_bg = Color::from_rgb(0x28, 0x2A, 0x2E);
    let tick_color = Color::from_rgb(0x60, 0x64, 0x6C);
    let text_color = Color::from_rgb(0xA0, 0xA4, 0xAC);
    let tick_paint = outline_paint(tick_color, 1.0);

    let mut bg_paint = Paint::default();
    bg_paint.set_color(ruler_bg);
    bg_paint.set_style(PaintStyle::Fill);

    let mut text_paint = Paint::default();
    text_paint.set_color(text_color);
    text_paint.set_anti_alias(true);

    let mut font = Font::default();
    font.set_size(9.0);

    let ruler_w = 20.0f32;
    let vw = camera.viewport_width as f32;
    let vh = camera.viewport_height as f32;

    // 1. Top horizontal ruler
    canvas.draw_rect(SkRect::new(0.0, 0.0, vw, ruler_w), &bg_paint);
    // 2. Left vertical ruler
    canvas.draw_rect(SkRect::new(0.0, 0.0, ruler_w, vh), &bg_paint);

    // Corner box
    let mut corner_paint = Paint::default();
    corner_paint.set_color(Color::from_rgb(0x1E, 0x20, 0x22));
    canvas.draw_rect(SkRect::new(0.0, 0.0, ruler_w, ruler_w), &corner_paint);

    // Compute step based on zoom
    let doc_step = if camera.zoom > 2.0 {
        20.0
    } else if camera.zoom > 0.5 {
        50.0
    } else {
        100.0
    };

    // Horizontal ticks
    let start_doc_x = camera.screen_to_doc(GPoint::new(ruler_w as f64, 0.0)).x;
    let end_doc_x = camera.screen_to_doc(GPoint::new(vw as f64, 0.0)).x;
    let first_tick_x = (start_doc_x / doc_step).floor() * doc_step;

    let mut curr_x = first_tick_x;
    while curr_x <= end_doc_x {
        let sx = camera.doc_to_screen(GPoint::new(curr_x, 0.0)).x as f32;
        if sx >= ruler_w {
            canvas.draw_line(Point::new(sx, ruler_w - 6.0), Point::new(sx, ruler_w), &tick_paint);
            let label_str = format!("{:.0}", curr_x);
            canvas.draw_str(&label_str, Point::new(sx + 2.0, ruler_w - 8.0), &font, &text_paint);
        }
        curr_x += doc_step;
    }

    // Vertical ticks
    let start_doc_y = camera.screen_to_doc(GPoint::new(0.0, ruler_w as f64)).y;
    let end_doc_y = camera.screen_to_doc(GPoint::new(0.0, vh as f64)).y;
    let first_tick_y = (start_doc_y / doc_step).floor() * doc_step;

    let mut curr_y = first_tick_y;
    while curr_y <= end_doc_y {
        let sy = camera.doc_to_screen(GPoint::new(0.0, curr_y)).y as f32;
        if sy >= ruler_w {
            canvas.draw_line(Point::new(ruler_w - 6.0, sy), Point::new(ruler_w, sy), &tick_paint);
            let label_str = format!("{:.0}", curr_y);
            canvas.save();
            canvas.translate((ruler_w - 8.0, sy + 2.0));
            canvas.rotate(-90.0, None);
            canvas.draw_str(&label_str, Point::new(0.0, 0.0), &font, &text_paint);
            canvas.restore();
        }
        curr_y += doc_step;
    }
}
