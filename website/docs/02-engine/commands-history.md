# Commands + History

Toda mutação autoral precisa ser undoable e observável.

## Limitação do modelo atual

O `Command` atual guarda o estado anterior dentro de cada objeto de comando. Funciona no MVP, mas operações compostas, drag contínuo e jobs assíncronos exigem Transaction.

## Transaction

Uma **Transaction** agrupa várias mutações que devem ser tratadas como uma única ação do usuário.

Exemplo: mover três objetos juntos deve produzir um único Undo, mesmo que internamente altere três transforms.

```rust
pub struct Transaction {
    pub id: TransactionId,
    pub label: String,
    pub ops: Vec<DocumentOp>,
    pub inverse_ops: Vec<DocumentOp>,
    pub merge_key: Option<MergeKey>,
}
```

`DocumentOp` é uma mutação pequena, tipada e validável: InsertNode, RemoveNode, SetTransform, SetEffectParam, ReorderChild, ReplacePath etc.

## Atomicidade

**Atomicidade** significa “tudo ou nada”.

Se uma Transaction possui 10 operações e a operação 7 falha, o documento não pode permanecer com as 6 primeiras aplicadas.

~~~text
10 operações
   ↓
todas válidas? ── sim → commit
   │
   não
   ↓
nenhuma alteração permanente
~~~

Falha no meio não pode deixar Scene parcialmente alterada.

## Preview e commit

Ferramentas interativas usam:
- begin
- transient update
- render
- commit ou cancel.

O estado transient pode viver numa overlay de sessão. Commit produz uma única Transaction.

## Coalescing

**Coalescing** junta várias ações pequenas e consecutivas em uma única entrada de histórico.

Exemplo: arrastar um slider produz muitos valores intermediários, mas o usuário espera um único Undo.

Digitação, nudges e sliders podem se fundir quando:
- mesmo merge key
- mesma entidade/propriedade
- janela temporal adequada
- nenhum outro command interposto.

O Core/Engine não precisa conhecer “mouse move”.

## Redo

Novo Command após undo limpa redo branch. Se futuramente quisermos history tree, isso vira feature explícita; não complicar MVP.

## Dirty checkpoint

**Dirty** significa que o documento possui mudanças ainda não salvas.

No save, registramos a revisão atual como **checkpoint**:

`is_dirty = current_revision != saved_revision`.

Não precisamos manter um boolean manual que possa ficar fora de sincronia.

## Commands e plugins

UI, script, plugin e macro chamam a mesma Command API. Isso garante undo, validação e automação consistentes.

## Long jobs

Operação cara:
1. captura snapshot + expected revision
2. calcula em background
3. retorna resultado
4. valida se inputs ainda correspondem
5. cria Transaction
6. commit no thread dono do documento.

Se a revisão mudou, recomputar ou pedir resolução; nunca aplicar resultado obsoleto silenciosamente.

## Histórico e memória

Raster paint não deve guardar cópia integral da imagem por brush dab. Usar tile diffs/copy-on-write. Operações vetoriais guardam inverse ops ou estruturas anteriores pequenas.
