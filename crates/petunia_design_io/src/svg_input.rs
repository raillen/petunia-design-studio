//! Declared SVG input subset: basic paths/shapes, solid paints, inherited
//! presentation and transformed groups. Unsupported SVG is never discarded.
use petunia_design_document::{
    AppearanceStack, ContainerRole, Document, DocumentMutator, DocumentObject, FillItem, Paint,
    ShapeKind, StrokeCap, StrokeItem, StrokeJoin,
};
use petunia_design_foundation::{IdGenerator, ObjectId, PetuniaError, SurfaceId};
use petunia_design_geometry::{GAffine, GPath, GPoint, GRect, PathVerb};
use std::io::Read;
use std::path::Path;

const MAX_INPUT: usize = 16 * 1024 * 1024;
const MAX_VERBS: usize = 1_000_000;
fn invalid(reason: impl Into<String>) -> PetuniaError {
    PetuniaError::invalid_input(reason)
}
struct Numbers<'a> {
    source: &'a str,
    pos: usize,
}
impl<'a> Numbers<'a> {
    fn new(source: &'a str) -> Self {
        Self { source, pos: 0 }
    }
    fn skip(&mut self) {
        while self
            .source
            .as_bytes()
            .get(self.pos)
            .is_some_and(|b| b.is_ascii_whitespace() || *b == b',')
        {
            self.pos += 1;
        }
    }
    fn peek(&mut self) -> Option<u8> {
        self.skip();
        self.source.as_bytes().get(self.pos).copied()
    }
    fn number(&mut self) -> Result<f64, PetuniaError> {
        self.skip();
        let start = self.pos;
        let bytes = self.source.as_bytes();
        if bytes
            .get(self.pos)
            .is_some_and(|b| matches!(b, b'+' | b'-'))
        {
            self.pos += 1;
        }
        let mut digits = 0;
        while bytes.get(self.pos).is_some_and(u8::is_ascii_digit) {
            self.pos += 1;
            digits += 1;
        }
        if bytes.get(self.pos) == Some(&b'.') {
            self.pos += 1;
            while bytes.get(self.pos).is_some_and(u8::is_ascii_digit) {
                self.pos += 1;
                digits += 1;
            }
        }
        if digits == 0 {
            return Err(invalid("expected finite SVG number"));
        }
        if bytes
            .get(self.pos)
            .is_some_and(|b| matches!(b, b'e' | b'E'))
        {
            self.pos += 1;
            if bytes
                .get(self.pos)
                .is_some_and(|b| matches!(b, b'+' | b'-'))
            {
                self.pos += 1;
            }
            let exponent = self.pos;
            while bytes.get(self.pos).is_some_and(u8::is_ascii_digit) {
                self.pos += 1;
            }
            if exponent == self.pos {
                return Err(invalid("invalid SVG exponent"));
            }
        }
        let value = self.source[start..self.pos]
            .parse::<f64>()
            .map_err(|_| invalid("invalid SVG number"))?;
        if !value.is_finite() || value.abs() > 1_000_000_000. {
            return Err(invalid("SVG coordinate/metric budget exceeded"));
        }
        Ok(value)
    }
    fn point(&mut self, origin: GPoint, relative: bool) -> Result<GPoint, PetuniaError> {
        let x = self.number()?;
        let y = self.number()?;
        Ok(if relative {
            GPoint::new(origin.x + x, origin.y + y)
        } else {
            GPoint::new(x, y)
        })
    }
}
pub(super) fn parse_path(source: &str) -> Result<GPath, PetuniaError> {
    if source.len() > MAX_INPUT {
        return Err(invalid("SVG path byte budget exceeded"));
    }
    let mut tokens = Numbers::new(source);
    let mut path = GPath::new();
    let mut command = None;
    let mut current = GPoint::ORIGIN;
    let mut start = current;
    let mut cubic = None;
    let mut quad = None;
    while let Some(next) = tokens.peek() {
        if next.is_ascii_alphabetic() {
            tokens.pos += 1;
            command = Some(next)
        }
        let cmd = command.ok_or_else(|| invalid("SVG path needs a command"))?;
        let relative = cmd.is_ascii_lowercase();
        let upper = cmd.to_ascii_uppercase();
        if path.is_empty() && upper != b'M' {
            return Err(invalid("SVG path must start with moveto"));
        }
        if path.verbs.len() >= MAX_VERBS {
            return Err(invalid("SVG path verb budget exceeded"));
        }
        let verb = match upper {
            b'M' => {
                let p = tokens.point(current, relative)?;
                start = p;
                current = p;
                command = Some(if relative { b'l' } else { b'L' });
                PathVerb::MoveTo(p)
            }
            b'L' => {
                current = tokens.point(current, relative)?;
                PathVerb::LineTo(current)
            }
            b'H' => {
                let x = tokens.number()?;
                current.x = if relative { current.x + x } else { x };
                PathVerb::LineTo(current)
            }
            b'V' => {
                let y = tokens.number()?;
                current.y = if relative { current.y + y } else { y };
                PathVerb::LineTo(current)
            }
            b'C' => {
                let a = tokens.point(current, relative)?;
                let b = tokens.point(current, relative)?;
                let p = tokens.point(current, relative)?;
                current = p;
                cubic = Some(b);
                PathVerb::CubicTo(a, b, p)
            }
            b'S' => {
                let a = cubic.map_or(current, |p: GPoint| {
                    GPoint::new(2. * current.x - p.x, 2. * current.y - p.y)
                });
                let b = tokens.point(current, relative)?;
                let p = tokens.point(current, relative)?;
                current = p;
                cubic = Some(b);
                PathVerb::CubicTo(a, b, p)
            }
            b'Q' => {
                let a = tokens.point(current, relative)?;
                let p = tokens.point(current, relative)?;
                current = p;
                quad = Some(a);
                PathVerb::QuadTo(a, p)
            }
            b'T' => {
                let a = quad.map_or(current, |p: GPoint| {
                    GPoint::new(2. * current.x - p.x, 2. * current.y - p.y)
                });
                let p = tokens.point(current, relative)?;
                current = p;
                quad = Some(a);
                PathVerb::QuadTo(a, p)
            }
            b'Z' => {
                current = start;
                command = None;
                PathVerb::Close
            }
            b'A' => {
                return Err(invalid(
                    "unavailable SVG import capability: elliptical arcs",
                ))
            }
            _ => {
                return Err(invalid(format!(
                    "unavailable SVG path command: {}",
                    char::from(cmd)
                )))
            }
        };
        if !matches!(upper, b'C' | b'S') {
            cubic = None
        }
        if !matches!(upper, b'Q' | b'T') {
            quad = None
        }
        path.push(verb).map_err(invalid)?;
    }
    Ok(path)
}
fn list(source: &str) -> Result<Vec<f64>, PetuniaError> {
    let mut tokens = Numbers::new(source);
    let mut result = Vec::new();
    while tokens.peek().is_some() {
        if result.len() >= 4096 {
            return Err(invalid("SVG number list budget exceeded"));
        }
        result.push(tokens.number()?)
    }
    Ok(result)
}
fn transform(source: &str) -> Result<GAffine, PetuniaError> {
    let mut rest = source.trim();
    let mut result = GAffine::IDENTITY;
    let mut count = 0;
    while !rest.is_empty() {
        count += 1;
        if count > 128 {
            return Err(invalid("SVG transform budget exceeded"));
        }
        let open = rest
            .find('(')
            .ok_or_else(|| invalid("invalid SVG transform"))?;
        let close = rest[open + 1..]
            .find(')')
            .map(|i| i + open + 1)
            .ok_or_else(|| invalid("invalid SVG transform"))?;
        let values = list(&rest[open + 1..close])?;
        let value = match (rest[..open].trim(), values.as_slice()) {
            ("matrix", [a, b, c, d, e, f]) => GAffine::new([*a, *b, *c, *d, *e, *f]),
            ("translate", [x]) => GAffine::translate(*x, 0.),
            ("translate", [x, y]) => GAffine::translate(*x, *y),
            ("scale", [s]) => GAffine::scale(*s, *s),
            ("scale", [x, y]) => GAffine::scale(*x, *y),
            ("rotate", [angle]) => GAffine::rotate(angle.to_radians()),
            ("rotate", [angle, x, y]) => GAffine::translate(*x, *y)
                .after(GAffine::rotate(angle.to_radians()))
                .after(GAffine::translate(-x, -y)),
            ("skewX", [angle]) => GAffine::new([1., 0., angle.to_radians().tan(), 1., 0., 0.]),
            ("skewY", [angle]) => GAffine::new([1., angle.to_radians().tan(), 0., 1., 0., 0.]),
            _ => return Err(invalid("unavailable/invalid SVG transform")),
        };
        result = result.after(value);
        if !result.coeffs.iter().all(|n| n.is_finite()) || result.inverse().is_none() {
            return Err(invalid("singular or invalid SVG transform"));
        }
        rest = rest[close + 1..].trim_start_matches(|c: char| c.is_ascii_whitespace() || c == ',');
    }
    Ok(result)
}
fn length(source: &str) -> Result<f64, PetuniaError> {
    let text = source.trim();
    let units = [
        ("px", 0.75),
        ("pt", 1.),
        ("pc", 12.),
        ("mm", 72. / 25.4),
        ("cm", 72. / 2.54),
        ("in", 72.),
    ];
    for (suffix, factor) in units {
        if let Some(number) = text.strip_suffix(suffix) {
            return scalar(number).map(|v| v * factor);
        }
    }
    scalar(text).map(|v| v * 0.75)
}
fn scalar(text: &str) -> Result<f64, PetuniaError> {
    let values = list(text)?;
    if values.len() != 1 {
        return Err(invalid("expected one SVG scalar"));
    }
    Ok(values[0])
}
fn metric(node: roxmltree::Node<'_, '_>, name: &str, default: f64) -> Result<f64, PetuniaError> {
    node.attribute(name).map(scalar).unwrap_or(Ok(default))
}
fn color(value: &str) -> Result<Paint, PetuniaError> {
    let value = value.trim();
    if value == "none" {
        return Ok(Paint::None);
    }
    let normalized = match value {
        "black" => "#000000",
        "white" => "#ffffff",
        "red" => "#ff0000",
        "green" => "#008000",
        "blue" => "#0000ff",
        "yellow" => "#ffff00",
        _ => value,
    };
    if normalized.starts_with('#')
        && matches!(normalized.len(), 4 | 7)
        && normalized[1..].bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Ok(Paint::Solid(normalized.to_owned()));
    }
    Err(invalid(format!(
        "unavailable SVG solid paint syntax: {value}"
    )))
}
#[derive(Clone)]
struct Presentation {
    fill: Paint,
    stroke: Paint,
    width: f64,
    fill_opacity: f64,
    stroke_opacity: f64,
    cap: StrokeCap,
    join: StrokeJoin,
    miter: f64,
    dashes: Vec<f64>,
    dash_offset: f64,
    fill_rule: petunia_design_geometry::FillRule,
}
impl Default for Presentation {
    fn default() -> Self {
        Self {
            fill: Paint::Solid("#000000".into()),
            stroke: Paint::None,
            width: 1.,
            fill_opacity: 1.,
            stroke_opacity: 1.,
            cap: StrokeCap::Butt,
            join: StrokeJoin::Miter,
            miter: 4.,
            dashes: Vec::new(),
            dash_offset: 0.,
            fill_rule: petunia_design_geometry::FillRule::NonZero,
        }
    }
}
fn unit(value: f64) -> Result<f64, PetuniaError> {
    if (0. ..=1.).contains(&value) {
        Ok(value)
    } else {
        Err(invalid("SVG opacity must be in 0..=1"))
    }
}
fn presentation(
    node: roxmltree::Node<'_, '_>,
    inherited: &Presentation,
) -> Result<Presentation, PetuniaError> {
    let mut result = inherited.clone();
    for attr in node.attributes() {
        match attr.name() {
            "fill" => result.fill = color(attr.value())?,
            "stroke" => result.stroke = color(attr.value())?,
            "stroke-width" => result.width = scalar(attr.value())?,
            "fill-opacity" => result.fill_opacity = unit(scalar(attr.value())?)?,
            "stroke-opacity" => result.stroke_opacity = unit(scalar(attr.value())?)?,
            "stroke-linecap" => {
                result.cap = match attr.value() {
                    "butt" => StrokeCap::Butt,
                    "round" => StrokeCap::Round,
                    "square" => StrokeCap::Square,
                    _ => return Err(invalid("invalid SVG stroke cap")),
                }
            }
            "stroke-linejoin" => {
                result.join = match attr.value() {
                    "miter" => StrokeJoin::Miter,
                    "round" => StrokeJoin::Round,
                    "bevel" => StrokeJoin::Bevel,
                    _ => return Err(invalid("invalid SVG stroke join")),
                }
            }
            "stroke-miterlimit" => result.miter = scalar(attr.value())?,
            "stroke-dasharray" => {
                result.dashes = if attr.value() == "none" {
                    Vec::new()
                } else {
                    list(attr.value())?
                }
            }
            "stroke-dashoffset" => result.dash_offset = scalar(attr.value())?,
            "fill-rule" | "clip-rule" => {
                result.fill_rule = match attr.value() {
                    "evenodd" => petunia_design_geometry::FillRule::EvenOdd,
                    "nonzero" => petunia_design_geometry::FillRule::NonZero,
                    _ => return Err(invalid("invalid SVG fill rule")),
                }
            }
            _ => {}
        }
    }
    if result.width < 0. || result.miter < 1. || result.dashes.iter().any(|v| *v < 0.) {
        return Err(invalid("invalid SVG stroke metrics"));
    }
    Ok(result)
}
fn shape(node: roxmltree::Node<'_, '_>) -> Result<GPath, PetuniaError> {
    Ok(match node.tag_name().name() {
        "path" => parse_path(node.attribute("d").unwrap_or(""))?,
        "rect" => {
            let x = metric(node, "x", 0.)?;
            let y = metric(node, "y", 0.)?;
            let w = metric(node, "width", 0.)?;
            let h = metric(node, "height", 0.)?;
            if w < 0. || h < 0. {
                return Err(invalid("negative SVG rectangle size"));
            }
            let rx = metric(node, "rx", metric(node, "ry", 0.)?)?;
            let ry = metric(node, "ry", rx)?;
            if rx < 0. || ry < 0. {
                return Err(invalid("negative SVG rectangle radius"));
            }
            if w == 0. || h == 0. {
                GPath::new()
            } else {
                GPath::rect(GRect::new(x, y, x + w, y + h), rx, ry)
            }
        }
        "circle" | "ellipse" => {
            let center = GPoint::new(metric(node, "cx", 0.)?, metric(node, "cy", 0.)?);
            let rx = if node.has_tag_name("circle") {
                metric(node, "r", 0.)?
            } else {
                metric(node, "rx", 0.)?
            };
            let ry = if node.has_tag_name("circle") {
                rx
            } else {
                metric(node, "ry", 0.)?
            };
            if rx < 0. || ry < 0. {
                return Err(invalid("negative SVG ellipse radius"));
            }
            if rx == 0. || ry == 0. {
                GPath::new()
            } else {
                GPath::ellipse(center, rx, ry)
            }
        }
        "line" => GPath::line(
            GPoint::new(metric(node, "x1", 0.)?, metric(node, "y1", 0.)?),
            GPoint::new(metric(node, "x2", 0.)?, metric(node, "y2", 0.)?),
        ),
        "polygon" | "polyline" => {
            let values = list(node.attribute("points").unwrap_or(""))?;
            if values.len() % 2 != 0 {
                return Err(invalid("invalid SVG points"));
            }
            let mut path = GPath::new();
            for (i, p) in values.chunks_exact(2).enumerate() {
                path.push(if i == 0 {
                    PathVerb::MoveTo(GPoint::new(p[0], p[1]))
                } else {
                    PathVerb::LineTo(GPoint::new(p[0], p[1]))
                })
                .map_err(invalid)?;
            }
            if node.has_tag_name("polygon") && !path.is_empty() {
                path.push(PathVerb::Close).map_err(invalid)?
            }
            path
        }
        _ => {
            return Err(invalid(format!(
                "unavailable SVG import element: {}",
                node.tag_name().name()
            )))
        }
    })
}
fn attributes(node: roxmltree::Node<'_, '_>, root: bool) -> Result<(), PetuniaError> {
    const COMMON: &[&str] = &[
        "id",
        "data-name",
        "transform",
        "fill",
        "stroke",
        "stroke-width",
        "fill-opacity",
        "stroke-opacity",
        "stroke-linecap",
        "stroke-linejoin",
        "stroke-miterlimit",
        "stroke-dasharray",
        "stroke-dashoffset",
        "fill-rule",
        "clip-rule",
        "opacity",
    ];
    let specific: &[&str] = match node.tag_name().name() {
        "svg" if root => &[
            "width",
            "height",
            "viewBox",
            "version",
            "preserveAspectRatio",
        ],
        "g" => &[],
        "path" => &["d"],
        "rect" => &["x", "y", "width", "height", "rx", "ry"],
        "circle" => &["cx", "cy", "r"],
        "ellipse" => &["cx", "cy", "rx", "ry"],
        "line" => &["x1", "y1", "x2", "y2"],
        "polygon" | "polyline" => &["points"],
        "title" | "desc" => &[],
        _ => {
            return Err(invalid(format!(
                "unavailable SVG import element: {}",
                node.tag_name().name()
            )))
        }
    };
    for attr in node.attributes() {
        if attr.namespace().is_some()
            || (!COMMON.contains(&attr.name()) && !specific.contains(&attr.name()))
        {
            return Err(invalid(format!(
                "unavailable SVG import attribute: {}",
                attr.name()
            )));
        }
    }
    Ok(())
}
struct Import<'a> {
    mutator: DocumentMutator<'a>,
    ids: IdGenerator,
    surface: SurfaceId,
    objects: usize,
    verbs: usize,
}
impl Import<'_> {
    fn node(
        &mut self,
        node: roxmltree::Node<'_, '_>,
        parent: Option<ObjectId>,
        world: GAffine,
        inherited: &Presentation,
        depth: usize,
    ) -> Result<Option<ObjectId>, PetuniaError> {
        if depth >= 128 || self.objects >= 10_000 {
            return Err(invalid("SVG hierarchy/object budget exceeded"));
        }
        if !node.is_element() {
            return Ok(None);
        }
        if node
            .tag_name()
            .namespace()
            .is_some_and(|uri| uri != "http://www.w3.org/2000/svg")
        {
            return Err(invalid("foreign SVG namespace"));
        }
        attributes(node, false)?;
        if matches!(node.tag_name().name(), "title" | "desc") {
            return Ok(None);
        }
        let presentation = presentation(node, inherited)?;
        let world = world.after(
            node.attribute("transform")
                .map(transform)
                .unwrap_or(Ok(GAffine::IDENTITY))?,
        );
        let id = self.ids.next_object();
        self.objects += 1;
        let mut object = DocumentObject::new(
            id,
            node.attribute("data-name")
                .or_else(|| node.attribute("id"))
                .unwrap_or(node.tag_name().name()),
        );
        object.opacity = unit(metric(node, "opacity", 1.)?)?;
        object.fill_rule = presentation.fill_rule;
        if node.has_tag_name("g") {
            object.role = Some(ContainerRole::Group);
            self.mutator.add_object(self.surface, object)?;
            let mut children = Vec::new();
            for child in node.children() {
                if child.is_text() && child.text().is_some_and(|t| !t.trim().is_empty()) {
                    return Err(invalid("unavailable SVG text outside metadata"));
                }
                if let Some(id) = self.node(child, Some(id), world, &presentation, depth + 1)? {
                    children.push(id)
                }
            }
            self.mutator.force_children(id, children)?;
        } else {
            if node.children().any(|c| {
                c.is_element() || (c.is_text() && c.text().is_some_and(|t| !t.trim().is_empty()))
            }) {
                return Err(invalid("unavailable SVG shape child content"));
            }
            let path = shape(node)?.transformed(world);
            self.verbs = self
                .verbs
                .checked_add(path.verbs.len())
                .ok_or_else(|| invalid("SVG path budget overflow"))?;
            if self.verbs > MAX_VERBS {
                return Err(invalid("SVG aggregate path verb budget exceeded"));
            }
            let Some(bounds) = path.bounding_box() else {
                return Ok(None);
            };
            object.bounds = Some([
                bounds.x0,
                bounds.y0,
                bounds.width().max(1.),
                bounds.height().max(1.),
            ]);
            object.shape = Some(ShapeKind::Path(path));
            let mut appearance = AppearanceStack::new();
            appearance.fills.clear();
            appearance.strokes.clear();
            if presentation.fill != Paint::None {
                let mut fill = FillItem::solid(1, "#000000");
                fill.paint = presentation.fill.clone();
                fill.opacity = presentation.fill_opacity;
                appearance.fills.push(fill);
            }
            if presentation.stroke != Paint::None && presentation.width > 0. {
                let [a, b, c, d, _, _] = world.coeffs;
                let sx = a.hypot(b);
                let sy = c.hypot(d);
                if (sx - sy).abs() > 1e-8 * sx.max(sy) || (a * c + b * d).abs() > 1e-8 * sx * sy {
                    return Err(invalid(
                        "unavailable SVG import capability: anisotropic/sheared strokes",
                    ));
                }
                let mut stroke = StrokeItem::solid(1, "#000000", presentation.width * sx);
                stroke.paint = presentation.stroke.clone();
                stroke.opacity = presentation.stroke_opacity;
                stroke.cap = presentation.cap;
                stroke.join = presentation.join;
                stroke.miter_limit = presentation.miter;
                stroke.dash_array = presentation.dashes.iter().map(|v| v * sx).collect();
                stroke.dash_offset = presentation.dash_offset * sx;
                appearance.strokes.push(stroke);
            }
            object.appearance = Some(appearance);
            self.mutator.add_object(self.surface, object)?;
        }
        if let Some(parent) = parent {
            self.mutator.force_parent(id, Some(parent))?
        }
        Ok(Some(id))
    }
}
/// Loads a declared basic subset into a new editable document. Limits and
/// unsupported features are checked before the caller attaches a session.
pub fn import_svg(source: &str) -> Result<Document, PetuniaError> {
    if source.len() > MAX_INPUT {
        return Err(invalid("SVG input byte budget exceeded"));
    }
    let xml = roxmltree::Document::parse_with_options(
        source,
        roxmltree::ParsingOptions {
            allow_dtd: false,
            nodes_limit: 100_000,
        },
    )
    .map_err(|e| invalid(format!("invalid SVG XML: {e}")))?;
    let root = xml.root_element();
    if !root.has_tag_name("svg")
        || root
            .tag_name()
            .namespace()
            .is_some_and(|ns| ns != "http://www.w3.org/2000/svg")
    {
        return Err(invalid("expected SVG root"));
    }
    attributes(root, true)?;
    if root.attribute("opacity").is_some_and(|v| v != "1") {
        return Err(invalid("unavailable SVG root opacity"));
    }
    let view = root.attribute("viewBox").map(list).transpose()?;
    if view
        .as_ref()
        .is_some_and(|v| v.len() != 4 || v[2] <= 0. || v[3] <= 0.)
    {
        return Err(invalid("invalid SVG viewBox"));
    }
    let width = root
        .attribute("width")
        .map(length)
        .transpose()?
        .unwrap_or_else(|| view.as_ref().map_or(225., |v| v[2] * 0.75));
    let height = root
        .attribute("height")
        .map(length)
        .transpose()?
        .unwrap_or_else(|| view.as_ref().map_or(112.5, |v| v[3] * 0.75));
    if width <= 0. || height <= 0. || width > 1_000_000. || height > 1_000_000. {
        return Err(invalid("invalid SVG viewport"));
    }
    let world = if let Some(view) = view {
        let sx = width / view[2];
        let sy = height / view[3];
        match root
            .attribute("preserveAspectRatio")
            .unwrap_or("xMidYMid meet")
        {
            "none" => GAffine::scale(sx, sy).after(GAffine::translate(-view[0], -view[1])),
            "xMidYMid" | "xMidYMid meet" => {
                let scale = sx.min(sy);
                GAffine::translate(
                    (width - view[2] * scale) * 0.5,
                    (height - view[3] * scale) * 0.5,
                )
                .after(GAffine::scale(scale, scale))
                .after(GAffine::translate(-view[0], -view[1]))
            }
            _ => return Err(invalid("unavailable SVG aspect ratio mode")),
        }
    } else {
        GAffine::scale(0.75, 0.75)
    };
    let world = world.after(
        root.attribute("transform")
            .map(transform)
            .unwrap_or(Ok(GAffine::IDENTITY))?,
    );
    let style = presentation(root, &Presentation::default())?;
    let mut document = Document::new();
    let mut ids = IdGenerator::new();
    let surface = ids.next_surface();
    {
        let mut importer = Import {
            mutator: DocumentMutator::new(&mut document),
            ids,
            surface,
            objects: 0,
            verbs: 0,
        };
        importer.mutator.add_surface(surface, "SVG")?;
        importer
            .mutator
            .set_surface_geometry(surface, [0., 0.], [width, height])?;
        importer.mutator.set_surface_background(surface, None)?;
        for child in root.children() {
            if child.is_text() && child.text().is_some_and(|t| !t.trim().is_empty()) {
                return Err(invalid("unavailable SVG root text"));
            }
            importer.node(child, None, world, &style, 0)?;
        }
    }
    document.validate()?;
    Ok(document)
}
pub fn read_svg(path: &Path) -> Result<Document, PetuniaError> {
    let file = std::fs::File::open(path).map_err(|e| PetuniaError::io(e.to_string()))?;
    if file
        .metadata()
        .map_err(|e| PetuniaError::io(e.to_string()))?
        .len()
        > MAX_INPUT as u64
    {
        return Err(invalid("SVG input byte budget exceeded"));
    }
    let mut source = String::new();
    file.take((MAX_INPUT + 1) as u64)
        .read_to_string(&mut source)
        .map_err(|e| invalid(e.to_string()))?;
    import_svg(&source)
}
