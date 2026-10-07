# 12.4 — Code-Agent Goal Protocol, Microcontext, ADR, Plan DAG & Handoff

# Goal packet

Every nontrivial implementation starts with:

- Goal statement;
- locked criteria;
- non-goals;
- affected authority pages;
- current repository evidence;
- risks;
- target commands/tests.

# Explorer phase

Explorer/project-intelligence maps entry points, existing contracts and drift. It must not broaden design by guessing.

# Architecture phase

Architect creates/updates ADR when boundary or locked technical decision changes. Plan DAG decomposes tasks with dependencies, owners, required skills and verification.

# Microcontext

Each implementer receives only authoritative slices: relevant docs, headers/schemas, nearby code, tests and acceptance. Avoid dumping whole notebook into model context.

# Implementation

Implementer/editor/renderer agents follow contracts. Any discovered contradiction becomes explicit issue/ADR request; do not silently redesign.

# Verification

Tester executes deterministic suite; quality reviewer checks maintainability; security reviewer independent for trust boundary; performance agent verifies claims from profiler; accessibility reviewer verifies UI.

# Handoff

Record revision, files, behavior, tests, remaining known limitations, evidence paths and next role. Never claim “done” without evidence required by Goal.

# Provider independence

Where high-risk, reviewer/verifier should be independent from implementation role/provider per Prumo contracts.

# Documentation

Documentation-maintainer updates canonical pages only with verified state; future design remains clearly labeled planned/spec, not implemented.