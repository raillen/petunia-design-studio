//! Versioned UI preferences, separate from artwork and document history.
//!
//! Persona identifiers remain stable after renaming/reordering. Callers stage
//! customization in a clone, then validate and persist it before applying.

mod persistence;
mod shortcuts;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

pub use shortcuts::ShortcutBinding;

pub const PREFERENCES_VERSION: u32 = 1;
pub const MAX_PREFERENCE_BYTES: usize = 1_048_576;
pub const MAX_PERSONAS: usize = 128;
const MAX_KEYS: usize = 128;

pub type PreferenceResult<T> = std::result::Result<T, PreferenceError>;

#[derive(Debug, Error)]
pub enum PreferenceError {
    #[error("Cannot read or save UI preferences: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid UI preferences JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Unsupported UI preference version: {0}")]
    UnsupportedVersion(u32),
    #[error("UI preferences exceed the {MAX_PREFERENCE_BYTES}-byte limit")]
    TooLarge,
    #[error("Invalid UI preference: {0}")]
    Invalid(String),
    #[error("Unknown persona: {0}")]
    UnknownPersona(String),
    #[error("Activate another persona before hiding or removing the active persona")]
    ActivePersona,
    #[error("Keep at least one persona visible")]
    LastVisiblePersona,
    #[error("Built-in personas can be hidden or restored, but cannot be removed")]
    BuiltinPersona,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Appearance {
    Light,
    #[default]
    Dark,
    HighContrast,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IconFamily {
    #[default]
    Phosphor,
    Tabler,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IconStyle {
    #[default]
    Outline,
    Fill,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Density {
    #[default]
    Comfortable,
    Compact,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Persona {
    pub id: String,
    pub name: String,
    #[serde(rename = "icon")]
    pub icon_key: String,
    pub visible: bool,
    pub builtin: bool,
    /// Factory arrangement used by scoped restoration, including custom copies.
    pub template_id: String,
    pub panels: Vec<String>,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UiPreferences {
    pub version: u32,
    #[serde(rename = "activePersona")]
    pub active_persona_id: String,
    pub appearance: Appearance,
    pub icon_family: IconFamily,
    pub icon_style: IconStyle,
    pub density: Density,
    pub reduced_motion: bool,
    /// Scale typography and controls independently of document zoom and DPI.
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f64,
    pub personas: Vec<Persona>,
    /// Overrides only; omission in older v1 preference files retains defaults.
    #[serde(default)]
    pub shortcut_bindings: Vec<ShortcutBinding>,
}

/// Recovery never writes to the source file. The caller displays this diagnostic
/// and asks the user to apply/save preferences through the normal UI action.
#[derive(Debug)]
pub struct PreferenceRecovery {
    pub preferences: UiPreferences,
    pub diagnostic: Option<String>,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            version: PREFERENCES_VERSION,
            active_persona_id: "vector".into(),
            appearance: Appearance::default(),
            icon_family: IconFamily::default(),
            icon_style: IconStyle::default(),
            density: Density::default(),
            reduced_motion: false,
            ui_scale: default_ui_scale(),
            shortcut_bindings: Vec::new(),
            personas: ["vector", "pixel", "layout"]
                .into_iter()
                .filter_map(factory_persona)
                .collect(),
        }
    }
}

impl UiPreferences {
    pub fn active_persona(&self) -> PreferenceResult<&Persona> {
        self.persona(&self.active_persona_id)
    }

    pub fn persona(&self, id: &str) -> PreferenceResult<&Persona> {
        validate_identifier(id)?;
        self.personas
            .iter()
            .find(|persona| persona.id == id)
            .ok_or_else(|| PreferenceError::UnknownPersona(id.into()))
    }

    pub fn activate_persona(&mut self, id: &str) -> PreferenceResult<()> {
        self.validate()?;
        if !self.persona(id)?.visible {
            return Err(PreferenceError::Invalid(
                "Show the persona before activating it".into(),
            ));
        }
        self.active_persona_id = id.into();
        Ok(())
    }

    pub fn set_persona_visible(&mut self, id: &str, visible: bool) -> PreferenceResult<()> {
        self.validate()?;
        let index = self.persona_index(id)?;
        if !visible && self.personas[index].visible {
            if self
                .personas
                .iter()
                .filter(|persona| persona.visible)
                .count()
                == 1
            {
                return Err(PreferenceError::LastVisiblePersona);
            }
            if id == self.active_persona_id {
                return Err(PreferenceError::ActivePersona);
            }
        }
        self.personas[index].visible = visible;
        Ok(())
    }

    /// Creates a visible custom copy without changing the active activity.
    pub fn duplicate_persona(&mut self, id: &str, name: &str) -> PreferenceResult<String> {
        self.validate()?;
        validate_name(name)?;
        if self.personas.len() >= MAX_PERSONAS {
            return Err(PreferenceError::Invalid("Too many personas".into()));
        }
        let mut persona = self.persona(id)?.clone();
        let stem = format!("{}-copy", persona.template_id);
        let next_id = (1..=MAX_PERSONAS)
            .map(|number| format!("{stem}-{number}"))
            .find(|candidate| self.personas.iter().all(|item| item.id != *candidate))
            .ok_or_else(|| PreferenceError::Invalid("No available persona identifier".into()))?;
        persona.id.clone_from(&next_id);
        persona.name = name.trim().into();
        persona.builtin = false;
        persona.visible = true;
        self.personas.push(persona);
        Ok(next_id)
    }

    pub fn rename_persona(&mut self, id: &str, name: &str) -> PreferenceResult<()> {
        self.validate()?;
        validate_name(name)?;
        let index = self.persona_index(id)?;
        self.personas[index].name = name.trim().into();
        Ok(())
    }

    pub fn remove_persona(&mut self, id: &str) -> PreferenceResult<()> {
        self.validate()?;
        let index = self.persona_index(id)?;
        if self.personas[index].builtin {
            return Err(PreferenceError::BuiltinPersona);
        }
        if id == self.active_persona_id {
            return Err(PreferenceError::ActivePersona);
        }
        self.personas.remove(index);
        Ok(())
    }

    pub fn reorder_persona(&mut self, id: &str, destination: usize) -> PreferenceResult<()> {
        self.validate()?;
        let index = self.persona_index(id)?;
        if destination >= self.personas.len() {
            return Err(PreferenceError::Invalid(
                "Persona destination is outside the list".into(),
            ));
        }
        let persona = self.personas.remove(index);
        self.personas.insert(destination, persona);
        Ok(())
    }

    /// Restores only this profile's tools, panels and icon; retains its identity,
    /// name, visibility, order and the rest of the application's preferences.
    pub fn restore_persona(&mut self, id: &str) -> PreferenceResult<()> {
        self.validate()?;
        let index = self.persona_index(id)?;
        let baseline = factory_persona(&self.personas[index].template_id)
            .ok_or_else(|| PreferenceError::Invalid("Unknown persona template".into()))?;
        let persona = &mut self.personas[index];
        persona.icon_key = baseline.icon_key;
        persona.tools = baseline.tools;
        persona.panels = baseline.panels;
        Ok(())
    }

    /// Applies a staged edit atomically in memory. Unknown catalog keys are
    /// rejected so removed resources cannot become active controls.
    pub fn configure_persona(
        &mut self,
        id: &str,
        icon_key: &str,
        tools: Vec<String>,
        panels: Vec<String>,
    ) -> PreferenceResult<()> {
        self.validate()?;
        let mut staged = self.clone();
        let index = staged.persona_index(id)?;
        staged.personas[index].icon_key = icon_key.into();
        staged.personas[index].tools = tools;
        staged.personas[index].panels = panels;
        staged.validate()?;
        *self = staged;
        Ok(())
    }

    pub fn validate(&self) -> PreferenceResult<()> {
        if self.version != PREFERENCES_VERSION {
            return Err(PreferenceError::UnsupportedVersion(self.version));
        }
        if !self.ui_scale.is_finite() || !(1.0..=2.0).contains(&self.ui_scale) {
            return Err(PreferenceError::Invalid(
                "UI scale must be a finite number between 1.0 and 2.0".into(),
            ));
        }
        if self.personas.is_empty() || self.personas.len() > MAX_PERSONAS {
            return Err(PreferenceError::Invalid(
                "Persona count is outside the allowed range".into(),
            ));
        }
        validate_identifier(&self.active_persona_id)?;
        let mut ids = HashSet::new();
        for persona in &self.personas {
            validate_identifier(&persona.id)?;
            validate_name(&persona.name)?;
            if !ids.insert(&persona.id) {
                return Err(PreferenceError::Invalid(
                    "Duplicate persona identifier".into(),
                ));
            }
            if factory_persona(&persona.template_id).is_none() {
                return Err(PreferenceError::Invalid("Unknown persona template".into()));
            }
            let is_factory_id = matches!(persona.id.as_str(), "vector" | "pixel" | "layout");
            if persona.builtin != is_factory_id
                || (persona.builtin && persona.id != persona.template_id)
            {
                return Err(PreferenceError::Invalid(
                    "Invalid built-in persona identity".into(),
                ));
            }
            validate_catalog_keys(std::slice::from_ref(&persona.icon_key), ICON_KEYS)?;
            validate_catalog_keys(&persona.tools, TOOL_KEYS)?;
            validate_catalog_keys(&persona.panels, PANEL_KEYS)?;
        }
        if !self.personas.iter().any(|persona| persona.visible) {
            return Err(PreferenceError::LastVisiblePersona);
        }
        if !self.active_persona()?.visible {
            return Err(PreferenceError::Invalid(
                "Active persona must be visible".into(),
            ));
        }
        self.validate_shortcut_bindings()?;
        Ok(())
    }

    fn persona_index(&self, id: &str) -> PreferenceResult<usize> {
        validate_identifier(id)?;
        self.personas
            .iter()
            .position(|persona| persona.id == id)
            .ok_or_else(|| PreferenceError::UnknownPersona(id.into()))
    }
}

fn default_ui_scale() -> f64 {
    1.0
}

pub const ICON_KEYS: &[&str] = &["persona.vector", "persona.pixel", "persona.layout"];
pub const TOOL_KEYS: &[&str] = &[
    "tool.select",
    "tool.node",
    "tool.pen",
    "tool.brush",
    "tool.eraser",
    "tool.rectangle",
    "tool.ellipse",
    "tool.text",
    "tool.pan",
    "tool.zoom",
];
pub const PANEL_KEYS: &[&str] = &[
    "layers",
    "inspector",
    "color",
    "stroke",
    "assets",
    "history",
    "navigator",
    "brushes",
];

fn factory_persona(id: &str) -> Option<Persona> {
    let (name, tools, panels): (&str, &[&str], &[&str]) = match id {
        "vector" => (
            "Vetor",
            &[
                "tool.select",
                "tool.node",
                "tool.pen",
                "tool.rectangle",
                "tool.ellipse",
                "tool.text",
                "tool.pan",
                "tool.zoom",
            ],
            &["layers", "inspector", "color", "stroke", "history"],
        ),
        "pixel" => (
            "Pixel",
            &[
                "tool.select",
                "tool.brush",
                "tool.eraser",
                "tool.pan",
                "tool.zoom",
            ],
            &["layers", "inspector", "color", "brushes", "history"],
        ),
        "layout" => (
            "Layout",
            &[
                "tool.select",
                "tool.rectangle",
                "tool.ellipse",
                "tool.text",
                "tool.pan",
                "tool.zoom",
            ],
            &["layers", "inspector", "color", "assets", "history"],
        ),
        _ => return None,
    };
    Some(Persona {
        id: id.into(),
        name: name.into(),
        icon_key: format!("persona.{id}"),
        visible: true,
        builtin: true,
        template_id: id.into(),
        panels: panels.iter().map(|key| (*key).into()).collect(),
        tools: tools.iter().map(|key| (*key).into()).collect(),
    })
}

fn validate_name(name: &str) -> PreferenceResult<()> {
    if name.trim().is_empty() || name.chars().count() > 80 || name.chars().any(char::is_control) {
        return Err(PreferenceError::Invalid(
            "Persona name must contain 1–80 printable characters".into(),
        ));
    }
    Ok(())
}

fn validate_identifier(id: &str) -> PreferenceResult<()> {
    if id.is_empty()
        || id.len() > 80
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(PreferenceError::Invalid(
            "Malformed persona identifier".into(),
        ));
    }
    Ok(())
}

fn validate_catalog_keys(keys: &[String], catalog: &[&str]) -> PreferenceResult<()> {
    if keys.len() > MAX_KEYS {
        return Err(PreferenceError::Invalid("Resource list is too long".into()));
    }
    let mut unique = HashSet::new();
    for key in keys {
        if !catalog.contains(&key.as_str()) || !unique.insert(key) {
            let resource: String = key.chars().take(80).collect();
            return Err(PreferenceError::Invalid(format!(
                "Unknown or repeated resource: {resource}"
            )));
        }
    }
    Ok(())
}
