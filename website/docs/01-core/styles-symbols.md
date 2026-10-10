# Styles e Symbols

Styles e Symbols reutilizam intenção autoral sem duplicar dados.

Eles são Document State, possuem identidade estável e precisam continuar previsíveis mesmo quando centenas de objetos dependem deles.

## Princípio

~~~text
Style
→ reutiliza propriedades

Symbol
→ reutiliza estrutura
~~~

Style não é Symbol pequeno. Symbol não é um Style com children.

## Styles

A v0.1 possui três famílias semânticas:

~~~text
AppearanceStyle
CharacterStyle
ParagraphStyle
~~~

Cada uma usa `StyleId`, mas o tipo esperado continua explícito nas APIs.

Não usar um payload sem tipo apenas porque todos vivem no mesmo registry.

## StyleRegistry

Direção:

~~~rust
pub struct StyleRegistry {
    // lookup por StyleId
}
~~~

Registry resolve identidade. Ordem de apresentação, quando necessária, vive em coleção separada.

HashMap iteration nunca define palette/panel order.

## Style identity

Dois styles visualmente idênticos continuam entidades diferentes enquanto possuírem IDs diferentes.

~~~text
Style A = blue fill
Style B = blue fill

A != B
~~~

Não deduplicar styles automaticamente por igualdade visual.

Isso preserva intenção: o usuário pode querer editar A sem afetar B depois.

## Linked versus local

Objeto pode ter:

~~~text
Local
Linked Style
Linked Style + typed overrides
~~~

Para Appearance:

~~~rust
pub enum AppearanceSource {
    Local(Appearance),
    Style(StyleId),
    StyleWithOverrides {
        style: StyleId,
        overrides: AppearanceOverrides,
    },
}
~~~

Text styles seguem princípio equivalente em seu domínio.

## Overrides

Override é diferença explícita sobre o style base.

Não armazenar “propriedade arbitrária por string” para built-ins.

Direção:

~~~text
AppearanceItemId
+
typed property/value
→ override
~~~

Isso permite validar:

- alvo existente;
- tipo de propriedade;
- valor;
- migration.

## Style update

Editar Style é uma única mutação autoral no registry.

~~~text
Style S revision changes
↓
all linked consumers become stale
↓
evaluation/cache invalidation
~~~

Não copiar o style para cada objeto.

History guarda a alteração do Style, não centenas de mutations redundantes nos consumers.

## Detach Style

**Detach Style** materializa o valor resolvido no objeto.

~~~text
Style + overrides
↓ resolve
Local Appearance/Character/Paragraph data
↓ remove link
~~~

É Command explícito e undoable.

O Style original continua existindo no registry enquanto outras referências existirem.

## Style inheritance

A v0.1 **não possui inheritance entre Styles**.

Não:

~~~text
Style C extends Style B extends Style A
~~~

Motivos:

- evita ciclos;
- reduz cascata difícil de prever;
- simplifica serialization/migration;
- linked style + typed override já resolve o caso principal.

Inheritance só entra futuramente se existir necessidade real que não seja bem atendida por composição/overrides.

## Missing Style

Referência interna a StyleId inexistente é dangling reference e torna o Document inválido.

Import de formato externo que não consegue preservar style deve:

- materializar localmente;
- criar Style novo;
- ou emitir warning/fallback explícito.

Não deixar StyleId fictício.

## Swatches em Styles

Styles podem referenciar SwatchId.

~~~text
Style
↓ Paint
↓ SwatchId
~~~

Editar Swatch propaga para Style/objetos ligados através da dependency graph.

Isso não muda StyleId.

## Resources em Styles

Pattern, fonts e outros payloads podem referenciar ResourceId.

Resource GC considera referências vindas dos Styles.

## Symbols

`SymbolDefinition` é uma estrutura autoral reutilizável armazenada no `SymbolRegistry`.

~~~rust
pub struct SymbolDefinition {
    pub id: SymbolId,
    pub name: String,
    pub roots: Vec<ObjectId>,
    // subtree autoral
}
~~~

Os ObjectIds internos são globais no Document, conforme a política de identidade já definida.

## SymbolInstance

~~~rust
pub struct SymbolInstance {
    pub definition: SymbolId,
    pub overrides: SymbolOverrides,
}
~~~

Instance não copia a subtree da definition.

Evaluation produz subtree derivada para Render/queries.

## Definition versus Instance transform

Definition possui transforms locais próprios.

O SceneNode da SymbolInstance adiciona o transform da instância.

~~~text
definition local transforms
↓
instance SceneNode transform
↓
world transform
~~~

Editar posição da instance não reescreve a definition.

## Nested Symbols

SymbolDefinition pode conter SymbolInstance.

Isso permite composição reutilizável:

~~~text
Button Symbol
↓ contains
Icon Symbol
~~~

Mas o grafo de Symbol dependencies precisa ser acíclico.

Inválido:

~~~text
Symbol A → instance of B
Symbol B → instance of A
~~~

Core rejeita a Transaction que criaria o ciclo.

## Overrides de Symbol

A v0.1 permite overrides built-in tipados e não estruturais.

Famílias iniciais:

~~~text
TextContent
Paint/Color
Visibility
Resource
~~~

Direção conceitual:

~~~rust
pub struct SymbolOverride {
    pub target: ObjectId,
    pub value: SymbolOverrideValue,
}
~~~

`target` aponta para uma entidade estável dentro da definition.

Não usar property path textual como:

~~~text
"children[4].fill.color"
~~~

porque reorder/refactor quebraria o override silenciosamente.

## Structural overrides

Adicionar/remover/reparent arbitrary child dentro de uma Instance não entra na v0.1.

Isso criaria uma segunda árvore estrutural parcialmente divergente da definition.

Quando o usuário precisar estrutura independente, usa **Detach Symbol**.

Essa restrição mantém o modelo previsível.

## Target removido da Definition

Se uma edição da SymbolDefinition remove um objeto que possui overrides em Instances:

~~~text
definition removes target T
↓
same Transaction validates dependent overrides
↓
overrides targeting T are removed
~~~

History mantém dados inversos suficientes para Undo restaurar target + overrides.

Não manter dangling override ativo.

Uma implementação futura pode oferecer migração assistida, mas não é requisito do modelo.

## Editar Definition

Editar SymbolDefinition altera uma única source autoral.

Instances reavaliam de forma derivada.

~~~text
Definition revision changes
↓
dependent instances invalidated
↓
RenderSnapshot rebuild as needed
~~~

Não materializar mudanças em cada instance.

## Detach Symbol

Detach materializa a aparência/estrutura atual da instance.

Política v0.1:

1. avaliar definition + overrides;
2. substituir `SceneItem::SymbolInstance` por `Group` no mesmo SceneNode;
3. preservar o ObjectId da instance como root/group quando ela continua sendo a mesma entidade na cena;
4. materializar descendants com novos ObjectIds;
5. preservar Appearance/Text/Resource semantics resolvidas;
6. remover vínculo com SymbolId;
7. guardar inverse data para Undo restaurar a SymbolInstance original.

Isso evita trocar desnecessariamente a identidade do objeto que o usuário já selecionou/referenciou.

## Duplicate de SymbolInstance

Duplicar instance:

~~~text
new SceneNode ObjectId
same SymbolId
copied overrides
~~~

A definition não é duplicada dentro do mesmo Document.

Copy/paste para outro Document usa DocumentFragment dependency closure e remap policy.

## Duplicate de SymbolDefinition

Duplicar a própria definition cria:

~~~text
new SymbolId
new ObjectIds internos
rewritten internal references
~~~

Instances existentes continuam referenciando a definition original.

## Delete de Definition

Não deletar SymbolDefinition com instances vivas de forma implícita.

Command precisa escolher política explícita, por exemplo:

~~~text
DeleteDefinitionIfUnused
DetachInstancesAndDelete
~~~

A v0.1 não oferece “delete and leave broken instances”.

## Symbol appearance

Appearance pertence aos objetos internos da definition normalmente.

A SymbolInstance não ganha uma Appearance paralela genérica só para “tint”.

Mudanças globais na instance usam overrides tipados quando suportadas.

## Symbol cache

Expanded/evaluated symbol subtree é Derived State.

Cache key pode incluir:

~~~text
SymbolId
definition revision
instance overrides revision/key
color/resource dependencies
quality
~~~

Nunca persistir expanded subtree como fonte da instance.

## Styles dentro de Symbols

Definition pode usar Styles/Swatches/Resources normais do Document.

Dependency closure do Symbol inclui essas referências para copy/package/validation.

Não criar registry privado invisível por Symbol.

## Serialization

Persistir:

- Style definitions;
- Symbol definitions;
- Instance references;
- typed overrides;
- stable IDs.

Não persistir:

- resolved style copies;
- expanded symbol subtree;
- render primitives;
- cache revisions runtime.

## Validation

Document válido garante:

- StyleId único;
- SymbolId único;
- ObjectIds internos de symbols únicos globalmente;
- referências de Style existentes;
- SymbolInstance definition existente;
- symbol dependency DAG sem ciclos;
- override targets existentes;
- override value compatível com target/property;
- Resource/Swatch refs válidas.

## History

Editar uma definition/style é uma Transaction única.

Undo restaura a source e consumers voltam a avaliar o estado anterior.

Não criar HistoryEntry por consumer derivado.

## Invariantes

1. Style reutiliza propriedades; Symbol reutiliza estrutura.
2. Styles possuem identidade, não são deduplicados por aparência.
3. Linked e Local são estados distintos.
4. Overrides built-in são tipados.
5. Style inheritance fica fora da v0.1.
6. Style update não copia dados para cada consumer.
7. SymbolInstance referencia definition; expanded subtree é derivada.
8. Nested Symbols são permitidos, ciclos não.
9. Overrides de Symbol v0.1 são não estruturais.
10. Override target usa ObjectId estável, não property path textual.
11. Remover target da definition resolve overrides na mesma Transaction.
12. Detach Symbol preserva o ObjectId da instance/root e cria IDs novos para descendants materializados.
13. Deletar SymbolDefinition com instances exige política explícita.
14. Styles/Symbols participam normalmente de dependency closure, resource GC, copy/paste e History.

## Verificação da fiação estrutural (2026-10-10)

Escopo: `SpotRegistry`, subtrees de definição com ciclos reais e effect stacks no node. Revisão `7806e2c2d9d3b63ee1b05677b9ba94c7513f99a2` sobre branch `petunia-design-rust`.

| Gate executado | Resultado |
|---|---|
| `cargo test --workspace` | pass: 375 passed / 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `node website/scripts/verify-progress.cjs` | pass |

Riscos/limites: avaliadores por kind de efeito e render de gradients/patterns seguem futuros, com avisos explícitos até lá.
