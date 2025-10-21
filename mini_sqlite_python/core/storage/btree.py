"""Simplified in-memory B-Tree style table manager."""

from __future__ import annotations

import json
from typing import Any, Dict, Iterable, List, Optional


class BTreeStorage:
    """Manages table data, rudimentary indexes, and simple joins."""

    def __init__(self, pager: Any | None = None) -> None:
        self._pager = pager
        self._tables: Dict[str, Dict[str, Any]] = {}
        if self._pager:
            self._load()

    def create_table(self, name: str, columns: List[str]) -> None:
        self._tables[name] = {"columns": columns, "rows": [], "indexes": {}}
        self._persist()

    def drop_table(self, name: str) -> None:
        self._tables.pop(name, None)
        self._persist()

    def table_exists(self, name: str) -> bool:
        return name in self._tables

    def add_column(self, name: str, column: str) -> None:
        table = self._tables[name]
        if column not in table["columns"]:
            table["columns"].append(column)
            for row in table["rows"]:
                row[column] = None
            for col in list(table["indexes"].keys()):
                self._rebuild_index(name, col)
            self._persist()

    def create_index(self, name: str, column: str) -> None:
        table = self._tables[name]
        if column not in table["indexes"]:
            self._rebuild_index(name, column)
            self._persist()

    def drop_index(self, name: str, column: str) -> None:
        table = self._tables.get(name)
        if table:
            table["indexes"].pop(column, None)
            self._persist()

    def insert_row(self, name: str, values: List[Any]) -> Dict[str, Any]:
        table = self._tables[name]
        row = {col: val for col, val in zip(table["columns"], values)}
        table["rows"].append(row)
        for column, index in table["indexes"].items():
            index.setdefault(row.get(column), []).append(row)
        self._persist()
        return row

    def select_rows(
        self,
        table: str,
        columns: List[str],
        where: Optional[Dict[str, Any]],
        join: Optional[Dict[str, str]],
    ) -> List[Dict[str, Any]]:
        if join:
            return self._join_rows(table, columns, where, join)
        rows = list(self._rows_for(table, where))
        if columns == ["*"]:
            return [row.copy() for row in rows]
        selected: List[Dict[str, Any]] = []
        for row in rows:
            projected = {}
            for col in columns:
                lookup = col.split(".", 1)[-1]
                projected[col] = row.get(lookup)
            selected.append(projected)
        return selected

    def update_rows(
        self,
        table: str,
        assignments: Dict[str, Any],
        where: Optional[Dict[str, Any]],
    ) -> int:
        rows = list(self._rows_for(table, where))
        for row in rows:
            row.update(assignments)
        affected_cols = set(assignments.keys()) & set(self._tables[table]["indexes"].keys())
        for column in affected_cols:
            self._rebuild_index(table, column)
        if rows:
            self._persist()
        return len(rows)

    def delete_rows(self, table: str, where: Optional[Dict[str, Any]]) -> int:
        current = self._tables[table]["rows"]
        if not where:
            deleted = len(current)
            self._tables[table]["rows"] = []
        else:
            kept = [row for row in current if row.get(where["column"]) != where["value"]]
            deleted = len(current) - len(kept)
            self._tables[table]["rows"] = kept
        for column in self._tables[table]["indexes"]:
            self._rebuild_index(table, column)
        if deleted:
            self._persist()
        return deleted

    def _rows_for(self, table: str, where: Optional[Dict[str, Any]]) -> Iterable[Dict[str, Any]]:
        data = self._tables[table]
        if not where:
            yield from data["rows"]
            return
        column = where["column"]
        value = where["value"]
        index = data["indexes"].get(column)
        if index is not None:
            yield from index.get(value, [])
        else:
            for row in data["rows"]:
                if row.get(column) == value:
                    yield row

    def _join_rows(
        self,
        table: str,
        columns: List[str],
        where: Optional[Dict[str, Any]],
        join: Dict[str, str],
    ) -> List[Dict[str, Any]]:
        left_rows = list(self._rows_for(table, where))
        right_table = join["table"]
        right_data = self._tables[right_table]
        right_column = join["right_column"]
        index = right_data["indexes"].get(right_column)
        if index is None:
            self._rebuild_index(right_table, right_column)
            index = right_data["indexes"].get(right_column, {})
        result: List[Dict[str, Any]] = []
        for left in left_rows:
            key = left.get(join["left_column"])
            for right in index.get(key, []):
                left_prefix = join["left_table"]
                right_prefix = join["right_table"]
                combined = {f"{left_prefix}.{col}": left.get(col) for col in self._tables[table]["columns"]}
                combined.update({f"{right_prefix}.{col}": right.get(col) for col in right_data["columns"]})
                if columns != ["*"]:
                    projected = {}
                    for col in columns:
                        if "." in col:
                            projected[col] = combined.get(col)
                        elif col in self._tables[table]["columns"]:
                            projected[col] = left.get(col)
                        elif col in right_data["columns"]:
                            projected[col] = right.get(col)
                        else:
                            projected[col] = None
                    combined = projected
                result.append(combined)
        return result

    def _rebuild_index(self, table: str, column: str) -> None:
        data = self._tables[table]
        index = {}
        for row in data["rows"]:
            index.setdefault(row.get(column), []).append(row)
        data["indexes"][column] = index

    def describe(self) -> Dict[str, Dict[str, Any]]:
        summary: Dict[str, Dict[str, Any]] = {}
        for name, meta in self._tables.items():
            summary[name] = {
                "columns": list(meta["columns"]),
                "row_count": len(meta["rows"]),
                "indexes": sorted(meta["indexes"].keys()),
            }
        return summary

    def _load(self) -> None:
        blob = self._pager.read_blob()
        if not blob:
            return
        data = json.loads(blob.decode("utf-8"))
        for name, meta in data.items():
            columns = list(meta.get("columns", []))
            rows = [dict(row) for row in meta.get("rows", [])]
            self._tables[name] = {"columns": columns, "rows": rows, "indexes": {}}
            for column in meta.get("indexes", []):
                self._rebuild_index(name, column)

    def _persist(self) -> None:
        if not self._pager:
            return
        snapshot: Dict[str, Dict[str, Any]] = {}
        for name, meta in self._tables.items():
            rows = [dict(row) for row in meta["rows"]]
            snapshot[name] = {
                "columns": list(meta["columns"]),
                "rows": rows,
                "indexes": sorted(meta["indexes"].keys()),
            }
        payload = json.dumps(snapshot, separators=(",", ":")).encode("utf-8")
        self._pager.write_blob(payload)
