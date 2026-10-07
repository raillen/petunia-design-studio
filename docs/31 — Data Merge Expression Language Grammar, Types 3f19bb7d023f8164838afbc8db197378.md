# 31 — Data Merge Expression Language: Grammar, Types, Functions, Safety & Evaluation

# Purpose

Formalize the deterministic sandbox expression language used by bindings/filename templates.

# Goals

Readable field expressions, formatting and conditionals without arbitrary Python/JS execution.

# Design

Typed values, bounded evaluation, pure functions, no filesystem/network/process/reflection, deterministic locale/time rules and explicit null/error behavior.

# Deliverables

Lexical grammar, parser AST, static/type validation, standard function library, evaluation budgets, escaping, filename-safe subset, versioning and fuzz tests.