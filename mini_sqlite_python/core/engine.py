"""High level database engine wiring together parser, executor, and storage."""

from __future__ import annotations

from typing import Any, Dict, Iterable, List

from .executor import SQLExecutor
from .parser import SQLParser


class DatabaseEngine:
    """Coordinates request parsing and execution."""

    def __init__(self) -> None:
        self._parser = SQLParser()
        self._executor = SQLExecutor()

    def execute(self, query: str) -> Iterable[str]:
        """Parse and execute a SQL query string."""
        parsed = self._parser.parse(query)
        return self._executor.execute(parsed)

    def describe(self) -> dict[str, dict[str, dict[str, object]]]:
        """Return a snapshot of databases, tables, and columns."""
        return self._executor.describe()

    def active_database(self) -> str:
        """Expose the name of the currently active database."""
        return self._executor.active_database

    def databases(self) -> tuple[str, ...]:
        """List available database names."""
        return self._executor.databases()

    def lsm_entries(self) -> List[Dict[str, Any]]:
        """Expose pending LSM log entries for inspection."""
        return self._executor.lsm_entries()
