# Legado Python/C++ → Petunia Rust: reaproveitar ou reconstruir?

**Conclusão com evidência:** **reconstrução ampla em Rust é apropriada**, mas **não é correto dizer que nada serve do legado**. O projeto Python/C++ de 2026-10-07 está preservado no [commit histórico `13fe6b4`](https://github.com/raillen/petunia-ds/tree/13fe6b408752b244ee56d4765ea13435eb3ef921). Nesta inspeção estática confirmei código, testes e QML; **não executei o código antigo nem os testes**. O branch principal atual contém quatro crates Rust e já possui uma arquitetura nova documentada.

## 1. O que está presente no snapshot antigo

| Evidência histórica | O que preservar/avaliar | Política de integração |
|---|---|---|
| [`python/petunia_app/paths.py`](https://github.com/raillen/petunia-ds/blob/13fe6b408752b244ee56d4765ea13435eb3ef921/python/petunia_app/paths.py) | Nodes `CUSP/SMOOTH/SYMMETRIC`, handles opcionais, `FillRule`, Bézier math e flatten | **Port seletivo** de fórmulas/fixtures para Rust; respeitar novos types/IDs |
| [`python/petunia_app/boolean.py`](https://github.com/raillen/petunia-ds/blob/13fe6b408752b244ee56d4765ea13435eb3ef921/python/petunia_app/boolean.py) | Boolean, provenance, Shape Builder, testes degenerados | **Reference + differential oracle**, não copiar backend Python `pyclipper` cegamente |
| [`tests/test_boolean.py`](https://github.com/raillen/petunia-ds/blob/13fe6b408752b244ee56d4765ea13435eb3ef921/tests/test_boolean.py) | Known areas, holes, coincidences, open contour errors, order, fuzz, benchmark de 10k segments | **Recriar testes equivalentes em Rust**, manter fixtures e contratos, revisar bug expectations |
| [`tests/test_golden_workflow.py`](https://github.com/raillen/petunia-ds/blob/13fe6b408752b244ee56d4765ea13435eb3ef921/tests/test_golden_workflow.py) | Add/Move, Undo/Redo, Group/Ungroup, PNG export | Portar scenarios e assertions para Commands e History novos |
| [`python/petunia_app/ptnd/package.py`](https://github.com/raillen/petunia-ds/blob/13fe6b408752b244ee56d4765ea13435eb3ef921/python/petunia_app/ptnd/package.py) | ZIP safety, limits, canonical JSON e read/write package | **Portar políticas e testes**; evoluir schema conforme ADR-0003 |
| [`cpp/petunia_core/include/petunia/core/document/document.hpp`](https://github.com/raillen/petunia-ds/blob/13fe6b408752b244ee56d4765ea13435eb3ef921/cpp/petunia_core/include/petunia/core/document/document.hpp) | Typed IDs, DocumentStore, reparent/attach/remove | Estudar invariantes; o Core Rust é autoral e não adota este object model |
| [`python/petunia_app/qml/components/ContextToolbar.qml`](https://github.com/raillen/petunia-ds/blob/13fe6b408752b244ee56d4765ea13435eb3ef921/python/petunia_app/qml/components/ContextToolbar.qml) | Gestos/controles e layout contextual preexistente | **Candidato a adaptar QML**, mas auditar tamanho, tokens, handlers, bindings e UX atual |
| [`python/petunia_app/qml/views/CanvasViewport.qml`](https://github.com/raillen/petunia-ds/blob/13fe6b408752b244ee56d4765ea13435eb3ef921/python/petunia_app/qml/views/CanvasViewport.qml) | Overlay, viewport e interação já explorados | Ler/reaproveitar subcomponentes QML onde desacopláveis; substituir ponte Python pelo bridge CXX-Qt |
| [`python/petunia_app/qml/components/PetuniaSlider.qml`](https://github.com/raillen/petunia-ds/blob/13fe6b408752b244ee56d4765ea13435eb3ef921/python/petunia_app/qml/components/PetuniaSlider.qml) | Um controle QML pequeno e estilizado | Possível port parcial; melhorar labels, number input, foco e keyboard |
| [`tests/test_ui_bridge.py`](https://github.com/raillen/petunia-ds/blob/13fe6b408752b244ee56d4765ea13435eb3ef921/tests/test_ui_bridge.py) | Comportamento GUI↔backend anterior | Mapear para Qt signal/slot ↔ CXX-Qt e testar novo contrato |

Outros diretórios históricos encontrados: `python/petunia_app/bridge.py`, `model.py`, `text.py`, `raster.py`, `exporters.py`, `docking/`, QML de panels, dialogs, toolbars; `bindings/python/src/petunia_native.cpp`, `cpp/petunia_core/`, `benchmarks/`, `tooling/` e dezenas de docs históricas. O inventário original tinha centenas de arquivos; estes são **pontos iniciais, não auditoria exhaustiva de cada função**.

## 2. O que de fato reimplementar

- **Python backend / PySide6 bridge / nanobind:** não é o runtime da nova arquitetura; lógica e bindings são substituídos por Rust e CXX-Qt. Não manter duas fontes de verdade.
- **C++ model/storage:** o `petunia-core` Rust é canônico; migrar requisitos e invariantes, não manter `DocumentStore` paralelo.
- **GUI Qt/QML:** **não precisa ser reescrita do zero automaticamente**. QML é reutilizável entre backends se não acoplada a objetos registrados por Python. Extrair componentes pequenos, depois modernizar estrutura/tokens e bindings para Rust/CXX-Qt.
- **Ferramentas e maths:** formulas e state machines podem ser traduzidas, mas devem passar por specs mais recentes de Smart Path/Region/Selection; não copiar a semântica antiga em lugar de decisão aprovada.
- **Formatos/schemas/test data:** migrar compatibilidade e fixtures, não copiar serialização incompatível; compatibilidade com arquivos antigos é uma escolha explícita de import/migration.
- **Qualidade:** testes históricos devem virar critérios de regressão do Rust, mas devem ser atualizados se eles cristalizam um bug/limite antigo.

## 3. Limitação concreta da implementação booleana antiga

No cabeçalho de `python/petunia_app/boolean.py` o backend declara **`pyclipper` + curvas achatadas para linhas** com tolerance 0.25 units e quantização inteira. Essa implementação pode orientar casos de teste e tratamento de topologia, mas a reconstrução atual requer avaliar precisão Bézier, geometria derivada, tolerâncias e provenance com `i_overlay`/VectorCraft. O dado útil é o **comportamento esperado e o corpus** — não necessariamente seu algoritmo de flattening.

Em `paths.py` existiam NodeKind Cusp/Smooth/Symmetric e handles offset; no Core Rust e especificações atuais, `NodeId` persistente e `Option<Point>` continuam importantes. Não portar APIs Python literalmente.

## 4. Matriz de decisão por recurso do legado

Marcar cada candidato:

| Estado | Significado |
|---|---|
| `LEGACY_IDENTIFIED` | path/commit original confirmado |
| `BEHAVIOR_REVIEWED` | semântica e UX analisadas contra spec atual |
| `TESTS_EXTRACTED` | corpus, fixtures ou testes capturados para Rust |
| `REUSE_QML` | componente Qt/QML adapta-se a novo bridge, após revisão |
| `PORT_ALGORITHM` | algoritmo implementado em Rust preservando contrato |
| `REFERENCE_ONLY` | código serve para entender comportamento, não integrar |
| `DISCARD` | descartado por incompatibilidade comprovada, com motivo |
| `ADOPTED_TESTED` | integrado e testes executados com evidência |

## 5. Procedimento obrigatório para agents

```text
1. Encontre a feature na documentação canônica atual.
2. Procure arquivos de código/testes/QML em 13fe6b4.
3. Leia seus contratos, limites e dependências (não apenas nomes).
4. Compare com Rust current Core/Engine/Render/UI e decisões de UX.
5. Classifique: QML reuse / algorithm port / tests reuse / reference only / discard.
6. Se aproveitável, faça mudança mínima com identifiers, color/units/Undo corretos.
7. Execute testes Rust novos + fixtures migradas.
8. Atualize status e provenance, sem chamar de "legado totalmente portado".
```

Acesso **histórico em commit fixo**, sem recriar branch de legado nem executar `git reset --hard` na main. A documentação nova continua fonte canônica, não os documentos antigos Python/C++ removidos em commit posterior.

## 6. Recomendação por etapas

**Primeiro:** extrair tests e math de `paths.py`, `boolean.py`, `test_boolean.py`, `test_golden_workflow.py`; são úteis para a primeira onda de geometria e Undo Rust.

**Depois:** auditar o QML de toolbars, canvas e painéis como biblioteca de interação, priorizando refatorar componentes menores e A11y antes de readaptar o binding.

**Depois:** PTND, formatos e imagem por adapter + testes de compatibilidade, com análise de superfície de ataque.

**Evitar:** portar `bridge.py` gigante e `model.py` diretamente para Rust; são candidates para descoberta de responsabilidades e subdivisão por crate, não para virar novos arquivos monolíticos.

**Conclusão:** o Core e Engine Python/C++ vão majoritariamente dar lugar a novas implementações Rust; QML, testes e conhecimento técnico podem economizar trabalho real. A decisão fina depende da auditoria de cada componente, não de uma rejeição global baseada na linguagem.

[Índice dos agentes](#/docs/07-agents/index.md) · [Workflow](#/docs/07-agents/implementation-workflow.md) · [Referências open source](#/docs/06-references/index.md).
