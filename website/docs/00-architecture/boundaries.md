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
    Qt/QML via CXX-Qt existe somente nesta camada
```

### Core como base

`petunia-core` contém a representação autoral e persistente. Ele não conhece Qt/QML, CXX-Qt, widgets, janela, renderer ou algoritmos de UI.

Isso permite testar e usar o domínio sem inicializar interface gráfica.

### Engine

`petunia-engine` recebe tipos Petunia do Core, executa cálculos e devolve resultados Petunia.

Bibliotecas como `kurbo` e `i_overlay` podem ser usadas internamente, mas seus tipos não devem atravessar a API pública por conveniência quando forem apenas detalhes de implementação.

### Render

`petunia-render` transforma estado de leitura em pixels. Ele pode conhecer tipos estáveis do Core, mas não deve executar algoritmos que pertencem ao Engine nem modificar o documento.

Se Engine e Render passarem a precisar compartilhar um modelo derivado significativo, uma crate neutra de contrato poderá ser extraída. **Não criaremos essa quinta crate antecipadamente.**

Essa regra aplica o princípio de não criar abstração antes de existir necessidade real.

### Interface

`petunia-ui` usa Qt/QML via CXX-Qt para apresentação e interação.

Qt/QML pode conhecer:

- widgets;
- painéis;
- foco;
- eventos de pointer/teclado;
- estado transitório de ferramenta;
- layout do workspace.

Qt/QML não decide:

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
| Qt/QML via CXX-Qt | UI |

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

A classificação de estado responde a uma pergunta central:

> **Se este dado desaparecer, a obra muda ou apenas a experiência de edição?**

O Petunia usa quatro categorias. Não criar uma quinta categoria apenas para previews; preview é estado transitório de sessão até o commit.

| Categoria | Pergunta | Dono | Persistência | Undo/Redo | Muda `DocumentRevision`? |
|---|---|---|---|---|---|
| **Document State** | O que o artista criou? | Core | PTND | Sim | Sim |
| **Session State** | Como o usuário está editando agora? | UI / Session | opcional, fora do PTND | Não | Não |
| **Derived State** | O que pode ser recalculado? | Engine / Render | cache opcional | Não | Não |
| **Application Settings** | Como este usuário prefere usar o programa? | App / UI | configuração externa | Não | Não |

### Document State

É a **fonte autoral da verdade**.

Um dado pertence ao documento quando é necessário para reconstruir fielmente o trabalho depois de salvar, fechar e abrir o arquivo novamente.

Exemplos:

```text
Document
├── pages / artboards
├── scene graph
├── paths / shapes / text / images
├── transforms
├── appearance
├── effects
├── masks / clips
├── styles / symbols
├── resources
├── document color setup
├── guides
└── export slices
```

Persistente não significa automaticamente documental. Tema, atalhos e layout de painéis também podem ser persistidos, mas fora do PTND.

### Session State

Descreve a sessão de edição atual, não a obra.

Exemplos:

- seleção;
- hover;
- ferramenta ativa;
- subestado de ferramenta;
- pointer capture;
- drag atual;
- preview transitório;
- zoom;
- pan;
- rotação da view;
- modo de preview;
- snapping ligado/desligado.

Modelo recomendado:

```rust
pub struct EditorSession {
    pub active_document: DocumentHandle,
    pub selection: SelectionState,
    pub active_tool: ToolId,
    pub tool_session: ToolSession,
    pub active_view: ViewId,
    pub transient_edits: TransientEdits,
}
```

### View State

View State é parte da Session, mas merece tipo próprio porque um mesmo documento pode ter várias views simultâneas.

```rust
pub struct ViewState {
    pub zoom: ViewScale,
    pub pan: ViewTranslation,
    pub rotation: ViewRotation,
    pub viewport_size: DeviceSize,
    pub overlay_visibility: OverlayVisibility,
    pub preview_mode: PreviewMode,
}
```

```text
Document A
├── View 1 → 25%
├── View 2 → 400%
└── View 3 → soft proof
```

Na v0.1, View State não entra no PTND. Se quisermos restaurar a última sessão, isso deve ser armazenado separadamente e associado ao `DocumentId`.

### Preview e Transient Edit

**Transient** significa temporário: existe durante uma interação, mas ainda não virou edição autoral.

Durante um drag:

```text
Document revision 84
        +
Transient Transform
        ↓
Preview
```

A revisão continua 84.

Somente no commit:

```text
Transient Edit
      ↓
Transaction
      ↓
Commit
      ↓
Document revision 85
```

Portanto:

> Preview nunca deixa o documento dirty e nunca cria dezenas de entradas de histórico por movimento do ponteiro.

### Derived State

**Derived State** é qualquer informação que pode ser reconstruída a partir de dados autoritativos.

Exemplos:

```text
VectorPath → bounds
VectorPath → flattening → tessellation
Text       → shaping → glyph positions
Scene      → spatial index
Effects    → raster tiles
```

Se todos os dados derivados forem apagados, o documento continua correto. Apenas precisa recalcular.

Não criar um `DerivedState` gigante. Cada subsistema possui seus próprios caches:

```text
Geometry → GeometryCache
Spatial  → SpatialIndex
Text     → TextLayoutCache
Effects  → EffectCache
Render   → TessellationCache / GlyphCache / TileCache
```

### Invalidation

**Invalidar** um cache significa declarar que o resultado antigo não corresponde mais aos dados atuais.

```text
Path revision 12
Bounds cache revision 12
→ válido

Path revision 13
Bounds cache revision 12
→ inválido
```

A invalidação deve ser localizada. Mudar o nome de uma layer não deve apagar tessellation; alterar o path deve invalidar os resultados que dependem de sua geometria.

### Application Settings

São preferências do usuário ou da instalação, persistidas fora do documento.

Exemplos:

```rust
pub struct ApplicationSettings {
    pub theme: Theme,
    pub language: Language,
    pub reduced_motion: bool,
    pub autosave_policy: AutosavePolicy,
    pub shortcuts: ShortcutMap,
}
```

Workspace também é configuração da aplicação:

```rust
pub struct WorkspaceSettings {
    pub dock_layout: DockLayout,
    pub panel_visibility: PanelVisibility,
    pub toolbar_layout: ToolbarLayout,
}
```

Qt pode ser usado como mecanismo de persistência, mas os tipos Qt não definem o significado dessas configurações.

### Features que atravessam categorias

Uma mesma feature pode conter propriedades de categorias diferentes. Nesse caso, separar os tipos.

#### Grid

```text
GridDefinition       → Document
Grid visibility      → View
Grid render cache    → Derived
```

A geometria do grid pode fazer parte da construção; a decisão de exibi-lo é da view.

#### Guides

```text
Guide position       → Document
Guide locked         → Document
Guides visible       → View
Guide raster/overlay → Derived
```

Lock faz parte da intenção compartilhada de edição. Visibilidade é apenas como a view apresenta as guias.

#### Layers

`Layer.visible` é Document State porque altera render final, export e impressão.

Mostrar ou esconder overlays de seleção da layer é View State.

#### Snapping

Geometrias que podem gerar snap pertencem ao documento. Preferências como `snap enabled`, `snap to grid` e `snap to nodes` pertencem à Session/Workspace.

Desabilitar snapping nunca deixa o documento dirty.

#### Soft proof

Perfil de cor do documento é Document State.

`Soft Proof enabled` e `Gamut Warning visible` são View State porque simulam visualização e não alteram a obra.

#### Resources

Descrição e referência do recurso são autorais:

```text
ResourceId
URI/path
content hash
embed/link policy
metadata
```

Buffers decodificados, thumbnails e representações prontas para render são Derived State.

### Recovery não muda a classificação

Crash recovery pode persistir dados operacionais temporariamente:

```text
Recovery Journal
├── document revision
├── pending transactions
├── resource changes
└── timestamps
```

Isso é infraestrutura de recuperação, não parte do formato autoral PTND.

### DocumentRevision e dirty state

`DocumentRevision` identifica um **estado autoral do histórico**.

Uma nova alteração commitada cria uma nova revision. Undo e Redo não criam revisions artificiais: eles navegam entre estados já existentes.

```text
Revision 18 ← saved
    ↓
Revision 19
    ↓
Revision 20

Undo
↓
current_revision = 19

Undo
↓
current_revision = 18
→ clean novamente
```

Se o usuário cria uma nova edição depois de Undo, essa edição recebe uma nova identidade e o redo branch antigo é descartado na v0.1.

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DocumentRevision(u64);
```

A revision não muda por:

- hover;
- zoom;
- pan;
- seleção;
- troca de ferramenta;
- rebuild de cache;
- soft proof.

**Dirty** significa “o estado autoral atual é diferente do estado salvo”.

Não manter um boolean autoritativo. Derivar:

```text
is_dirty = current_revision != saved_revision
```

No save:

```text
saved_revision = current_revision
```

Essa semântica permite que Undo retorne exatamente ao estado salvo e torne o documento clean sem heurística adicional.

### Invariantes

1. Document State é a única verdade autoral persistida no PTND.
2. Session State controla a experiência de edição, não a obra.
3. Preview/Transient Edit pertence à Session até o commit.
4. Derived State pode sempre ser descartado e reconstruído.
5. Application Settings são persistidas separadamente do documento.
6. Uma propriedade possui exatamente um dono autoritativo.
7. `DocumentRevision` identifica estados autorais; novas revisions surgem em commits e Undo/Redo navegam entre revisions existentes.
8. Dirty state é derivado de `current_revision != saved_revision`.
9. View State não entra no PTND na v0.1.
10. Recovery pode persistir estado operacional sem transformá-lo em Document State.

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

Pergunte: “consigo executar isto headless?”. Boolean, snapping, import, export, text layout e filtros devem funcionar sem Qt e sem criar uma janela. Se uma operação matemática precisar de `QObject`, `QQuickItem` ou outro tipo Qt, a fronteira está errada.
