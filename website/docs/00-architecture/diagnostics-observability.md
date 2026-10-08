# Diagnósticos e observabilidade

O Petunia precisa explicar falhas sem misturar **erro de domínio**, **diagnóstico técnico** e **mensagem apresentada ao usuário**.

> Erro tipado decide controle de fluxo. Diagnóstico explica contexto. UI decide apresentação.

## Três camadas

~~~text
Typed Error
↓
Structured Diagnostic
↓
Presentation Message
~~~

### Typed Error

Pertence ao domínio/Engine que detectou a falha.

Exemplos:

~~~text
GeometryError::ComplexityLimitExceeded
TransactionError::RevisionConflict
ResourceError::Unresolved
RenderError::AllocationFailed
~~~

É usado por código para decidir comportamento.

### Structured Diagnostic

Adiciona contexto técnico sem mudar a semântica do erro.

Direção:

~~~rust
pub struct Diagnostic {
    pub id: DiagnosticId,
    pub severity: Severity,
    pub domain: DiagnosticDomain,
    pub code: DiagnosticCode,
    pub operation: Option<OperationId>,
    pub document: Option<DocumentId>,
    pub revision: Option<DocumentRevision>,
    pub object: Option<ObjectId>,
    pub job: Option<JobId>,
    pub message_key: DiagnosticMessageKey,
    pub technical_context: DiagnosticFields,
}
~~~

A estrutura concreta pode ser menor, mas IDs/contexto precisam permanecer tipados.

### Presentation

A camada UI traduz `message_key`, decide modal/toast/panel e quais detalhes técnicos mostrar.

Essa parte será discutida junto com GUI/UX.

Core/Engine nunca armazenam string localizada como contrato de erro.

## Severity

Direção:

~~~text
Info
Warning
Error
Fatal
~~~

**Warning** significa que a operação pode continuar com perda/degradação explicitamente conhecida.

**Error** significa que a operação solicitada falhou, mas a aplicação continua íntegra.

**Fatal** significa que aquele processo/contexto não consegue continuar com segurança.

Não promover warning a error apenas para simplificar controle de fluxo.

## DiagnosticCode

Código estável permite:

- testes;
- busca em logs;
- documentação;
- suporte;
- automação.

Exemplo conceitual:

~~~text
geometry.boolean.complexity_limit
io.save.external_conflict
render.effect.unsupported_backend
plugin.permission.denied
~~~

Não usar texto humano como identificador.

## OperationId

Uma ação de alto nível recebe `OperationId` runtime.

~~~text
user invokes Export
↓ OperationId X
├── snapshot
├── color transform
├── render job
└── file write
~~~

Todos os diagnósticos relacionados podem carregar X.

Isso permite reconstruir causalidade sem transformar o ID em estado autoral.

## JobId e Revision

Jobs já possuem JobId e source revision.

Logs/diagnósticos de worker carregam ambos quando disponíveis:

~~~text
job = J42
document = D7
source_revision = R18
phase = FittingCurves
~~~

Quando resultado stale é descartado, o diagnóstico consegue mostrar por quê.

## Structured logging

Logs técnicos usam campos estruturados, não somente strings concatenadas.

Exemplo conceitual:

~~~text
level=error
domain=io
code=io.save.write_failed
document=D7
revision=R18
operation=O91
path_kind=user_selected
os_error=...
~~~

Campos permitem filtrar e testar.

A escolha concreta de crate de logging/tracing não atravessa a API de Core.

## Conteúdo sensível

Por padrão, logs não incluem:

- texto integral do documento;
- bytes de imagens;
- payload de clipboard;
- plugin payload desconhecido;
- paths completos quando não necessários;
- tokens/credentials;
- dados de rede;
- conteúdo MCP.

Preferir:

~~~text
ResourceId
ContentHash
ObjectId
format
dimensions
error code
~~~

quando suficientes para diagnóstico.

## Paths

Quando path for necessário localmente para suporte, tratá-lo como dado potencialmente sensível.

Logs persistentes devem preferir redaction ou representação controlada.

Não enviar path para telemetry sem consentimento explícito.

## Telemetry

A arquitetura **não depende de telemetry remota** para funcionar.

Telemetry/analytics, se algum dia existir:

- é opt-in ou segue política explícita do produto;
- não carrega documento/arte por padrão;
- não é necessária para diagnóstico local;
- possui contrato de privacy separado.

A v0.1 pode funcionar apenas com logging local estruturado.

## Tracing

**Tracing** mede uma operação composta por etapas.

Exemplo:

~~~text
render frame
├── compile snapshot
├── tessellate
├── effect tiles
├── composite
└── output transform
~~~

Spans permitem medir duração e relação parent/child.

Hot paths não devem pagar custo alto de tracing detalhado em builds normais. Instrumentação fina pode ser feature/debug/profiling configuration.

## Metrics

Metrics runtime podem observar:

- latency;
- throughput;
- cache hit rate;
- allocation/peak memory;
- queue depth;
- cancelled/stale jobs;
- time-to-first-frame.

São Derived/operational State.

Nunca entram no PTND.

## Panic policy

Panic não é mecanismo normal de erro de input.

Regras:

- input inválido retorna erro tipado;
- plugin/resource/parser failure retorna erro;
- allocation/complexity guards retornam erro quando recuperáveis;
- `panic!` fica para invariantes internas impossíveis/bugs de programação.

Fronteiras de processo/app podem capturar panic apenas para:

- registrar diagnóstico;
- tentar preservar recovery;
- encerrar de forma controlada quando seguro.

Não usar `catch_unwind` para fingir que estado possivelmente corrompido pode continuar normalmente.

## Worker panic

Worker que entra em panic:

~~~text
worker panic
↓ mark job failed
↓ diagnostic
↓ no Transaction commit
~~~

Se o runtime/pool puder continuar seguro, outros jobs permanecem. Se a falha indicar corrupção compartilhada, elevar para fatal.

## Plugin diagnostics

Plugin não escreve diretamente nos logs internos arbitrariamente.

Host API fornece logging limitado:

~~~text
log.write(level, message, fields)
~~~

Host anexa:

- PluginId;
- invocation/JobId;
- limits;
- permission context.

Mensagens de plugin são consideradas input não confiável e passam por limites de tamanho.

## Render diagnostics

Render degradation precisa ser explícita.

Exemplo:

~~~text
UnsupportedEffect
↓ fallback CPU
↓ warning diagnostic
~~~

ou:

~~~text
AllocationFailed
↓ frame não pode ser produzido
↓ error
~~~

Não ocultar fallback que muda fidelidade.

## Diagnostics e tests

Testes podem afirmar códigos/contexto estáveis:

~~~text
invalid PTND path traversal
→ io.container.path_traversal

stale background result
→ jobs.stale_revision
~~~

Não testar tradução/localização dentro de Core/Engine.

## Performance diagnostics

Benchmarks e profiling podem habilitar campos adicionais:

- phase timings;
- object counts;
- tile counts;
- cache misses;
- recursion depth;
- topology complexity.

Esses dados ajudam otimização sem virar API persistente.

## Retenção

Logs locais precisam de política de tamanho/rotação.

Não deixar arquivo de log crescer indefinidamente.

Valores absolutos dependem da aplicação/plataforma.

Recovery Store e logs são storages diferentes; apagar um não apaga o outro.

## Crash report

Crash report automático não é requisito do motor.

Se implementado futuramente, deve:

- ser sanitizado;
- evitar conteúdo autoral;
- depender de política/consentimento;
- anexar versões, platform info e diagnostics relevantes;
- nunca ser condição para recovery local.

## Invariantes

1. Typed Error, Diagnostic e mensagem de UI são camadas diferentes.
2. Core/Engine não retornam strings localizadas como contrato.
3. DiagnosticCode é estável e pesquisável.
4. OperationId/JobId/Revision conectam eventos runtime sem virar Document State.
5. Logging é estruturado e local por padrão.
6. Documento/conteúdo sensível não entra em logs por conveniência.
7. Telemetry remota não é dependência arquitetural.
8. Panic não trata input inválido esperado.
9. Worker/plugin failure nunca aplica Transaction parcial.
10. Metrics/tracing são Derived/operational State.
11. Render fallback que altera comportamento produz diagnóstico.
12. Retenção de logs é limitada e independente de recovery.
