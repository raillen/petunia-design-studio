# 09.12.5 — Export Naming, Multi-Output Transactions, Atomicity, Conflicts & Validation

# OutputPlan

Before encoding, resolve immutable list:

OutputTargetId, Surface/slice/source revision, exporter/preset, final destination grant/path, temp destination, expected size optional, conflict policy and degradation plan.

# Naming

Template tokens: document, Surface, slice, index, scale, record fields (Data Merge), suffix. Sanitizer removes traversal/reserved names and applies platform length rules. Preview lists every final filename.

# Conflicts

Ask, Overwrite, Skip, AutoRename, Fail. Batch “Ask” can collect all conflicts before starting where practical.

# Multi-output atomicity

Each output independently temp+validate+commit. Whole batch is not filesystem-atomic across many files; report explicitly marks partial success. Optional staging directory then directory swap only for controlled folder export feature.

# Revision

All outputs in one batch use pinned document snapshot unless user chooses per-record generated document semantics. Continued editing doesn't change mid-batch result.

# Validation

PNG/JPEG/etc decode header/full smoke as configured; SVG XML/schema structural checks; PDF parser/preflight validator where available. Validation happens before replacing existing target.

# Cancel

Cancel prevents future output commits and stops active encoder cooperatively; already committed outputs remain and report as completed. Temp partials cleaned.

# Retry

Retry failed/skipped target against same retained snapshot if still available, otherwise new export explicitly reports new revision.

# Tests

Unicode/reserved filenames, collisions, disk full, overwrite crash, cancellation, 1000 outputs, external modification and validation failure.