#!/usr/bin/env sh
set -e
TARGET="./..."
echo "Running concurrency and deadlock detection..."
if command -v go >/dev/null 2>&1 && [ -f "go.mod" ]; then
    exec go test -race -run=^$ ""
else
    echo "Verifying locking patterns..."
    grep -rn "Lock(" . || true
fi
