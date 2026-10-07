# 12 — Code Agent Implementation Handbook & Living Documentation

# Operating model

Agents implementam contratos, não criam arquitetura oculta.

# Source-of-truth order

1. Goal/locked acceptance criteria;
2. este notebook canônico e ADRs;
3. schemas/headers públicos;
4. implementation;
5. tests/evidence;
6. derived docs.

Conflito entre implementação e contrato exige parar a expansão de escopo, registrar drift e atualizar autoridade com revisão.

# Python quality

- Pyright strict obrigatório;
- Ruff obrigatório;
- todas as public functions/classes tipadas;
- Protocol/TypedDict/dataclass/Enum para contratos, nunca dict mágico;
- dataclass(frozen=True, slots=True) para value types quando adequado;
- pathlib para paths;
- Result/typed exception model por boundary;
- nenhuma bare except;
- pickle/eval proibidos para conteúdo não confiável;
- dependency injection explícita;
- composição preferida a herança;
- async apenas onde existe I/O/coordenação real.

# C++ quality

- C++23;
- RAII;
- Rule of Zero preferida;
- unique_ptr por ownership exclusivo; shared_ptr somente com ownership realmente compartilhado;
- span/string_view para views com lifetime claro;
- expected/typed errors quando adequado;
- no naked new/delete em feature code;
- warnings as errors;
- clang-format/clang-tidy;
- ASan/UBSan/TSan gates.

# Change protocol

Goal -> impact map -> microcontext -> ADR se necessário -> plan DAG -> implementation -> tests -> independent review -> docs -> evidence.

# Living docs

Mudança de tool, schema, format, permission, action, panel ou export atualiza a página canônica correspondente e o changelog técnico na mesma alteração.

[12.1 — Python Strict Typing, Data Models, Errors & Package Discipline](12%201%20%E2%80%94%20Python%20Strict%20Typing,%20Data%20Models,%20Errors%20&%203f19bb7d023f8135b9f4c2cee9d709a5.md)

[12.2 — C++23 Safety, Ownership, Error Handling, Memory & Concurrency Discipline](12%202%20%E2%80%94%20C++23%20Safety,%20Ownership,%20Error%20Handling,%20Me%203f19bb7d023f81a5ac78df16fb1cfafb.md)

[12.3 — Design Patterns, Clean Architecture & Anti-Overengineering Rules](12%203%20%E2%80%94%20Design%20Patterns,%20Clean%20Architecture%20&%20Anti-%203f19bb7d023f811ba637d88affed549a.md)

[12.4 — Code-Agent Goal Protocol, Microcontext, ADR, Plan DAG & Handoff](12%204%20%E2%80%94%20Code-Agent%20Goal%20Protocol,%20Microcontext,%20ADR%203f19bb7d023f816dae01f264f60d753a.md)

[12.5 — Living Documentation, Schemas, API Reference, Examples & Bilingual Publishing](12%205%20%E2%80%94%20Living%20Documentation,%20Schemas,%20API%20Referenc%203f19bb7d023f8163a934ef6a54cb90f7.md)

[12.6 — Review Gates: Maintainability, Security, Performance, Accessibility & Release Claims](12%206%20%E2%80%94%20Review%20Gates%20Maintainability,%20Security,%20Per%203f19bb7d023f8128aae3e8709047ba6d.md)

[12.7 — Repository, Contribution, Review & Documentation Maintenance Rules for Agents](12%207%20%E2%80%94%20Repository,%20Contribution,%20Review%20&%20Document%203f19bb7d023f816eb54ac753ce3062ab.md)

[12.8 — Requirement Status, Scope Taxonomy, Backlog Semantics & Ambiguity Elimination](12%208%20%E2%80%94%20Requirement%20Status,%20Scope%20Taxonomy,%20Backlog%203f19bb7d023f8100a79ae7646fff15ff.md)

[12.9 — Change Impact Matrix: What Must Update for Every Feature Change](12%209%20%E2%80%94%20Change%20Impact%20Matrix%20What%20Must%20Update%20for%20E%203f19bb7d023f81e7af4fd9aca1ff2502.md)

[12.10 — Prumo CLI Workflow, Goals/Waves, Dossiers, Microcontexts & Agent Handoffs](12%2010%20%E2%80%94%20Prumo%20CLI%20Workflow,%20Goals%20Waves,%20Dossiers,%203f19bb7d023f8149818df26443c79c13.md)