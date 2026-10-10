# Verificação, profiling e quality gates

Qualidade técnica no Petunia não é uma fase final. Cada domínio define como provar correção, estabilidade e custo.

> Uma feature só está pronta quando sabemos como detectar regressão funcional, numérica, visual e de performance.

## Camadas de verificação

Usar testes diferentes para problemas diferentes:

| Tipo | Objetivo |
|---|---|
| Unit test | provar função/regra pequena |
| Property test | provar invariantes para muitos inputs |
| Fuzz test | procurar crashes, hangs e estados inesperados |
| Roundtrip test | provar persistência/import-export |
| Golden test | detectar regressão visual |
| Integration test | validar fluxo entre crates |
| Benchmark | medir custo/latência/throughput |
| Soak/stress test | revelar leaks, crescimento e falhas sob carga |

Nenhum tipo substitui os outros.

## Core

Core testa invariantes determinísticos.

Exemplos:

- IDs tipados não se confundem;
- SceneGraph rejeita ciclos;
- parent/children permanecem consistentes;
- Path rejeita NaN/Inf;
- serialization canonicaliza `-0.0`;
- migrations preservam semântica;
- Document não contém dangling references obrigatórias.

Core tests não inicializam Qt.

## Property tests

**Property testing** gera muitos inputs e verifica uma propriedade geral.

Exemplos:

~~~text
Transform inverse:
inverse(T) × T × p ≈ p

Reverse contour twice:
reverse(reverse(path)) == original semantics

Split cubic:
left(t) + right(t) reconstruct original curve

Serialize roundtrip:
decode(encode(x)) == canonical(x)
~~~

Para floats, usar tolerância específica da propriedade. Não usar tolerância ampla global.

## Fuzzing

Fuzzing é obrigatório em superfícies que recebem input complexo não confiável.

Prioridade:

1. PTND container/parser;
2. JSON/DTO migrations;
3. SVG/import parsers/adapters;
4. image metadata/decoding boundaries;
5. ICC/profile parsing boundary;
6. font parsing boundary;
7. path/boolean edge cases;
8. plugin Host API message decoding.

Objetivos:

- nenhum panic em input externo;
- nenhum loop infinito;
- nenhum allocation sem guard;
- nenhum stack overflow previsível;
- erro tipado em input inválido.

Corpus de regressão guarda todo crash relevante descoberto.

## Geometry tests

Geometry possui três classes.

### Analytic cases

Casos com resposta conhecida:

- line-line intersection;
- rectangle boolean;
- circle/ellipse bounds;
- known cubic extrema;
- simple offsets;
- known winding.

### Degenerate cases

- zero-length;
- coincident endpoints;
- tangent intersections;
- nearly parallel lines;
- self-intersections;
- tiny/huge coordinate scales;
- near-singular transforms.

### Metamorphic tests

**Metamorphic testing** verifica relações que precisam continuar verdadeiras mesmo sem resposta exata fácil.

Exemplos:

~~~text
translate(A ∪ B)
≈
translate(A) ∪ translate(B)

boolean union(A, empty)
≈ A

offset distance 0
≈ source
~~~

## Shape Builder tests

Validar:

- face walking;
- no duplicate/open face inesperada;
- deterministic ordering derived;
- provenance preservada;
- same input/order produces same materialized result;
- cancellation/complexity guards;
- concave face interior sampling.

Manter fixtures de arrangements difíceis.

## Text tests

Test corpus inclui:

- Latin;
- Arabic;
- Hebrew;
- Devanagari;
- combining marks;
- emoji sequences;
- mixed BiDi;
- missing glyphs;
- variable fonts;
- ligatures;
- line-break edge cases;
- linked-frame overflow;
- COLR/CPAL glyphs;
- embedded raster glyphs;
- SVG-in-OpenType fallback/safety;
- missing/unsupported color glyph.

Comparar shaping/layout contra fixtures versionadas e, quando útil, contra resultados de referência da biblioteca upstream.

## Generated content tests

### Image Trace

Fixtures obrigatórias:

- logo high-contrast;
- line art com antialias;
- holes;
- diagonais;
- transparência;
- limited-color illustration;
- gradiente;
- imagem ruidosa/fotográfica;
- source muito grande com cancelamento.

Validar:

~~~text
same source bytes/profile
+ same TraceSpec
+ same semantic version
→ same palette/regions/order/geometry contract
~~~

Métricas incluem:

- max geometric deviation;
- número de regions;
- número de nodes;
- erro de cor;
- memória;
- tempo;
- deterministic ordering.

Preview e Authoring podem usar quality diferente, mas `Expand Trace` nunca materializa geometria de preview.

### QR Code

Usar test vectors do standard/backend e validar a matrix produzida antes de testar Render.

Além disso:

~~~text
QrCodeSpec
↓ Engine
VectorPath
↓ software renderer
bitmap
↓ decoder independente
payload original
~~~

Cobrir:

- todos os níveis ECC;
- Auto/Fixed Version;
- Auto/Fixed Mask;
- payload Unicode/bytes;
- limites de capacidade;
- quiet zone;
- transform não uniforme diagnosticado.

### Barcode

Para Code 128, EAN-13 e UPC-A:

- test vectors de module pattern;
- charset inválido;
- checksum válido/inválido;
- check digit calculado;
- quiet zone;
- roundtrip por decoder independente quando disponível.

O teste não depende apenas da aparência visual.

## Raster/Brush tests

Testar:

- same stroke input + seed → same software result;
- COW preserva tile antigo;
- cancel stroke não altera PixelSurface;
- Undo restaura versões exatas de tiles;
- spacing não depende da frequência bruta de pointer events;
- selection coverage modula dab corretamente;
- filter ROI produz o mesmo interior que full-surface evaluation.

## Color tests

Color management usa fixtures de perfis conhecidos.

Testar:

- Assign não altera channels;
- Convert altera channels conforme transform;
- roundtrip dentro da tolerância permitida;
- rendering intents;
- BPC;
- soft proof não altera Document;
- invalid profile retorna erro;
- Spot identity sobrevive a preview/export paths compatíveis.

Não usar screenshot como única prova de color transform.

## Render reference tests

Software renderer é referência semântica.

### Golden scenes

Manter cenas pequenas focadas em:

- fill rules;
- cubic edges;
- strokes/joins/caps;
- gradient interpolation;
- premultiplied alpha;
- blend modes;
- nested isolation;
- masks;
- blur/shadow;
- text;
- images;
- color output.

Cada golden declara target size, color context, quality, comparison policy e motivo do fixture.

### Pixel comparison

Não exigir bitwise global quando a operação admite diferença numérica legítima.

Usar exact para casos discretos, per-channel tolerance pequena e métricas perceptuais somente quando realmente adequadas.

Alterar golden exige revisão explícita; “teste falhou, regenere tudo” é proibido.

## CPU × futuros backends

~~~text
same RenderSnapshot
↓
Software Reference
↓
reference image

same RenderSnapshot
↓
GPU
↓
candidate image
~~~

Diferença precisa ficar dentro do contrato visual da operação. Backend não pode mudar semântica para ficar mais rápido.

## DocumentFragment tests

Copy/Paste e Duplicate precisam validar:

- dependency closure mínima e completa;
- ID remapping de Object/Node/Effect/Resource/Style/Symbol;
- referências internas preservadas;
- dangling reference rejeitada;
- clip/mask dependencies;
- symbol definition + overrides;
- linked/embedded resources;
- font/resource policy;
- paste entre Pages/Documents;
- Paste in Place;
- fragment schema migration;
- input hostil/oversized.

Propriedade importante:

~~~text
build fragment
↓ remap
↓ paste
↓ undo
↓ redo
→ mesmos IDs novos do primeiro paste
~~~

Duplicate reutiliza o mesmo pipeline e precisa produzir semântica equivalente sem serialização textual obrigatória.

## Persistence + Recovery tests

Além do save principal, testar recovery como sistema independente:

- checkpoint válido + journal;
- record final truncado;
- checksum inválido;
- replay interrompido em Transaction inválida;
- crash durante rotação de checkpoint;
- recovery de PixelLayer/tile/resource;
- documento nunca salvo;
- clean shutdown;
- recovery schema migration;
- budget/pruning sem remover recovery do documento dirty aberto.

Propriedade principal:

> qualquer prefixo confirmado do journal precisa reconstruir um Document válido ou falhar no último estado consistente conhecido.

## Serialization tests

Cada SchemaVersion possui fixtures.

~~~text
old fixture
↓ migrate
current DTO
↓ domain
↓ save
current PTND
↓ reopen
same semantic document
~~~

Também testar versões futuras incompatíveis, unknown extensions e corrupted entries.

## Import/export roundtrip

### PTND

Objetivo: lossless semântico/autoral.

### SVG

Objetivo: high-fidelity para subset suportado.

### PDF

Objetivo: output correto; import best-effort.

### Raster

Objetivo: pixel/color fidelity dentro do codec.

Teste nunca cobra roundtrip perfeito de formato que não representa a feature original.

## Determinism tests

Executar operações repetidas com mesma entrada e comparar:

- topology;
- stable ordering;
- serialization;
- materialized IDs quando o algoritmo define geração determinística;
- seeded procedural output.

Quando paralelismo participa, repetir com diferentes worker counts quando possível.

## Performance methodology

Todo benchmark registra:

- dataset/fixture;
- hardware class;
- build profile;
- operation;
- input size;
- metric;
- baseline;
- variance.

Métricas principais:

~~~text
latency
throughput
peak memory
allocations quando relevante
cache hit rate
time-to-first-frame/startup
~~~

## Performance suites

Manter três escalas:

~~~text
Small
Normal
Stress
~~~

Stress revela a curva de degradação; não significa suporte infinito.

## Interactive budgets

Operações no caminho de input têm orçamento próprio.

Medir hit-test, snap, selection query, transform preview e incremental render preparation.

Não congelar milissegundos universais antes dos primeiros benchmarks reais. Depois que hardware-alvo mínimo for definido junto com UX, budgets absolutos podem virar quality gate.

## Memory budgets

Verificar:

- caches respeitam soft/hard budget;
- cancel job libera snapshot;
- closing document libera resources;
- repeated open/close não cresce indefinidamente;
- history pruning libera payloads;
- render surface pool não cresce sem limite.

## Plugin + automation tests

Host API e automação são fronteiras de segurança e precisam de testes próprios.

### Capability tests

Para cada capability:

~~~text
permission absent
→ operation denied

permission granted
→ request still passes Engine/Core validation
~~~

Permissão nunca transforma input inválido em operação válida.

### WASM host contract

Testar:

- invalid/oversized messages;
- memory/fuel/stack limit;
- cancelled invocation;
- plugin crash/trap;
- stale handles;
- excessive job creation;
- resource stream limits;
- no direct filesystem/network without grant;
- no partial Transaction on failure.

O runtime concreto pode mudar; o mesmo contract suite precisa passar em qualquer runtime aprovado.

### MCP

Testar Query/Command equivalência com a API normal do Engine:

~~~text
same command payload
via app
via MCP
→ same validation + transaction semantics
~~~

Também testar servidor disabled-by-default e denial de capabilities não concedidas.

## Concurrency tests

Testar cancellation races, stale job result, revision conflict, snapshot lifetime, result arrival after document close e multiple documents/jobs.

Evitar testes baseados apenas em `sleep()`; preferir barriers/channels controlados.

## Failure injection

**Failure injection** força erros em pontos conhecidos.

Exemplos:

- allocation refused;
- disk full/write failure;
- resource disappears;
- plugin terminates;
- exporter fails mid-stream;
- CMM returns failure.

Objetivo: provar atomicidade e recovery.

## Save safety test

Simular falha em:

~~~text
before temp write
during temp write
after write before replace
during replace boundary
~~~

O último PTND válido nunca deve ser perdido por truncamento prematuro.

## Security limits

Cada parser/job com input potencialmente hostil possui testes nos limites:

- exact max;
- max + 1;
- nested structures;
- decompression ratio;
- huge declared dimensions;
- duplicate IDs;
- cycle attempts.

## Vector Edit contextual — testes de interação

O [ADR-0011](#/docs/00-architecture/adr/0011-hybrid-vector-edit.md) estabelece a fronteira Select / Vector Edit e a regra de que hover não muta documento.

Testes de unidade/integração de UI devem confirmar:

- duplo clique em Path, Enter com Path selecionado e ativação explícita de Node entram no mesmo contexto sem criar HistoryEntry;
- Group, Text, Shape e Symbol abrem seus contextos sem Convert to Curves;
- Escape durante drag/preview cancela sem Commit; Escape ocioso sobe exatamente um contexto;
- Enter em controle de texto/foco não aciona Vector Edit;
- Node→Bend→Node preserva sub-selection válida e nunca deforma segmento sem Bend explícito;
- documento não muda por hover, seleção, marquee ou mudança de ferramenta;
- paths locked/hidden e seleção mista não recebem mutação silenciosa;
- captura de pointer persiste ao sair dos limites da viewport até Up/Cancel;
- zoom, DPR, rotação de canvas e transform de grupo preservam significado do hit-test;
- keyboard/screen reader conseguem localizar modo, targets e actions;
- commit de um drag cria uma única entrada de Undo;
- stale revision rejeita aplicação silenciosa do resultado antigo.

**Concluir apenas a documentação não equivale a esses testes terem passado.** Implementação e QA de GUI/UX continuam futuros.

### Testes adicionais de seleção, nodes e handles

A proposta detalhada de [Seleção, nodes e handles](#/docs/04-ui/selection-nodes-handles.md) define testes a serem fechados com a decisão de UX:

- click em node selecionado versus não selecionado durante multi-node drag;
- hit-test de handle × node sobrepostos, com desambiguação;
- conversão Cusp/Smooth/Symmetric preservando invariantes e IDs;
- show selected/all handles não alterando conteúdo documental;
- bounding transform em seleção degenerada sem NaN;
- seleção de objetos por containment/crossing conforme política escolhida;
- click/drag threshold, pointer capture e focus por device/zoom/DPR.

O modelo híbrido está aprovado, mas as três escolhas específicas de UX dessa página ainda **não** estão congeladas.

## Ferramentas criativas — testes de aceitação do motor

Cada capability adicionada ao roadmap tem testes de Query/Preview/Commit/Undo/Redo/Save/Load, além de falha tipada, limites e cancelamento quando aplicável.

| Grupo | Caso de referência obrigatório |
|---|---|
| Smart Delete | dentro/fora da tolerância; cusp; closed path; preservar IDs sobreviventes |
| Direct Bend | mesmo resultado Preview/Commit; tangentes/limite de erro; cursor em extremos |
| Clean Vector | relatório sem alterar source; idempotência; max deviation; no accidental topology change |
| Select Similar | comparação perceptual/escopo; locked/hidden; nunca dirty |
| Shape Builder | overlaps, holes, tangency, touch-only, FillRule, style conflict |
| Region Paint | stable binding; source edit; ambiguous remap unresolved; virtual gap sem editar source |
| Close Gap | nearest gap != always correct; no bridge across barriers; undo |
| Intertwine | local z-order sem alterar global; multi-way crossing; release |
| Repeat/Symmetry | deterministic transforms; source edit live; mirror seam; no SceneNode explosion |
| Objects on Path | arc-length spacing; cusps; source removal; reverse spine |
| Blend | transforms/colors/easing; contour mismatch -> diagnostic or crossfade, never bogus morph |
| Scatter/Brushes | seeded determinism, pressure, memory budget, cancellation |
| Recolor | ICC/Spot/alpha; lock handling; gamut warnings; swatch bindings |
| Dimensions | accurate f64 geometry; source edit recomputes; target deleted unresolved |
| Width/Pattern | profiles and tile transforms survive save/load; Expand equivalence |
| Brand Sheet | generated objects remain editable; refs; profile-dependent values |
| Vector Feather | holes, acute corners, zoom, nested masks, ROI, falloff |
| Perspective/Envelope | degenerate projection rejected; visual-error bound |
| Mesh Gradient | when specified: patch continuity, interpolation, rendering accuracy |

Testar documents pequenos e grandes; working color profiles, nested groups, symbols, masks e mixed unit types. A comparação CPU reference renderer vs futuras acelerações usa tolerância visual documentada, nunca divergência silenciosa.

## CI gates

Antes de merge em código técnico afetado:

~~~text
format
↓
compile/check
↓
unit/integration
↓
clippy/lints
↓
selected property tests
↓
security/parser regression tests
~~~

Fuzzing contínuo e benchmarks pesados podem rodar em jobs próprios, não em todo commit.

## Quality gate de feature

Uma feature de Core/Engine/Render só muda de “implementada” para “pronta” quando possui:

1. owner/domínio correto;
2. invariantes documentadas;
3. error model;
4. undo/transaction behavior quando autoral;
5. deterministic policy;
6. serialization/migration quando persistente;
7. tests funcionais;
8. degenerate/failure tests;
9. performance measurement se hot path;
10. memory/cancellation behavior se job pesado;
11. headless behavior quando aplicável;
12. docs atualizadas.

## Regra de regressão

Uma otimização só é aceita quando:

~~~text
mesma semântica
+
melhor métrica relevante
+
complexidade adicional justificada
~~~

Se mudar semântica, não é apenas otimização; precisa de decisão arquitetural/ADR.

## Invariantes

1. Correção não depende de inspeção manual.
2. Parser externo é fuzz target prioritário.
3. Golden test não substitui teste matemático.
4. Software renderer é referência visual.
5. Benchmarks usam fixtures reproduzíveis.
6. Performance interativa é medida separadamente de batch throughput.
7. Failure injection valida atomicidade.
8. CI rápido e suites pesadas possuem cadências diferentes.
9. Feature sem testes de seus invariantes não está tecnicamente fechada.
10. UX/GUI quality gates serão definidos junto com a discussão de Interface.

## Verificação das superfícies externas do Core (2026-10-10)

Escopo: fuzz guards determinísticos do DTO, UUID e ContentHash. Revisão `8a214d418dfe62475baa46149ebb545309a5ee6d` sobre branch `petunia-design-rust`.

| Gate executado | Resultado |
|---|---|
| `cargo test --workspace` | pass: 359 passed / 0 failed (Core com 10 testes de fuzz) |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `node website/scripts/verify-progress.cjs` | pass |

Riscos/limites: detecção estrita de chaves duplicadas em mapas de identidade continua futura; ContentHash segue SHA-256 sem tag de algoritmo (BLAKE3 tagueado é follow-up).

## Verificação de chaves duplicadas (2026-10-10)

Escopo: `serialization.rs` com rejeição estrita ligada a todos os mapas de identidade. Revisão `f2ba26553d2451ba8efb914f879bfedeab9ce940` sobre branch `petunia-design-rust`.

| Gate executado | Resultado |
|---|---|
| `cargo test --workspace` | pass: 361 passed / 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `node website/scripts/verify-progress.cjs` | pass |

Riscos/limites: duplicatas em vetores ordenados (children, roots, order) continuam validadas por invariante própria, não no parse.
