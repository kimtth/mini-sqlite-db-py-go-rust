# 🔄 Language Comparison Guide: Python → Rust & Go

> 📚 **For Python Developers**: Understanding Rust and Go concepts through real implementation examples

This guide explains Rust and Go language concepts by comparing them with Python implementations from the mini_sqlite project. Perfect for Python developers learning systems programming languages.

## 📋 Table of Contents

1. [Type Systems](#-type-systems)
2. [Data Structures](#-data-structures)
3. [Memory Management](#-memory-management)
4. [Error Handling](#-error-handling)
5. [Pattern Matching](#-pattern-matching)
6. [Modules & Packages](#-modules--packages)
7. [Collections & Iteration](#-collections--iteration)
8. [Concurrency](#-concurrency)
9. [Code Examples Side-by-Side](#-code-examples-side-by-side)

---

## 🎯 Type Systems

### Python: Dynamic Typing
```python
# Types are checked at runtime
def insert_row(self, name: str, values: list) -> dict:
    # Type hints are optional, not enforced
    row = {col: val for col, val in zip(self.columns, values)}
    return row  # Can return anything
```

### Rust: Static + Strong Typing
```rust
// Types checked at compile time, explicit
pub fn insert_row(&mut self, name: &str, values: Vec<Value>) 
    -> Result<Row, String> {
    // ^ Return type MUST match
    // Row and Value are custom types (enums/structs)
    let mut row = HashMap::new();
    Ok(row)  // Must wrap in Result
}
```

**Key Rust Concepts:**
- **Static Typing**: All types known at compile time
- **Type Inference**: `let x = 5;` infers `i32` automatically
- **Explicit Returns**: Function signature declares exact return type
- **No Implicit Conversions**: `i32` ≠ `i64`, must convert explicitly

### Go: Static + Interface-Based
```go
// Static typing with interface flexibility
func (b *BTreeStorage) InsertRow(name string, values []interface{}) Row {
    // interface{} = "any type" (like Python's Any)
    row := make(Row)  // Row = map[string]interface{}
    return row
}
```

**Key Go Concepts:**
- **Static Typing**: Types checked at compile time
- **Type Inference**: `x := 5` infers `int`
- **`interface{}`**: Empty interface accepts any type (Python's `Any`)
- **Duck Typing**: Interfaces satisfied implicitly

---

## 📦 Data Structures

### Value Types: Representing SQL Data

#### Python: Built-in Types
```python
# Uses Python's native types directly
values = [1, 3.14, "Alice", None]  # int, float, str, None
# Type: List[Any]
```

#### Rust: Custom Enum
```rust
// Explicit enum for type safety
pub enum Value {
    Integer(i64),    // Holds an i64
    Float(f64),      // Holds an f64
    Text(String),    // Holds a String
    Null,            // No value
}

// Usage:
let values = vec![
    Value::Integer(1),
    Value::Float(3.14),
    Value::Text("Alice".to_string()),
    Value::Null,
];
```

**Why Enums in Rust?**
- **Type Safety**: Compiler knows exactly what's inside
- **Pattern Matching**: Can't forget a variant
- **No Runtime Errors**: Invalid types caught at compile time
- **Memory Efficient**: Size known at compile time

#### Go: Interface{} + Type Assertions
```go
// Flexible but requires runtime checks
values := []interface{}{1, 3.14, "Alice", nil}

// Later, need type assertion:
if num, ok := values[0].(int); ok {
    // num is now int
}
```

**Why interface{} in Go?**
- **Flexibility**: Can store any type
- **Type Assertions**: Check type at runtime with `, ok` pattern
- **Similar to Python**: Dynamic-like behavior in static language

---

### Row Representation

#### Python: Dict
```python
# Native dict with string keys
Row = Dict[str, Any]
row = {"id": 1, "name": "Alice", "email": "alice@example.com"}
```

#### Rust: HashMap
```rust
// Imported from std library
use std::collections::HashMap;

type Row = HashMap<String, Value>;
let mut row = HashMap::new();
row.insert("id".to_string(), Value::Integer(1));
row.insert("name".to_string(), Value::Text("Alice".to_string()));
```

**Key Differences:**
- **Ownership**: Rust HashMap owns its keys/values
- **No Implicit Copying**: Must `.clone()` to duplicate
- **Type Safety**: Keys are `String`, values are `Value` enum

#### Go: Map
```go
// Built-in map type
type Row map[string]interface{}
row := make(Row)
row["id"] = 1
row["name"] = "Alice"
// Or: row := Row{"id": 1, "name": "Alice"}
```

**Key Features:**
- **Reference Type**: Maps are references (like Python dicts)
- **Nil Maps**: Uninitialized maps are `nil` (can't insert)
- **Make Required**: Use `make()` to initialize

---

## 💾 Memory Management

### Python: Garbage Collection
```python
# Automatic memory management
def create_table(self, name: str, columns: list):
    self._tables[name] = {"columns": columns, "rows": []}
    # Python GC handles cleanup automatically
    # Reference counting + cycle detection
```

### Rust: Ownership System
```rust
pub fn create_table(&mut self, name: &str, columns: Vec<String>) {
    //                  ^^^^ mutable reference (borrow)
    //                              ^^^ ownership transferred
    let meta = TableMeta { columns, rows: Vec::new(), ... };
    self.tables.insert(name.to_string(), meta);
    //                                   ^^^^ meta moved into HashMap
    // Can't use 'meta' after this point!
}
```

**Rust Ownership Rules:**

1. **Each value has ONE owner**
   ```rust
   let x = vec![1, 2, 3];  // x owns the vector
   let y = x;              // Ownership moved to y
   // println!("{:?}", x); // ERROR! x no longer valid
   ```

2. **Borrowing (References)**
   ```rust
   let x = vec![1, 2, 3];
   let y = &x;             // Borrow (read-only reference)
   println!("{:?}", x);    // OK! x still owns it
   println!("{:?}", y);    // OK! y just borrows
   ```

3. **Mutable Borrowing**
   ```rust
   let mut x = vec![1, 2, 3];
   let y = &mut x;         // Mutable borrow
   y.push(4);              // Can modify through y
   // println!("{:?}", x); // ERROR! Can't use x while mutably borrowed
   ```

**Common Rust Patterns:**

| Python | Rust Equivalent | Meaning |
|--------|----------------|---------|
| `self` | `&self` | Immutable borrow (read-only) |
| `self` (modify) | `&mut self` | Mutable borrow (can modify) |
| Return object | `-> Type` | Ownership transferred to caller |
| Copy object | `.clone()` | Explicit deep copy |

### Go: Garbage Collection (Like Python)
```go
func (b *BTreeStorage) CreateTable(name string, columns []string) {
    b.tables[name] = &TableMeta{Columns: columns, Rows: make([]Row, 0)}
    //               ^ Pointer to struct (allocated on heap)
    // Go GC handles cleanup automatically
}
```

**Go Memory Model:**
- **GC-Based**: Like Python, automatic cleanup
- **Pointers**: Explicit with `*` and `&` (safer than C)
- **No Manual Free**: GC handles it
- **Stack vs Heap**: Go decides automatically (escape analysis)

---

## 🚨 Error Handling

### Python: Exceptions
```python
# Raise exceptions for errors
def parse_value(self, text: str) -> Any:
    try:
        return ast.literal_eval(text)
    except Exception:
        return text.strip("'")

# OR: Simplified (our refactored version)
def parse_value(self, text: str) -> Any:
    # Just return default on error
    matches = re.findall(pattern, text)
    if not matches:
        return {}  # Empty dict indicates error
```

### Rust: Result Type
```rust
// No exceptions! Use Result<T, E>
pub fn insert_row(&mut self, name: &str, values: Vec<Value>) 
    -> Result<Row, String> {
    //  ^^^^^^ Success    ^^^^^^ Error type
    
    if !self.tables.contains_key(name) {
        return Err(format!("Table '{}' not found", name));
        //     ^^^ Error variant
    }
    
    let row = /* ... */;
    Ok(row)  // Success variant
    //  ^^
}

// Caller must handle Result:
match storage.insert_row("users", values) {
    Ok(row) => println!("Inserted: {:?}", row),
    Err(e) => println!("Error: {}", e),
}

// Or use ? operator (propagate error):
let row = storage.insert_row("users", values)?;
//                                            ^ Returns error if Err
```

**Result Enum:**
```rust
enum Result<T, E> {
    Ok(T),   // Success with value
    Err(E),  // Failure with error
}
```

**Common Patterns:**
- `?` operator: Early return on error
- `.unwrap()`: Panic if error (don't use in production!)
- `.unwrap_or(default)`: Use default value on error
- `match`: Handle both cases explicitly

### Go: Multiple Return Values
```go
// Return (value, error) tuple
func (p *SQLParser) parseValue(text string) (interface{}, error) {
    num, err := strconv.Atoi(text)
    if err != nil {
        return text, nil  // Return string if not a number
    }
    return num, nil
}

// Caller checks error:
value, err := parser.parseValue("123")
if err != nil {
    // Handle error
    return err
}
// Use value

// Or ignore error (simplified version):
func (p *SQLParser) parseValue(text string) interface{} {
    num, err := strconv.Atoi(text)
    if err != nil {
        return text  // Return default on error
    }
    return num
}
```

**Go Error Patterns:**
- **No Exceptions**: Errors are values
- **`if err != nil`**: Standard error check
- **`panic/recover`**: Exists but rarely used (like exceptions)
- **Simplified**: Can return empty values on error (like our code)

---

## 🎨 Pattern Matching

### Python: Dict Dispatch + If/Elif
```python
# Dict-based dispatch
parsers = {
    ("CREATE", "DATABASE"): lambda: ("CREATE_DATABASE", {...}),
    ("CREATE", "TABLE"): lambda: ("CREATE_TABLE", {...}),
}

key = (command, tokens[1].upper())
if key in parsers:
    command, details = parsers[key]()
else:
    command, details = "UNKNOWN", {}
```

### Rust: Match Expression (Powerful!)
```rust
// Pattern matching on enums
let command = match command_str.as_str() {
    "COMMIT" => CommandType::Commit,
    
    "CREATE" if tokens.len() > 1 => match tokens[1].to_uppercase().as_str() {
        "DATABASE" => CommandType::CreateDatabase { 
            name: tokens[2].to_lowercase() 
        },
        "TABLE" => /* parse table */,
        "INDEX" => /* parse index */,
        _ => CommandType::Unknown,  // Default case
    },
    
    "INSERT" => /* parse insert */,
    _ => CommandType::Unknown,
};
```

**Match Features:**
- **Exhaustive**: Compiler ensures all cases covered
- **Guards**: `if` conditions in patterns
- **Destructuring**: Extract values from enums/structs
- **No Fallthrough**: Each arm is independent

**More Match Examples:**
```rust
// Match on Option<T>
match some_option {
    Some(value) => println!("Got: {}", value),
    None => println!("Nothing"),
}

// Match on Result<T, E>
match result {
    Ok(data) => process(data),
    Err(e) => handle_error(e),
}

// Match with destructuring
match parsed_command.command {
    CommandType::Insert { table, values } => {
        // Use table and values directly
    },
    _ => {},
}
```

### Go: Switch Statement
```go
// Switch without break needed
switch command {
case "COMMIT":
    details = map[string]interface{}{}
    
case "INSERT":
    command, details = "INSERT", p.parseInsert(text)
    
case "UPDATE":
    command, details = "UPDATE", p.parseUpdate(text)
    
default:
    command, details = "UNKNOWN", map[string]interface{}{}
}

// Can switch on types:
switch v := value.(type) {
case int:
    fmt.Printf("Integer: %d\n", v)
case string:
    fmt.Printf("String: %s\n", v)
default:
    fmt.Printf("Unknown type\n")
}
```

**Go Switch Features:**
- **No Break**: Automatic (no fallthrough by default)
- **Multiple Cases**: `case 1, 2, 3:`
- **Type Switch**: Switch on interface{} types
- **Conditions**: `case x > 10:`

---

## 📁 Modules & Packages

### Python: Modules & Imports
```python
# File: mini_sqlite/core/parser.py
from typing import Dict, List, Any
import re
from dataclasses import dataclass

class SQLParser:
    # ...

# File: mini_sqlite/core/engine.py
from mini_sqlite.core.parser import SQLParser
from mini_sqlite.core.executor import SQLExecutor

class DatabaseEngine:
    def __init__(self):
        self.parser = SQLParser()
```

**Python Structure:**
- Files are modules
- Folders with `__init__.py` are packages
- Import by path: `from package.module import Class`

### Rust: Modules & Crates
```rust
// File: src/core/parser.rs
use std::collections::HashMap;  // From standard library
use regex::Regex;               // From external crate

pub struct SQLParser;
//  ^^^ public (accessible outside module)

impl SQLParser {
    pub fn parse(&self, query: &str) -> ParsedCommand {
        // ...
    }
}

// File: src/core/mod.rs (module declaration)
pub mod parser;    // Makes parser.rs accessible
pub mod executor;  // Makes executor.rs accessible
pub mod storage;   // Makes storage/ folder accessible

// File: src/core/engine.rs
use crate::core::parser::SQLParser;
//  ^^^^^ current crate root
use crate::core::executor::SQLExecutor;

pub struct DatabaseEngine {
    parser: SQLParser,
    executor: SQLExecutor,
}
```

**Rust Module System:**
- **Crate**: A compilation unit (library or binary)
- **Module**: Namespace within crate (file or folder)
- **`mod.rs`**: Declares submodules (like `__init__.py`)
- **`pub`**: Make items public (private by default!)
- **`use`**: Import items (like Python's `from ... import`)
- **`crate::`**: Absolute path from crate root
- **`super::`**: Parent module
- **`self::`**: Current module

### Go: Packages
```go
// File: core/parser.go
package core  // Package name (same for all files in directory)

import (
    "regexp"
    "strings"
)

type SQLParser struct {
    // Capitalized = public
    // lowercase = private to package
}

// Capitalized = public function
func NewSQLParser() *SQLParser {
    return &SQLParser{}
}

// File: core/engine.go
package core  // Same package!

// Can access SQLParser directly (same package)
type DatabaseEngine struct {
    parser   *SQLParser
    executor *SQLExecutor
}

// File: main.go
package main

import (
    "mini_sqlite_golang/core"  // Import package path
)

func main() {
    engine := core.NewDatabaseEngine()
    //        ^^^^ Package name used
}
```

**Go Package System:**
- **Package**: All files in a directory
- **Capitalization**: Determines visibility (Public/private)
- **Import Path**: Based on file system
- **`package main`**: Entry point package
- **No Circular Imports**: Not allowed

---

## 🔄 Collections & Iteration

### Lists/Vectors/Slices

#### Python: Lists
```python
# Dynamic, can grow
columns = ["id", "name", "email"]
columns.append("age")
columns.extend(["phone", "address"])

for col in columns:
    print(col)

# List comprehension
uppercase = [col.upper() for col in columns]
```

#### Rust: Vectors
```rust
// Growable array
let mut columns = vec!["id", "name", "email"];
columns.push("age");
columns.extend(vec!["phone", "address"]);

for col in &columns {  // Borrow each element
    println!("{}", col);
}

// Iterator chains (like Python comprehensions)
let uppercase: Vec<String> = columns
    .iter()
    .map(|col| col.to_uppercase())
    .collect();
```

**Rust Iteration Patterns:**
```rust
// Consume (move) items
for col in columns { }  // Can't use columns after

// Borrow items (read-only)
for col in &columns { }  // Can still use columns

// Borrow mutably
for col in &mut columns { }  // Can modify items

// With index
for (i, col) in columns.iter().enumerate() { }
```

#### Go: Slices
```go
// Dynamic array
columns := []string{"id", "name", "email"}
columns = append(columns, "age")
columns = append(columns, []string{"phone", "address"}...)

for _, col := range columns {
    //  ^ Ignore index
    fmt.Println(col)
}

// With index
for i, col := range columns {
    fmt.Printf("%d: %s\n", i, col)
}
```

**Go Range:**
- `range` iterates over slices, maps, channels
- Returns `(index, value)` for slices
- Returns `(key, value)` for maps
- `_` discards unwanted values

### Dictionaries/HashMaps/Maps

#### Python: Dicts
```python
# Built-in dict
indexes = {"id": [], "name": []}
indexes["email"] = []

for column, rows in indexes.items():
    print(f"{column}: {len(rows)}")

# Dict comprehension
lengths = {col: len(rows) for col, rows in indexes.items()}
```

#### Rust: HashMap
```rust
use std::collections::HashMap;

let mut indexes: HashMap<String, Vec<Row>> = HashMap::new();
indexes.insert("id".to_string(), Vec::new());

for (column, rows) in &indexes {
    println!("{}: {}", column, rows.len());
}

// Iterator transformation
let lengths: HashMap<String, usize> = indexes
    .iter()
    .map(|(col, rows)| (col.clone(), rows.len()))
    .collect();
```

#### Go: Maps
```go
indexes := make(map[string][]Row)
indexes["id"] = []Row{}
indexes["name"] = []Row{}

for column, rows := range indexes {
    fmt.Printf("%s: %d\n", column, len(rows))
}

// Build new map
lengths := make(map[string]int)
for col, rows := range indexes {
    lengths[col] = len(rows)
}
```

---

## 🧵 Concurrency

### Python: GIL + Threading/Asyncio
```python
# Threading (limited by GIL for CPU-bound)
from threading import Thread

def handle_request():
    # Process request
    pass

thread = Thread(target=handle_request)
thread.start()

# Asyncio (for I/O-bound)
import asyncio

async def handle_request():
    await asyncio.sleep(1)
    return "Done"

asyncio.run(handle_request())
```

### Rust: Threads + Arc/Mutex (Web Server Example)
```rust
use std::sync::{Arc, Mutex};
use std::thread;

// Arc = Atomic Reference Counted (thread-safe shared ownership)
// Mutex = Mutual Exclusion (thread-safe mutable access)

let engine = Arc::new(Mutex::new(DatabaseEngine::new()));
//           ^^^ Thread-safe reference counting
//                    ^^^^^ Thread-safe lock

// Clone Arc (cheap - just increments counter)
let engine_clone = Arc::clone(&engine);

thread::spawn(move || {
    // This thread owns engine_clone
    let mut eng = engine_clone.lock().unwrap();
    //                         ^^^^^ Acquire lock
    eng.execute_query("SELECT * FROM users");
    // Lock released when eng goes out of scope
});
```

**Why Arc<Mutex<T>>?**
- **Arc**: Share ownership across threads (like Python's shared object)
- **Mutex**: Ensure only one thread modifies at a time
- **lock()**: Acquire mutex (blocks if locked)
- **Drop**: Lock released automatically (RAII pattern)

### Go: Goroutines + Channels
```go
// Goroutines (lightweight threads)
go func() {
    // Runs concurrently
    handleRequest()
}()

// Channels (message passing)
ch := make(chan string)

go func() {
    ch <- "Hello"  // Send to channel
}()

msg := <-ch  // Receive from channel
fmt.Println(msg)

// Web server example
http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
    // Each request runs in its own goroutine automatically!
    query := r.FormValue("query")
    result := engine.Execute(query)
    fmt.Fprintf(w, result)
})
```

**Go Concurrency:**
- **Goroutines**: Ultra-lightweight (start 1000s easily)
- **Channels**: Communicate by sharing, not sharing memory
- **Automatic**: `http` package handles goroutines for you

---

## 🔗 Rust Reference Types Overview

- `&T` (shared borrow): read-only access, any number of simultaneous borrows.
- `&mut T` (mutable borrow): exclusive access, at most one active mutable borrow.
- Lifetimes (`'a`): annotate how long references remain valid (compiler infers in most cases).
- Interior mutability: use `Cell`, `RefCell`, or `Mutex` when mutation is needed through shared references.
- Smart pointers: `Rc<T>`/`Arc<T>` for shared ownership, optionally paired with `RefCell<T>` or `Mutex<T>` for mutation.
- Slices and string slices (`&[T]`, `&str`): borrowed views into contiguous data without copying.

```rust
fn describe(name: &str, rows: &[Row]) -> usize {
    rows.iter().filter(|row| row.contains_key(name)).count()
}
```

```rust
use std::sync::{Arc, Mutex};

let shared = Arc::new(Mutex::new(Vec::new()));
{
    let mut guard = shared.lock().unwrap();
    guard.push("row");
} // lock released here
```

---

## 📝 Code Examples Side-by-Side

### Example 1: Parsing INSERT Statement

#### Python
```python
def _parse_insert(self, text: str) -> Dict[str, Any]:
    match = re.match(
        r"INSERT\s+INTO\s+(?P<table>\w+)\s+VALUES\s*\((?P<values>.+)\)$",
        text,
        re.IGNORECASE
    )
    if not match:
        return {}
    
    values = self._parse_value_list(match.group("values"))
    return {
        "table": match.group("table").lower(),
        "values": values
    }
```

#### Rust
```rust
fn parse_insert(&self, text: &str) -> CommandType {
    let re = Regex::new(
        r"(?i)INSERT\s+INTO\s+(?P<table>\w+)\s+VALUES\s*\((?P<values>.+)\)$"
    ).unwrap();
    
    match re.captures(text) {
        Some(caps) => {
            let table = caps.name("table").unwrap().as_str().to_lowercase();
            let values_str = caps.name("values").unwrap().as_str();
            let values = self.parse_value_list(values_str);
            
            CommandType::Insert { table, values }
        },
        None => CommandType::Unknown,
    }
}
```

**Key Differences:**
- Rust returns `CommandType` enum (not dict)
- Pattern matching instead of if/return
- Explicit error handling with `Option<T>`

#### Go
```go
func (p *SQLParser) parseInsert(text string) map[string]interface{} {
    re := regexp.MustCompile(
        `(?i)INSERT\s+INTO\s+(?P<table>\w+)\s+VALUES\s*\((?P<values>.+)\)$`,
    )
    
    matches := re.FindStringSubmatch(text)
    if matches == nil {
        return map[string]interface{}{}
    }
    
    table := strings.ToLower(matches[1])
    valuesStr := matches[2]
    values := p.parseValueList(valuesStr)
    
    return map[string]interface{}{
        "table":  table,
        "values": values,
    }
}
```

**Key Differences:**
- Returns map (like Python dict)
- No named groups in standard regexp (use index)
- Simple nil check

### Example 2: Table Storage

#### Python
```python
def create_table(self, name: str, columns: List[str]) -> None:
    self._tables[name] = {
        "columns": columns,
        "rows": [],
        "indexes": {}
    }
    if self._pager:
        self._pager.allocate_page()
```

#### Rust
```rust
pub fn create_table(&mut self, name: &str, columns: Vec<String>) {
    //                 ^^^^ Must borrow mutably to modify
    let meta = TableMeta {
        columns,  // Ownership moved
        rows: Vec::new(),
        indexes: HashMap::new(),
    };
    self.tables.insert(name.to_string(), meta);
    //                      ^^^^^^^^^^^ Create owned String from &str
    
    if let Some(ref mut pager) = self.pager {
        //  ^^^^^^^ Pattern match + mutable borrow
        pager.allocate_page();
    }
}
```

**Key Concepts:**
- `&mut self`: Mutable borrow needed to modify
- `name.to_string()`: Convert `&str` to owned `String`
- `if let`: Pattern match on `Option<T>`
- `ref mut`: Mutable reference inside pattern

#### Go
```go
func (b *BTreeStorage) CreateTable(name string, columns []string) {
    b.tables[name] = &TableMeta{
        //           ^ Pointer to struct
        Columns: columns,
        Rows:    make([]Row, 0),
        Indexes: make(map[string]Index),
    }
    
    if b.pager != nil {
        b.pager.AllocatePage()
    }
}
```

**Key Concepts:**
- Receiver `(b *BTreeStorage)`: Method on pointer (can modify)
- `&TableMeta{}`: Create pointer to struct
- `make()`: Initialize slice/map
- `nil`: Go's null value

### Example 3: Filtering Rows

#### Python
```python
def _rows_for(self, table: str, where: Optional[Dict[str, Any]]) -> Iterable[Dict[str, Any]]:
    """Return rows matching condition."""
    meta = self._tables[table]
    
    if not where:
        return meta["rows"]
    
    column = where["column"]
    value = where["value"]
    
    # Use index if available
    if column in meta["indexes"]:
        return meta["indexes"][column].get(value, [])
    
    # Linear scan
    return (row for row in meta["rows"] if row.get(column) == value)
```

#### Rust
```rust
fn rows_for<'a>(
    &'a self,
    table: &str,
    condition: Option<&Condition>,
) -> Box<dyn Iterator<Item = &'a Row> + 'a> {
    //  ^^^ Boxed trait object (dynamic dispatch)
    //                           ^^ Lifetime annotation
    
    let meta = self.tables.get(table).unwrap();
    
    match condition {
        None => Box::new(meta.rows.iter()),
        
        Some(cond) => {
            if let Some(index) = meta.indexes.get(&cond.column) {
                // Use index
                let key = value_to_string(&cond.value);
                Box::new(index.get(&key).map_or(
                    Vec::new().iter(),
                    |indices| indices.iter().map(|&i| &meta.rows[i])
                ))
            } else {
                // Linear scan
                Box::new(meta.rows.iter().filter(move |row| {
                    row.get(&cond.column)
                        .map_or(false, |v| v == &cond.value)
                }))
            }
        }
    }
}
```

**Key Rust Concepts:**
- **Lifetimes** (`'a`): Ensure iterator doesn't outlive data
- **Trait Objects** (`dyn Iterator`): Runtime polymorphism
- **Closures**: `move` captures ownership
- **Iterator Chains**: `filter`, `map`, etc.

#### Go
```go
func (b *BTreeStorage) rowsFor(table string, where map[string]interface{}) []Row {
    meta := b.tables[table]
    
    if where == nil {
        return meta.Rows
    }
    
    column := where["column"].(string)
    //                         ^^^^^^^^ Type assertion
    value := where["value"]
    
    // Use index if available
    if index, ok := meta.Indexes[column]; ok {
        if rows, ok := index[value]; ok {
            return rows
        }
        return []Row{}
    }
    
    // Linear scan
    var result []Row
    for _, row := range meta.Rows {
        if row[column] == value {
            result = append(result, row)
        }
    }
    return result
}
```

**Key Go Concepts:**
- **Type Assertion**: `value.(type)` extracts concrete type
- **Comma-ok Idiom**: `if x, ok := map[key]; ok { }`
- **No Generics** (pre-Go 1.18): Use `interface{}`
- **Simple Loops**: No complex iterator chains

---

## 🎓 Learning Path Recommendations

### For Rust
1. ✅ **Start**: [The Rust Book](https://doc.rust-lang.org/book/) - Chapters 1-10
2. ✅ **Practice**: [Rustlings](https://github.com/rust-lang/rustlings) - Interactive exercises
3. ✅ **Deep Dive**: Focus on:
   - Ownership & Borrowing (Chapter 4)
   - Enums & Pattern Matching (Chapter 6)
   - Error Handling with Result (Chapter 9)
   - Lifetimes (Chapter 10.3)
4. ✅ **Project**: Read `mini_sqlite_rust` source code in order:
   - `src/core/parser.rs` - Enums & pattern matching
   - `src/core/storage/btree.rs` - Ownership & borrowing
   - `src/web/server.rs` - Arc/Mutex concurrency

### For Go
1. ✅ **Start**: [A Tour of Go](https://go.dev/tour/) - Complete interactive tour
2. ✅ **Read**: [Effective Go](https://go.dev/doc/effective_go) - Idiomatic patterns
3. ✅ **Practice**: [Go by Example](https://gobyexample.com/) - Practical examples
4. ✅ **Deep Dive**: Focus on:
   - Slices & Maps
   - Interfaces & Type Assertions
   - Goroutines & Channels
   - Error Handling Patterns
5. ✅ **Project**: Read `mini_sqlite_golang` source code in order:
   - `core/parser.go` - Maps & interfaces
   - `core/storage/btree.go` - Structs & methods
   - `web/server.go` - http.Server patterns

---

## 🔑 Key Takeaways

### Python → Rust
| Concept | Python | Rust | Reason |
|---------|--------|------|--------|
| **Memory** | GC | Ownership | Zero-cost abstractions |
| **Types** | Dynamic | Static + Enums | Compile-time safety |
| **Errors** | Exceptions | Result<T, E> | Explicit handling |
| **Null** | None | Option<T> | No null pointer errors |
| **Mutability** | Default | Explicit `mut` | Controlled side effects |

### Python → Go
| Concept | Python | Go | Reason |
|---------|--------|-----|--------|
| **Memory** | GC | GC | Ease of use |
| **Types** | Dynamic | Static + interface{} | Flexibility + safety |
| **Errors** | Exceptions | (value, error) | Explicit handling |
| **Null** | None | nil | Simple |
| **Concurrency** | Threading/Asyncio | Goroutines | Built-in, lightweight |

### When to Use Each

**Use Python when:**
- 🎓 Learning/prototyping
- 📊 Data science/scripting
- ⚡ Development speed > execution speed

**Use Rust when:**
- 🚀 Maximum performance needed
- 🔒 Memory safety critical
- 🧵 Systems programming
- 📦 Zero runtime overhead required

**Use Go when:**
- ⚖️ Balance of simplicity + performance
- 🌐 Building web services/APIs
- 🔄 Concurrency is important
- ⚡ Fast compilation matters

---

## 📚 Quick Reference

### Common Patterns Translation

| Task | Python | Rust | Go |
|------|--------|------|-----|
| **Create list** | `[]` | `Vec::new()` | `make([]Type, 0)` |
| **Create dict/map** | `{}` | `HashMap::new()` | `make(map[K]V)` |
| **Iterate** | `for x in list:` | `for x in &list {` | `for _, x := range list {` |
| **Check exists** | `if key in dict:` | `if dict.contains_key(key)` | `if _, ok := m[key]; ok {` |
| **String concat** | `f"{a} {b}"` | `format!("{} {}", a, b)` | `fmt.Sprintf("%s %s", a, b)` |
| **Optional value** | `x or default` | `opt.unwrap_or(default)` | `if v, ok := m[k]; ok { v } else { default }` |
| **Error handling** | `try/except` | `Result<T,E>` + `?` | `if err != nil { return err }` |

---

## 🤔 Decoding Complex Go Types

### The Confusing `map[string]map[string]map[string]interface{}`

You found this in `executor.go` line 165:
```go
func (e *SQLExecutor) Describe() map[string]map[string]map[string]interface{} {
```

Let's break it down **from right to left** (Go type declarations work this way):

#### Step-by-Step Breakdown

```go
map[string]map[string]map[string]interface{}
    ^        ^        ^        ^
    |        |        |        |
    |        |        |        └─ innermost: interface{} (any value)
    |        |        └────────── map with string keys, interface{} values
    |        └─────────────────── map with string keys, map values
    └──────────────────────────── outermost: map with string keys
```

#### What It Represents

This is a **3-level nested structure**:

```
Level 1: Database names
    └─ Level 2: Table names
        └─ Level 3: Table properties (columns, row_count, indexes)
            └─ Values: Any type (interface{})
```

#### Visual Example

```go
{
    "default": {                           // Database name (Level 1)
        "users": {                         // Table name (Level 2)
            "columns": []string{"id", "name"},      // Property (Level 3)
            "row_count": 5,                         // Property (Level 3)
            "indexes": []string{"id"}               // Property (Level 3)
        },
        "orders": {                        // Another table (Level 2)
            "columns": []string{"order_id", "user_id"},
            "row_count": 10,
            "indexes": []string{}
        }
    },
    "demo": {                              // Another database (Level 1)
        "products": {
            "columns": []string{"product_id", "name", "price"},
            "row_count": 20,
            "indexes": []string{"product_id"}
        }
    }
}
```

#### Reading the Type (Right to Left)

1. **`interface{}`**: The actual values (can be anything: int, string, []string, etc.)
2. **`map[string]interface{}`**: A map where keys are property names ("columns", "row_count", "indexes")
3. **`map[string]map[string]interface{}`**: A map where keys are table names ("users", "orders")
4. **`map[string]map[string]map[string]interface{}`**: A map where keys are database names ("default", "demo")

#### Comparison Across Languages

**Python:**
```python
def describe(self) -> dict[str, dict[str, dict[str, object]]]:
    # Much easier to read!
    # Returns: dict of databases -> dict of tables -> dict of properties
    return {
        "default": {
            "users": {
                "columns": ["id", "name"],
                "row_count": 5,
                "indexes": ["id"]
            }
        }
    }
```

**Rust:**
```rust
pub fn describe(&self) -> HashMap<String, HashMap<String, serde_json::Value>> {
    // Uses serde_json::Value for flexibility
    // Returns: HashMap of databases -> HashMap of tables -> JSON values
}
```

**Go:**
```go
func (e *SQLExecutor) Describe() map[string]map[string]map[string]interface{} {
    // Type specified inline - can be verbose!
    // Returns: map of databases -> map of tables -> map of properties -> any value
}
```

#### Why So Complex in Go?

**Reason 1: No Generics (pre-Go 1.18)**
```go
// Can't do this in older Go:
// type TableInfo struct {
//     Columns []string
//     RowCount int
//     Indexes []string
// }
// func Describe() map[string]map[string]TableInfo

// So we use interface{} for flexibility:
map[string]interface{} // Can hold any value type
```

**Reason 2: Dynamic Schema**
```go
// Each table might have different properties
// interface{} allows different types per key:
{
    "columns": []string{"id", "name"},   // slice of strings
    "row_count": 5,                       // int
    "indexes": []string{"id"},            // slice of strings
}
```

**Reason 3: JSON-like Structure**
```go
// This mimics JSON structure:
// {
//   "db_name": {
//     "table_name": {
//       "property": value
//     }
//   }
// }
```

#### Simplification Strategies

**Strategy 1: Type Alias** (makes it readable)
```go
type TableInfo map[string]interface{}
type DatabaseInfo map[string]TableInfo
type SchemaSnapshot map[string]DatabaseInfo

func (e *SQLExecutor) Describe() SchemaSnapshot {
    // Much clearer!
}
```

**Strategy 2: Structs** (type-safe, but rigid)
```go
type TableInfo struct {
    Columns []string
    RowCount int
    Indexes []string
}

type DatabaseInfo map[string]TableInfo
type SchemaSnapshot map[string]DatabaseInfo

func (e *SQLExecutor) Describe() SchemaSnapshot {
    // Type-safe but all tables must have same structure
}
```

**Strategy 3: Custom Types** (best of both worlds)
```go
type PropertyMap map[string]interface{}
type TableMap map[string]PropertyMap
type DatabaseMap map[string]TableMap

func (e *SQLExecutor) Describe() DatabaseMap {
    // Clear intent, still flexible
}
```

#### How to Access This Type

```go
// Get the description
desc := executor.Describe()

// Access database
defaultDB := desc["default"]  // type: map[string]map[string]interface{}

// Access table
usersTable := defaultDB["users"]  // type: map[string]interface{}

// Access property (need type assertion!)
columns := usersTable["columns"].([]string)  // Assert to []string
//                                 ^^^^^^^^^ Type assertion required!
rowCount := usersTable["row_count"].(int)    // Assert to int
indexes := usersTable["indexes"].([]string)  // Assert to []string
```

#### Common Go Type Patterns

| Pattern | Meaning | Example |
|---------|---------|---------|
| `map[K]V` | Map with keys of type K, values of type V | `map[string]int` |
| `[]T` | Slice (dynamic array) of type T | `[]string` |
| `interface{}` | Any type | `interface{}` |
| `map[string]interface{}` | Dict-like structure | JSON object |
| Nested maps | Multi-level data | Tree structures |

#### Mental Model for Reading Complex Types

**Rule: Read from RIGHT to LEFT, outside to inside**

```go
map[A]map[B]map[C]D
      3     2     1 <- Reading order

1. Start with innermost: map[C]D = "map with C keys, D values"
2. Next level: map[B](map[C]D) = "map with B keys, where values are map[C]D"
3. Outermost: map[A](map[B]map[C]D) = "map with A keys, where values are map[B]map[C]D"
```

#### Practice Examples

```go
// Simple map
map[string]int
// "map with string keys and int values"

// Map of slices
map[string][]int
// "map with string keys and slice-of-int values"

// Slice of maps
[]map[string]int
// "slice where each element is a map[string]int"

// Map of maps
map[string]map[int]bool
// "map with string keys, where values are map[int]bool"

// The confusing one!
map[string]map[string]map[string]interface{}
// "map with string keys, where values are map[string]map[string]interface{}"
//   which means values are "map with string keys, where values are map[string]interface{}"
//     which means values are "map with string keys and any-type values"
```

#### When You See This Pattern

✅ **Good sign:** You're dealing with hierarchical data (like database schema)
⚠️ **Watch out:** Type assertions required when accessing values
💡 **Tip:** Consider type aliases for readability

---

## 🎓 Key Insights for Python Developers

### What Makes Each Language Special

#### Python: Simplicity First 🐍
**Philosophy**: "There should be one-- and preferably only one --obvious way to do it."

You already know Python's strengths:
- Dynamic typing lets you prototype quickly
- Duck typing means interfaces are implicit
- Everything is an object (even functions!)
- Batteries included (rich standard library)

**When moving to Rust/Go, you'll miss:**
- REPL for quick experiments
- List/dict comprehensions
- Easy string manipulation
- `import this` zen

#### Rust: Safety Without Compromise 🦀
**Philosophy**: "Fearless concurrency" and "Zero-cost abstractions"

**What Python developers find eye-opening:**

1. **Compile-time guarantees eliminate entire bug categories**
   ```rust
   // This won't compile:
   let mut data = vec![1, 2, 3];
   let first = &data[0];  // Borrow data
   data.push(4);          // ERROR: Can't mutate while borrowed!
   println!("{}", first);
   ```
   In Python, this would crash at runtime. Rust catches it before you run.

2. **Enums are powerful type unions**
   ```rust
   // Python uses union types or isinstance checks
   # value: int | float | str | None
   
   // Rust uses exhaustive pattern matching
   match value {
       Value::Integer(i) => /* must handle */,
       Value::Float(f) => /* must handle */,
       Value::Text(s) => /* must handle */,
       Value::Null => /* must handle */,
       // Forgot a variant? Compiler error!
   }
   ```

3. **Zero runtime overhead**
   - No garbage collection pauses
   - Iterators compile to the same code as hand-written loops
   - Abstractions cost nothing at runtime

**The Learning Curve:**
- Week 1: Fight the borrow checker (frustrating!)
- Week 2: Understand ownership (aha moments!)
- Week 3: Love the compiler (it's helping you!)
- Month 2: Write fearless concurrent code

#### Go: Pragmatic Simplicity 🐹
**Philosophy**: "Less is more" and "Do more with less"

**What Python developers appreciate:**

1. **Familiar feel with performance gains**
   ```go
   // Looks almost like Python!
   users := map[string]User{
       "alice": User{ID: 1, Name: "Alice"},
       "bob":   User{ID: 2, Name: "Bob"},
   }
   
   for name, user := range users {
       fmt.Printf("%s: %v\n", name, user)
   }
   ```

2. **Built-in concurrency is magical**
   ```go
   // Goroutines = lightweight threads
   go fetchData()  // Runs concurrently!
   
   // Channels = safe communication
   ch := make(chan Result)
   go func() { ch <- compute() }()
   result := <-ch  // Wait for result
   ```

3. **Fast compilation keeps you in flow**
   - 2-second builds (vs Rust's 60 seconds)
   - Feels almost like Python's instant feedback
   - Change → Build → Run cycle stays fast

**The Trade-offs:**
- More verbose than Python (explicit error checking)
- Less flexible than Python (static typing)
- More pragmatic than Rust (GC for simplicity)

---

## 🔍 Real Implementation Patterns

### Pattern 1: Representing Dynamic Data

**The Problem:** SQL values can be integers, floats, strings, or NULL

#### Python Approach: Duck Typing
```python
# Just use native types
values = [1, 3.14, "Alice", None]

# Type checking at runtime if needed
if isinstance(value, int):
    # handle integer
```

**Pros:** Super flexible, no ceremony  
**Cons:** Runtime errors, no IDE autocomplete

#### Rust Approach: Type-Safe Enums
```rust
// Define all possible types upfront
enum Value {
    Integer(i64),
    Float(f64),
    Text(String),
    Null,
}

// Compiler forces you to handle all cases
match value {
    Value::Integer(i) => /* always know it's i64 */,
    Value::Float(f) => /* always know it's f64 */,
    Value::Text(s) => /* always know it's String */,
    Value::Null => /* must handle or compiler error */,
}
```

**Pros:** Zero runtime type errors, exhaustive checks  
**Cons:** More upfront design, can't add types easily

#### Go Approach: Runtime Type Checks
```go
// Use interface{} for flexibility
values := []interface{}{1, 3.14, "Alice", nil}

// Check types when needed
switch v := value.(type) {
case int:
    // v is int here
case float64:
    // v is float64 here
case string:
    // v is string here
default:
    // handle unknown
}
```

**Pros:** Flexible like Python, static like Rust  
**Cons:** Verbose type assertions, runtime panics possible

**💡 Python Developer Insight:**  
Rust forces you to think about all cases upfront (design time), while Go lets you check types as needed (runtime), similar to Python's `isinstance()`. Pick Rust when correctness is critical, Go when flexibility matters.

---

### Pattern 2: Handling Missing Values

**The Problem:** A table column might not exist in a row

#### Python: `None` and `.get()`
```python
# Simple and elegant
row = {"id": 1, "name": "Alice"}
email = row.get("email")  # Returns None if missing

# Or with default
email = row.get("email", "no-email@example.com")
```

#### Rust: `Option<T>` Type
```rust
// Explicit optional values
let row: HashMap<String, Value> = /* ... */;
let email: Option<&Value> = row.get("email");

// Must handle both cases
match email {
    Some(value) => println!("Email: {}", value),
    None => println!("No email"),
}

// Or use default
let email = row.get("email").unwrap_or(&Value::Null);
```

**Key Insight:**  
Rust's `Option<T>` is like Python's `None`, but the compiler forces you to check it. No more `AttributeError: 'NoneType' object has no attribute 'x'`

#### Go: Comma-ok Idiom
```go
row := map[string]interface{}{"id": 1, "name": "Alice"}

// Check if key exists
if email, ok := row["email"]; ok {
    // email exists and is in 'email' variable
} else {
    // email doesn't exist
}

// Or just access (returns zero value if missing)
email := row["email"]  // nil if missing
```

**� Python Developer Insight:**  
Coming from Python, Go's comma-ok pattern feels natural (like `try/except KeyError`), while Rust's `Option<T>` feels strict but prevents bugs. Both are more explicit than Python's permissive approach.

---

### Pattern 3: Error Handling Philosophy

#### Python: Exceptions (EAFP - Easier to Ask Forgiveness than Permission)
```python
# Just try it, catch if it fails
try:
    result = risky_operation()
    process(result)
except Exception as e:
    handle_error(e)
```

#### Rust: Result Type (Explicit Error Handling)
```rust
// Every error is visible in the function signature
fn risky_operation() -> Result<Data, Error> {
    if something_wrong {
        return Err(Error::new("oops"));
    }
    Ok(data)
}

// Caller must handle explicitly
match risky_operation() {
    Ok(data) => process(data),
    Err(e) => handle_error(e),
}

// Or propagate with ? operator (like re-raising in Python)
let result = risky_operation()?;
```

#### Go: Error Values (Check Every Call)
```go
// Errors are return values
func riskyOperation() (Data, error) {
    if somethingWrong {
        return Data{}, errors.New("oops")
    }
    return data, nil
}

// Check every call
data, err := riskyOperation()
if err != nil {
    return err  // Handle explicitly
}
process(data)
```

**💡 For This Project:**  
We intentionally **avoided verbose error handling** to keep code simple:
- Python: Return empty values on error
- Rust: Minimal `Result` usage
- Go: Simple returns without extensive checking

This is fine for educational code. Production code would be more defensive.

---

### Pattern 4: Memory and Performance

#### Where Python Shines
```python
# Creating 1 million rows - easy!
rows = [{"id": i, "name": f"User{i}"} for i in range(1_000_000)]

# Python handles memory automatically
# Just write your logic, GC handles cleanup
```

**Cost:** ~200MB RAM, ~2 seconds

#### Rust's Zero-Cost Approach
```rust
// Same task, but more explicit
let rows: Vec<Row> = (0..1_000_000)
    .map(|i| {
        let mut row = HashMap::new();
        row.insert("id".to_string(), Value::Integer(i));
        row.insert("name".to_string(), Value::Text(format!("User{}", i)));
        row
    })
    .collect();
```

**Cost:** ~80MB RAM, ~0.3 seconds  
**Why faster?** No GC overhead, direct memory allocation

#### Go's Balanced Approach
```go
// Similar to Python, with GC
rows := make([]Row, 1_000_000)
for i := 0; i < 1_000_000; i++ {
    rows[i] = Row{
        "id":   i,
        "name": fmt.Sprintf("User%d", i),
    }
}
```

**Cost:** ~120MB RAM, ~0.5 seconds  
**Why faster than Python?** Compiled code, efficient GC

**💡 Python Developer Insight:**  
Python optimizes developer time. Rust optimizes runtime. Go balances both. Choose based on your bottleneck: development speed or execution speed.

---

## �🚀 Practical Migration Tips

### From Python to Rust

**1. Start with the type system**
```python
# Python
def process(data: dict[str, Any]) -> list[str]:
    return [str(v) for v in data.values()]
```

```rust
// Rust - think about types first
use std::collections::HashMap;

fn process(data: HashMap<String, Value>) -> Vec<String> {
    data.values()
        .map(|v| v.to_string())
        .collect()
}
```

**Tip:** Design your types (enums/structs) before writing logic

**2. Embrace the borrow checker**
- Don't fight it - it's preventing bugs!
- Use references (`&`) for reading
- Use mutable references (`&mut`) for modifying
- Clone only when ownership transfer is needed

**3. Iterator chains replace comprehensions**
```python
# Python
result = [x * 2 for x in numbers if x > 0]
```

```rust
// Rust
let result: Vec<i32> = numbers
    .iter()
    .filter(|&&x| x > 0)
    .map(|&x| x * 2)
    .collect();
```

### From Python to Go

**1. Embrace explicit error handling**
```python
# Python
result = risky_call()  # Might raise exception
```

```go
// Go
result, err := riskyCall()
if err != nil {
    return err  // Handle explicitly
}
```

**Tip:** Write `if err != nil` checks - it becomes muscle memory

**2. Use struct tags for JSON**
```python
# Python
@dataclass
class User:
    id: int
    name: str
    
# Automatic JSON serialization
json.dumps(user.__dict__)
```

```go
// Go
type User struct {
    ID   int    `json:"id"`
    Name string `json:"name"`
}

// Explicit but clear
data, _ := json.Marshal(user)
```

**3. Channel-based concurrency**
```python
# Python
from concurrent.futures import ThreadPoolExecutor

with ThreadPoolExecutor() as executor:
    futures = [executor.submit(process, item) for item in items]
```

```go
// Go
ch := make(chan Result)
for _, item := range items {
    go func(it Item) {
        ch <- process(it)
    }(item)
}
```

---

## 🎯 Decision Matrix: When to Use What

### Use Python When...
- ✅ Rapid prototyping (hours to working code)
- ✅ Data science / ML / scripting
- ✅ Team is Python-focused
- ✅ Integration with Python ecosystem
- ✅ Development speed > execution speed
- ✅ Code clarity is paramount

### Use Rust When...
- ✅ Maximum performance required (game engines, browsers)
- ✅ Memory safety is critical (embedded systems, OS)
- ✅ Zero runtime overhead needed
- ✅ Concurrent programming without data races
- ✅ Execution speed > development speed
- ✅ Long-term maintenance (compiler catches bugs)

### Use Go When...
- ✅ Building web services / APIs / microservices
- ✅ Team productivity + good performance
- ✅ Fast iteration (2-second builds)
- ✅ Simple concurrent operations
- ✅ Cloud-native applications
- ✅ Balance of simplicity and speed

---

## 📚 Learning Resources

### Rust for Python Developers
1. 🎓 [Rust Book](https://doc.rust-lang.org/book/) - Chapters 1-10 essential
2. 🏃 [Rustlings](https://github.com/rust-lang/rustlings) - Interactive exercises
3. 📖 [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
4. 💡 Focus on: Ownership (Ch 4), Enums (Ch 6), Error Handling (Ch 9)

### Go for Python Developers
1. 🎓 [Tour of Go](https://go.dev/tour/) - Complete in 2-3 hours
2. 📖 [Effective Go](https://go.dev/doc/effective_go) - Idiomatic patterns
3. 🏃 [Go by Example](https://gobyexample.com/) - Practical snippets
4. 💡 Focus on: Goroutines, Channels, Interfaces, Error handling

---

## 💡 Final Wisdom

**From This Project, You've Learned:**

1. **Same problem, three solutions** - Each language offers different trade-offs
2. **Type systems matter** - Static typing catches bugs earlier but requires more upfront design
3. **Concurrency models differ** - GIL vs Ownership vs Goroutines
4. **Simplicity is intentional** - We avoided verbose error handling to focus on core concepts
5. **Performance comes with complexity** - Rust is fastest but most complex, Python is slowest but simplest

**Remember:**
- 🐍 Python: "Simple is better than complex"
- 🦀 Rust: "Safety and speed without compromise"
- 🐹 Go: "Less is exponentially more"

