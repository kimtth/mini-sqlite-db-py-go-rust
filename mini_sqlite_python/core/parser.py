"""Lightweight SQL parser returning structured command dictionaries."""

from __future__ import annotations

import ast
import re
from dataclasses import dataclass
from typing import Any, Dict, List


Condition = Dict[str, Any]


@dataclass
class ParsedCommand:
    """Container describing a parsed SQL command."""

    command: str
    raw: str
    tokens: List[str]
    details: Dict[str, Any]


class SQLParser:
    """Parse a subset of SQL into simple dictionaries."""

    _SELECT_RE = re.compile(
        r"SELECT\s+(?P<cols>.+?)\s+FROM\s+(?P<table>\w+)"
        r"(?:\s+INNER\s+JOIN\s+(?P<join_table>\w+)\s+ON\s+(?P<left_table>\w+)\.(?P<left_col>\w+)\s*=\s*(?P<right_table>\w+)\.(?P<right_col>\w+))?"
        r"(?:\s+WHERE\s+(?P<where_col>\w+)\s*=\s*(?P<where_val>.+))?$",
        re.IGNORECASE,
    )

    def parse(self, query: str) -> ParsedCommand:
        """Parse SQL text into a :class:`ParsedCommand`."""
        raw = query.strip()
        if not raw:
            return ParsedCommand(command="EMPTY", raw="", tokens=[], details={})
        text = raw.rstrip(";")
        tokens = text.split()
        command = tokens[0].upper()
        
        parsers = {
            ("CREATE", "DATABASE"): lambda: ("CREATE_DATABASE", {"name": tokens[2].lower()}),
            ("ALTER", "DATABASE"): lambda: ("ALTER_DATABASE", {"name": tokens[2].lower()}),
            ("CREATE", "TABLE"): lambda: ("CREATE_TABLE", self._parse_create_table(text)),
            ("ALTER", "TABLE"): lambda: ("ALTER_TABLE", self._parse_alter_table(text)),
            ("DROP", "TABLE"): lambda: ("DROP_TABLE", {"table": tokens[2].lower()}),
            ("CREATE", "INDEX"): lambda: ("CREATE_INDEX", {"table": tokens[2].lower(), "column": tokens[3].lower()}),
            ("DROP", "INDEX"): lambda: ("DROP_INDEX", {"table": tokens[2].lower(), "column": tokens[3].lower()}),
        }
        
        if command == "COMMIT":
            details = {}
        elif command == "USE":
            if len(tokens) > 1:
                command, details = "USE_DATABASE", {"name": tokens[1].lower()}
            else:
                command, details = "UNKNOWN", {}
        elif command in ("INSERT", "UPDATE", "DELETE", "SELECT"):
            parser = getattr(self, f"_parse_{command.lower()}")
            command, details = command, parser(text)
        elif len(tokens) > 1:
            key = (command, tokens[1].upper())
            if key in parsers:
                command, details = parsers[key]()
            else:
                command, details = "UNKNOWN", {}
        else:
            command, details = "UNKNOWN", {}

        return ParsedCommand(command=command, raw=raw, tokens=tokens, details=details)

    def _parse_create_table(self, text: str) -> Dict[str, Any]:
        header, cols = text.split("(", 1)
        table = header.split()[2].lower()
        inner = cols.rsplit(")", 1)[0]
        column_defs = []
        for chunk in inner.split(","):
            parts = chunk.strip().split()
            if not parts:
                continue
            name = parts[0].lower()
            col_type = parts[1].upper() if len(parts) > 1 else "TEXT"
            column_defs.append({"name": name, "type": col_type})
        return {"table": table, "columns": column_defs}

    def _parse_alter_table(self, text: str) -> Dict[str, Any]:
        tokens = text.split()
        table = tokens[2].lower()
        if len(tokens) >= 7 and tokens[3].upper() == "ADD" and tokens[4].upper() == "COLUMN":
            name = tokens[5].lower()
            col_type = tokens[6].upper() if len(tokens) > 6 else "TEXT"
            return {"action": "ADD_COLUMN", "table": table, "column": {"name": name, "type": col_type}}
        return {}

    def _parse_insert(self, text: str) -> Dict[str, Any]:
        match = re.match(r"INSERT\s+INTO\s+(?P<table>\w+)\s+VALUES\s*\((?P<values>.+)\)$", text, re.IGNORECASE)
        if not match:
            return {}
        values = self._parse_value_list(match.group("values"))
        return {"table": match.group("table").lower(), "values": values}

    def _parse_update(self, text: str) -> Dict[str, Any]:
        upper = text.upper()
        where_part = None
        if " WHERE " in upper:
            prefix, where_part = self._split_keyword(text, "WHERE")
        else:
            prefix = text
        table = prefix.split()[1].lower()
        set_clause = prefix.split(" SET ", 1)[1]
        assignments = {}
        for chunk in set_clause.split(","):
            column, value = chunk.split("=", 1)
            assignments[column.strip().lower()] = self._parse_literal(value.strip())
        where = self._parse_condition(where_part) if where_part else None
        return {"table": table, "assignments": assignments, "where": where}

    def _parse_delete(self, text: str) -> Dict[str, Any]:
        upper = text.upper()
        where_part = None
        if " WHERE " in upper:
            prefix, where_part = self._split_keyword(text, "WHERE")
        else:
            prefix = text
        table = prefix.split()[2].lower()
        where = self._parse_condition(where_part) if where_part else None
        return {"table": table, "where": where}

    def _parse_select(self, text: str) -> Dict[str, Any]:
        match = self._SELECT_RE.match(text)
        if not match:
            return {}
        cols = [col.strip() for col in match.group("cols").split(",")]
        table = match.group("table").lower()
        where = {"column": match.group("where_col").lower(), "value": self._parse_literal(match.group("where_val"))} if match.group("where_col") else None
        join = {
            "table": match.group("join_table").lower(),
            "left_table": match.group("left_table").lower(),
            "left_column": match.group("left_col").lower(),
            "right_table": match.group("right_table").lower(),
            "right_column": match.group("right_col").lower(),
        } if match.group("join_table") else None
        return {"table": table, "columns": cols, "where": where, "join": join}

    def _parse_value_list(self, segment: str) -> List[Any]:
        values = ast.literal_eval(f"({segment})")
        if not isinstance(values, tuple):
            values = (values,)
        return list(values)

    def _parse_condition(self, clause: str | None) -> Condition:
        if not clause:
            return {}
        column, value = clause.split("=", 1)
        return {"column": column.strip().lower(), "value": self._parse_literal(value.strip())}

    def _parse_literal(self, text: str) -> Any:
        try:
            return ast.literal_eval(text)
        except Exception:
            return text.strip("'")

    def _split_keyword(self, text: str, keyword: str) -> List[str]:
        match = re.search(rf"\b{keyword}\b", text, re.IGNORECASE)
        if not match:
            return [text]
        return [text[:match.start()].strip(), text[match.end():].strip()]
