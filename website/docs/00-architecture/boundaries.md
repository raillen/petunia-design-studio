# Fronteiras e invariantes

Esta página define **quem é dono de cada decisão**. A regra principal é simples: o dado permanente pertence ao Núcleo; o cálculo pertence ao Engine; transformar estado em pixels pertence ao Render; interação e apresentação pertencem à Interface.

> Uma funcionalidade pode atravessar os quatro domínios. O importante é que cada parte tenha um único dono.

## Direção de dependências

```text
petunia-ui ───────┐
                  ├──> petunia-engine ───> petunia-core
petunia-render ───┘            │
        └──────────────────────>┘
```

- `petunia-core` não conhece Qt, GPU, filesystem de UI ou widgets.
- `petunia-engine` conhece modelos do Core, mas não QML/Qt.
- `petunia-render` lê snapshots do Core e resultados avaliados; não altera o documento.
- `petunia-ui` traduz intenção humana em Commands e estados de sessão.

## Quatro tipos de estado

| Estado | Exemplo | Dono | Serializa no documento? |
|---|---|---|---|
| Documento | path, layer, cor, efeito, guia | Núcleo | Sim |
| Sessão | seleção, ferramenta ativa, zoom | Interface / session | Não |
| Derivado | bounds, tesselação, spatial index | Engine / Render | Não |
| Configuração | atalhos, tema, idioma | Interface / app settings | Fora do documento |

Misturar esses estados é uma fonte comum de bugs. Zoom não deve alterar geometria; cache não deve virar dado permanente; seleção não deve mudar o arquivo salvo.

## Fluxo de edição

```text
Pointer/teclado
      ↓
UI Tool Controller
      ↓
Engine: hit-test / snap / cálculo
      ↓
Command ou Transaction
      ↓
Core Document
      ↓
Snapshot / revisions
      ↓
Engine evaluation
      ↓
Render
```

A UI nunca deve modificar `SceneNode` diretamente durante uma ferramenta. Ela solicita uma operação ao Engine; a alteração final entra no documento por Command/Transaction.

## Precisão e tipos numéricos

A matriz recomendada é:

| Uso | Tipo |
|---|---|
| Geometria, paths, transforms, bounds | `f64` |
| Cor autoral e parâmetros normalizados | `f32` |
| Buffers raster 8/16-bit | tipos inteiros explícitos |
| Pipeline HDR/linear | `f16` ou `f32` conforme backend |
| IDs persistentes | UUID fortemente tipado |

Geometria usa `f64` porque operações booleanas, interseções e sequências longas de transforms acumulam erro. Render pode converter para `f32` no limite da GPU.

## Regras de mutabilidade

1. O documento autoritativo é alterado por transações explícitas.
2. Previews são transitórios e não entram no histórico a cada movimento do ponteiro.
3. Jobs de background recebem snapshots imutáveis.
4. Jobs retornam resultados/patches; não guardam `&mut Document`.
5. Caches são descartáveis e reconstruíveis.
6. O Renderer é logicamente read-only em relação ao documento.

## Vocabulário obrigatório

**Source** é o dado original. **Operation** é uma edição parametrizada. **Evaluation** calcula o resultado de operações. **Bake** materializa um resultado e perde a capacidade de editar etapas anteriores. **Snapshot** é uma visão imutável de uma revisão. **Overlay** é desenho de interface sobre o canvas, como handles e guias, e não faz parte do documento.

## Decisões que exigem ADR

**ADR — Architecture Decision Record** é um documento curto que registra uma decisão arquitetural importante, seu contexto, alternativas consideradas e consequências.

Criar um ADR quando mudarem: representação canônica de paths; formato PTND; modelo de efeito; estratégia de scene graph; color management/CMM; modelo de raster tiles; renderer GPU; threading; ABI de plugins; ou qualquer dependência que atravesse crates.

## Critério de teste de fronteira

**Headless** significa executar sem janela, display server ou interface gráfica.

Pergunte: “consigo executar isto headless?”. Boolean, snapping, import, export, text layout e filtros devem funcionar sem Qt. Se uma operação matemática precisar instanciar `QObject`, a fronteira está errada.
