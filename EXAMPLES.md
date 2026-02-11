# Usage Examples

## Building Queries Programmatically

### 1. Simple SELECT

```rust
use query_language::*;
use std::collections::HashMap;

fn main() {
    let mut db = Database::new();
    
    // Create a table
    let table = Table {
        name: "users".into(),
        columns: vec!["id".into(), "name".into()],
        rows: vec![
            // Row 1: id=1, name="Alice"
            {
                let mut data = HashMap::new();
                data.insert("id".into(), Value::Int(1));
                data.insert("name".into(), Value::String("Alice".into()));
                Row { data }
            },
            // Row 2: id=2, name="Bob"
            {
                let mut data = HashMap::new();
                data.insert("id".into(), Value::Int(2));
                data.insert("name".into(), Value::String("Bob".into()));
                Row { data }
            },
        ],
    };
    
    db.add_table(table);
    
    // Parse and execute query
    let query = parse("SELECT * FROM users").unwrap();
    let results = db.execute(&query).unwrap();
    
    // Print results
    for row in results {
        println!("{:?}", row.data);
    }
}
```

### 2. WHERE Clause

```rust
// SELECT name FROM users WHERE age > 25
let query = parse("SELECT name FROM users WHERE age > 25").unwrap();
let results = db.execute(&query).unwrap();
```

### 3. JOIN

```rust
// SELECT u.name, o.amount FROM users u 
// JOIN orders o ON u.id = o.user_id
let query = parse(
    "SELECT users.name, orders.amount FROM users \
     JOIN orders ON users.id = orders.user_id"
).unwrap();
let results = db.execute(&query).unwrap();
```

### 4. ORDER BY

```rust
// SELECT * FROM users ORDER BY age DESC LIMIT 10
let query = parse(
    "SELECT * FROM users ORDER BY age DESC LIMIT 10"
).unwrap();
let results = db.execute(&query).unwrap();
```

### 5. GROUP BY

```rust
// SELECT dept FROM users GROUP BY dept
let query = parse("SELECT dept FROM users GROUP BY dept").unwrap();
let results = db.execute(&query).unwrap();
```

## Parsing Examples

The parser is lenient with whitespace and case-insensitive:

```rust
// All equivalent:
parse("SELECT id FROM users")?;
parse("select id from users")?;
parse("SELECT id FROM users")?;
parse("SELECT   id   FROM   users")?;
```

## Query Structure

After parsing, you can inspect the Query AST:

```rust
let query = parse("SELECT name, age FROM users WHERE age > 30").unwrap();

assert_eq!(query.select_cols, vec!["name", "age"]);
assert_eq!(query.from_table, "users");
assert!(query.where_clause.is_some());
assert_eq!(query.group_by, vec![]); // Empty if not specified
assert_eq!(query.order_by, vec![]); // Empty if not specified
assert_eq!(query.limit, None);      // None if not specified
```

## Expression Evaluation

Expressions are evaluated against row data:

```rust
// Create expression: age > 30
let expr = Expr::BinOp(
    Box::new(Expr::Column("age".into())),
    ">".into(),
    Box::new(Expr::Literal(Value::Int(30)))
);

// Evaluates true or false depending on row data
```

Supported operators:
- Comparison: `=`, `!=`, `<`, `>`, `<=`, `>=`
- Boolean: `AND`, `OR`

## Error Handling

Parse errors return descriptive messages:

```rust
match parse("INVALID SQL") {
    Ok(query) => println!("Parsed: {:?}", query),
    Err(e) => eprintln!("Parse error: {}", e),
}
```

Execute errors (table not found, etc.):

```rust
match db.execute(&query) {
    Ok(rows) => println!("Got {} rows", rows.len()),
    Err(e) => eprintln!("Execute error: {}", e),
}
```

## Data Types

Create values for different types:

```rust
let int_val = Value::Int(42);
let float_val = Value::Float(3.14);
let string_val = Value::String("hello".into());
let bool_val = Value::Bool(true);
let null_val = Value::Null;
```

## Building Rows Efficiently

Helper function for cleaner code:

```rust
fn make_row(pairs: &[(&str, Value)]) -> Row {
    let mut data = HashMap::new();
    for (k, v) in pairs {
        data.insert(k.to_string(), v.clone());
    }
    Row { data }
}

// Usage:
let row = make_row(&[
    ("id", Value::Int(1)),
    ("name", Value::String("Alice".into())),
]);
```

## Complete Example: Multi-Table Analysis

```rust
use query_language::*;
use std::collections::HashMap;

fn main() {
    let mut db = Database::new();
    
    // Setup tables
    setup_tables(&mut db);
    
    // Run analysis queries
    println!("=== Sales by Department ===");
    run_query(&db, 
        "SELECT users.dept, orders.amount FROM users \
         JOIN orders ON users.id = orders.user_id \
         WHERE orders.amount > 500");
    
    println!("\n=== Top 5 Customers ===");
    run_query(&db,
        "SELECT name, age FROM users \
         ORDER BY age DESC LIMIT 5");
    
    println!("\n=== Young Workers ===");
    run_query(&db,
        "SELECT name FROM users \
         WHERE age < 30 AND dept = 'Engineering'");
}

fn setup_tables(db: &mut Database) {
    // Create users table
    // Create orders table
    // ... populate with data
}

fn run_query(db: &Database, sql: &str) {
    match parse(sql) {
        Ok(query) => {
            match db.execute(&query) {
                Ok(rows) => {
                    for row in rows {
                        println!("{:?}", row.data);
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}
```

## Testing Custom Queries

The test framework is ready for custom queries:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_query() {
        let mut db = Database::new();
        // Setup your table...
        
        let q = parse("SELECT * FROM my_table").unwrap();
        let result = db.execute(&q).unwrap();
        
        assert_eq!(result.len(), 5); // Your assertion
    }
}
```

Run with:
```bash
cargo test test_custom_query
```

## Performance Tips

1. **Queries with WHERE are faster** - Reduces rows early
2. **ORDER BY on indexed columns** - Would be faster with indexes (not yet implemented)
3. **LIMIT after WHERE** - Reduces result set
4. **JOIN on exact matches** - ON expression is evaluated for every row pair

## Extending the Engine

To add new features:

1. **New operators**: Add to `tokenize()` and `apply_binop()`
2. **New aggregate functions**: Modify `apply_group_by()` 
3. **DISTINCT**: Filter unique rows before LIMIT
4. **Subqueries**: Nest Query in FROM clause
5. **Indexes**: Cache sorted/hashed column data

See the README for more extension ideas.
