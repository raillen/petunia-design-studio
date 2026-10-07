# 09.24 — Application, Document Session, Multi-Window & Shutdown Lifecycle

# Application lifecycle

bootstrap -> load safe preferences -> initialize logging/platform -> native core -> registries -> Qt shell -> restore workspace/session -> ready.

# DocumentSession

Owns native document handle, history, jobs, resource watchers and semantic revision. Multiple DocumentViews can point to one session with independent viewport/selection.

# Open

Acquire -> parse/migrate/validate native -> create session -> create first view -> attach panels. UI never observes half-loaded mutable document.

# Close view

If last view and dirty session, prompt/save policy. Closing one duplicate view does not close document.

# Close application

Collect dirty sessions; user resolves; stop accepting new jobs/plugins/MCP writes; cancel/wait; persist workspace/settings; terminate plugin hosts; shutdown renderer/core.

# Background jobs

Export can optionally continue after closing view if session retained by job, but app shutdown policy must surface blocking jobs rather than orphan process silently.

# OS events

Open-file event, logout/shutdown, sleep/wake, GPU reset and display profile changes have adapter paths and bounded behavior.

# Crash

Recovery journal already contains checkpoints; normal shutdown marks clean session state.