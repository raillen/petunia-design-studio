# 19.21 — Recovery, Corruption & Compatibility Dialog Specifications

# Recovery

List candidate document, original path, explicit save time, recovery time/revision and safe thumbnail. Actions Open Recovery, Discard, Reveal Original.

# Open

Creates unsaved recovered session; never overwrites original.

# Corruption

Structured report groups missing/corrupt entries and salvageable state. Actions Open Read-only, Attempt Salvage to New File, Cancel depending safety.

# Compatibility

Newer required capability dialog names missing capability/plugin/version and options Install/Update Provider, Open Read-only fallback if valid, Cancel.

# Data safety

No dialog offers “Repair in place” by default. Salvage always writes a new file.

# Tests

stale recovery vs newer explicit save, corrupt tile, unknown capability, discard confirmation and keyboard access.