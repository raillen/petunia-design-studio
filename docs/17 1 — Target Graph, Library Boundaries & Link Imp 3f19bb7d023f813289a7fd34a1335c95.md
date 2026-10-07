# 17.1 — Target Graph, Library Boundaries & Link/Import Rules

# C++ target graph

```
petunia_core
 ├─ petunia_geometry
 ├─ petunia_color
 └─ contracts consumed by text/raster/io/render

petunia_jobs
petunia_text -> core + color + shaping adapters
petunia_raster -> core + color + jobs
petunia_render -> core + geometry + raster + text + color
petunia_io -> core + geometry/raster/text/color contracts + jobs
petunia_application -> core + services + registries + jobs
petunia_native -> application + render + io
```

# Link rule

Native leaf engines may depend on core value/contracts but not on Python/Qt. petunia_native is the only ordinary product target that knows Python binding APIs.

# Python import graph

```
petunia_app.application -> petunia_native
petunia_app.actions/tools -> application contracts
petunia_app.models -> application snapshots
petunia_app.qt -> actions/models/design_system
petunia_app.panels/dialogs -> qt + typed application services
petunia_app.plugins/mcp -> application semantic services
apps/petunia_studio -> composition/bootstrap only
```

# Forbidden imports

panels -> cpp internals;

qt widgets -> direct mutable native containers;

core -> render backend concrete;

core -> filesystem/UI;

plugins -> petunia_native private API;

mcp -> QWidget.

# Enforcement

CMake dependency graph checks + Python import-linter rules + architecture-quality review.

# Public headers

Each C++ target has include/petunia/<module>/ public surface; internal headers under src/internal or private include paths.

# Friend/internal

Avoid broad friend relationships. If modules need privileged mutation, design an application/domain service interface instead.