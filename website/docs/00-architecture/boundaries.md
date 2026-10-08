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

## Precisão numérica, tolerâncias e determinismo

A representação numérica precisa preservar a obra sem transformar detalhes de implementação em alterações autorais.

A matriz base é:

| Uso | Tipo |
|---|---|
| Geometria, paths, transforms, bounds e medidas | `f64` |
| Cor autoral e parâmetros normalizados | `f32` |
| Buffers raster 8/16-bit | tipos inteiros explícitos |
| Pipeline HDR/linear | `f16` ou `f32` conforme backend |
| IDs persistentes | UUID fortemente tipado |

Geometria canônica permanece em `f64`. Render/GPU pode converter para `f32` em uma fronteira controlada, sem reduzir a precisão armazenada no documento.

### Formatação não é mutação

A UI pode mostrar:

```text
12.345678901 mm
↓ formatação
12.35 mm
```

mas o valor autoral continua completo.

> Exibir menos casas decimais nunca quantiza silenciosamente o Document.

Somente uma edição explícita do usuário muda o valor.

### Não existe EPSILON universal

**Tolerância** é o erro máximo aceitável para uma decisão específica.

Hit-test, snapping, boolean, flattening e inversão de matriz possuem significados diferentes de “próximo o suficiente”.

Por isso não usar um único:

```rust
const EPSILON: f64 = ...;
```

para toda a aplicação.

A API deve deixar o contexto explícito, por tipo ou nome:

```text
CoincidenceTolerance
FlattenTolerance
BooleanTolerance
HitTestTolerancePx
SnapTolerancePx
```

Esses tipos entram quando melhorarem segurança e leitura; não precisam ser criados preventivamente todos de uma vez.

### Document-space e screen-space

Tolerâncias geométricas autorais normalmente trabalham em **document-space**.

Tolerâncias de interação normalmente trabalham em **screen-space**, isto é, pixels percebidos na tela.

```text
document_tolerance =
screen_tolerance_px / view_scale
```

Isso mantém nodes, handles e snap utilizáveis em diferentes níveis de zoom.

### Três noções de igualdade

Não tratar todas as comparações da mesma maneira.

- **Exact equality** — valores discretos que precisam ser idênticos, como IDs e enums.
- **Semantic equality** — valores geométricos considerados equivalentes dentro de tolerância declarada.
- **Visual equality** — diferenças menores que a percepção/output relevante da view atual.

Uma tolerância algorítmica não autoriza modificar coordenadas permanentemente. “Considerar coincidente” e “fundir pontos” são decisões diferentes.

### Valores não finitos

`NaN`, `+Inf` e `-Inf` são rejeitados em Commands, import, plugins e outras fronteiras antes de entrar no Document.

Eles quebram premissas de comparação, ordenação, bounds, índices espaciais e serialização.

### Canonicalização numérica

Persistência deve preservar round-trip de `f64` sem quantização de UI.

Representações semanticamente irrelevantes podem ser canonicalizadas, por exemplo:

```text
-0.0 → 0.0
```

**Canonicalizar** significa escolher uma representação estável para valores semanticamente equivalentes.

Isso reduz diffs, hashes e cache keys desnecessariamente diferentes.

### Determinismo

**Determinismo semântico** significa:

> mesma entrada + mesmos parâmetros → mesmo resultado autoral relevante.

Não permitir que ordem acidental de `HashMap`, scheduling de threads ou timing de execução determine:

- z-order;
- ordem persistida;
- IDs derivados;
- topologia;
- serialização;
- resultado lógico de operações.

Quando um cálculo paralelo produz resultados em ordem arbitrária, normalizar/ordenar antes de transformar essa ordem em semântica persistente.

### Floating point e paralelismo

Aritmética de ponto flutuante não é perfeitamente associativa:

```text
(a + b) + c
pode diferir levemente de
a + (b + c)
```

Paralelismo pode alterar a ordem de combinação.

Por isso o Petunia exige determinismo **semântico**, não promessa geral de framebuffer bit-a-bit idêntico em qualquer CPU/GPU.

Bitwise determinism só é requisito onde uma página específica declarar explicitamente.

### Aleatoriedade autoral

Qualquer operação persistente que use aleatoriedade — brush jitter, scatter, noise ou pattern procedural — precisa de seed explícita.

```text
same input
+ same parameters
+ same seed
→ same authored result
```

“Randomize” gera uma nova seed de forma explícita.

Tempo do sistema não entra implicitamente em uma operação autoral estática.

### Predicates robustos

Um **predicate geométrico** responde uma decisão discreta, como orientação de três pontos ou lado de uma linha.

Essas respostas podem definir topologia.

```text
orientation(A, B, C)
→ clockwise
→ counter-clockwise
→ collinear
```

Predicates críticos precisam de implementações robustas e tolerâncias apropriadas. Comparações improvisadas perto de zero podem produzir loops, contornos invertidos ou booleans instáveis.

### Testes

O tipo de comparação precisa combinar com a semântica testada.

| Caso | Comparação |
|---|---|
| IDs, enums, ordem autoral | exata |
| pontos e bounds calculados | tolerância numérica específica |
| topologia | estrutura/ordem definida |
| renderer CPU | golden/reference conforme contrato |
| GPU vs reference | tolerância visual definida |
| serialization round-trip | equivalência canônica |

Evitar tanto `assert_eq!` indiscriminado em floats quanto tolerâncias grandes o suficiente para esconder regressões.

### Invariantes numéricas

1. Geometria autoral usa `f64`; menor precisão só aparece em fronteiras controladas.
2. Formatação da UI nunca quantiza silenciosamente dados autorais.
3. Não existe tolerância global universal.
4. Cada algoritmo usa tolerância coerente com sua semântica.
5. Tolerância de interação é normalmente screen-space.
6. NaN e infinito nunca entram no Document.
7. Persistência preserva round-trip e pode canonicalizar representações equivalentes como `-0.0`.
8. Ordem de containers não ordenados nunca define resultado persistente.
9. Paralelismo não torna saída autoral dependente da ordem de execução.
10. Determinismo exigido por padrão é semântico, não bitwise cross-platform.
11. Aleatoriedade autoral usa seed explícita.
12. Predicates topológicos críticos usam implementação robusta.
13. Testes escolhem igualdade/tolerância conforme a semântica.
14. Caches preferem revisions a inferir mudança comparando floats.

## Snapshots e concorrência

O Petunia usa **single writer + snapshots imutáveis** como modelo principal de concorrência.

**Concorrência** significa permitir que diferentes trabalhos avancem sem obrigar todos a acessar o mesmo estado mutável ao mesmo tempo.

A regra é:

> O Document autoritativo possui um único escritor. Leitores concorrentes recebem visões imutáveis de uma revisão.

~~~text
                 Single Writer
                      │
               Authoring Document
                      │
                 commit 42
                      ↓
              Immutable Snapshot
              ↙       ↓       ↘
          Render    Engine     Jobs
                                │
                                ↓
                         resultado tipado
                                │
                                ↓
                          Transaction API
                                │
                                ↓
                           Single Writer
~~~

### Single writer

**Single writer** significa que somente um fluxo possui autoridade para alterar o Document.

Isso não significa que a aplicação possua apenas uma thread. Geometry, raster, export e outros trabalhos podem ocorrer em paralelo.

Significa apenas:

~~~text
workers
→ leitura imutável

document owner
→ única autoridade de commit
~~~

Evitar espalhar `Arc<Mutex<Document>>` pela aplicação.

`Arc` permite compartilhar ownership de um valor entre threads por contagem de referências.

`Mutex` permite acesso exclusivo a um valor compartilhado: apenas um fluxo por vez entra na seção protegida.

Ambos são ferramentas válidas. O problema é transformá-los no modelo geral de acesso ao Document, o que aumentaria contenção, dificuldade de raciocínio e risco de deadlocks.

### Deadlock

**Deadlock** ocorre quando dois fluxos esperam indefinidamente recursos que o outro possui.

~~~text
Thread A
segura Lock 1
↓
espera Lock 2

Thread B
segura Lock 2
↓
espera Lock 1

→ nenhum avança
~~~

Single writer e snapshots imutáveis reduzem drasticamente essa classe de problema.

## Snapshot

Um **snapshot** é uma visão consistente e somente-leitura do documento em uma `DocumentRevision` específica.

Direção conceitual:

~~~rust
pub struct DocumentSnapshot {
    pub revision: DocumentRevision,
    pub scene: SceneSnapshot,
    pub resources: ResourceSnapshot,
    pub styles: StyleSnapshot,
}
~~~

A estrutura exata ainda não está congelada. O contrato é mais importante que a implementação:

- representa uma única revision;
- não pode ser alterado pelo consumidor;
- pode ser compartilhado com workers;
- deixa de ser necessário quando nenhum consumidor o referencia.

### Snapshot não é PTND

Snapshot é estrutura runtime.

PTND é formato persistente.

Não serializar o Document para JSON e desserializar novamente para produzir um snapshot durante uso normal.

~~~text
Document
↓
snapshot runtime
✓

Document
↓
JSON/PTND
↓
parse novamente
↓
snapshot
✗
~~~

Persistência e leitura concorrente têm objetivos diferentes.

### Consistência de revisão

Um snapshot nunca mistura partes de revisions diferentes.

Inválido:

~~~text
Scene      → revision 40
Resources  → revision 42
Styles     → revision 41
~~~

Válido:

~~~text
DocumentSnapshot
revision = 42
├── Scene consistente com 42
├── Resources consistentes com 42
└── Styles consistentes com 42
~~~

A criação do snapshot precisa ser logicamente atômica mesmo que internamente reutilize memória.

## Structural sharing

**Structural sharing** significa que dois estados imutáveis podem reutilizar partes idênticas em memória em vez de copiá-las.

~~~text
Revision 41
├── A ─────────────┐
├── B              │
└── C ──────────┐  │
                 │  │
Revision 42      │  │
├── A ───────────┘  │  compartilhado
├── B'               │  alterado
└── C ───────────────┘  compartilhado
~~~

Isso pode ser implementado com técnicas como `Arc<T>`, copy-on-write ou estruturas persistentes.

A escolha concreta **não está definida ainda**. Primeiro mediremos custo, padrões de edição e pressão de memória.

Não adotar uma estrutura complexa apenas porque snapshots existem.

## Copy-on-write

**Copy-on-write — COW** significa compartilhar dados enquanto há apenas leitura e copiar somente quando uma escrita precisa produzir uma versão diferente.

~~~text
Snapshot 40 ─┐
             ├── Tile A
Snapshot 41 ─┘
~~~

Se Tile A muda na nova revisão:

~~~text
Snapshot 40 → Tile A
Snapshot 41 → Tile A'
~~~

COW é especialmente promissor para:

- raster tiles;
- imagens decodificadas grandes;
- blobs de recursos;
- outros dados grandes e imutáveis.

Não assumir que COW é melhor para cada node pequeno do SceneGraph. Isso precisa ser medido.

## Jobs

Um **job** é um trabalho que pode continuar fora do fluxo interativo principal, como Image Trace, thumbnail, import pesado ou export.

Conceitualmente:

~~~rust
pub struct JobInput<T> {
    pub revision: DocumentRevision,
    pub snapshot: DocumentSnapshot,
    pub request: T,
}
~~~

Fluxo:

~~~text
Document revision 120
        ↓
     Snapshot
        ↓
   Background Job
        ↓
   resultado tipado
expected_revision = 120
        ↓
  Document Owner
        ↓
validate + Transaction
~~~

O worker nunca recebe `&mut Document`.

### Validação de resultado na v0.1

Na v0.1, resultados de jobs que pretendem alterar o documento usam a revision documental completa como barreira de segurança.

~~~text
job expected = 120
current      = 120
→ pode preparar commit

job expected = 120
current      = 123
→ resultado potencialmente obsoleto
~~~

Isso é conservador: uma alteração não relacionada pode invalidar um job mesmo quando suas entradas específicas não mudaram.

Aceitamos esse custo inicial por simplicidade e segurança.

No futuro, se profiling demonstrar necessidade, podemos validar por **dependency revisions**: revisions específicas das entradas realmente usadas pelo job.

Não implementar isso antecipadamente.

## Cancellation

Jobs longos precisam suportar **cancelamento cooperativo**.

Cooperativo significa que o algoritmo verifica periodicamente se ainda deve continuar e encerra em um ponto seguro.

~~~text
processa lote
↓
cancelled?
├── não → próximo lote
└── sim → encerra limpo
~~~

Não matar threads à força.

O intervalo entre verificações depende do custo de cada etapa: cancelamento deve responder rápido sem transformar cada operação microscópica em consulta ao token.

## Progress

Progresso de job é Session/Application State, não Document State.

Direção:

~~~rust
pub struct JobProgress {
    pub completed: u64,
    pub total: Option<u64>,
    pub phase: JobPhase,
}
~~~

Engine/job publica dados. QML decide como mostrar porcentagem, fase ou indicador indeterminado.

Jobs não chamam componentes Qt diretamente.

## Qt thread affinity

**Thread affinity** significa que um objeto está associado a uma thread e deve respeitar as regras de acesso dessa thread.

Objetos Qt como `QObject` e itens visuais como `QQuickItem` permanecem na thread de UI conforme as regras do Qt.

~~~text
Qt UI thread
├── QML
├── QObject / QQuickItem
├── foco e input
└── apresentação

Workers
├── geometry
├── image processing
├── text/layout computation
├── export
└── evaluation pesada
~~~

Worker retorna dados Rust/contratos Petunia para a fronteira de aplicação. A entrega para Qt acontece depois, de forma compatível com o modelo de threading do Qt.

Engine, Core e Render não carregam ponteiros Qt em jobs.

## Task concurrency e data parallelism

São problemas diferentes.

**Task concurrency** executa trabalhos independentes ao mesmo tempo:

~~~text
thumbnail
+
image trace
+
export
~~~

**Data parallelism** divide um mesmo trabalho em partes:

~~~text
1000 tiles
    ↓
workers
    ↓
resultado combinado
~~~

`rayon` é adequado principalmente para data parallelism e tarefas CPU-bound, mas não define a arquitetura de jobs.

**CPU-bound** significa que o custo principal é computação no processador, e não espera por disco, rede ou UI.

## Classes de trabalho

Usar três classes conceituais:

| Classe | Objetivo | Exemplos |
|---|---|---|
| **Interactive** | proteger latência da interação | hit-test, snapping, viewport evaluation |
| **Background** | trabalho útil sem bloquear edição | thumbnail, image trace, preload |
| **Batch** | throughput de operações grandes | export de muitas páginas, processamento em lote |

**Latência** é o tempo entre uma ação e a resposta percebida.

**Throughput** é a quantidade total de trabalho concluída em um período.

Para edição interativa, reduzir latência costuma ser mais importante do que maximizar throughput.

> Trabalho Background ou Batch nunca deve saturar a máquina a ponto de tornar pointer, snapping ou viewport visivelmente lentos.

A implementação concreta de scheduler fica para o tópico de jobs/performance. Aqui definimos somente a política.

## Scheduler e threading concreto

O **scheduler** decide qual trabalho executa, com qual prioridade e quando um resultado deve ser descartado ou entregue.

A arquitetura distingue autoridade lógica de thread física.

~~~text
Qt Main Thread
├── QML / QObject / input / presentation
│
├── Document Commit Lane
│   ├── Transaction
│   ├── History
│   ├── Revision
│   └── publish snapshot
│
└── Worker Pool
    ├── Geometry
    ├── Raster
    ├── Trace
    ├── Effects
    ├── Layout
    └── Export
~~~

**Document Commit Lane** é uma autoridade lógica: o caminho único pelo qual commits autorais entram no Document.

Na v0.1 ela não precisa ser uma thread própria. Pode executar no fluxo principal da aplicação, desde que cálculos caros aconteçam fora desse caminho.

### Frame budget

**Frame budget** é o tempo disponível para produzir uma atualização visual antes que a interface comece a perder fluidez.

Em uma tela a 60 Hz:

~~~text
1 segundo / 60
≈ 16,67 ms por frame
~~~

Esse tempo também é usado por Qt/QML, renderização e sistema operacional. Portanto uma operação executada diretamente no caminho de input deve ter latência pequena e mensurável.

Não congelar metas numéricas como “snap < 2 ms” sem benchmark. A regra é medir o caminho interativo e impedir trabalho pesado nele.

### Scheduler mínimo

O scheduler da v0.1 precisa apenas de conceitos próprios do Petunia:

~~~text
JobId
JobClass
Cancellation
DocumentRevision
Progress
Result
~~~

Não criar scheduler distribuído, dependency graph genérico ou work-stealing próprio.

### Interactive não significa assíncrono

Uma operação Interactive barata, como ranking de poucos candidatos de snap, pode executar imediatamente.

~~~text
operação de microssegundos
→ executa diretamente
~~~

Transformar cada cálculo pequeno em job, channel e callback pode custar mais que o próprio cálculo.

### Rayon não é Job API

`rayon` resolve data parallelism CPU.

Ele pode aparecer dentro de um job:

~~~text
Petunia Job
    ↓
algoritmo
    ↓
rayon divide tiles/objetos
~~~

Evitar chamadas de `rayon::spawn` espalhadas por UI e ferramentas como mecanismo de jobs do produto.

### Jobs substituíveis

Previews caros podem ficar obsoletos antes de terminar.

~~~text
Blur 10 → job A
Blur 15 → job B
Blur 25 → job C
~~~

Se A e B representam o mesmo preview que C substitui, processar todos até o fim desperdiça CPU.

Uma futura `ReplacementKey` pode identificar jobs cujo resultado mais novo substitui o anterior.

Isso é uma capacidade planejada quando surgir o primeiro caso real; não é requisito estrutural do scheduler MVP.

### Backpressure

**Backpressure** impede que trabalho seja produzido mais rápido do que o consumidor consegue processar.

~~~text
120 eventos/s
↓
cada preview custa 40 ms
↓
fila cresce indefinidamente
✗
~~~

Em previews interativos, normalmente vale a política:

> **latest state wins** — o estado mais recente vale mais que processar uma fila histórica de previews obsoletos.

O scheduler pode cancelar, substituir ou descartar trabalho antigo conforme o tipo de operação.

### Debounce e throttle

**Debounce** espera a atividade parar antes de executar.

~~~text
evento evento evento
        ↓ pausa
      executa
~~~

**Throttle** permite execução contínua, mas limita sua frequência.

~~~text
100 eventos/s
     ↓
limite
     ↓
20 atualizações/s
~~~

Usar apenas onde a semântica permitir:

- busca textual pode usar debounce;
- progress visual pode usar throttle;
- pointer geometry normalmente não deve usar debounce porque atrasaria a interação.

### Progress

Progress é feedback, não telemetria de cada iteração interna.

Um worker pode atualizar seu progresso frequentemente, mas a UI não precisa receber milhares de mensagens por segundo.

A frequência de publicação deve ser limitada quando necessário para evitar que feedback vire nova fonte de contenção.

### Render thread

A arquitetura **não define uma thread física fixa para Render**.

Qt Quick pode possuir seu próprio modelo de render thread conforme backend e configuração. Headless/CPU render pode executar em workers.

A regra é somente:

> Render é logicamente independente da UI e trabalha sobre estado de leitura; a thread física pertence à implementação do backend.

### Snapshot por revision, não por frame

Não criar deep copy do Document em todo frame.

~~~text
revision não mudou
→ reutiliza snapshot

revision mudou
→ publica snapshot novo
~~~

Preview também não cria revision autoral artificial:

~~~text
Snapshot 54
+
TransientOverrides
+
Overlays
→ frame
~~~

### Erros de worker

Worker nunca mostra diálogo ou manipula QML diretamente.

Ele retorna erro tipado. A camada de aplicação decide se deve:

- ignorar cancelamento esperado;
- tentar novamente;
- recalcular;
- mostrar mensagem ao usuário;
- registrar diagnóstico.

### Invariantes do scheduler

1. Qt Main Thread é dona dos objetos Qt e da apresentação.
2. Document Commit Lane é autoridade lógica e não exige thread dedicada na v0.1.
3. Trabalho caro não executa no caminho da UI.
4. O scheduler distingue Interactive, Background e Batch.
5. Operação Interactive pequena pode executar diretamente.
6. `rayon` implementa paralelismo interno; não é a Job API do produto.
7. Jobs são identificáveis e canceláveis quando longos.
8. Resultados voltam por message passing.
9. A thread física de Render depende do backend.
10. Snapshot é reutilizado enquanto a revision não muda.
11. Preview usa snapshot base + estado transitório.
12. Filas aplicam backpressure; previews obsoletos podem ser descartados.

## Message passing

**Message passing** significa trocar mensagens ou resultados entre componentes em vez de permitir acesso livre ao mesmo estado mutável.

~~~text
Document Owner
     │ request
     ▼
   Worker
     │ result
     ▼
Document Owner
~~~

Não congelamos nesta página qual crate de channel será usada.

O contrato importa mais que o mecanismo.

## Renderer

Render também trabalha sobre estado de leitura.

Direção:

~~~rust
fn render(
    snapshot: &RenderSnapshot,
    frame: &FrameContext,
    target: &mut RenderTarget,
) -> Result<RenderStats>;
~~~

`RenderSnapshot` é uma direção arquitetural, não uma crate ou formato já congelado.

Nunca:

~~~rust
fn render(document: &mut Document)
~~~

Render pode possuir caches internos mutáveis porque eles são Derived State, não estado autoral.

## Cache e sincronização

Antes de criar um cache global com lock, perguntar se ele realmente precisa ser compartilhado.

Preferir quando adequado:

- scratch memory por thread;
- cache local de job;
- cache de propriedade exclusiva do renderer;
- dados imutáveis compartilhados.

**Scratch memory** é memória temporária reutilizada durante cálculos para evitar alocações repetidas.

Sincronização compartilhada entra quando houver benefício medido, não como padrão automático.

## Lifetime de snapshots

Aqui, **lifetime** significa por quanto tempo o snapshot permanece retido em memória.

Jobs concluídos ou cancelados devem liberar suas referências:

~~~text
job termina
↓
drop snapshot
↓
partes sem outras referências podem ser liberadas
~~~

Não criar um registry global que retenha indefinidamente todas as revisions.

Histórico e snapshots possuem necessidades diferentes: manter uma HistoryEntry não implica manter para sempre um snapshot completo daquela revision.

## Invariantes de concorrência

1. O Document autoritativo possui um único escritor.
2. Leitores concorrentes recebem snapshots imutáveis.
3. Um snapshot representa exatamente uma `DocumentRevision`.
4. Snapshot runtime não é serialização PTND.
5. Workers nunca recebem tipos Qt ou `&mut Document`.
6. Jobs retornam dados tipados; alterações voltam pela Transaction API.
7. Na v0.1, jobs mutadores validam contra a revision documental completa.
8. Jobs longos suportam cancelamento cooperativo.
9. `rayon` é mecanismo de paralelismo, não arquitetura.
10. Trabalho Interactive possui prioridade sobre Background e Batch.
11. Caches compartilhados continuam sendo Derived State.
12. `Arc<Mutex<Document>>` não é o modelo geral de concorrência do Petunia.

## Vocabulário obrigatório

**Source** é o dado original. **Operation** é uma edição parametrizada. **Evaluation** calcula o resultado de operações. **Bake** materializa um resultado e perde a capacidade de editar etapas anteriores. **Snapshot** é uma visão imutável de uma revisão. **Overlay** é desenho de interface sobre o canvas, como handles e guias, e não faz parte do documento.

## Decisões que exigem ADR

**ADR — Architecture Decision Record** é um documento curto que registra uma decisão arquitetural importante, seu contexto, alternativas consideradas e consequências.

Criar um ADR quando mudarem: representação canônica de paths; formato PTND; modelo de efeito; estratégia de scene graph; color management/CMM; modelo de raster tiles; renderer GPU; threading; ABI de plugins; ou qualquer dependência que atravesse crates.

## Critério de teste de fronteira

**Headless** significa executar sem janela, display server ou interface gráfica.

Pergunte: “consigo executar isto headless?”. Boolean, snapping, import, export, text layout e filtros devem funcionar sem Qt e sem criar uma janela. Se uma operação matemática precisar de `QObject`, `QQuickItem` ou outro tipo Qt, a fronteira está errada.
