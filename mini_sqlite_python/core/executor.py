"""SQL executor orchestrating DDL, DML, and simple commits."""

from __future__ import annotations

from pathlib import Path
from typing import Any, Dict, Iterable, List

from .parser import ParsedCommand
from .storage.btree import BTreeStorage
from .storage.lsm_tree import LSMTreeStorage
from .storage.pager import Pager


class SQLExecutor:
    """Execute parsed commands against in-memory storage."""

    def __init__(self, data_root: Path | None = None) -> None:
        self._lsm = LSMTreeStorage()
        self._data_dir = Path(data_root) if data_root else Path(__file__).resolve().parent.parent / "data"
        self._data_dir.mkdir(parents=True, exist_ok=True)
        self._databases: dict[str, BTreeStorage] = {}
        for file in sorted(self._data_dir.glob("*.dat")):
            name = file.stem.lower()
            self._databases[name] = BTreeStorage(Pager(file, page_size=4096))
        if not self._databases:
            self._ensure_database("default")
        self._active_db = "default" if "default" in self._databases else next(iter(self._databases))

    def execute(self, parsed: ParsedCommand) -> Iterable[str]:
        cmd, details = parsed.command, parsed.details
        
        if cmd == "EMPTY":
            return ("",)
        if cmd == "CREATE_DATABASE":
            self._ensure_database(details["name"])
            self._active_db = details["name"]
            return (f"Database '{details['name']}' ready.",)
        if cmd == "ALTER_DATABASE":
            self._ensure_database(details["name"])
            self._active_db = details["name"]
            return (f"Using database '{details['name']}'.",)
        if cmd == "USE_DATABASE":
            name = details["name"]
            if name not in self._databases:
                return (f"Database '{name}' not found.",)
            self._active_db = name
            return (f"Using database '{name}'.",)
        
        storage = self._databases[self._active_db]
        table = details.get("table")
        
        if cmd == "CREATE_TABLE":
            if storage.table_exists(table):
                return (f"Table '{table}' already exists.",)
            storage.create_table(table, [col["name"] for col in details["columns"]])
            return (f"Table '{table}' created.",)
        
        if table and not storage.table_exists(table):
            return (f"Table '{table}' not found.",)
        
        if cmd == "ALTER_TABLE":
            storage.add_column(table, details["column"]["name"])
            return (f"Column '{details['column']['name']}' added to '{table}'.",)
        if cmd == "DROP_TABLE":
            storage.drop_table(table)
            return (f"Table '{table}' dropped.",)
        if cmd == "CREATE_INDEX":
            storage.create_index(table, details["column"])
            return (f"Index on {table}.{details['column']} built.",)
        if cmd == "DROP_INDEX":
            storage.drop_index(table, details["column"])
            return (f"Index on {table}.{details['column']} removed.",)
        if cmd == "INSERT":
            row = storage.insert_row(table, details["values"])
            self._lsm.log({"db": self._active_db, "command": "INSERT", "row": row})
            return ("1 row inserted.",)
        if cmd == "UPDATE":
            count = storage.update_rows(table, details["assignments"], details.get("where"))
            self._lsm.log({"db": self._active_db, "command": "UPDATE", "count": count})
            return (f"{count} row(s) updated.",)
        if cmd == "DELETE":
            count = storage.delete_rows(table, details.get("where"))
            self._lsm.log({"db": self._active_db, "command": "DELETE", "count": count})
            return (f"{count} row(s) deleted.",)
        if cmd == "SELECT":
            join = details.get("join")
            if join and not storage.table_exists(join["table"]):
                return (f"Table '{join['table']}' not found.",)
            rows = storage.select_rows(table, details["columns"], details.get("where"), join)
            return self._format_rows(rows, details["columns"])
        if cmd == "COMMIT":
            entries = self._lsm.commit()
            return (f"Committed {len(entries)} entr{'y' if len(entries)==1 else 'ies'}.",)
        return (f"Command '{parsed.raw}' not understood.",)

    def _ensure_database(self, name: str) -> None:
        if name not in self._databases:
            path = self._data_dir / f"{name}.dat"
            self._databases[name] = BTreeStorage(Pager(path, page_size=4096))

    def _format_rows(self, rows: List[dict[str, object]], requested: List[str]) -> Iterable[str]:
        if not rows:
            return ("(no rows)",)
        headers = requested if requested != ["*"] else list(rows[0].keys())
        lines = [" | ".join(headers)]
        for row in rows:
            lines.append(" | ".join(str(row.get(col, "")) for col in headers))
        return tuple(lines)

    def describe(self) -> dict[str, dict[str, dict[str, object]]]:
        return {name: storage.describe() for name, storage in self._databases.items()}

    @property
    def active_database(self) -> str:
        return self._active_db

    def databases(self) -> tuple[str, ...]:
        return tuple(sorted(self._databases))

    def lsm_entries(self) -> List[Dict[str, Any]]:
        return self._lsm.snapshot()
