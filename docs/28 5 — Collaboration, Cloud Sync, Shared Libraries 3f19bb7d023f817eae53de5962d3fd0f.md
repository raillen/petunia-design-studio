# 28.5 — Collaboration, Cloud Sync, Shared Libraries & Multi-User Editing Roadmap

# Local-first

Canonical PTND and core editing remain fully local/offline.

# Cloud file sync

Treat as storage adapter with version/conflict semantics; not equivalent to real-time collaboration.

# Shared libraries

Can sync versioned ResourceLibrary packages with identity/conflict/provenance and offline cache.

# Real-time collaboration

Separate research: operation/CRDT/OT semantics, presence, permissions, asset transfer, text/vector/raster conflict resolution, history and security. Existing Command model is useful but not assumed sufficient.

# Comments/review

Could arrive before real-time co-editing as separate annotation/review model.

# Accounts

No account required for local V1. Any account/cloud service must preserve export/ownership/offline escape path.

# Security

End-to-end/privacy/data residency and organization controls become new architecture domain if collaboration is promoted.

# Status

Research/Post-V1; never a hidden requirement for core V1.