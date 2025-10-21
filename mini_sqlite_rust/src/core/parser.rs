/// Lightweight SQL parser returning structured command dictionaries.
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Text(String),
    Null,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Text(s) => write!(f, "{}", s),
            Value::Null => write!(f, "NULL"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub column: String,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinInfo {
    pub table: String,
    pub left_table: String,
    pub left_column: String,
    pub right_table: String,
    pub right_column: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub col_type: String,
}

#[derive(Debug, Clone)]
pub enum CommandType {
    Empty,
    CreateDatabase {
        name: String,
    },
    AlterDatabase {
        name: String,
    },
    UseDatabase {
        name: String,
    },
    CreateTable {
        table: String,
        columns: Vec<ColumnDef>,
    },
    AlterTable {
        table: String,
        column: ColumnDef,
    },
    DropTable {
        table: String,
    },
    CreateIndex {
        table: String,
        column: String,
    },
    DropIndex {
        table: String,
        column: String,
    },
    Insert {
        table: String,
        values: Vec<Value>,
    },
    Update {
        table: String,
        assignments: HashMap<String, Value>,
        condition: Option<Condition>,
    },
    Delete {
        table: String,
        condition: Option<Condition>,
    },
    Select {
        table: String,
        columns: Vec<String>,
        condition: Option<Condition>,
        join: Option<JoinInfo>,
    },
    Commit,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ParsedCommand {
    pub command: CommandType,
    pub raw: String,
}

pub struct SQLParser;

impl SQLParser {
    pub fn new() -> Self {
        SQLParser
    }

    pub fn parse(&self, query: &str) -> ParsedCommand {
        let raw = query.trim();
        if raw.is_empty() {
            return ParsedCommand {
                command: CommandType::Empty,
                raw: String::new(),
            };
        }

        let text = raw.trim_end_matches(';');
        let tokens: Vec<&str> = text.split_whitespace().collect();

        if tokens.is_empty() {
            return ParsedCommand {
                command: CommandType::Unknown,
                raw: raw.to_string(),
            };
        }

        let command_str = tokens[0].to_uppercase();

        let command = match command_str.as_str() {
            "COMMIT" => CommandType::Commit,
            "CREATE" if tokens.len() > 1 && tokens[1].to_uppercase() == "DATABASE" => {
                if tokens.len() > 2 {
                    CommandType::CreateDatabase {
                        name: tokens[2].to_lowercase(),
                    }
                } else {
                    CommandType::Unknown
                }
            }
            "ALTER" if tokens.len() > 1 && tokens[1].to_uppercase() == "DATABASE" => {
                if tokens.len() > 2 {
                    CommandType::AlterDatabase {
                        name: tokens[2].to_lowercase(),
                    }
                } else {
                    CommandType::Unknown
                }
            }
            "USE" if tokens.len() > 1 => CommandType::UseDatabase {
                name: tokens[1].to_lowercase(),
            },
            "CREATE" if tokens.len() > 1 && tokens[1].to_uppercase() == "TABLE" => {
                self.parse_create_table(text)
            }
            "ALTER" if tokens.len() > 1 && tokens[1].to_uppercase() == "TABLE" => {
                self.parse_alter_table(text)
            }
            "DROP" if tokens.len() > 2 && tokens[1].to_uppercase() == "TABLE" => {
                CommandType::DropTable {
                    table: tokens[2].to_lowercase(),
                }
            }
            "CREATE" if tokens.len() > 3 && tokens[1].to_uppercase() == "INDEX" => {
                CommandType::CreateIndex {
                    table: tokens[2].to_lowercase(),
                    column: tokens[3].to_lowercase(),
                }
            }
            "DROP" if tokens.len() > 3 && tokens[1].to_uppercase() == "INDEX" => {
                CommandType::DropIndex {
                    table: tokens[2].to_lowercase(),
                    column: tokens[3].to_lowercase(),
                }
            }
            "INSERT" => self.parse_insert(text),
            "UPDATE" => self.parse_update(text),
            "DELETE" => self.parse_delete(text),
            "SELECT" => self.parse_select(text),
            _ => CommandType::Unknown,
        };

        ParsedCommand {
            command,
            raw: raw.to_string(),
        }
    }

    fn parse_create_table(&self, text: &str) -> CommandType {
        if let Some(paren_start) = text.find('(') {
            if let Some(paren_end) = text.rfind(')') {
                let header = &text[..paren_start];
                let tokens: Vec<&str> = header.split_whitespace().collect();
                if tokens.len() < 3 {
                    return CommandType::Unknown;
                }
                let table = tokens[2].to_lowercase();

                let inner = &text[paren_start + 1..paren_end];
                let mut columns = Vec::new();

                for chunk in inner.split(',') {
                    let parts: Vec<&str> = chunk.trim().split_whitespace().collect();
                    if parts.is_empty() {
                        continue;
                    }
                    let name = parts[0].to_lowercase();
                    let col_type = if parts.len() > 1 {
                        parts[1].to_uppercase()
                    } else {
                        "TEXT".to_string()
                    };
                    columns.push(ColumnDef { name, col_type });
                }

                return CommandType::CreateTable { table, columns };
            }
        }
        CommandType::Unknown
    }

    fn parse_alter_table(&self, text: &str) -> CommandType {
        let tokens: Vec<&str> = text.split_whitespace().collect();
        if tokens.len() >= 7
            && tokens[3].to_uppercase() == "ADD"
            && tokens[4].to_uppercase() == "COLUMN"
        {
            let table = tokens[2].to_lowercase();
            let name = tokens[5].to_lowercase();
            let col_type = if tokens.len() > 6 {
                tokens[6].to_uppercase()
            } else {
                "TEXT".to_string()
            };
            return CommandType::AlterTable {
                table,
                column: ColumnDef { name, col_type },
            };
        }
        CommandType::Unknown
    }

    fn parse_insert(&self, text: &str) -> CommandType {
        let re = Regex::new(r"(?i)INSERT\s+INTO\s+(\w+)\s+VALUES\s*\((.+)\)").unwrap();
        if let Some(caps) = re.captures(text) {
            let table = caps.get(1).unwrap().as_str().to_lowercase();
            let values_str = caps.get(2).unwrap().as_str();
            let values = self.parse_value_list(values_str);
            return CommandType::Insert { table, values };
        }
        CommandType::Unknown
    }

    fn parse_update(&self, text: &str) -> CommandType {
        let upper = text.to_uppercase();
        let (prefix, where_part) = if let Some(idx) = upper.find(" WHERE ") {
            let prefix = &text[..idx];
            let where_part = &text[idx + 7..];
            (prefix, Some(where_part))
        } else {
            (text, None)
        };

        let tokens: Vec<&str> = prefix.split_whitespace().collect();
        if tokens.len() < 2 {
            return CommandType::Unknown;
        }

        let table = tokens[1].to_lowercase();

        if let Some(set_idx) = prefix.to_uppercase().find(" SET ") {
            let set_clause = &prefix[set_idx + 5..];
            let mut assignments = HashMap::new();

            for chunk in set_clause.split(',') {
                if let Some(eq_idx) = chunk.find('=') {
                    let column = chunk[..eq_idx].trim().to_lowercase();
                    let value = self.parse_literal(chunk[eq_idx + 1..].trim());
                    assignments.insert(column, value);
                }
            }

            let condition = where_part.and_then(|w| self.parse_condition(w));

            return CommandType::Update {
                table,
                assignments,
                condition,
            };
        }
        CommandType::Unknown
    }

    fn parse_delete(&self, text: &str) -> CommandType {
        let upper = text.to_uppercase();
        let (prefix, where_part) = if let Some(idx) = upper.find(" WHERE ") {
            let prefix = &text[..idx];
            let where_part = &text[idx + 7..];
            (prefix, Some(where_part))
        } else {
            (text, None)
        };

        let tokens: Vec<&str> = prefix.split_whitespace().collect();
        if tokens.len() < 3 {
            return CommandType::Unknown;
        }

        let table = tokens[2].to_lowercase();
        let condition = where_part.and_then(|w| self.parse_condition(w));

        CommandType::Delete { table, condition }
    }

    fn parse_select(&self, text: &str) -> CommandType {
        let re = Regex::new(
            r"(?i)SELECT\s+(?P<cols>.+?)\s+FROM\s+(?P<table>\w+)(?:\s+INNER\s+JOIN\s+(?P<join_table>\w+)\s+ON\s+(?P<left_table>\w+)\.(?P<left_col>\w+)\s*=\s*(?P<right_table>\w+)\.(?P<right_col>\w+))?(?:\s+WHERE\s+(?P<where_col>\w+)\s*=\s*(?P<where_val>.+))?"
        ).unwrap();

        if let Some(caps) = re.captures(text) {
            let cols_str = caps.name("cols").unwrap().as_str();
            let columns: Vec<String> = cols_str.split(',').map(|s| s.trim().to_string()).collect();
            let table = caps.name("table").unwrap().as_str().to_lowercase();

            let condition = if let Some(where_col) = caps.name("where_col") {
                let column = where_col.as_str().to_lowercase();
                let value = self.parse_literal(caps.name("where_val").unwrap().as_str());
                Some(Condition { column, value })
            } else {
                None
            };

            let join = if let Some(join_table) = caps.name("join_table") {
                Some(JoinInfo {
                    table: join_table.as_str().to_lowercase(),
                    left_table: caps.name("left_table").unwrap().as_str().to_lowercase(),
                    left_column: caps.name("left_col").unwrap().as_str().to_lowercase(),
                    right_table: caps.name("right_table").unwrap().as_str().to_lowercase(),
                    right_column: caps.name("right_col").unwrap().as_str().to_lowercase(),
                })
            } else {
                None
            };

            return CommandType::Select {
                table,
                columns,
                condition,
                join,
            };
        }
        CommandType::Unknown
    }

    fn parse_value_list(&self, segment: &str) -> Vec<Value> {
        let mut values = Vec::new();
        let mut current = String::new();
        let mut in_string = false;

        for ch in segment.chars() {
            match ch {
                '\'' => {
                    if in_string {
                        values.push(Value::Text(current.clone()));
                        current.clear();
                        in_string = false;
                    } else {
                        in_string = true;
                    }
                }
                ',' if !in_string => {
                    if !current.trim().is_empty() {
                        values.push(self.parse_literal(current.trim()));
                        current.clear();
                    }
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        if !current.trim().is_empty() {
            if in_string {
                values.push(Value::Text(current));
            } else {
                values.push(self.parse_literal(current.trim()));
            }
        }

        values
    }

    fn parse_condition(&self, clause: &str) -> Option<Condition> {
        if let Some(eq_idx) = clause.find('=') {
            let column = clause[..eq_idx].trim().to_lowercase();
            let value = self.parse_literal(clause[eq_idx + 1..].trim());
            return Some(Condition { column, value });
        }
        None
    }

    fn parse_literal(&self, text: &str) -> Value {
        let trimmed = text.trim();

        // Remove quotes if present
        if (trimmed.starts_with('\'') && trimmed.ends_with('\''))
            || (trimmed.starts_with('"') && trimmed.ends_with('"'))
        {
            return Value::Text(trimmed[1..trimmed.len() - 1].to_string());
        }

        // Try parsing as integer
        if let Ok(i) = trimmed.parse::<i64>() {
            return Value::Integer(i);
        }

        // Try parsing as float
        if let Ok(f) = trimmed.parse::<f64>() {
            return Value::Float(f);
        }

        // Default to text
        Value::Text(trimmed.to_string())
    }
}

impl Default for SQLParser {
    fn default() -> Self {
        Self::new()
    }
}
