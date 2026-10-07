# 22.4 — Object, Character & Paragraph Style Libraries

# StyleResource

StyleId, domain, schemaVersion, name, tags, parentStyleId optional, property map keyed stable PropertyId, exclusions/inheritance metadata.

# Domains

ObjectStyle controls appearance/effects and approved geometry-independent properties.

CharacterStyle controls text-run properties.

ParagraphStyle controls paragraph/layout properties.

# Inheritance

Acyclic style graph. Computed style derived; local overrides on objects/runs separate.

# Library use

Applying library style can import/copy into document with new/document StyleId while preserving provenance. Optional linked external styles are Post-V1 unless consistency strategy defined.

# Conflicts

Same source style version change never silently redefines document style. Update action presents affected object count/properties.

# Packaging

StylePack can include swatches/fonts references/assets only through declared dependencies and legal embedding policies.

# Tests

Inheritance, circular import rejection, missing dependency, update preview, local overrides and migration.