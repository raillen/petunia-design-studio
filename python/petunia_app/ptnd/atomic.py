from __future__ import annotations

import os
from collections.abc import Callable
from pathlib import Path


def atomic_write(path: Path, write_fn: Callable[[Path], None]) -> None:
    tmp = path.with_name(path.name + ".tmp")
    try:
        write_fn(tmp)
        with open(tmp, "rb") as fh:
            os.fsync(fh.fileno())
        os.replace(tmp, path)
        dir_fd = os.open(path.parent, os.O_RDONLY)
        try:
            os.fsync(dir_fd)
        finally:
            os.close(dir_fd)
    finally:
        if tmp.exists():
            tmp.unlink()
