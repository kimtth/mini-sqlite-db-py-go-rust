/// Simplified in-memory B-Tree style table manager.
use crate::core::parser::{Condition, JoinInfo, Value};
use crate::core::storage::pager::Pager;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_vec};
use std::collections::HashMap;

type Row = HashMap<String, Value>;
type Index = HashMap<String, Vec<usize>>;

#[derive(Clone)]
pub struct TableMeta {
    pub columns: Vec<String>,
    pub rows: Vec<Row>,
    pub indexes: HashMap<String, Index>,
}

#[derive(Serialize, Deserialize)]
struct TableSnapshot {
    columns: Vec<String>,
    rows: Vec<Row>,
    indexes: Vec<String>,
}

pub struct BTreeStorage {
    pager: Option<Pager>,
    tables: HashMap<String, TableMeta>,
}

impl BTreeStorage {
    pub fn new(pager: Option<Pager>) -> Self {
        let mut storage = BTreeStorage {
            pager,
            tables: HashMap::new(),
        };
        storage.load();
        storage
    }

    pub fn create_table(&mut self, name: &str, columns: Vec<String>) {
        let meta = TableMeta {
            columns,
            rows: Vec::new(),
            indexes: HashMap::new(),
        };
        self.tables.insert(name.to_string(), meta);
        self.persist();
    }

    pub fn drop_table(&mut self, name: &str) {
        self.tables.remove(name);
        self.persist();
    }

    pub fn table_exists(&self, name: &str) -> bool {
        self.tables.contains_key(name)
    }

    pub fn add_column(&mut self, name: &str, column: String) {
        let indexes_to_rebuild = {
            let table = match self.tables.get_mut(name) {
                Some(table) => table,
                None => return,
            };
            if table.columns.contains(&column) {
                return;
            }
            table.columns.push(column.clone());
            for row in &mut table.rows {
                row.insert(column.clone(), Value::Null);
            }
            table.indexes.keys().cloned().collect::<Vec<String>>()
        };

        for col in indexes_to_rebuild {
            self.rebuild_index(name, &col);
        }
        self.persist();
    }

    pub fn create_index(&mut self, table_name: &str, column: &str) {
        if let Some(table) = self.tables.get_mut(table_name) {
            if !table.indexes.contains_key(column) {
                self.rebuild_index(table_name, column);
            }
        }
        self.persist();
    }

    pub fn drop_index(&mut self, table_name: &str, column: &str) {
        if let Some(table) = self.tables.get_mut(table_name) {
            table.indexes.remove(column);
            self.persist();
        }
    }

    pub fn insert_row(&mut self, table_name: &str, values: Vec<Value>) -> Result<Row, String> {
        let table = self
            .tables
            .get_mut(table_name)
            .ok_or_else(|| format!("Table '{}' not found", table_name))?;

        if values.len() != table.columns.len() {
            return Err("Value count does not match table schema".to_string());
        }

        let mut row = Row::new();
        for (col, val) in table.columns.iter().zip(values.iter()) {
            row.insert(col.clone(), val.clone());
        }

        let row_idx = table.rows.len();
        table.rows.push(row.clone());

        // Update indexes
        for (column, index) in &mut table.indexes {
            if let Some(value) = row.get(column) {
                let key = format!("{:?}", value);
                index.entry(key).or_insert_with(Vec::new).push(row_idx);
            }
        }

        self.persist();

        Ok(row)
    }

    pub fn select_rows(
        &self,
        table_name: &str,
        columns: &[String],
        condition: Option<&Condition>,
        join: Option<&JoinInfo>,
    ) -> Result<Vec<Row>, String> {
        if let Some(join_info) = join {
            return self.join_rows(table_name, columns, condition, join_info);
        }

        if !self.tables.contains_key(table_name) {
            return Err(format!("Table '{}' not found", table_name));
        }

        let rows: Vec<&Row> = self.rows_for(table_name, condition).collect();

        if columns.len() == 1 && columns[0] == "*" {
            return Ok(rows.iter().map(|r| (*r).clone()).collect());
        }

        let mut selected = Vec::new();
        for row in rows {
            let mut projected = Row::new();
            for col in columns {
                let lookup = col.split('.').last().unwrap_or(col);
                projected.insert(col.clone(), row.get(lookup).cloned().unwrap_or(Value::Null));
            }
            selected.push(projected);
        }

        Ok(selected)
    }

    pub fn update_rows(
        &mut self,
        table_name: &str,
        assignments: &HashMap<String, Value>,
        condition: Option<&Condition>,
    ) -> Result<usize, String> {
        if !self.tables.contains_key(table_name) {
            return Err(format!("Table '{}' not found", table_name));
        }

        let indexes_to_rebuild = self
            .tables
            .get(table_name)
            .unwrap()
            .indexes
            .keys()
            .cloned()
            .collect::<Vec<String>>();

        let indices = {
            let table = self.tables.get(table_name).unwrap();
            if let Some(cond) = condition {
                table
                    .rows
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, row)| match row.get(&cond.column) {
                        Some(value) if Self::values_equal(value, &cond.value) => Some(idx),
                        _ => None,
                    })
                    .collect::<Vec<usize>>()
            } else {
                (0..table.rows.len()).collect::<Vec<usize>>()
            }
        };

        {
            let table = self.tables.get_mut(table_name).unwrap();
            for idx in &indices {
                if let Some(row) = table.rows.get_mut(*idx) {
                    for (key, value) in assignments {
                        row.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        for column in indexes_to_rebuild {
            self.rebuild_index(table_name, &column);
        }

        if !indices.is_empty() {
            self.persist();
        }

        Ok(indices.len())
    }

    pub fn delete_rows(
        &mut self,
        table_name: &str,
        condition: Option<&Condition>,
    ) -> Result<usize, String> {
        let (kept, deleted, index_columns) = {
            let table = self
                .tables
                .get(table_name)
                .ok_or_else(|| format!("Table '{}' not found", table_name))?;
            let mut kept_rows = Vec::new();
            let mut deleted_count = 0;
            if let Some(cond) = condition {
                for row in &table.rows {
                    match row.get(&cond.column) {
                        Some(value) if Self::values_equal(value, &cond.value) => deleted_count += 1,
                        _ => kept_rows.push(row.clone()),
                    }
                }
            } else {
                deleted_count = table.rows.len();
            }
            let columns = table.indexes.keys().cloned().collect::<Vec<String>>();
            (kept_rows, deleted_count, columns)
        };

        let table = self.tables.get_mut(table_name).unwrap();
        table.rows = kept;

        for column in index_columns {
            self.rebuild_index(table_name, &column);
        }

        if deleted > 0 {
            self.persist();
        }

        Ok(deleted)
    }

    fn rows_for<'a>(
        &'a self,
        table_name: &str,
        condition: Option<&'a Condition>,
    ) -> Box<dyn Iterator<Item = &'a Row> + 'a> {
        let table = match self.tables.get(table_name) {
            Some(t) => t,
            None => return Box::new(std::iter::empty()),
        };

        if let Some(cond) = condition {
            let column = cond.column.clone();
            let value = cond.value.clone();

            if let Some(index) = table.indexes.get(&column) {
                let key = format!("{:?}", value);
                if let Some(indices) = index.get(&key) {
                    let rows: Vec<&Row> = indices
                        .iter()
                        .filter_map(|idx| table.rows.get(*idx))
                        .collect();
                    return Box::new(rows.into_iter());
                }
                return Box::new(std::iter::empty());
            }

            Box::new(table.rows.iter().filter(move |row| {
                if let Some(row_value) = row.get(&column) {
                    Self::values_equal(row_value, &value)
                } else {
                    false
                }
            }))
        } else {
            Box::new(table.rows.iter())
        }
    }

    fn join_rows(
        &self,
        left_table_name: &str,
        columns: &[String],
        condition: Option<&Condition>,
        join: &JoinInfo,
    ) -> Result<Vec<Row>, String> {
        let left_table = self
            .tables
            .get(left_table_name)
            .ok_or_else(|| format!("Table '{}' not found", left_table_name))?;
        let right_table = self
            .tables
            .get(&join.table)
            .ok_or_else(|| format!("Table '{}' not found", join.table))?;

        // Ensure index exists on right table
        if !right_table.indexes.contains_key(&join.right_column) {
            // We need mutable access, so we'll do this differently
            // For now, we'll scan without index
        }

        let left_rows: Vec<&Row> = self.rows_for(left_table_name, condition).collect();
        let mut result = Vec::new();

        for left_row in left_rows {
            if let Some(key_value) = left_row.get(&join.left_column) {
                for right_row in &right_table.rows {
                    if let Some(right_value) = right_row.get(&join.right_column) {
                        if Self::values_equal(key_value, right_value) {
                            let mut combined = Row::new();

                            // Add left table columns with prefix
                            for col in &left_table.columns {
                                let key = format!("{}.{}", join.left_table, col);
                                combined
                                    .insert(key, left_row.get(col).cloned().unwrap_or(Value::Null));
                            }

                            // Add right table columns with prefix
                            for col in &right_table.columns {
                                let key = format!("{}.{}", join.right_table, col);
                                combined.insert(
                                    key,
                                    right_row.get(col).cloned().unwrap_or(Value::Null),
                                );
                            }

                            // Project requested columns if not *
                            if columns.len() == 1 && columns[0] == "*" {
                                result.push(combined);
                            } else {
                                let mut projected = Row::new();
                                for col in columns {
                                    if col.contains('.') {
                                        projected.insert(
                                            col.clone(),
                                            combined.get(col).cloned().unwrap_or(Value::Null),
                                        );
                                    } else if left_table.columns.contains(col) {
                                        projected.insert(
                                            col.clone(),
                                            left_row.get(col).cloned().unwrap_or(Value::Null),
                                        );
                                    } else if right_table.columns.contains(col) {
                                        projected.insert(
                                            col.clone(),
                                            right_row.get(col).cloned().unwrap_or(Value::Null),
                                        );
                                    } else {
                                        projected.insert(col.clone(), Value::Null);
                                    }
                                }
                                result.push(projected);
                            }
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    fn rebuild_index(&mut self, table_name: &str, column: &str) {
        if let Some(table) = self.tables.get_mut(table_name) {
            let mut index = Index::new();

            for (idx, row) in table.rows.iter().enumerate() {
                if let Some(value) = row.get(column) {
                    let key = format!("{:?}", value);
                    index.entry(key).or_insert_with(Vec::new).push(idx);
                }
            }

            table.indexes.insert(column.to_string(), index);
        }
    }

    fn values_equal(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Integer(x), Value::Integer(y)) => x == y,
            (Value::Float(x), Value::Float(y)) => (x - y).abs() < f64::EPSILON,
            (Value::Text(x), Value::Text(y)) => x == y,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }

    pub fn describe(&self) -> HashMap<String, HashMap<String, serde_json::Value>> {
        let mut summary = HashMap::new();

        for (name, table) in &self.tables {
            let mut table_info = HashMap::new();
            table_info.insert(
                "columns".to_string(),
                serde_json::to_value(&table.columns).unwrap(),
            );
            table_info.insert(
                "row_count".to_string(),
                serde_json::to_value(table.rows.len()).unwrap(),
            );
            let index_keys: Vec<String> = table.indexes.keys().cloned().collect();
            table_info.insert(
                "indexes".to_string(),
                serde_json::to_value(index_keys).unwrap(),
            );
            summary.insert(name.clone(), table_info);
        }

        summary
    }

    fn load(&mut self) {
        let data = match self.pager.as_ref().map(|pager| pager.read_blob()) {
            Some(bytes) if !bytes.is_empty() => bytes,
            _ => return,
        };
        if let Ok(snapshot) = from_slice::<HashMap<String, TableSnapshot>>(&data) {
            for (name, table) in snapshot {
                let meta = TableMeta {
                    columns: table.columns.clone(),
                    rows: table.rows.clone(),
                    indexes: HashMap::new(),
                };
                self.tables.insert(name.clone(), meta);
                for column in table.indexes {
                    self.rebuild_index(&name, &column);
                }
            }
        }
    }

    fn persist(&mut self) {
        if let Some(ref mut pager) = self.pager {
            let mut snapshot = HashMap::new();
            for (name, table) in &self.tables {
                let indexes: Vec<String> = table.indexes.keys().cloned().collect();
                let table_snapshot = TableSnapshot {
                    columns: table.columns.clone(),
                    rows: table.rows.clone(),
                    indexes,
                };
                snapshot.insert(name.clone(), table_snapshot);
            }
            if let Ok(bytes) = to_vec(&snapshot) {
                pager.write_blob(&bytes);
            }
        }
    }
}
