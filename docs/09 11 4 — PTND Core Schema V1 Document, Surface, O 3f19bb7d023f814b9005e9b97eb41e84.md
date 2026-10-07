# 09.11.4 — PTND Core Schema V1: Document, Surface, Object Union, Appearance & Resources

# Purpose

This is the semantic blueprint for the actual JSON Schema files under schemas/ptnd/v1. Field names may only change through schema review before implementation lock.

# DocumentV1

```
format: "ptnd.document"
documentId: DocumentId
schemaVersion: 1
metadata:
  title?
  author?
  createdAt?
  modifiedAt?
units:
  documentUnit
color:
  workingSpaceRef
  bitDepthPolicy
surfaces: SurfaceId[]
objects: map<ObjectId,ObjectRecord>
resources: map<ResourceId,ResourceRecord>
styles: map<StyleId,StyleRecord>
symbols: map<SymbolId,SymbolDefinition>
stories: map<StoryId,TextStoryRecord>
dataSources/bindings
extensionRefs
root/pasteboard metadata
```

# SurfaceRecord

id, role (artboard|page|exportRegion|generic), name, position, size, transform if permitted, background, bleed, margins, columns, baselineGrid, rootObjectIds, templateRef, page metadata, export flags.

# Object tagged union

Common fields:

id, type, name?, parent/container relation, transform, visible, locked, opacity, blendMode, appearance?, masks?, metadata?, extensionRefs?.

Variants:

Group, VectorPath, ParametricShape, RasterLayer, TextObject, PlacedResource, Adjustment, LiveFilter, LiveBoolean, SymbolInstance, ClipGroup and future types via namespaced tag.

# Group

children[], isolationMode/passThrough semantics.

# VectorPath

VectorPathData reference/inline contours + appearance.

# ParametricShape

shapeKind + versioned parameter object + appearance. Unknown required kind triggers capability handling.

# RasterLayer

RasterResourceId/tileSetRef, pixelExtent, transform, pixelFormat descriptor, source profile ref, optional originalResource link.

# TextObject

StoryId + text mode Artistic|Frame|Path; frame geometry/path relation; start/end offsets; style defaults; flow link IDs.

# Adjustment/Filter

kind ID + parameter object + mask refs + evaluation scope semantics.

# Appearance

ordered entries each with EntryId + type Fill|Stroke|Effect + typed payload; object opacity/blend separately canonical.

# Referential integrity

Every referenced ID must exist and match expected type. Parent/child cycles forbidden. Surface roots own reachable objects according containment model.

# JSON rules

UTF-8, finite numbers, explicit enums, no duplicate object keys after Unicode/JSON parser interpretation, bounded nesting/string/array lengths. Unknown optional fields preservation policy versioned.

# Implementation

Generate C++/Python validation DTO metadata from schemas where practical, but domain classes are not generic JSON dictionaries.

# Tests

Minimal valid, maximal representative, each union variant, missing refs, cycles, unknown optional/required types, duplicate IDs and migration fixtures.