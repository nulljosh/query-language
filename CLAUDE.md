# Query Language - Dev Notes

## Project Overview

SQL-like query engine in Rust. 803 LOC (target ~700).

**Location**: `~/Documents/Code/query-language/`

## File Structure

```
query-language/
├── Cargo.toml              # Project manifest
├── src/
│   ├── lib.rs              # Core engine (553 LOC)
│   └── main.rs             # Demo & tests (250 LOC)
├── README.md               # User-facing documentation
└── CLAUDE.md               # This file
```

## Code Breakdown

| Component | LOC | Purpose |
|-----------|-----|---------|
| AST Types | 45 | Value, Row, Table, Query, Expr |
| Lexer | 70 | Tokenization |
| Parser | 250 | Recursive descent parser |
| Executor | 180 | Query execution engine |
| Demo/Tests | 258 | 8 demo queries + 8 unit tests |
| **Total** | **803** | ~700 target |

## Architecture Deep Dive

### Stage 1: Lexer (Tokenization)
**File**: `src/lib.rs` lines ~45-85

Converts raw SQL string into tokens.

Algorithm:
- Scan character by character
- Recognize patterns: whitespace (skip), operators, keywords (case-insensitive), identifiers, literals, punctuation
- Time: O(n) where n = input length

### Stage 2: Parser (AST Construction)
**File**: `src/lib.rs` lines ~85-280

Recursive descent parser. Precedence (lowest to highest):
1. OR
2. AND
3. Comparison (=, !=, <, >, <=, >=)
4. Primary (literals, columns, parentheses)

Parser Methods:
- `parse_query()` - Entry point, handles full SELECT
- `parse_select_list()` - SELECT clause
- `parse_column_list()` - GROUP BY, ORDER BY columns
- `parse_expr()` - Full expression (delegates to precedence)
- `parse_or_expr()` - OR operator (lowest precedence)
- `parse_and_expr()` - AND operator
- `parse_comparison()` - Comparison operators
- `parse_primary()` - Literals, columns, parentheses

Time: O(m) where m = token count

### Stage 3: Executor (Query Evaluation)
**File**: `src/lib.rs` lines ~280-450

Execution pipeline:
1. Load table rows from FROM clause - O(n) copy
2. WHERE filter - O(n × e) where e = expression complexity
3. JOIN - O(n × m × e) cartesian product with ON filtering
4. GROUP BY - O(n × k) where k = GROUP BY columns
5. ORDER BY - O(n log n) sort
6. SELECT projection - O(n × c) where c = column count
7. LIMIT - O(1) truncation

Expression Evaluation:
```rust
match expr {
    Column(name) => row.get(name),
    Literal(val) => val,
    BinOp(left, op, right) => {
        lval = eval_expr(left, row)
        rval = eval_expr(right, row)
        apply_op(lval, op, rval)
    }
}
```

## Test Coverage

8 unit tests in `main.rs`:
- `test_parse_select` - Basic parsing
- `test_parse_where` - WHERE clause parsing
- `test_parse_join` - JOIN parsing
- `test_parse_group_by` - GROUP BY parsing
- `test_parse_order_by` - ORDER BY parsing
- `test_parse_limit` - LIMIT parsing
- `test_execute_select` - Basic execution
- `test_execute_where` - WHERE filtering
- `test_execute_order_by` - ORDER BY sorting
- `test_execute_limit` - LIMIT truncation
- `test_execute_join` - JOIN execution

Sample data:
- `users` table (4 rows: Alice, Bob, Carol, David)
- `orders` table (4 rows with amounts and user IDs)

## Demo Queries

All included in `main.rs`:

1. `SELECT id, name FROM users`
2. `SELECT name, age FROM users WHERE age > 28`
3. `SELECT name, age FROM users ORDER BY age DESC`
4. `SELECT name FROM users LIMIT 2`
5. `SELECT users.name, orders.amount FROM users JOIN orders ON users.id = orders.user_id`
6. `SELECT users.name, orders.amount FROM users JOIN orders ON ... WHERE orders.amount > 400`
7. `SELECT * FROM users`
8. `SELECT name, age FROM users WHERE age > 25 AND dept = 'Sales'`

## Design Decisions

1. **Lexer before Parser** - Simpler character scanning, clear token boundaries
2. **Recursive Descent Parser** - No external libraries, easy to modify, good for SQL's shallow nesting
3. **In-Memory Tables** - Simplifies execution, matches demo use case
4. **Pipeline Architecture** - Independent steps, easy to debug, natural optimization points
5. **No Indexes** - Simpler, fine for small datasets, future optimization path

## Extension Points

### Add New Operators
**Where**: `tokenize()`, `apply_binop()`
```rust
// In tokenize(): recognize new operator
// In apply_binop(): handle new operator
```

### Add Aggregate Functions
**Where**: `apply_group_by()`
```rust
// Instead of just grouping, compute SUM, COUNT, AVG, etc.
```

### Add DISTINCT
**Where**: Between SELECT and LIMIT
```rust
rows.sort();
rows.dedup_by(|a, b| same_values(a, b));
```

### Add Subqueries
**Where**: `parse_primary()` for nested SELECT
```rust
// Allow SELECT in FROM clause
// Treat result as a temporary table
```

### Add Indexes
**Where**: Add Index struct to Table
```rust
struct Index {
    column: String,
    values: BTreeMap<Value, Vec<RowId>>
}
// Use in WHERE evaluation: O(log n) lookup instead of O(n) scan
```

## Comparison to nullC Compiler

Both use similar techniques:

| Stage | Query Engine | Compiler |
|-------|--------------|----------|
| Input | SQL string | Source code |
| Lexer | tokenize() | Lexer |
| Parser | Recursive descent | Recursive descent |
| AST | Query + Expr | AST for program |
| Evaluation | eval_expr() on data | codegen/interpreter |
| Output | Rows | Binary/IR |

Key learning: Separation of parsing from execution applies to both domains.

## Performance Characteristics

| Operation | Time | Space | Notes |
|-----------|------|-------|-------|
| Parse | O(m) | O(m) | m = token count |
| Single table WHERE | O(n×e) | O(n) | n = rows, e = expr complexity |
| JOIN | O(n×m×e) | O(n×m) | Cartesian product |
| GROUP BY | O(n) | O(g) | g = unique groups |
| ORDER BY | O(n log n) | O(n) | Standard sort |
| SELECT | O(n) | O(n) | Projection copy |
| LIMIT | O(1) | O(1) | Truncate in-place |

## Future Optimization Ideas

1. Query rewriting: Reorder operations for speed
2. Predicate pushdown: Move WHERE down before JOIN
3. Lazy evaluation: Don't materialize intermediate results
4. Columnar storage: Better cache locality
5. Vectorized execution: Process batches, not rows
6. Parallel execution: Run independent operations in parallel

## Running

### Demo
```bash
cargo run
```

### Tests
```bash
cargo test
```

### Release Build
```bash
cargo build --release
# Binary at target/release/query-language
```

## Status

- Complete: All specified features implemented
- Tested: Unit + integration tests pass
- Documented: Architecture + usage examples
- Production-ready pattern: Clean architecture, extensible design
- Educational: Learn compiler/parser techniques

Can be used as:
- Standalone query engine for in-memory data
- Template for building domain-specific languages (DSLs)
- Foundation for a real SQL implementation
- Reference implementation for parser + executor
