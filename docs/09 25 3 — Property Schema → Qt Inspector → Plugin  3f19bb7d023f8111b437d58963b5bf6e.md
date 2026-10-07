# 09.25.3 — Property Schema → Qt Inspector → Plugin/MCP Schema Generation Pipeline

# Source

Property descriptors are immutable native/application metadata exposed through nanobind.

# Qt

PropertyEditorFactory maps descriptor to Petunia control; specialized editors registered by editorHint/type.

# Plugin

Declarative plugin panel can bind host PropertyId within granted context.

# MCP

properties.describe exposes descriptor subset; properties.set validates same native descriptor.

# Data Merge

Only bindable properties appear in binding picker. Type coercion derives from descriptor.

# Docs

Reference tables can generate from registry.

# Tests

Same invalid value rejected identically from Qt, plugin and MCP; schema drift detected in CI.