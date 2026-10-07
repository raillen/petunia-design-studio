# 07.4 — AI-Assisted Development Boundaries, Agent Safety & Implementation Reality

# Role

AI/code agents may implement, inspect and test Petunia, but they do not redefine canonical contracts without ADR/review.

# Grounding

Agent receives minimum authoritative microcontext: requirement, relevant Notion/repo docs, headers/schemas/tests and current code. It must inspect repository before asserting implementation state.

# No fabricated completion

A feature is implemented only with repository evidence/tests. Documentation target state is not proof code exists.

# Risk

Agents editing parser/security/build/release code require specialized reviewer roles. Generated code undergoes same static/dynamic tests.

# External research

Technology choices use current authoritative sources and benchmarks; agent cannot cite popularity as evidence.

# Secrets

Prompts/evidence must not expose private user artwork, credentials or environment secrets unnecessarily.

# Handoff

Every task records exact changed files, tests run/skipped, known limitations and next reviewer.