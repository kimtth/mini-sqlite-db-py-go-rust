package core

import "mini_sqlite/core/storage"

type DatabaseEngine struct {
	parser   *SQLParser
	executor *SQLExecutor
}

func NewDatabaseEngine() *DatabaseEngine {
	return &DatabaseEngine{
		parser:   NewSQLParser(),
		executor: NewSQLExecutor(),
	}
}

func (e *DatabaseEngine) Execute(query string) []string {
	parsed := e.parser.Parse(query)
	return e.executor.Execute(parsed)
}

func (e *DatabaseEngine) Describe() map[string]map[string]map[string]interface{} {
	return e.executor.Describe()
}

func (e *DatabaseEngine) ActiveDatabase() string {
	return e.executor.ActiveDatabase()
}

func (e *DatabaseEngine) Databases() []string {
	return e.executor.Databases()
}

func (e *DatabaseEngine) LSMEntries() []storage.LogEntry {
	return e.executor.LSMEntries()
}
