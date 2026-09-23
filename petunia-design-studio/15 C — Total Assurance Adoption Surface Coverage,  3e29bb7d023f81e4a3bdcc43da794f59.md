# 15.C — Total Assurance Adoption: Surface Coverage, Evidence, Blind Spots & 10/10 Gates

# Principle

Petunia Design Studio adopts the Prumo anti-false-green model. A feature is not complete because it compiles, renders or has a button.

# State ladder

documented → implemented → reachable → exercised → evidenced → verified → accepted → released.

# Surface Manifest

Inventory every tool, menu/submenu, toolbar action, context command, panel control, modal/popover/dropdown, shortcut/modifier, file action, plugin/MCP capability and recovery action. Each record includes stable ID, scope, preconditions, enabled/disabled rules, input, behavior, mutation, feedback, undo, persistence, failure, security, performance, accessibility, test, evidence and status.

# Evidence graph

Requirement → Acceptance → Surface → Implementation → Invariant → Test → Evidence → Verification → Regression → Documentation → Release Gate.

# Oracle register

Classify correctness oracle as exact, invariant, metamorphic, differential, structured human review, probabilistic or weak/unknown. Weak oracle cannot support high-risk PASS alone.

# Negative-space tests

Prove what must not happen: cancel must not mutate committed state; UI layout must not dirty the document; failed save must not clear dirty state; disabled action must not execute; denied plugin capability must not create a side effect.

# Blind-spot review

Every material subsystem asks which plausible failure class is absent from requirements. Lenses include state, persistence, concurrency, cancellation, performance, resource exhaustion, malformed input, security, compatibility, accessibility, localization, GPU/device, update/migration and long-session degradation.

# Final gate

10/10 is a contract state, not praise. Required fail=0, untested=0, unknown=0, stale evidence=0 and known unwaived defect=0. Scores can compare builds but never override hard gates.