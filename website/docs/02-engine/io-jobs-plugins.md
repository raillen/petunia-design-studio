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

```rust
pub struct JobSpec {
    pub priority: JobPriority,
    pub cancellable: bool,
    pub document_revision: Revision,
}
```

Jobs precisam progress, cancellation e resultado tipado. `rayon` serve computação paralela, mas scheduler de editor precisa também filas/prioridades.

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
