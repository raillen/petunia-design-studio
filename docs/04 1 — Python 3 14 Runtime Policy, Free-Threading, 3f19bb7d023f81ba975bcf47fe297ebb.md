# 04.1 — Python 3.14 Runtime Policy, Free-Threading, Packaging & Embedded Distribution

# Baseline

Petunia targets **CPython 3.14.8 or newer compatible 3.14.x** for the initial production branch. Python 3.14 is the latest stable feature series as of 2026-10-06.

# Distribution

Petunia ships its own supported Python runtime with the application. It must not depend on whatever Python happens to be installed on the user's system.

# Standard vs free-threaded build

The default application build should initially use the regular GIL-enabled CPython build because:

- PySide6/Qt ecosystem compatibility is the first requirement;
- most CPU-heavy work is native C++ and already releases the GIL;
- plugin/process isolation reduces the need for in-process Python CPU parallelism.

A free-threaded Python experiment may exist behind a build profile only after PySide6, nanobind and dependency compatibility is demonstrated.

# Python responsibility

Python handles:

- application orchestration;
- Qt GUI;
- ToolControllers/state machines;
- panel and dialog presentation;
- workspace/settings;
- trusted built-in extensions;
- MCP/plugin orchestration;
- test tooling.

Python does not own:

- canonical document mutation internals;
- raster loops;
- geometry booleans;
- text shaping/layout hot paths;
- color conversion loops;
- GPU resource submission;
- large serialization transforms.

# Startup

Bootstrap sequence:

1. configure crash/logging;
2. load minimal safe settings;
3. import petunia_native;
4. initialize native ApplicationCore;
5. register built-in contributions;
6. create QApplication;
7. initialize Qt adapters;
8. restore workspace/session;
9. open requested/recent document.

# Packaging

Use isolated Python environment generated from locked dependencies. Wheels for petunia_native are produced per OS/architecture/Python ABI as part of application build; end-user pip installation is not the primary deployment model.

# Python stdlib safety

Do not use pickle for documents/plugins/RPC. Archive extraction always uses explicit safe path checks. subprocess/process execution is isolated behind a broker/service where product behavior needs it.

# Version support

The app pins one production Python minor initially. Supporting multiple Python minors is a plugin-SDK concern only if third-party host distribution later requires it.