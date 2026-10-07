# 09.19 — Observability, Structured Diagnostics, Logging, Profiling & Crash Bundles

# Observability model

Structured events use stable EventCode, subsystem, severity, timestamps, session/document identifiers, ActionId/JobId where relevant and bounded metadata. Logs are not the semantic API.

# Privacy

Artwork text/pixels, file contents, auth credentials and unrestricted paths are excluded by default. User opt-in diagnostic bundle can include sanitized details.

# Python

Top-level exception hook, Qt message handler, application timing spans, slow event-loop detection and plugin/MCP transport diagnostics.

# C++

Structured logger sinks plus Tracy/perf/ETW/Instruments-compatible zones. Native exceptions/errors translated before Python boundary while retaining error code and trace correlation ID.

# Renderer

GPU debug labels/markers, frame graph timings, shader/backend error, device lost reason, VRAM/resource statistics.

# Jobs

Queue wait, start/end/cancel, progress phase, stale result discard and failure.

# Crash

Platform minidump/backtrace where available + BuildId + dependency/backend summary + last bounded events. Crash collection cannot attempt complex document mutation.

# Dev UI

Diagnostics panel filters by subsystem/severity; Copy Diagnostic includes stable code/help topic, not giant raw log.