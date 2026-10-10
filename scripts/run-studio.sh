#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
studio_root="$PWD"
if ! command -v cargo >/dev/null 2>&1 && [[ -x "$studio_root/work/cargo/bin/cargo" ]]; then
    export PATH="$studio_root/work/cargo/bin:$PATH"
    export CARGO_HOME="$studio_root/work/cargo"
    export RUSTUP_HOME="$studio_root/work/rustup"
fi
if [[ -z "${QMAKE:-}" ]]; then
    if command -v qmake6 >/dev/null 2>&1; then
        export QMAKE
        QMAKE=$(command -v qmake6)
    elif [[ -x "$studio_root/work/qt/usr/lib/qt6/bin/qmake" ]]; then
        export QMAKE="$studio_root/work/qt/usr/lib/qt6/bin/qmake"
    fi
fi
if [[ "${QMAKE:-}" == "$studio_root/work/qt/usr/lib/qt6/bin/qmake" ]]; then
    export LD_LIBRARY_PATH="$studio_root/work/qt/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
    export QT_PLUGIN_PATH="$studio_root/work/qt/usr/lib/x86_64-linux-gnu/qt6/plugins${QT_PLUGIN_PATH:+:$QT_PLUGIN_PATH}"
fi
exec cargo run -p petunia-desktop --features native --bin petunia-studio -- "$@"
