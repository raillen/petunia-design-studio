//! Undoable commands executed through [`aubrieta_document::DocumentMutator`].

use aubrieta_document::{ChangeSet, Document, DocumentMutator, DocumentObject};
use aubrieta_foundation::{AubrietaError, ObjectId, SurfaceId};

/// Single undoable command with explicit IDs (no hidden state).
#[derive(Clone, Debug)]
pub enum Command {
    /// Create a surface.
    CreateSurface { id: SurfaceId, name: String },
    /// Create an object on a surface.
    CreateObject {
        surface: SurfaceId,
        id: ObjectId,
        name: String,
    },
    /// Delete an object by stable ID.
    DeleteObject { id: ObjectId },
    /// Set an object's semantic fill token.
    SetFill { id: ObjectId, fill: Option<String> },
    /// Set an object's visibility flag.
    SetVisibility { id: ObjectId, visible: bool },
    /// Set an object's locked flag.
    SetLocked { id: ObjectId, locked: bool },
    /// Set an object's opacity factor in [0.0, 1.0].
    SetOpacity { id: ObjectId, opacity: f64 },
    /// Set an object's stroke and stroke width.
    SetStroke {
        id: ObjectId,
        stroke: Option<String>,
        width: f64,
    },
    /// Set an object's bounds and rotation.
    SetBounds {
        id: ObjectId,
        bounds: Option<[f64; 4]>,
        rotation: f64,
    },
    /// Reorder an object within a surface.
    ReorderObject {
        surface: SurfaceId,
        id: ObjectId,
        new_index: usize,
    },
    /// Set an object's complete appearance stack (10.4).
    SetAppearance {
        id: ObjectId,
        appearance: Option<aubrieta_document::AppearanceStack>,
    },
    /// Groups objects into a container under the One-Tree invariant (10.5).
    GroupObjects {
        surface: SurfaceId,
        group_id: ObjectId,
        child_ids: Vec<ObjectId>,
        role: aubrieta_document::ContainerRole,
    },
    /// Ungroups a container object, moving its children to its parent.
    Ungroup { group_id: ObjectId },
    /// Reparents an object to a new container or root.
    ReparentObject {
        id: ObjectId,
        new_parent: Option<ObjectId>,
        target_index: usize,
        preserve_world_transform: bool,
    },
    /// Creates a clipping mask group where mask_id clips content_ids.
    CreateClipGroup {
        surface: SurfaceId,
        group_id: ObjectId,
        mask_id: ObjectId,
        content_ids: Vec<ObjectId>,
    },
    /// Releases a clipping mask group.
    ReleaseClipGroup { group_id: ObjectId },
    /// Sets a surface's origin and dimensions (10.7).
    SetSurfaceGeometry {
        surface: SurfaceId,
        origin: [f64; 2],
        dimensions: [f64; 2],
    },
    /// Sets a surface's bleed configuration.
    SetSurfaceBleed {
        surface: SurfaceId,
        bleed: aubrieta_document::Bleed,
    },
    /// Sets a surface's margins configuration.
    SetSurfaceMargins {
        surface: SurfaceId,
        margins: aubrieta_document::Margins,
    },
    /// Sets a surface's background color token.
    SetSurfaceBackground {
        surface: SurfaceId,
        background: Option<String>,
    },
    /// Adds a layout guide to a surface.
    AddGuide {
        surface: SurfaceId,
        guide: aubrieta_document::Guide,
    },
    /// Removes a layout guide from a surface.
    RemoveGuide { surface: SurfaceId, guide_id: u32 },
    /// Moves an object between surfaces with coordinate compensation.
    MoveObjectToSurface {
        id: ObjectId,
        target_surface: SurfaceId,
        preserve_world_transform: bool,
    },
    /// Registers a variable data source (10.11).
    AddDataSource {
        source: aubrieta_document::DataSourceDefinition,
    },
    /// Removes a variable data source (10.11).
    RemoveDataSource { id: aubrieta_document::DataSourceId },
    /// Adds a data binding (10.11).
    AddDataBinding {
        binding: aubrieta_document::DataBinding,
    },
    /// Removes a data binding (10.11).
    RemoveDataBinding { id: aubrieta_document::BindingId },
    /// Materializes variable data records into surfaces (10.11).
    MaterializeDataMerge {
        source_id: aubrieta_document::DataSourceId,
        template_surface: SurfaceId,
    },
    /// Create an object with explicit shape and appearance.
    CreateShapeObject {
        surface: SurfaceId,
        id: ObjectId,
        name: String,
        shape: aubrieta_document::ShapeKind,
        bounds: Option<[f64; 4]>,
        fill: Option<String>,
        stroke: Option<String>,
        stroke_width: f64,
    },
    /// Set an object's vector shape or text content.
    SetShape {
        id: ObjectId,
        shape: Option<aubrieta_document::ShapeKind>,
    },
    /// Executes a vector boolean operation on two objects.
    ApplyBoolean {
        surface: SurfaceId,
        target_id: ObjectId,
        subject_id: ObjectId,
        clip_id: ObjectId,
        op: aubrieta_geometry::BooleanOp,
    },
    /// Converts a parametric shape or text object to an editable vector path (10.3, 10.6).
    ConvertToCurves { id: ObjectId },
    /// Bakes corner geometry into an explicit vector path (10.2, 10.3).
    BakeCorners { id: ObjectId },
    /// Offsets a path or object bounds outward or inward (10.3).
    OffsetPath { id: ObjectId, delta: f64 },
    /// Aligns multiple objects relative to their collective bounds (10.1).
    AlignObjects {
        surface: SurfaceId,
        ids: Vec<ObjectId>,
        mode: aubrieta_document::AlignmentMode,
    },
    /// Distributes objects evenly along an axis (10.1).
    DistributeObjects {
        surface: SurfaceId,
        ids: Vec<ObjectId>,
        axis: aubrieta_document::DistributionAxis,
    },
    /// Slices or splits a path object at a specific point (10.2).
    SlicePath { id: ObjectId, point: [f64; 2] },
}

/// Validated command ready for execution.
#[derive(Clone, Debug)]
pub struct CommandRequest {
    /// Command to execute.
    pub command: Command,
}

impl CommandRequest {
    /// Wraps a command.
    #[must_use]
    pub fn new(command: Command) -> Self {
        Self { command }
    }
}

/// Executes one command, returning the produced [`ChangeSet`].
pub fn execute(
    document: &mut Document,
    request: &CommandRequest,
) -> Result<ChangeSet, AubrietaError> {
    let mut mutator = DocumentMutator::new(document);
    match &request.command {
        Command::CreateSurface { id, name } => mutator.add_surface(*id, name.clone()),
        Command::CreateObject { surface, id, name } => {
            mutator.add_object(*surface, DocumentObject::new(*id, name.clone()))
        }
        Command::DeleteObject { id } => mutator.remove_object(*id),
        Command::SetFill { id, fill } => mutator.set_fill(*id, fill.clone()),
        Command::SetVisibility { id, visible } => mutator.set_visibility(*id, *visible),
        Command::SetLocked { id, locked } => mutator.set_locked(*id, *locked),
        Command::SetOpacity { id, opacity } => mutator.set_opacity(*id, *opacity),
        Command::SetStroke { id, stroke, width } => mutator.set_stroke(*id, stroke.clone(), *width),
        Command::SetBounds {
            id,
            bounds,
            rotation,
        } => mutator.set_bounds(*id, *bounds, *rotation),
        Command::ReorderObject {
            surface,
            id,
            new_index,
        } => mutator.reorder_object(*surface, *id, *new_index),
        Command::SetAppearance { id, appearance } => {
            mutator.set_appearance(*id, appearance.clone())
        }
        Command::GroupObjects {
            surface,
            group_id,
            child_ids,
            role,
        } => mutator.group_objects(*surface, *group_id, child_ids.clone(), *role),
        Command::Ungroup { group_id } => mutator.ungroup_objects(*group_id),
        Command::ReparentObject {
            id,
            new_parent,
            target_index,
            preserve_world_transform,
        } => mutator.reparent_object(*id, *new_parent, *target_index, *preserve_world_transform),
        Command::CreateClipGroup {
            surface,
            group_id,
            mask_id,
            content_ids,
        } => mutator.create_clip_group(*surface, *group_id, *mask_id, content_ids.clone()),
        Command::ReleaseClipGroup { group_id } => mutator.release_clip_group(*group_id),
        Command::SetSurfaceGeometry {
            surface,
            origin,
            dimensions,
        } => mutator.set_surface_geometry(*surface, *origin, *dimensions),
        Command::SetSurfaceBleed { surface, bleed } => mutator.set_surface_bleed(*surface, *bleed),
        Command::SetSurfaceMargins { surface, margins } => {
            mutator.set_surface_margins(*surface, *margins)
        }
        Command::SetSurfaceBackground {
            surface,
            background,
        } => mutator.set_surface_background(*surface, background.clone()),
        Command::AddGuide { surface, guide } => mutator.add_surface_guide(*surface, guide.clone()),
        Command::RemoveGuide { surface, guide_id } => {
            mutator.remove_surface_guide(*surface, *guide_id)
        }
        Command::MoveObjectToSurface {
            id,
            target_surface,
            preserve_world_transform,
        } => mutator.move_object_between_surfaces(*id, *target_surface, *preserve_world_transform),
        Command::AddDataSource { source } => mutator.add_data_source(source.clone()),
        Command::RemoveDataSource { id } => mutator.remove_data_source(*id),
        Command::AddDataBinding { binding } => mutator.add_data_binding(binding.clone()),
        Command::RemoveDataBinding { id } => mutator.remove_data_binding(*id),
        Command::MaterializeDataMerge {
            source_id,
            template_surface,
        } => {
            let max_surf_id = mutator
                .document()
                .surfaces
                .iter()
                .map(|s| s.id.raw())
                .max()
                .unwrap_or(0);
            let max_obj_id = mutator
                .document()
                .surfaces
                .iter()
                .flat_map(|s| s.objects.iter().map(|o| o.id.raw()))
                .max()
                .unwrap_or(0);
            let start = max_surf_id.max(max_obj_id) + 1;
            let mut gen = aubrieta_foundation::IdGenerator::with_start(start);
            mutator.materialize_data_merge(*source_id, *template_surface, &mut gen)
        }
        Command::CreateShapeObject {
            surface,
            id,
            name,
            shape,
            bounds,
            fill,
            stroke,
            stroke_width,
        } => {
            let mut obj = DocumentObject::new(*id, name.clone());
            obj.shape = Some(shape.clone());
            obj.bounds = *bounds;
            obj.fill = fill.clone();
            obj.stroke = stroke.clone();
            obj.stroke_width = *stroke_width;
            mutator.add_object(*surface, obj)
        }
        Command::SetShape { id, shape } => mutator.set_shape(*id, shape.clone()),
        Command::ApplyBoolean {
            surface,
            target_id,
            subject_id,
            clip_id,
            op,
        } => {
            let subject = mutator
                .document()
                .find_object(*subject_id)
                .ok_or_else(|| AubrietaError::not_found(format!("subject `{subject_id}` not found")))?
                .clone();
            let clip = mutator
                .document()
                .find_object(*clip_id)
                .ok_or_else(|| AubrietaError::not_found(format!("clip `{clip_id}` not found")))?
                .clone();

            let subj_path = subject.to_path();
            let clip_path = clip.to_path();

            let subj_input = aubrieta_geometry::BooleanInput::new(subj_path.to_polygons(0.5));
            let clip_input = aubrieta_geometry::BooleanInput::new(clip_path.to_polygons(0.5));

            let result_contours = aubrieta_geometry::boolean_op(&subj_input, &clip_input, *op);
            let result_path = aubrieta_geometry::GPath::from_polygons(&result_contours);
            let bounds = result_path
                .bounding_box()
                .map(|r| [r.x0, r.y0, r.width(), r.height()]);

            let mut result_obj = DocumentObject::new(*target_id, format!("{op:?} Result"));
            result_obj.shape = Some(aubrieta_document::ShapeKind::Path(result_path));
            result_obj.bounds = bounds;
            result_obj.fill = subject.fill.clone();
            result_obj.stroke = subject.stroke.clone();
            result_obj.stroke_width = subject.stroke_width;

            let mut changes = ChangeSet::empty();
            let c1 = mutator.remove_object(*subject_id)?;
            changes.extend(c1);
            let c2 = mutator.remove_object(*clip_id)?;
            changes.extend(c2);
            let c3 = mutator.add_object(*surface, result_obj)?;
            changes.extend(c3);

            Ok(changes)
        }
        Command::ConvertToCurves { id } => mutator.convert_to_curves(*id),
        Command::BakeCorners { id } => mutator.bake_corners(*id),
        Command::OffsetPath { id, delta } => mutator.offset_path(*id, *delta),
        Command::AlignObjects { surface, ids, mode } => mutator.align_objects(*surface, ids, *mode),
        Command::DistributeObjects { surface, ids, axis } => {
            mutator.distribute_objects(*surface, ids, *axis)
        }
        Command::SlicePath { id, point } => mutator.slice_path(*id, *point),
    }
}
