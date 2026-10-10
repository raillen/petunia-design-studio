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
use std::collections::BTreeMap;

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
        let candidate = Self {
            font,
            size,
            color,
            tracking,
            baseline_shift,
            language: language.into(),
            slant: FontSlant::Normal,
            features: Vec::new(),
        };
        candidate.validate()?;
        Ok(candidate)
    }

    /// Re-check the construction invariants on any instance,
    /// including deserialized ones.
    pub fn validate(&self) -> Result<()> {
        if !self.size.is_finite() || self.size <= 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "invalid type size rejected: {}",
                self.size
            )));
        }
        for (label, value) in [
            ("tracking", self.tracking),
            ("baseline shift", self.baseline_shift),
        ] {
            if !value.is_finite() {
                return Err(CoreError::InvariantViolation(format!(
                    "non-finite {label} rejected: {value}"
                )));
            }
        }
        if self.language.trim().is_empty() {
            return Err(CoreError::InvariantViolation(
                "character style needs a language".to_string(),
            ));
        }
        self.font.validate()
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
        let candidate = Self {
            alignment,
            line_height,
            space_before,
            space_after,
            first_line_indent: 0.0,
            left_indent: 0.0,
            right_indent: 0.0,
            hyphenation: false,
            baseline_grid: None,
        };
        candidate.validate()?;
        Ok(candidate)
    }

    /// Re-check the construction invariants on any instance,
    /// including deserialized ones.
    pub fn validate(&self) -> Result<()> {
        if !self.line_height.is_finite() || self.line_height <= 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "invalid line height rejected: {}",
                self.line_height
            )));
        }
        for (label, value) in [
            ("space before", self.space_before),
            ("space after", self.space_after),
            ("first-line indent", self.first_line_indent),
            ("left indent", self.left_indent),
            ("right indent", self.right_indent),
        ] {
            if !value.is_finite() {
                return Err(CoreError::InvariantViolation(format!(
                    "non-finite {label} rejected: {value}"
                )));
            }
            if (label == "space before" || label == "space after") && value < 0.0 {
                return Err(CoreError::InvariantViolation(format!(
                    "invalid {label} rejected: {value}"
                )));
            }
        }
        Ok(())
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
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StyleRegistry {
    #[serde(deserialize_with = "crate::serialization::deserialize_unique_btree_map")]
    styles: BTreeMap<StyleId, StyleDefinition>,
}

impl StyleRegistry {
    /// Remove one record by identity; callers validate remaining references.
    pub fn remove(&mut self, id: StyleId) -> Option<StyleDefinition> {
        self.styles.remove(&id)
    }

    #[must_use]
    pub fn new() -> Self {
        Self {
            styles: BTreeMap::new(),
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

    /// Iterate definitions in deterministic order. Iteration order is
    /// a traversal convenience for validation and tooling; it never
    /// defines presentation or authorial order.
    pub fn iter(&self) -> impl Iterator<Item = (StyleId, &StyleDefinition)> {
        self.styles.iter().map(|(id, definition)| (*id, definition))
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
