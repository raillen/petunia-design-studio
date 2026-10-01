#!/usr/bin/env bash
set -euo pipefail
audit_repo="$(cd "$(dirname "$0")/../../../.." && pwd)"
audit_dir="$(mktemp -d "${TMPDIR:-/tmp}/petunia-audit.XXXXXX")"
mkdir -p "$audit_dir/src"
cp "$audit_repo/docs/public/audit/2026-09-30/probes.rs" "$audit_dir/src/main.rs"
python3 - "$audit_repo" "$audit_dir" <<'PY'
import json, pathlib, sys
root=pathlib.Path(sys.argv[1]); target=pathlib.Path(sys.argv[2])
names=['foundation','geometry','document','application','color','raster','render','io','shell']
lines=['[package]','name = "petunia-audit-probes"','version = "0.1.0"','edition = "2021"','[dependencies]']
for name in names:
    crate='petunia_design_'+name
    lines.append(crate+' = { path = '+json.dumps(str(root/'crates'/crate))+' }')
(target/'Cargo.toml').write_text('\n'.join(lines)+'\n')
(target/'Cargo.lock').write_bytes((root/'Cargo.lock').read_bytes())
PY
export PETUNIA_AUDIT_PDF="$audit_dir/audit-probe.pdf"
cargo run --manifest-path "$audit_dir/Cargo.toml" --offline "$@"
if command -v pdfinfo >/dev/null; then pdfinfo "$PETUNIA_AUDIT_PDF"; fi
printf 'Evidence workspace: %s\n' "$audit_dir"
