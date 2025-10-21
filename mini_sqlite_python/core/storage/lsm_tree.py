"""In-memory placeholder for an LSM-style commit log."""

from __future__ import annotations

from typing import Any, Dict, List


class LSMTreeStorage:
    """Collects mutations until they are committed."""

    def __init__(self) -> None:
        self._segments: List[Dict[str, Any]] = []

    def log(self, entry: Dict[str, Any]) -> None:
        """Record a mutation event."""
        self._segments.append(entry)

    def pending(self) -> int:
        """Return the number of uncommitted entries."""
        return len(self._segments)

    def snapshot(self) -> List[Dict[str, Any]]:
        """Return a copy of the current pending entries."""
        return list(self._segments)

    def commit(self) -> List[Dict[str, Any]]:
        """Flush all pending entries and compact the log."""
        flushed = list(self._segments)
        self._segments.clear()
        self.compact()
        return flushed

    def compact(self) -> None:
        """Retain only a limited window of committed history."""
        self._segments = self._segments[-10:]
