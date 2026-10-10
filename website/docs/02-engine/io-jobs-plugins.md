# I/O, Jobs e Plugins

Esses subsistemas ficam no Engine porque coordenam algoritmos e recursos externos sem pertencer à UI.

## Import

### Format sniffing

**Sniffing** identifica o formato pelo conteúdo real do arquivo, não apenas pela extensão.

Exemplo: um arquivo chamado `imagem.png` pode não conter PNG válido. O importer deve validar assinatura/header antes de confiar no nome.

Pipeline:

```text
bytes
→ sniff/format detection
→ parser seguro
→ formato intermediário
→ normalize
→ Core DTO  # estrutura de transferência validada antes do domínio
→ validate invariants
→ Document
```

**DTO — Data Transfer Object** é uma estrutura intermediária usada para transportar dados entre o parser e o domínio sem expor diretamente tipos externos.

SVG/PDF/PSD não devem vazar structs próprias para SceneGraph.

## Export

Export recebe snapshot imutável + `ExportSpec`.

```rust
pub struct ExportSpec {
    pub format: ExportFormat,
    pub area: ExportArea,
    pub color: ExportColorOptions,
    pub raster_scale: f64,
}
```

SVG/PDF preservam vetores quando possível; PNG/JPEG pedem raster output ao Render.

## Roundtrip

**Roundtrip** significa importar um arquivo, editar/salvar e exportar novamente preservando o máximo possível de sua estrutura e aparência.

Definir níveis:
- lossless nativo PTND
- high-fidelity import/export
- best-effort interchange.

Nunca prometer roundtrip perfeito para formato que não representa features Petunia.

## I/O contracts

Import/export usam interfaces Petunia e adapters por formato. Parser/encoder externo nunca retorna SceneNode diretamente.

~~~rust
pub trait Importer {
    fn probe(&self, input: &mut dyn ReadSeek) -> Result<ProbeResult>;
    fn import(&self, input: &mut dyn ReadSeek, ctx: &ImportContext) -> Result<ImportResult>;
}

pub trait Exporter {
    fn export(&self, snapshot: &DocumentSnapshot, spec: &ExportSpec, out: &mut dyn Write) -> Result<()>;
}
~~~

A assinatura concreta pode variar, mas os papéis ficam separados.

### Probe

`probe` lê somente bytes mínimos necessários para identificar formato/capabilities. Não decodifica um documento inteiro só para descobrir se o arquivo é suportado.

Resultado pode declarar confidence, format id e requisitos.

### ImportResult

Importer retorna DTO/intermediate validado + warnings estruturados.

~~~text
ImportedDocument
ImportedObjects
ImportedResource
~~~

Importar um SVG para o documento atual é diferente de abrir um PTND como novo Document.

### Warnings

Perda de fidelidade não é string solta.

Direção:

~~~text
UnsupportedFeature
RasterizedSubtree
FontSubstituted
ProfileUnavailable
UnknownMetadataPreserved
UnknownMetadataDropped
~~~

A UI futura decide como apresentar os warnings.

## Política de formatos

### PTND

É o único formato com objetivo de roundtrip autoral lossless.

Save usa DTO atual + container writer e escrita segura:

~~~text
snapshot/current DTO
↓
write temporary file
↓
finish container + flush
↓
validate minimum manifest/container integrity
↓
platform-safe replace
~~~

O arquivo antigo não é truncado antes do novo estar completo.

### SVG

SVG é formato vetorial de interchange de alta fidelidade, não formato nativo.

Import preserva paths, groups, transforms, fills/strokes, gradients, text e masks quando o modelo Petunia possui semântica equivalente. Features sem equivalente geram warning e fallback explícito.

Export tenta manter vetor. Um subtree que exija efeito não representável pode ser rasterizado isoladamente em vez de rasterizar a página inteira.

### PDF

PDF é principalmente formato de output/interchange final.

Export preserva texto/vetor/spot/profile quando o backend e a feature permitirem. Effects não representáveis podem gerar transparency groups ou rasterização localizada.

Import de PDF é best-effort e não promete reconstruir intenção editorial original; PDF descreve resultado gráfico, não necessariamente objetos de edição equivalentes.

### Raster images

PNG/JPEG/WebP e formatos suportados por decoder entram como ImageResource por padrão.

Import como PixelLayer é ação explícita quando o usuário quiser pixels editáveis materializados.

JPEG nunca é usado como storage lossless de PixelLayer autoral.

## Export capability negotiation

Cada exporter declara capabilities.

~~~text
vector paths
live text
gradients
spot colors
ICC profiles
transparency
blend modes
raster effects
multi-page
~~~

Antes do export, Engine percorre o snapshot e produz `ExportPlan`:

~~~text
preserve natively
convert
expand
rasterize subtree
warn/error
~~~

O exporter não decide silenciosamente como destruir uma feature.

## Raster export sizing

Raster export resolve tamanho por uma política explícita:

~~~text
document physical size + DPI
ou
pixel dimensions / scale
~~~

Não misturar Device Pixel Ratio da tela com DPI de export.

## Metadata e privacy

Exporters recebem policy explícita para metadata.

Informações como author, timestamps, EXIF/XMP importado ou custom metadata não são copiadas para todo formato automaticamente sem policy.

Isso evita vazamento acidental de metadata em assets publicados.

## I/O resource limits

Importers aplicam limites antes e durante parsing:

- bytes de input;
- tamanho descompactado;
- dimensions;
- object/node counts;
- recursion depth;
- string sizes;
- embedded resource counts;
- total embedded bytes.

Formato válido mas acima do limite operacional retorna erro diagnosticável; não tenta alocar até o sistema falhar.


## Jobs

Jobs são trabalhos potencialmente demorados que não devem bloquear a interação principal.

Exemplos:

- Image Trace;
- import pesado;
- geração de thumbnail;
- filtros demorados;
- export de muitas páginas;
- indexação de fontes/recursos.

### Contrato mínimo

Direção de API:

~~~rust
pub struct JobRequest<T> {
    pub id: JobId,
    pub class: JobClass,
    pub source_revision: DocumentRevision,
    pub payload: T,
}

pub enum JobClass {
    Interactive,
    Background,
    Batch,
}
~~~

O resultado preserva a origem:

~~~rust
pub struct JobResult<T> {
    pub id: JobId,
    pub source_revision: DocumentRevision,
    pub result: Result<T, JobError>,
}
~~~

Esses tipos são direção de contrato. Não exigem uma infraestrutura genérica complexa antes de existir uso real.

### Scheduler

O scheduler da v0.1 precisa:

~~~text
submit
cancel
progress
receive result
priority class
revision metadata
~~~

Ele não precisa implementar work stealing próprio, DAG genérica ou persistência de jobs.

`rayon` pode ser usado dentro dos algoritmos para data parallelism, mas não é a API de scheduling da UI.

### Cancelamento cooperativo

Jobs longos verificam um token de cancelamento em pontos seguros.

~~~text
executa bloco
↓
cancelado?
├── não → continua
└── sim → encerra e libera recursos
~~~

Nunca terminar uma thread à força como mecanismo normal de cancelamento.

### Jobs filhos

Se um job gera subtarefas, o cancelamento do pai precisa alcançar as subtarefas relevantes.

Exemplo:

~~~text
Image Trace
├── segmentation
├── contour extraction
└── curve fitting
~~~

Não é necessário criar agora uma árvore sofisticada de tokens; apenas não esconder trabalho filho impossível de cancelar.

### Backpressure e substituição

Jobs de preview podem se tornar obsoletos rapidamente.

~~~text
efeito radius 10
↓
radius 15
↓
radius 25
~~~

O resultado de 10 e 15 pode deixar de ter valor.

O scheduler deve permitir descartar/cancelar trabalho obsoleto em vez de manter fila infinita.

Uma futura `ReplacementKey` pode representar “este job substitui outro preview do mesmo alvo”. Ela entra apenas quando existir caso concreto.

### Progress

Progress é estado operacional:

~~~rust
pub struct JobProgress {
    pub completed: u64,
    pub total: Option<u64>,
    pub phase: JobPhase,
}
~~~

Não persistir no PTND.

Não publicar cada iteração interna para a UI. Limitar atualizações quando a frequência gerar custo maior que o benefício visual.

### Resultado e commit

Job nunca altera o Document diretamente.

~~~text
snapshot
↓
job
↓
typed result
↓
revision validation
↓
Command / Transaction
↓
Document owner
~~~

Na v0.1, um job que pretende gerar mutação autoral valida contra a `DocumentRevision` completa usada como origem.

Se a revision divergiu, o resultado é potencialmente obsoleto e não é aplicado silenciosamente.

### Qt

Jobs não recebem `QObject`, `QQuickItem` ou outro estado Qt.

Quando terminam, devolvem dados Petunia. A camada UI/aplicação faz a entrega para Qt no contexto de thread correto.

### Erros

`JobError` deve distinguir pelo menos conceitos como:

- cancelled;
- stale revision;
- invalid input;
- resource unavailable;
- algorithm failure;
- I/O failure quando aplicável.

Cancelamento solicitado pelo usuário é um resultado esperado de controle de fluxo, não necessariamente um erro que exige diálogo.


## Plugins

A arquitetura de plugins segue **sandbox e capabilities por padrão**.

A v0.1 não carrega bibliotecas nativas arbitrárias de terceiros dentro do processo principal.

### Modelo de extensão

Existem três classes conceituais:

~~~text
Built-in extension
→ crate compilada com o Petunia

Sandboxed plugin
→ WebAssembly + Host API

Native external integration
→ processo separado + IPC, futuro
~~~

**Built-in** é código do próprio produto/distribuição e segue as mesmas regras de crates internas.

**Sandboxed plugin** é o formato público inicial para extensões de Engine.

**Native external integration** fica reservado a integrações que realmente precisem de bibliotecas/sistemas não viáveis em WASM; roda fora do processo para limitar impacto de crash/corrupção.

### Por que WebAssembly

**WASM — WebAssembly** fornece bytecode portátil executado dentro de runtime controlado.

O plugin não recebe automaticamente:

- memória do processo;
- filesystem inteiro;
- network;
- process spawning;
- ponteiros Rust;
- Qt;
- `&mut Document`.

Ele recebe somente funções explicitamente oferecidas pelo Host API.

A escolha do runtime Rust concreto pode ser feita no milestone de implementação com benchmark de tamanho, startup e desempenho. O formato/Host API não depende do runtime.

### Plugin Manifest

Pacote de plugin contém manifest versionado:

~~~text
plugin_id
plugin_version
host_api_version
entrypoints
capabilities
permissions requested
metadata
optional resources
~~~

`plugin_id` usa namespace estável, por exemplo domínio reverso ou UUID definido pelo autor.

Não usar nome de exibição localizado como identidade.

### Host API

Host API é capability-oriented.

Exemplos:

~~~text
document.query
command.submit
resource.read
import.register
export.register
effect.evaluate
log.write
job.spawn-scoped
~~~

Um plugin só recebe handles opacos e DTOs estáveis.

Nunca recebe:

~~~rust
&mut Document
&SceneGraph
QObject*
raw Rust trait object
~~~

### Queries

Leitura acontece por snapshot/query.

~~~text
Plugin
↓ query request
Host
↓ validated snapshot DTO
Plugin
~~~

Queries grandes precisam paginação/streaming ou handles de leitura para evitar copiar o documento inteiro para linear memory do WASM.

### Mutação

Mutação sempre volta pela Command API:

~~~text
Plugin intent
↓
Host Command DTO
↓ validation
Engine Command Handler
↓
Transaction
↓
Document
~~~

Assim plugin ganha automaticamente Undo/Redo, atomicidade, revision validation e invariantes de Core.

### Effects e filters

Plugin effect declara contrato equivalente a um effect built-in:

- input/output kind;
- parameters/schema version;
- bounds expansion;
- ROI/input region;
- color-space semantics;
- deterministic requirements;
- cancellation;
- materialization behavior.

Na v0.1, plugin effect CPU trabalha em buffers/tiles fornecidos pelo host.

Plugins não recebem handle GPU nativo. GPU plugin API pública fica fora do escopo inicial porque quebraria portabilidade e sandbox.

### Importer/exporter plugins

Importer recebe bytes/stream autorizado pelo host, não path arbitrário.

Exporter recebe snapshot DTO/Render service e um output stream autorizado.

Isso permite aplicar os mesmos limites de memória, paths e permissões do I/O nativo.

### Tool logic e UI extension

Tool logic pode futuramente usar a Command/Query API.

**Panel/UI extension não é especificada aqui**. Ela será discutida junto com GUI/UX para não congelar um modelo de extensão de Qt sem revisar a experiência completa.

## Segurança de plugins

Permissions são deny-by-default.

Classes:

~~~text
filesystem.read-selected
filesystem.write-selected
network
clipboard
process
ui-extension
~~~

A concessão é feita pela aplicação/usuário conforme política futura.

Plugin sem capability não consegue a operação correspondente.

### Filesystem

O host prefere handles de arquivos/diretórios concedidos pelo usuário, não paths globais.

Mesmo com permissão, normalizar paths e bloquear traversal fora do escopo concedido.

### Network e process

Desabilitados por padrão.

Plugin de filtro/import normal não precisa de network.

Process spawning não existe para plugin WASM v0.1.

### Limits

Runtime aplica limites:

- memória linear;
- tempo/fuel quando suportado;
- tamanho de mensagens;
- número de handles;
- recursion/stack;
- jobs simultâneos;
- output size.

Exceder limite termina a invocação do plugin sem corromper Document.

### Crash/failure

Falha do plugin produz erro tipado e descarta a operação em andamento.

Nenhuma Transaction parcialmente aplicada.

## ABI

**ABI — Application Binary Interface** define como código compilado conversa em nível binário.

A ABI Rust não é contrato público estável. Portanto:

> trait objects Rust, layouts de structs Rust e symbols internos nunca são a ABI de plugins.

Para WASM, o contrato público é Host API versionada + DTOs/handles.

### Native external plugins

Se surgir necessidade de integração nativa de alto desempenho, a direção é **processo separado + IPC**.

**IPC — Inter-Process Communication** significa trocar mensagens entre processos.

~~~text
Petunia
↓ protocol
Plugin Host Process
↓ native library
~~~

Um crash do processo externo pode encerrar a integração sem destruir memória do processo principal.

C ABI in-process não é caminho público inicial.

## Scripting

Scripting e macro usam o mesmo Command/Query host.

A linguagem concreta de scripting não precisa ser decidida para estabilizar o motor: o contrato é independente da linguagem.

Macro recorder grava intenção semântica/Commands, não eventos crus.

## MCP

**MCP — Model Context Protocol** entra como camada de automação externa sobre as mesmas APIs de Query/Command, não como atalho para o domínio.

Direção:

~~~text
MCP Client / Agent
↓
Petunia MCP Adapter
├── read/query tools
├── resource inspection
├── render/export requests
└── command submission
        ↓
     Engine
        ↓
   Transaction
~~~

### Segurança MCP

Servidor MCP:

- desabilitado por padrão;
- iniciado explicitamente pela aplicação/usuário;
- usa transporte local por padrão;
- não expõe filesystem ou network além das capabilities autorizadas;
- não fornece ponteiro/objeto Qt;
- mutações passam por Commands;
- operações destrutivas continuam Commands explícitos;
- pode exigir confirmação/policy na camada de aplicação para ações sensíveis.

A especificação concreta de UX/permissões do MCP será discutida junto com GUI/UX, mas a fronteira de motor fica definida agora.

## Versionamento de extensões

Separar:

~~~text
Plugin package version
Host API version
Plugin data schema version
~~~

Uma atualização do plugin não implica mudança de Host API.

Payload persistente de plugin usa namespace + versão e pode ser preservado opacamente quando plugin está ausente.

Load do documento nunca executa plugin automaticamente apenas porque um payload está presente.

## Invariantes de plugins/automação

1. Plugins públicos de Engine são sandboxed por padrão.
2. WASM + Host API versionada é o formato público inicial.
3. Código nativo arbitrário não roda in-process na v0.1.
4. Plugins leem via Query/Snapshot e mutam via Command.
5. Plugin não recebe tipos Rust internos, Qt ou `&mut Document`.
6. Permissions são deny-by-default.
7. Effect/filter plugins obedecem ROI, color, determinism e cancellation contracts.
8. UI extension fica para a discussão de GUI/UX.
9. Native high-performance extension, se necessária, usa processo separado + IPC.
10. MCP usa a mesma Query/Command boundary.
11. Load de PTND não executa payload de plugin/MCP automaticamente.
12. Host API, package version e data schema version são independentes.

## Verificação inicial das políticas do host (2026-10-10; revisão histórica)

Esta revisão histórica não executava bytecode WASM: testava políticas e contadores do host. A auditoria reabriu o checkpoint; a execução real foi implementada na correção descrita abaixo.

Escopo original: `WasmPluginHost` com validação de magic `\0asm\1\0\0\0`, medição de combustível (fuel metering), limites de memória linear, tabela de handles com detecção de stale handles, e invocação de Host API com gates de permissão deny-by-default. Revisão `d236eb01c9e5fd681213991a0703c2c4b8d8143e` sobre branch `petunia-design-rust`.

| Gate executado | Resultado |
|---|---|
| `cargo test --workspace` | pass: 404 passed / 0 failed (Engine com testes das políticas do host) |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `node website/scripts/verify-progress.cjs` | pass |

Riscos/limites: plugins de interface gráfica (QML/UI) permanecem fora do escopo v0.1.

## Runtime e adapters verificados em 2026-10-10

`WasmPluginHost` executa bytecode com wasmi, portable dispatch, fuel e limites de memória/tabela/stack. Grants do host são independentes das capabilities declaradas pelo módulo. DocumentQuery, CommandSubmit e LogWrite possuem ABI concreta; comandos são staged e publicados como uma transação somente após o guest terminar com sucesso. Trap, módulo inválido e limite excedido não publicam edits. Os testes executam módulos WASM reais, inclusive loop sem término e tentativa de exceder recursos. Isso substitui a validação inicial de magic/version e listas de operações simuladas.

O scheduler limita a fila, oferece lane interativa reservada quando há pelo menos dois workers, fairness na retirada da fila e shutdown com deadline. Cancelamento de um job em execução depende da cooperação do algoritmo. Não há preempção de closures arbitrárias; quantum de fila não é timeslicing de execução. Evidência: `scheduler_spatial_regressions.rs` (13 testes) e testes do módulo `plugins`.

PNG/JPEG são limitados antes da decodificação completa; o serializer SVG preserva segmentos Line/Cubic, handles unilaterais e fechamento curvo. Importador SVG, exportação de documento, demais funções da Host API e guards de aplicação de resultados por sessão/recurso continuam pendentes. Gates em [Verification](#/docs/00-architecture/verification.md).
