//! Resource Packs, Design Tokens, Themes, Icons and Localization (09.12 & 09.16).
//!
//! Provides the core design resource layer for Petunia Design Studio:
//! - Design tokens conforming to DTCG with alias cycle detection.
//! - Multi-locale catalogs with mandatory `en-US` and `pt-BR` synchronization.
//! - Resource pack bundles and semantic icon maps.

pub mod i18n;
pub mod pack;
pub mod shell_strings;
pub mod tokens;

pub use i18n::{
    Locale, LocaleCatalog, LocalizationService, TextId, ID_ACTION_EXPORT, ID_ACTION_REDO,
    ID_ACTION_SAVE, ID_ACTION_UNDO, ID_APP_NAME, ID_EXPORT_SUMMARY, ID_TOOL_BRUSH, ID_TOOL_PEN,
    ID_TOOL_SELECT, ID_TOOL_TEXT,
};
pub use pack::{IconId, IconMap, PackManifest, ResourcePack};
pub use tokens::{ThemeMode, TokenEntry, TokenError, TokenRegistry, TokenValue};
