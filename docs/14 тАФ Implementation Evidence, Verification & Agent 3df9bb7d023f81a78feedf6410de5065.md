# 14 — Implementation Evidence, Verification & AgentOps Atlas

<aside>
🧪

**Canonical evidence atlas.** Sections 08–10 specify what Aubrieta must look like, mean and do. Section 14 specifies what evidence is required before humans or code agents may claim that implementation conforms to those contracts.

</aside>

# Purpose

Aubrieta treats correctness, usability, performance, security, compatibility and modularity as **provable product properties**. A feature is not complete because it compiles, looks plausible or passes a happy-path test.

# Evidence hierarchy

For an implementation claim, prefer:

1. deterministic contract/conformance tests;
2. property tests and invariant checks;
3. fixture-based integration or headless E2E;
4. semantic UI/accessibility snapshots;
5. fuzz/security corpus evidence;
6. benchmark/performance traces;
7. visual goldens where pixels matter;
8. structured human usability evidence where automation cannot establish quality.

Screenshots alone cannot prove interaction correctness. Scores alone cannot replace raw evidence.

# Proven state

A work package may use the state **Proven for Milestone** only when all applicable evidence classes are either passing or explicitly Not Applicable with a documented reason. Deferred evidence needs a formal scope/dependency reason and may block release even when code exists.

# Evidence identity

Stable identifiers use semantic namespaces such as:

AUB-TEST-*, AUB-FIX-*, AUB-PERF-*, AUB-FUZZ-*, AUB-UI-*, AUB-MIG-*, AUB-SEC- *and AUB-CONF-*.

IDs survive file moves and appear in implementation dossiers, CI reports and release evidence.

# Agent contract

Code agents must state which evidence IDs support each completion claim. If a new behavior has no appropriate fixture/test/benchmark, creating that evidence is part of the change.

# Relationship to Prumo

Prumo organizes Goals, Waves, dossiers and Gauntlet Loops. It does not redefine Aubrieta evidence. Section 14 provides the Aubrieta-specific proof contracts consumed by Prumo workflows.

# Child atlas

14.1 defines test architecture and conformance.

14.2 defines hardware tiers and benchmark policy.

14.3 defines the Golden Project/Fixture Corpus.

14.4 proves modular detach and dependency boundaries.

14.5 covers security/fuzz/hostile-input evidence.

14.6 covers native-format migration/corruption/recovery.

14.7 turns Interface Atlas into UI/UX evidence.

14.8 defines deterministic AgentOps/CI commands and evidence bundles.

# No evidence laundering

Do not weaken thresholds, remove failing fixtures, update visual baselines blindly or rewrite acceptance criteria merely to make a Gauntlet score rise. A changed expectation requires a legitimate specification/ADR change first.

[14.1 — Test Architecture, Conformance Matrix & Regression Policy](14%201%20%E2%80%94%20Test%20Architecture,%20Conformance%20Matrix%20&%20Reg%203df9bb7d023f81708697efc517887807.md)

[14.2 — Performance Contract, Hardware Tiers, Benchmark Corpus & Regression Gates](14%202%20%E2%80%94%20Performance%20Contract,%20Hardware%20Tiers,%20Bench%203df9bb7d023f812e8de5e0c0328093ee.md)

[14.3 — Golden Project, Fixture & Reproduction Corpus](14%203%20%E2%80%94%20Golden%20Project,%20Fixture%20&%20Reproduction%20Corp%203df9bb7d023f81e7894fd7bc8dbe5141.md)

[14.4 — Capability Detach, Feature Isolation & Dependency Conformance](14%204%20%E2%80%94%20Capability%20Detach,%20Feature%20Isolation%20&%20Depe%203df9bb7d023f819c8cc0efa23f7a2a0e.md)

[14.5 — Security Verification, Fuzzing & Hostile-Input Evidence](14%205%20%E2%80%94%20Security%20Verification,%20Fuzzing%20&%20Hostile-In%203df9bb7d023f8141850fdbebb6adbd67.md)

[14.6 — Native Format Migration, Corruption, Atomic Save & Recovery Torture Suite](14%206%20%E2%80%94%20Native%20Format%20Migration,%20Corruption,%20Atomic%203df9bb7d023f811fa42ef99f802cda98.md)

[14.7 — UI/UX Evidence Integration, Semantic Goldens & Affinity Workflow Comparisons](14%207%20%E2%80%94%20UI%20UX%20Evidence%20Integration,%20Semantic%20Golden%203df9bb7d023f81219888de43696f641d.md)

[14.8 — AgentOps, cargo xtask, CI Commands, Evidence Bundles & Deterministic Handoff](14%208%20%E2%80%94%20AgentOps,%20cargo%20xtask,%20CI%20Commands,%20Evidenc%203df9bb7d023f814daef3ebc28510d779.md)