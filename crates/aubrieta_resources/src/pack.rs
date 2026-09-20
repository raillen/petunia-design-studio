//! Resource pack bundle, manifest and icon mapping (09.12).
//!
//! Bundles together:
//! - Manifest (`manifest.json` / metadata)
//! - Tokens registry with theme support
//! - Localization catalogs
//! - Semantic icon maps

use crate::i18n::{
    LocalizationService, ID_ACTION_EXPORT, ID_ACTION_SAVE, ID_ACTION_UNDO, ID_TOOL_BRUSH,
    ID_TOOL_PEN, ID_TOOL_SELECT, ID_TOOL_TEXT,
};
use crate::tokens::{ThemeMode, TokenEntry, TokenRegistry, TokenValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Semantic icon identifier.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IconId(pub String);

impl IconId {
    /// Creates a new `IconId`.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for IconId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IconId(\"{}\")", self.0)
    }
}

impl fmt::Display for IconId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Manifest describing a resource pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackManifest {
    /// Unique pack identifier (e.g. `aubrieta.core.pack`).
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Semantic version.
    pub version: String,
    /// Author / Organization.
    pub author: String,
    /// Pack description.
    pub description: String,
    /// List of supported BCP-47 locale tags.
    pub supported_locales: Vec<String>,
}

/// Map of semantic icons to SVG definitions or paths.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IconMap {
    icons: HashMap<IconId, String>,
}

impl IconMap {
    /// Creates an empty icon map.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts an icon definition.
    pub fn insert(&mut self, id: IconId, svg: impl Into<String>) {
        self.icons.insert(id, svg.into());
    }

    /// Retrieves an icon definition.
    #[must_use]
    pub fn get(&self, id: &IconId) -> Option<&str> {
        self.icons.get(id).map(String::as_str)
    }

    /// Number of icons.
    #[must_use]
    pub fn len(&self) -> usize {
        self.icons.len()
    }

    /// Checks if icon map is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.icons.is_empty()
    }
}

/// Complete resource pack bundling tokens, strings, icons, and metadata.
#[derive(Debug, Clone)]
pub struct ResourcePack {
    /// Pack metadata manifest.
    pub manifest: PackManifest,
    /// Design tokens registry.
    pub tokens: TokenRegistry,
    /// Localization service.
    pub localization: LocalizationService,
    /// Icon definitions map.
    pub icons: IconMap,
}

impl ResourcePack {
    /// Creates a new resource pack.
    pub fn new(
        manifest: PackManifest,
        tokens: TokenRegistry,
        localization: LocalizationService,
        icons: IconMap,
    ) -> Self {
        Self {
            manifest,
            tokens,
            localization,
            icons,
        }
    }

    /// Creates the standard canonical core resource pack.
    #[must_use]
    pub fn core_pack() -> Self {
        let manifest = PackManifest {
            id: "aubrieta.core.pack".to_string(),
            name: "Aubrieta Core Resources".to_string(),
            version: "0.1.0".to_string(),
            author: "Aubrieta Team".to_string(),
            description: "Default tokens, themes, strings and icons for Aubrieta Design"
                .to_string(),
            supported_locales: vec!["en-US".to_string(), "pt-BR".to_string()],
        };

        let mut tokens = TokenRegistry::new();
        // Primitives
        tokens.insert(TokenEntry::new(
            "palette.neutral.0",
            TokenValue::Color("#ffffff".to_string()),
        ));
        tokens.insert(TokenEntry::new(
            "palette.neutral.900",
            TokenValue::Color("#111827".to_string()),
        ));
        tokens.insert(TokenEntry::new(
            "palette.neutral.100",
            TokenValue::Color("#f3f4f6".to_string()),
        ));
        tokens.insert(TokenEntry::new(
            "palette.neutral.800",
            TokenValue::Color("#1f2937".to_string()),
        ));
        tokens.insert(TokenEntry::new(
            "palette.blue.500",
            TokenValue::Color("#3b82f6".to_string()),
        ));
        tokens.insert(TokenEntry::new(
            "palette.red.500",
            TokenValue::Color("#ef4444".to_string()),
        ));

        // Semantics (Light default)
        tokens.insert(TokenEntry::new(
            "surface.canvas",
            TokenValue::Alias("palette.neutral.0".to_string()),
        ));
        tokens.insert(TokenEntry::new(
            "text.primary",
            TokenValue::Alias("palette.neutral.900".to_string()),
        ));
        tokens.insert(TokenEntry::new(
            "border.subtle",
            TokenValue::Alias("palette.neutral.100".to_string()),
        ));
        tokens.insert(TokenEntry::new(
            "color.accent",
            TokenValue::Alias("palette.blue.500".to_string()),
        ));

        // Dark Theme Overrides
        tokens.insert_override(
            ThemeMode::Dark,
            TokenEntry::new(
                "surface.canvas",
                TokenValue::Alias("palette.neutral.900".to_string()),
            ),
        );
        tokens.insert_override(
            ThemeMode::Dark,
            TokenEntry::new(
                "text.primary",
                TokenValue::Alias("palette.neutral.0".to_string()),
            ),
        );
        tokens.insert_override(
            ThemeMode::Dark,
            TokenEntry::new(
                "border.subtle",
                TokenValue::Alias("palette.neutral.800".to_string()),
            ),
        );

        // Core Icons
        let mut icons = IconMap::new();
        icons.insert(
            IconId::new(ID_TOOL_SELECT),
            "<svg><path d=\"M2 2l7 18 3-7 7-3z\"/></svg>".to_string(),
        );
        icons.insert(
            IconId::new(ID_TOOL_PEN),
            "<svg><path d=\"M12 2l10 10-14 14H2v-6z\"/></svg>".to_string(),
        );
        icons.insert(
            IconId::new(ID_TOOL_BRUSH),
            "<svg><circle cx=\"12\" cy=\"12\" r=\"8\"/></svg>".to_string(),
        );
        icons.insert(
            IconId::new(ID_TOOL_TEXT),
            "<svg><path d=\"M4 4h16v4h-6v12h-4V8H4z\"/></svg>".to_string(),
        );
        icons.insert(
            IconId::new(ID_ACTION_EXPORT),
            "<svg><path d=\"M5 12h14M12 5l7 7-7 7\"/></svg>".to_string(),
        );
        icons.insert(
            IconId::new(ID_ACTION_SAVE),
            "<svg><path d=\"M19 21H5a2 2 0 01-2-2V5a2 2 0 012-2h11l5 5v11a2 2 0 01-2 2z\"/></svg>"
                .to_string(),
        );
        icons.insert(
            IconId::new(ID_ACTION_UNDO),
            "<svg><path d=\"M3 7v6h6M3 13a9 9 0 102.5-6.5L3 13\"/></svg>".to_string(),
        );

        let localization = LocalizationService::with_defaults();

        Self::new(manifest, tokens, localization, icons)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{Locale, TextId, ID_APP_NAME};

    #[test]
    fn core_pack_initializes_with_valid_resources() {
        let pack = ResourcePack::core_pack();
        assert_eq!(pack.manifest.id, "aubrieta.core.pack");

        // Token resolution
        let light_canvas = pack
            .tokens
            .resolve("surface.canvas", Some(ThemeMode::Light))
            .expect("light canvas");
        let dark_canvas = pack
            .tokens
            .resolve("surface.canvas", Some(ThemeMode::Dark))
            .expect("dark canvas");

        assert_eq!(light_canvas, TokenValue::Color("#ffffff".to_string()));
        assert_eq!(dark_canvas, TokenValue::Color("#111827".to_string()));

        // Localization
        let app_name = pack
            .localization
            .get(&TextId::new(ID_APP_NAME), &Locale::PtBr);
        assert_eq!(app_name, "Aubrieta Design");

        // Icons
        assert!(pack.icons.get(&IconId::new(ID_TOOL_SELECT)).is_some());
    }
}
