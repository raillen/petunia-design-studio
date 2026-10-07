# IDs e serialização

Identidade persistente é necessária para undo, plugins, symbols, masks, references e colaboração futura.

## IDs fortemente tipados

Manter wrappers separados:

```rust
DocumentId
ObjectId
NodeId
ContourId
EffectId
ResourceId
StyleId
SymbolId
GuideId
PageId
```

Isso impede usar acidentalmente `ResourceId` onde a API espera `ObjectId`.

## UUID

O projeto atual usa UUID v4. É adequado para identidade local/distribuída e deve continuar até existir motivo concreto para mudar. Ordenação temporal não deve ser requisito oculto do ID.

## Índice não é ID

`Vec` index, pointer e arena key são detalhes de runtime. Podem coexistir com UUID para performance, mas nunca entram como referência persistente pública.

## PTND

Separar:
- modelo Rust
- schema persistente
- container físico.

Para evolução, recomendar:

```text
document.ptnd
├── manifest.json
├── document.json
├── resources/
│   ├── images/...
│   └── profiles/...
└── previews/
    └── thumbnail.webp
```

O container pode ser ZIP-like no futuro. JSON permanece excelente para metadata/schema; grandes rasters ficam binários.

## SchemaVersion

Toda carga passa por:

```text
detect version
→ validate structural safety
→ migrate version N → current
→ deserialize current DTO
→ validate domain invariants
→ build Document
```

Não fazer `serde_json::from_str::<Document>` diretamente em arquivos não confiáveis no modelo final.

## DTO versus domínio

Schemas externos podem ter DTOs próprios. Isso permite renomear/refatorar structs Rust sem quebrar o formato do arquivo.

## Unknown fields

Decidir por contexto:
- formato nativo: preservar extensões namespaced quando possível.
- estruturas críticas: rejeitar versões futuras incompatíveis.
- plugin payload: preservar blob opaco se plugin ausente, sem executá-lo.

## Segurança

Definir limites de:
- número de nodes
- profundidade da árvore
- tamanho de strings
- dimensão raster
- bytes descompactados
- quantidade de effects
- recursion/import nesting.

Arquivo criativo é input não confiável.
