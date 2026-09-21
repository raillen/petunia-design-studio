//! Hierarchy planning: group, ungroup validation and clip-group planning (10.5).
//!
//! Pure planning over selected IDs (no document access). Callers allocate
//! fresh container IDs, submit the returned commands atomically, and update
//! the selection to the new container.

use petunia_design_document::ContainerRole;
use petunia_design_foundation::{PetuniaError, ObjectId, SurfaceId};

use super::commands::Command;

/// Validated grouping request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupPlan {
    /// Surface owning the children.
    pub surface: SurfaceId,
    /// Fresh container ID (allocated by the caller).
    pub group_id: ObjectId,
    /// Children in z-order.
    pub child_ids: Vec<ObjectId>,
    /// Container role.
    pub role: ContainerRole,
}

/// Validates a grouping over the current selection (Table B).
pub fn plan_group(
    surface: SurfaceId,
    group_id: ObjectId,
    child_ids: Vec<ObjectId>,
    role: ContainerRole,
) -> Result<GroupPlan, PetuniaError> {
    if child_ids.is_empty() {
        return Err(PetuniaError::invalid_input("no objects selected to group"));
    }
    Ok(GroupPlan {
        surface,
        group_id,
        child_ids,
        role,
    })
}

/// Validated clip-group request: first selected object is the mask.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClipGroupPlan {
    /// Surface owning the members.
    pub surface: SurfaceId,
    /// Fresh clip-group container ID (allocated by the caller).
    pub group_id: ObjectId,
    /// Mask boundary object: always the first selected ID.
    pub mask_id: ObjectId,
    /// Clipped content: the remaining selected IDs.
    pub content_ids: Vec<ObjectId>,
}

/// Validates a clip group over the current selection (Table B).
/// Requires at least two selected objects (mask + content).
pub fn plan_clip_group(
    surface: SurfaceId,
    group_id: ObjectId,
    selected: &[ObjectId],
) -> Result<ClipGroupPlan, PetuniaError> {
    if selected.len() < 2 {
        return Err(PetuniaError::invalid_input(
            "clipping mask requires at least two selected objects (mask + content)",
        ));
    }
    Ok(ClipGroupPlan {
        surface,
        group_id,
        mask_id: selected[0],
        content_ids: selected[1..].to_vec(),
    })
}

/// Builds the submission commands for a validated group plan.
#[must_use]
pub fn group_commands(plan: GroupPlan) -> Vec<Command> {
    vec![Command::GroupObjects {
        surface: plan.surface,
        group_id: plan.group_id,
        child_ids: plan.child_ids,
        role: plan.role,
    }]
}

/// Builds the submission commands for a validated clip-group plan.
#[must_use]
pub fn clip_group_commands(plan: ClipGroupPlan) -> Vec<Command> {
    vec![Command::CreateClipGroup {
        surface: plan.surface,
        group_id: plan.group_id,
        mask_id: plan.mask_id,
        content_ids: plan.content_ids,
    }]
}
