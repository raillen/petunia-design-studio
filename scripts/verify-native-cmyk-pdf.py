#!/usr/bin/env python3
"""Check native CMYK engineering fixtures independently of the Rust exporter.

Usage: python3 scripts/verify-native-cmyk-pdf.py ARTIFACT_DIRECTORY
Requires pypdf and Poppler on PATH. This does not establish PDF/X or print
conformance. Expected samples come from crates/petunia_design_io/tests/native_cmyk.rs.
"""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess

import pypdf


def images(resources, inherited=None):
    spaces = dict(inherited or {})
    spaces.update(resources.get("/ColorSpace", {}))
    for ref in resources.get("/XObject", {}).values():
        obj = ref.get_object()
        if obj.get("/Subtype") == "/Image":
            yield obj, spaces
        elif obj.get("/Subtype") == "/Form" and "/Resources" in obj:
            yield from images(obj["/Resources"], spaces)


def verify(root, name, bits, profile):
    path = root / f"{name}.pdf"
    reader = pypdf.PdfReader(path, strict=True)
    assert len(reader.pages) == 1, "one page expected"
    found = list(images(reader.pages[0]["/Resources"]))
    assert len(found) == 1, "one authored native image expected"
    image, spaces = found[0]
    assert (image["/Width"], image["/Height"]) == (3, 1)
    space = image["/ColorSpace"]
    if isinstance(space, pypdf.generic.NameObject):
        space = spaces[space].get_object()
    assert space[0] == "/ICCBased"
    icc = space[1].get_object()
    assert icc["/N"] == 4 and icc.get_data() == profile, "original CMYK ICC required"
    assert image["/BitsPerComponent"] == bits
    samples = (
        [255, 0, 0, 0, 255, 0, 0, 0, 200, 127, 1, 2, 3, 4, 0]
        if bits == 8 else
        [65535, 1, 257, 0, 65535, 0, 0, 0, 0x1234, 0x4567, 123, 456, 789, 1234, 0]
    )
    original = b"".join(
        value.to_bytes(bits // 8, "big")
        for index, value in enumerate(samples) if index % 5 != 4
    )
    alpha = b"".join(
        value.to_bytes(bits // 8, "big")
        for index, value in enumerate(samples) if index % 5 == 4
    )
    assert image.get_data() == original, "original CMYK samples required"
    mask = image["/SMask"].get_object()
    assert mask["/BitsPerComponent"] == bits
    assert mask["/ColorSpace"] == "/DeviceGray"
    assert (mask["/Width"], mask["/Height"]) == (3, 1)
    assert mask.get_data() == alpha, "independent exact alpha required"
    subprocess.run(
        ["pdftoppm", "-scale-to", "600", "-png", "-singlefile", str(path),
         str(root / f"{name}-poppler")],
        check=True, capture_output=True,
    )
    with (root / f"{name}-images.txt").open("w") as output:
        subprocess.run(["pdfimages", "-list", str(path)], check=True, stdout=output)
    return {
        "file": path.name,
        "inks": 4,
        "bits_per_component": bits,
        "soft_mask_bits": bits,
        "icc_sha256": hashlib.sha256(icc.get_data()).hexdigest(),
        "ink_samples_exact": True,
        "alpha_samples_exact": True,
        "reader": f"pypdf {pypdf.__version__}",
        "independent_render": "Poppler",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    root = parser.parse_args().directory
    profile = (
        Path(__file__).resolve().parents[1] / "fixtures/color/synthetic-cmyk.icc"
    ).read_bytes()
    reports = [
        verify(root, f"{prefix}-Cmyka{bits}", bits, profile)
        for prefix in ("native", "image") for bits in (8, 16)
    ]
    serialized = json.dumps(reports, indent=2) + "\n"
    (root / "independent-pdf.json").write_text(serialized)
    print(serialized, end="")


if __name__ == "__main__":
    main()
