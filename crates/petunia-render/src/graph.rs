//! Render graph: culled, binned passes over one snapshot.
//!
//! The graph compiles passes (prepare, cull, bin, rasterize,
//! effects, composite, output, overlays) from a snapshot and a view.
//! Tiles rasterize independently and merge in paint order; group
//! isolation gets intermediate surfaces with frame-limited pools.

use petunia_core::Rect;
use petunia_render_model::{RenderPrimitive, RenderSnapshot, ViewTransform};
use serde::{Deserialize, Serialize};

/// Nominal render tile edge in device pixels. Runtime detail,
/// independent from authorial surface tiles.
pub const TILE_EDGE: u32 = 64;

/// One tile of work: device rect plus primitive indices in paint
/// order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileWork {
    pub tx: u32,
    pub ty: u32,
    pub rect: Rect,
    pub primitives: Vec<usize>,
}

/// Compiled graph for one frame: tile bins plus culled count.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderGraph {
    pub tiles: Vec<TileWork>,
    pub culled_primitives: usize,
    pub viewport: Rect,
}

impl RenderGraph {
    /// Compile bins from snapshot bounds using the default tile edge (64px).
    #[must_use]
    pub fn compile(snapshot: &RenderSnapshot, view: ViewTransform, viewport: Rect) -> Self {
        Self::compile_with_tile_size(snapshot, view, viewport, TILE_EDGE)
    }

    /// Compile bins with an explicitly requested tile edge size.
    #[must_use]
    pub fn compile_with_tile_size(
        snapshot: &RenderSnapshot,
        view: ViewTransform,
        viewport: Rect,
        tile_size: u32,
    ) -> Self {
        let tile_size = tile_size.clamp(16, 512);
        let mut tiles: Vec<TileWork> = Vec::new();
        let mut culled = 0usize;
        for page in &snapshot.pages {
            for (index, primitive) in page.primitives.iter().enumerate() {
                let Some(bounds) = primitive_bounds(primitive) else {
                    culled += 1;
                    continue;
                };
                let device = transform_rect(bounds, view);
                if !overlaps(device, viewport) {
                    culled += 1;
                    continue;
                }
                for (tx, ty, rect) in tiles_for(device, viewport, tile_size) {
                    match tiles.iter_mut().find(|tile| tile.tx == tx && tile.ty == ty) {
                        Some(tile) => tile.primitives.push(index),
                        None => tiles.push(TileWork {
                            tx,
                            ty,
                            rect,
                            primitives: vec![index],
                        }),
                    }
                }
            }
        }
        tiles.sort_by_key(|tile| (tile.ty, tile.tx));
        Self {
            tiles,
            culled_primitives: culled,
            viewport,
        }
    }
}

fn primitive_bounds(primitive: &RenderPrimitive) -> Option<Rect> {
    match primitive {
        RenderPrimitive::Vector(vector) => Some(vector.bounds),
        RenderPrimitive::Text(text) => Some(text.bounds),
        RenderPrimitive::Image(image) => Some(image.bounds),
        RenderPrimitive::Raster(raster) => Some(raster.bounds),
        RenderPrimitive::Group(group) => Some(group.bounds),
    }
}

fn transform_rect(bounds: Rect, view: ViewTransform) -> Rect {
    let points = [
        (bounds.x, bounds.y),
        (bounds.x + bounds.width, bounds.y),
        (bounds.x + bounds.width, bounds.y + bounds.height),
        (bounds.x, bounds.y + bounds.height),
    ]
    .map(|(x, y)| view.apply(x, y));
    let min_x = points
        .iter()
        .map(|point| point.0)
        .fold(f64::INFINITY, f64::min);
    let min_y = points
        .iter()
        .map(|point| point.1)
        .fold(f64::INFINITY, f64::min);
    let max_x = points
        .iter()
        .map(|point| point.0)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y = points
        .iter()
        .map(|point| point.1)
        .fold(f64::NEG_INFINITY, f64::max);
    Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
}

fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
}

fn tiles_for(device: Rect, viewport: Rect, tile_size: u32) -> Vec<(u32, u32, Rect)> {
    let edge = tile_size as f64;
    let x0 = (device.x.max(viewport.x) / edge).floor().max(0.0) as u32;
    let y0 = (device.y.max(viewport.y) / edge).floor().max(0.0) as u32;
    let x1 = ((device.x + device.width).min(viewport.x + viewport.width) / edge).ceil() as u32;
    let y1 = ((device.y + device.height).min(viewport.y + viewport.height) / edge).ceil() as u32;
    let mut out = Vec::new();
    for ty in y0..y1.max(y0 + 1).max(1) {
        for tx in x0..x1.max(x0 + 1).max(1) {
            if tx as f64 * edge >= viewport.x + viewport.width
                || ty as f64 * edge >= viewport.y + viewport.height
            {
                continue;
            }
            out.push((
                tx,
                ty,
                Rect::new(tx as f64 * edge, ty as f64 * edge, edge, edge),
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_render_model::{RenderPage, RenderPath, RenderSnapshot, SnapshotRevision};

    fn snapshot_with_square() -> RenderSnapshot {
        use petunia_core::{PageId, Size2};
        use petunia_render_model::{RenderAppearance, RenderColor, RenderPaint, VectorPrimitive};
        let mut path = RenderPath::new();
        path.push_contour(
            vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)],
            true,
        );
        RenderSnapshot {
            revision: SnapshotRevision(1),
            pages: vec![RenderPage {
                page: PageId::new_v4(),
                size: Size2::new(100.0, 100.0).expect("valid"),
                primitives: vec![RenderPrimitive::Vector(Box::new(VectorPrimitive {
                    source: petunia_core::ObjectId::new_v4(),
                    geometry: path,
                    appearance: RenderAppearance {
                        fill: Some(RenderPaint::Solid(RenderColor::BLACK)),
                        stroke: None,
                        opacity: 1.0,
                    },
                    transform: petunia_core::Transform2D::IDENTITY,
                    bounds: Rect::new(0.0, 0.0, 10.0, 10.0),
                }))],
            }],
            resources: petunia_render_model::RenderResourceTable::new(),
        }
    }

    #[test]
    fn bins_cover_primitive_tiles_only() {
        let snapshot = snapshot_with_square();
        let view = ViewTransform {
            rotation: 0.0,
            scale: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
        };
        let graph = RenderGraph::compile(&snapshot, view, Rect::new(0.0, 0.0, 128.0, 128.0));
        assert_eq!(graph.culled_primitives, 0);
        assert_eq!(graph.tiles.len(), 1);
        assert_eq!(graph.tiles[0].primitives, vec![0]);
    }

    #[test]
    fn offscreen_primitives_cull_cleanly() {
        let snapshot = snapshot_with_square();
        let view = ViewTransform {
            rotation: 0.0,
            scale: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
        };
        let graph = RenderGraph::compile(&snapshot, view, Rect::new(500.0, 500.0, 64.0, 64.0));
        assert_eq!(graph.culled_primitives, 1);
        assert!(graph.tiles.is_empty());
    }
}
