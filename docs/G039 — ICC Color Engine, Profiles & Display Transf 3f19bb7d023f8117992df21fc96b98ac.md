# G039 — ICC Color Engine, Profiles & Display Transform Baseline

# Goal

Make color profile semantics real in document/render/import/export.

# Depends

Color 09.10/04.8, renderer.

# Primary

systems-architect + performance-agent.

# Deliverables

ColorValue/ColorSpace/Profile resources; LittleCMS adapter; assign/convert actions; transform cache; display-profile adapter; CPU oracle; renderer conversion hooks.

# Acceptance

Known RGB/Gray/CMYK profile fixtures convert/proof consistently and display transform changes do not mutate canonical color.

# Tests

Profile corpus, malformed ICC, monitor switch, conversion tolerances and cache.