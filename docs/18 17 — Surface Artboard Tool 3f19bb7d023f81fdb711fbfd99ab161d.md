# 18.17 — Surface / Artboard Tool

# Identity

ToolId ptnd.tool.surface.

# Purpose

Create/select/move/resize Surfaces representing artboards/pages/export regions without conflating them with ordinary vector rectangles.

# Creation

Drag new Surface or choose preset. Shift/Alt constraints follow rectangle grammar. Context can create exact size/orientation.

# Selection

Surface handles differ visually from object selection. Clicking outside content label/border selects Surface when tool active.

# Move/resize

Policy toggle “move contents with Surface” explicit. Page-role Surface order metadata separate from spatial position.

# Context

role, preset, W/H, orientation, position, units, background/display, bleed, margins, export flag, content-follow toggle.

# Commands

CreateSurface, MoveSurface, ResizeSurface, DuplicateSurface, DeleteSurface, SetSurfaceProperties.

# Constraints

Deleting Surface with content requires policy: move content to pasteboard/other Surface or delete with explicit confirmation.

# Accessibility

Surface list/page panel provides keyboard selection/reorder and numeric geometry.

# Tests

Multiple Surfaces, page reorder vs position, move contents policy, export target, bleed, save/undo.