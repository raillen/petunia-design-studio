# 01 — Architecture, Core/UI Boundary & Canonical Document

# Regra arquitetural

**O core C++ é a autoridade do documento. Python é a camada de produto. Qt é uma implementação de GUI.** Nenhum objeto Qt, QObject, QWidget, signal ou Python object pode ser armazenado dentro do modelo canônico.

# Camadas

```
apps/petunia_studio          Python/PySide6
python/petunia              Application services, tools, panels
bindings/petunia_native     nanobind boundary
cpp/petunia_core            document + commands + IDs
cpp/petunia_geometry        paths, boolean, stroke, snapping
cpp/petunia_raster          tiles, brushes, masks, selections
cpp/petunia_text            shaping/layout
cpp/petunia_color           profiles/conversion/proofing
cpp/petunia_render          render scene/compositor
cpp/petunia_io              .PTND + import/export
cpp/petunia_jobs            scheduler/cancellation
cpp/petunia_plugin_protocol runtime-neutral plugin contracts
```

# Canonical document

DocumentStore contém IDs estáveis e tipos semanticamente ricos: Surface, Layer, Group, VectorObject, RasterLayer, TextObject, Mask, Adjustment, EffectChain, Symbol, Style, Resource, DataBinding e ExtensionPayload.

# Estados separados

- **Canonical:** serializável e undoable.
- **Derived:** bounds, tessellation, thumbnails, text layout cache, evaluation graph.
- **Selection:** local à view/session.
- **Viewport:** zoom, pan, overlays.
- **UI state:** docking, panel expansion, active tabs.
- **Render cache:** GPU/CPU disposable.

# Mutation path

UI, shortcut, plugin e MCP convergem em:

```
ActionId -> validation/context -> Command -> Transaction -> DocumentMutator
-> ChangeSet -> history -> invalidation -> UI/render notifications
```

Nenhum widget edita containers do documento diretamente.

# Python/C++ ownership

- C++ possui o lifetime do documento e recursos nativos.
- Python recebe handles/IDs e snapshots imutáveis.
- Objetos expostos por nanobind não mantêm ponteiros frágeis para elementos internos.
- Long-running calls liberam o GIL.
- Callbacks para Python são enviados na main thread por filas/event bridges.
- Exceptions C++ são convertidas para error types estáveis; não atravessam ABI cruas.

# Ports and adapters

Filesystem, clipboard, native dialogs, windowing, font enumeration, tablet, color display profiles e URLs são ports. Qt implementa ports no desktop; testes usam fakes/headless adapters.

# Design patterns permitidos

Command, Transaction/Unit of Work, Repository apenas quando há storage boundary real, Strategy para backends, Adapter para formatos/plataformas, Visitor somente quando simplifica árvores estáveis, Observer/event bus tipado, State Machine para tools, Registry para capabilities, Facade para bindings.

# Anti-patterns proibidos

God Manager, Service Locator global, QObject como domain model, signal spaghetti, singleton mutável, deep inheritance trees, Python dictionaries sem schema atravessando o core, raw pointers como identidade, catch-all exceptions, lógica de negócio em widgets.

[01.1 — Clean Architecture Layers, Dependency Rule & Composition Root](01%201%20%E2%80%94%20Clean%20Architecture%20Layers,%20Dependency%20Rule%20%203f19bb7d023f8177aba6f916890476d6.md)

[01.2 — ApplicationCore, DocumentSession, ViewSession & Service Ownership](01%202%20%E2%80%94%20ApplicationCore,%20DocumentSession,%20ViewSessi%203f19bb7d023f818a9edafbd9d66e09bb.md)

[01.3 — Snapshot, Query & Presentation Projection Contracts](01%203%20%E2%80%94%20Snapshot,%20Query%20&%20Presentation%20Projection%20C%203f19bb7d023f814bb73af41078470d60.md)

[01.4 — Event Model, Change Propagation, Typed Notifications & Feedback-Loop Prevention](01%204%20%E2%80%94%20Event%20Model,%20Change%20Propagation,%20Typed%20Noti%203f19bb7d023f8166822ccc3cc3805ea2.md)

[01.5 — Thread Ownership, Main-Thread Rules, Immutable Jobs & Locking Strategy](01%205%20%E2%80%94%20Thread%20Ownership,%20Main-Thread%20Rules,%20Immuta%203f19bb7d023f8191957ae326e3b51935.md)

[01.1 — Layered Architecture, Dependency Direction & Composition Root](01%201%20%E2%80%94%20Layered%20Architecture,%20Dependency%20Direction%20%203f19bb7d023f81ceb54ae5c237ebefa1.md)

[01.2 — Canonical State vs Derived/View/UI State Ownership Matrix](01%202%20%E2%80%94%20Canonical%20State%20vs%20Derived%20View%20UI%20State%20Ow%203f19bb7d023f81bfb1afc8496ed1da4f.md)

[01.3 — DocumentSession, Snapshot, Revision & Query Consistency Model](01%203%20%E2%80%94%20DocumentSession,%20Snapshot,%20Revision%20&%20Query%203f19bb7d023f81389046c02914eb759d.md)

[01.4 — Event, ChangeSet, Invalidation & Notification Contract](01%204%20%E2%80%94%20Event,%20ChangeSet,%20Invalidation%20&%20Notificati%203f19bb7d023f811988deda931fb0c81b.md)

[01.5 — Capability Registry, Feature Availability & Missing-Capability Degradation](01%205%20%E2%80%94%20Capability%20Registry,%20Feature%20Availability%20&%203f19bb7d023f8131abcdd94a3d65dcdc.md)

[01.6 — Error Taxonomy, Validation Boundaries, Fault Containment & User Recovery](01%206%20%E2%80%94%20Error%20Taxonomy,%20Validation%20Boundaries,%20Faul%203f19bb7d023f81d4adffc96f30fa6d90.md)