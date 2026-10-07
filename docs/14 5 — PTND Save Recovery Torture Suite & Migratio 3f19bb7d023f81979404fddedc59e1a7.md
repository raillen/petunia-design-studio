# 14.5 — .PTND Save/Recovery Torture Suite & Migration Evidence

# Save faults

Disk full, permission revoked, process kill between phases, fsync/rename failure simulation, destination replaced externally, network/removable drive semantics.

# Package faults

Missing manifest, duplicate paths, traversal, huge ratio, wrong hash, missing resource, corrupt tile, unknown optional extension, unknown required capability.

# Migration

Every historical schema fixture opens -> migrates -> validates -> saves current -> reopens -> semantic comparison. Stepwise migration versions separately tested.

# Atomicity

At every injected interruption, either previous valid destination or new valid destination remains; never half-overwritten canonical file.

# Recovery

Dirty edits -> checkpoints -> crash -> candidate discovery -> open recovery unsaved -> Save As. Discard removes journal only after explicit confirmation.

# Salvage

Corrupt source remains untouched. Salvage output clearly identifies lost/recovered resources and never claims full fidelity when unavailable.

# Third-party readability

Open format specification fixtures can be inspected by reference unpack/validator CLI independent from GUI.