package core

import (
	"fmt"
	"mini_sqlite/core/storage"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

type SQLExecutor struct {
	lsm       *storage.LSMTreeStorage
	databases map[string]*storage.BTreeStorage
	activeDB  string
	dataDir   string
}

func NewSQLExecutor() *SQLExecutor {
	dataDir := filepath.Join("mini_sqlite_golang", "data")
	_ = os.MkdirAll(dataDir, 0o755)
	exec := &SQLExecutor{
		lsm:       storage.NewLSMTreeStorage(),
		databases: make(map[string]*storage.BTreeStorage),
		activeDB:  "default",
		dataDir:   dataDir,
	}
	exec.loadDatabases()
	if len(exec.databases) == 0 {
		exec.ensureDatabase("default")
	}
	if _, ok := exec.databases[exec.activeDB]; !ok {
		for name := range exec.databases {
			exec.activeDB = name
			break
		}
	}
	return exec
}

func (e *SQLExecutor) Execute(parsed ParsedCommand) []string {
	cmd, details := parsed.Command, parsed.Details

	if cmd == "EMPTY" {
		return []string{""}
	}

	if cmd == "CREATE_DATABASE" {
		name := details["name"].(string)
		e.ensureDatabase(name)
		e.activeDB = name
		return []string{fmt.Sprintf("Database '%s' ready.", name)}
	}

	if cmd == "ALTER_DATABASE" {
		name := details["name"].(string)
		e.ensureDatabase(name)
		e.activeDB = name
		return []string{fmt.Sprintf("Using database '%s'.", name)}
	}

	if cmd == "USE_DATABASE" {
		name := details["name"].(string)
		if _, ok := e.databases[name]; !ok {
			return []string{fmt.Sprintf("Database '%s' not found.", name)}
		}
		e.activeDB = name
		return []string{fmt.Sprintf("Using database '%s'.", name)}
	}

	store := e.databases[e.activeDB]
	table := ""
	if t, ok := details["table"].(string); ok {
		table = t
	}

	if cmd == "CREATE_TABLE" {
		if store.TableExists(table) {
			return []string{fmt.Sprintf("Table '%s' already exists.", table)}
		}
		columns := extractColumns(details["columns"])
		store.CreateTable(table, columns)
		return []string{fmt.Sprintf("Table '%s' created.", table)}
	}

	if table != "" && !store.TableExists(table) {
		return []string{fmt.Sprintf("Table '%s' not found.", table)}
	}

	switch cmd {
	case "ALTER_TABLE":
		column := details["column"].(map[string]string)["name"]
		store.AddColumn(table, column)
		return []string{fmt.Sprintf("Column '%s' added to '%s'.", column, table)}

	case "DROP_TABLE":
		store.DropTable(table)
		return []string{fmt.Sprintf("Table '%s' dropped.", table)}

	case "CREATE_INDEX":
		column := details["column"].(string)
		store.CreateIndex(table, column)
		return []string{fmt.Sprintf("Index on %s.%s built.", table, column)}

	case "DROP_INDEX":
		column := details["column"].(string)
		store.DropIndex(table, column)
		return []string{fmt.Sprintf("Index on %s.%s removed.", table, column)}

	case "INSERT":
		values := details["values"].([]interface{})
		row := store.InsertRow(table, values)
		e.lsm.Log(storage.LogEntry{"db": e.activeDB, "command": "INSERT", "row": row})
		return []string{"1 row inserted."}

	case "UPDATE":
		assignments := details["assignments"].(map[string]interface{})
		where, _ := details["where"].(map[string]interface{})
		count := store.UpdateRows(table, assignments, where)
		e.lsm.Log(storage.LogEntry{"db": e.activeDB, "command": "UPDATE", "count": count})
		return []string{fmt.Sprintf("%d row(s) updated.", count)}

	case "DELETE":
		where, _ := details["where"].(map[string]interface{})
		count := store.DeleteRows(table, where)
		e.lsm.Log(storage.LogEntry{"db": e.activeDB, "command": "DELETE", "count": count})
		return []string{fmt.Sprintf("%d row(s) deleted.", count)}

	case "SELECT":
		join, _ := details["join"].(map[string]interface{})
		if join != nil {
			joinTable := join["table"].(string)
			if !store.TableExists(joinTable) {
				return []string{fmt.Sprintf("Table '%s' not found.", joinTable)}
			}
		}
		where, _ := details["where"].(map[string]interface{})
		columns := details["columns"].([]interface{})
		colStrs := make([]string, len(columns))
		for i, col := range columns {
			colStrs[i] = col.(string)
		}
		rows := store.SelectRows(table, colStrs, where, join)
		return e.formatRows(rows, colStrs)

	case "COMMIT":
		entries := e.lsm.Commit()
		word := "entries"
		if len(entries) == 1 {
			word = "entry"
		}
		return []string{fmt.Sprintf("Committed %d %s.", len(entries), word)}
	}

	return []string{fmt.Sprintf("Command '%s' not understood.", parsed.Raw)}
}

func (e *SQLExecutor) ensureDatabase(name string) {
	if _, ok := e.databases[name]; !ok {
		path := filepath.Join(e.dataDir, name+".dat")
		e.databases[name] = storage.NewBTreeStorage(storage.NewPager(path, 4096))
	}
}

func (e *SQLExecutor) formatRows(rows []storage.Row, requested []string) []string {
	if len(rows) == 0 {
		return []string{"(no rows)"}
	}

	var headers []string
	if len(requested) == 1 && requested[0] == "*" {
		for k := range rows[0] {
			headers = append(headers, k)
		}
	} else {
		headers = requested
	}

	lines := []string{strings.Join(headers, " | ")}
	for _, row := range rows {
		var values []string
		for _, col := range headers {
			val := row[col]
			values = append(values, fmt.Sprintf("%v", val))
		}
		lines = append(lines, strings.Join(values, " | "))
	}
	return lines
}

func (e *SQLExecutor) Describe() map[string]map[string]map[string]interface{} {
	result := make(map[string]map[string]map[string]interface{})
	for name, store := range e.databases {
		result[name] = store.Describe()
	}
	return result
}

func (e *SQLExecutor) ActiveDatabase() string {
	return e.activeDB
}

func (e *SQLExecutor) Databases() []string {
	names := make([]string, 0, len(e.databases))
	for name := range e.databases {
		names = append(names, name)
	}
	sort.Strings(names)
	return names
}

func (e *SQLExecutor) LSMEntries() []storage.LogEntry {
	return e.lsm.Snapshot()
}

func (e *SQLExecutor) loadDatabases() {
	entries, err := os.ReadDir(e.dataDir)
	if err != nil {
		return
	}
	for _, entry := range entries {
		if entry.IsDir() {
			continue
		}
		if filepath.Ext(entry.Name()) != ".dat" {
			continue
		}
		name := strings.TrimSuffix(strings.ToLower(entry.Name()), filepath.Ext(entry.Name()))
		path := filepath.Join(e.dataDir, entry.Name())
		e.databases[name] = storage.NewBTreeStorage(storage.NewPager(path, 4096))
	}
}

func extractColumns(cols interface{}) []string {
	colMaps := cols.([]interface{})
	result := make([]string, len(colMaps))
	for i, c := range colMaps {
		colMap := c.(map[string]string)
		result[i] = colMap["name"]
	}
	return result
}
