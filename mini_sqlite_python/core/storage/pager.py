"""Disk-backed pager persisting fixed-size pages in a .dat file."""

from __future__ import annotations

from pathlib import Path
from typing import Dict, List


MAGIC = b"MDB1"
HEADER_SIZE = 16


class Pager:
    """Maintains a list of pages mirrored to disk."""

    def __init__(self, file_path: str | Path, page_size: int = 4096) -> None:
        self.page_size = page_size
        self._path = Path(file_path)
        self._path.parent.mkdir(parents=True, exist_ok=True)
        self._pages: List[bytearray] = []
        self._length = 0
        self._load()

    def allocate_page(self) -> int:
        """Allocate a fresh zeroed page and return its index."""
        self._pages.append(bytearray(self.page_size))
        return len(self._pages) - 1

    def write_page(self, index: int, data: bytes) -> None:
        """Write data to an existing page, truncating if necessary."""
        while index >= len(self._pages):
            self._pages.append(bytearray(self.page_size))
        buffer = bytearray(self.page_size)
        buffer[: len(data)] = data[: self.page_size]
        self._pages[index] = buffer
        self._flush()

    def read_page(self, index: int) -> bytes:
        """Return the bytes stored at a page index."""
        return bytes(self._pages[index])

    def write_blob(self, data: bytes) -> None:
        """Persist an arbitrary payload across consecutive pages."""
        self._length = len(data)
        if not data:
            self._pages = []
            self._flush()
            return
        needed = (len(data) + self.page_size - 1) // self.page_size
        buffer = data.ljust(needed * self.page_size, b"\x00")
        self._pages = [bytearray(buffer[i : i + self.page_size]) for i in range(0, len(buffer), self.page_size)]
        self._flush()

    def read_blob(self) -> bytes:
        """Return the concatenated payload stored in the pager."""
        if not self._pages or self._length == 0:
            return b""
        data = b"".join(bytes(page) for page in self._pages)
        return data[: self._length]

    def stats(self) -> Dict[str, int]:
        """Provide simple pager statistics."""
        return {"pages": len(self._pages), "page_size": self.page_size}

    def _load(self) -> None:
        if not self._path.exists():
            return
        raw = self._path.read_bytes()
        if len(raw) < HEADER_SIZE or not raw.startswith(MAGIC):
            return
        stored_size = int.from_bytes(raw[4:8], "little") or self.page_size
        self.page_size = stored_size
        self._length = int.from_bytes(raw[8:16], "little")
        payload = raw[HEADER_SIZE:]
        self._pages = []
        for offset in range(0, len(payload), self.page_size):
            chunk = bytearray(payload[offset : offset + self.page_size])
            if len(chunk) < self.page_size:
                chunk.extend(b"\x00" * (self.page_size - len(chunk)))
            self._pages.append(chunk)

    def _flush(self) -> None:
        if not self._pages:
            if self._path.exists():
                self._path.write_bytes(b"")
            return
        header = MAGIC + self.page_size.to_bytes(4, "little") + self._length.to_bytes(8, "little")
        body = b"".join(bytes(page) for page in self._pages)
        self._path.write_bytes(header + body)
