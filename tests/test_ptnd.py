import hashlib
from pathlib import Path
from typing import Any

import pytest

from petunia_app.ptnd import PtndError, atomic_write, read_package, write_package

DOC: dict[str, Any] = {
    "version": 1,
    "documentId": "11111111-2222-3333-4444-555555555555",
    "name": "T",
    "surfaces": [],
}
MANIFEST: dict[str, Any] = {
    "formatId": "ptnd.document",
    "containerVersion": 1,
    "schemaVersion": 1,
    "minimumReaderVersion": "1.0",
    "writer": {"appVersion": "0.1.0"},
    "documentId": "11111111-2222-3333-4444-555555555555",
    "requiredCapabilities": [],
    "resources": [],
    "extensions": [],
    "interchange": [],
}


def test_roundtrip(tmp_path: Path) -> None:
    pkg = tmp_path / "a.ptnd"
    write_package(pkg, MANIFEST, DOC)
    manifest, document = read_package(pkg)
    assert document["name"] == "T"
    assert manifest["formatId"] == "ptnd.document"


def test_deterministic_rewrite(tmp_path: Path) -> None:
    a, b = tmp_path / "a.ptnd", tmp_path / "b.ptnd"
    write_package(a, MANIFEST, DOC)
    write_package(b, dict(MANIFEST), dict(DOC))
    assert a.read_bytes() == b.read_bytes()


def test_resource_hash_mismatch(tmp_path: Path) -> None:
    data = b"hello"
    bad = {
        **MANIFEST,
        "resources": [
            {
                "id": "r1",
                "kind": "raster",
                "path": "resources/r1.bin",
                "mediaType": "application/octet-stream",
                "byteSize": len(data),
                "sha256": "0" * 64,
                "canonical": True,
            }
        ],
    }
    pkg = tmp_path / "a.ptnd"
    with pytest.raises(PtndError, match="mismatch"):
        write_package(pkg, bad, DOC, resource_files={"resources/r1.bin": data})


def test_resource_included_and_checked(tmp_path: Path) -> None:
    data = b"hello"
    good = {
        **MANIFEST,
        "resources": [
            {
                "id": "r1",
                "kind": "raster",
                "path": "resources/r1.bin",
                "mediaType": "application/octet-stream",
                "byteSize": len(data),
                "sha256": hashlib.sha256(data).hexdigest(),
                "canonical": True,
            }
        ],
    }
    pkg = tmp_path / "a.ptnd"
    write_package(pkg, good, DOC, resource_files={"resources/r1.bin": data})
    read_package(pkg)


def test_corrupt_package_rejected(tmp_path: Path) -> None:
    pkg = tmp_path / "a.ptnd"
    pkg.write_bytes(b"not a zip")
    with pytest.raises(PtndError):
        read_package(pkg)


def test_atomic_write_preserves_last_valid(tmp_path: Path) -> None:
    target = tmp_path / "doc.ptnd"
    target.write_bytes(b"valid")

    def boom(tmp: Path) -> None:
        tmp.write_bytes(b"partial")
        raise OSError("disk full")

    with pytest.raises(OSError):
        atomic_write(target, boom)
    assert target.read_bytes() == b"valid"
