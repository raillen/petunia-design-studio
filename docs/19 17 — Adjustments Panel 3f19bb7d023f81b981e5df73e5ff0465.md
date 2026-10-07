# 19.17 — Adjustments Panel

# Identity

PanelId ptnd.panel.adjustments.

# Browser

Searchable categorized list of AdjustmentDescriptors with icon, name, short explanation and availability by color mode/bit depth.

# Add

Click creates live AdjustmentNode at contextually valid layer position; optional presets submenu. Newly added node becomes active in Properties/adjustment editor.

# Current adjustment

Panel can switch to editor mode or defer full parameters to Properties; architecture chooses one consistent pattern. Reset/disable/delete and mask actions available.

# Presets

Adjustment presets are versioned ParameterPreset resources; preview-on-hover optional only if performant/cancellable.

# Unsupported

Unavailable adjustment remains discoverable with reason when useful (e.g., unsupported CMYK semantics), not silently absent.

# Commands

AddAdjustment, SetAdjustmentParams, ToggleAdjustment, DeleteAdjustment, SaveAdjustmentPreset.

# Accessibility

Search/list and parameter controls fully keyboard accessible.

# Tests

Insertion position, different color modes, preset migration, mask creation, undo and stale preview.