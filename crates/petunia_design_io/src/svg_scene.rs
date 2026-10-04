//! Scope-faithful SVG uses prepared local ink and explicit world transforms.
//! UTF-8 text is exported as shaped glyph outlines; the PTND source stays text.
use base64::{engine::general_purpose::STANDARD, Engine};
use petunia_design_document::{
    BlendMode, ContainerRole, Document, EffectKind, MaskMode, Paint, ShapeKind, StrokeAlignment,
    StrokeCap, StrokeJoin, TextFlow,
};
use petunia_design_foundation::{ObjectId, PetuniaError};
use petunia_design_geometry::{GAffine, GRect};
use petunia_design_render::{RenderNode, RenderSurface};
use std::collections::HashSet;
use std::fmt::Write;

fn invalid(reason: impl Into<String>) -> PetuniaError {
    PetuniaError::invalid_input(reason)
}
fn unavailable(id: ObjectId, feature: &str) -> PetuniaError {
    invalid(format!(
        "object {id}: unavailable SVG capability: {feature}"
    ))
}
fn xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
fn matrix(affine: GAffine) -> String {
    let [a, b, c, d, e, f] = affine.coeffs;
    format!("matrix({a} {b} {c} {d} {e} {f})")
}
fn paint_alpha(paint: &Paint) -> f64 {
    match paint {
        Paint::Solid(token) => f64::from(petunia_design_document::resolve_color_to_rgba(token)[3]),
        _ => 1.,
    }
}
fn blend(mode: BlendMode) -> &'static str {
    match mode {
        BlendMode::Normal => "normal",
        BlendMode::Multiply => "multiply",
        BlendMode::Screen => "screen",
        BlendMode::Overlay => "overlay",
        BlendMode::Darken => "darken",
        BlendMode::Lighten => "lighten",
        BlendMode::ColorDodge => "color-dodge",
        BlendMode::ColorBurn => "color-burn",
        BlendMode::HardLight => "hard-light",
        BlendMode::SoftLight => "soft-light",
        BlendMode::Difference => "difference",
        BlendMode::Exclusion => "exclusion",
        BlendMode::Hue => "hue",
        BlendMode::Saturation => "saturation",
        BlendMode::Color => "color",
        BlendMode::Luminosity => "luminosity",
    }
}
struct Writer<'a> {
    scene: &'a RenderSurface,
    colors: Option<petunia_design_color::CmykDisplayTransform>,
    definitions: String,
    serial: usize,
    masks: HashSet<(ObjectId, MaskMode)>,
    active_masks: HashSet<ObjectId>,
}
impl Writer<'_> {
    fn rgb(&self, token: &str) -> Result<String, PetuniaError> {
        let c = if let Some(ink) = petunia_design_color::parse_cmyk_token(token)? {
            self.colors
                .as_ref()
                .ok_or_else(|| invalid("CMYK SVG export requires an assigned ICC press profile"))?
                .convert(ink)?
        } else {
            petunia_design_document::resolve_color_to_rgb(token)
        };
        Ok(format!(
            "rgb({}% {}% {}%)",
            c[0] * 100.,
            c[1] * 100.,
            c[2] * 100.
        ))
    }

    fn id(&mut self, prefix: &str) -> String {
        self.serial += 1;
        format!("{prefix}-{}-{}", self.scene.id(), self.serial)
    }
    fn paint(&mut self, paint: &Paint) -> Result<String, PetuniaError> {
        let (opening, stops, closing) = match paint {
            Paint::None => return Ok("none".into()),
            Paint::Solid(token) => return self.rgb(token),
            Paint::LinearGradient(g) => {
                if g.stops.len() < 2 || g.start == g.end {
                    return Err(invalid(
                        "SVG gradient needs two stops and a nondegenerate vector",
                    ));
                }
                let id = self.id("paint");
                (format!("<linearGradient id=\"{id}\" gradientUnits=\"userSpaceOnUse\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" color-interpolation=\"sRGB\">", g.start[0],g.start[1],g.end[0],g.end[1]), (&g.stops, id), "</linearGradient>")
            }
            Paint::RadialGradient(g) => {
                if g.stops.len() < 2 || g.radius <= 0. {
                    return Err(invalid(
                        "SVG gradient needs two stops and a positive radius",
                    ));
                }
                let id = self.id("paint");
                (format!("<radialGradient id=\"{id}\" gradientUnits=\"userSpaceOnUse\" cx=\"{}\" cy=\"{}\" r=\"{}\" color-interpolation=\"sRGB\">",g.center[0],g.center[1],g.radius), (&g.stops, id), "</radialGradient>")
            }
        };
        self.definitions.push_str(&opening);
        let (stops, id) = stops;
        // Match the reference renderer's stable stop ordering without changing source order.
        let mut ordered: Vec<_> = stops.iter().collect();
        ordered.sort_by(|a, b| a.offset.total_cmp(&b.offset));
        for stop in ordered {
            let color = self.rgb(&stop.color)?;
            write!(
                self.definitions,
                "<stop offset=\"{}\" stop-color=\"{}\" stop-opacity=\"{}\"/>",
                stop.offset,
                color,
                stop.opacity
                    * f64::from(petunia_design_document::resolve_color_to_rgba(&stop.color)[3])
            )
            .unwrap();
        }
        self.definitions.push_str(closing);
        Ok(format!("url(#{id})"))
    }
    fn mask(&mut self, id: ObjectId, mode: MaskMode, depth: usize) -> Result<String, PetuniaError> {
        let name = format!(
            "mask-{}-{}-{}",
            self.scene.id(),
            id,
            match mode {
                MaskMode::Vector => "vector",
                MaskMode::Alpha => "alpha",
                MaskMode::Luminance => "luminance",
            }
        );
        if self.masks.contains(&(id, mode)) {
            return Ok(name);
        }
        if !self.active_masks.insert(id) {
            return Err(invalid("cyclic SVG mask reference"));
        }
        let scene = self.scene;
        let node = scene.node(id).ok_or_else(|| invalid("missing SVG mask"))?;
        if !node.source().is_clip_mask {
            return Err(invalid("SVG mask reference is not a mask"));
        }
        let definition = if mode == MaskMode::Vector {
            let d = if node.source().visible {
                crate::export_path_d(node.geometry())
            } else {
                String::new()
            };
            format!("<clipPath id=\"{name}\" clipPathUnits=\"userSpaceOnUse\"><path d=\"{d}\" transform=\"{}\" clip-rule=\"{}\"/></clipPath>", matrix(node.local_to_world()), rule(node))
        } else {
            // Include the same canonical mask subtree, opacity and paints.
            let body = self.node(id, depth + 1, true)?;
            let [x, y, w, h] = self.scene.bounds();
            let b = node
                .visual_bounds()
                .unwrap_or(GRect::new(x, y, x + w, y + h))
                .union(GRect::new(x, y, x + w, y + h))
                .unwrap();
            format!("<mask id=\"{name}\" maskUnits=\"userSpaceOnUse\" maskContentUnits=\"userSpaceOnUse\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" style=\"mask-type:{}\" color-interpolation=\"linearRGB\">{body}</mask>", b.x0,b.y0,b.width(),b.height(),if mode==MaskMode::Alpha {"alpha"} else {"luminance"})
        };
        self.definitions.push_str(&definition);
        self.active_masks.remove(&id);
        self.masks.insert((id, mode));
        Ok(name)
    }
    fn node(&mut self, id: ObjectId, depth: usize, as_mask: bool) -> Result<String, PetuniaError> {
        if depth >= 128 {
            return Err(invalid("SVG hierarchy/mask depth budget exceeded"));
        }
        let scene = self.scene;
        let node = scene.node(id).ok_or_else(|| invalid("missing SVG node"))?;
        let obj = node.source();
        if !obj.visible || (obj.is_clip_mask && !as_mask) {
            return Ok(String::new());
        }
        let app = obj.effective_appearance();
        if app.adjustments.iter().any(|a| a.visible) {
            return Err(unavailable(
                id,
                "linear-light live adjustments; use PNG output",
            ));
        }
        if obj.modifiers.iter().any(|m| {
            m.enabled
                && matches!(
                    m.kind,
                    petunia_design_document::ModifierKind::TransparentGradient { .. }
                )
        }) {
            return Err(unavailable(id, "spatial opacity modifiers; use PNG output"));
        }
        let transform = matrix(node.local_to_world());
        let d = crate::export_path_d(node.geometry());
        let mut body = String::new();
        if let Some(shape @ (ShapeKind::Image { .. } | ShapeKind::Raster { .. })) = &obj.shape {
            if obj.modifiers.iter().any(|m| {
                m.enabled
                    && !matches!(
                        m.kind,
                        petunia_design_document::ModifierKind::CropRect { .. }
                    )
            }) {
                return Err(unavailable(id, "raster geometry resampling"));
            }
            let raw = match shape {
                ShapeKind::Image {
                    data: Some(source), ..
                } => crate::import_display_raster(source.as_slice(), 32 * 1024 * 1024)?,
                ShapeKind::Image { .. } => return Err(unavailable(id, "embedded image source")),
                ShapeKind::Raster { layer } => crate::display_raster_layer(layer, &|| false)?,

                _ => unreachable!(),
            };
            let (png, report) = crate::export_raster(
                &raw,
                &crate::RasterExportOptions {
                    format: crate::RasterFormat::Png,
                    allow_degradations: false,
                    ..Default::default()
                },
            )?;
            if !report.is_empty() {
                return Err(unavailable(id, "lossless embedded PNG conversion"));
            }
            let [_, _, w, h] = obj
                .bounds
                .ok_or_else(|| invalid("SVG image frame missing"))?;
            let clip = self.id("image-coverage");
            write!(self.definitions,"<clipPath id=\"{clip}\" clipPathUnits=\"userSpaceOnUse\"><path d=\"{d}\" transform=\"{transform}\"/></clipPath>").unwrap();
            write!(body,"<g clip-path=\"url(#{clip})\"><image width=\"{w}\" height=\"{h}\" transform=\"{transform}\" preserveAspectRatio=\"none\" href=\"data:image/png;base64,{}\"/></g>",STANDARD.encode(png)).unwrap();
        } else {
            for fill in app.fills.iter().filter(|f| f.visible) {
                let paint = self.paint(&fill.paint)?;
                write!(body,"<path d=\"{d}\" transform=\"{transform}\" fill=\"{paint}\" fill-rule=\"{}\" fill-opacity=\"{}\" style=\"mix-blend-mode:{}\"/>",rule(node),fill.opacity*paint_alpha(&fill.paint),blend(fill.blend_mode)).unwrap();
            }
        }
        for stroke in app.strokes.iter().filter(|s| s.visible) {
            if stroke.alignment != StrokeAlignment::Center {
                return Err(unavailable(id, "inside/outside stroke representation"));
            }
            let paint = self.paint(&stroke.paint)?;
            let cap = match stroke.cap {
                StrokeCap::Butt => "butt",
                StrokeCap::Round => "round",
                StrokeCap::Square => "square",
            };
            let join = match stroke.join {
                StrokeJoin::Miter => "miter",
                StrokeJoin::Round => "round",
                StrokeJoin::Bevel => "bevel",
            };
            let dash = if stroke.dash_array.is_empty() {
                "none".into()
            } else {
                stroke
                    .dash_array
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            write!(body,"<path d=\"{d}\" transform=\"{transform}\" fill=\"none\" stroke=\"{paint}\" stroke-width=\"{}\" stroke-opacity=\"{}\" stroke-linecap=\"{cap}\" stroke-linejoin=\"{join}\" stroke-miterlimit=\"{}\" stroke-dasharray=\"{dash}\" stroke-dashoffset=\"{}\" style=\"mix-blend-mode:{}\"/>",stroke.width,stroke.opacity*paint_alpha(&stroke.paint),stroke.miter_limit,stroke.dash_offset,blend(stroke.blend_mode)).unwrap();
        }
        if matches!(obj.shape, Some(ShapeKind::Text { .. }))
            && obj.text_style.flow == TextFlow::Frame
        {
            let [_, _, w, h] = obj
                .bounds
                .ok_or_else(|| invalid("SVG text frame missing"))?;
            let clip = self.id("text-frame");
            write!(self.definitions,"<clipPath id=\"{clip}\" clipPathUnits=\"userSpaceOnUse\"><rect width=\"{w}\" height=\"{h}\" transform=\"{transform}\"/></clipPath>").unwrap();
            body = format!("<g clip-path=\"url(#{clip})\">{body}</g>");
        }
        for child in &obj.children {
            body.push_str(&self.node(*child, depth + 1, as_mask)?);
            if body.len() + self.definitions.len() > 128 * 1024 * 1024 {
                return Err(invalid("SVG output byte budget exceeded"));
            }
        }
        let mut effects = String::new();
        for (i, effect) in app.effects.iter().filter(|e| e.visible).enumerate() {
            match &effect.kind {
                EffectKind::GaussianBlur { radius } => write!(effects,"<feGaussianBlur stdDeviation=\"{radius}\" result=\"effect-{i}\"/>").unwrap(),
                EffectKind::DropShadow { offset, blur, opacity, color } => write!(effects,"<feDropShadow dx=\"{}\" dy=\"{}\" stdDeviation=\"{blur}\" flood-color=\"{}\" flood-opacity=\"{}\" result=\"effect-{i}\"/>",offset[0],offset[1],self.rgb(color)?,opacity*f64::from(petunia_design_document::resolve_color_to_rgba(color)[3])).unwrap(),
                _ => return Err(unavailable(id,"effect kernel")),
            }
        }
        if !effects.is_empty() {
            if let Some(bounds) = node.visual_bounds() {
                let filter = self.id("effects");
                write!(self.definitions,"<filter id=\"{filter}\" filterUnits=\"userSpaceOnUse\" primitiveUnits=\"userSpaceOnUse\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" color-interpolation-filters=\"sRGB\">{effects}</filter>",bounds.x0,bounds.y0,bounds.width(),bounds.height()).unwrap();
                body = format!("<g filter=\"url(#{filter})\">{body}</g>");
            }
        }
        if let Some(mask_id) = obj.clip_mask_id {
            let grouped = obj
                .parent
                .and_then(|p| self.scene.node(p))
                .is_some_and(|parent| {
                    parent.source().role == Some(ContainerRole::ClipGroup)
                        && parent.source().children.contains(&mask_id)
                });
            if !grouped {
                let mask = self
                    .scene
                    .node(mask_id)
                    .ok_or_else(|| invalid("missing SVG clip mask"))?;
                let mode = if mask.source().mask_mode != MaskMode::Vector {
                    mask.source().mask_mode
                } else {
                    obj.mask_mode
                };
                let reference = self.mask(mask_id, mode, depth)?;
                let attr = if mode == MaskMode::Vector {
                    "clip-path"
                } else {
                    "mask"
                };
                body = format!("<g {attr}=\"url(#{reference})\">{body}</g>");
            }
        }
        if obj.role == Some(ContainerRole::ClipGroup) {
            let masks: Vec<_> = obj
                .children
                .iter()
                .filter(|child| {
                    self.scene
                        .node(**child)
                        .is_some_and(|n| n.source().is_clip_mask)
                })
                .copied()
                .collect();
            if masks.len() != 1 {
                return Err(invalid("SVG ClipGroup needs one mask"));
            }
            let mode = self.scene.node(masks[0]).unwrap().source().mask_mode;
            let reference = self.mask(masks[0], mode, depth)?;
            let attr = if mode == MaskMode::Vector {
                "clip-path"
            } else {
                "mask"
            };
            body = format!("<g {attr}=\"url(#{reference})\">{body}</g>");
        }
        let instance = if as_mask {
            self.id("mask-object")
        } else {
            obj.id.to_string()
        };
        Ok(format!("<g id=\"{instance}\" data-object-id=\"{}\" data-name=\"{}\" opacity=\"{}\" style=\"isolation:isolate;mix-blend-mode:{}\">{body}</g>",obj.id,xml(&obj.name),app.opacity,blend(app.blend_mode)))
    }
}
fn rule(node: &RenderNode) -> &'static str {
    if matches!(node.source().shape, Some(ShapeKind::Text { .. }))
        || node.source().fill_rule == petunia_design_geometry::FillRule::NonZero
    {
        "nonzero"
    } else {
        "evenodd"
    }
}
pub(super) fn export(document: &Document) -> Result<String, PetuniaError> {
    document.validate()?;
    let included: Vec<_> = document
        .surfaces()
        .iter()
        .filter(|s| s.export_enabled)
        .collect();
    if included.is_empty() {
        return Ok("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1 1\"></svg>\n".into());
    }
    let bounds = included
        .iter()
        .map(|s| {
            GRect::new(
                s.origin[0],
                s.origin[1],
                s.origin[0] + s.dimensions[0],
                s.origin[1] + s.dimensions[1],
            )
        })
        .reduce(|a, b| a.union(b).unwrap())
        .unwrap();
    // A single page retains its authored physical dimensions, without the
    // cancellation error introduced by subtracting a large world origin.
    let [width, height] = if included.len() == 1 {
        included[0].dimensions
    } else {
        [bounds.width(), bounds.height()]
    };
    let mut output=format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}pt\" height=\"{height}pt\" viewBox=\"{} {} {width} {height}\">\n",bounds.x0,bounds.y0);
    for surface in included {
        let scene = RenderSurface::extract(surface).map_err(|e| invalid(e.to_string()))?;
        let mut writer = Writer {
            scene: &scene,
            colors: surface
                .cmyk_profile
                .as_ref()
                .map(|profile| {
                    petunia_design_color::CmykDisplayTransform::new(profile, Default::default())
                })
                .transpose()?,
            definitions: String::new(),
            serial: 0,
            masks: HashSet::new(),
            active_masks: HashSet::new(),
        };
        let mut body = String::new();
        for root in scene.roots() {
            body.push_str(&writer.node(*root, 0, false)?);
        }
        let [x, y, w, h] = scene.bounds();
        let clip = writer.id("artboard");
        write!(writer.definitions,"<clipPath id=\"{clip}\" clipPathUnits=\"userSpaceOnUse\"><rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\"/></clipPath>").unwrap();
        writeln!(output,"<defs>{}</defs><g id=\"{}\" data-name=\"{}\" clip-path=\"url(#{clip})\" style=\"isolation:isolate\">{body}</g>",writer.definitions,surface.id,xml(&surface.name)).unwrap();
        if output.len() > 128 * 1024 * 1024 {
            return Err(invalid("SVG output byte budget exceeded"));
        }
    }
    output.push_str("</svg>\n");
    Ok(output)
}
