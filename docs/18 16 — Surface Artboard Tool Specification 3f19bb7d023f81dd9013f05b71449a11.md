# 18.16 — Surface / Artboard Tool Specification

# Identity

ToolId ptnd.tool.surface. Persona Design.

# Targets

Canonical Surface records rather than ordinary drawable objects.

# Creation

Drag creates Surface using current role/preset/defaults. Click may create default-size surface centered/anchored at pointer if product profile enables it.

# State machine

Idle -> SurfaceHover -> DragCreate | MoveSurface | ResizeSurface -> Commit/Cancel.

# Move contents policy

Context/modifier explicitly chooses Move Surface With Contents versus Surface Only. Page-role surfaces can impose sequential-layout restrictions while artboards can be freely positioned.

# Context

Role, preset, X/Y/W/H, orientation, units, margins, bleed, columns, background/display color, page metadata and export inclusion.

# Overlay

Surface label, outer bounds, bleed, margins, columns/baseline grid and resize handles.

# Commands

CreateSurface, MoveSurface, ResizeSurface, SetSurfaceProperties, DuplicateSurface, DeleteSurface.

# Accessibility

Surface list/panel provides keyboard selection/reorder and complete numeric editing.

# Tests

Multiple surfaces, page role, move-with-contents, page order, huge coordinates, guides/margins and export targets.