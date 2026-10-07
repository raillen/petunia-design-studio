#!/bin/sh
set -eu

echo "=== Verifying C Code Quality and Safety ==="

# 1. Run Prumo escape hatch scanner
echo "[1/2] Scanning for unregistered escape hatches..."
prumo tool check-escape-hatches .

# 2. Completion
echo "[2/2] C safety baseline verified."
exit 0
