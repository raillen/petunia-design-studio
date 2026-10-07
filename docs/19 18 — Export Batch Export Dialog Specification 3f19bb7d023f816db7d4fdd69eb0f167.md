# 19.18 — Export / Batch Export Dialog Specification

# Export

DialogId ptnd.dialog.export. Typed ExportRequest builder with format/preset, target area, size/scale, rasterization, color/profile, metadata, fidelity policy and destination grant.

# Dynamic controls

Exporter schema controls visible/enabled options. Switching format preserves only compatible values and never carries invalid hidden option.

# Preview/preflight

Degradation/issues panel updates async from snapshot. Selecting issue navigates/highlights if dialog nonmodal/preview-capable.

# Destination

File/folder grant selected through native dialog. Multi-output naming preview lists collisions before start.

# Start

Validate request → snapshot revision → Export Job. Dialog can close while job continues if background workflow supports; Job center owns progress.

# Batch

Table rows target/preset/path/status; conflict policy; retry failed; cancel stops pending and safely handles in-progress temp.

# Tests

Format switch, stale doc revision, destination collision, cancel, output validation failure and keyboard accessibility.