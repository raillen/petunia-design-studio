# 22.3 — Styles, Symbols & Asset Library Contracts

# ObjectStyle

Typed property bundle over allowed appearance/object PropertyIds with optional parent style. Inheritance cycle forbidden.

# Character/Paragraph styles

Store typography/paragraph property subsets with inheritance and local override model. Deleting referenced style requires replacement/detach policy.

# Symbols

Library symbol can contain serialized document fragment + resource dependency manifest + preview. Placing into document chooses Copy Definition or Link External only if external-link semantics supported.

# Assets

Arbitrary reusable document fragment with bounds, insertion origin, resource dependencies, tags and thumbnail.

# Portability

Embedding into PTND remaps IDs while preserving internal references. Resource dedup uses fingerprints but does not collapse mutable semantic identity incorrectly.

# Upgrade

External asset/symbol update can compare version/fingerprint and offer update with preview/conflict; never auto-rewrite customized instances silently.

# Tests

ID remap, dependency bundle, style inheritance, symbol overrides, missing external library and asset update conflicts.