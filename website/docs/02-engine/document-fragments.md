# Fragmentos de documento e Copy/Paste

Copy/paste não deve copiar structs runtime nem depender da posição atual do SceneGraph.

O Engine usa um **DocumentFragment**: um pacote transitório e serializável contendo entidades autorais suficientes para reconstruir uma seleção em outro contexto.

> Clipboard do sistema é integração de plataforma. O fragmento Petunia é o contrato de motor.

## Objetivos

Um fragmento precisa permitir:

- duplicar dentro do mesmo documento;
- copiar entre documentos;
- copiar entre instâncias do Petunia;
- preservar relações internas;
- remapear IDs;
- carregar resources/styles/symbols necessários;
- rejeitar referências externas inválidas;
- evitar colidir com identidade já existente.

## DocumentFragment

Direção:

~~~rust
pub struct DocumentFragment {
    pub roots: Vec<ObjectId>,
    pub scene: FragmentScene,
    pub resources: Vec<ResourceRecord>,
    pub styles: Vec<StyleRecord>,
    pub symbols: Vec<SymbolDefinition>,
    pub swatches: Vec<Swatch>,
    pub metadata: FragmentMetadata,
}
~~~

Essa estrutura é DTO de interchange interno, não uma mini-Document autoritativa.

## Closure de dependências

Ao copiar roots, o Engine calcula a **dependency closure**.

Dependency closure é o conjunto transitivo de entidades necessárias para que o fragmento continue semanticamente válido.

Exemplo:

~~~text
Path A
↓ usa Style S
↓ Style S usa Swatch C
↓ Pattern usa Resource R
~~~

O fragmento precisa carregar S, C e R quando a política exigir conteúdo self-contained.

Não copiar o Document inteiro por conveniência.

## Relações internas

Se dois objetos copiados referenciam um ao outro, a relação é preservada no fragmento.

~~~text
A masked by B
A + B copiados
→ relação preservada após remap
~~~

Se A é copiado sem B, a política depende do tipo de relação.

Nunca manter dangling reference silenciosa.

## Política por referência

Cada referência persistente declara comportamento de copy:

~~~text
OwnedDependency
SharedDependency
ExternalDependency
OptionalReference
~~~

Direção conceitual:

- **OwnedDependency** — deve ser incluída/remapeada;
- **SharedDependency** — pode ser reutilizada no destino se identidade compatível existir, senão copiar;
- **ExternalDependency** — preserva referência externa quando permitido;
- **OptionalReference** — pode ser removida com warning se semântica permitir.

A classificação concreta é definida por tipo, não por flags genéricas espalhadas.

## ID remapping

No paste/duplicate:

~~~text
old ObjectId O1 → new ObjectId N1
old NodeId   A1 → new NodeId   B1
old EffectId E1 → new EffectId F1
~~~

Todas as referências internas são reescritas usando o mesmo remap.

IDs persistentes do fragmento nunca são inseridos diretamente quando representariam novas entidades.

Undo do paste remove as novas entidades; Redo restaura **os mesmos IDs novos** criados no primeiro commit.

## Duplicate no mesmo documento

Duplicate usa o mesmo pipeline de fragmento/remap, mas pode otimizar sem serialização textual.

~~~text
selection
↓ build fragment
↓ remap
↓ insert near source
↓ Transaction
~~~

Isso evita uma implementação separada de clone que diverge do copy/paste.

## Paste entre documentos

O destino controla:

- Parent/Page de inserção;
- posição/offset;
- policy de styles/swatches existentes;
- resource dedup;
- conflitos de capability.

O fragmento não carrega pointer/runtime handle da origem.

## Resources

ResourceId novo não significa necessariamente bytes duplicados.

Se o destino já possui conteúdo com mesmo ContentHash:

~~~text
Resource logical entity nova
→ pode apontar para blob físico compartilhado/deduplicado
~~~

A decisão de dedup física não muda a identidade autoral.

Linked resource pode permanecer linked se a URI/policy for válida no destino; caso contrário o paste pode incorporar, marcar unresolved ou exigir policy explícita.

## Styles e Swatches

Não fazer merge automático apenas porque dois styles “parecem iguais”.

Identidade de Style/Swatch é semântica.

Políticas possíveis:

~~~text
CopyAsNew
ReuseExactKnownIdentity
ReuseByExplicitUserChoice
~~~

Na v0.1, copy entre documentos cria identidade nova salvo quando o fragmento referencia uma entidade explicitamente reconhecida como compartilhável pelo formato/host.

Isso evita heurística visual destrutiva.

## Symbols

Copiar SymbolInstance exige decidir se a SymbolDefinition necessária acompanha o fragmento.

Regra v0.1:

- definition pertencente ao mesmo fragmento/destino reconhecido → remap/reuse coerente;
- definition ausente no destino → incluir definition + subtree na dependency closure;
- cycles continuam proibidos após remap.

Não expandir symbol automaticamente só para simplificar paste.

## Text e fonts

FontRef autoral é preservado.

Fonte incorporada necessária pode entrar como Resource conforme policy/licença.

Fonte instalada externa não precisa ser copiada como bytes só porque foi usada.

Missing font continua resolvível pelo Text Engine sem reescrever FontRef.

## Page-local coordinates

Fragmento usa um **fragment origin** derivado dos roots copiados.

Direção:

~~~text
source geometry in Page space
↓ subtract fragment origin
fragment-local placement
↓ paste destination origin
destination Page space
~~~

Transforms internos são preservados.

Isso evita que colar de uma página distante mantenha coordenadas absolutas absurdas.

## Paste in place

Paste in place pode preservar a posição Page-local quando source/destination possuem contexto compatível.

É uma policy de Command, não outro formato de fragmento.

## Serialization interna

Para clipboard entre processos/instâncias, DocumentFragment possui schema/version próprios.

Não reutilizar diretamente `document.json` inteiro.

Direção de MIME/application type:

~~~text
application/x-petunia-fragment
~~~

O payload pode usar JSON canônico + blobs/resources associados conforme integração de clipboard permitir.

Formato do clipboard externo é integração de plataforma; o DTO do fragmento continua independente.

## Interchange fallback

Clipboard pode também publicar formatos externos derivados:

~~~text
SVG
PNG
plain text quando selection for texto
~~~

Esses são fallbacks de interoperabilidade.

Ao colar, Petunia prefere o fragmento nativo quando disponível e compatível porque preserva editabilidade.

A prioridade/UX final será discutida junto com GUI.

## Segurança

Fragmento vindo de outro processo é input não confiável.

Aplicar os mesmos princípios de import:

- limites de tamanho;
- DTO structural validation;
- UUID validation;
- recursion/dependency limits;
- resource limits;
- plugin payload não executado;
- path/URI safety.

Clipboard nunca ganha confiança especial só por ser local.

## Transaction

Paste materializa tudo em uma única Transaction:

~~~text
validate fragment
↓ remap IDs
↓ resolve dependencies
↓ prepare insertion
↓ atomic commit
~~~

Falha no resource/style número 20 não deixa os 19 anteriores inseridos.

## Provenance

FragmentMetadata pode manter informação diagnóstica/interchange como source application/schema, mas ela não vira identidade.

Não usar DocumentId original para inferir que objetos devem preservar ObjectId.

## Invariantes

1. Copy/paste usa DocumentFragment, não structs runtime.
2. Duplicate reutiliza o mesmo remap pipeline.
3. Dependency closure contém somente dependências necessárias.
4. Nova entidade recebe novo ID.
5. Relações internas são reescritas atomicamente.
6. Dangling references não são aceitas silenciosamente.
7. Resource blob pode ser deduplicado sem fundir identidade autoral.
8. Styles/Swatches não são fundidos por igualdade visual implícita.
9. Symbols preservam definition quando possível; não expandem automaticamente.
10. Clipboard externo é input não confiável.
11. Paste inteiro é uma Transaction.
12. Fragment schema é separado do schema completo de Document.

## Closure e paste verificados em 2026-10-10

Fragments coletam dependências transitivas de objetos, clip/mask/text refs, styles, fontes, resources, spots, swatches, símbolos, efeitos e grids. Remapping inclui IDs de contours, anchors e appearance items. Resources embedded recebem novos nomes de entry, evitando colisão dentro do PTND. Grids de Page são reassociados à página de destino.

Copy de um subtree preserva seu world transform; paste em grupo usa a inversa do world transform do destino para manter a posição no documento. Inserção de registries, grids e subtrees constitui uma transação com undo conjunto. O clipboard nativo da sessão transporta fragment e blobs; hashes, decode e budgets são verificados antes do commit. Linked resources preservam sua URI e não disparam leitura implícita.

Evidência: `history_fragments_regressions.rs` (12 testes) e `session_boundary.rs`. Policies de reuso externo e closure de novas entidades devem acompanhar seus futuros handlers. Gates em [Verification](#/docs/00-architecture/verification.md).
