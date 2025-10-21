package web

import (
	"encoding/json"
	"fmt"
	"html"
	"mini_sqlite/core"
	"net/http"
	"sort"
	"strings"
)

const htmlTemplate = `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <title>mini SQL shell</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 2rem; display: grid; gap: 2rem; grid-template-columns: 2fr 1fr; }
        main { grid-column: 1; }
        aside { grid-column: 2; }
        textarea { width: 100%%; height: 8rem; }
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
            <textarea name="query" placeholder="SELECT * FROM users;">%s</textarea>
            <br />
            <button type="submit">Run</button>
        </form>
        <h2>Result</h2>
        <pre>%s</pre>
    </main>
    <aside>
        <h2>Databases</h2>
        <p>Active: <strong>%s</strong></p>
		<form method="POST" class="db-switcher">
			<label for="use_database">Switch</label>
			<select name="use_database" id="use_database">
				%s
			</select>
			<button type="submit">Use</button>
		</form>
        <div class="schema-tree">%s</div>
        <section class="log-panel">
            <h2>Pending log</h2>
            %s
        </section>
    </aside>
</body>
</html>`

var engine *core.DatabaseEngine

func RunServer(host string, port int) {
	engine = core.NewDatabaseEngine()
	addr := fmt.Sprintf("%s:%d", host, port)

	http.HandleFunc("/", handleRequest)

	fmt.Printf("Serving mini SQL UI on http://%s\n", addr)
	if err := http.ListenAndServe(addr, nil); err != nil {
		fmt.Printf("Server error: %v\n", err)
	}
}

func handleRequest(w http.ResponseWriter, r *http.Request) {
	query := ""
	result := ""

	if r.Method == "POST" {
		r.ParseForm()
		if use := r.FormValue("use_database"); use != "" {
			results := engine.Execute(fmt.Sprintf("USE %s;", use))
			result = strings.Join(results, "\n")
		}
		query = r.FormValue("query")
		if query != "" {
			results := engine.Execute(query)
			result = strings.Join(results, "\n")
		}
	}

	if result == "" {
		result = "(no output)"
	}

	schema := generateSchemaHTML()
	activeDB := engine.ActiveDatabase()

	page := fmt.Sprintf(htmlTemplate,
		html.EscapeString(query),
		html.EscapeString(result),
		html.EscapeString(activeDB),
		databaseOptionsHTML(),
		schema,
		lsmLogHTML())

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	w.Write([]byte(page))
}

func generateSchemaHTML() string {
	snapshot := engine.Describe()
	if len(snapshot) == 0 {
		return "<p>No databases yet.</p>"
	}

	var lines []string
	lines = append(lines, "<ul>")

	var dbNames []string
	for name := range snapshot {
		dbNames = append(dbNames, name)
	}
	sort.Strings(dbNames)

	for _, dbName := range dbNames {
		tables := snapshot[dbName]
		lines = append(lines, fmt.Sprintf("<li><strong>%s</strong>", html.EscapeString(dbName)))

		if len(tables) > 0 {
			lines = append(lines, "<ul>")
			var tableNames []string
			for name := range tables {
				tableNames = append(tableNames, name)
			}
			sort.Strings(tableNames)

			for _, tableName := range tableNames {
				info := tables[tableName]
				columns := formatColumns(info["columns"])
				rowCount := info["row_count"]
				indexes := formatIndexes(info["indexes"])

				lines = append(lines, fmt.Sprintf(
					"<li><span class='table-name'>%s</span><br />"+
						"<small>cols: %s</small><br />"+
						"<small>rows: %v, idx: %s</small></li>",
					html.EscapeString(tableName),
					html.EscapeString(columns),
					rowCount,
					html.EscapeString(indexes)))
			}
			lines = append(lines, "</ul>")
		}
		lines = append(lines, "</li>")
	}
	lines = append(lines, "</ul>")

	return strings.Join(lines, "")
}

func databaseOptionsHTML() string {
	options := []string{}
	names := engine.Databases()
	if len(names) == 0 {
		return "<option value=\"\">(none)</option>"
	}
	active := engine.ActiveDatabase()
	for _, name := range names {
		selected := ""
		if name == active {
			selected = " selected"
		}
		options = append(options, fmt.Sprintf("<option value='%s'%s>%s</option>", html.EscapeString(name), selected, html.EscapeString(name)))
	}
	return strings.Join(options, "")
}

func lsmLogHTML() string {
	entries := engine.LSMEntries()
	if len(entries) == 0 {
		return "<p>No pending log entries.</p>"
	}
	lines := make([]string, 0, len(entries))
	for _, entry := range entries {
		payload, err := json.MarshalIndent(entry, "", "  ")
		if err != nil {
			payload = []byte("{}")
		}
		lines = append(lines, string(payload))
	}
	return "<pre>" + html.EscapeString(strings.Join(lines, "\n")) + "</pre>"
}

func formatColumns(cols interface{}) string {
	if cols == nil {
		return "(no columns)"
	}
	colSlice, ok := cols.([]string)
	if !ok {
		return "(no columns)"
	}
	if len(colSlice) == 0 {
		return "(no columns)"
	}
	return strings.Join(colSlice, ", ")
}

func formatIndexes(idxs interface{}) string {
	if idxs == nil {
		return "none"
	}
	idxSlice, ok := idxs.([]string)
	if !ok {
		return "none"
	}
	if len(idxSlice) == 0 {
		return "none"
	}
	return strings.Join(idxSlice, ", ")
}
