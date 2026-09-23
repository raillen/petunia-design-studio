use petunia_design_resources::{
    Locale, LocalizationService, TextId, TokenEntry, TokenError, TokenRegistry, TokenValue,
};
use proptest::prelude::*;
use std::collections::HashMap;

proptest! {
    #[test]
    fn single_step_alias_always_resolves(val in "[a-zA-Z0-9_#]{1,20}") {
        let mut registry = TokenRegistry::new();
        registry.insert(TokenEntry::new("target", TokenValue::String(val.clone())));
        registry.insert(TokenEntry::new("alias", TokenValue::Alias("target".to_string())));

        let resolved = registry.resolve("alias", None).unwrap();
        prop_assert_eq!(resolved, TokenValue::String(val));
    }

    #[test]
    fn self_referential_token_fails_cycle_check(name in "[a-z]{1,10}") {
        let mut registry = TokenRegistry::new();
        registry.insert(TokenEntry::new(&name, TokenValue::Alias(name.clone())));

        let err = registry.resolve(&name, None).unwrap_err();
        prop_assert!(matches!(err, TokenError::CircularAlias(_)));
    }

    #[test]
    fn message_param_interpolation_roundtrip(count in 0u32..1000) {
        let service = LocalizationService::with_defaults();
        let summary_id = TextId::new(petunia_design_resources::ID_EXPORT_SUMMARY);

        let mut params = HashMap::new();
        params.insert("count".to_string(), count.to_string());
        params.insert("format".to_string(), "PDF".to_string());

        let formatted = service.format(&summary_id, &Locale::PtBr, &params);
        prop_assert!(formatted.contains(&count.to_string()));
        prop_assert!(formatted.contains("PDF"));
    }
}
