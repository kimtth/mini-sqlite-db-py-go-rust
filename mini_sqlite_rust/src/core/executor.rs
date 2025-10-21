/// SQL executor orchestrating DDL, DML, and simple commits.
use crate::core::parser::{CommandType, ParsedCommand, Value};
use crate::core::storage::btree::BTreeStorage;
use crate::core::storage::lsm_tree::{LSMTreeStorage, LogEntry};
use crate::core::storage::pager::Pager;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct SQLExecutor {
    lsm: LSMTreeStorage,
    databases: HashMap<String, BTreeStorage>,
    active_db: String,
    data_dir: PathBuf,
}

impl SQLExecutor {
    pub fn new() -> Self {
        let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
        let _ = fs::create_dir_all(&data_dir);
        let mut executor = SQLExecutor {
            lsm: LSMTreeStorage::new(),
            databases: HashMap::new(),
            active_db: "default".to_string(),
            data_dir,
        };
        executor.load_databases();
        if executor.databases.is_empty() {
            executor.ensure_database("default");
        }
        if !executor.databases.contains_key(&executor.active_db) {
            if let Some(first) = executor.databases.keys().next().cloned() {
                executor.active_db = first;
            }
        }
        executor
    }

    pub fn execute(&mut self, parsed: &ParsedCommand) -> Vec<String> {
        match &parsed.command {
            CommandType::Empty => vec![String::new()],

            CommandType::CreateDatabase { name } => {
                self.ensure_database(name);
                self.active_db = name.clone();
                vec![format!("Database '{}' ready.", name)]
            }

            CommandType::AlterDatabase { name } => {
                self.ensure_database(name);
                self.active_db = name.clone();
                vec![format!("Using database '{}'.", name)]
            }

            CommandType::UseDatabase { name } => {
                if self.databases.contains_key(name) {
                    self.active_db = name.clone();
                    vec![format!("Using database '{}'.", name)]
                } else {
                    vec![format!("Database '{}' not found.", name)]
                }
            }

            CommandType::CreateTable { table, columns } => {
                let storage = self.databases.get_mut(&self.active_db).unwrap();
                if storage.table_exists(table) {
                    return vec![format!("Table '{}' already exists.", table)];
                }
                let column_names: Vec<String> = columns.iter().map(|c| c.name.clone()).collect();
                storage.create_table(table, column_names);
                vec![format!("Table '{}' created.", table)]
            }

            CommandType::AlterTable { table, column } => {
                let storage = self.databases.get_mut(&self.active_db).unwrap();
                if !storage.table_exists(table) {
                    return vec![format!("Table '{}' not found.", table)];
                }
                storage.add_column(table, column.name.clone());
                vec![format!("Column '{}' added to '{}'.", column.name, table)]
            }

            CommandType::DropTable { table } => {
                let storage = self.databases.get_mut(&self.active_db).unwrap();
                if !storage.table_exists(table) {
                    return vec![format!("Table '{}' not found.", table)];
                }
                storage.drop_table(table);
                vec![format!("Table '{}' dropped.", table)]
            }

            CommandType::CreateIndex { table, column } => {
                let storage = self.databases.get_mut(&self.active_db).unwrap();
                if !storage.table_exists(table) {
                    return vec![format!("Table '{}' not found.", table)];
                }
                storage.create_index(table, column);
                vec![format!("Index on {}.{} built.", table, column)]
            }

            CommandType::DropIndex { table, column } => {
                let storage = self.databases.get_mut(&self.active_db).unwrap();
                if !storage.table_exists(table) {
                    return vec![format!("Table '{}' not found.", table)];
                }
                storage.drop_index(table, column);
                vec![format!("Index on {}.{} removed.", table, column)]
            }

            CommandType::Insert { table, values } => {
                let storage = self.databases.get_mut(&self.active_db).unwrap();
                if !storage.table_exists(table) {
                    return vec![format!("Table '{}' not found.", table)];
                }
                match storage.insert_row(table, values.clone()) {
                    Ok(_row) => {
                        let mut details = HashMap::new();
                        details.insert("table".to_string(), serde_json::to_value(table).unwrap());
                        self.lsm.log(LogEntry {
                            db: self.active_db.clone(),
                            command: "INSERT".to_string(),
                            details,
                        });
                        vec!["1 row inserted.".to_string()]
                    }
                    Err(e) => vec![format!("Error: {}", e)],
                }
            }

            CommandType::Update {
                table,
                assignments,
                condition,
            } => {
                let storage = self.databases.get_mut(&self.active_db).unwrap();
                if !storage.table_exists(table) {
                    return vec![format!("Table '{}' not found.", table)];
                }
                match storage.update_rows(table, assignments, condition.as_ref()) {
                    Ok(count) => {
                        let mut details = HashMap::new();
                        details.insert("count".to_string(), serde_json::to_value(count).unwrap());
                        self.lsm.log(LogEntry {
                            db: self.active_db.clone(),
                            command: "UPDATE".to_string(),
                            details,
                        });
                        vec![format!("{} row(s) updated.", count)]
                    }
                    Err(e) => vec![format!("Error: {}", e)],
                }
            }

            CommandType::Delete { table, condition } => {
                let storage = self.databases.get_mut(&self.active_db).unwrap();
                if !storage.table_exists(table) {
                    return vec![format!("Table '{}' not found.", table)];
                }
                match storage.delete_rows(table, condition.as_ref()) {
                    Ok(count) => {
                        let mut details = HashMap::new();
                        details.insert("count".to_string(), serde_json::to_value(count).unwrap());
                        self.lsm.log(LogEntry {
                            db: self.active_db.clone(),
                            command: "DELETE".to_string(),
                            details,
                        });
                        vec![format!("{} row(s) deleted.", count)]
                    }
                    Err(e) => vec![format!("Error: {}", e)],
                }
            }

            CommandType::Select {
                table,
                columns,
                condition,
                join,
            } => {
                let storage = self.databases.get(&self.active_db).unwrap();
                if !storage.table_exists(table) {
                    return vec![format!("Table '{}' not found.", table)];
                }
                if let Some(join_info) = join {
                    if !storage.table_exists(&join_info.table) {
                        return vec![format!("Table '{}' not found.", join_info.table)];
                    }
                }
                match storage.select_rows(table, columns, condition.as_ref(), join.as_ref()) {
                    Ok(rows) => self.format_rows(&rows, columns),
                    Err(e) => vec![format!("Error: {}", e)],
                }
            }

            CommandType::Commit => {
                let entries = self.lsm.commit();
                let count = entries.len();
                let entry_word = if count == 1 { "entry" } else { "entries" };
                vec![format!("Committed {} {}.", count, entry_word)]
            }

            CommandType::Unknown => {
                vec![format!("Command '{}' not understood.", parsed.raw)]
            }
        }
    }

    fn ensure_database(&mut self, name: &str) {
        if !self.databases.contains_key(name) {
            let path = self.data_dir.join(format!("{}.dat", name));
            let pager = Pager::new(path, 4096);
            self.databases
                .insert(name.to_string(), BTreeStorage::new(Some(pager)));
        }
    }

    fn format_rows(&self, rows: &[HashMap<String, Value>], requested: &[String]) -> Vec<String> {
        if rows.is_empty() {
            return vec!["(no rows)".to_string()];
        }

        let headers = if requested.len() == 1 && requested[0] == "*" {
            let mut keys: Vec<String> = rows[0].keys().cloned().collect();
            keys.sort();
            keys
        } else {
            requested.to_vec()
        };

        let mut lines = Vec::new();
        lines.push(headers.join(" | "));

        for row in rows {
            let values: Vec<String> = headers
                .iter()
                .map(|col| {
                    row.get(col)
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| String::new())
                })
                .collect();
            lines.push(values.join(" | "));
        }

        lines
    }

    pub fn describe(&self) -> HashMap<String, HashMap<String, serde_json::Value>> {
        let mut result = HashMap::new();
        for (db_name, storage) in &self.databases {
            let mut tables = HashMap::new();
            for (table_name, info) in storage.describe() {
                tables.insert(table_name, serde_json::to_value(info).unwrap());
            }
            result.insert(db_name.clone(), tables);
        }
        result
    }

    pub fn active_database(&self) -> &str {
        &self.active_db
    }

    pub fn databases(&self) -> Vec<String> {
        let mut names: Vec<String> = self.databases.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn lsm_entries(&self) -> Vec<LogEntry> {
        self.lsm.snapshot()
    }

    fn load_databases(&mut self) {
        if let Ok(entries) = fs::read_dir(&self.data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("dat") {
                    continue;
                }
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let pager = Pager::new(path.clone(), 4096);
                    self.databases
                        .insert(stem.to_lowercase(), BTreeStorage::new(Some(pager)));
                }
            }
        }
    }
}

impl Default for SQLExecutor {
    fn default() -> Self {
        Self::new()
    }
}
