/// Minimal HTTP server exposing the database engine via a form.
use crate::core::engine::DatabaseEngine;
use serde_json::to_string_pretty;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

const HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <title>mini SQL shell</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 2rem; display: grid; gap: 2rem; grid-template-columns: 2fr 1fr; }
        main { grid-column: 1; }
        aside { grid-column: 2; }
        textarea { width: 100%; height: 8rem; }
        pre { background: #f0f0f0; padding: 1rem; white-space: pre-wrap; }
        .schema-tree ul { list-style: none; padding-left: 1rem; }
        .schema-tree li { margin: 0.25rem 0; }
        .schema-tree strong { color: #1f5aa6; }
        .schema-tree .table-name { font-weight: 600; }
        .db-switcher { display: flex; gap: 0.5rem; align-items: center; margin-bottom: 1rem; }
        .db-switcher select { flex: 1; }
        .log-panel { border: 1px solid #e0e0e0; padding: 1rem; border-radius: 0.5rem; background: #fafafa; }
        .log-panel pre { max-height: 18rem; overflow-y: auto; }
    </style>
</head>
<body>
    <main>
        <h1>mini SQL shell</h1>
        <form method="POST">
            <textarea name="query" placeholder="SELECT * FROM users;">{query}</textarea>
            <br />
            <button type="submit">Run</button>
        </form>
        <h2>Result</h2>
        <pre>{result}</pre>
    </main>
    <aside>
        <h2>Databases</h2>
        <p>Active: <strong>{active_db}</strong></p>
        <form method="POST" class="db-switcher">
            <label for="use_database">Switch</label>
            <select name="use_database" id="use_database">
                {db_options}
            </select>
            <button type="submit">Use</button>
        </form>
        <div class="schema-tree">{schema}</div>
        <section class="log-panel">
            <h2>Pending log</h2>
            {lsm_log}
        </section>
    </aside>
</body>
</html>
"#;

fn handle_client(mut stream: TcpStream, engine: Arc<Mutex<DatabaseEngine>>) {
    let mut buffer = [0; 4096];
    let bytes_read = stream.read(&mut buffer).unwrap_or(0);

    if bytes_read == 0 {
        return;
    }

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let lines: Vec<&str> = request.lines().collect();

    if lines.is_empty() {
        return;
    }

    let request_line = lines[0];
    let parts: Vec<&str> = request_line.split_whitespace().collect();

    if parts.len() < 2 {
        return;
    }

    let method = parts[0];

    let (query, result) = if method == "POST" {
        // Find the body
        let body_start = request.find("\r\n\r\n").unwrap_or(0) + 4;
        let body = &request[body_start..];

        // Parse form data
        let use_db = parse_form_value(body, "use_database");
        let mut result_text = String::new();
        if !use_db.is_empty() {
            let mut eng = engine.lock().unwrap();
            let results = eng.execute(&format!("USE {};", use_db));
            result_text = results.join("\n");
        }
        let query_text = parse_form_value(body, "query");
        if !query_text.is_empty() {
            let mut eng = engine.lock().unwrap();
            let results = eng.execute(&query_text);
            result_text = results.join("\n");
        }

        (query_text, result_text)
    } else {
        (String::new(), String::new())
    };

    let eng = engine.lock().unwrap();
    let schema = generate_schema_html(&eng);
    let active_db = eng.active_database().to_string();
    let db_options = database_options(&eng);
    let log_html = lsm_log_html(&eng);
    drop(eng);

    let result_display = if result.is_empty() {
        "(no output)".to_string()
    } else {
        result
    };

    let html = HTML_TEMPLATE
        .replace("{query}", &html_escape(&query))
        .replace("{result}", &html_escape(&result_display))
        .replace("{schema}", &schema)
        .replace("{active_db}", &html_escape(&active_db))
        .replace("{db_options}", &db_options)
        .replace("{lsm_log}", &log_html);

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
        html.len(),
        html
    );

    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn parse_form_value(body: &str, key: &str) -> String {
    for param in body.split('&') {
        if let Some(rest) = param.strip_prefix(&format!("{}=", key)) {
            return url_decode(rest);
        }
    }
    String::new()
}

fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '+' => result.push(' '),
            '%' => {
                let mut hex = String::new();
                if let Some(h1) = chars.next() {
                    hex.push(h1);
                }
                if let Some(h2) = chars.next() {
                    hex.push(h2);
                }
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                }
            }
            _ => result.push(ch),
        }
    }

    result
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn generate_schema_html(engine: &DatabaseEngine) -> String {
    let snapshot = engine.describe();

    if snapshot.is_empty() {
        return "<p>No databases yet.</p>".to_string();
    }

    let mut lines = vec!["<ul>".to_string()];
    let mut db_names: Vec<&String> = snapshot.keys().collect();
    db_names.sort();

    for db_name in db_names {
        let tables = &snapshot[db_name];
        lines.push(format!("<li><strong>{}</strong>", html_escape(db_name)));

        if !tables.is_empty() {
            lines.push("<ul>".to_string());
            let mut table_names: Vec<&String> = tables.keys().collect();
            table_names.sort();

            for table_name in table_names {
                let info = &tables[table_name];

                let columns = info
                    .get("columns")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_else(|| "(no columns)".to_string());

                let row_count = info.get("row_count").and_then(|v| v.as_u64()).unwrap_or(0);

                let indexes = info
                    .get("indexes")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        let idx_list: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
                        if idx_list.is_empty() {
                            "none".to_string()
                        } else {
                            idx_list.join(", ")
                        }
                    })
                    .unwrap_or_else(|| "none".to_string());

                lines.push(format!(
                    "<li><span class='table-name'>{}</span><br />\
                    <small>cols: {}</small><br />\
                    <small>rows: {}, idx: {}</small></li>",
                    html_escape(table_name),
                    html_escape(&columns),
                    row_count,
                    html_escape(&indexes)
                ));
            }
            lines.push("</ul>".to_string());
        }
        lines.push("</li>".to_string());
    }
    lines.push("</ul>".to_string());

    lines.join("")
}

fn database_options(engine: &DatabaseEngine) -> String {
    let names = engine.databases();
    if names.is_empty() {
        return "<option value=\"\">(none)</option>".to_string();
    }
    let active = engine.active_database();
    let mut options = Vec::new();
    for name in names {
        let selected = if name == active { " selected" } else { "" };
        options.push(format!(
            "<option value='{}'{}>{}</option>",
            html_escape(&name),
            selected,
            html_escape(&name)
        ));
    }
    options.join("")
}

fn lsm_log_html(engine: &DatabaseEngine) -> String {
    let entries = engine.lsm_entries();
    if entries.is_empty() {
        return "<p>No pending log entries.</p>".to_string();
    }

    let mut rendered = Vec::with_capacity(entries.len());
    for entry in entries {
        match to_string_pretty(&entry) {
            Ok(json) => rendered.push(html_escape(&json)),
            Err(_) => rendered.push(String::from("{}")),
        }
    }

    format!("<pre>{}</pre>", rendered.join("\n"))
}

pub fn run_server(host: &str, port: u16) {
    let addr = format!("{}:{}", host, port);
    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to {}: {}", addr, e);
            return;
        }
    };

    println!("Serving mini SQL UI on http://{}", addr);

    let engine = Arc::new(Mutex::new(DatabaseEngine::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let engine_clone = Arc::clone(&engine);
                handle_client(stream, engine_clone);
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}
