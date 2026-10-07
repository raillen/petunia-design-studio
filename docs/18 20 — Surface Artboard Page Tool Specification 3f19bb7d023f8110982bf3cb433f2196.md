# 18.20 — Surface / Artboard / Page Tool Specification

# Identity

ToolId ptnd.tool.surface.

# Creation

Drag creates Surface with role Artboard/Page according current mode; presets/numeric creation also available.

# Selection

Tool selects Surfaces independently from artwork. Handles resize; drag moves spatial artboard. Sequential Page mode may restrict spatial move and emphasize order.

# Move contents

Context toggle “Move contents with Surface” explicit. Drag preview shows which objects are scoped/moving.

# Properties

size, position, orientation, role, background, bleed, margins, columns, export inclusion and template.

# Duplication

Alt/Option drag duplicates Surface and contents according explicit duplicate action.

# Commands

CreateSurface, ResizeSurface, MoveSurface, DuplicateSurface, DeleteSurface, SetSurfaceRole.

# Tests

Multiple artboards, page order, content move on/off, bleed, templates and export targeting.