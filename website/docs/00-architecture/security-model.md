# Modelo de segurança

O Petunia trata arquivos, recursos externos, plugins e automação como **input potencialmente não confiável**.

> Segurança não depende da UI perguntar “tem certeza?”. O motor precisa permanecer seguro mesmo quando recebe dados inválidos diretamente.

## Superfícies de confiança

~~~text
Trusted authorial state
        ↑
   validation boundary
        ↑
untrusted / external input
~~~

Entradas consideradas não confiáveis:

- PTND recebido de terceiros;
- SVG/PDF/PSD e outros imports;
- imagens;
- fontes;
- perfis ICC;
- clipboard/document fragments;
- plugins WASM;
- payloads de plugins;
- requests MCP;
- arquivos linked que mudaram externamente;
- metadata externa;
- dados vindos de processos auxiliares.

## Core é a última barreira

Parser/Engine pode validar antes, mas o Core continua protegendo invariantes fundamentais.

~~~text
External Input
↓ parser validation
DTO
↓ Engine validation
Command / Transaction
↓
Core invariant validation
↓
Document
~~~

Nenhuma camada anterior autoriza o Core a confiar cegamente.

## Safe Rust

Core, Engine e Render usam Rust seguro por padrão.

Direção:

~~~rust
#![forbid(unsafe_code)]
~~~

onde a crate não precisar de FFI/low-level integration.

Dependências externas podem internamente usar unsafe; por isso dependência é revisada por manutenção, escopo e segurança.

“Sem unsafe autoral” reduz risco, mas não é prova de ausência de bugs lógicos ou vulnerabilidades em dependências.

## FFI

**FFI — Foreign Function Interface** é a fronteira onde Rust conversa com código de outra linguagem/ABI.

No Petunia, o principal caso é Rust ↔ C++/Qt via CXX-Qt.

FFI fica confinada à borda de integração.

~~~text
Qt / C++
↓ CXX-Qt bridge
petunia-ui integration
↓ typed Petunia contracts
Core / Engine / Render
~~~

Não espalhar pointers Qt ou layouts C++ pelo domínio.

Se algum backend futuro exigir unsafe Rust autoral, ele precisa ficar em módulo/crate de bridge explicitamente auditável.

## Invariantes de FFI

Toda entrada de FFI precisa validar:

- nullability;
- length/range;
- enum discriminants;
- ownership/lifetime;
- thread affinity;
- encoding;
- integer conversion/overflow.

Nenhuma referência Rust pode sobreviver além do lifetime garantido pelo outro lado.

## Panics atravessando FFI

Panic Rust não deve atravessar uma ABI C/C++ não preparada para isso.

Bridge captura/trata a fronteira conforme o mecanismo seguro da integração e transforma falha em diagnóstico/erro controlado.

Da mesma forma, exception C++ não atravessa para Rust como comportamento implícito.

## Parsers

Parsers de input externo obedecem:

~~~text
bytes
↓ size limits
↓ structural parse
↓ semantic validation
DTO/result
~~~

Requisitos:

- sem allocations ilimitadas baseadas em valores declarados pelo arquivo;
- recursion depth limitada;
- dimensions/counts validados antes de multiplicações/alocações;
- arithmetic checked para tamanhos;
- time/complexity guards em formatos potencialmente patológicos;
- erro tipado em vez de panic.

## Integer overflow

Cálculos de tamanho usam arithmetic checked.

Exemplo:

~~~text
width × height × bytes_per_pixel
~~~

nunca usa multiplicação que possa overflow e depois alocar buffer menor que o necessário.

Falha retorna `SizeOverflow`/erro equivalente.

## Decompression

Container/resource compressed aplica limites antes de expansão irrestrita.

Validar:

- tamanho comprimido;
- tamanho declarado;
- total acumulado;
- compression ratio;
- quantidade de entries.

Decompression bomb não pode transformar um arquivo pequeno em consumo arbitrário de memória/disco.

## Filesystem paths

Path vindo de arquivo/plugin/MCP nunca é concatenado diretamente com diretório base.

Regras:

- rejeitar absolute path quando o contrato exige relativo;
- rejeitar `..` escapando do root;
- resolver canonicalização/normalização com cuidado;
- não seguir symlink fora do escopo concedido quando isso viola a policy;
- criar arquivos somente dentro do destino autorizado.

## TOCTOU

**TOCTOU — Time Of Check To Time Of Use** é o risco de algo mudar entre validação e uso.

Exemplo:

~~~text
verifica path
↓
outro processo troca symlink
↓
abre path
~~~

Quando a plataforma permitir, preferir handles/files já abertos/autorizados a validar string e reabrir depois.

Para linked resources, detectar mudança externa por metadata/content identity antes de sobrescrever conteúdo.

## Temporary files

Save/recovery temp files:

- são criados no diretório/storage apropriado;
- usam nomes não previsíveis quando segurança exigir;
- não seguem symlink arbitrário;
- têm permissions privadas quando contêm trabalho do usuário;
- são removidos best-effort após sucesso/falha.

## Permissions de arquivo

Recovery, logs com contexto e temp PTND podem conter informação sensível.

Na plataforma que suportar, criar com permissions restritivas ao usuário atual.

Não depender apenas de “pasta escondida”.

## Linked resources

Linked resource nunca ganha confiança permanente só porque foi válido no import.

O arquivo pode mudar.

~~~text
ResourceId
↓ resolve
external bytes
↓ validate/decode
derived resource
~~~

Reload passa novamente pelas validações/limits.

## Image decoders

Imagem declara dimensions antes de produzir pixels.

Aplicar limites de:

- width/height;
- total pixels;
- channels/bit depth;
- decoded bytes;
- metadata size.

Decoder externo fica atrás de adapter.

## Fonts

Font parser processa bytes não confiáveis.

Fontes incorporadas ou instaladas podem conter tabelas inválidas.

Shaping/raster/outlines passam por bibliotecas focadas e limites apropriados; não tratar fonte local como automaticamente segura.

## ICC profiles

ICC profile também é input binário complexo.

Color Management adapter valida tamanho/tipo antes de entregar ao Little CMS.

Falha do CMM/profile não altera cores autorais nem causa fallback silencioso para outro perfil.

## Plugins WASM

Plugin público roda sandboxed.

Deny-by-default para:

- filesystem;
- network;
- process;
- clipboard;
- UI extension.

Host limita:

- linear memory;
- fuel/time quando runtime suportar;
- stack;
- handles;
- message size;
- jobs;
- output size.

Plugin nunca recebe memória/pointer do Document.

## Native integration

Código nativo de terceiros não é carregado arbitrariamente in-process na v0.1.

Integração nativa futura usa processo separado + IPC quando possível.

Isso limita blast radius de crash/memory corruption.

## MCP

MCP é automação externa, não authority bypass.

Request MCP:

~~~text
transport
↓ authentication/session policy
↓ capability check
↓ typed Query/Command
↓ same Engine validation
~~~

MCP não ganha acesso direto a filesystem, Qt ou `&mut Document`.

Servidor é desabilitado por padrão no contrato atual.

Detalhes de consentimento/apresentação ficam para GUI/UX.

## Command authorization

Uma capability autorizada permite **pedir** uma operação.

Ela não ignora invariantes.

~~~text
plugin has command.submit
↓
submits DeleteObjects
↓
Engine validates
↓
Core validates
~~~

Permissions e correctness são camadas diferentes.

## Resource exhaustion

Ataques/inputs podem tentar consumir:

- CPU;
- RAM;
- disk;
- file descriptors;
- worker slots;
- history;
- cache;
- output size.

Cada subsistema caro possui budget/guard.

Falhar por limite é melhor que degradar o sistema inteiro.

## Algorithmic complexity

Geometria também pode ser input hostil.

Exemplo:

~~~text
milhares de segmentos
×
milhares de intersections
→ arrangement explosivo
~~~

Boolean, Shape Builder, Trace e layout possuem:

- cancellation;
- complexity counters;
- recursion guards;
- allocation guards;
- error diagnosticável.

Não impor limite autoral pequeno ao documento quando basta limitar uma operação.

## Denial of service por jobs

Plugin/MCP não pode criar jobs ilimitados.

Scheduler aplica quotas/capabilities e backpressure.

Interactive work mantém prioridade sobre Background/Batch.

## Serialization safety

Deserializer externo nunca constrói `Document` mutável diretamente.

~~~text
bytes
↓ DTO
↓ validation
↓ migration
↓ domain construction
~~~

Unknown plugin payload é preservado como dado, não executado.

## Secrets

Petunia não deve armazenar credentials/tokens dentro do PTND por conveniência.

Integrações que precisem secrets usam storage seguro da aplicação/plataforma e referenciam uma identidade lógica.

Export/Package não copia secrets.

## Logging

Logs não incluem conteúdo autoral ou secrets por padrão.

IDs/hashes/contexto técnico são preferidos.

Ver [Diagnósticos e observabilidade](#/docs/00-architecture/diagnostics-observability.md).

## Dependency security

Nova dependência precisa de:

- licença compatível;
- manutenção razoável;
- escopo controlável;
- versão pinada de forma reproduzível;
- advisories considerados no processo de release;
- adapter quando a dependência cruza domínio.

Não atualizar major version automaticamente sem testes/migration quando comportamento puder mudar.

## Supply chain

CI/release deve:

- usar lockfile;
- revisar mudanças relevantes de dependencies;
- gerar artefatos de forma reproduzível quando viável;
- evitar executar scripts externos não necessários;
- proteger signing/release credentials fora do repositório.

Signing de binários/packages é preocupação de release, não Core.

## Error disclosure

Mensagem apresentada ao usuário não precisa conter:

- stack trace;
- path interno sensível;
- memory address;
- secret;
- payload externo completo.

O diagnóstico técnico local pode manter mais contexto sanitizado.

## Security testing

Prioridades:

1. fuzz PTND/container;
2. fuzz DTO/migrations;
3. fuzz SVG/import adapters;
4. fuzz resource metadata boundaries;
5. fuzz plugin/MCP message decoding;
6. property tests de path normalization;
7. complexity/limit tests;
8. failure injection em save/recovery;
9. dependency/advisory review em release.

## Invariantes

1. Input externo nunca é confiável por origem.
2. Core continua sendo última barreira de invariantes.
3. Unsafe autoral é evitado e FFI fica confinada.
4. Panics/exceptions não atravessam FFI sem boundary controlada.
5. Tamanhos/alocações usam arithmetic checked e limits.
6. Paths externos não escapam de roots autorizados.
7. Resource reload revalida bytes.
8. WASM é deny-by-default e limitado.
9. MCP/plugins usam Query/Command, nunca bypass do domínio.
10. Native third-party code não roda arbitrariamente in-process na v0.1.
11. Jobs/algoritmos caros possuem guards contra resource exhaustion.
12. Secrets não entram no PTND.
13. Logs não incluem conteúdo sensível por padrão.
14. Security tests fazem parte dos quality gates.
