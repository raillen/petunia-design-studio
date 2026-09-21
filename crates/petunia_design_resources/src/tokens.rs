//! Design tokens implementation conforming to DTCG-aligned token architecture (09.12).
//!
//! Separates tokens into three levels:
//! - Primitive: Raw palette, spacing, and typography values (e.g. `palette.red.500`)
//! - Semantic: Purpose-driven tokens aliased to primitives (e.g. `surface.canvas`, `text.primary`)
//! - Component: Specific UI elements aliased to semantics (e.g. `button.primary.background`)
//!
//! Features transitive alias resolution with strict cycle detection.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Errors that can occur during design token resolution.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TokenError {
    /// Token key was not found in registry.
    #[error("token `{0}` not found")]
    TokenNotFound(String),

    /// Circular reference detected in alias chain.
    #[error("circular alias detected in token chain: {0:?}")]
    CircularAlias(Vec<String>),

    /// Exceeded maximum alias resolution depth.
    #[error("maximum alias resolution depth exceeded for token `{0}`")]
    MaxDepthExceeded(String),

    /// Token JSON parsing error.
    #[error("invalid token payload: {0}")]
    InvalidPayload(String),
}

/// Token value representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum TokenValue {
    /// Color hex or css string (e.g. `"#ff3366"` or `"rgba(255, 51, 102, 1.0)"`).
    Color(String),
    /// Numeric dimension with unit (e.g. `16.0, "px"`).
    Dimension { value: f64, unit: String },
    /// Scalar numeric value.
    Number(f64),
    /// Textual or font family value.
    String(String),
    /// Alias referencing another token by path (e.g. `"{palette.blue.500}"` or `"palette.blue.500"`).
    Alias(String),
}

impl TokenValue {
    /// Strips enclosing `{}` from an alias if present.
    #[must_use]
    pub fn normalize_alias_target(raw: &str) -> &str {
        let trimmed = raw.trim();
        if trimmed.starts_with('{') && trimmed.ends_with('}') && trimmed.len() >= 2 {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        }
    }
}

/// Token entry with metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenEntry {
    /// Name / path of the token (e.g. `color.brand.primary`).
    pub name: String,
    /// Token value (literal or alias).
    pub value: TokenValue,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
}

impl TokenEntry {
    /// Creates a new token entry.
    pub fn new(name: impl Into<String>, value: TokenValue) -> Self {
        Self {
            name: name.into(),
            value,
            description: None,
        }
    }

    /// Sets the description.
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Theme mode for token overrides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThemeMode {
    /// Light theme mode.
    Light,
    /// Dark theme mode.
    Dark,
}

/// Registry managing base tokens and theme overrides.
#[derive(Debug, Clone, Default)]
pub struct TokenRegistry {
    base_tokens: HashMap<String, TokenEntry>,
    dark_overrides: HashMap<String, TokenEntry>,
    light_overrides: HashMap<String, TokenEntry>,
}

impl TokenRegistry {
    /// Creates an empty token registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a base token.
    pub fn insert(&mut self, entry: TokenEntry) {
        self.base_tokens.insert(entry.name.clone(), entry);
    }

    /// Inserts a theme-specific override.
    pub fn insert_override(&mut self, theme: ThemeMode, entry: TokenEntry) {
        match theme {
            ThemeMode::Dark => {
                self.dark_overrides.insert(entry.name.clone(), entry);
            }
            ThemeMode::Light => {
                self.light_overrides.insert(entry.name.clone(), entry);
            }
        }
    }

    /// Returns the raw entry without resolving aliases, considering the active theme.
    #[must_use]
    pub fn get_raw(&self, name: &str, theme: Option<ThemeMode>) -> Option<&TokenEntry> {
        if let Some(t) = theme {
            let override_map = match t {
                ThemeMode::Dark => &self.dark_overrides,
                ThemeMode::Light => &self.light_overrides,
            };
            if let Some(entry) = override_map.get(name) {
                return Some(entry);
            }
        }
        self.base_tokens.get(name)
    }

    /// Resolves a token name to its final literal value with cycle detection.
    pub fn resolve(&self, name: &str, theme: Option<ThemeMode>) -> Result<TokenValue, TokenError> {
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        self.resolve_internal(name, theme, &mut visited, &mut path)
    }

    fn resolve_internal(
        &self,
        name: &str,
        theme: Option<ThemeMode>,
        visited: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Result<TokenValue, TokenError> {
        if path.len() > 32 {
            return Err(TokenError::MaxDepthExceeded(name.to_string()));
        }

        if !visited.insert(name.to_string()) {
            let mut cycle = path.clone();
            cycle.push(name.to_string());
            return Err(TokenError::CircularAlias(cycle));
        }
        path.push(name.to_string());

        let entry = self
            .get_raw(name, theme)
            .ok_or_else(|| TokenError::TokenNotFound(name.to_string()))?;

        let result = match &entry.value {
            TokenValue::Alias(target) => {
                let clean_target = TokenValue::normalize_alias_target(target);
                self.resolve_internal(clean_target, theme, visited, path)?
            }
            literal => literal.clone(),
        };

        path.pop();
        visited.remove(name);
        Ok(result)
    }

    /// Returns total number of base tokens.
    #[must_use]
    pub fn len(&self) -> usize {
        self.base_tokens.len()
    }

    /// Checks if registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.base_tokens.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_literal_resolution() {
        let mut registry = TokenRegistry::new();
        registry.insert(TokenEntry::new(
            "palette.red.500",
            TokenValue::Color("#ef4444".to_string()),
        ));

        let resolved = registry
            .resolve("palette.red.500", None)
            .expect("should resolve");
        assert_eq!(resolved, TokenValue::Color("#ef4444".to_string()));
    }

    #[test]
    fn transitive_alias_resolution() {
        let mut registry = TokenRegistry::new();
        registry.insert(TokenEntry::new(
            "palette.blue.600",
            TokenValue::Color("#2563eb".to_string()),
        ));
        registry.insert(TokenEntry::new(
            "color.brand.primary",
            TokenValue::Alias("{palette.blue.600}".to_string()),
        ));
        registry.insert(TokenEntry::new(
            "button.primary.bg",
            TokenValue::Alias("color.brand.primary".to_string()),
        ));

        let resolved = registry
            .resolve("button.primary.bg", None)
            .expect("should resolve through chain");
        assert_eq!(resolved, TokenValue::Color("#2563eb".to_string()));
    }

    #[test]
    fn circular_alias_detection() {
        let mut registry = TokenRegistry::new();
        registry.insert(TokenEntry::new(
            "token.a",
            TokenValue::Alias("token.b".to_string()),
        ));
        registry.insert(TokenEntry::new(
            "token.b",
            TokenValue::Alias("token.c".to_string()),
        ));
        registry.insert(TokenEntry::new(
            "token.c",
            TokenValue::Alias("token.a".to_string()),
        ));

        let err = registry.resolve("token.a", None).unwrap_err();
        match err {
            TokenError::CircularAlias(chain) => {
                assert!(chain.contains(&"token.a".to_string()));
                assert!(chain.contains(&"token.b".to_string()));
                assert!(chain.contains(&"token.c".to_string()));
            }
            other => panic!("expected CircularAlias, got {other:?}"),
        }
    }

    #[test]
    fn theme_override_resolution() {
        let mut registry = TokenRegistry::new();
        registry.insert(TokenEntry::new(
            "surface.canvas",
            TokenValue::Color("#ffffff".to_string()),
        ));
        registry.insert_override(
            ThemeMode::Dark,
            TokenEntry::new("surface.canvas", TokenValue::Color("#1e1e1e".to_string())),
        );

        let light = registry
            .resolve("surface.canvas", Some(ThemeMode::Light))
            .expect("light");
        let dark = registry
            .resolve("surface.canvas", Some(ThemeMode::Dark))
            .expect("dark");

        assert_eq!(light, TokenValue::Color("#ffffff".to_string()));
        assert_eq!(dark, TokenValue::Color("#1e1e1e".to_string()));
    }

    #[test]
    fn missing_token_returns_error() {
        let registry = TokenRegistry::new();
        let err = registry.resolve("nonexistent", None).unwrap_err();
        assert_eq!(err, TokenError::TokenNotFound("nonexistent".to_string()));
    }
}
