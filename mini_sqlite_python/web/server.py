"""Minimal HTTP server exposing the database engine via a form."""

from __future__ import annotations

import json
from html import escape
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import parse_qs

from core.engine import DatabaseEngine

HTML_TEMPLATE = """
<!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta charset=\"utf-8\" />
    <title>mini SQL shell</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 2rem; display: grid; gap: 2rem; grid-template-columns: 2fr 1fr; }}
        main {{ grid-column: 1; }}
        aside {{ grid-column: 2; }}
        textarea {{ width: 100%; height: 8rem; }}
        pre {{ background: #f0f0f0; padding: 1rem; white-space: pre-wrap; }}
        .schema-tree ul {{ list-style: none; padding-left: 1rem; }}
        .schema-tree li {{ margin: 0.25rem 0; }}
        .schema-tree strong {{ color: #1f5aa6; }}
        .schema-tree .table-name {{ font-weight: 600; }}
        .db-switcher {{ display: flex; gap: 0.5rem; align-items: center; margin-bottom: 1rem; }}
        .db-switcher select {{ flex: 1; }}
        .log-panel pre {{ max-height: 18rem; overflow-y: auto; }}
        .log-panel {{ border: 1px solid #e0e0e0; padding: 1rem; border-radius: 0.5rem; background: #fafafa; }}
    </style>
</head>
<body>
    <main>
        <h1>mini SQL shell</h1>
        <form method=\"POST\">
            <textarea name=\"query\" placeholder=\"SELECT * FROM users;\">{query}</textarea>
            <br />
            <button type=\"submit\">Run</button>
        </form>
        <h2>Result</h2>
        <pre>{result}</pre>
    </main>
    <aside>
        <h2>Databases</h2>
        <p>Active: <strong>{active_db}</strong></p>
        <form method=\"POST\" class=\"db-switcher\">
            <label for=\"use_database\">Switch</label>
            <select name=\"use_database\" id=\"use_database\">
                {db_options}
            </select>
            <button type=\"submit\">Use</button>
        </form>
        <div class=\"schema-tree\">{schema}</div>
        <section class=\"log-panel\">
            <h2>Pending log</h2>
            {lsm_log}
        </section>
    </aside>
</body>
</html>
"""


class SQLRequestHandler(BaseHTTPRequestHandler):
    """Serve a simple HTML page that accepts SQL queries."""

    engine = DatabaseEngine()

    def do_GET(self) -> None:  # noqa: N802 - HTTP verb naming convention
        self._render_page()

    def do_POST(self) -> None:  # noqa: N802 - HTTP verb naming convention
        length = int(self.headers.get("Content-Length", "0"))
        body = self.rfile.read(length).decode("utf-8") if length else ""
        params = parse_qs(body)
        use_db = params.get("use_database", [""])[0]
        result_lines = ""
        if use_db:
            result_lines = "\n".join(self.engine.execute(f"USE {use_db};"))
        query = params.get("query", [""])[0]
        if query:
            result_lines = "\n".join(self.engine.execute(query))
        self._render_page(query=query, result=result_lines)

    def _render_page(self, query: str = "", result: str = "") -> None:
        payload = HTML_TEMPLATE.format(
            query=escape(query),
            result=escape(result or "(no output)"),
            schema=self._schema_html(),
            active_db=self.engine.active_database(),
            db_options=self._database_options(),
            lsm_log=self._lsm_log_html(),
        )
        encoded = payload.encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(encoded)))
        self.end_headers()
        self.wfile.write(encoded)

    def _schema_html(self) -> str:
        snapshot = self.engine.describe()
        if not snapshot:
            return "<p>No databases yet.</p>"
        lines = ["<ul>"]
        for db_name in sorted(snapshot.keys()):
            tables = snapshot[db_name]
            lines.append(f"<li><strong>{escape(db_name)}</strong>")
            if tables:
                lines.append("<ul>")
                for table_name in sorted(tables.keys()):
                    info = tables[table_name]
                    columns = ", ".join(info["columns"]) or "(no columns)"
                    indexes = ", ".join(info["indexes"]) or "none"
                    lines.append(
                        "<li>"
                        f"<span class='table-name'>{escape(table_name)}</span>"
                        f"<br /><small>cols: {escape(columns)}</small>"
                        f"<br /><small>rows: {info['row_count']}, idx: {escape(indexes)}</small>"
                        "</li>"
                    )
                lines.append("</ul>")
            lines.append("</li>")
        lines.append("</ul>")
        return "".join(lines)

    def _database_options(self) -> str:
        names = self.engine.databases()
        active = self.engine.active_database()
        if not names:
            return "<option value=\"\">(none)</option>"
        options = []
        for name in names:
            selected = " selected" if name == active else ""
            safe = escape(name)
            options.append(f"<option value='{safe}'{selected}>{safe}</option>")
        return "".join(options)

    def _lsm_log_html(self) -> str:
        entries = self.engine.lsm_entries()
        if not entries:
            return "<p>No pending log entries.</p>"
        rendered = [json.dumps(entry, ensure_ascii=False, sort_keys=True) for entry in entries]
        return "<pre>" + escape("\n".join(rendered)) + "</pre>"


def run_server(host: str = "127.0.0.1", port: int = 8000) -> None:
    """Start the HTTP server and serve until interrupted."""
    httpd = HTTPServer((host, port), SQLRequestHandler)
    print(f"Serving mini SQL UI on http://{host}:{port}")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down web server...")
    finally:
        httpd.server_close()


if __name__ == "__main__":
    run_server()
