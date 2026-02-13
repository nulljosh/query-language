# SQL Query Engine

Lightweight in-memory SQL query engine in Rust. 800 LOC with full support for SELECT, WHERE, JOIN, GROUP BY, ORDER BY, and LIMIT.

## Features

- SELECT with column projection and `*` wildcard
- WHERE with comparisons (=, !=, <, >, <=, >=) and boolean logic (AND/OR)
- JOIN with ON conditions
- GROUP BY for grouping rows
- ORDER BY with ASC/DESC
- LIMIT for result truncation

## Data Types

- Int (i64)
- Float (f64)
- String
- Bool
- Null

## Quick Start

### Build
```bash
cargo build --release
```

### Run Demo
```bash
cargo run
```

Sample output:
```
Query: SELECT name, age FROM users WHERE age > 28
  name | age
  Carol | 35
  David | 28
```

### Run Tests
```bash
cargo test
```

## Usage

### Parse and Execute
```rust
use query_language::*;
use std::collections::HashMap;

let mut db = Database::new();

// Create table
let table = Table {
    name: "users".into(),
    columns: vec!["id".into(), "name".into()],
    rows: vec![
        {
            let mut data = HashMap::new();
            data.insert("id".into(), Value::Int(1));
            data.insert("name".into(), Value::String("Alice".into()));
            Row { data }
        },
    ],
};

db.add_table(table);

// Parse and execute
let query = parse("SELECT * FROM users").unwrap();
let results = db.execute(&query).unwrap();

for row in results {
    println!("{:?}", row.data);
}
```

## Example Queries

```sql
-- Select all
SELECT * FROM users;

-- With WHERE
SELECT name, age FROM users WHERE age > 25;

-- With ORDER BY
SELECT name FROM users ORDER BY age DESC;

-- With LIMIT
SELECT * FROM users LIMIT 10;

-- JOIN
SELECT users.name, orders.amount FROM users
JOIN orders ON users.id = orders.user_id;

-- Complex WHERE
SELECT name FROM users
WHERE age > 25 AND dept = 'Engineering';

-- GROUP BY
SELECT dept FROM users GROUP BY dept;
```

## Architecture

### Pipeline
```
SQL String
    ↓ tokenize()
Tokens [SELECT, FROM, WHERE, ...]
    ↓ Parser::parse_query()
Query AST
    ↓ Database::execute()
Filtered & Transformed Rows
    ↓
Result Vec<Row>
```

### Execution Steps
1. FROM: Load base table rows
2. WHERE: Filter with expression evaluation
3. JOIN: Cartesian product with ON condition
4. GROUP BY: Group rows by column values
5. ORDER BY: Sort rows
6. SELECT: Project to selected columns
7. LIMIT: Truncate result set

### Parser (Recursive Descent)
- parse_query() - Top-level SELECT statement
- parse_select_list() - Column names or `*`
- parse_column_list() - List of columns (GROUP BY, ORDER BY)
- parse_expr() - Expressions with operator precedence
  - OR expressions → AND expressions → Comparisons → Primaries

## Performance

- Time: O(n²) for JOINs (cartesian product), O(n log n) for sorting
- Space: In-memory, no indexes (full table scans)

## Limitations

- No aggregate functions (SUM, COUNT, AVG) - GROUP BY only groups
- No subqueries or CTEs
- No DISTINCT
- No indexes (full table scans)
- Single FROM table base only

## Extension Ideas

1. Aggregate functions (SUM, COUNT, AVG, MIN, MAX)
2. Subqueries and CTEs
3. DISTINCT
4. INNER/LEFT/RIGHT/FULL JOINs
5. Indexes for WHERE optimization
6. UNION / INTERSECT / EXCEPT
7. Window functions
8. Prepared statements

## Code Statistics

- Total: ~800 LOC
- Lexer: 70 LOC
- Parser: 250 LOC
- Executor: 180 LOC
- Tests: 258 LOC

## Pairs With

Uses similar parser techniques to the **nullC compiler**:
- Token-based lexing
- Recursive descent parsing
- Expression precedence handling
- AST representation
