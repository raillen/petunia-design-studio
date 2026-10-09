//! Shape Builder: merge and extract regions across shapes.
//!
//! Selected shapes combine through boolean chains while every output
//! region remembers which inputs built it. Union merges everything;
//! subtraction carves all other shapes out of one minuend. Curves
//! flatten first; exact topology comes from the boolean kernel.

use crate::geometry::boolean::{boolean_rings, BooleanOp};
use crate::geometry::bounds::point_in_polygon;
use petunia_core::{FillRule, ObjectId, Point};

/// One input shape: identity plus flattened rings.
#[derive(Debug, Clone)]
pub struct BuilderShape {
    pub id: ObjectId,
    pub rings: Vec<Vec<Point>>,
}

/// One output region plus the inputs covering it.
#[derive(Debug, Clone)]
pub struct BuiltRegion {
    pub rings: Vec<Vec<Point>>,
    pub sources: Vec<ObjectId>,
}

/// Region combiner over a working set of shapes.
#[derive(Debug, Clone, Default)]
pub struct ShapeBuilder {
    shapes: Vec<BuilderShape>,
}

impl ShapeBuilder {
    /// Empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self { shapes: Vec::new() }
    }

    /// Add one shape; rings with fewer than three points are ignored.
    pub fn add_shape(&mut self, id: ObjectId, rings: Vec<Vec<Point>>) {
        let rings: Vec<Vec<Point>> = rings.into_iter().filter(|ring| ring.len() >= 3).collect();
        if !rings.is_empty() {
            self.shapes.push(BuilderShape { id, rings });
        }
    }

    /// Number of registered shapes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.shapes.len()
    }

    /// True when nothing is registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }

    /// Merge everything into regions, each tagged with its
    /// contributing inputs.
    #[must_use]
    pub fn build_union(&self) -> Vec<BuiltRegion> {
        let mut flat: Vec<Vec<Point>> = Vec::new();
        for shape in &self.shapes {
            flat.extend(shape.rings.iter().cloned());
        }
        if flat.is_empty() {
            return Vec::new();
        }
        // Union chain: accumulate shapes one by one.
        let mut merged = vec![flat.remove(0)];
        for ring in flat {
            let combined = boolean_rings(&merged, &[ring], BooleanOp::Union, FillRule::NonZero);
            merged = combined.into_iter().flatten().collect();
        }
        regions_with_sources(&merged, &self.shapes)
    }

    /// Carve every other shape out of the minuend.
    #[must_use]
    pub fn subtract(&self, minuend: ObjectId) -> Vec<BuiltRegion> {
        let Some(base) = self.shapes.iter().find(|shape| shape.id == minuend) else {
            return Vec::new();
        };
        let others: Vec<Vec<Point>> = self
            .shapes
            .iter()
            .filter(|shape| shape.id != minuend)
            .flat_map(|shape| shape.rings.iter().cloned())
            .collect();
        if others.is_empty() {
            return regions_with_sources(&base.rings, &self.shapes);
        }
        let carved = boolean_rings(
            &base.rings,
            &others,
            BooleanOp::Difference,
            FillRule::NonZero,
        );
        let flat: Vec<Vec<Point>> = carved.into_iter().flatten().collect();
        regions_with_sources(&flat, &self.shapes)
    }
}

fn regions_with_sources(rings: &[Vec<Point>], shapes: &[BuilderShape]) -> Vec<BuiltRegion> {
    // Group output rings by identical source sets so each region
    // reports one provenance list.
    let mut regions: Vec<BuiltRegion> = Vec::new();
    for ring in rings {
        let sources = sources_covering(ring, shapes);
        if let Some(region) = regions.iter_mut().find(|region| region.sources == sources) {
            region.rings.push(ring.clone());
        } else {
            regions.push(BuiltRegion {
                rings: vec![ring.clone()],
                sources,
            });
        }
    }
    regions
}

/// Inputs covering a ring: any nudged vertex strictly inside it,
/// or any of its nudged vertices strictly inside an input ring.
/// Vertices nudge 1% toward their own centroid so shared boundary
/// points never decide coverage.
fn sources_covering(ring: &[Point], shapes: &[BuilderShape]) -> Vec<ObjectId> {
    let mut sources = Vec::new();
    for shape in shapes {
        let covers = shape.rings.iter().any(|input| {
            let input_center = centroid(input);
            let ring_center = centroid(ring);
            input.iter().any(|vertex| {
                point_in_polygon(nudge(*vertex, input_center), ring, FillRule::NonZero)
            }) || ring.iter().any(|vertex| {
                point_in_polygon(nudge(*vertex, ring_center), input, FillRule::NonZero)
            })
        });
        if covers {
            sources.push(shape.id);
        }
    }
    sources.sort();
    sources
}

fn centroid(ring: &[Point]) -> Point {
    let count = ring.len().max(1) as f64;
    let (sum_x, sum_y) = ring
        .iter()
        .fold((0.0, 0.0), |(x, y), point| (x + point.x, y + point.y));
    Point::new(sum_x / count, sum_y / count)
}

fn nudge(vertex: Point, toward: Point) -> Point {
    Point::new(
        vertex.x + (toward.x - vertex.x) * 0.01,
        vertex.y + (toward.y - vertex.y) * 0.01,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(x: f64, y: f64, size: f64) -> Vec<Point> {
        vec![
            Point::new(x, y),
            Point::new(x + size, y),
            Point::new(x + size, y + size),
            Point::new(x, y + size),
        ]
    }

    #[test]
    fn union_merges_with_provenance() {
        let mut builder = ShapeBuilder::new();
        assert!(builder.is_empty());
        let a = ObjectId::new_v4();
        let b = ObjectId::new_v4();
        builder.add_shape(a, vec![square(0.0, 0.0, 10.0)]);
        builder.add_shape(b, vec![square(20.0, 20.0, 10.0)]);
        assert_eq!(builder.len(), 2);
        let regions = builder.build_union();
        assert_eq!(regions.len(), 2);
        for region in &regions {
            assert_eq!(region.sources.len(), 1);
        }
        // Overlapping pair merges into one region with both sources.
        let mut overlapping = ShapeBuilder::new();
        overlapping.add_shape(a, vec![square(0.0, 0.0, 10.0)]);
        overlapping.add_shape(b, vec![square(5.0, 0.0, 10.0)]);
        let regions = overlapping.build_union();
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].sources.len(), 2);
    }

    #[test]
    fn subtract_carves_other_shapes_out() {
        let mut builder = ShapeBuilder::new();
        let base = ObjectId::new_v4();
        let cutter = ObjectId::new_v4();
        builder.add_shape(base, vec![square(0.0, 0.0, 10.0)]);
        builder.add_shape(cutter, vec![square(5.0, 0.0, 10.0)]);
        let regions = builder.subtract(base);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].sources, vec![base]);
        // Unknown minuend yields nothing, never a panic.
        assert!(builder.subtract(ObjectId::new_v4()).is_empty());
    }

    #[test]
    fn degenerate_rings_are_ignored() {
        let mut builder = ShapeBuilder::new();
        builder.add_shape(ObjectId::new_v4(), vec![vec![Point::new(0.0, 0.0)]]);
        assert!(builder.is_empty());
        assert!(builder.build_union().is_empty());
    }
}
