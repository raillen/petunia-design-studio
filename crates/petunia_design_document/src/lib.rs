#![forbid(unsafe_code)]

//! Canonical document model: one object tree, typed IDs, explicit changes.
//!
//! Nothing touches storage directly. Mutations arrive as [`CommandRequest`]
//! analogues via [`DocumentMutator`] and produce a [`ChangeSet`].

pub mod appearance;
mod changeset;
mod document;
mod document_object;
pub mod hierarchy;
pub mod modifiers;
mod mutator;
pub mod shape_factory;
pub mod surface_metadata;
pub mod variable_data;

pub use appearance::{
    AppearanceStack, BlendMode, EffectItem, EffectKind, FillItem, GradientStop, LinearGradient,
    Paint, RadialGradient, StrokeAlignment, StrokeCap, StrokeItem, StrokeJoin,
    resolve_color_to_rgb,
};
pub use changeset::{Change, ChangeSet};
pub use document::{Document, Surface};
pub use document_object::{AlignmentMode, ArrangePosition, DistributionAxis, DocumentObject, ShapeKind};
pub use hierarchy::{ContainerRole, HierarchyValidation, MaskMode};
pub use modifiers::{ModifierItem, ModifierKind, OpacityStop, evaluate_modifiers, evaluate_opacity_at};
pub use mutator::DocumentMutator;
pub use surface_metadata::{Bleed, Guide, GuideOrientation, Margins};
pub use variable_data::{
    BindingId, DataBinding, DataMergeEvaluator, DataRecord, DataSourceDefinition, DataSourceFormat,
    DataSourceId, DataSourceParser, DataSourceSchema, FieldDescriptor, FieldId, FieldType,
    FieldValue, MissingValuePolicy, PathSecurity, PreflightFinding, TargetProperty, ValueFormatter,
};
