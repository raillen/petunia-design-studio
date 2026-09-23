//! Native file dialog service contracts and headless mock adapter (09.17).

use crate::error::PlatformError;
use std::path::{Path, PathBuf};

/// File type filter specification for open and save dialogs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileFilter {
    /// Native Petunia Design Studio projects (`*.PTND`).
    PtndPackage,
    /// Scalable Vector Graphics (`*.svg`).
    SvgVector,
    /// Standard raster formats (`*.png`, `*.jpg`, `*.webp`, `*.tiff`).
    RasterImage,
    /// PDF document format (`*.pdf`).
    PdfDocument,
    /// Any file (`*.*`).
    AllFiles,
    /// Custom named filter with extensions.
    Custom {
        /// Name of the filter shown in the dialog.
        name: String,
        /// List of file extension strings without leading dot (e.g. `["json", "toml"]`).
        extensions: Vec<String>,
    },
}

impl FileFilter {
    /// Checks whether a given path matches this filter.
    #[must_use]
    pub fn matches(&self, path: &Path) -> bool {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        match self {
            Self::PtndPackage => ext.eq_ignore_ascii_case("ptnd"),
            Self::SvgVector => ext == "svg",
            Self::RasterImage => matches!(
                ext.as_str(),
                "png" | "jpg" | "jpeg" | "webp" | "tiff" | "tif"
            ),
            Self::PdfDocument => ext == "pdf",
            Self::AllFiles => true,
            Self::Custom { extensions, .. } => {
                extensions.iter().any(|e| e.eq_ignore_ascii_case(&ext))
            }
        }
    }
}

/// Abstract contract for platform file dialogs.
pub trait FileDialogService: Send + Sync {
    /// Shows open file dialog.
    fn open_file(&self, filter: &FileFilter) -> Result<Option<PathBuf>, PlatformError>;

    /// Shows save file dialog.
    fn save_file(
        &self,
        suggested_name: &str,
        filter: &FileFilter,
    ) -> Result<Option<PathBuf>, PlatformError>;

    /// Shows pick folder dialog.
    fn pick_folder(&self) -> Result<Option<PathBuf>, PlatformError>;
}

/// Headless / mock file dialog provider for testing and automation.
#[derive(Debug, Default, Clone)]
pub struct HeadlessFileDialog {
    preseeded_open: Option<PathBuf>,
    preseeded_save: Option<PathBuf>,
    preseeded_folder: Option<PathBuf>,
}

impl HeadlessFileDialog {
    /// Creates a headless file dialog provider with no preseeded values (returns None).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Pre-seeds the path returned by `open_file`.
    #[must_use]
    pub fn with_open_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.preseeded_open = Some(path.into());
        self
    }

    /// Pre-seeds the path returned by `save_file`.
    #[must_use]
    pub fn with_save_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.preseeded_save = Some(path.into());
        self
    }

    /// Pre-seeds the folder path returned by `pick_folder`.
    #[must_use]
    pub fn with_folder_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.preseeded_folder = Some(path.into());
        self
    }
}

impl FileDialogService for HeadlessFileDialog {
    fn open_file(&self, filter: &FileFilter) -> Result<Option<PathBuf>, PlatformError> {
        if let Some(ref path) = self.preseeded_open {
            if filter.matches(path) {
                return Ok(Some(path.clone()));
            }
        }
        Ok(None)
    }

    fn save_file(
        &self,
        _suggested_name: &str,
        _filter: &FileFilter,
    ) -> Result<Option<PathBuf>, PlatformError> {
        Ok(self.preseeded_save.clone())
    }

    fn pick_folder(&self) -> Result<Option<PathBuf>, PlatformError> {
        Ok(self.preseeded_folder.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_matching() {
        let pkg_filter = FileFilter::PtndPackage;
        assert!(pkg_filter.matches(Path::new("project.PTND")));
        assert!(pkg_filter.matches(Path::new("project.ptnd")));
        // Legacy suffixes are not offered by the picker; migration uses the
        // reader path in petunia_design_io.
        assert!(!pkg_filter.matches(Path::new("project.aubrieta")));
        assert!(!pkg_filter.matches(Path::new("project.svg")));

        let svg_filter = FileFilter::SvgVector;
        assert!(svg_filter.matches(Path::new("icon.svg")));
        assert!(!svg_filter.matches(Path::new("icon.png")));
    }

    #[test]
    fn headless_dialog_respects_filter() {
        let dialog = HeadlessFileDialog::new().with_open_path("/tmp/test.svg");
        let result_svg = dialog.open_file(&FileFilter::SvgVector).unwrap();
        assert_eq!(result_svg, Some(PathBuf::from("/tmp/test.svg")));

        let result_aubri = dialog.open_file(&FileFilter::PtndPackage).unwrap();
        assert_eq!(result_aubri, None);
    }
}
