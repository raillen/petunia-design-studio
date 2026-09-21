//! Boolean operation planning shared by tools, panels and MCP (10.3).
//!
//! Pure planning (no document access): given a selection and a modifier
//! intent, decides subject/clip/operation. Execution stays a single
//! `Command::ApplyBoolean`, submitted atomically by the caller.

use aubrieta_foundation::{AubrietaError, ObjectId};
use aubrieta_geometry::BooleanOp;

/// Planned boolean operation over two operands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BooleanPlan {
    /// Shape the operation subtracts from / unions with.
    pub subject_id: ObjectId,
    /// Second operand.
    pub clip_id: ObjectId,
    /// Operation to apply.
    pub op: BooleanOp,
}

/// Plans a boolean from the current selection (F-21, Table B).
/// Requires at least two selected objects; uses the first two in order.
/// `prefer_difference` (e.g. Alt/Center modifier) selects `Difference`,
/// otherwise `Union`.
pub fn plan_boolean(
    selected: &[ObjectId],
    prefer_difference: bool,
) -> Result<BooleanPlan, AubrietaError> {
    if selected.len() < 2 {
        return Err(AubrietaError::invalid_input(
            "boolean operation requires at least two selected objects",
        ));
    }
    Ok(BooleanPlan {
        subject_id: selected[0],
        clip_id: selected[1],
        op: if prefer_difference {
            BooleanOp::Difference
        } else {
            BooleanOp::Union
        },
    })
}
