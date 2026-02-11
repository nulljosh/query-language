# Design & Architecture

## Overview

The query engine is a three-stage pipeline:

```
Input SQL String → Parser → Executor → Result Rows
```

## Stage 1: Lexer (Tokenization)

**File**: `src/lib.rs` lines ~45-85

### Purpose
Convert raw SQL string into discrete tokens (keywords, identifiers, operators, literals).

### Algorithm
```rust
fn tokenize(input: &str) -> Vec<Token>
```

- Scan input character by character
- Recognize patterns:
  - **Whitespace**: Skip
  - **Operators** (`=`, `<`, `>`, etc.): Recognize comparison ops
  - **Keywords** (`SELECT`, `FROM`, etc.): Case-insensitive
  - **Identifiers** (table/column names): Alphanumeric + underscore
  - **Literals**: Numbers (int/float) and strings (quoted)
  - **Punctuation**: `(`, `)`, `,`, `*`

### Complexity
- **Time**: O(n) where n = input length
- **Space**: O(n) for token list

### Example
```
Input:  "SELECT id, name FROM users WHERE age > 30"
Output: [
    Token::Select,
    Token::Ident("id"),
    Token::Comma,
    Token::Ident("name"),
    Token::From,
    Token::Ident("users"),
    Token::Where,
    Token::Ident("age"),
    Token::Op(">"),
    Token::Number("30")
]
```

## Stage 2: Parser (AST Construction)

**File**: `src/lib.rs` lines ~85-280

### Purpose
Convert token stream into Abstract Syntax Tree (AST).

### Parser Type
**Recursive Descent** - Simplest parser, perfect for SQL's shallow structure.

### Grammar (Approximate)

```ebnf
Query      := SELECT SelectList FROM Table [Join]* [WHERE Expr]? 
              [GROUP BY ColList]? [ORDER BY OrderList]? [LIMIT Num]?

SelectList := "*" | Expr ("," Expr)*

Join       := JOIN Table ON Expr

Expr       := OrExpr
OrExpr     := AndExpr ("OR" AndExpr)*
AndExpr    := Comparison ("AND" Comparison)*
Comparison := Primary CompOp Primary
Primary    := Column | Number | String | "(" Expr ")"

Table      := Identifier
Column     := Identifier
ColList    := Column ("," Column)*
OrderList  := Column [ASC|DESC]? ("," Column [ASC|DESC]?)*
```

### Parser Methods

| Method | Purpose |
|--------|---------|
| `parse_query()` | Entry point, handles full SELECT statement |
| `parse_select_list()` | SELECT clause (columns or *) |
| `parse_column_list()` | GROUP BY, ORDER BY column lists |
| `parse_expr()` | Full expression (delegates to precedence) |
| `parse_or_expr()` | OR operator (lowest precedence) |
| `parse_and_expr()` | AND operator |
| `parse_comparison()` | Comparison operators (=, <, >, etc.) |
| `parse_primary()` | Literals, columns, parenthesized expressions |

### Precedence (Lowest to Highest)
1. OR
2. AND  
3. Comparison (=, !=, <, >, <=, >=)
4. Primary (literals, columns, parentheses)

### Complexity
- **Time**: O(m) where m = token count
- **Space**: O(k) where k = AST depth (usually small)

### Example

Input tokens: `[Select, Ident("id"), From, Ident("users"), Where, Ident("age"), Op(">"), Number("30")]`

```
Query {
    select_cols: ["id"],
    from_table: "users",
    joins: [],
    where_clause: Some(BinOp(
        Column("age"),
        ">",
        Literal(Int(30))
    )),
    group_by: [],
    order_by: [],
    limit: None
}
```

## Stage 3: Executor (Query Evaluation)

**File**: `src/lib.rs` lines ~280-450

### Purpose
Evaluate Query AST against actual data tables.

### Execution Pipeline

```
1. Load table rows from FROM clause
   ↓
2. Apply WHERE filter (optional)
   ↓
3. Apply JOINs (optional, multiple)
   ↓
4. Apply GROUP BY (optional)
   ↓
5. Apply ORDER BY (optional)
   ↓
6. Project SELECT columns
   ↓
7. Apply LIMIT (optional)
   ↓
8. Return result rows
```

#### Step 1: Load Table
```rust
let mut rows = tables[from_table].rows.clone();
```
- **Time**: O(n) copy, where n = row count

#### Step 2: WHERE Filtering
```rust
rows.retain(|row| eval_expr(where_clause, row).is_true());
```
- Evaluate expression against each row
- Keep only rows where expression evaluates to `Bool(true)`
- **Time**: O(n × e) where e = expression complexity

#### Step 3: JOIN
```rust
for each right_row in join_table.rows {
    for each left_row in rows {
        merged = left_row + right_row
        if eval_expr(on_condition, merged) {
            result.push(merged)
        }
    }
}
```
- Cartesian product with ON filtering
- Merges columns from both tables
- **Time**: O(n × m × e) where n,m = row counts, e = condition complexity

#### Step 4: GROUP BY
```rust
// Group rows by column values
groups = HashMap<GroupKey, Vec<Row>>
for (key, group) in groups {
    result_row = { key columns... }
}
```
- Creates map of groups by column values
- One output row per distinct group
- **Note**: Currently just groups, doesn't aggregate (SUM, COUNT, etc.)
- **Time**: O(n × k) where k = GROUP BY columns

#### Step 5: ORDER BY
```rust
rows.sort_by(|a, b| compare(a[col], b[col]));
```
- Sorts rows by specified column(s)
- Supports multiple columns (sorts in order given)
- Supports ASC (ascending) and DESC (descending)
- **Time**: O(n log n) for sort

#### Step 6: SELECT Projection
```rust
for row in rows {
    new_row[col] = row[col] for col in select_columns
}
```
- Extracts only requested columns
- Supports `*` (all columns) or specific list
- **Time**: O(n × c) where c = column count

#### Step 7: LIMIT
```rust
rows.truncate(limit_count);
```
- Keeps only first N rows
- **Time**: O(1) for truncation

### Expression Evaluation

Central to execution: `eval_expr(expr, row) → Value`

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

**Supported Operations**:

| Op | Type | Example | Result |
|----|------|---------|--------|
| `=` | Comparison | `5 = 5` | `true` |
| `!=` | Comparison | `5 != 3` | `true` |
| `<` | Comparison | `3 < 5` | `true` |
| `>` | Comparison | `5 > 3` | `true` |
| `<=` | Comparison | `5 <= 5` | `true` |
| `>=` | Comparison | `5 >= 3` | `true` |
| `AND` | Boolean | `true AND false` | `false` |
| `OR` | Boolean | `true OR false` | `true` |

**Type Support**:
- Int-to-Int comparisons
- String-to-String equality
- Bool-to-Bool logic
- Mixed types → Null (no implicit conversion)

### Overall Complexity

For a complete query:

```
WHERE: O(n₁ × e₁)
JOIN:  O(n₁ × n₂ × e₂)
GROUP: O(n × k)
SORT:  O(n log n)
LIMIT: O(1)
Total: O(n₁ × n₂ × e) for join queries, O(n log n) for sort-heavy
```

Where:
- n₁, n₂ = table sizes
- e = expression complexity
- k = GROUP BY columns

## Data Model

### Value
```rust
enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}
```

Represents any cell value in the database.

### Row
```rust
struct Row {
    data: HashMap<String, Value>
}
```

A single row of data, column_name → value mapping.

### Table
```rust
struct Table {
    name: String,
    columns: Vec<String>,
    rows: Vec<Row>,
}
```

Complete table with metadata and data.

### Query
```rust
struct Query {
    select_cols: Vec<String>,
    from_table: String,
    joins: Vec<Join>,
    where_clause: Option<Expr>,
    group_by: Vec<String>,
    order_by: Vec<(String, bool)>,
    limit: Option<usize>,
}
```

Complete parsed query representation.

### Expr (Expression AST)
```rust
enum Expr {
    Column(String),
    Literal(Value),
    BinOp(Box<Expr>, String, Box<Expr>),
    FuncCall(String, Vec<Expr>), // prepared for future
}
```

Expression tree for WHERE, ON, ORDER BY.

## Design Decisions

### 1. Lexer before Parser
- Simpler character-by-character scanning
- Easier to add operator variants later
- Clear token boundaries

### 2. Recursive Descent Parser
- No external parser library needed
- Easy to understand and modify
- Good enough for SQL's shallow nesting
- Could switch to Precedence Climbing if needed

### 3. In-Memory Tables
- Simplifies execution (no disk I/O)
- Matches learning/demo use case
- Can be extended with persistence layer

### 4. Pipeline Architecture
- Each step is independent
- Easy to debug (output of each stage)
- Natural optimization points (e.g., push WHERE down)

### 5. No Indexes
- Simpler to understand
- Fine for small datasets
- Optimization path for future (B-trees, hash indexes)

## Extension Points

### 1. Add New Operators
**Where**: `tokenize()`, `apply_binop()`
```rust
// In tokenize(): recognize new operator
// In apply_binop(): handle new operator
```

### 2. Add Aggregate Functions
**Where**: `apply_group_by()`
```rust
// Instead of just grouping, compute SUM, COUNT, AVG, etc.
// Track aggregates during GROUP BY
```

### 3. Add DISTINCT
**Where**: Between SELECT and LIMIT
```rust
rows.sort();
rows.dedup_by(|a, b| same_values(a, b));
```

### 4. Add Subqueries
**Where**: `parse_primary()` for nested SELECT
```rust
// Allow SELECT in FROM clause
// Treat result as a temporary table
```

### 5. Add Indexes
**Where**: Add Index struct to Table
```rust
struct Index {
    column: String,
    values: BTreeMap<Value, Vec<RowId>>
}
// Use in WHERE evaluation: O(log n) lookup instead of O(n) scan
```

## Testing Strategy

### Unit Tests (in main.rs)
- Test each stage independently
- Parsing tests: verify AST structure
- Execution tests: verify results
- Edge cases: empty results, LIMIT beyond data, etc.

### Integration Tests
- Run complete queries end-to-end
- Verify interaction between stages
- Use realistic sample data

### Coverage
- All operators
- All clauses
- Empty results
- NULL handling
- Type mismatches

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

Key learning: **Separation of parsing from execution** applies to both domains.

## Future Optimization Ideas

1. **Query rewriting**: Reorder operations for speed
2. **Predicate pushdown**: Move WHERE down before JOIN
3. **Lazy evaluation**: Don't materialize intermediate results
4. **Columnar storage**: Better cache locality
5. **Vectorized execution**: Process batches, not rows
6. **Parallel execution**: Run independent operations in parallel
