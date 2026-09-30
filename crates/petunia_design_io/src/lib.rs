#![forbid(unsafe_code)]

//! Native `.PTND` package: open ZIP with readable `manifest.json`,
//! schema-versioned `document/document.json`, atomic save via temp + rename.
//!
//! Identity comes from media-type/manifest/schema, never the suffix. `.aubri`
//! is the accepted short alias. `.abrt` and `.pds` are rejected glossary
//! terms, reported as explicit errors.

pub mod image_io;
mod package;
pub mod pdf;
mod svg;

pub use image_io::{
    export_raster, import_raster, RasterExportOptions, RasterFormat, RawRasterImage,
};
pub use package::{
    has_native_extension, open_package, save_package, with_native_extension, OpenedPackage,
    PackageFormat, PackageManifest, MEDIA_TYPE, NATIVE_EXTENSION_DISPLAY, NATIVE_SUFFIX,
    SCHEMA_NAMESPACE,
};
pub use pdf::{
    export_document_pdf, DegradationItem, FidelityGrade, PdfExportOptions, PreflightReport,
};
pub use petunia_design_raster::PixelFormat;
pub use svg::{export_document_svg, export_path_d, parse_path_d};
