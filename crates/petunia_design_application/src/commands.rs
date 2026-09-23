//! Undoable commands executed through [`petunia_design_document::DocumentMutator`].

use petunia_design_document::{ChangeSet, Document, DocumentMutator, DocumentObject};
use petunia_design_foundation::{PetuniaError, ObjectId, SurfaceId};

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
    /// Arrange an object one step or to a z-order edge (10.1, F-16).
    ArrangeObject {
        surface: SurfaceId,
        id: ObjectId,
        position: petunia_design_document::ArrangePosition,
    },
    /// Duplicate an object onto the same surface under a new identity.
    ///
    /// The copy keeps appearance, shape and flags but drops container
    /// relationships: it must not inherit a parent or children it does not
    /// actually contain.
    DuplicateObject {
        surface: SurfaceId,
        source: ObjectId,
        id: ObjectId,
        offset: [f64; 2],
    },
    /// Set an object's complete appearance stack (10.4).
    SetAppearance {
        id: ObjectId,
        appearance: Option<petunia_design_document::AppearanceStack>,
    },
    /// Append one fill entry (F-18, 10.4). Id collisions are reassigned.
    AddFill {
        id: ObjectId,
        fill: petunia_design_document::FillItem,
    },
    /// Remove one fill entry by local id (F-18).
    RemoveFill { id: ObjectId, fill_id: u32 },
    /// Set one fill entry's opacity (F-18).
    SetFillItemOpacity {
        id: ObjectId,
        fill_id: u32,
        opacity: f64,
    },
    /// Set one fill entry's blend mode (F-18).
    SetFillItemBlend {
        id: ObjectId,
        fill_id: u32,
        blend_mode: petunia_design_document::BlendMode,
    },
    /// Reorder one fill entry within stack order (F-18).
    ReorderFill {
        id: ObjectId,
        fill_id: u32,
        new_index: usize,
    },
    /// Append one stroke entry (F-18). Id collisions are reassigned.
    AddStroke {
        id: ObjectId,
        stroke: petunia_design_document::StrokeItem,
    },
    /// Remove one stroke entry by local id (F-18).
    RemoveStroke { id: ObjectId, stroke_id: u32 },
    /// Set one stroke entry's width (F-18).
    SetStrokeItemWidth {
        id: ObjectId,
        stroke_id: u32,
        width: f64,
    },
    /// Reorder one stroke entry within stack order (F-18).
    ReorderStroke {
        id: ObjectId,
        stroke_id: u32,
        new_index: usize,
    },
    /// Append one effect entry (F-18). Id collisions are reassigned.
    AddEffect {
        id: ObjectId,
        effect: petunia_design_document::EffectItem,
    },
    /// Remove one effect entry by local id (F-18).
    RemoveEffect { id: ObjectId, effect_id: u32 },
    /// Enable/disable one effect entry without deleting it (F-18).
    ToggleEffect {
        id: ObjectId,
        effect_id: u32,
        visible: bool,
    },
    /// Set whole-stack opacity (F-18).
    SetStackOpacity { id: ObjectId, opacity: f64 },
    /// Set whole-stack blend mode (F-18).
    SetStackBlend {
        id: ObjectId,
        blend_mode: petunia_design_document::BlendMode,
    },
    /// Copy an appearance stack between objects (F-18, Copy/Paste Style).
    PasteAppearance {
        source_id: ObjectId,
        dest_id: ObjectId,
    },
    /// Clear all effects, keeping fills/strokes (F-18).
    ClearEffects { id: ObjectId },
    /// Groups objects into a container under the One-Tree invariant (10.5).
    GroupObjects {
        surface: SurfaceId,
        group_id: ObjectId,
        child_ids: Vec<ObjectId>,
        role: petunia_design_document::ContainerRole,
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
        bleed: petunia_design_document::Bleed,
    },
    /// Sets a surface's margins configuration.
    SetSurfaceMargins {
        surface: SurfaceId,
        margins: petunia_design_document::Margins,
    },
    /// Sets a surface's background color token.
    SetSurfaceBackground {
        surface: SurfaceId,
        background: Option<String>,
    },
    /// Adds a layout guide to a surface.
    AddGuide {
        surface: SurfaceId,
        guide: petunia_design_document::Guide,
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
        source: petunia_design_document::DataSourceDefinition,
    },
    /// Removes a variable data source (10.11).
    RemoveDataSource { id: petunia_design_document::DataSourceId },
    /// Adds a data binding (10.11).
    AddDataBinding {
        binding: petunia_design_document::DataBinding,
    },
    /// Removes a data binding (10.11).
    RemoveDataBinding { id: petunia_design_document::BindingId },
    /// Materializes variable data records into surfaces (10.11).
    MaterializeDataMerge {
        source_id: petunia_design_document::DataSourceId,
        template_surface: SurfaceId,
    },
    /// Create an object with explicit shape and appearance.
    CreateShapeObject {
        surface: SurfaceId,
        id: ObjectId,
        name: String,
        shape: petunia_design_document::ShapeKind,
        bounds: Option<[f64; 4]>,
        fill: Option<String>,
        stroke: Option<String>,
        stroke_width: f64,
    },
    /// Set an object's vector shape or text content.
    SetShape {
        id: ObjectId,
        shape: Option<petunia_design_document::ShapeKind>,
    },
    /// Executes a vector boolean operation on two objects.
    ApplyBoolean {
        surface: SurfaceId,
        target_id: ObjectId,
        subject_id: ObjectId,
        clip_id: ObjectId,
        op: petunia_design_geometry::BooleanOp,
    },
    /// Divides two overlapping objects into non-overlapping pieces (10.3, F-21).
    /// Produces up to three results: subject-only, clip-only, intersection.
    /// Empty pieces are skipped; caller must supply three distinct fresh IDs
    /// and read back which results exist via the returned `ChangeSet`.
    DivideObjects {
        surface: SurfaceId,
        subject_id: ObjectId,
        clip_id: ObjectId,
        subject_only_id: ObjectId,
        clip_only_id: ObjectId,
        intersection_id: ObjectId,
    },
    /// Converts a parametric shape or text object to an editable vector path (10.3, 10.6).
    ConvertToCurves { id: ObjectId },
    /// Bakes corner geometry into an explicit vector path (10.2, 10.3).
    BakeCorners { id: ObjectId },
    /// Offsets an outline, non-destructively (09.31, 10.3).
    /// Upserts the live `ContourOffset` modifier; base geometry is untouched.
    OffsetPath { id: ObjectId, delta: f64 },
    /// Replaces an object's live modifier chain (09.31, one undo entry).
    SetModifiers {
        id: ObjectId,
        modifiers: Vec<petunia_design_document::ModifierItem>,
    },
    /// Bakes live contour offsets into base geometry (explicit user op, 09.31).
    BakeContour { id: ObjectId },
    /// Bakes live transparency gradients into base opacity (explicit, 09.31).
    /// Documented approximation: the center sample flattens the mask.
    BakeTransparency { id: ObjectId },
    /// Aligns multiple objects relative to their collective bounds (10.1).
    AlignObjects {
        surface: SurfaceId,
        ids: Vec<ObjectId>,
        mode: petunia_design_document::AlignmentMode,
    },
    /// Distributes objects evenly along an axis (10.1).
    DistributeObjects {
        surface: SurfaceId,
        ids: Vec<ObjectId>,
        axis: petunia_design_document::DistributionAxis,
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

/// Outcome of executing a command (F-22, 09.3).
/// `Committed` carries the produced `ChangeSet`; `NoOp` means the command
/// was valid but changed nothing (empty set) and must not touch history.
#[derive(Clone, Debug)]
pub enum CommandResult {
    /// Mutation produced observable changes.
    Committed(ChangeSet),
    /// Valid command with no observable change.
    NoOp,
}

impl CommandResult {
    /// True when a commit happened.
    #[must_use]
    pub fn committed(&self) -> bool {
        matches!(self, Self::Committed(_))
    }

    /// Returns the change set for committed results.
    #[must_use]
    pub fn changeset(self) -> ChangeSet {
        match self {
            Self::Committed(cs) => cs,
            Self::NoOp => ChangeSet::empty(),
        }
    }
}

/// Executes one command, returning the produced [`ChangeSet`].
pub fn execute(
    document: &mut Document,
    request: &CommandRequest,
) -> Result<ChangeSet, PetuniaError> {
    let mut mutator = DocumentMutator::new(document);
    match &request.command {
        Command::CreateSurface { id, name } => mutator.add_surface(*id, name.clone()),
        Command::CreateObject { surface, id, name } => {
            mutator.add_object(*surface, DocumentObject::new(*id, name.clone()))
        }
        Command::DeleteObject { id } => mutator.remove_object(*id),
        Command::DuplicateObject {
            surface,
            source,
            id,
            offset,
        } => {
            let original = mutator
                .document()
                .find_object(*source)
                .cloned()
                .ok_or_else(|| {
                    petunia_design_foundation::PetuniaError::invalid_input(format!(
                        "cannot duplicate unknown object `{source}`"
                    ))
                })?;
            let mut copy = original;
            copy.id = *id;
            copy.parent = None;
            copy.children = Vec::new();
            copy.role = None;
            copy.clip_mask_id = None;
            if let Some([x, y, w, h]) = copy.bounds {
                copy.bounds = Some([x + offset[0], y + offset[1], w, h]);
            }
            mutator.add_object(*surface, copy)
        }
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
        Command::ArrangeObject {
            surface,
            id,
            position,
        } => mutator.arrange_object(*surface, *id, *position),
        Command::SetAppearance { id, appearance } => {
            mutator.set_appearance(*id, appearance.clone())
        }
        Command::AddFill { id, fill } => mutator.add_fill(*id, fill.clone()),
        Command::RemoveFill { id, fill_id } => mutator.remove_fill(*id, *fill_id),
        Command::SetFillItemOpacity {
            id,
            fill_id,
            opacity,
        } => mutator.set_fill_item_opacity(*id, *fill_id, *opacity),
        Command::SetFillItemBlend {
            id,
            fill_id,
            blend_mode,
        } => mutator.set_fill_item_blend(*id, *fill_id, *blend_mode),
        Command::ReorderFill {
            id,
            fill_id,
            new_index,
        } => mutator.reorder_fill(*id, *fill_id, *new_index),
        Command::AddStroke { id, stroke } => mutator.add_stroke(*id, stroke.clone()),
        Command::RemoveStroke { id, stroke_id } => mutator.remove_stroke(*id, *stroke_id),
        Command::SetStrokeItemWidth {
            id,
            stroke_id,
            width,
        } => mutator.set_stroke_item_width(*id, *stroke_id, *width),
        Command::ReorderStroke {
            id,
            stroke_id,
            new_index,
        } => mutator.reorder_stroke(*id, *stroke_id, *new_index),
        Command::AddEffect { id, effect } => mutator.add_effect(*id, effect.clone()),
        Command::RemoveEffect { id, effect_id } => mutator.remove_effect(*id, *effect_id),
        Command::ToggleEffect {
            id,
            effect_id,
            visible,
        } => mutator.toggle_effect(*id, *effect_id, *visible),
        Command::SetStackOpacity { id, opacity } => mutator.set_stack_opacity(*id, *opacity),
        Command::SetStackBlend { id, blend_mode } => mutator.set_stack_blend(*id, *blend_mode),
        Command::PasteAppearance { source_id, dest_id } => {
            mutator.paste_appearance(*source_id, *dest_id)
        }
        Command::ClearEffects { id } => mutator.clear_effects(*id),
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
                .surfaces()
                .iter()
                .map(|s| s.id.raw())
                .max()
                .unwrap_or(0);
            let max_obj_id = mutator
                .document()
                .surfaces()
                .iter()
                .flat_map(|s| s.objects().iter().map(|o| o.id.raw()))
                .max()
                .unwrap_or(0);
            let start = max_surf_id.max(max_obj_id) + 1;
            let mut gen = petunia_design_foundation::IdGenerator::with_start(start);
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
                .ok_or_else(|| {
                    PetuniaError::not_found(format!("subject `{subject_id}` not found"))
                })?
                .clone();
            let clip = mutator
                .document()
                .find_object(*clip_id)
                .ok_or_else(|| PetuniaError::not_found(format!("clip `{clip_id}` not found")))?
                .clone();

            // Explicit flatten tolerance (F-21): part of the operation's
            // evidence, no longer a magic literal. Operands read evaluated
            // (09.31): live modifiers participate without being consumed.
            let tolerance = petunia_design_geometry::GeometryTolerance::default_tolerance().clamped();
            let subj_path = subject.evaluated_path();
            let clip_path = clip.evaluated_path();

            let subj_input =
                petunia_design_geometry::BooleanInput::new(subj_path.to_polygons(tolerance.flatten));
            let clip_input =
                petunia_design_geometry::BooleanInput::new(clip_path.to_polygons(tolerance.flatten));

            let result_contours = petunia_design_geometry::boolean_op(&subj_input, &clip_input, *op);
            let result_path = petunia_design_geometry::GPath::from_polygons(&result_contours);
            let bounds = result_path
                .bounding_box()
                .map(|r| [r.x0, r.y0, r.width(), r.height()]);

            let mut result_obj = DocumentObject::new(*target_id, format!("{op:?} Result"));
            result_obj.shape = Some(petunia_design_document::ShapeKind::Path(result_path));
            result_obj.bounds = bounds;
            // Style provenance (10.3): inherit the subject's full appearance
            // stack instead of only legacy fill/stroke.
            result_obj.appearance = subject.appearance.clone();
            result_obj.fill = subject.fill.clone();
            result_obj.stroke = subject.stroke.clone();
            result_obj.stroke_width = subject.stroke_width;
            result_obj.opacity = subject.opacity;

            // All fallible work is done: mutate in one atomic sequence.
            let mut changes = ChangeSet::empty();
            let c1 = mutator.remove_object(*subject_id)?;
            changes.extend(c1);
            // Subject and clip may be the same object (self-operation).
            if *clip_id != *subject_id {
                let c2 = mutator.remove_object(*clip_id)?;
                changes.extend(c2);
            }
            let c3 = mutator.add_object(*surface, result_obj)?;
            changes.extend(c3);

            Ok(changes)
        }
        Command::DivideObjects {
            surface,
            subject_id,
            clip_id,
            subject_only_id,
            clip_only_id,
            intersection_id,
        } => mutator.divide_objects(
            *surface,
            *subject_id,
            *clip_id,
            *subject_only_id,
            *clip_only_id,
            *intersection_id,
        ),
        Command::ConvertToCurves { id } => mutator.convert_to_curves(*id),
        Command::BakeCorners { id } => mutator.bake_corners(*id),
        Command::OffsetPath { id, delta } => mutator.offset_path(*id, *delta),
        Command::SetModifiers { id, modifiers } => mutator.set_modifiers(*id, modifiers.clone()),
        Command::BakeContour { id } => mutator.bake_contour(*id),
        Command::BakeTransparency { id } => mutator.bake_transparency(*id),
        Command::AlignObjects { surface, ids, mode } => mutator.align_objects(*surface, ids, *mode),
        Command::DistributeObjects { surface, ids, axis } => {
            mutator.distribute_objects(*surface, ids, *axis)
        }
        Command::SlicePath { id, point } => mutator.slice_path(*id, *point),
    }
}
