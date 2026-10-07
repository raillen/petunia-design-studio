# I/O, Jobs e Plugins

Esses subsistemas ficam no Engine porque coordenam algoritmos e recursos externos sem pertencer à UI.

## Import

Pipeline:

```text
bytes
→ sniff/format detection
→ parser seguro
→ formato intermediário
→ normalize
→ Core DTO
→ validate invariants
→ Document
```

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

Rust ABI não é estável. Se plugins nativos externos forem objetivo, definir C ABI/IPC/WASM ou versão de SDK explícita; não exportar trait objects Rust como contrato binário público.

## Scripting

Script chama Commands e queries do mesmo host de plugins. Macro recorder grava intenção/Commands, não eventos de mouse crus.
