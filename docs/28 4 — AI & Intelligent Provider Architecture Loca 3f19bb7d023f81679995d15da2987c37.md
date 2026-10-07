# 28.4 — AI & Intelligent Provider Architecture: Local Models, Network Providers, Privacy & Consent

# Principle

AI is provider capability, not hard dependency of document model or UI.

# Candidate capabilities

Background/object selection, semantic mask, denoise/upscale, generative fill/remove, style/reference generation, text assistance and asset tagging/search.

# Provider classes

BuiltInLocal, PluginLocal, PluginNetwork/Remote. Each declares data types consumed, hardware/model requirements, network hosts, privacy statement and deterministic/non-deterministic result behavior.

# Permission

Remote image/document upload requires explicit network/data permission and user-visible provider identity. No silent fallback from local to remote.

# Job semantics

Provider runs against immutable input snapshot, returns staged pixels/masks/objects/resources, then ordinary transaction commits. Source/result provenance can be stored in metadata only as privacy/product policy allows.

# Reproducibility

Nondeterministic AI operation stores resulting resource, not assumption that rerunning prompt reproduces output. Undo never needs provider/network.

# Failure

Model missing, provider offline, timeout, quota or unsafe result leaves document unchanged and gives structured recovery.

# Status

Research/Plugin Territory until product requirements and legal/privacy model are approved.