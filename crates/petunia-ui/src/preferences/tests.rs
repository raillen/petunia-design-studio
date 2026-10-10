use super::*;
use std::fs;
use std::path::PathBuf;

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new(name: &str) -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "petunia-preferences-{name}-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn file(&self) -> PathBuf {
        self.0.join("preferences.json")
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn defaults_roundtrip_and_serialized_contract() {
    let original = UiPreferences::default();
    original.validate().unwrap();
    let json = original.to_json().unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["activePersona"], "vector");
    assert_eq!(value["iconFamily"], "phosphor");
    assert_eq!(value["iconStyle"], "outline");
    assert_eq!(value["personas"][0]["icon"], "persona.vector");
    assert_eq!(UiPreferences::from_json(&json).unwrap(), original);
    assert_eq!(original.active_persona().unwrap().id, "vector");
}

#[test]
fn personas_preserve_active_and_visible_invariants() {
    let mut preferences = UiPreferences::default();
    let before = preferences.clone();
    assert!(matches!(
        preferences.set_persona_visible("vector", false),
        Err(PreferenceError::ActivePersona)
    ));
    assert_eq!(preferences, before);
    preferences.set_persona_visible("pixel", false).unwrap();
    preferences.set_persona_visible("layout", false).unwrap();
    assert!(matches!(
        preferences.set_persona_visible("vector", false),
        Err(PreferenceError::LastVisiblePersona)
    ));
    assert!(preferences.activate_persona("pixel").is_err());
    preferences.set_persona_visible("pixel", true).unwrap();
    preferences.activate_persona("pixel").unwrap();
    preferences.set_persona_visible("vector", false).unwrap();
    assert_eq!(preferences.active_persona().unwrap().id, "pixel");
    assert!(matches!(
        preferences.remove_persona("vector"),
        Err(PreferenceError::BuiltinPersona)
    ));
}

#[test]
fn duplicate_edit_reorder_restore_and_remove_are_scoped() {
    let mut preferences = UiPreferences {
        appearance: Appearance::HighContrast,
        ..UiPreferences::default()
    };
    let source = preferences.persona("vector").unwrap().clone();
    let id = preferences
        .duplicate_persona("vector", "Minha ilustração")
        .unwrap();
    assert_ne!(id, source.id);
    assert_eq!(preferences.active_persona_id, "vector");
    preferences.rename_persona(&id, " Ilustração ").unwrap();
    preferences
        .configure_persona(
            &id,
            "persona.layout",
            vec!["tool.pen".into()],
            vec!["history".into()],
        )
        .unwrap();
    preferences.reorder_persona(&id, 0).unwrap();
    preferences.restore_persona(&id).unwrap();
    let restored = &preferences.personas[0];
    assert_eq!(restored.id, id);
    assert_eq!(restored.name, "Ilustração");
    assert_eq!(restored.panels, source.panels);
    assert_eq!(restored.tools, source.tools);
    assert_eq!(restored.icon_key, source.icon_key);
    assert!(!restored.builtin);
    assert_eq!(preferences.appearance, Appearance::HighContrast);
    preferences.activate_persona(&id).unwrap();
    let before = preferences.clone();
    assert!(matches!(
        preferences.remove_persona(&id),
        Err(PreferenceError::ActivePersona)
    ));
    assert_eq!(preferences, before);
    preferences.activate_persona("vector").unwrap();
    preferences.remove_persona(&id).unwrap();
    assert!(preferences.persona(&id).is_err());
}

#[test]
fn invalid_edits_are_transactional() {
    let mut preferences = UiPreferences::default();
    let before = preferences.clone();
    assert!(preferences.rename_persona("vector", "\n").is_err());
    assert!(preferences.reorder_persona("vector", usize::MAX).is_err());
    assert!(preferences.duplicate_persona("unknown", "New").is_err());
    assert!(preferences
        .configure_persona(
            "vector",
            "missing.icon",
            vec!["tool.select".into()],
            vec!["layers".into()]
        )
        .is_err());
    assert!(preferences
        .configure_persona(
            "vector",
            "persona.vector",
            vec!["tool.unknown".into()],
            vec!["layers".into()]
        )
        .is_err());
    assert_eq!(preferences, before);
}

#[test]
fn shortcut_overrides_persist_and_reject_conflicts_before_changes() {
    use crate::shortcuts::{ActionId, KeyCombo};
    let mut preferences = UiPreferences::default();
    preferences
        .set_shortcut_binding(ActionId::SelectTool, KeyCombo::key("s"))
        .unwrap();
    let before = preferences.clone();
    assert!(preferences
        .set_shortcut_binding(ActionId::Undo, KeyCombo::key("s"))
        .is_err());
    assert!(preferences
        .set_shortcut_binding(ActionId::Undo, KeyCombo::key("S"))
        .is_err());
    assert!(preferences
        .set_shortcut_binding(ActionId::Undo, KeyCombo::key("alt"))
        .is_err());
    assert_eq!(preferences, before);
    let persisted = UiPreferences::from_json(&preferences.to_json().unwrap()).unwrap();
    assert_eq!(
        persisted
            .shortcut_table()
            .unwrap()
            .action_for(&KeyCombo::key("s")),
        Some(ActionId::SelectTool)
    );
    preferences
        .reset_shortcut_binding(ActionId::SelectTool)
        .unwrap();
    assert_eq!(
        preferences
            .shortcut_table()
            .unwrap()
            .action_for(&KeyCombo::key("v")),
        Some(ActionId::SelectTool)
    );
}

#[test]
fn shortcut_swap_loads_without_intermediate_conflict_and_reset_is_transactional() {
    use crate::shortcuts::{ActionId, KeyCombo};
    let mut preferences = UiPreferences {
        shortcut_bindings: vec![
            ShortcutBinding {
                action: ActionId::SelectTool,
                combo: KeyCombo::key("n"),
            },
            ShortcutBinding {
                action: ActionId::NodeTool,
                combo: KeyCombo::key("v"),
            },
        ],
        ..UiPreferences::default()
    };
    preferences.validate().unwrap();
    let table = preferences.shortcut_table().unwrap();
    assert_eq!(
        table.action_for(&KeyCombo::key("n")),
        Some(ActionId::SelectTool)
    );
    assert_eq!(
        table.action_for(&KeyCombo::key("v")),
        Some(ActionId::NodeTool)
    );
    let before = preferences.clone();
    assert!(preferences
        .reset_shortcut_binding(ActionId::SelectTool)
        .is_err());
    assert_eq!(preferences, before);
    preferences.reset_shortcut_bindings().unwrap();
    assert!(preferences.shortcut_bindings.is_empty());
    assert_eq!(
        preferences.shortcut_table().unwrap(),
        crate::shortcuts::ShortcutTable::defaults()
    );
}

#[test]
fn older_v1_files_without_shortcuts_keep_defaults() {
    let mut value = serde_json::to_value(UiPreferences::default()).unwrap();
    value.as_object_mut().unwrap().remove("shortcutBindings");
    let preferences = UiPreferences::from_json(&value.to_string()).unwrap();
    assert!(preferences.shortcut_bindings.is_empty());
    preferences.validate().unwrap();
}

#[test]
fn workspace_shortcuts_persist_and_detect_conflicts_against_document_actions() {
    use crate::shortcuts::{ActionId, KeyCombo};
    let mut preferences = UiPreferences::default();
    preferences
        .set_shortcut_binding(ActionId::FocusNext, KeyCombo::key("f7"))
        .unwrap();
    preferences
        .set_shortcut_binding(ActionId::CommandPalette, KeyCombo::key("l").with_ctrl())
        .unwrap();
    let before = preferences.clone();
    assert!(preferences
        .set_shortcut_binding(ActionId::Preferences, KeyCombo::key("z").with_ctrl())
        .is_err());
    assert_eq!(preferences, before);
    let loaded = UiPreferences::from_json(&preferences.to_json().unwrap()).unwrap();
    let table = loaded.shortcut_table().unwrap();
    assert_eq!(
        table.action_for(&KeyCombo::key("f7")),
        Some(ActionId::FocusNext)
    );
    assert_eq!(
        table.action_for(&KeyCombo::key("l").with_ctrl()),
        Some(ActionId::CommandPalette)
    );
    assert_eq!(
        table.action_for(&KeyCombo::key("f6").with_shift()),
        Some(ActionId::FocusPrevious)
    );
}

#[test]
fn ui_scale_roundtrips_and_older_v1_files_default_to_one() {
    let preferences = UiPreferences {
        ui_scale: 2.0,
        ..UiPreferences::default()
    };
    let json = preferences.to_json().unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["uiScale"], 2.0);
    assert_eq!(UiPreferences::from_json(&json).unwrap().ui_scale, 2.0);
    value.as_object_mut().unwrap().remove("uiScale");
    assert_eq!(
        UiPreferences::from_json(&value.to_string())
            .unwrap()
            .ui_scale,
        1.0
    );
}

#[test]
fn invalid_ui_scale_is_rejected_without_overwriting_original_preferences() {
    let directory = TestDirectory::new("ui-scale");
    let path = directory.file();
    let mut preferences = UiPreferences::default();
    preferences.save(&path).unwrap();
    let original = fs::read(&path).unwrap();
    for scale in [0.0, 0.99, 2.01, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        preferences.ui_scale = scale;
        assert!(preferences.validate().is_err());
        assert!(preferences.to_json().is_err());
        assert!(preferences.save(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
    }
    for scale in [1.0, 1.25, 1.5, 2.0] {
        preferences.ui_scale = scale;
        preferences.validate().unwrap();
    }
    let mut value = serde_json::to_value(UiPreferences::default()).unwrap();
    for scale in [0.9, 2.1] {
        value["uiScale"] = scale.into();
        assert!(UiPreferences::from_json(&value.to_string()).is_err());
    }
}

#[test]
fn corrupted_unknown_or_future_data_is_rejected() {
    let defaults = UiPreferences::default();
    assert!(UiPreferences::from_json("{").is_err());
    let mut value = serde_json::to_value(defaults).unwrap();
    value["version"] = 2.into();
    assert!(matches!(
        UiPreferences::from_json(&value.to_string()),
        Err(PreferenceError::UnsupportedVersion(2))
    ));
    value["version"] = 1.into();
    value["iconFamily"] = "unknown".into();
    assert!(UiPreferences::from_json(&value.to_string()).is_err());
    value["iconFamily"] = "tabler".into();
    value["activePersona"] = "removed".into();
    assert!(UiPreferences::from_json(&value.to_string()).is_err());
    value["activePersona"] = "vector".into();
    value["personas"][0]["id"] = "pixel".into();
    assert!(UiPreferences::from_json(&value.to_string()).is_err());
    assert!(matches!(
        UiPreferences::from_json(&" ".repeat(MAX_PREFERENCE_BYTES + 1)),
        Err(PreferenceError::TooLarge)
    ));
}

#[test]
fn saved_preferences_replace_atomically_and_load() {
    let directory = TestDirectory::new("roundtrip");
    let path = directory.file();
    let mut preferences = UiPreferences::default();
    preferences.save(&path).unwrap();
    preferences.icon_family = IconFamily::Tabler;
    preferences.icon_style = IconStyle::Fill;
    preferences.reduced_motion = true;
    preferences.activate_persona("pixel").unwrap();
    preferences.save(&path).unwrap();
    assert_eq!(UiPreferences::load(&path).unwrap(), preferences);
    assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 1);
}

#[test]
fn invalid_save_preserves_original_bytes() {
    let directory = TestDirectory::new("invalid-save");
    let path = directory.file();
    let mut preferences = UiPreferences::default();
    preferences.save(&path).unwrap();
    let original = fs::read(&path).unwrap();
    preferences.personas.clear();
    assert!(preferences.save(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
    assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 1);
}

#[test]
fn recovery_reports_corruption_without_rewriting_user_data() {
    let directory = TestDirectory::new("recovery");
    let path = directory.file();
    fs::write(&path, b"broken data").unwrap();
    let recovered = UiPreferences::load_recovering(&path);
    assert_eq!(recovered.preferences, UiPreferences::default());
    assert!(recovered.diagnostic.is_some());
    assert_eq!(fs::read(&path).unwrap(), b"broken data");
    let missing = UiPreferences::load_recovering(directory.0.join("missing.json"));
    assert!(missing.diagnostic.is_none());
    assert_eq!(missing.preferences, UiPreferences::default());
    assert!(UiPreferences::default()
        .save(directory.0.join("missing-folder/settings.json"))
        .is_err());
}

#[test]
fn oversized_file_is_bounded_and_preserved() {
    let directory = TestDirectory::new("oversized");
    let path = directory.file();
    fs::write(&path, vec![b' '; MAX_PREFERENCE_BYTES + 1]).unwrap();
    assert!(matches!(
        UiPreferences::load(&path),
        Err(PreferenceError::TooLarge)
    ));
    assert!(UiPreferences::load_recovering(&path).diagnostic.is_some());
    assert_eq!(
        fs::metadata(&path).unwrap().len(),
        MAX_PREFERENCE_BYTES as u64 + 1
    );
}

#[cfg(unix)]
#[test]
fn preference_symlinks_do_not_read_or_overwrite_targets() {
    use std::os::unix::fs::symlink;
    let directory = TestDirectory::new("symlink");
    let target = directory.0.join("real.json");
    let link = directory.file();
    fs::write(&target, b"valuable original").unwrap();
    symlink(&target, &link).unwrap();
    assert!(UiPreferences::load(&link).is_err());
    assert!(UiPreferences::default().save(&link).is_err());
    assert_eq!(fs::read(&target).unwrap(), b"valuable original");
    assert!(fs::symlink_metadata(&link)
        .unwrap()
        .file_type()
        .is_symlink());
}
