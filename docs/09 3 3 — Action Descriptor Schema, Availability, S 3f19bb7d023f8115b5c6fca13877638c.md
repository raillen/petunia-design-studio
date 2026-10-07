# 09.3.3 — Action Descriptor Schema, Availability, Shortcuts & Command Mapping

# ActionDescriptor

ActionId, TextId title/description, category, IconId, default shortcuts/profile, parameter schema, context requirements, availability evaluator, required permissions/capabilities, execution strategy and HelpId.

# Action vs command

Action may activate a tool/view mode, gather UI/file input, invoke one Command, build a transaction or start a Job whose result commits Commands. Command itself has no localized title/shortcut.

# Availability

Returns enabled, visible, checked and disabledReasonCode+params based on active session, selection capabilities, Persona/tool, permissions and modal state.

# Parameters

Simple actions expose typed schema to command palette/MCP/plugins. Complex UI gathers a typed request then calls same service.

# Shortcut

KeyChord is metadata profile; user override independent.

# Tests

Menu/shortcut/palette/MCP helper invoke same semantic path; disabled reason stable; locale change never changes identity.