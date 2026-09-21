use petunia_design_extension::{PluginId, PluginManifest, PluginPermission};
use proptest::prelude::*;

proptest! {
    #[test]
    fn plugin_id_string_roundtrip(name in "[a-z0-9_.-]{3,30}") {
        let id = PluginId::new(&name);
        prop_assert_eq!(id.as_str(), name.as_str());
        prop_assert_eq!(id.to_string(), name);
    }

    #[test]
    fn manifest_serialization_preserves_declared_permissions(
        read in any::<bool>(),
        write in any::<bool>(),
        clip in any::<bool>(),
    ) {
        let id = PluginId::new("org.test.dynamic");
        let mut manifest = PluginManifest::new(id, "Dynamic", "0.1.0", "entry.lua");
        if read { manifest = manifest.with_permission(PluginPermission::DocumentRead); }
        if write { manifest = manifest.with_permission(PluginPermission::DocumentWrite); }
        if clip { manifest = manifest.with_permission(PluginPermission::Clipboard); }

        let json = serde_json::to_string(&manifest).unwrap();
        let restored: PluginManifest = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(manifest.permissions.contains(&PluginPermission::DocumentRead), read);
        prop_assert_eq!(restored.permissions.contains(&PluginPermission::DocumentRead), read);
        prop_assert_eq!(restored.permissions.contains(&PluginPermission::DocumentWrite), write);
        prop_assert_eq!(restored.permissions.contains(&PluginPermission::Clipboard), clip);
    }
}
