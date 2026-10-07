# Commands + History

Toda mutação autoral do Petunia precisa ser **explícita, validável, atômica, observável e reversível**.

A regra central é:

> **UI, scripts e plugins expressam intenção. O Engine transforma essa intenção em uma Transaction. Somente a Transaction altera o Document.**

## Quatro conceitos diferentes

Não usar “Command” para representar todas as etapas da edição.

| Conceito | Responsabilidade |
|---|---|
| **Command** | intenção semântica, como “mover objetos” ou “converter em curvas” |
| **DocumentOp** | mutação documental pequena e tipada |
| **Transaction** | conjunto atômico de DocumentOps |
| **HistoryEntry** | registro que permite Undo/Redo e navegação entre estados documentais |

Fluxo:

~~~text
UI / Script / Plugin
        ↓
      Command
        ↓
 Command Handler
        ↓
Transaction Request
        ↓
      Prepare
        ↓
PreparedTransaction
        ↓
  Atomic Commit
    ↙    ↓    ↘
Document History Revision
~~~

## Command

Um **Command** representa intenção do usuário ou de uma automação.

Exemplos:

~~~text
MoveObjects
DeleteSelection
SetFillColor
ConvertToCurves
ExpandAppearance
AddGuide
ReorderLayers
~~~

Evitar Commands que descrevam detalhes estruturais como `SetVecIndex4` ou `UpdateHashMapEntry`. Eles acoplam consumidores à implementação.

Uma forma possível:

~~~rust
pub enum DocumentCommand {
    MoveObjects(MoveObjectsCommand),
    DeleteObjects(DeleteObjectsCommand),
    SetFill(SetFillCommand),
    ConvertToCurves(ConvertToCurvesCommand),
}
~~~

A escolha entre enum, handlers registrados ou outra representação ainda pode evoluir. A semântica é a parte definida.

## Command Handler

O **Command Handler** recebe a intenção, consulta o estado atual, executa cálculos necessários no Engine e produz uma Transaction.

~~~text
MoveObjects { A, B, C, delta }
            ↓
         snapping
            ↓
        constraints
            ↓
        Transaction
       ├─ SetTransform(A)
       ├─ SetTransform(B)
       └─ SetTransform(C)
~~~

A UI não precisa saber quantas mutações internas são necessárias.

## DocumentOp

Uma **DocumentOp** é uma mutação estrutural pequena e explicitamente representada.

~~~rust
pub enum DocumentOp {
    InsertNode {
        parent: ObjectId,
        index: usize,
        node: SceneNode,
    },

    RemoveSubtree {
        root: ObjectId,
    },

    SetTransform {
        object: ObjectId,
        transform: Transform2D,
    },

    ReplacePath {
        object: ObjectId,
        path: VectorPath,
    },

    SetEffectParameter {
        effect: EffectId,
        parameter: EffectParameter,
    },

    ReorderChild {
        parent: ObjectId,
        child: ObjectId,
        index: usize,
    },
}
~~~

DocumentOp é API interna de mutação. QML não monta DocumentOps diretamente.

~~~text
QML / Tool
    ↓ intenção
Command
    ↓
Engine
    ↓
DocumentOp
    ↓
Core
~~~

## Transaction

Uma **Transaction** agrupa todas as mutações que formam uma única alteração autoral.

Mover três objetos juntos pode alterar três transforms, mas para o usuário continua sendo:

~~~text
1 ação
↓
1 Transaction
↓
1 HistoryEntry
↓
1 Undo
~~~

Modelo conceitual:

~~~rust
pub struct TransactionRequest {
    pub command_id: CommandId,
    pub operations: Vec<DocumentOp>,
    pub merge_key: Option<MergeKey>,
}
~~~

## Atomicidade

**Atomicidade** significa “tudo ou nada”.

Se uma Transaction contém dez operações e a sétima é inválida, o documento não pode manter as seis primeiras.

~~~text
10 operações
    ↓
prepare + validate
    ↓
todas válidas?
 ┌──┴──┐
não   sim
 ↓      ↓
erro  commit completo
~~~

Estado parcial é erro arquitetural.

## Prepare → Commit

Evitar aplicar operações enquanto ainda estamos descobrindo se elas são válidas.

### Prepare

A fase **Prepare** trabalha sobre leitura do estado atual e executa tudo que puder falhar de forma previsível:

- validar IDs e referências;
- validar invariantes;
- calcular geometria;
- gerar IDs novos quando necessário;
- capturar dados necessários ao Undo;
- construir inverse operations;
- registrar objetos afetados;
- conferir a revisão-base.

~~~rust
pub fn prepare_transaction(
    document: &Document,
    request: TransactionRequest,
) -> Result<PreparedTransaction, TransactionError>;
~~~

### Commit

Depois de preparada, a Transaction possui informação suficiente para uma aplicação curta e previsível:

~~~rust
pub fn commit_transaction(
    document: &mut Document,
    transaction: PreparedTransaction,
) -> Result<AppliedTransaction, CommitError>;
~~~

O objetivo é que Commit tenha pouquíssimas razões legítimas para falhar.

Se surgir uma inconsistência inesperada, o sistema interrompe a operação, restaura o estado anterior usando os dados inversos disponíveis e registra diagnóstico. Estado parcial nunca é apresentado como sucesso.

## PreparedTransaction

Direção de modelo:

~~~rust
pub struct PreparedTransaction {
    pub expected_revision: DocumentRevision,
    pub forward: Vec<DocumentOp>,
    pub inverse: Vec<DocumentOp>,
    pub affected_objects: Vec<ObjectId>,
    pub command_id: CommandId,
    pub merge_key: Option<MergeKey>,
}
~~~

### expected_revision

Uma Transaction é preparada contra um estado documental específico.

~~~text
job começa em revision 51
        ↓
usuário edita documento
        ↓
documento chega à revision 52
        ↓
job termina com expected_revision = 51
~~~

O commit detecta:

~~~text
expected = 51
current  = 52
→ RevisionConflict
~~~

O chamador pode recalcular, descartar ou pedir resolução explícita conforme a operação. Resultado obsoleto nunca é aplicado silenciosamente.

## Inverse operations

Undo precisa saber como recuperar o estado anterior.

~~~text
SetTransform
old = A
new = B
~~~

gera inversa:

~~~text
SetTransform
transform = A
~~~

Para remoção:

~~~text
Forward
RemoveSubtree(A)

Inverse
InsertSubtree(
    subtree = A,
    parent = previous_parent,
    index = previous_index
)
~~~

O histórico guarda os dados necessários para reverter. Não copiar o Document inteiro quando uma representação menor é suficiente.

## HistoryEntry

Uma entrada de histórico representa a ligação entre dois estados documentais.

~~~rust
pub struct HistoryEntry {
    pub transaction: AppliedTransaction,
    pub before_revision: DocumentRevision,
    pub after_revision: DocumentRevision,
    pub merge_key: Option<MergeKey>,
    pub description: HistoryDescription,
}
~~~

~~~text
Revision 30
   │ Transaction A
   ▼
Revision 31
   │ Transaction B
   ▼
Revision 32
~~~

Undo de B aplica sua inversa e volta para Revision 31. Redo reaplica o forward e retorna para Revision 32.

## DocumentRevision

**DocumentRevision identifica um estado autoral do histórico.**

Ela não significa simplesmente “quantas operações aconteceram nesta sessão”.

~~~text
Revision 10 ← saved
    ↓
Revision 11
    ↓
Revision 12
    ↓
Revision 13
~~~

Depois de Undo:

~~~text
current_revision = 12
~~~

Outro Undo:

~~~text
current_revision = 11
~~~

Se uma nova edição ocorrer nesse ponto:

~~~text
Revision 10
    ↓
Revision 11
    ├────────→ Revision 12 → Revision 13   antigo redo branch
    │
    └────────→ Revision 14                 nova edição
~~~

Na v0.1, o redo branch antigo é descartado quando a nova edição é commitada. History tree só entra se virar feature explícita.

## Dirty state

**Dirty** significa que existem alterações autorais ainda não salvas.

O save registra:

~~~text
saved_revision = current_revision
~~~

E:

~~~text
is_dirty = current_revision != saved_revision
~~~

Undo pode tornar o documento clean novamente ao retornar exatamente à revisão salva.

Não manter um boolean `is_dirty` autoritativo separado.

## HistoryDescription e localização

O Engine não guarda texto localizado como “Mover objetos” dentro da semântica do histórico.

Preferir identidade estável:

~~~rust
pub enum HistoryDescription {
    MoveObjects,
    DeleteObjects,
    SetFill,
    ConvertToCurves,
}
~~~

ou `HistoryActionId` equivalente.

A UI traduz essa identidade para o idioma atual ao apresentar History.

## Preview

Ferramentas interativas não alteram o Document a cada movimento do ponteiro.

~~~text
pointer down
    ↓
begin transient edit

pointer move
    ↓
replace/update preview

pointer move
    ↓
replace/update preview

pointer up
    ↓
Command
    ↓
Transaction
    ↓
1 Commit
~~~

Um drag longo continua produzindo uma única alteração autoral.

### Transient Edit

Uma representação inicial pode ser estreita:

~~~rust
pub enum TransientEdit {
    Transform(TransformPreview),
    Path(PathPreview),
    EffectParameter(EffectPreview),
    BrushStroke(BrushPreview),
}
~~~

Não criar um `PreviewDocument` completo enquanto não houver necessidade real.

## Preview e Commit usam a mesma matemática

Preview e resultado final não podem ter algoritmos independentes.

~~~text
Move Tool
   ↓
Engine transform + constraints + snapping
          ↙                 ↘
       Preview             Commit
   transient result       DocumentOp
~~~

A diferença é o destino do resultado, não a matemática.

Isso evita o erro em que o objeto “pula” para outra posição quando o usuário encerra a interação.

## Cancel

Como Preview não alterou o documento autoritativo:

~~~text
Escape / cancel
      ↓
discard TransientEdit
      ↓
Document permanece igual
~~~

Não é necessário criar Undo para cancelar algo que nunca foi commitado.

## Coalescing

**Coalescing** junta vários commits pequenos e consecutivos em uma única experiência de Undo.

É útil principalmente para:

- digitação;
- nudges repetidos por teclado;
- backspace/delete repetidos.

Condições típicas:

~~~text
mesmo merge_key
+ mesmo alvo
+ mesma propriedade/operação
+ ações consecutivas
+ janela temporal apropriada
~~~

### Não usar coalescing para esconder preview mal modelado

Slider, drag e outras interações contínuas devem preferir:

~~~text
begin preview
→ updates transitórios
→ 1 commit
~~~

em vez de gerar muitos commits e tentar fundi-los depois.

### Save checkpoint

Nunca fazer coalescing atravessando uma revisão que foi salva.

Caso contrário, uma entrada anterior poderia ser reescrita depois do save e tornar ambígua a relação entre `saved_revision` e o estado persistido.

## Single writer

**Single writer** significa que existe apenas um fluxo com autoridade para alterar o Document.

Isso não significa que o programa só usa uma thread.

~~~text
workers
→ snapshots somente leitura
→ resultados

document owner / commit lane
→ valida
→ aplica Transaction
→ atualiza History/Revision
~~~

Na v0.1, esse owner pode ser o serviço de documento coordenado pela aplicação. A arquitetura não deve depender de espalhar `Arc<Mutex<Document>` pela aplicação.

## Background jobs

Jobs caros trabalham sobre snapshots:

~~~text
Document Revision 40
        ↓
     Snapshot
        ↓
 Background Job
        ↓
      Result
expected_revision = 40
        ↓
 Document Owner
        ↓
revision ainda é 40?
  ┌─────┴─────┐
 não          sim
  ↓            ↓
conflict     prepare + commit
~~~

`rayon` pode executar cálculo paralelo. Ele não recebe `&mut Document`.

## Core continua protegendo invariantes

Command/Engine valida a operação do ponto de vista funcional, mas o Core continua rejeitando estados estruturalmente inválidos.

~~~text
Engine
→ usuário pediu reparent A para B

Core
→ B é descendente de A?
→ CycleDetected
~~~

Uma falha de UI, plugin ou Command Handler não pode permitir ciclo estrutural apenas porque uma validação anterior foi esquecida.

Campos estruturais do SceneGraph não devem ficar publicamente mutáveis por conveniência.

## Plugins, scripts e macros

Plugins e scripts usam a mesma Command API da UI:

~~~text
Qt/QML ───┐
Script ───┤
Plugin ───┼→ Command → Engine → Transaction → Core
CLI ──────┘
~~~

Eles não recebem `&mut Document`.

### Macro recorder

Macro registra intenção semântica:

~~~text
CreateRectangle
SetFill
MoveObjects
Duplicate
~~~

e não eventos crus de mouse ou `pointer move`.

Isso torna macros mais estáveis e reutilizáveis.

## Erros

Separar erros por estágio:

~~~rust
pub enum CommandError {
    InvalidSelection,
    UnsupportedOperation,
    InvalidGeometry,
}

pub enum TransactionError {
    RevisionConflict,
    InvariantViolation,
    MissingObject(ObjectId),
}
~~~

A UI converte os erros para mensagem humana e pode anexar detalhes técnicos para diagnóstico.

## Histórico e memória

Operações vetoriais normalmente conseguem guardar inverse operations ou pequenas estruturas anteriores.

Raster exige política diferente: um brush stroke não deve copiar uma imagem inteira. O histórico raster deve trabalhar com tiles, diffs ou copy-on-write conforme definido no Raster Engine.

## Invariantes

1. Toda mutação autoral entra por Command/Transaction.
2. Command representa intenção; DocumentOp representa mutação interna.
3. Transaction é atômica.
4. Preview nunca modifica o Document autoritativo.
5. Preview e Commit reutilizam a mesma matemática do Engine.
6. Background jobs operam sobre snapshots, nunca sobre `&mut Document`.
7. O Document possui um único escritor autoritativo.
8. Undo/Redo navega entre revisions de estados documentais.
9. Plugins, scripts e macros usam a mesma Command API.
10. Core valida invariantes mesmo quando Engine já validou a operação.
