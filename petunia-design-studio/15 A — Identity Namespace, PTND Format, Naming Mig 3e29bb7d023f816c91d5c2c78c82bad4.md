# 15.A — Identity Namespace, .PTND Format, Naming Migration & Legacy Compatibility

# Brand identity contract

**Petunia Design Studio** is always written in full in product titles, installers, About, documentation title pages and first mention. **Petunia Design** may be used as a short conversational form after the full name is established. Do not use "Petunia Studio" as a separate product identity because it becomes ambiguous inside the Petunia family.

# Native extension

- canonical display and write suffix: **.PTND**;
- opening may accept **.ptnd** as a case-tolerant spelling without creating a second format;
- legacy .aubrieta and .aubri are accepted only by the migration reader when legacy fixtures exist;
- Save As defaults to .PTND;
- file picker description: **Petunia Design Studio Project (*.PTND)**;
- prevent duplicated suffixes such as name.PTND.PTND;
- extension handling must never silently overwrite an unrelated file.

# Internal format identity

- format family: petunia-design-studio;
- suggested media type: application/vnd.petunia-design-studio.project+zip;
- suggested schema namespace: ptnd;
- manifest records format identity plus schema version;
- future format revisions use schema migration, not arbitrary new suffixes.

# Public namespace migration

- aubrieta. *resource/action IDs → ptnd.*;
- aubrieta_ *Rust crates → petunia_design_*;
- AubrietaGuiBridge → PetuniaDesignGuiBridge;
- aubrieta-cli → petunia-design;
- old identifiers may be read through versioned migration maps but are never emitted in new projects.

# Migration rules

Distinguish product text, code symbols, persisted IDs, file suffixes, plugin/API namespaces and historical prose. Persisted identifiers require real migrations; historical ADRs may keep old names only with a clear former-name marker.

# Compatibility

A legacy project converted to PTND must preserve semantic document data, stable references and resources. Destructive migration requires recoverable backup. The only legacy copy must never be overwritten silently.

# Search audit

Before RC, search repository/docs for Aubrieta, aubrieta, .aubri, .aubrieta, AubrietaGuiBridge, old MIME IDs, old resource namespaces and GPUI-primary claims. Classify every hit ACTIVE-MIGRATE, LEGACY-COMPAT, HISTORICAL-ALLOWED or FALSE-POSITIVE.