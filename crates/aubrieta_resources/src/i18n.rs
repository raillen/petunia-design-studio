//! Localization, string catalogs and message formatting (09.16).
//!
//! Enforces:
//! - No user-visible strings hardcoded in UI/feature logic.
//! - Namespaced `TextId` keys.
//! - Canonical `en-US` source catalog with synchronized `pt-BR` mirror.
//! - Automatic locale fallback (pt-BR -> en-US -> placeholder).
//! - Named parameter interpolation (`{param}`).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Stable namespaced identifier for user-visible strings.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TextId(pub String);

impl TextId {
    /// Creates a new `TextId`.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns a string slice of the ID.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for TextId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TextId(\"{}\")", self.0)
    }
}

impl fmt::Display for TextId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Supported locale identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Locale {
    /// American English (canonical source).
    EnUs,
    /// Brazilian Portuguese (mandatory release mirror).
    PtBr,
    /// Other custom BCP-47 locale tag.
    Custom(String),
}

impl Locale {
    /// Returns the canonical BCP-47 tag string.
    #[must_use]
    pub fn tag(&self) -> &str {
        match self {
            Self::EnUs => "en-US",
            Self::PtBr => "pt-BR",
            Self::Custom(c) => c.as_str(),
        }
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tag())
    }
}

/// Catalog of localized messages for a specific locale.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocaleCatalog {
    /// Associated locale.
    pub locale: Option<Locale>,
    /// Mappings from `TextId` to message template string.
    pub messages: HashMap<TextId, String>,
}

impl LocaleCatalog {
    /// Creates a new empty catalog.
    #[must_use]
    pub fn new(locale: Locale) -> Self {
        Self {
            locale: Some(locale),
            messages: HashMap::new(),
        }
    }

    /// Inserts a localized message.
    pub fn insert(&mut self, id: TextId, message: impl Into<String>) {
        self.messages.insert(id, message.into());
    }

    /// Retrieves a message by ID.
    #[must_use]
    pub fn get(&self, id: &TextId) -> Option<&str> {
        self.messages.get(id).map(String::as_str)
    }
}

/// Localization service managing multi-locale catalogs and fallback resolution.
#[derive(Debug, Clone, Default)]
pub struct LocalizationService {
    catalogs: HashMap<Locale, LocaleCatalog>,
}

impl LocalizationService {
    /// Creates an empty localization service.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a localization service preloaded with default built-in core strings (en-US & pt-BR).
    #[must_use]
    pub fn with_defaults() -> Self {
        let mut service = Self::new();
        service.load_builtin_strings();
        service
    }

    /// Registers or updates a catalog for a locale.
    pub fn register_catalog(&mut self, locale: Locale, catalog: LocaleCatalog) {
        self.catalogs.insert(locale, catalog);
    }

    /// Resolves a localized string with automatic fallback.
    ///
    /// Fallback order:
    /// 1. Requested locale
    /// 2. Canonical locale (`en-US`) if requested wasn't `en-US`
    /// 3. Placeholder: `[missing: {id}]`
    #[must_use]
    pub fn get(&self, id: &TextId, locale: &Locale) -> String {
        if let Some(cat) = self.catalogs.get(locale) {
            if let Some(msg) = cat.get(id) {
                return msg.to_string();
            }
        }

        // Fallback to canonical en-US if requested locale differed
        if *locale != Locale::EnUs {
            if let Some(en_cat) = self.catalogs.get(&Locale::EnUs) {
                if let Some(msg) = en_cat.get(id) {
                    return msg.to_string();
                }
            }
        }

        format!("[missing: {}]", id.0)
    }

    /// Formats a localized string with named parameters (e.g. `{count}`).
    #[must_use]
    pub fn format(&self, id: &TextId, locale: &Locale, params: &HashMap<String, String>) -> String {
        let template = self.get(id, locale);
        let mut result = template;
        for (key, val) in params {
            let placeholder = format!("{{{key}}}");
            result = result.replace(&placeholder, val);
        }
        result
    }

    /// Preloads built-in core translations.
    fn load_builtin_strings(&mut self) {
        let mut en = LocaleCatalog::new(Locale::EnUs);
        en.insert(TextId::new(ID_APP_NAME), "Aubrieta Design");
        en.insert(TextId::new(ID_ACTION_EXPORT), "Export");
        en.insert(TextId::new(ID_ACTION_SAVE), "Save");
        en.insert(TextId::new(ID_ACTION_UNDO), "Undo");
        en.insert(TextId::new(ID_ACTION_REDO), "Redo");
        en.insert(TextId::new(ID_TOOL_SELECT), "Select Tool");
        en.insert(TextId::new(ID_TOOL_PEN), "Pen Tool");
        en.insert(TextId::new(ID_TOOL_BRUSH), "Brush Tool");
        en.insert(TextId::new(ID_TOOL_TEXT), "Text Tool");
        en.insert(
            TextId::new(ID_EXPORT_SUMMARY),
            "Exported {count} items to {format} successfully",
        );

        let mut pt = LocaleCatalog::new(Locale::PtBr);
        pt.insert(TextId::new(ID_APP_NAME), "Aubrieta Design");
        pt.insert(TextId::new(ID_ACTION_EXPORT), "Exportar");
        pt.insert(TextId::new(ID_ACTION_SAVE), "Salvar");
        pt.insert(TextId::new(ID_ACTION_UNDO), "Desfazer");
        pt.insert(TextId::new(ID_ACTION_REDO), "Refazer");
        pt.insert(TextId::new(ID_TOOL_SELECT), "Ferramenta de Seleção");
        pt.insert(TextId::new(ID_TOOL_PEN), "Ferramenta Caneta");
        pt.insert(TextId::new(ID_TOOL_BRUSH), "Ferramenta Pincel");
        pt.insert(TextId::new(ID_TOOL_TEXT), "Ferramenta de Texto");
        pt.insert(
            TextId::new(ID_EXPORT_SUMMARY),
            "Exportados {count} itens para {format} com sucesso",
        );

        self.register_catalog(Locale::EnUs, en);
        self.register_catalog(Locale::PtBr, pt);
    }
}

// Canonical string ID constants
pub const ID_APP_NAME: &str = "aubrieta.app.name";
pub const ID_ACTION_EXPORT: &str = "aubrieta.action.export.title";
pub const ID_ACTION_SAVE: &str = "aubrieta.action.save.title";
pub const ID_ACTION_UNDO: &str = "aubrieta.action.undo.title";
pub const ID_ACTION_REDO: &str = "aubrieta.action.redo.title";
pub const ID_TOOL_SELECT: &str = "aubrieta.tool.select.title";
pub const ID_TOOL_PEN: &str = "aubrieta.tool.pen.title";
pub const ID_TOOL_BRUSH: &str = "aubrieta.tool.brush.title";
pub const ID_TOOL_TEXT: &str = "aubrieta.tool.text.title";
pub const ID_EXPORT_SUMMARY: &str = "aubrieta.export.summary.message";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_strings_resolve_correctly_for_en_and_pt() {
        let service = LocalizationService::with_defaults();
        let export_id = TextId::new(ID_ACTION_EXPORT);

        let en_str = service.get(&export_id, &Locale::EnUs);
        let pt_str = service.get(&export_id, &Locale::PtBr);

        assert_eq!(en_str, "Export");
        assert_eq!(pt_str, "Exportar");
    }

    #[test]
    fn fallback_to_en_us_when_key_missing_in_pt_br() {
        let mut service = LocalizationService::new();
        let mut en = LocaleCatalog::new(Locale::EnUs);
        let test_id = TextId::new("aubrieta.test.only_in_en");
        en.insert(test_id.clone(), "English exclusive");
        service.register_catalog(Locale::EnUs, en);

        let resolved = service.get(&test_id, &Locale::PtBr);
        assert_eq!(resolved, "English exclusive");
    }

    #[test]
    fn missing_key_returns_bracketed_placeholder() {
        let service = LocalizationService::new();
        let test_id = TextId::new("aubrieta.nonexistent");
        let resolved = service.get(&test_id, &Locale::EnUs);
        assert_eq!(resolved, "[missing: aubrieta.nonexistent]");
    }

    #[test]
    fn parameter_formatting() {
        let service = LocalizationService::with_defaults();
        let summary_id = TextId::new(ID_EXPORT_SUMMARY);

        let mut params = HashMap::new();
        params.insert("count".to_string(), "5".to_string());
        params.insert("format".to_string(), "SVG".to_string());

        let en = service.format(&summary_id, &Locale::EnUs, &params);
        let pt = service.format(&summary_id, &Locale::PtBr, &params);

        assert_eq!(en, "Exported 5 items to SVG successfully");
        assert_eq!(pt, "Exportados 5 itens para SVG com sucesso");
    }
}
