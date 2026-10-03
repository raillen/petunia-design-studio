//! Disposable raster edit drafts. Canonical pixels are published through one
//! transaction after checking session identity, revision and active surface.
use crate::preview::PreviewSource;
use crate::session::SessionIdentity;
use crate::{Command, DocumentSession, RasterSelection};
use petunia_design_document::{ChangeSet, ShapeKind};
use petunia_design_foundation::{ObjectId, PetuniaError, SurfaceId};
use petunia_design_geometry::{GAffine, GPoint};
use petunia_design_raster::{
    BitDepth, BlendMode, BrushDab, RasterLayer, RasterLayerKind, TileCoord, TILE_SIZE,
};
use std::{collections::HashMap, sync::Arc};

const STROKE_WORK: usize = 16_777_216;
const MAX_SELECTION_VERTICES: usize = 65_536;
const MAX_INTERPOLATED_DABS: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RasterBrush {
    pub radius: f64,
    pub hardness: f32,
    pub opacity: f32,
    pub flow: f32,
    pub color: [f32; 4],
    /// Literal process ink, when supplied. RGB brushes are converted once at
    /// stroke admission through the selected layer's ICC profile.
    pub ink: Option<[f32; 4]>,
    pub erase: bool,
}
impl RasterBrush {
    fn validate(&self) -> Result<(), PetuniaError> {
        if self
            .ink
            .is_some_and(|p| p.iter().any(|v| !v.is_finite() || !(0.0..=1.0).contains(v)))
        {
            return Err(PetuniaError::invalid_input("invalid native ink brush"));
        }
        if !self.radius.is_finite()
            || !(0.1..=256.0).contains(&self.radius)
            || !self.hardness.is_finite()
            || !(0.0..=1.0).contains(&self.hardness)
            || !self.flow.is_finite()
            || !(0.0..=1.0).contains(&self.flow)
            || !self.opacity.is_finite()
            || !(0.0..=1.0).contains(&self.opacity)
            || self
                .color
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            return Err(PetuniaError::invalid_input("invalid raster brush settings"));
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
struct SelectionStencil {
    active: bool,
    path: Option<Arc<tiny_skia::Path>>,
    tiles: HashMap<TileCoord, Arc<Vec<u8>>>,
}
impl SelectionStencil {
    fn prepare(
        selection: &RasterSelection,
        world_to_pixels: GAffine,
    ) -> Result<Self, PetuniaError> {
        if selection.feather != 0.0 && selection.is_active() {
            return Err(PetuniaError::capability_unavailable(
                "feathered pixel painting requires a prepared soft selection mask",
            ));
        }
        let vertices = selection.contours.iter().map(Vec::len).sum::<usize>();
        if vertices > MAX_SELECTION_VERTICES {
            return Err(PetuniaError::invalid_input(
                "raster selection vertex budget exceeded",
            ));
        }
        let mut path = tiny_skia::PathBuilder::new();
        for contour in &selection.contours {
            for (index, point) in contour.iter().enumerate() {
                let p = world_to_pixels.apply(*point);
                if !p.x.is_finite()
                    || !p.y.is_finite()
                    || p.x.abs() > 16_777_216.0
                    || p.y.abs() > 16_777_216.0
                {
                    return Err(PetuniaError::invalid_input(
                        "selection outside raster coordinate budget",
                    ));
                }
                if index == 0 {
                    path.move_to(p.x as f32, p.y as f32);
                } else {
                    path.line_to(p.x as f32, p.y as f32);
                }
            }
            if !contour.is_empty() {
                path.close();
            }
        }
        Ok(Self {
            active: selection.is_active(),
            path: path.finish().map(Arc::new),
            tiles: HashMap::new(),
        })
    }
    fn coverage(&mut self, x: i64, y: i64) -> Result<f32, PetuniaError> {
        if !self.active {
            return Ok(1.0);
        }
        let Some(path) = &self.path else {
            return Ok(0.0);
        };
        let coord = TileCoord::from_pixel(x, y)
            .ok_or_else(|| PetuniaError::invalid_input("invalid selection tile"))?;
        if !self.tiles.contains_key(&coord) {
            if self.tiles.len() >= petunia_design_raster::tile::MAX_RESIDENT_TILES {
                return Err(PetuniaError::invalid_input(
                    "selection stencil memory budget exceeded",
                ));
            }
            let mut mask = tiny_skia::Mask::new(TILE_SIZE as u32, TILE_SIZE as u32)
                .ok_or_else(|| PetuniaError::invalid_input("selection stencil allocation"))?;
            mask.fill_path(
                path,
                tiny_skia::FillRule::EvenOdd,
                true,
                tiny_skia::Transform::from_translate(
                    -(coord.x as f32) * TILE_SIZE as f32,
                    -(coord.y as f32) * TILE_SIZE as f32,
                ),
            );
            self.tiles.insert(coord, Arc::new(mask.data().to_vec()));
        }
        let index = y.rem_euclid(TILE_SIZE as i64) as usize * TILE_SIZE
            + x.rem_euclid(TILE_SIZE as i64) as usize;
        Ok(f32::from(self.tiles[&coord][index]) / 255.0)
    }
}

#[derive(Clone, Debug)]
pub struct RasterStroke {
    session: SessionIdentity,
    revision: u64,
    surface: SurfaceId,
    target: Option<ObjectId>,
    bounds: [f64; 4],
    pixels_to_world: GAffine,
    working: RasterLayer,
    original: RasterLayer,
    accumulation: RasterLayer,
    selection: SelectionStencil,
    brush: RasterBrush,
    last: Option<(GPoint, f64)>,
    remaining_work: usize,
    changed: bool,
    failed: bool,
    preview: Arc<PreviewSource>,
}
impl RasterStroke {
    pub fn begin(session: &DocumentSession, mut brush: RasterBrush) -> Result<Self, PetuniaError> {
        brush.validate()?;
        let surface = session
            .active_surface()
            .ok_or_else(|| PetuniaError::invalid_input("no active surface"))?;
        let source = session.document().surface(surface)?;
        let selected = &session.selection.selected_ids;
        if selected.len() > 1 {
            return Err(PetuniaError::invalid_input(
                "select a single pixel or mask layer for painting",
            ));
        }
        let object = selected
            .first()
            .and_then(|id| session.document().find_object(*id));
        let (target, bounds, pixels_to_world, working) = if let Some(object) =
            object.filter(|o| matches!(o.shape, Some(ShapeKind::Raster { .. })))
        {
            if session.document().find_object_surface(object.id) != Some(surface) {
                return Err(PetuniaError::invalid_input(
                    "paint target is outside active surface",
                ));
            }
            let mut ancestor = Some(object.id);
            while let Some(id) = ancestor {
                let current = session
                    .document()
                    .find_object(id)
                    .ok_or_else(|| PetuniaError::invalid_input("missing paint ancestor"))?;
                if current.locked || !current.visible {
                    return Err(PetuniaError::invalid_input(
                        "pixel layer or ancestor is locked/hidden",
                    ));
                }
                ancestor = current.parent;
            }
            let Some(ShapeKind::Raster { layer }) = &object.shape else {
                unreachable!()
            };
            let bounds = object
                .bounds
                .ok_or_else(|| PetuniaError::invalid_input("pixel layer has no frame"))?;
            let transform = session
                .document()
                .world_transform(object.id)?
                .after(GAffine::scale(
                    bounds[2] / f64::from(layer.width()),
                    bounds[3] / f64::from(layer.height()),
                ));
            (Some(object.id), bounds, transform, layer.as_ref().clone())
        } else {
            if brush.erase {
                return Err(PetuniaError::invalid_input(
                    "select an editable pixel or mask layer to erase",
                ));
            }
            let [x, y, w, h] = source.bounds();
            if ![x, y, w, h].iter().all(|v| v.is_finite())
                || w <= 0.0
                || h <= 0.0
                || w.ceil() > f64::from(petunia_design_raster::layer::MAX_LAYER_DIMENSION)
                || h.ceil() > f64::from(petunia_design_raster::layer::MAX_LAYER_DIMENSION)
            {
                return Err(PetuniaError::invalid_input(
                    "surface exceeds painting dimensions",
                ));
            }
            let layer = RasterLayer::new(
                w.ceil() as u32,
                h.ceil() as u32,
                RasterLayerKind::Pixels,
                BitDepth::Eight,
            )?;
            let transform = GAffine::translate(x, y).after(GAffine::scale(
                w / f64::from(layer.width()),
                h / f64::from(layer.height()),
            ));
            (None, [x, y, w, h], transform, layer)
        };
        let inverse = pixels_to_world
            .inverse()
            .ok_or_else(|| PetuniaError::invalid_input("singular paint frame"))?;
        let selection = SelectionStencil::prepare(&session.raster_selection, inverse)?;
        if working.is_cmyk() {
            if brush.ink.is_none() {
                let profile = working.cmyk_profile().ok_or_else(|| {
                    PetuniaError::invalid_input("paint target CMYK profile missing")
                })?;
                brush.ink = Some(
                    petunia_design_color::rgb_to_cmyk(
                        &petunia_design_color::IccProfile::srgb()?,
                        profile,
                        &[[brush.color[0], brush.color[1], brush.color[2]]],
                        Default::default(),
                    )?[0],
                );
            }
        } else if brush.ink.is_some() {
            return Err(PetuniaError::invalid_input(
                "literal ink painting requires a native CMYK layer",
            ));
        }
        let original = working.clone();
        let accumulation = RasterLayer::new(
            working.width(),
            working.height(),
            RasterLayerKind::Mask,
            BitDepth::Sixteen,
        )?;
        Ok(Self {
            original,
            accumulation,
            session: session.identity(),
            revision: session.current_revision(),
            surface,
            target,
            bounds,
            pixels_to_world,
            working,
            selection,
            brush,
            last: None,
            remaining_work: STROKE_WORK,
            changed: false,
            failed: false,
            preview: PreviewSource::capture(source, session.current_revision()),
        })
    }
    pub fn belongs_to(&self, session: &DocumentSession) -> bool {
        self.session == session.identity()
            && self.revision == session.current_revision()
            && session.active_surface() == Some(self.surface)
    }
    pub fn preview_source(&self) -> Arc<PreviewSource> {
        self.preview.clone()
    }
    pub fn sample(&mut self, point: GPoint, pressure: f64) -> Result<(), PetuniaError> {
        if self.failed {
            return Err(PetuniaError::invalid_input(
                "failed raster gesture cannot continue",
            ));
        }
        let result = self.sample_inner(point, pressure);
        self.failed |= result.is_err();
        result
    }
    fn sample_inner(&mut self, point: GPoint, pressure: f64) -> Result<(), PetuniaError> {
        if !point.x.is_finite()
            || !point.y.is_finite()
            || !pressure.is_finite()
            || !(0.0..=1.0).contains(&pressure)
        {
            return Err(PetuniaError::invalid_input("invalid paint pointer sample"));
        }
        if self
            .last
            .is_some_and(|(last, p)| last.distance_to(point) < 1e-9 && p == pressure)
        {
            return Ok(());
        }
        let (from, old_pressure) = self.last.unwrap_or((point, pressure));
        let mut event_changed = false;
        let spacing = (self.brush.radius * 0.2).max(0.25);
        let count = (from.distance_to(point) / spacing).ceil().max(1.0);
        let inverse = self
            .pixels_to_world
            .inverse()
            .ok_or_else(|| PetuniaError::invalid_input("singular brush frame"))?;
        let [a, b, c, d, _, _] = inverse.coeffs;
        let radius = self.brush.radius * old_pressure.max(pressure);
        let dx = (2.0 * radius * a.hypot(c)).ceil() + 2.0;
        let dy = (2.0 * radius * b.hypot(d)).ceil() + 2.0;
        let estimated = dx.min(f64::from(self.working.width()))
            * dy.min(f64::from(self.working.height()))
            * count;
        if !estimated.is_finite()
            || estimated > 1_048_576.0
            || estimated > self.remaining_work as f64
        {
            return Err(PetuniaError::invalid_input(
                "brush event pixel-work budget exceeded; gesture was not committed",
            ));
        }
        if count > MAX_INTERPOLATED_DABS as f64 {
            return Err(PetuniaError::invalid_input(
                "brush interpolation budget exceeded",
            ));
        }
        for index in 1..=count as usize {
            let t = index as f64 / count;
            let p = GPoint::new(
                from.x + (point.x - from.x) * t,
                from.y + (point.y - from.y) * t,
            );
            let pressure = old_pressure + (pressure - old_pressure) * t;
            let dab = BrushDab {
                center_x: p.x,
                center_y: p.y,
                radius: self.brush.radius * pressure,
                hardness: self.brush.hardness,
                opacity: self.brush.flow,
                color: self.brush.color,
                blend_mode: if self.brush.erase {
                    BlendMode::DestinationOut
                } else {
                    BlendMode::Normal
                },
            };
            event_changed |= if let Some(ink) = self.brush.ink {
                self.working.stamp_cmyk(
                    &dab,
                    ink,
                    self.pixels_to_world,
                    &self.original,
                    &mut self.accumulation,
                    self.brush.opacity,
                    &mut self.remaining_work,
                    |x, y| self.selection.coverage(x, y),
                )?
            } else {
                self.working.stamp(
                    &dab,
                    self.pixels_to_world,
                    &self.original,
                    &mut self.accumulation,
                    self.brush.opacity,
                    &mut self.remaining_work,
                    |x, y| self.selection.coverage(x, y),
                )?
            };
        }
        self.last = Some((point, pressure));
        self.changed |= event_changed;
        if event_changed {
            self.preview = self.preview.with_raster_edit(
                self.target,
                Arc::new(self.working.clone()),
                self.pixels_to_world,
            );
        }
        Ok(())
    }
    /// Lift does not stamp a second dab at an unchanged point when a device
    /// reports zero pressure (or falls back to mouse pressure) on pointer-up.
    pub fn finish_sample(&mut self, point: GPoint, pressure: f64) -> Result<(), PetuniaError> {
        if self
            .last
            .is_some_and(|(last, _)| last.distance_to(point) < 1e-9)
        {
            return Ok(());
        }
        self.sample(point, pressure)
    }

    /// Four-connected scanline flood fill of the immutable source, constrained
    /// by selection coverage. Work, stack, memory and cancellation are bounded.
    pub fn flood_fill(
        &mut self,
        world: GPoint,
        tolerance: f32,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<(), PetuniaError> {
        if self.failed {
            return Err(PetuniaError::invalid_input(
                "failed raster gesture cannot continue",
            ));
        }
        let result = self.flood_fill_inner(world, tolerance, cancelled);
        self.failed |= result.is_err();
        result
    }
    fn flood_fill_inner(
        &mut self,
        world: GPoint,
        tolerance: f32,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<(), PetuniaError> {
        if self.brush.erase
            || !world.x.is_finite()
            || !world.y.is_finite()
            || !tolerance.is_finite()
            || !(0.0..=1.0).contains(&tolerance)
        {
            return Err(PetuniaError::invalid_input("invalid pixel flood fill"));
        }
        let inverse = self
            .pixels_to_world
            .inverse()
            .ok_or_else(|| PetuniaError::invalid_input("singular fill frame"))?;
        let p = inverse.apply(world);
        let width = self.working.width() as usize;
        let height = self.working.height() as usize;
        if p.x < 0.0 || p.y < 0.0 || p.x >= width as f64 || p.y >= height as f64 {
            return Ok(());
        }
        let (sx, sy) = (p.x.floor() as usize, p.y.floor() as usize);
        let seed_ink = if self.original.is_cmyk() {
            Some(self.original.cmyka_pixel(sx as i64, sy as i64)?)
        } else {
            None
        };
        let seed = if seed_ink.is_none() {
            self.original.pixel(sx as i64, sy as i64)?
        } else {
            [0.; 4]
        };
        let mut visited = vec![0u8; (width * height).div_ceil(8)];
        let mut stack = vec![(sx, sy)];
        let mut work = 0usize;
        let matches = |selection: &mut SelectionStencil,
                       x: usize,
                       y: usize,
                       visited: &[u8],
                       work: &mut usize|
         -> Result<bool, PetuniaError> {
            *work += 1;
            if *work > STROKE_WORK * 8 {
                return Err(PetuniaError::invalid_input(
                    "fill pixel-work budget exceeded",
                ));
            }
            let index = y * width + x;
            if visited[index / 8] & (1 << (index % 8)) != 0 {
                return Ok(false);
            }
            let same = if let Some(seed) = seed_ink {
                let pixel = self.original.cmyka_pixel(x as i64, y as i64)?;
                (seed[4] == 0. && pixel[4] == 0.)
                    || (0..5).all(|c| (seed[c] - pixel[c]).abs() <= tolerance)
            } else {
                let pixel = self.original.pixel(x as i64, y as i64)?;
                (seed[3] == 0.0 && pixel[3] == 0.0)
                    || (0..4).all(|channel| (pixel[channel] - seed[channel]).abs() <= tolerance)
            };
            Ok(same && selection.coverage(x as i64, y as i64)? > 0.0)
        };
        while let Some((x, y)) = stack.pop() {
            if cancelled() {
                return Err(PetuniaError::cancelled("pixel fill was cancelled"));
            }
            if !matches(&mut self.selection, x, y, &visited, &mut work)? {
                continue;
            }
            let mut left = x;
            let mut right = x;
            while left > 0 && matches(&mut self.selection, left - 1, y, &visited, &mut work)? {
                left -= 1;
            }
            while right + 1 < width
                && matches(&mut self.selection, right + 1, y, &visited, &mut work)?
            {
                right += 1;
            }
            for x in left..=right {
                let index = y * width + x;
                visited[index / 8] |= 1 << (index % 8);
                let coverage = self.selection.coverage(x as i64, y as i64)?
                    * self.brush.opacity
                    * self.brush.color[3];
                if let Some(ink) = self.brush.ink {
                    let mut dst = self.original.cmyka_pixel(x as i64, y as i64)?;
                    let alpha = coverage + dst[4] * (1.0 - coverage);
                    if alpha > 0. {
                        for c in 0..4 {
                            dst[c] = ((ink[c] * coverage + dst[c] * dst[4] * (1.0 - coverage))
                                / alpha)
                                .clamp(0., 1.);
                        }
                    }
                    dst[4] = alpha;
                    self.changed |= self.working.set_cmyka_pixel(x as i64, y as i64, dst)?;
                    continue;
                }
                let original = self.original.pixel(x as i64, y as i64)?;
                let result = if self.working.kind() == RasterLayerKind::Mask {
                    let luminance = 0.2126 * self.brush.color[0]
                        + 0.7152 * self.brush.color[1]
                        + 0.0722 * self.brush.color[2];
                    [
                        1.0,
                        1.0,
                        1.0,
                        (original[3] + (luminance - original[3]) * coverage).clamp(0.0, 1.0),
                    ]
                } else {
                    let mut color = self.brush.color;
                    color[3] = coverage;
                    BlendMode::Normal.blend(color, original)
                };
                self.changed |= self.working.set_pixel(x as i64, y as i64, result)?;
            }
            for row in [y.checked_sub(1), y.checked_add(1).filter(|y| *y < height)]
                .into_iter()
                .flatten()
            {
                let mut in_run = false;
                for x in left..=right {
                    let candidate = matches(&mut self.selection, x, row, &visited, &mut work)?;
                    if candidate && !in_run {
                        if stack.len() >= 262_144 {
                            return Err(PetuniaError::invalid_input(
                                "fill span-stack budget exceeded",
                            ));
                        }
                        stack.push((x, row));
                    }
                    in_run = candidate;
                }
            }
        }
        Ok(())
    }
    pub fn commit(mut self, session: &mut DocumentSession) -> Result<ChangeSet, PetuniaError> {
        if self.failed {
            return Err(PetuniaError::invalid_input(
                "failed/cancelled raster gesture cannot be committed",
            ));
        }
        if !self.belongs_to(session) {
            return Err(PetuniaError::invalid_input(
                "paint draft is stale; canonical pixels were not changed",
            ));
        }
        if !self.changed {
            return Ok(ChangeSet::empty());
        }
        self.working.commit();
        self.working.validate()?;
        let shape = ShapeKind::Raster {
            layer: Arc::new(self.working),
        };
        let id = self.target.unwrap_or_else(|| session.next_object_id());
        let command = if self.target.is_some() {
            Command::SetShape {
                id,
                shape: Some(shape),
            }
        } else {
            Command::CreateShapeObject {
                surface: self.surface,
                id,
                name: "Pixel layer".into(),
                shape,
                bounds: Some(self.bounds),
                fill: None,
                stroke: None,
                stroke_width: 0.0,
            }
        };
        let changes = session.transact(
            if self.brush.erase {
                "Erase pixels"
            } else {
                "Paint pixels"
            },
            vec![command],
        )?;
        if self.target.is_none() && !changes.is_empty() {
            session.selection.select_exact(vec![id]);
        }
        Ok(changes)
    }
}
