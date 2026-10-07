# 10.7.3 — Surface Templates / Master-like Content, Overrides & Page Number Fields

# Template

SurfaceTemplate is reusable definition containing layout settings and template object subtree.

# Application

Surface references TemplateId; template content appears as inherited instances/projections. Surface-specific override model must use stable template object IDs/properties.

# Overrides

Allow selected properties/text/image placeholders to override; structural detach/override only through explicit command. UI indicates inherited/overridden.

# Update

Editing template propagates to all non-overridden instances via derived/reference semantics, not copying every object silently.

# Page fields

Text can contain page number/page count/section fields evaluated from Surface sequence. Source token remains canonical, display text derived.

# Detach

Detach Template materializes local object copies with new IDs, preserving appearance.

# Tests

template update, local override, reapply/reset override, page reorder number update and save/reopen.