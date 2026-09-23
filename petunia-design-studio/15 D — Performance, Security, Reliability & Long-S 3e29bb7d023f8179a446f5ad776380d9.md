# 15.D — Performance, Security, Reliability & Long-Session Constitution

# Performance as correctness

For a creative desktop application, responsiveness and resource behavior are product requirements. Establish workloads and baselines before hard targets.

# Required performance dimensions

Startup/time-to-interactive, frame time and pacing, pointer-to-feedback latency, transform/Node/Pen latency, Layers scroll, docking drag, text typing, font menu, thumbnails, import/export, save/open, undo/redo, booleans, raster brush, histogram, RAM, VRAM, allocation rate, disk/cache growth and long-session degradation.

# Workload tiers

Maintain empty, small, typical, large and stress fixtures. Hardware tiers record CPU, RAM, GPU/VRAM, OS, graphics backend and driver. A benchmark without workload/environment metadata is invalid evidence.

# Endurance

Run representative loops: open → edit vector → photo adjustment → panel switch → undo/redo → import asset → save → export → close/reopen → repeat. Observe RAM, VRAM, handles, queues, caches and latency trend.

# Security

Treat .PTND, SVG, PDF, images, fonts, ICC profiles, external links, plugins, scripts and imported formats as trust boundaries. Test malformed, truncated, oversized and pathological inputs. Bound dimensions/counts before allocation where possible.

# Filesystem and integrity

Handle symlink, relative/absolute paths, safe temp files, permissions, read-only, disk full, atomic save, migration backup and missing links. Failed saves must preserve previous valid data when architecture permits.

# Resource exhaustion

Consider RAM, VRAM, file handles, workers, texture dimensions, layer counts, path/node counts and queues. Controlled refusal is preferable to host crash.

# Plugin/scripting

Capability-based permissions, filesystem/network/process scopes, API versioning, timeout/resource limits and crash containment. Lua/WASI cannot bypass Action/Command mutation.

# Release

Dependency audit, license/provenance, locked versions, artifact hashes/signing when available, sanitized diagnostics and no hidden telemetry/network.