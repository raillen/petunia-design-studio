#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build -p petunia-desktop --features native
python3 scripts/verify-icons.py
mkdir -p work
gui_results=$(mktemp -d "$PWD/work/gui-validation-XXXXXX")
for gui_scale in 1 2; do
    gui_run="$gui_results/dpi-$gui_scale"
    mkdir -p "$gui_run"
    QT_QPA_PLATFORM=offscreen QT_QUICK_BACKEND=software QT_SCALE_FACTOR="$gui_scale" \
        PETUNIA_CONFIG_DIR="$gui_run/config" \
        timeout 45 target/debug/petunia-studio --smoke-test "$gui_run" >"$gui_run/run.log" 2>&1
    cat "$gui_run/run.log"
    rg -q PETUNIA_NATIVE_SMOKE_PASS "$gui_run/run.log"
    if rg -q 'PETUNIA_NATIVE_SMOKE_FAIL|Unable to assign|ReferenceError|TypeError|Binding loop' "$gui_run/run.log"; then
        exit 1
    fi
done
echo "Native GUI validation artifacts: $gui_results"
