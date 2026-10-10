//! Live trace evaluation over immutable color-managed image snapshots.
//!
//! Labels use eight-neighbor connectivity. Marching Squares extracts subpixel
//! boundaries, resolving diagonal ambiguity in favor of that connectivity.
//! Compound EvenOdd paths retain holes. Geometry fitting reuses the Engine's
//! Schneider fitter; no authorial IDs or source pixels are changed.

use crate::analysis::{
    check_cancel, decode_srgb, distance, srgb_color, srgb_to_oklab, AnalysisError, AnalysisLimits,
    ColorQuantizer, PaletteAlphaPolicy, RgbaSnapshot, ANALYSIS_SEMANTIC_VERSION,
};
use crate::generated::stabilize_path;
use crate::geometry::{fit_cubics, point_in_polygon, simplify_closed};
use crate::jobs::CancelToken;
use petunia_core::color::{BuiltinColorSpace, ColorSpaceRef};
use petunia_core::generated::{AnalysisAlphaPolicy, ImageTraceSpec, TraceMode};
use petunia_core::{ColorValue, Contour, FillRule, NodeKind, PathNode, Point, Vec2, VectorPath};
use std::collections::{BTreeMap, VecDeque};

/// Versioned background matching policy in Euclidean Oklab distance.
pub const BACKGROUND_MATCH_DISTANCE: f64 = 0.01;

#[derive(Debug, Clone, PartialEq)]
pub struct TracedPath {
    pub path: VectorPath,
    pub color: ColorValue,
}

#[derive(Debug)]
struct Region {
    id: u32,
    cluster: u16,
    pixels: usize,
    bounds: [usize; 4], // inclusive left, top, right, bottom
    first: usize,
}

/// Corner sensitivity 0 preserves only reversals; 1 protects every turn.
/// The semantic-version constant covers this mapping for preview and expansion.
#[must_use]
pub fn corner_angle_threshold(sensitivity: f32) -> f64 {
    std::f64::consts::PI * (1.0 - f64::from(sensitivity.clamp(0.0, 1.0)))
}

/// Complete result or typed failure. Never returns partially evaluated geometry.
pub fn evaluate_trace(
    image: &RgbaSnapshot,
    spec: &ImageTraceSpec,
    limits: &AnalysisLimits,
    token: &CancelToken,
) -> Result<Vec<TracedPath>, AnalysisError> {
    check_cancel(token)?;
    spec.validate()
        .map_err(|error| AnalysisError::InvalidInput(error.to_string()))?;
    let colors = match spec.mode {
        TraceMode::Monochrome => 1,
        TraceMode::Grayscale { levels } => levels,
        TraceMode::Color { colors } => colors,
    };
    limits.validate(image, usize::from(colors))?;
    let background = spec.ignore_background.as_ref().map(color_lab).transpose()?;
    let (palette, labels) = classify_pixels(image, spec, limits, token)?;
    let ignored: Vec<bool> = palette
        .iter()
        .map(|color| {
            background.is_some_and(|reference| {
                color_lab(color)
                    .is_ok_and(|lab| distance(reference, lab) <= BACKGROUND_MATCH_DISTANCE.powi(2))
            })
        })
        .collect();
    let (components, mut regions) = connected_regions(
        &labels,
        image.width(),
        image.height(),
        &ignored,
        limits,
        token,
    )?;
    regions.retain(|region| region.pixels as f64 >= spec.min_region_area_px);
    let source_hash = blake3::hash(image.pixels());
    let spec_bytes =
        serde_json::to_vec(spec).map_err(|error| AnalysisError::InvalidInput(error.to_string()))?;
    let mut results = Vec::with_capacity(regions.len());
    let mut total_vertices = 0;
    let mut scanned_cells = 0usize;
    let mut all_rings = Vec::with_capacity(regions.len());
    for region in &regions {
        check_cancel(token)?;
        let rings = marching_squares(
            &components,
            image.width(),
            image.height(),
            region,
            limits,
            token,
            &mut total_vertices,
            &mut scanned_cells,
        )?;
        all_rings.push(rings);
    }
    // Nesting precedes cluster/area/tie ordering. Region order never depends on a
    // hash map or worker scheduling; an island in a hole paints after its parent.
    let depths: Vec<usize> = all_rings
        .iter()
        .enumerate()
        .map(|(index, rings)| {
            let Some(point) = rings.first().and_then(|ring| ring.first()) else {
                return 0;
            };
            all_rings
                .iter()
                .enumerate()
                .filter(|(other, rings)| {
                    *other != index
                        && rings
                            .iter()
                            .filter(|ring| point_in_polygon(*point, ring, FillRule::EvenOdd))
                            .count()
                            % 2
                            == 1
                })
                .count()
        })
        .collect();
    let mut order: Vec<usize> = (0..regions.len()).collect();
    order.sort_by(|&a, &b| {
        depths[a]
            .cmp(&depths[b])
            .then_with(|| regions[a].cluster.cmp(&regions[b].cluster))
            .then_with(|| regions[b].pixels.cmp(&regions[a].pixels))
            .then_with(|| regions[a].first.cmp(&regions[b].first))
    });
    let mut output_nodes = 0usize;
    for index in order {
        check_cancel(token)?;
        let region = &regions[index];
        let mut path = VectorPath {
            contours: Vec::new(),
            fill_rule: FillRule::EvenOdd,
        };
        for ring in &all_rings[index] {
            let contour = fit_ring(ring, spec, token)?;
            output_nodes = output_nodes.saturating_add(contour.nodes.len());
            if output_nodes > limits.max_vertices {
                return Err(AnalysisError::LimitExceeded("output node count"));
            }
            path.contours.push(contour);
        }
        let mut seed = Vec::with_capacity(spec_bytes.len() + 64);
        seed.extend_from_slice(source_hash.as_bytes());
        seed.extend_from_slice(&spec_bytes);
        seed.extend_from_slice(ANALYSIS_SEMANTIC_VERSION.as_bytes());
        seed.extend_from_slice(&(image.width() as u64).to_le_bytes());
        seed.extend_from_slice(&(image.height() as u64).to_le_bytes());
        seed.extend_from_slice(&(region.first as u64).to_le_bytes());
        stabilize_path(&mut path, &seed);
        results.push(TracedPath {
            path,
            color: palette[usize::from(region.cluster)].clone(),
        });
    }
    check_cancel(token)?;
    Ok(results)
}

fn classify_pixels(
    image: &RgbaSnapshot,
    spec: &ImageTraceSpec,
    limits: &AnalysisLimits,
    token: &CancelToken,
) -> Result<(Vec<ColorValue>, Vec<Option<u16>>), AnalysisError> {
    if let TraceMode::Color { colors } = spec.mode {
        let alpha = match spec.alpha_policy {
            AnalysisAlphaPolicy::Ignore => PaletteAlphaPolicy::IgnoreAlpha,
            AnalysisAlphaPolicy::UseAsMask => PaletteAlphaPolicy::WeightByAlpha,
        };
        let quantized = ColorQuantizer.quantize(image, colors, alpha, limits, token)?;
        return Ok((
            quantized
                .palette
                .into_iter()
                .map(|entry| entry.color)
                .collect(),
            quantized.assignments,
        ));
    }
    let levels = match spec.mode {
        TraceMode::Grayscale { levels } => levels,
        _ => 1,
    };
    let palette = if spec.mode == TraceMode::Monochrome {
        vec![srgb_color([0.0; 3])]
    } else {
        (0..levels)
            .map(|index| {
                let linear = if levels == 1 {
                    0.5
                } else {
                    f64::from(index) / f64::from(levels - 1)
                };
                let encoded = if linear <= 0.0031308 {
                    12.92 * linear
                } else {
                    1.055 * linear.powf(1.0 / 2.4) - 0.055
                };
                srgb_color([encoded as f32; 3])
            })
            .collect()
    };
    let mut labels = Vec::with_capacity(image.width() * image.height());
    for index in 0..image.width() * image.height() {
        if index % 4096 == 0 {
            check_cancel(token)?;
        }
        let pixel = image.pixel(index);
        let alpha = f64::from(pixel[3]) / 255.0;
        if spec.alpha_policy == AnalysisAlphaPolicy::UseAsMask && alpha <= 0.0 {
            labels.push(None);
            continue;
        }
        let luminance = 0.2126 * decode_srgb(f64::from(pixel[0]) / 255.0)
            + 0.7152 * decode_srgb(f64::from(pixel[1]) / 255.0)
            + 0.0722 * decode_srgb(f64::from(pixel[2]) / 255.0);
        if spec.mode == TraceMode::Monochrome {
            let coverage = if spec.alpha_policy == AnalysisAlphaPolicy::UseAsMask {
                1.0 - alpha * (1.0 - luminance)
            } else {
                luminance
            };
            labels.push((coverage < f64::from(spec.threshold)).then_some(0));
        } else {
            labels.push(Some((luminance * f64::from(levels - 1)).round() as u16));
        }
    }
    Ok((palette, labels))
}

fn color_lab(color: &ColorValue) -> Result<[f64; 3], AnalysisError> {
    match color {
        ColorValue::Process(process)
            if process.space == ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb) =>
        {
            let rgb = process
                .as_rgb()
                .ok_or(AnalysisError::UnsupportedColorContext)?;
            process
                .validate()
                .map_err(|error| AnalysisError::InvalidInput(error.to_string()))?;
            Ok(srgb_to_oklab([
                f64::from(rgb.r),
                f64::from(rgb.g),
                f64::from(rgb.b),
            ]))
        }
        _ => Err(AnalysisError::UnsupportedColorContext),
    }
}

fn connected_regions(
    labels: &[Option<u16>],
    width: usize,
    height: usize,
    ignored: &[bool],
    limits: &AnalysisLimits,
    token: &CancelToken,
) -> Result<(Vec<u32>, Vec<Region>), AnalysisError> {
    let mut map = vec![0u32; labels.len()];
    let mut regions = Vec::new();
    let mut queue = VecDeque::new();
    for first in 0..labels.len() {
        if first % 4096 == 0 {
            check_cancel(token)?;
        }
        let Some(cluster) = labels[first] else {
            continue;
        };
        if map[first] != 0 || ignored[usize::from(cluster)] {
            continue;
        }
        if regions.len() >= limits.max_regions || regions.len() >= u32::MAX as usize - 1 {
            return Err(AnalysisError::LimitExceeded("connected region count"));
        }
        let id = regions.len() as u32 + 1;
        let x = first % width;
        let y = first / width;
        let mut region = Region {
            id,
            cluster,
            pixels: 0,
            bounds: [x, y, x, y],
            first,
        };
        map[first] = id;
        queue.push_back(first);
        while let Some(index) = queue.pop_front() {
            region.pixels += 1;
            if region.pixels.is_multiple_of(1024) {
                check_cancel(token)?;
            }
            let x = index % width;
            let y = index / width;
            region.bounds[0] = region.bounds[0].min(x);
            region.bounds[1] = region.bounds[1].min(y);
            region.bounds[2] = region.bounds[2].max(x);
            region.bounds[3] = region.bounds[3].max(y);
            for ny in y.saturating_sub(1)..=(y + 1).min(height - 1) {
                for nx in x.saturating_sub(1)..=(x + 1).min(width - 1) {
                    let adjacent = ny * width + nx;
                    if map[adjacent] == 0 && labels[adjacent] == Some(cluster) {
                        map[adjacent] = id;
                        queue.push_back(adjacent);
                    }
                }
            }
        }
        regions.push(region);
    }
    Ok((map, regions))
}

type Vertex = (i64, i64); // doubled coordinates avoid floating-point hash keys
type Adjacency = BTreeMap<Vertex, Vec<Vertex>>;

#[allow(clippy::too_many_arguments)]
fn marching_squares(
    map: &[u32],
    width: usize,
    height: usize,
    region: &Region,
    limits: &AnalysisLimits,
    token: &CancelToken,
    total_vertices: &mut usize,
    scanned_cells: &mut usize,
) -> Result<Vec<Vec<Point>>, AnalysisError> {
    let [left, top, right, bottom] = region.bounds;
    let mut adjacency = Adjacency::new();
    let inside = |x: i64, y: i64| -> bool {
        x >= 0
            && y >= 0
            && (x as usize) < width
            && (y as usize) < height
            && map[y as usize * width + x as usize] == region.id
    };
    for y in top as i64 - 1..=bottom as i64 {
        check_cancel(token)?;
        for x in left as i64 - 1..=right as i64 {
            *scanned_cells = scanned_cells.saturating_add(1);
            if *scanned_cells > limits.max_pixels.saturating_mul(32) {
                return Err(AnalysisError::LimitExceeded("contour cell work budget"));
            }
            let case = u8::from(inside(x, y))
                | (u8::from(inside(x + 1, y)) << 1)
                | (u8::from(inside(x + 1, y + 1)) << 2)
                | (u8::from(inside(x, y + 1)) << 3);
            let points = [
                (2 * x + 2, 2 * y + 1),
                (2 * x + 3, 2 * y + 2),
                (2 * x + 2, 2 * y + 3),
                (2 * x + 1, 2 * y + 2),
            ];
            // edge 0=top, 1=right, 2=bottom, 3=left. Cases 5/10 keep
            // diagonal foreground connected, implementing the eight-neighbor policy.
            let segments: &[(usize, usize)] = match case {
                0 | 15 => &[],
                1 | 14 => &[(3, 0)],
                2 | 13 => &[(0, 1)],
                3 | 12 => &[(3, 1)],
                4 | 11 => &[(1, 2)],
                5 => &[(0, 1), (2, 3)],
                6 | 9 => &[(0, 2)],
                7 | 8 => &[(2, 3)],
                10 => &[(3, 0), (1, 2)],
                _ => &[],
            };
            for &(a, b) in segments {
                for (a, b) in [(points[a], points[b]), (points[b], points[a])] {
                    if !adjacency.contains_key(&a) {
                        *total_vertices = total_vertices.saturating_add(1);
                        if *total_vertices > limits.max_vertices {
                            return Err(AnalysisError::LimitExceeded("contour vertex count"));
                        }
                        let bytes = adjacency
                            .len()
                            .saturating_add(1)
                            .saturating_mul(256)
                            .saturating_add(map.len().saturating_mul(32))
                            .saturating_add(total_vertices.saturating_mul(16));
                        if bytes > limits.max_temporary_bytes {
                            return Err(AnalysisError::LimitExceeded("boundary allocation budget"));
                        }
                    }
                    let neighbors = adjacency.entry(a).or_default();
                    if neighbors.len() >= 2 {
                        return Err(AnalysisError::InvalidBoundary);
                    }
                    neighbors.push(b);
                }
            }
        }
    }
    let mut rings = Vec::new();
    while let Some((&start, _)) = adjacency.first_key_value() {
        check_cancel(token)?;
        let mut ring = Vec::new();
        let mut previous = None;
        let mut current = start;
        loop {
            if ring.len() % 1024 == 0 {
                check_cancel(token)?;
            }
            let neighbors = adjacency
                .remove(&current)
                .ok_or(AnalysisError::InvalidBoundary)?;
            if neighbors.len() != 2 {
                return Err(AnalysisError::InvalidBoundary);
            }
            let next = if previous == Some(neighbors[0]) {
                neighbors[1]
            } else {
                neighbors[0]
            };
            ring.push(Point::new(current.0 as f64 / 2.0, current.1 as f64 / 2.0));
            if next == start {
                break;
            }
            previous = Some(current);
            current = next;
        }
        if ring.len() < 3 {
            return Err(AnalysisError::InvalidBoundary);
        }
        rings.push(ring);
    }
    rings.sort_by(|a, b| {
        signed_area(b)
            .abs()
            .total_cmp(&signed_area(a).abs())
            .then_with(|| a[0].y.total_cmp(&b[0].y))
            .then_with(|| a[0].x.total_cmp(&b[0].x))
    });
    // Explicit Y-down orientation: exterior positive (clockwise), holes negative.
    for index in 0..rings.len() {
        let depth = rings
            .iter()
            .enumerate()
            .filter(|(other, ring)| {
                *other != index && point_in_polygon(rings[index][0], ring, FillRule::EvenOdd)
            })
            .count();
        if (signed_area(&rings[index]) > 0.0) != (depth % 2 == 0) {
            rings[index].reverse();
        }
        canonical_rotate(&mut rings[index]);
    }
    Ok(rings)
}

fn signed_area(ring: &[Point]) -> f64 {
    ring.iter()
        .zip(ring.iter().cycle().skip(1))
        .take(ring.len())
        .map(|(a, b)| a.x * b.y - b.x * a.y)
        .sum::<f64>()
        / 2.0
}
fn canonical_rotate(ring: &mut [Point]) {
    let index = ring
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.y.total_cmp(&b.y).then_with(|| a.x.total_cmp(&b.x)))
        .map_or(0, |(index, _)| index);
    ring.rotate_left(index);
}

fn fit_ring(
    ring: &[Point],
    spec: &ImageTraceSpec,
    token: &CancelToken,
) -> Result<Contour, AnalysisError> {
    check_cancel(token)?;
    let tolerance = spec.curve_tolerance;
    // Keep each reusable fitter call bounded; raw closed contours larger than
    // this are valid input but require a less detailed trace specification.
    if ring.len() > 16_384 {
        return Err(AnalysisError::LimitExceeded("curve-fit contour size"));
    }
    let mut simplified =
        simplify_closed(ring, tolerance * f64::from(spec.smoothing).min(1.0) * 0.5);
    if simplified.len() < 3 {
        simplified = ring.to_vec();
    }
    canonical_rotate(&mut simplified);
    let count = simplified.len();
    let threshold = corner_angle_threshold(spec.corner_sensitivity);
    let mut corners = vec![0];
    for index in 1..count {
        let before = simplified[(index + count - 1) % count];
        let point = simplified[index];
        let after = simplified[(index + 1) % count];
        let a = Vec2::new(point.x - before.x, point.y - before.y);
        let b = Vec2::new(after.x - point.x, after.y - point.y);
        let dot = (a.dx * b.dx + a.dy * b.dy) / (a.length() * b.length()).max(1e-15);
        if dot.clamp(-1.0, 1.0).acos() >= threshold {
            corners.push(index);
        }
    }
    if corners.len() == 1 {
        corners.push(count / 2);
    }
    let mut contour = Contour::new(true);
    if tolerance == 0.0 || spec.smoothing == 0.0 {
        contour.nodes = simplified
            .into_iter()
            .map(|point| PathNode::line(point, NodeKind::Cusp))
            .collect();
        return Ok(contour);
    }
    let mut curves = Vec::new();
    for position in 0..corners.len() {
        check_cancel(token)?;
        let start = corners[position];
        let end = corners[(position + 1) % corners.len()];
        let end = if end <= start { end + count } else { end };
        let section: Vec<_> = (start..=end)
            .map(|index| simplified[index % count])
            .collect();
        let tangent = |a: Point, b: Point| {
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let length = dx.hypot(dy).max(1e-15);
            Vec2::new(dx / length, dy / length)
        };
        curves.extend(fit_cubics(
            &section,
            tangent(section[0], section[1]),
            tangent(section[section.len() - 2], section[section.len() - 1]),
            tolerance * 0.5,
        ));
    }
    if curves.len() < 3 {
        // A two-anchor closed cubic loop is valid, but tiny rings are better
        // represented by their original topology-preserving polygon.
        contour.nodes = simplified
            .into_iter()
            .map(|point| PathNode::line(point, NodeKind::Cusp))
            .collect();
    } else {
        for (index, curve) in curves.iter().enumerate() {
            contour.nodes.push(PathNode::with_handles(
                curve.p0,
                Some(curves[(index + curves.len() - 1) % curves.len()].p2),
                Some(curve.p1),
                NodeKind::Cusp,
            ));
        }
    }
    Ok(contour)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn mono(width: usize, height: usize, mask: &[bool]) -> RgbaSnapshot {
        RgbaSnapshot::new(
            width,
            height,
            mask.iter()
                .flat_map(|dark| {
                    if *dark {
                        [0, 0, 0, 255]
                    } else {
                        [255, 255, 255, 255]
                    }
                })
                .collect(),
        )
        .unwrap()
    }
    fn polygon_spec() -> ImageTraceSpec {
        ImageTraceSpec {
            min_region_area_px: 0.0,
            smoothing: 0.0,
            curve_tolerance: 0.0,
            ..Default::default()
        }
    }
    fn contains(path: &VectorPath, point: Point) -> bool {
        path.contours
            .iter()
            .filter(|contour| {
                point_in_polygon(
                    point,
                    &contour
                        .nodes
                        .iter()
                        .map(|node| node.point)
                        .collect::<Vec<_>>(),
                    FillRule::EvenOdd,
                )
            })
            .count()
            % 2
            == 1
    }
    #[test]
    fn donut_retains_hole_and_source_is_immutable_and_ids_deterministic() {
        let mask: Vec<_> = (0..49)
            .map(|index| {
                let x = index % 7;
                let y = index / 7;
                x > 0 && y > 0 && x < 6 && y < 6 && !(x > 1 && x < 5 && y > 1 && y < 5)
            })
            .collect();
        let image = mono(7, 7, &mask);
        let source = image.clone();
        let token = CancelToken::new();
        let result =
            evaluate_trace(&image, &polygon_spec(), &AnalysisLimits::default(), &token).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path.contours.len(), 2);
        assert!(!contains(&result[0].path, Point::new(3.5, 3.5)));
        assert!(contains(&result[0].path, Point::new(1.5, 3.5)));
        assert_eq!(
            result,
            evaluate_trace(&image, &polygon_spec(), &AnalysisLimits::default(), &token).unwrap()
        );
        assert_eq!(image, source);
        let exterior: Vec<_> = result[0].path.contours[0]
            .nodes
            .iter()
            .map(|node| node.point)
            .collect();
        let hole: Vec<_> = result[0].path.contours[1]
            .nodes
            .iter()
            .map(|node| node.point)
            .collect();
        assert!(signed_area(&exterior) > 0.0);
        assert!(signed_area(&hole) < 0.0);
    }
    #[test]
    fn diagonal_ambiguity_is_eight_connected_and_minimum_area_is_explicit() {
        let image = mono(2, 2, &[true, false, false, true]);
        let token = CancelToken::new();
        let mut spec = polygon_spec();
        let result = evaluate_trace(&image, &spec, &AnalysisLimits::default(), &token).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path.contours.len(), 1);
        spec.min_region_area_px = 3.0;
        assert!(
            evaluate_trace(&image, &spec, &AnalysisLimits::default(), &token)
                .unwrap()
                .is_empty()
        );
    }
    #[test]
    fn color_gray_transparency_and_background_policy() {
        let image =
            RgbaSnapshot::new(3, 1, vec![255, 0, 0, 255, 0, 0, 255, 255, 0, 0, 0, 0]).unwrap();
        let mut spec = polygon_spec();
        spec.mode = TraceMode::Color { colors: 3 };
        spec.alpha_policy = AnalysisAlphaPolicy::UseAsMask;
        let result = evaluate_trace(
            &image,
            &spec,
            &AnalysisLimits::default(),
            &CancelToken::new(),
        )
        .unwrap();
        assert_eq!(result.len(), 2);
        spec.ignore_background = Some(srgb_color([1.0, 0.0, 0.0]));
        assert_eq!(
            evaluate_trace(
                &image,
                &spec,
                &AnalysisLimits::default(),
                &CancelToken::new()
            )
            .unwrap()
            .len(),
            1
        );
        spec.ignore_background = None;
        spec.mode = TraceMode::Grayscale { levels: 3 };
        let gray = RgbaSnapshot::new(
            3,
            1,
            vec![0, 0, 0, 255, 188, 188, 188, 255, 255, 255, 255, 255],
        )
        .unwrap();
        assert_eq!(
            evaluate_trace(
                &gray,
                &spec,
                &AnalysisLimits::default(),
                &CancelToken::new()
            )
            .unwrap()
            .len(),
            3
        );
    }
    #[test]
    fn linear_luminance_not_encoded_average_and_alpha_coverage() {
        let image = RgbaSnapshot::new(2, 1, vec![255, 0, 0, 255, 0, 0, 0, 64]).unwrap();
        let mut spec = polygon_spec();
        spec.threshold = 0.3;
        spec.alpha_policy = AnalysisAlphaPolicy::UseAsMask;
        let result = evaluate_trace(
            &image,
            &spec,
            &AnalysisLimits::default(),
            &CancelToken::new(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert!(contains(&result[0].path, Point::new(0.5, 0.5)));
        assert!(!contains(&result[0].path, Point::new(1.5, 0.5)));
    }
    #[test]
    fn cancel_and_complexity_failure_produce_no_partial_paths() {
        let image = mono(2, 2, &[true, false, false, true]);
        let token = CancelToken::new();
        token.cancel();
        assert_eq!(
            evaluate_trace(&image, &polygon_spec(), &AnalysisLimits::default(), &token),
            Err(AnalysisError::Cancelled)
        );
        let limits = AnalysisLimits {
            max_vertices: 1,
            ..AnalysisLimits::default()
        };
        assert!(matches!(
            evaluate_trace(&image, &polygon_spec(), &limits, &CancelToken::new()),
            Err(AnalysisError::LimitExceeded(_))
        ));
    }
}
