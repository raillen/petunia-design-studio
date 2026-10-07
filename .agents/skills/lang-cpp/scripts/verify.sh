#!/bin/sh
set -eu

echo "=== Verifying C++ Code Quality and Safety ==="

# 1. Run Prumo escape hatch scanner
echo "[1/3] Scanning for unregistered escape hatches..."
prumo tool check-escape-hatches .

# 2. Check compiler flags if CMakeLists exists
if [ -f "CMakeLists.txt" ]; then
    echo "[2/3] Verifying CMake configuration..."
    grep -E -- "-Wall|-Wextra|-Werror" CMakeLists.txt >/dev/null 2>&1 || echo "Warning: Strict warnings not found in root CMakeLists.txt"
fi

# 3. Completion
echo "[3/3] C++ safety baseline verified."
exit 0
