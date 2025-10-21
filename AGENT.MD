
# AGENT.MD

## GOAL
Build an **educational lightweight SQL database engine** (like SQLite) **from scratch** in Python.

The focus is **learning** how databases work internally, not creating a production system.
Code should remain **simple**, **minimal**, and **clear**.

---

## CONTEXT
We want a small Python project that demonstrates:

- A **storage engine** using either a **B-Tree**, **B+ Tree**, or **LSM Tree**.
- A **query processor** capable of parsing and executing a subset of SQL commands.
- A **basic SQL shell** for console interaction.
- A **minimal web UI** for submitting SQL and showing results.

This project will serve as a learning tool for database internals and data structure design.

---

## REQUIREMENTS

### 1. Supported SQL Commands
Only the most fundamental SQL operations are needed:

```

SELECT        -- extracts data from a database
UPDATE        -- updates data in a database
DELETE        -- deletes data from a database
INSERT INTO   -- inserts new data into a database
CREATE DATABASE
ALTER DATABASE
CREATE TABLE
ALTER TABLE
DROP TABLE
CREATE INDEX
DROP INDEX
COMMIT

```

- `SELECT` supports only **INNER JOIN**.
- No advanced SQL or error handling required.

---

### 2. Components to Implement

#### A. Core Engine
- **Parser**: Tokenize and parse simple SQL strings into AST-like objects.
- **Executor**: Execute parsed commands by interacting with storage.
- **Storage Engine**: Implement B-Tree/B+ Tree or LSM Tree for data persistence.
- **Pager / Disk Manager**: Handle reading/writing pages (simulate persistence).

#### B. Console Interface
- Simple REPL (`db> SELECT * FROM users;`)
- Text-based interaction

#### C. Web Interface
- Simple HTML + minimal Python backend (Flask or http.server)
- Input box for SQL query
- Display table-like query results

---

## IMPLEMENTATION GUIDELINES

- Keep functions and classes small and focused.
- Use **Python lists, dicts, or simple classes** to simulate internal structures.
- Avoid complex error handling, logging, or optimizations.
- Each module should contain **docstrings** explaining its role.
- Include **stub classes and methods** to show where logic will go.

---

## OUTPUT EXPECTATION

**Generate the initial project scaffolding**:
- Create empty or minimal code for each file.
- Include comments or docstrings explaining purpose and next steps.
- The project should be runnable and importable (e.g., `python main.py` launches CLI or web UI).

---

## STYLE
- Educational tone, clear naming, concise comments.
- Consistent Python 3.11+ syntax.
- Use `if __name__ == "__main__":` for entry points.

---

## EXAMPLE OUTPUT SNIPPET

```python
# core/parser.py
"""
Simple SQL parser.
Converts SQL text into a minimal internal representation (AST-like).
"""
class SQLParser:
    def parse(self, query: str):
        # TODO: Implement minimal SQL parsing
        return {"command": "SELECT", "table": "users"}
````

---

## SUMMARY

This project demonstrates how a minimal database engine can be built from scratch, focusing on:

* **Data structures (B-Tree / LSM Tree)**
* **Query execution pipeline**
* **Simple I/O persistence**
* **Educational clarity**

```


