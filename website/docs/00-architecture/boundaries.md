# Fronteiras e invariantes

Esta página define **quem é dono de cada decisão**. A regra principal é simples: o dado permanente pertence ao Núcleo; o cálculo pertence ao Engine; transformar estado em pixels pertence ao Render; interação e apresentação pertencem à Interface.

> Uma funcionalidade pode atravessar os quatro domínios. O importante é que cada parte tenha um único dono.

## Direção de dependências

**Dependência** aqui significa uma crate conhecer tipos ou APIs de outra crate. Isso não é a mesma coisa que a ordem em que funções são chamadas durante a execução.

A direção definida para a arquitetura atual é:

```text
                 petunia-ui
                ↙    ↓    ↘
             Core  Engine  Render
                    ↓       ↓
                 petunia-core
```

De forma explícita:

```text
petunia-core
└── não depende de nenhum outro domínio Petunia

petunia-engine
└── pode depender de petunia-core
    não depende de petunia-render ou petunia-ui

petunia-render
└── pode depender de petunia-core
    não depende de petunia-engine ou petunia-ui

petunia-ui
└── pode depender de Core, Engine e Render
    egui existe somente nesta camada
```

### Core como base

`petunia-core` contém a representação autoral e persistente. Ele não conhece `egui`, widgets, janela, renderer ou algoritmos de UI.

Isso permite testar e usar o domínio sem inicializar interface gráfica.

### Engine

`petunia-engine` recebe tipos Petunia do Core, executa cálculos e devolve resultados Petunia.

Bibliotecas como `kurbo` e `i_overlay` podem ser usadas internamente, mas seus tipos não devem atravessar a API pública por conveniência quando forem apenas detalhes de implementação.

### Render

`petunia-render` transforma estado de leitura em pixels. Ele pode conhecer tipos estáveis do Core, mas não deve executar algoritmos que pertencem ao Engine nem modificar o documento.

Se Engine e Render passarem a precisar compartilhar um modelo derivado significativo, uma crate neutra de contrato poderá ser extraída. **Não criaremos essa quinta crate antecipadamente.**

Essa regra aplica o princípio de não criar abstração antes de existir necessidade real.

### Interface

`petunia-ui` usa `egui` para apresentação e interação.

`egui` pode conhecer:

- widgets;
- painéis;
- foco;
- eventos de pointer/teclado;
- estado transitório de ferramenta;
- layout do workspace.

`egui` não decide:

- geometria;
- topologia;
- snapping;
- regras de documento;
- shaping de texto;
- efeitos;
- composição.

### Dependências externas

Uma biblioteca externa precisa ter um domínio proprietário.

Exemplos:

| Dependência | Proprietário principal |
|---|---|
| `kurbo` | Engine / Geometry |
| `i_overlay` | Engine / Geometry |
| `palette` | Engine / Color |
| `rustybuzz` | Engine / Text |
| `fontdue` | Render / rasterização de glifos |
| `rayon` | Engine e Render, internamente |
| `serde` | Core / persistência |
| `uuid` | Core / identidade |
| `egui` | UI |

“Proprietário” significa que aquele domínio decide como a biblioteca é encapsulada. Não significa que uma dependência nunca possa ser usada em outro lugar; exceções precisam de justificativa arquitetural.

### Adapter

Um **adapter** é uma camada pequena que converte entre o modelo Petunia e a API de uma biblioteca.

```text
VectorPath Petunia
      ↓ adapter
tipo esperado por kurbo/i_overlay
      ↓ algoritmo
resultado externo
      ↓ adapter
VectorPath Petunia
```

Isso impede que trocar uma biblioteca obrigue a alterar o formato PTND ou toda a aplicação.

### Regra

> Tipos de terceiros não atravessam fronteiras públicas de domínio apenas por conveniência.

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

Pergunte: “consigo executar isto headless?”. Boolean, snapping, import, export, text layout e filtros devem funcionar sem `egui` e sem criar uma janela. Se uma operação matemática precisar de `egui::Context`, a fronteira está errada.
