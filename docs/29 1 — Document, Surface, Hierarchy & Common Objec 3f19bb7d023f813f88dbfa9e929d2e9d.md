# 29.1 — Document, Surface, Hierarchy & Common Object Record Schemas

# Document

DocumentId, schema/capabilities, metadata, units, color context, Surface order, object/resource/style/story registries and extension namespace refs.

# Surface

SurfaceId, role, name, position/size, background/display, margins, bleed, columns, baseline grid, template/page metadata, root object IDs and export flags.

# Common ObjectRecord

ObjectId; ObjectTypeId; name; parent relation; visible; locked; transform; opacity; blend; appearance/masks where applicable; metadata/extension refs; creation semantic version if required.

# Ownership

Exactly one hierarchy parent/container for drawable object unless object type uses reference semantics (Symbol definition/resource). Parent/child graph acyclic. Surface roots define rendering containment.

# Transform

Canonical local-to-parent transform representation. Singular transforms allowed only if downstream semantics explicitly support; otherwise validation error. Units remain document coordinates.

# Lock/visibility

Inherited effective state is derived from ancestors; canonical per-object flags store local state only.

# Properties

Common PropertyIds map name, visible, locked, transform, opacity, blend; mutation always through Commands.

# Validation

Missing parent, multiple ownership, cycle, unknown required type, nonfinite transform/value, orphan root and illegal Surface relation produce stable ErrorCodes.