# IDs, identidade e serialização

Identidade persistente é necessária para undo/redo, referências, symbols, masks, plugins, import/export e evolução segura do formato PTND.

A regra principal é:

> **ID responde “quem é esta entidade?”, não “onde ela está armazenada?”.**

## IDs fortemente tipados

Entidades diferentes usam wrappers diferentes mesmo quando compartilham a mesma representação interna.

~~~rust
pub struct DocumentId(Uuid);
pub struct ObjectId(Uuid);
pub struct NodeId(Uuid);
pub struct ContourId(Uuid);
pub struct EffectId(Uuid);
pub struct AppearanceItemId(Uuid);
pub struct ResourceId(Uuid);
pub struct StyleId(Uuid);
pub struct SymbolId(Uuid);
pub struct SpotColorId(Uuid);
pub struct SwatchId(Uuid);
pub struct GuideId(Uuid);
pub struct GridId(Uuid);
pub struct PageId(Uuid);
pub struct SpreadId(Uuid);
pub struct SliceId(Uuid);
~~~

Isso impede usar, por acidente, um `ResourceId` em uma API que espera `ObjectId`.

## ObjectId, NodeId e ContourId

`ObjectId` identifica um objeto do SceneGraph. `ContourId` identifica um contour dentro de um VectorPath. `NodeId` identifica um node geométrico dentro desse contour.

~~~text
Scene Object
ObjectId = O1
└── VectorPath
    ├── ContourId = C1
    │   ├── NodeId = N1
    │   ├── NodeId = N2
    │   └── NodeId = N3
    └── ContourId = C2
~~~

Mover N2 mantém `NodeId = N2`. Alterar coordenadas não cria nova identidade.

## UUID v4

**UUID — Universally Unique Identifier** é um identificador de 128 bits com probabilidade extremamente baixa de colisão sem depender de um contador global.

A v0.1 usa UUID v4 para identidade persistente.

Razões:

- geração local sem coordenação;
- adequado para copy/import e documentos independentes;
- fácil de representar em texto;
- não exige storage central;
- evita acoplar identidade à ordem.

Não migrar para UUID v7, ULID ou IDs incrementais sem benefício concreto.

## ID não define ordem

UUID não é z-order, índice, posição no arquivo ou ordem cronológica.

~~~text
Group
children:
1. Object X
2. Object Q
3. Object B
~~~

A sequência de `children` define ordem autoral. Os valores dos UUIDs são irrelevantes.

Nunca ordenar SceneGraph por UUID para inferir pintura.

## ID não é índice

Não serializar como identidade:

- posição em `Vec`;
- pointer;
- endereço de memória;
- arena slot;
- hash table bucket;
- runtime handle.

Esses valores são detalhes de execução.

## Identidade sobrevive a mutações

A mesma entidade mantém ID ao mover, renomear, alterar transform, alterar aparência, reparent ou reorder.

Nova entidade semântica recebe novo ID.

| Operação | Política |
|---|---|
| mover objeto | mantém ID |
| mover node | mantém ID |
| reordenar efeito | mantém EffectId |
| duplicar | novos IDs |
| copy/paste | novos IDs |
| criar node | novo NodeId |
| delete + Undo | restaura IDs antigos |
| Expand criando novos paths | novos IDs para novas entidades |

## Undo restaura identidade

Se `ObjectId = A` é removido e Undo restaura o objeto, ele volta como A.

~~~text
Delete A
↓
A não está no branch ativo

Undo
↓
A é restaurado
~~~

Não gerar ID novo no Undo.

## IDs persistentes não são reciclados

Uma identidade removida não é reutilizada para representar outra entidade.

UUID é barato; reciclar identidade traz risco de referências stale sem benefício material.

## Duplicate, clone e copy/paste

Clonar uma estrutura exige **ID remapping**.

**ID remapping** cria um mapa entre identidades antigas e novas.

~~~text
old Object A → new Object A2
old Object B → new Object B2
old Effect E → new Effect E2
~~~

Depois, referências internas do clone são reescritas usando o mesmo mapa.

Direção de API:

~~~rust
pub struct IdRemap {
    // representação interna não congelada
}
~~~

O remap conhece cada categoria tipada; não transformar tudo em `Uuid` genérico.

## Copy/paste entre documentos

Colar em outro documento cria entidades novas.

~~~text
Document A
ObjectId O1
      ↓ copy/paste
Document B
ObjectId O9
~~~

Mesmo que o conteúdo seja idêntico, a identidade autoral é nova.

## Import

Import de formatos externos também cria IDs Petunia novos, exceto ao restaurar PTND nativo cuja identidade já existe no arquivo.

IDs estrangeiros podem ser preservados como metadata de interoperabilidade quando útil, mas não substituem IDs Petunia.

## Runtime handles

Performance pode justificar uma representação runtime adicional.

~~~rust
pub struct RuntimeHandle {
    pub index: u32,
    pub generation: u32,
}
~~~

`RuntimeHandle` nunca é identidade persistente.

~~~text
ObjectId
↓ lookup runtime
RuntimeHandle
↓
storage
~~~

## Generational Arena

Uma **arena** guarda objetos em slots indexados.

~~~text
slot 0 → A
slot 1 → B
slot 2 → C
~~~

Se B é removido e o slot 1 for reutilizado para D, um índice simples antigo poderia acessar D pensando que ainda é B.

Uma **generation** resolve isso:

~~~text
B = (index 1, generation 4)
remove B

D = (index 1, generation 5)

handle antigo (1,4)
≠
handle atual (1,5)
~~~

Um handle antigo inválido é chamado **stale handle**.

### Decisão

**◐ Em avaliação.**

Generational Arena não é requisito da v0.1. Começar com a representação mais simples que satisfaça comportamento e performance. Migrar para arena somente se profiling ou ergonomia estrutural justificarem.

## Referências persistentes

Relações internas usam IDs tipados.

~~~rust
pub struct SymbolInstance {
    pub definition: SymbolId,
}
~~~

Não usar `Uuid` cru quando o tipo do alvo é conhecido.

## Dangling reference

Uma **dangling reference** aponta para uma entidade interna que não existe mais.

~~~text
Mask → ObjectId A
mas A não existe
~~~

Um Document válido não contém dangling references em relações que exigem alvo existente.

Delete/Reparent precisam atualizar, rejeitar ou materializar dependências através de Transaction explícita.

Cada relação define sua política; comportamento implícito é proibido.

## Unresolved resource

Uma referência externa indisponível é diferente.

~~~text
ResourceId R
✓ existe no Document

linked file
✗ não encontrado
~~~

O Document continua estruturalmente válido. O resource fica **unresolved** e pode ser reparado por Relink.

## ResourceId e ContentHash

`ResourceId` identifica o recurso lógico.

`ContentHash` identifica conteúdo binário.

~~~text
ResourceId
→ qual recurso é este?

ContentHash
→ estes bytes são iguais?
~~~

Duas Resources diferentes podem compartilhar o mesmo ContentHash.

Hash ajuda em cache, deduplicação, change detection e integridade. Hash nunca substitui identidade autoral.

## PTND: três camadas distintas

Separar:

~~~text
Domain Model
≠
Persistent DTO
≠
Physical Container
~~~

### Domain Model

Tipos usados pelo Core durante execução.

### Persistent DTO

**DTO — Data Transfer Object** é uma estrutura criada especificamente para transportar/serializar dados.

~~~rust
pub struct DocumentDtoV1 {
    pub schema_version: SchemaVersion,
    pub document_id: String,
    pub scene: SceneDtoV1,
}
~~~

Refatorar structs internas não deve automaticamente mudar o formato PTND.

### Physical Container

O arquivo físico que guarda manifest, documento e resources.

Essas três camadas evoluem de forma independente.

## SchemaVersion

`SchemaVersion` é versão do formato de dados, independente da versão do aplicativo.

~~~text
Petunia 0.4.2 → schema 3
Petunia 0.5.0 → schema 3
Petunia 0.6.0 → schema 4
~~~

Uma release não cria schema novo sem mudança persistente real.

## Pipeline de leitura

~~~text
bytes
 ↓
container safety validation
 ↓
manifest + schema detection
 ↓
DTO structural validation
 ↓
migration chain
 ↓
Current DTO
 ↓
domain construction
 ↓
domain invariant validation
 ↓
Document
~~~

Não fazer `serde_json::from_str::<Document>` diretamente em conteúdo não confiável como arquitetura final.

## Structural validation

**Structural validation** verifica se a representação externa é segura e plausível antes de construir o domínio.

Exemplos:

- campos obrigatórios;
- tipos;
- tamanho de strings;
- quantidade de nodes;
- profundidade;
- dimensões raster;
- tamanho descompactado;
- quantidade de effects;
- nesting de import.

## Domain validation

**Domain validation** verifica regras semânticas do Core.

Exemplos:

- IDs únicos;
- parent existe;
- ausência de ciclos;
- referências obrigatórias resolvidas;
- contours válidos;
- transforms finitos;
- Scene invariants.

## Migration chain

Migrações são explícitas.

~~~text
V1 DTO
↓ migrate
V2 DTO
↓ migrate
V3 DTO
↓ build domain
Document
~~~

Uma migration é determinística e testável com fixtures reais.

## PTND como container

Modelo conceitual:

~~~text
document.ptnd
├── manifest.json
├── document.json
├── resources/
│   ├── images/
│   ├── fonts/
│   └── profiles/
└── previews/
    └── thumbnail.webp
~~~

### manifest.json

Manifest é pequeno e barato de ler.

Responsabilidades candidatas:

- formato/magic lógico;
- SchemaVersion;
- DocumentId;
- capabilities obrigatórias;
- índice/resumo de entries;
- hashes quando aplicável.

Não duplicar o Document inteiro no manifest.

### Container físico

**✅ Definido para a v0.1: ZIP/ZIP64 compatível.**

PTND usa ZIP como container multi-entry e aceita ZIP64 quando tamanho/quantidade de entries ultrapassar limites clássicos.

A escolha é deliberadamente conservadora:

- formato amplamente suportado;
- inspecionável sem ferramenta proprietária;
- implementação madura em várias plataformas;
- entries independentes para JSON, resources e previews;
- permite streaming de leitura por entry;
- combina bem com escrita temporária + atomic replace.

A v0.1 aceita reescrever o container completo no save. Incremental save interno não é requisito inicial e não justifica um container proprietário.

Detalhes físicos completos estão em [Formato PTND](#/docs/00-architecture/ptnd-format.md).

## Segurança de container

Arquivo criativo é input não confiável.

Limites obrigatórios:

- número de entries;
- tamanho por entry;
- total descompactado;
- compression ratio;
- strings;
- dimensões raster;
- profundidade estrutural.

### Decompression bomb

Uma **decompression bomb** é pequena comprimida, mas expande para volume enorme.

~~~text
20 MB
↓ decompress
300 GB
~~~

Nunca extrair sem limites.

### Path traversal

Entries como:

~~~text
../../../../etc/passwd
~~~

tentam escapar do diretório de destino.

Paths internos precisam ser normalizados/validados. PTND loader não escreve arbitrariamente paths fornecidos pelo arquivo.

## Unknown fields e extensões

Política depende do contexto.

### Core schema

Campo obrigatório incompatível ou versão futura crítica pode exigir erro de compatibilidade.

### Extensões namespaced

Dados de extensão usam namespace estável para evitar colisões.

~~~text
com.example.plugin/filter-state
~~~

Quando seguro, o host pode preservar payload desconhecido para roundtrip.

### Opaque blob

**Opaque blob** é conteúdo que o Core preserva sem interpretar.

Preservar não significa executar, carregar código, desserializar objeto arbitrário ou confiar.

## Representação de UUID no arquivo

Usar string UUID canônica.

~~~json
{
  "id": "a9518df6-5f42-49b6-a784-d13356e8f5aa"
}
~~~

Política:

- lowercase;
- hífens canônicos;
- parse estrito;
- serialize sempre na mesma forma.

## Ordem de serialização

Coleções com ordem autoral preservam essa ordem.

Coleções sem ordem semântica recebem ordem canônica antes de serializar para evitar diffs aleatórios.

Não deixar iteração de HashMap definir output. Não ordenar tudo por UUID indiscriminadamente.

## Save seguro

Persistência não deve truncar o único arquivo válido antes de terminar o novo save.

~~~text
arquivo atual permanece válido
        ↓
write temporary container
        ↓
flush / validate
        ↓
atomic replace
~~~

**Atomic replace** minimiza o período em que o destino não possui versão válida.

Detalhes variam por sistema operacional e pertencem ao I/O Engine.

## Invariantes

1. Persistent IDs são fortemente tipados.
2. UUID v4 é identidade persistente na v0.1.
3. ID nunca representa ordem, pointer, índice ou arena slot.
4. A mesma entidade mantém ID através de edições normais.
5. Undo restaura a identidade original.
6. Nova entidade semântica recebe novo ID.
7. Duplicate/copy/import usam ID remapping quando criam entidades novas.
8. Runtime handles nunca são serializados.
9. Generational Arena permanece em avaliação.
10. Document válido não possui dangling references internas obrigatórias.
11. External resources podem permanecer unresolved.
12. DTO persistente é separado do Domain Model.
13. SchemaVersion é independente da versão do aplicativo.
14. Load separa structural validation de domain validation.
15. PTND v0.1 usa ZIP/ZIP64 multi-entry; evolução futura exige migration/compatibility policy.
16. UUIDs usam representação textual canônica.
17. ContentHash não substitui identidade.
18. Save deve permitir escrita temporária + replace seguro.
