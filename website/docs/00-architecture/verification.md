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
- linked-frame overflow.

Comparar shaping/layout contra fixtures versionadas e, quando útil, contra resultados de referência da biblioteca upstream.

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
