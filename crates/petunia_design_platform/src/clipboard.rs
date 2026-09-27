//! Platform clipboard abstractions and in-memory headless implementation (09.17).

use crate::error::PlatformError;
use serde::{Deserialize, Serialize};

/// High-level clipboard payload types supported by Petunia.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClipboardContent {
    /// Plain text string.
    Text(String),
    /// SVG vector markup.
    Svg(String),
    /// Raw RGBA8 bitmap.
    ImageRgba8 {
        /// Width in pixels.
        width: u32,
        /// Height in pixels.
        height: u32,
        /// Raw pixel bytes (length must equal width * height * 4).
        data: Vec<u8>,
    },
    /// Internal serialized document slice / fragment.
    DocumentFragment(Vec<u8>),
}

/// Abstract contract for clipboard access.
pub trait ClipboardService: Send + Sync {
    /// Reads plain text if available.
    fn get_text(&self) -> Result<Option<String>, PlatformError>;

    /// Sets plain text.
    fn set_text(&mut self, text: &str) -> Result<(), PlatformError>;

    /// Reads structured content if available.
    fn get_content(&self) -> Result<Option<ClipboardContent>, PlatformError>;

    /// Sets structured content.
    fn set_content(&mut self, content: ClipboardContent) -> Result<(), PlatformError>;

    /// Clears the clipboard.
    fn clear(&mut self) -> Result<(), PlatformError>;
}

/// Deterministic in-memory clipboard for headless tests, CI and CLI workflows.
#[derive(Debug, Default, Clone)]
pub struct HeadlessClipboard {
    content: Option<ClipboardContent>,
}

impl HeadlessClipboard {
    /// Creates a new empty headless clipboard.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl ClipboardService for HeadlessClipboard {
    fn get_text(&self) -> Result<Option<String>, PlatformError> {
        match &self.content {
            Some(ClipboardContent::Text(t)) => Ok(Some(t.clone())),
            Some(ClipboardContent::Svg(s)) => Ok(Some(s.clone())),
            _ => Ok(None),
        }
    }

    fn set_text(&mut self, text: &str) -> Result<(), PlatformError> {
        self.content = Some(ClipboardContent::Text(text.to_string()));
        Ok(())
    }

    fn get_content(&self) -> Result<Option<ClipboardContent>, PlatformError> {
        Ok(self.content.clone())
    }

    fn set_content(&mut self, content: ClipboardContent) -> Result<(), PlatformError> {
        self.content = Some(content);
        Ok(())
    }

    fn clear(&mut self) -> Result<(), PlatformError> {
        self.content = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_roundtrip() {
        let mut cb = HeadlessClipboard::new();
        assert!(cb.get_text().unwrap().is_none());

        cb.set_text("Hello Petunia").unwrap();
        assert_eq!(cb.get_text().unwrap().as_deref(), Some("Hello Petunia"));

        cb.clear().unwrap();
        assert!(cb.get_text().unwrap().is_none());
    }

    #[test]
    fn structured_image_content() {
        let mut cb = HeadlessClipboard::new();
        let img = ClipboardContent::ImageRgba8 {
            width: 2,
            height: 2,
            data: vec![255; 16],
        };

        cb.set_content(img.clone()).unwrap();
        assert_eq!(cb.get_content().unwrap(), Some(img));
        assert!(cb.get_text().unwrap().is_none());
    }
}
