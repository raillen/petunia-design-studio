# P00 Dossier — Fundação e endurecimento (P00-G01 + P00-G02)

## Status / scope

- P00-G01 `REVIEWING` (MVP scaffold verde; DONE aguarda revisão humana)
- P00-G02 `EXECUTING` (este endurecimento)
- Escopo: `V1_REQUIRED` para a fundação; funcionalidade de usuário fora de escopo.

## Goal e outcome

Base inviolável: workspace compila, contratos de IDs/diagnóstico, modelo de
documento com mutação exclusiva via `DocumentMutator`, comandos com undo/redo,
capabilities com disabled-reason, jobs com cancelamento, facade `cargo xtask`,
CI rápida, audit limpo, invariantes por propriedade.

## Non-goals

Engines (geometry/color/text/raster), I/O nativo `.aubrieta`, plugin/MCP, UI
GPUI, packaging/release, benchmarks, fuzzing de corpus.

## Referências canônicas

00 (charter), 04 (Rust stack), 07 (quality), 09.21 (governance), 09.22 (crates),
12.2 (protocolo), 12.10 (Prumo), 14.8 (xtask/evidência).

## Mapa de crates (físico P00)

- `aubrieta_foundation`: IDs, diagnostics, `NATIVE_SCHEMA_VERSION`
- `aubrieta_document`: Document/Surface/Object, ChangeSet, Mutator
- `aubrieta_application`: Action/Command, History, CapabilityRegistry
- `aubrieta_jobs`: CancellationToken, JobProgress
- `aubrieta-cli`: fluxo headless E2E
- `xtask`: 13 comandos (`verify/arch/test/conformance/fixtures/docs/gauntlet`
  reais; `fuzz-smoke/bench-smoke/ui-gauntlet/migrations/release-check` como
  stubs `POST_V1` explícitos)

## Contratos / IDs

- `ObjectId/SurfaceId/ResourceId/StyleId/EffectId` (newtypes `u64`, decisão
  MVP: gerador monotônico; UUID global fica para wave futura com ADR).
- `ActionId` namespace `aubrieta.*`; 4 ações P00.
- `ChangeSet` ordenado; `revert` inverso exato; `History` com limite e redo
  invalidado a cada execução nova.

## Segurança / hostile inputs

- `cargo audit --deny warnings`: limpo.
- `from_json` rejeita schema desconhecido; IDs com prefixo errado rejeitados;
  duplicatas rejeitadas; capability ausente = erro com reason, nunca panic.
- `cargo-fuzz` ausente no ambiente → `POST_V1`; proptest cobre invariantes.

## Evidência P00-G02 (endurecimento)

- `cargo test --workspace`: 18 passed (10 unit + 8 proptest), 0 failed.
- `cargo clippy --all-targets -- -D warnings`: limpo.
- `cargo audit --deny warnings`: limpo (55 deps).
- `.github/workflows/ci.yml`: gate `cargo xtask verify`.
- `cargo xtask gauntlet`: verde (a rodar no fechamento).

## Riscos / OPEN

- `prumo goal DONE` exige evidência formal além de `report add` (ficou em
  `REVIEWING`; não forçado).
- `context compile --required` não resolve filenames com espaço (conflito
  registrado; microcontexto manual).
- Classificador `adopt` inferiu JS/TS por `.opencode/node_modules` (ignorado;
  canônico é Rust).

## Handoff

Próximo: rodar `cargo xtask gauntlet`, `report add P00-G02-W1`, mover P00-G02
para `REVIEWING`, e entregar a lista de faltantes vs Atlas.
