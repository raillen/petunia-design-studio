"""Validate PTND v1 JSON documents against the published schemas."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import jsonschema

ROOT = Path(__file__).resolve().parents[1]
SCHEMAS = {
    "manifest": ROOT / "schemas/ptnd/v1/manifest.schema.json",
    "document": ROOT / "schemas/ptnd/v1/document.schema.json",
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("kind", choices=sorted(SCHEMAS))
    parser.add_argument("path", type=Path)
    args = parser.parse_args()

    schema = json.loads(SCHEMAS[args.kind].read_text())
    data = json.loads(args.path.read_text())
    try:
        jsonschema.validate(data, schema)
    except jsonschema.ValidationError as exc:
        print(f"INVALID: {exc.message}", file=sys.stderr)
        return 1
    print("OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
