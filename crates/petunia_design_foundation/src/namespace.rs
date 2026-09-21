//! Public namespace identity and legacy read migration (15.A).
//!
//! New public IDs use the `ptnd.*` namespace. Resources, actions and color
//! tokens written by earlier builds used `aubrieta.*`; those are read
//! through [`normalize_legacy_namespace`] and are never emitted again.
//!
//! Contract (15.A): "old identifiers may be read through versioned migration
//! maps but are never emitted in new projects."

/// Current public namespace prefix for new identifiers.
pub const NAMESPACE: &str = "ptnd";

/// Legacy namespace prefix, accepted on read only.
pub const LEGACY_NAMESPACE: &str = "aubrieta";

/// Rewrites a legacy `aubrieta.*` identifier to `ptnd.*`.
///
/// Returns `None` when no rewrite is needed, so hot paths can avoid an
/// allocation for the common (already current) case.
#[must_use]
pub fn normalize_legacy_namespace(token: &str) -> Option<String> {
    let rest = token.strip_prefix(LEGACY_NAMESPACE)?;
    if !rest.starts_with('.') {
        return None;
    }
    Some(format!("{NAMESPACE}{rest}"))
}

/// Owned namespace-normalized form. Always returns a usable identifier.
#[must_use]
pub fn normalized(token: &str) -> String {
    normalize_legacy_namespace(token).unwrap_or_else(|| token.to_string())
}

/// True when the token already belongs to the current namespace.
#[must_use]
pub fn is_current_namespace(token: &str) -> bool {
    token
        .strip_prefix(NAMESPACE)
        .is_some_and(|rest| rest.starts_with('.'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_prefix_is_rewritten() {
        assert_eq!(
            normalize_legacy_namespace("aubrieta.blue/500").as_deref(),
            Some("ptnd.blue/500")
        );
        assert_eq!(
            normalize_legacy_namespace("aubrieta.edit.undo").as_deref(),
            Some("ptnd.edit.undo")
        );
    }

    #[test]
    fn current_and_unrelated_tokens_are_untouched() {
        assert_eq!(normalize_legacy_namespace("ptnd.blue/500"), None);
        assert_eq!(normalize_legacy_namespace("#ff0000"), None);
        assert_eq!(normalize_legacy_namespace("aubrietax.y"), None);
        assert_eq!(normalize_legacy_namespace("aubrieta"), None);
    }

    #[test]
    fn normalized_is_idempotent() {
        let once = normalized("aubrieta.red/500");
        assert_eq!(once, "ptnd.red/500");
        assert_eq!(normalized(&once), once);
        assert!(is_current_namespace(&once));
    }
}
