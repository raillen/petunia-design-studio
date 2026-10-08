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

Plugin API não expõe `&mut Document`.

Capacidades:
- Command API
- query snapshot
- importer/exporter
- effect/filter
- tool logic
- panel/UI extension separada.

## Segurança de plugins

Permissões/capabilities explícitas para filesystem, network, process e UI. Plugin crash não deve corromper documento.

## ABI

**ABI — Application Binary Interface** define como código compilado conversa em nível binário: layout de dados, convenção de chamadas, símbolos e outras regras.

A ABI Rust não é estável entre versões do compilador, então trait objects Rust não devem ser contrato binário público de plugins.

Alternativas:

- **C ABI** — interface binária simples e amplamente estável;
- **IPC — Inter-Process Communication** — plugin roda em outro processo e conversa por mensagens;
- **WASM — WebAssembly** — formato sandboxável com host API controlada.

A escolha só deve ser fechada quando o sistema de plugins entrar em implementação.

## Scripting

Script chama Commands e queries do mesmo host de plugins. Macro recorder grava intenção/Commands, não eventos de mouse crus.
