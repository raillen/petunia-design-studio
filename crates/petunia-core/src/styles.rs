//! Shared styles: reusable appearance, character and paragraph intent.
//!
//! A style reuses properties under a stable identity; it is never
//! deduplicated by visual equality. Objects stay linked, detach
//! explicitly, and overrides stay typed. There is no inheritance
//! between styles in v0.1: linked style plus typed overrides covers
//! the case without cycles or cascading surprises.

use crate::appearance::Appearance;
use crate::error::{CoreError, Result};
use crate::id::{GridId, StyleId};
use crate::paint::ColorSource;
use crate::text::FontRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The three semantic style families. Each uses `StyleId`, but the
/// expected kind stays explicit wherever a style is referenced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StyleKind {
    Appearance,
    Character,
    Paragraph,
}

/// Shared appearance payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppearanceStyle {
    pub appearance: Appearance,
}

/// Shared character payload: font intent, metrics and paint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterStyle {
    pub font: FontRef,
    pub size: f64,
    pub color: ColorSource,
    pub tracking: f64,
    pub baseline_shift: f64,
    pub language: String,
    pub slant: FontSlant,
    pub features: Vec<String>,
}

/// Upright, italic or oblique face selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FontSlant {
    #[default]
    Normal,
    Italic,
    Oblique,
}

impl CharacterStyle {
    /// Size must be finite and positive; tracking and baseline shift
    /// finite; language named.
    pub fn new(
        font: FontRef,
        size: f64,
        color: ColorSource,
        tracking: f64,
        baseline_shift: f64,
        language: impl Into<String>,
    ) -> Result<Self> {
        if !size.is_finite() || size <= 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "invalid type size rejected: {size}"
            )));
        }
        for (label, value) in [("tracking", tracking), ("baseline shift", baseline_shift)] {
            if !value.is_finite() {
                return Err(CoreError::InvariantViolation(format!(
                    "non-finite {label} rejected: {value}"
                )));
            }
        }
        let language = language.into();
        if language.trim().is_empty() {
            return Err(CoreError::InvariantViolation(
                "character style needs a language".to_string(),
            ));
        }
        Ok(Self {
            font,
            size,
            color,
            tracking,
            baseline_shift,
            language,
            slant: FontSlant::Normal,
            features: Vec::new(),
        })
    }
}

/// Text alignment vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TextAlignment {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

/// Shared paragraph payload: spacing, indents and policies. Line
/// layout itself stays derived.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParagraphStyle {
    pub alignment: TextAlignment,
    pub line_height: f64,
    pub space_before: f64,
    pub space_after: f64,
    pub first_line_indent: f64,
    pub left_indent: f64,
    pub right_indent: f64,
    pub hyphenation: bool,
    pub baseline_grid: Option<GridId>,
}

impl ParagraphStyle {
    /// Line height must be finite and positive; spacing and indents
    /// finite (negative indents express hanging layouts).
    pub fn new(
        alignment: TextAlignment,
        line_height: f64,
        space_before: f64,
        space_after: f64,
    ) -> Result<Self> {
        if !line_height.is_finite() || line_height <= 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "invalid line height rejected: {line_height}"
            )));
        }
        for (label, value) in [("space before", space_before), ("space after", space_after)] {
            if !value.is_finite() || value < 0.0 {
                return Err(CoreError::InvariantViolation(format!(
                    "invalid {label} rejected: {value}"
                )));
            }
        }
        Ok(Self {
            alignment,
            line_height,
            space_before,
            space_after,
            first_line_indent: 0.0,
            left_indent: 0.0,
            right_indent: 0.0,
            hyphenation: false,
            baseline_grid: None,
        })
    }
}

/// One registry entry: identity plus a single-kind payload. Untyped
/// payloads are rejected even though families share the registry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StyleDefinition {
    Appearance(AppearanceStyle),
    Character(CharacterStyle),
    Paragraph(ParagraphStyle),
}

impl StyleDefinition {
    /// Which family this definition belongs to.
    #[must_use]
    pub fn kind(&self) -> StyleKind {
        match self {
            Self::Appearance(_) => StyleKind::Appearance,
            Self::Character(_) => StyleKind::Character,
            Self::Paragraph(_) => StyleKind::Paragraph,
        }
    }
}

/// Lookup by `StyleId`. Presentation order, when needed, lives in a
/// separate ordered collection: map iteration never defines it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StyleRegistry {
    styles: HashMap<StyleId, StyleDefinition>,
}

impl StyleRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            styles: HashMap::new(),
        }
    }

    /// Insert or replace the definition for a fresh or existing ID.
    pub fn insert(&mut self, id: StyleId, definition: StyleDefinition) {
        self.styles.insert(id, definition);
    }

    /// Look up a definition by identity.
    #[must_use]
    pub fn get(&self, id: StyleId) -> Option<&StyleDefinition> {
        self.styles.get(&id)
    }

    /// Number of tracked definitions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.styles.len()
    }

    /// True when no definition is tracked.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.styles.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{BuiltinColorSpace, ColorSpaceRef, ColorValue, ProcessColor};
    use crate::color::{ProcessColorValue, Rgba};

    fn paint() -> ColorSource {
        ColorSource::Value(ColorValue::Process(ProcessColor {
            value: ProcessColorValue::Rgb(Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                alpha: 1.0,
            }),
            space: ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
        }))
    }

    fn font() -> FontRef {
        FontRef::new("Inter", None, None, Vec::new()).expect("valid test font")
    }

    #[test]
    fn character_style_validates_metrics() {
        assert!(CharacterStyle::new(font(), 12.0, paint(), 0.0, 0.0, "en").is_ok());
        assert!(CharacterStyle::new(font(), 0.0, paint(), 0.0, 0.0, "en").is_err());
        assert!(CharacterStyle::new(font(), 12.0, paint(), 0.0, 0.0, "  ").is_err());
    }

    #[test]
    fn registry_keeps_identities_distinct() {
        let mut registry = StyleRegistry::new();
        let first = StyleId::new_v4();
        let second = StyleId::new_v4();
        let definition = StyleDefinition::Appearance(AppearanceStyle {
            appearance: Appearance { items: Vec::new() },
        });
        registry.insert(first, definition.clone());
        registry.insert(second, definition);
        assert_eq!(registry.len(), 2);
        assert_eq!(
            registry.get(first).expect("present").kind(),
            StyleKind::Appearance
        );
    }

    #[test]
    fn paragraph_style_validates_spacing() {
        assert!(ParagraphStyle::new(TextAlignment::Justify, 14.0, 6.0, 6.0).is_ok());
        assert!(ParagraphStyle::new(TextAlignment::Left, 0.0, 0.0, 0.0).is_err());
        assert!(ParagraphStyle::new(TextAlignment::Left, 14.0, -1.0, 0.0).is_err());
    }
}
