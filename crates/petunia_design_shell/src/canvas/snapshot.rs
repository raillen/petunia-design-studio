use std::collections::HashSet;
use std::sync::Arc;

use petunia_design_application::preview::PreviewSource;
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
    ///
    /// Shared across snapshots via the revision-keyed [`SnapshotCache`]:
    /// a cache hit bumps the `Arc`, it never re-flattens the geometry.
    pub outline: Option<Arc<GPath>>,
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub stroke_width: f64,
    pub opacity: f64,
    /// Shared like `outline`: the document owns the canonical shape, the
    /// snapshot only carries a reference-counted view of it.
    pub shape: Option<Arc<ShapeKind>>,
    pub active: bool,
    /// Shared like `outline`: cloned once per revision, bumped per frame.
    pub effects: Arc<Vec<petunia_design_document::EffectItem>>,
    /// Shared like `outline`: cloned once per revision, bumped per frame.
    pub adjustments: Arc<Vec<petunia_design_document::adjustments::AdjustmentItem>>,
    pub raster_tiles: Vec<petunia_design_raster::Tile>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanvasSnapshot {
    pub revision: u64,
    /// Immutable worker source shared independently of selection/camera.
    pub preview_source: Option<Arc<PreviewSource>>,
    pub camera: ViewportCamera,
    pub overlays: CanvasOverlays,
    pub surface: Option<SurfaceView>,
    pub objects: Vec<CanvasObjectProjection>,
}

/// One cached scene evaluation: the full unculled projection list plus the
/// surface view, valid for exactly one `(revision, selection_version,
/// surface)` triple. Same interior-mutable pattern as `GeoCache`: readers
/// hold `&self` and share the entry without signature churn.
#[derive(Clone, Debug, Default)]
pub struct SnapshotCache {
    entry: Option<SnapshotCacheEntry>,
    preview: Option<Arc<PreviewSource>>,
}

#[derive(Clone, Debug)]
struct SnapshotCacheEntry {
    revision: u64,
    selection_version: u64,
    surface: SurfaceId,
    surface_view: SurfaceView,
    /// Every projectable object, independent of the camera. Each
    /// `canvas_snapshot` call filters this list by the *current*
    /// `visible_doc_rect` and clones only the visible projections (cheap
    /// `Arc` bumps), so pan/zoom never invalidate the entry.
    projections: Vec<CanvasObjectProjection>,
}

impl SnapshotCache {
    /// Creates an empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prepared_scene(
        &self,
        revision: u64,
        surface: SurfaceId,
    ) -> Option<Arc<petunia_design_render::RenderSurface>> {
        self.preview
            .as_ref()
            .filter(|source| source.revision() == revision && source.surface_id() == surface)?
            .prepared_scene()
    }
    /// Drops the entry (document open/new/close: the session — and therefore
    /// every stable id — was replaced).
    pub fn clear(&mut self) {
        self.entry = None;
        self.preview = None;
    }

    /// Returns the cached scene when the key matches, else `None`.
    fn get(
        &self,
        revision: u64,
        selection_version: u64,
        surface: SurfaceId,
    ) -> Option<&SnapshotCacheEntry> {
        self.entry.as_ref().filter(|entry| {
            entry.revision == revision
                && entry.selection_version == selection_version
                && entry.surface == surface
        })
    }

    /// Stores a fresh full-scene evaluation.
    fn insert(&mut self, entry: SnapshotCacheEntry) {
        self.entry = Some(entry);
    }
}

/// True when the world-space bounds overlap the visible document rect.
fn overlaps_visible(world_bounds: [f64; 4], visible: petunia_design_geometry::GRect) -> bool {
    world_bounds[0] < visible.x1
        && world_bounds[0] + world_bounds[2] > visible.x0
        && world_bounds[1] < visible.y1
        && world_bounds[1] + world_bounds[3] > visible.y0
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
                preview_source: None,
                surface: None,
                objects: Vec::new(),
            };
        };
        let revision = session.current_revision();
        let selection_version = session.selection.version();
        let Some(surface_id) = session.active_surface() else {
            return CanvasSnapshot {
                revision,
                camera,
                overlays,
                preview_source: None,
                surface: None,
                objects: Vec::new(),
            };
        };
        let Ok(surface) = session.surface(surface_id) else {
            return CanvasSnapshot {
                revision,
                camera,
                overlays,
                preview_source: None,
                surface: None,
                objects: Vec::new(),
            };
        };
        let preview_source = {
            let mut cache = self.bridge.snapshot_cache_mut();
            if cache.preview.as_ref().is_none_or(|source| {
                source.revision() != revision || source.surface_id() != surface_id
            }) {
                cache.preview = Some(PreviewSource::capture(surface, revision));
            }
            overlays
                .raster_preview_source
                .clone()
                .or_else(|| cache.preview.clone())
        };
        // Cache hit: filter the stored full-scene projections by the current
        // viewport and clone only the visible ones. Every shared payload
        // travels as an `Arc` bump; camera and overlays stay live outside
        // the key, so pan/zoom/filtering never rebuild the scene.
        if let Some(hit) = self
            .bridge
            .snapshot_cache()
            .get(revision, selection_version, surface_id)
        {
            let visible = camera.visible_doc_rect();
            let objects = hit
                .projections
                .iter()
                .filter(|projection| overlaps_visible(projection.world_bounds, visible))
                .cloned()
                .collect();
            return CanvasSnapshot {
                revision,
                camera,
                overlays,
                preview_source,
                surface: Some(hit.surface_view.clone()),
                objects,
            };
        }
        let selected: HashSet<ObjectId> = session.selection.selected_ids.iter().copied().collect();
        // Cache miss: rebuild the full scene without camera culling (the
        // filter above runs per call), then store it under the current key.
        let projections = surface
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
                let origin = world_transform.apply(GPoint::ORIGIN);
                let rotation = world_transform.coeffs[1].atan2(world_transform.coeffs[0]);
                // Reuses the revision-keyed cache: this is a memoized read,
                // not a per-frame re-evaluation. That distinction is the whole
                // point of the cache (PERF_REPORT §8).
                let outline = session
                    .cached_world_path(object.id)
                    .filter(|path| !path.is_empty())
                    .map(Arc::new);
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
                    shape: object.shape.clone().map(Arc::new),
                    active: selected.contains(&object.id),
                    effects: Arc::new(
                        object
                            .appearance
                            .as_ref()
                            .map(|a| a.effects.clone())
                            .unwrap_or_default(),
                    ),
                    adjustments: Arc::new(
                        object
                            .appearance
                            .as_ref()
                            .map(|a| a.adjustments.clone())
                            .unwrap_or_default(),
                    ),
                    raster_tiles: Vec::new(),
                })
            })
            .collect::<Vec<_>>();
        let surface_view = SurfaceView {
            id: surface_id,
            bounds: surface.bounds(),
            background: surface.background.clone(),
            guides: surface.guides.clone(),
        };
        self.bridge.snapshot_cache_mut().insert(SnapshotCacheEntry {
            revision,
            selection_version,
            surface: surface_id,
            surface_view: surface_view.clone(),
            projections: projections.clone(),
        });
        let visible = camera.visible_doc_rect();
        let objects = projections
            .into_iter()
            .filter(|projection| overlaps_visible(projection.world_bounds, visible))
            .collect();
        CanvasSnapshot {
            revision,
            camera,
            overlays,
            preview_source,
            surface: Some(surface_view),
            objects,
        }
    }
}
