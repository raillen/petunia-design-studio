#![forbid(unsafe_code)]

//! Canonical document model: one object tree, typed IDs, explicit changes.
//!
//! Nothing touches storage directly. Mutations arrive as [`CommandRequest`]
//! analogues via [`DocumentMutator`] and produce a [`ChangeSet`].

mod changeset;
mod document;
mod document_object;
mod mutator;

pub use changeset::{Change, ChangeSet};
pub use document::{Document, Surface};
pub use document_object::DocumentObject;
pub use mutator::DocumentMutator;
