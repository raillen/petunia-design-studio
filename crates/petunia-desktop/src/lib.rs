//! Native desktop boundary. Document edits remain in StudioSession and Engine.
//! Qt is optional so headless logic remains testable without a GUI SDK.
#![deny(unsafe_code)]

pub mod controller;
pub mod icons;

// SAFETY: CXX-Qt generates the audited Rust/Qt FFI for this boundary only.
// No Qt types or unsafe code enter Core, Engine, Render or headless UI.
#[cfg(feature = "native")]
#[allow(unsafe_code)]
pub mod bridge;
