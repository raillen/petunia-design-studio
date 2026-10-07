# Petunia Design Studio

Desktop creative editor (vector + raster + layout) — Python 3.14 / PySide6 / Qt **QML (Qt Quick)**, C++23 core, nanobind bindings.

## Bootstrap

```bash
uv venv --python 3.14 .venv
uv pip install --python .venv/bin/python nanobind pytest ruff pyright PySide6 jsonschema pyclipper
cmake --preset debug -Dnanobind_DIR=$(.venv/bin/python -m nanobind --cmake_dir)
cmake --build --preset debug
```

## Everyday commands

```bash
# C++ tests
./build/debug/cpp/petunia_core/tests/core_smoke
ctest --preset debug

# Python tests
.venv/bin/python -m pytest

# Lint / typecheck
.venv/bin/ruff check python tests tooling
.venv/bin/pyright

# App smoke launch (offscreen)
PYTHONPATH=python QT_QPA_PLATFORM=offscreen .venv/bin/python -m petunia_app --smoke

# PTND schema validation
.venv/bin/python tooling/validate_ptnd.py document examples/minimal.document.json
```

## Layout

- `cpp/petunia_core` — C++23 core (ids, math, error, document, command, jobs)
- `bindings/python` — nanobind module `petunia_native`
- `python/petunia_app` — PySide6 application shell
- `python/petunia_app/qml/main.qml` — Qt Quick studio shell (toolbar, canvas, inspector, layers)
- `python/petunia_app/bridge.py` — QObject bridge between QML and document model (G014/G016/G017)
- `python/petunia_app/model.py` — document model, commands, history, snapping, groups (G023–G025 baseline)
- `python/petunia_app/paths.py` — canonical VectorPath/Contour/Node model, cubic math, topology ops (G029)
- `python/petunia_app/boolean.py` — boolean engine, compound paths, Shape Builder region engine (G031)
- `python/stubs/pyclipper.pyi` — type stubs for the pyclipper backend
- `python/petunia_app/exporters.py` — SVG export baseline (G027)
- `schemas/ptnd/v1` — PTND v1 JSON schemas
- `docs/` — canonical design notebook (779 docs)
- `examples/` — minimal valid PTND JSON
