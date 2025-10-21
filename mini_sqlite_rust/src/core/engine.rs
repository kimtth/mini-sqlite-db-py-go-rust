use crate::core::executor::SQLExecutor;
/// High level database engine wiring together parser, executor, and storage.
use crate::core::parser::SQLParser;
use crate::core::storage::lsm_tree::LogEntry;
use std::collections::HashMap;

pub struct DatabaseEngine {
    parser: SQLParser,
    executor: SQLExecutor,
}

impl DatabaseEngine {
    pub fn new() -> Self {
        DatabaseEngine {
            parser: SQLParser::new(),
            executor: SQLExecutor::new(),
        }
    }

    /// Parse and execute a SQL query string.
    pub fn execute(&mut self, query: &str) -> Vec<String> {
        let parsed = self.parser.parse(query);
        self.executor.execute(&parsed)
    }

    /// Return a snapshot of databases, tables, and columns.
    pub fn describe(&self) -> HashMap<String, HashMap<String, serde_json::Value>> {
        self.executor.describe()
    }

    /// Expose the name of the currently active database.
    pub fn active_database(&self) -> &str {
        self.executor.active_database()
    }

    pub fn databases(&self) -> Vec<String> {
        self.executor.databases()
    }

    pub fn lsm_entries(&self) -> Vec<LogEntry> {
        self.executor.lsm_entries()
    }
}

impl Default for DatabaseEngine {
    fn default() -> Self {
        Self::new()
    }
}
