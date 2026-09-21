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

/// Action domains that moved from `ptnd.<domain>.*` to the canonical
/// `ptnd.action.<domain>.*` grammar mandated by 15.G.
const ACTION_DOMAINS: &[&str] = &["object", "edit", "fill", "surface"];

/// Rewrites a pre-grammar action id to its canonical form.
///
/// `ptnd.object.align` becomes `ptnd.action.object.align`. Ids that already
/// carry the `action` segment, ids in other namespaces (`ptnd.tool.*`, ...)
/// and unknown strings are returned unchanged. Read-only shim: canonical ids
/// are the only form ever written back out.
#[must_use]
pub fn normalize_action_id(action_id: &str) -> String {
    let Some(rest) = action_id.strip_prefix("ptnd.") else {
        return action_id.to_owned();
    };
    let Some((domain, _)) = rest.split_once('.') else {
        return action_id.to_owned();
    };
    if !ACTION_DOMAINS.contains(&domain) {
        return action_id.to_owned();
    }
    format!("{NAMESPACE}.action.{rest}")
}

/// True when the id already uses the canonical action grammar.
#[must_use]
pub fn is_canonical_action_id(action_id: &str) -> bool {
    action_id.starts_with("ptnd.action.")
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
    fn pre_grammar_actions_are_rewritten_once() {
        assert_eq!(normalize_action_id("ptnd.object.align"), "ptnd.action.object.align");
        assert_eq!(normalize_action_id("ptnd.edit.delete"), "ptnd.action.edit.delete");
        assert_eq!(normalize_action_id("ptnd.fill.set"), "ptnd.action.fill.set");
        assert_eq!(normalize_action_id("ptnd.surface.create"), "ptnd.action.surface.create");
        let canonical = normalize_action_id("ptnd.object.align");
        assert_eq!(normalize_action_id(&canonical), canonical);
        assert!(is_canonical_action_id(&canonical));
    }

    #[test]
    fn tools_and_foreign_ids_keep_their_namespace() {
        assert_eq!(normalize_action_id("ptnd.tool.pen"), "ptnd.tool.pen");
        assert_eq!(normalize_action_id("ptnd.panel.layers"), "ptnd.panel.layers");
        assert_eq!(normalize_action_id("aubrieta.object.align"), "aubrieta.object.align");
        assert_eq!(normalize_action_id("ptnd"), "ptnd");
        assert!(!is_canonical_action_id("ptnd.tool.pen"));
    }

    #[test]
    fn normalized_is_idempotent() {
        let once = normalized("aubrieta.red/500");
        assert_eq!(once, "ptnd.red/500");
        assert_eq!(normalized(&once), once);
        assert!(is_current_namespace(&once));
    }
}
