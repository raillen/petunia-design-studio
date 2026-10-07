from __future__ import annotations

import hashlib
import io
import json
import zipfile
from pathlib import Path, PurePosixPath
from typing import Any, cast

MIMETYPE = b"application/x-petunia-document"
MAX_ENTRIES = 4096
MAX_TOTAL_BYTES = 512 * 1024 * 1024
MAX_ENTRY_BYTES = 256 * 1024 * 1024
MAX_JSON_DEPTH = 64


class PtndError(Exception):
    pass


def _check_depth(value: Any, depth: int = 0) -> None:
    if depth > MAX_JSON_DEPTH:
        raise PtndError("json nesting too deep")
    if isinstance(value, dict):
        values = cast(dict[str, Any], value).values()
        for v in values:
            _check_depth(v, depth + 1)
    elif isinstance(value, list):
        items = cast(list[Any], value)
        for v in items:
            _check_depth(v, depth + 1)


def _safe_entry(name: str) -> str:
    p = PurePosixPath(name)
    if p.is_absolute() or ".." in p.parts:
        raise PtndError(f"unsafe entry path: {name}")
    return name


def _canonical_json(obj: Any) -> bytes:
    text = json.dumps(obj, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
    return (text + "\n").encode("utf-8")


def write_package(
    path: Path,
    manifest: dict[str, Any],
    document: dict[str, Any],
    resource_files: dict[str, bytes] | None = None,
) -> None:
    for name, obj in (("manifest", manifest), ("document", document)):
        _check_depth(obj)
        if not obj:
            raise PtndError(f"{name} must not be empty")

    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w", allowZip64=True) as zf:
        zi = zipfile.ZipInfo("mimetype", date_time=(1980, 1, 1, 0, 0, 0))
        zi.compress_type = zipfile.ZIP_STORED
        zf.writestr(zi, MIMETYPE)
        doc_bytes = _canonical_json(document)
        sorted_resources: list[dict[str, Any]] = sorted(
            manifest.get("resources", []), key=lambda r: str(r.get("id", ""))
        )
        manifest = {**manifest, "resources": sorted_resources}
        manifest_bytes = _canonical_json(manifest)
        zi = zipfile.ZipInfo("manifest.json", date_time=(1980, 1, 1, 0, 0, 0))
        zi.compress_type = zipfile.ZIP_DEFLATED
        zf.writestr(zi, manifest_bytes)
        zi = zipfile.ZipInfo("document/document.json", date_time=(1980, 1, 1, 0, 0, 0))
        zi.compress_type = zipfile.ZIP_DEFLATED
        zf.writestr(zi, doc_bytes)
        for res in manifest.get("resources", []):
            data = (resource_files or {}).get(res["path"])
            if data is None:
                raise PtndError(f"missing resource bytes: {res['path']}")
            digest = hashlib.sha256(data).hexdigest()
            if digest != res["sha256"] or len(data) != int(res["byteSize"]):
                raise PtndError(f"resource metadata mismatch: {res['id']}")
            _safe_entry(res["path"])
            zi = zipfile.ZipInfo(res["path"], date_time=(1980, 1, 1, 0, 0, 0))
            zi.compress_type = zipfile.ZIP_DEFLATED
            zf.writestr(zi, data)

    tmp = path.with_suffix(path.suffix + ".tmp")
    tmp.write_bytes(buf.getvalue())
    tmp.replace(path)


def read_package(path: Path) -> tuple[dict[str, Any], dict[str, Any]]:
    try:
        zf = zipfile.ZipFile(path)
    except zipfile.BadZipFile as exc:
        raise PtndError("not a zip container") from exc
    with zf:
        names = zf.namelist()
        if len(names) > MAX_ENTRIES:
            raise PtndError("too many entries")
        total = sum(i.file_size for i in zf.infolist())
        if total > MAX_TOTAL_BYTES:
            raise PtndError("declared size too large")
        for i in zf.infolist():
            _safe_entry(i.filename)
            if i.file_size > MAX_ENTRY_BYTES:
                raise PtndError("entry too large")

        if names[0] != "mimetype" or zf.read("mimetype") != MIMETYPE:
            raise PtndError("missing/invalid mimetype (must be first, uncompressed)")

        try:
            manifest = json.loads(zf.read("manifest.json"))
            document = json.loads(zf.read("document/document.json"))
        except (KeyError, json.JSONDecodeError) as exc:
            raise PtndError("missing or invalid canonical JSON") from exc

        _check_depth(manifest)
        _check_depth(document)

        for res in manifest.get("resources", []):
            data = zf.read(_safe_entry(res["path"]))
            if len(data) != int(res["byteSize"]):
                raise PtndError(f"size mismatch: {res['id']}")
            if hashlib.sha256(data).hexdigest() != res["sha256"]:
                raise PtndError(f"sha256 mismatch: {res['id']}")

        unsupported = {"ptnd.unsupported"}
        for cap in manifest.get("requiredCapabilities", []):
            if cap in unsupported:
                raise PtndError(f"required capability unsupported: {cap}")

        return manifest, document
