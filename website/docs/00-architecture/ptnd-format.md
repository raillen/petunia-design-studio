# Formato PTND

PTND é o formato nativo do Petunia Design Studio.

Seu objetivo é **roundtrip autoral lossless**: salvar e reabrir sem perder identidade, estrutura, intenção, parâmetros, recursos ou editabilidade suportada pelo Petunia.

> PTND não é um dump da memória do processo. É um contrato persistente versionado.

## Estrutura física v0.1

A v0.1 usa **ZIP/ZIP64 compatível** como container.

~~~text
document.ptnd
├── manifest.json
├── document.json
├── resources/
│   ├── images/
│   ├── fonts/
│   ├── profiles/
│   └── blobs/
├── extensions/
└── previews/
    └── thumbnail.webp
~~~

ZIP64 é permitido quando tamanho do arquivo, quantidade de entries ou tamanho individual ultrapassar limites clássicos do ZIP.

Não depender de extensions proprietárias do container quando uma representação ZIP/ZIP64 padrão for suficiente.

## Por que ZIP

A decisão privilegia:

- simplicidade;
- inspeção manual;
- interoperabilidade com tooling comum;
- leitura seletiva por entry;
- suporte maduro em várias plataformas;
- implementação segura mais fácil que um container próprio;
- compatibilidade com escrita temporária + replace atômico.

A v0.1 aceita reescrever o container inteiro durante save. Incremental save interno fica para uma evolução futura se profiling demonstrar necessidade.

## manifest.json

Manifest precisa ser pequeno e barato de ler.

Direção:

~~~json
{
  "format": "petunia-design-document",
  "schema_version": 1,
  "document_id": "a9518df6-5f42-49b6-a784-d13356e8f5aa",
  "required_capabilities": [],
  "entries": []
}
~~~

Responsabilidades:

- identificar PTND sem depender apenas da extensão;
- declarar SchemaVersion;
- declarar DocumentId;
- listar capabilities obrigatórias;
- fornecer metadata mínima para validar entries importantes;
- permitir leitura preliminar sem parsear o documento inteiro.

Não duplicar SceneGraph, estilos ou metadata autoral extensa no manifest.

## document.json

`document.json` contém o DTO persistente principal.

Ele descreve:

- Document setup;
- Pages/Spreads;
- SceneGraph;
- Styles;
- Symbols;
- Swatches;
- Guides/Grids;
- Slices;
- Resource records;
- referências para entries binárias;
- payloads de extensão conhecidos pelo schema.

Não contém caches ou estruturas runtime.

## JSON canônico

A serialização precisa ser estável.

Regras:

- UTF-8;
- nomes de campos definidos pelo schema;
- UUID lowercase com hífens;
- floats com representação decimal round-trip;
- `-0.0` canonicalizado para `0.0`;
- ordem autoral preservada onde é semântica;
- coleções semanticamente não ordenadas serializadas em ordem canônica;
- nenhum NaN/Inf.

O objetivo é reduzir diffs inúteis e tornar fixtures/recovery/testes reproduzíveis.

## Metadata ZIP

Metadata do filesystem host não entra como semântica do documento.

Entry timestamps, permissions e atributos de plataforma devem ser normalizados quando não forem semanticamente necessários.

`created` e `modified` do documento vivem no modelo/manifest apropriado, não no timestamp acidental da entry ZIP.

Isso melhora determinismo de save.

## Resources

Blobs grandes não ficam base64 dentro de `document.json`.

~~~text
ResourceRecord
↓ logical ResourceEntryRef
resources/images/...
~~~

O path físico interno é detalhe do DTO/container e não vira identidade do Resource.

`ResourceId` continua sendo a identidade lógica.

## ContentHash

A v0.1 usa hash de conteúdo **BLAKE3-256** para integridade, cache e deduplicação quando hash for necessário.

A representação persistente é algorithm-tagged:

~~~text
blake3:<hex>
~~~

Isso evita tornar a escolha do algoritmo implicitamente eterna.

Hash não substitui ResourceId.

BLAKE3 é escolhido por desempenho, streaming eficiente e implementação Rust pequena/focada. O uso aqui é integridade/content identity, não assinatura digital.

## Compressão

A política física pode variar por entry sem mudar o domínio.

Direção v0.1:

- JSON/texto → DEFLATE;
- formatos já comprimidos como JPEG/PNG/WebP → STORE quando recompressão não trouxer benefício;
- blobs/tile data → compressão lossless definida pelo writer conforme tipo;
- nunca usar compressão lossy para PixelLayer autoral.

A escolha de compressão de uma entry não faz parte da identidade do conteúdo.

## Entries e paths

Paths internos:

- são relativos;
- usam separador canônico do container;
- não começam por `/`;
- não contêm `..`;
- não são symlinks;
- não são usados diretamente como path do filesystem host.

Loader rejeita path traversal e entries ambíguas/case-conflicting quando a plataforma puder produzir interpretação insegura.

## Limites de segurança

Antes de extrair/decomprimir, validar:

- número de entries;
- tamanho comprimido;
- tamanho descompactado individual;
- tamanho total descompactado;
- compression ratio;
- profundidade de path;
- tamanho de nome;
- quantidade de resources;
- dimensões declaradas de raster.

Limites pertencem à aplicação/configuração operacional; o parser sempre possui guardrails.

## Capabilities

`required_capabilities` declara features sem as quais o documento não pode ser interpretado fielmente.

Exemplos conceituais:

~~~text
core.symbols.v1
core.spot-color.v1
plugin:com.example.effect@2
~~~

Capability conhecida mas não implementada pode resultar em abertura read-only/degradada conforme contrato.

Capability desconhecida marcada como obrigatória não pode ser ignorada silenciosamente.

Extensões opcionais preserváveis podem existir sem entrar em `required_capabilities`.

## Extensions

Dados de plugins/extensões ficam namespaced.

~~~text
extensions/
└── com.example.plugin/
    └── ...
~~~

Payload pode ser JSON ou blob opaco conforme contrato.

Load nunca executa código apenas porque payload existe.

Unknown optional payload pode ser preservado para roundtrip quando seguro.

## Preview

`previews/thumbnail.webp` é opcional e descartável.

Precisa declarar ou ser associado a uma revision/content identity para o loader saber se está stale.

Preview nunca é fonte autoral.

## Load

~~~text
file
↓ ZIP structural validation
manifest.json
↓ format/schema/capabilities
entry limits + integrity checks
document.json
↓ DTO structural validation
migration chain
↓ current DTO
domain construction
↓ Core invariant validation
Document
~~~

Resources podem ser resolvidos lazy quando a feature permitir, mas manifest/document precisam ser validados antes de expor Document como carregado com sucesso.

## Save

~~~text
Document Snapshot
↓ DTO adapter
Current DTO
↓ canonical serialization
temporary PTND
↓ finish ZIP + flush
minimum validation
↓ platform-safe atomic replace
destination
~~~

O arquivo anterior continua intacto até a nova versão estar completa.

## Atomic replace

**Atomic replace** é a troca do arquivo antigo pelo novo usando primitive segura do sistema operacional quando disponível.

Não significa que todo filesystem remoto garanta atomicidade perfeita.

A implementação precisa:

1. escrever arquivo temporário no mesmo filesystem/diretório quando possível;
2. finalizar e flushar conteúdo;
3. validar manifest/container mínimo;
4. substituir o destino;
5. manter diagnóstico se o replace falhar.

Nunca truncar o único PTND válido antes de o novo estar pronto.

## Compatibilidade

Loader precisa distinguir:

- schema suportado;
- schema antigo migrável;
- schema futuro conhecido apenas parcialmente;
- schema futuro incompatível.

Não “tentar parsear mesmo assim” quando uma versão futura muda semântica obrigatória.

## Determinismo

Dois saves do mesmo estado autoral podem diferir em metadata explicitamente temporal, como `modified`.

Fora isso, ordem e representação devem ser canônicas sempre que possível.

Não exigir arquivo ZIP bit-a-bit idêntico se compressor/backend produzir metadata binária diferente, mas o conteúdo lógico deve permanecer determinístico.

## Recovery

Autosave/recovery não escreve diretamente por cima do PTND principal.

Recovery usa armazenamento próprio e está detalhado em [Autosave e Recovery](#/docs/02-engine/persistence-recovery.md).

## Invariantes

1. PTND é ZIP/ZIP64 compatível na v0.1.
2. `manifest.json` é pequeno e obrigatório.
3. `document.json` contém DTO, não structs runtime.
4. Resources grandes ficam em entries binárias.
5. JSON é canonicalizado e preserva round-trip numérico.
6. Entry paths nunca escapam do container.
7. Loader aplica limites antes da descompressão irrestrita.
8. BLAKE3-256 é o ContentHash inicial, com algoritmo explicitamente identificado.
9. PixelLayer autoral nunca usa compressão lossy.
10. Preview é descartável.
11. Load não executa plugin.
12. Save usa temporary file + atomic replace.
13. SchemaVersion é independente da versão da aplicação.
14. Mudança física futura do container exige compatibilidade/migration explícita.
