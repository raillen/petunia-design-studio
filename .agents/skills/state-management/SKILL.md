# State Management

## Purpose
Design predictable, unidirectional frontend and backend state machines.

## Use when
- Active Goal or Task explicitly requires state management operations.
- Operating in mode(s): `implementation`.

## Do not use when
- Task is out of scope or unrelated to state management.
- Bounded token budget or capability policy denies required operations.

## Required context
- API specifications
- Domain models
- UI wireframes

## Procedure
1. Review contracts and interface definitions for state management.
2. Implement business logic with strict boundary validation.
3. Maintain explicit error handling and status code mapping.
4. Run contract verification and integration tests.
5. Record evidence and update API documentation.

## Decision rules
- Always prefer smallest sufficient changeset over broad refactoring.
- Preserve existing public contracts and architecture invariants.
- Never weaken locked Goal acceptance criteria.

## Evidence required
- test

## Output contract
- Tested web / backend service
- API test evidence

## Stop conditions
- Task acceptance criteria satisfied with evidence
- Token budget exhausted
- Blocked on external dependency

## Escalation rules
- Escalate to lead architect or human if locked Goal criteria cannot be met.
- Escalate immediately upon discovering unexpected security vulnerabilities or data loss risks.
