# 19.19 — Resource Manager / Relink / Missing Fonts Dialog Specifications

# Resource Manager

Table ResourceId/name/type/link mode/status/path/source profile/usage count. Actions Relink, Embed, Unembed, Replace, Reveal, Collect.

# Relink

File picker grant; validate media/type/fingerprint compatibility; preview affected usage. Batch “find missing in folder” searches only granted directory and reports matches/ambiguities.

# Missing Fonts

Group requested faces with affected object/story count and current fallback. Replacement picker shows font metadata/coverage. Scope selected face/all compatible/style.

# Safety

Replacing resource/font is a document Command and undoable where storage cost permits. File access brokered.

# Performance

Usage count/query async for large docs; font list indexed.

# Tests

missing image, changed same-name file, ambiguous folder match, font substitution, undo and permission revoked.