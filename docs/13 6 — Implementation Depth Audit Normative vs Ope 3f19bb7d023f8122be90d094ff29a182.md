# 13.6 — Implementation Depth Audit: Normative vs Open-by-ADR

# Purpose

Distinguish true documentation gaps from deliberate decisions that must remain open until evidence exists.

# Normative / implementation-grade now

- Python/C++/Qt boundary and dependency direction;
- DocumentSession/revision/snapshot/ChangeSet;
- Actions/Commands/Properties architecture and catalogs;
- geometry value/topology/curve/boolean/hit/snap contracts;
- raster pixel/TileStore/brush/undo/mask contracts;
- RenderScene/compositor/frame graph/resource lifetime;
- text storage/shaping/layout/caret/font contracts;
- color semantic/ICC/CMYK/spot/prepress contracts;
- PTND manifest/core object/resource schemas at normative prose level;
- plugin host/RPC/UI schema;
- MCP v1 method/query/permission contracts;
- individual Design/Photo tool contracts;
- major panel/dialog contracts;
- history, presets/libraries, effects, interchange matrices and performance SLO framework;
- first 30 executable Goals.

# Deliberately open pending ADR/benchmark

1. primary renderer backend (Skia/Dawn/custom/Qt-related candidate);
2. native dependency manager (vcpkg/Conan/other);
3. exact geometry boolean library;
4. exact PDF parser/writer;
5. exact PTILE binary byte layout/compression after benchmark;
6. exact canonical raster tile size if benchmark rejects 256;
7. final compositor intermediate float16 vs float32 policy by operation;
8. optional ICU dependency scope;
9. optional OCIO role;
10. WASM plugin tier milestone/runtime;
11. minimum supported OS/driver versions after renderer spike;
12. advanced RAW/ML/photo roadmap.

# Remaining work is schema/code generation, not conceptual gap

Several pages describe exact fields and contracts but the repository machine-readable JSON Schema/IDL/C++ headers do not exist until implementation Goals run. This is expected target-spec state, not missing documentation.

# Rule

Any implementer discovering an unstated behavior must add it here as Gap or open ADR before inventing local semantics.