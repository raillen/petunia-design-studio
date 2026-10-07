# 09.22 — CMake Targets, Python Package Topology, Dependency Direction & Feature Boundaries

# C++ targets

petunia_core has no Qt/Python.

petunia_geometry depends core value contracts only.

petunia_raster depends core/color abstractions as approved.

petunia_text depends core + shaping adapters.

petunia_color owns color contracts/adapters.

petunia_render consumes evaluated scene.

petunia_io consumes core services/schemas.

petunia_jobs is low-level scheduling infrastructure.

petunia_native binds public application facades.

# Python packages

petunia_app.domain_facade consumes petunia_native.

petunia_app.actions/tools/panels depend typed facade + presentation services.

petunia_app.qt contains Qt adapters/widgets/models.

petunia_app.plugins and mcp use semantic application services.

Composition root alone wires concrete adapters.

# Forbidden edges

core -> Qt/Python; geometry -> GUI; io parser -> QWidget; panel -> DocumentStore mutable internals; plugin -> native pointer.

# Build checks

CMake target link graph and Python import-linter/dependency checker enforce edges. Include-what-you-use optional evidence aid.

# Feature flags

Compile-time flags only for platform/backend/optional heavyweight capability; product feature availability is registry/runtime capability, not preprocessor forest.