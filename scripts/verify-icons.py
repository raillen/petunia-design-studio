"""Verify the provenance and integrity of embedded desktop icon assets."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent.parent / "crates/petunia-desktop/assets/icons"
manifest = json.loads((root / "manifest.json").read_text())
assets = manifest["assets"]
outline = {(item["key"], item["family"]) for item in assets if item["style"] == "outline"}
for item in assets:
    if "file" in item:
        data = (root / item["file"]).read_bytes()
        assert hashlib.sha256(data).hexdigest() == item["sha256"], item["file"]
        assert b"<svg" in data
        assert manifest["revision"][item["family"]] in item["upstream"]
    else:
        assert item["style"] == "fill" and item["fallback"] == "outline"
        assert (item["key"], item["family"]) in outline
for family in manifest["revision"]:
    assert "MIT" in (root / f"{family}-LICENSE").read_text()
print(f"Icon provenance verified: {len(assets)} catalog entries, both MIT notices preserved.")
