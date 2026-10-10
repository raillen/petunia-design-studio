//! Derived parametric shape evaluation. The authorial spec stays intact.
use petunia_core::{
    Contour, EllipseArc, NodeKind, ParametricShape, PathNode, Point, Tolerance, VectorPath,
};

pub fn shape_path(shape: ParametricShape, tolerance: Tolerance) -> VectorPath {
    let mut points = Vec::new();
    let mut closed = true;
    match shape {
        ParametricShape::Rectangle(rect) => {
            let w = rect.size.width;
            let h = rect.size.height;
            // CSS-like common scale preserves the corner proportions.
            let radii = [
                rect.corners.top_left,
                rect.corners.top_right,
                rect.corners.bottom_right,
                rect.corners.bottom_left,
            ];
            let scale = [
                w / (radii[0].dx + radii[1].dx).max(w),
                w / (radii[3].dx + radii[2].dx).max(w),
                h / (radii[0].dy + radii[3].dy).max(h),
                h / (radii[1].dy + radii[2].dy).max(h),
            ]
            .into_iter()
            .fold(1.0_f64, f64::min);
            for (i, radius) in radii.into_iter().enumerate() {
                let rx = radius.dx * scale;
                let ry = radius.dy * scale;
                let (cx, cy) = match i {
                    0 => (rx, ry),
                    1 => (w - rx, ry),
                    2 => (w - rx, h - ry),
                    _ => (rx, h - ry),
                };
                arc_points(
                    &mut points,
                    (cx, cy),
                    (rx, ry),
                    std::f64::consts::PI + i as f64 * std::f64::consts::FRAC_PI_2,
                    std::f64::consts::FRAC_PI_2,
                    tolerance,
                );
            }
        }
        ParametricShape::Ellipse(ellipse) => {
            let (start, sweep) = match ellipse.arc {
                EllipseArc::Full => (0.0, std::f64::consts::TAU),
                EllipseArc::Open { start, sweep } => {
                    closed = false;
                    (start.radians(), sweep.radians())
                }
                EllipseArc::Chord { start, sweep } | EllipseArc::Pie { start, sweep } => {
                    (start.radians(), sweep.radians())
                }
            };
            arc_points(
                &mut points,
                (0.0, 0.0),
                (ellipse.radii.dx, ellipse.radii.dy),
                start,
                sweep,
                tolerance,
            );
            if matches!(ellipse.arc, EllipseArc::Pie { .. }) {
                points.push(Point::new(0.0, 0.0));
            }
            if matches!(ellipse.arc, EllipseArc::Full) {
                points.pop();
            }
        }
        ParametricShape::Polygon(polygon) => {
            for i in 0..polygon.sides {
                let angle = polygon.rotation.radians()
                    + i as f64 * std::f64::consts::TAU / polygon.sides as f64;
                points.push(Point::new(
                    polygon.radius * angle.cos(),
                    polygon.radius * angle.sin(),
                ));
            }
        }
        ParametricShape::Star(star) => {
            for i in 0..star.points * 2 {
                let angle =
                    star.rotation.radians() + i as f64 * std::f64::consts::PI / star.points as f64;
                let radius = star.outer_radius * if i % 2 == 0 { 1.0 } else { star.inner_ratio };
                points.push(Point::new(radius * angle.cos(), radius * angle.sin()));
            }
        }
    }
    points.dedup();
    let mut contour = Contour::new(closed);
    for point in points {
        contour.push_node(PathNode::line(point, NodeKind::Cusp));
    }
    let mut path = VectorPath::new();
    path.push_contour(contour);
    path
}

fn arc_points(
    points: &mut Vec<Point>,
    center: (f64, f64),
    radii: (f64, f64),
    start: f64,
    sweep: f64,
    tolerance: Tolerance,
) {
    let radius = radii.0.max(radii.1);
    let delta = if radius > 0.0 {
        (1.0 - tolerance.0 / radius).clamp(-1.0, 1.0).acos() * 2.0
    } else {
        std::f64::consts::FRAC_PI_2
    };
    let count = (sweep.abs() / delta.max(0.001)).ceil().clamp(1.0, 65536.0) as usize;
    for i in 0..=count {
        let angle = start + sweep * i as f64 / count as f64;
        points.push(Point::new(
            center.0 + radii.0 * angle.cos(),
            center.1 + radii.1 * angle.sin(),
        ));
    }
}

/// Evaluated stroke outline in local space, preserving authorial geometry.
pub fn stroke_outline(
    path: &VectorPath,
    style: &petunia_core::StrokeStyle,
    tolerance: Tolerance,
) -> petunia_render_model::RenderPath {
    use crate::geometry::bezier::flatten_contour;
    use petunia_core::StrokeAlignment;
    let mut result = petunia_render_model::RenderPath::new();
    for contour in &path.contours {
        let mut flat = flatten_contour(contour, tolerance);
        flat.dedup();
        if flat.len() > 1 && flat.first() == flat.last() {
            flat.pop();
        }
        if flat.len() < 2 {
            continue;
        }
        let closed = contour.closed;
        let width = style.width
            * if closed && style.alignment != StrokeAlignment::Center {
                2.0
            } else {
                1.0
            };
        let runs = dashed_runs(&flat, closed, &style.dash);
        let total: f64 = flat
            .windows(2)
            .map(|p| (p[1].x - p[0].x).hypot(p[1].y - p[0].y))
            .sum::<f64>()
            + if closed {
                (flat[0].x - flat[flat.len() - 1].x).hypot(flat[0].y - flat[flat.len() - 1].y)
            } else {
                0.0
            };
        let mut rings = Vec::new();
        for (run, run_closed) in runs {
            outline_run(&run, run_closed, width, style, total, tolerance, &mut rings);
        }
        if closed && style.alignment != StrokeAlignment::Center {
            use crate::geometry::boolean::{boolean_rings, BooleanOp};
            let operation = if style.alignment == StrokeAlignment::Inside {
                BooleanOp::Intersection
            } else {
                BooleanOp::Difference
            };
            rings = boolean_rings(&rings, &[flat], operation, petunia_core::FillRule::NonZero)
                .into_iter()
                .flatten()
                .collect();
        }
        for ring in rings {
            result.push_contour(ring.into_iter().map(|p| (p.x, p.y)).collect(), true);
        }
    }
    result
}

fn dashed_runs(
    points: &[Point],
    closed: bool,
    dash: &petunia_core::DashPattern,
) -> Vec<(Vec<(Point, f64)>, bool)> {
    let mut vertices = points.to_vec();
    if closed {
        vertices.push(vertices[0]);
    }
    if dash.lengths.is_empty() {
        let mut distance = 0.0;
        let mut run = vec![(vertices[0], 0.0)];
        for pair in vertices.windows(2) {
            distance += (pair[1].x - pair[0].x).hypot(pair[1].y - pair[0].y);
            run.push((pair[1], distance));
        }
        return vec![(run, closed)];
    }
    let mut index = 0;
    let mut offset = dash.offset;
    while offset >= dash.lengths[index] {
        offset -= dash.lengths[index];
        index = (index + 1) % dash.lengths.len();
    }
    let mut remaining = dash.lengths[index] - offset;
    let mut result = Vec::new();
    let mut run = Vec::new();
    let mut distance = 0.0;
    for pair in vertices.windows(2) {
        let length = (pair[1].x - pair[0].x).hypot(pair[1].y - pair[0].y);
        if length == 0.0 {
            continue;
        }
        let mut used = 0.0;
        while used < length {
            if remaining <= 1e-12 {
                if run.len() > 1 {
                    result.push((std::mem::take(&mut run), false));
                }
                index = (index + 1) % dash.lengths.len();
                remaining = dash.lengths[index];
                continue;
            }
            let take = remaining.min(length - used);
            let interpolate = |t: f64| {
                Point::new(
                    pair[0].x + (pair[1].x - pair[0].x) * t / length,
                    pair[0].y + (pair[1].y - pair[0].y) * t / length,
                )
            };
            if index % 2 == 0 {
                if run.is_empty() {
                    run.push((interpolate(used), distance + used));
                }
                run.push((interpolate(used + take), distance + used + take));
            }
            used += take;
            remaining -= take;
        }
        distance += length;
    }
    if run.len() > 1 {
        result.push((run, false));
    }
    result
}

fn profile_scale(style: &petunia_core::StrokeStyle, distance: f64, total: f64) -> f64 {
    let Some(profile) = &style.variable_width else {
        return 1.0;
    };
    let t = (distance / total.max(1e-12)) as f32;
    let Some(first) = profile.points.first() else {
        return 1.0;
    };
    if t <= first.position {
        return f64::from(first.scale);
    }
    for pair in profile.points.windows(2) {
        if t <= pair[1].position {
            let fraction = (t - pair[0].position) / (pair[1].position - pair[0].position);
            return f64::from(pair[0].scale + (pair[1].scale - pair[0].scale) * fraction);
        }
    }
    profile
        .points
        .last()
        .map_or(1.0, |point| f64::from(point.scale))
}

fn outline_run(
    run: &[(Point, f64)],
    closed: bool,
    width: f64,
    style: &petunia_core::StrokeStyle,
    total: f64,
    tolerance: Tolerance,
    rings: &mut Vec<Vec<Point>>,
) {
    use petunia_core::{StrokeCap, StrokeJoin};
    let mut normals = Vec::new();
    for pair in run.windows(2) {
        let dx = pair[1].0.x - pair[0].0.x;
        let dy = pair[1].0.y - pair[0].0.y;
        let length = dx.hypot(dy);
        normals.push(if length > 0.0 {
            (-dy / length, dx / length)
        } else {
            (0.0, 0.0)
        });
    }
    let offset = |point: Point, n: (f64, f64), distance: f64| {
        Point::new(point.x + n.0 * distance, point.y + n.1 * distance)
    };
    for (i, pair) in run.windows(2).enumerate() {
        let n = normals[i];
        let h0 = width * profile_scale(style, pair[0].1, total) * 0.5;
        let h1 = width * profile_scale(style, pair[1].1, total) * 0.5;
        add_ring(
            rings,
            vec![
                offset(pair[0].0, n, h0),
                offset(pair[1].0, n, h1),
                offset(pair[1].0, n, -h1),
                offset(pair[0].0, n, -h0),
            ],
        );
    }
    let last = run.len() - 1;
    for i in 0..last {
        if i == 0 && !closed {
            continue;
        }
        let previous = if i == 0 {
            normals[last - 1]
        } else {
            normals[i - 1]
        };
        let next = normals[i];
        let center = run[i].0;
        let half = width * profile_scale(style, run[i].1, total) * 0.5;
        if style.join == StrokeJoin::Round {
            add_circle(rings, center, half, tolerance);
            continue;
        }
        for side in [-1.0, 1.0] {
            let a = offset(center, previous, half * side);
            let b = offset(center, next, half * side);
            let denominator = 1.0 + previous.0 * next.0 + previous.1 * next.1;
            let mut join = vec![center, a];
            if style.join == StrokeJoin::Miter && denominator > 1e-12 {
                let miter = offset(
                    center,
                    (previous.0 + next.0, previous.1 + next.1),
                    half * side / denominator,
                );
                if (miter.x - center.x).hypot(miter.y - center.y) <= half * style.miter_limit {
                    join.push(miter);
                }
            }
            join.push(b);
            add_ring(rings, join);
        }
    }
    if !closed {
        for (i, n, sign) in [(0, normals[0], -1.0), (last, normals[last - 1], 1.0)] {
            let center = run[i].0;
            let half = width * profile_scale(style, run[i].1, total) * 0.5;
            match style.cap {
                StrokeCap::Butt => {}
                StrokeCap::Round => add_circle(rings, center, half, tolerance),
                StrokeCap::Square => {
                    let end = offset(center, (n.1, -n.0), half * sign);
                    add_ring(
                        rings,
                        vec![
                            offset(center, n, half),
                            offset(end, n, half),
                            offset(end, n, -half),
                            offset(center, n, -half),
                        ],
                    );
                }
            }
        }
    }
}
fn add_circle(rings: &mut Vec<Vec<Point>>, center: Point, radius: f64, tolerance: Tolerance) {
    let mut points = Vec::new();
    arc_points(
        &mut points,
        (center.x, center.y),
        (radius, radius),
        0.0,
        std::f64::consts::TAU,
        tolerance,
    );
    points.pop();
    add_ring(rings, points);
}
fn add_ring(rings: &mut Vec<Vec<Point>>, mut points: Vec<Point>) {
    let area: f64 = points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .map(|(a, b)| a.x * b.y - a.y * b.x)
        .sum();
    if area < 0.0 {
        points.reverse();
    }
    if area != 0.0 {
        rings.push(points);
    }
}
