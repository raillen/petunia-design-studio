# 04 — Canonical Python/C++ Stack & Engine Boundaries

# Baseline

- Python 3.14.x;
- PySide6 / Qt 6.12 LTS;
- C++23;
- nanobind;
- CMake + Ninja;
- uv + pyproject;
- Pyright strict;
- Ruff;
- pytest;
- clang-format/clang-tidy;
- ASan/UBSan/TSan;
- GoogleTest/Catch2 conforme módulo;
- Criterion/Google Benchmark ou harness próprio para C++ microbenchmarks.

# Bindings

nanobind expõe facades grossas: DocumentSession, ActionRegistry, QuerySnapshot, JobHandle, ExportRequest, ToolPreviewSession. Evitar bindings de cada container interno.

# GIL

Funções C++ com custo relevante usam GIL release. C++ não chama Python durante sections sem GIL. Completion é entregue por future/job polling/event message na UI thread.

# Concurrency

C++ usa std::jthread/stop_token e scheduler próprio com prioridades: interactive, render, background, IO. Python coordena jobs, não cria threads para pixel/render loops. Qt main thread é dona da GUI.

# Core libraries

HarfBuzz/FreeType para text; LittleCMS como baseline ICC; OpenColorIO opcional para workflows avançados; libvips e codecs nativos para IO; zstd para chunks próprios quando necessário; miniz/libzip para container .PTND conforme conformance profile.

# Renderer

IRenderBackend é C++ puro. O V1 deve executar um spike comparativo entre Skia GPU, Dawn/WebGPU e backend próprio sobre APIs nativas. A escolha final exige benchmark/driver matrix; o core/document não depende dela.

# Qt boundary

Qt pode fornecer window surface, input, clipboard, file dialogs, font discovery e accessibility bridge. Renderer não usa QPainter como pipeline principal do documento.

# Build products

- petunia-core library;
- petunia-native Python extension;
- petunia-studio Python app;
- petunia-cli headless;
- petunia-plugin-host helper;
- petunia-mcp server;
- tests/benchmarks/fixtures tooling.

# Dependency direction

app -> python services -> bindings -> C++ application/domain -> infrastructure adapters.

Nunca C++ core -> Python/PySide6.

[04.1 — Python 3.14 Runtime Policy, Free-Threading, Packaging & Embedded Distribution](04%201%20%E2%80%94%20Python%203%2014%20Runtime%20Policy,%20Free-Threading,%203f19bb7d023f81ba975bcf47fe297ebb.md)

[04.2 — PySide6 / Qt 6.12 LTS Versioning, Licensing, Modules & Deployment Profile](04%202%20%E2%80%94%20PySide6%20Qt%206%2012%20LTS%20Versioning,%20Licensing,%20%203f19bb7d023f8186b15dd5476d402296.md)

[04.3 — nanobind Contract, Native Module Surface, Type Stubs & Call-Cost Budget](04%203%20%E2%80%94%20nanobind%20Contract,%20Native%20Module%20Surface,%20T%203f19bb7d023f8189b6fcee81538f917e.md)

[04.4 — C++23 Toolchain, Compilers, Standard Library, Sanitizers & Build Profiles](04%204%20%E2%80%94%20C++23%20Toolchain,%20Compilers,%20Standard%20Librar%203f19bb7d023f8125bc19dc98b761d523.md)

[04.5 — Native Third-Party Libraries: Geometry, Raster, Text, Color, Compression & IO Evaluation](04%205%20%E2%80%94%20Native%20Third-Party%20Libraries%20Geometry,%20Rast%203f19bb7d023f811f9c42cbb56f156967.md)

[04.6 — Native Renderer Candidate Matrix: Skia GPU, Dawn/WebGPU, Qt RHI & Custom Backends](04%206%20%E2%80%94%20Native%20Renderer%20Candidate%20Matrix%20Skia%20GPU,%20%203f19bb7d023f81078b50fbda235d86c7.md)

[04.7 — Text Stack: HarfBuzz, FreeType, Font Discovery, ICU/Unicode Services & Hyphenation](04%207%20%E2%80%94%20Text%20Stack%20HarfBuzz,%20FreeType,%20Font%20Discove%203f19bb7d023f811ea072ff3de5939eb6.md)

[04.8 — Color Stack: LittleCMS, OCIO, ICC Resources, Display Profiles & GPU LUT Strategy](04%208%20%E2%80%94%20Color%20Stack%20LittleCMS,%20OCIO,%20ICC%20Resources,%203f19bb7d023f81bda372d65daf2d0b35.md)

[04.9 — Image IO, libvips, Codecs, Streaming Decode, Metadata & Security Boundaries](04%209%20%E2%80%94%20Image%20IO,%20libvips,%20Codecs,%20Streaming%20Decode%203f19bb7d023f81a4a3d5f1473edb589a.md)

[04.10 — Build/Dependency Tooling: CMake, Ninja, uv, pyproject, vcpkg/Conan Evaluation & Reproducibility](04%2010%20%E2%80%94%20Build%20Dependency%20Tooling%20CMake,%20Ninja,%20uv,%203f19bb7d023f810e87dff3742502b31e.md)

[04.11 — Process Topology: Main App, Plugin Hosts, MCP Server, Helpers & Crash Containment](04%2011%20%E2%80%94%20Process%20Topology%20Main%20App,%20Plugin%20Hosts,%20M%203f19bb7d023f81a79cc7f5a37c723ec4.md)

[04.12 — Stack Decision Ledger, Compatibility Windows & Upgrade Policy](04%2012%20%E2%80%94%20Stack%20Decision%20Ledger,%20Compatibility%20Windo%203f19bb7d023f813cbf21cea4e9cdd001.md)