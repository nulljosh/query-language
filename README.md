# SQL-Like Query Engine

A lightweight, in-memory SQL query engine written in Rust. ~700 LOC with full support for SELECT, WHERE, JOIN, GROUP BY, ORDER BY, and LIMIT.

## Features

### Core Capabilities
- **Parser**: Tokenizes and parses SQL statements into an AST
- **Executor**: Executes queries against in-memory tables
- **Optimizer**: Simple query planning and optimization
- **Supported Operations**:
  - `SELECT` with column projection and `*` wildcard
  - `WHERE` with comparison operators and boolean logic (AND/OR)
  - `JOIN` with ON conditions
  - `GROUP BY` for row aggregation
  - `ORDER BY` with ASC/DESC sorting
  - `LIMIT` for result truncation

### Data Types
- `Int` (i64)
- `Float` (f64)
- `String`
- `Bool`
- `Null`

## Architecture

### Lexer (`tokenize`)
Converts raw SQL string into tokens (keywords, identifiers, operators, literals).

**Time:** O(n) where n = input length

### Parser (`parse`)
Recursive descent parser that builds an AST from tokens.

- `parse_query()` - Top-level SELECT statement
- `parse_select_list()` - Column names or `*`
- `parse_column_list()` - List of columns (GROUP BY, ORDER BY)
- `parse_expr()` - Expressions with operator precedence
  - OR expressions → AND expressions → Comparisons → Primaries

**Time:** O(m) where m = token count

### Executor (`Database::execute`)
Evaluates queries step-by-step:

1. **FROM**: Load base table rows
2. **WHERE**: Filter with expression evaluation
3. **JOIN**: Cartesian product with ON condition
4. **GROUP BY**: Group rows by column values
5. **ORDER BY**: Sort rows
6. **SELECT**: Project to selected columns
7. **LIMIT**: Truncate result set

**Time:** O(n²) for JOINs (cartesian product), O(n log n) for sorting

## Usage

### Building
```bash
cargo build --release
```

### Running Demo
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

### Running Tests
```bash
cargo test
```

Tests cover:
- Parsing all SQL constructs
- WHERE filtering
- ORDER BY sorting
- LIMIT truncation
- JOIN operations
- Complex WHERE conditions

## API

### Database
```rust
let mut db = Database::new();
db.add_table(table);
let result = db.execute(&query)?;
```

### Parsing
```rust
let query = parse("SELECT * FROM users WHERE age > 30")?;
```

### Building Tables Manually
```rust
let mut table = Table {
    name: "users".into(),
    columns: vec!["id".into(), "name".into()],
    rows: vec![]
};

// Add rows with Row { data: HashMap<String, Value> }
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
SELECT u.name, o.amount FROM users u 
JOIN orders o ON u.id = o.user_id;

-- Complex WHERE
SELECT name FROM users 
WHERE age > 25 AND dept = 'Engineering';

-- GROUP BY
SELECT dept FROM users GROUP BY dept;
```

## Performance Notes

- **In-memory**: All data loaded into RAM
- **Index-free**: Full table scans for WHERE/JOIN
- **No aggregates**: GROUP BY only groups, doesn't aggregate (SUM, COUNT, etc.)
- **No subqueries**: Single SELECT only
- **Single table base**: Queries start with one FROM table

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

- Total LOC: ~700
- Main AST/Types: ~50 LOC
- Lexer: ~70 LOC
- Parser: ~250 LOC
- Executor: ~180 LOC
- Main/Tests: ~150 LOC

## Pairs With

This query engine uses similar parser techniques to the **nullC compiler**:
- Token-based lexing
- Recursive descent parsing
- Expression precedence handling
- AST representation
