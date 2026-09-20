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
mod mutator;
pub mod surface_metadata;

pub use appearance::{
    AppearanceStack, BlendMode, EffectItem, EffectKind, FillItem, GradientStop, LinearGradient,
    Paint, RadialGradient, StrokeAlignment, StrokeCap, StrokeItem, StrokeJoin,
};
pub use changeset::{Change, ChangeSet};
pub use document::{Document, Surface};
pub use document_object::DocumentObject;
pub use hierarchy::{ContainerRole, HierarchyValidation, MaskMode};
pub use mutator::DocumentMutator;
pub use surface_metadata::{Bleed, Guide, GuideOrientation, Margins};
