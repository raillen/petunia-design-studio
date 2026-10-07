# Context Optimization

## Purpose
Optimize token consumption and filter out redundant context payload.

## Use when
- Active Goal or Task explicitly requires context optimization operations.
- Operating in mode(s): `implementation`.

## Do not use when
- Task is out of scope or unrelated to context optimization.
- Bounded token budget or capability policy denies required operations.

## Required context
- Task requirements
- System architecture
- Relevant source files

## Procedure
1. Initialize minimal context for context optimization.
2. Execute scoped operations adhering to protocol standards.
3. Verify correctness with automated test gates.
4. Record task intelligence and evidence.
5. Garbage-collect transient state.

## Decision rules
- Always prefer smallest sufficient changeset over broad refactoring.
- Preserve existing public contracts and architecture invariants.
- Never weaken locked Goal acceptance criteria.

## Evidence required
- test

## Output contract
- Implementation / verification output
- Evidence record

## Stop conditions
- Task acceptance criteria satisfied with evidence
- Token budget exhausted
- Blocked on external dependency

## Escalation rules
- Escalate to lead architect or human if locked Goal criteria cannot be met.
- Escalate immediately upon discovering unexpected security vulnerabilities or data loss risks.
