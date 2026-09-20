#![forbid(unsafe_code)]

//! Native `.aubrieta` package: open ZIP with readable `manifest.json`,
//! schema-versioned `document/document.json`, atomic save via temp + rename.
//!
//! Identity comes from media-type/manifest/schema, never the suffix. `.aubri`
//! is the accepted short alias. `.abrt` and `.pds` are rejected glossary
//! terms, reported as explicit errors.

mod package;
mod svg;

pub use package::{open_package, save_package, PackageManifest, MEDIA_TYPE};
pub use svg::{export_document_svg, export_path_d, parse_path_d};
