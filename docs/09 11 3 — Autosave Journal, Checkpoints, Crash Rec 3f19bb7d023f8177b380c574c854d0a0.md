# 09.11.3 — Autosave Journal, Checkpoints, Crash Recovery, Atomic Replace & Salvage

# Journal separation

Explicit save and recovery are independent. Recovery data lives app recovery storage keyed by DocumentId/session/source fingerprint, not continuously rewriting user .PTND.

# Checkpoint strategy

Periodic canonical snapshot + incremental command/tile deltas candidate; final policy benchmarked for size/recovery speed. Journal records base explicit-save fingerprint and monotonically increasing revision.

# Trigger

Time interval + meaningful dirty activity + idle/bounded background. Brush pointer move does not fsync every sample.

# Failure

Disk quota/permission failure surfaces one persistent warning and diagnostics; editing continues; dirty state remains.

# Startup discovery

Compare clean-shutdown marker, explicit source fingerprint, recovery revision/time. Present candidate list, never auto-overwrite source.

# Open Recovery

Reconstruct in temp/session, validate, show as recovered unsaved document. User chooses Save/Save As.

# Atomic explicit save

Snapshot revision -> write sibling temp -> flush/fsync policy -> reopen/validate -> atomic replace -> directory sync if supported -> record saved revision/source fingerprint.

# Destination race

If target changed after save began, conflict policy prevents blind overwrite unless user explicitly approves.

# Salvage

Read bounded valid package parts, record missing/corrupt entries, reconstruct only provably valid model subset, write new recovered PTND. Original unchanged.

# Tests

Kill/fault injection at each save phase; journal truncation; stale recovery; source changed; disk full; concurrent save request coalescing.