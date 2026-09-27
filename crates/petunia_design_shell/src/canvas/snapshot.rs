use std::collections::HashSet;

use petunia_design_document::ShapeKind;
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_geometry::{GAffine, GPath, GPoint};

use super::{CanvasOverlays, ViewportCamera};
use crate::shell::PetuniaShell;

#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceView {
    pub id: SurfaceId,
    pub bounds: [f64; 4],
    pub background: Option<String>,
    pub guides: Vec<petunia_design_document::Guide>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanvasObjectProjection {
    pub id: ObjectId,
    pub frame_origin: [f64; 2],
    pub size: [f64; 2],
    pub rotation: f64,
    pub world_transform: GAffine,
    pub world_bounds: [f64; 4],
    /// Evaluated world-space outline. The adapter cannot draw a star, an
    /// ellipse or a Bézier path from a bounding box alone, so the geometry
    /// travels with the frame instead of being reconstructed downstream.
    /// `None` only for objects with no evaluable geometry.
    pub outline: Option<GPath>,
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub stroke_width: f64,
    pub opacity: f64,
    pub shape: Option<ShapeKind>,
    pub active: bool,
    pub raster_tiles: Vec<petunia_design_raster::Tile>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanvasSnapshot {
    pub revision: u64,
    pub camera: ViewportCamera,
    pub overlays: CanvasOverlays,
    pub surface: Option<SurfaceView>,
    pub objects: Vec<CanvasObjectProjection>,
}

impl PetuniaShell {
    pub fn canvas_snapshot(&self) -> CanvasSnapshot {
        let camera = self.view_camera();
        let overlays = self.overlays();
        let Some(session) = self.bridge.session() else {
            return CanvasSnapshot {
                revision: 0,
                camera,
                overlays,
                surface: None,
                objects: Vec::new(),
            };
        };
        let revision = session.current_revision();
        let Some(surface_id) = session.active_surface() else {
            return CanvasSnapshot {
                revision,
                camera,
                overlays,
                surface: None,
                objects: Vec::new(),
            };
        };
        let Ok(surface) = session.surface(surface_id) else {
            return CanvasSnapshot {
                revision,
                camera,
                overlays,
                surface: None,
                objects: Vec::new(),
            };
        };
        let visible = camera.visible_doc_rect();
        let selected: HashSet<ObjectId> = session.selection.selected_ids.iter().copied().collect();
        let objects = surface
            .objects()
            .iter()
            .filter(|object| object.visible)
            .filter_map(|object| {
                let bounds = object.bounds?;
                let world_bounds = session
                    .cached_world_bounds(object.id)
                    .or_else(|| session.cached_world_frame_bounds(object.id))
                    .or_else(|| session.cached_bounds(object.id))?;
                let world_transform = session
                    .cached_world_transform(object.id)
                    .or_else(|| session.document().world_transform_checked(object.id).ok())?;
                let [_, _, width, height] = bounds;
                let overlaps = world_bounds[0] < visible.x1
                    && world_bounds[0] + world_bounds[2] > visible.x0
                    && world_bounds[1] < visible.y1
                    && world_bounds[1] + world_bounds[3] > visible.y0;
                if !overlaps {
                    return None;
                }
                let origin = world_transform.apply(GPoint::ORIGIN);
                let rotation = world_transform.coeffs[1].atan2(world_transform.coeffs[0]);
                // Reuses the revision-keyed cache: this is a memoized read,
                // not a per-frame re-evaluation. That distinction is the whole
                // point of the cache (PERF_REPORT §8).
                let outline = session.cached_world_path(object.id).filter(|path| !path.is_empty());
                Some(CanvasObjectProjection {
                    id: object.id,
                    frame_origin: [origin.x, origin.y],
                    size: [width, height],
                    rotation,
                    world_transform,
                    world_bounds,
                    outline,
                    fill: object.fill.clone(),
                    stroke: object.stroke.clone(),
                    stroke_width: object.stroke_width,
                    opacity: object.opacity,
                    shape: object.shape.clone(),
                    active: selected.contains(&object.id),
                    raster_tiles: Vec::new(),
                })
            })
            .collect();
        CanvasSnapshot {
            revision,
            camera,
            overlays,
            surface: Some(SurfaceView {
                id: surface_id,
                bounds: surface.bounds(),
                background: surface.background.clone(),
                guides: surface.guides.clone(),
            }),
            objects,
        }
    }
}
