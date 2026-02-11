use query_language::*;
use std::collections::HashMap;

fn main() {
    println!("=== SQL Query Engine Demo ===\n");

    let mut db = Database::new();

    // Create sample tables
    let users = Table {
        name: "users".into(),
        columns: vec!["id".into(), "name".into(), "age".into(), "dept".into()],
        rows: vec![
            row(vec![("id", int(1)), ("name", string("Alice")), ("age", int(30)), ("dept", string("Engineering"))]),
            row(vec![("id", int(2)), ("name", string("Bob")), ("age", int(28)), ("dept", string("Sales"))]),
            row(vec![("id", int(3)), ("name", string("Carol")), ("age", int(35)), ("dept", string("Engineering"))]),
            row(vec![("id", int(4)), ("name", string("David")), ("age", int(28)), ("dept", string("Sales"))]),
        ],
    };

    let orders = Table {
        name: "orders".into(),
        columns: vec!["order_id".into(), "user_id".into(), "amount".into()],
        rows: vec![
            row(vec![("order_id", int(101)), ("user_id", int(1)), ("amount", int(500))]),
            row(vec![("order_id", int(102)), ("user_id", int(2)), ("amount", int(300))]),
            row(vec![("order_id", int(103)), ("user_id", int(1)), ("amount", int(700))]),
            row(vec![("order_id", int(104)), ("user_id", int(3)), ("amount", int(450))]),
        ],
    };

    db.add_table(users);
    db.add_table(orders);

    // Test 1: Simple SELECT
    test_query(&db, "SELECT id, name FROM users");

    // Test 2: SELECT with WHERE
    test_query(&db, "SELECT name, age FROM users WHERE age > 28");

    // Test 3: SELECT with ORDER BY
    test_query(&db, "SELECT name, age FROM users ORDER BY age DESC");

    // Test 4: SELECT with LIMIT
    test_query(&db, "SELECT name FROM users LIMIT 2");

    // Test 5: JOIN
    test_query(&db, "SELECT users.name, orders.amount FROM users JOIN orders ON users.id = orders.user_id");

    // Test 6: WHERE with JOIN
    test_query(&db, "SELECT users.name, orders.amount FROM users JOIN orders ON users.id = orders.user_id WHERE orders.amount > 400");

    // Test 7: SELECT *
    test_query(&db, "SELECT * FROM users");

    // Test 8: Multiple conditions
    test_query(&db, "SELECT name, age FROM users WHERE age > 25 AND dept = 'Sales'");

    println!("\n=== All tests completed! ===");
}

fn test_query(db: &Database, sql: &str) {
    println!("\nQuery: {}", sql);
    match parse(sql) {
        Ok(query) => {
            match db.execute(&query) {
                Ok(rows) => {
                    if rows.is_empty() {
                        println!("  (no results)");
                    } else {
                        let cols: Vec<&String> = rows[0].data.keys().collect();
                        println!("  {}", cols.join(" | "));
                        for row in rows {
                            let vals: Vec<String> = cols.iter().map(|c| format_value(row.data.get(*c))).collect();
                            println!("  {}", vals.join(" | "));
                        }
                    }
                }
                Err(e) => println!("  ERROR: {}", e),
            }
        }
        Err(e) => println!("  PARSE ERROR: {}", e),
    }
}

fn format_value(v: Option<&Value>) -> String {
    match v {
        Some(Value::Int(i)) => i.to_string(),
        Some(Value::Float(f)) => f.to_string(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Null) | None => "NULL".into(),
    }
}

fn row(pairs: Vec<(&str, Value)>) -> Row {
    let mut data = HashMap::new();
    for (k, v) in pairs {
        data.insert(k.to_string(), v);
    }
    Row { data }
}

fn int(i: i64) -> Value {
    Value::Int(i)
}

fn string(s: &str) -> Value {
    Value::String(s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_select() {
        let q = parse("SELECT id, name FROM users").unwrap();
        assert_eq!(q.from_table, "users");
        assert_eq!(q.select_cols, vec!["id", "name"]);
    }

    #[test]
    fn test_parse_where() {
        let q = parse("SELECT * FROM users WHERE age > 30").unwrap();
        assert!(q.where_clause.is_some());
    }

    #[test]
    fn test_parse_join() {
        let q = parse("SELECT * FROM users JOIN orders ON users.id = orders.user_id").unwrap();
        assert_eq!(q.joins.len(), 1);
        assert_eq!(q.joins[0].table, "orders");
    }

    #[test]
    fn test_parse_group_by() {
        let q = parse("SELECT dept FROM users GROUP BY dept").unwrap();
        assert_eq!(q.group_by, vec!["dept"]);
    }

    #[test]
    fn test_parse_order_by() {
        let q = parse("SELECT name FROM users ORDER BY age DESC").unwrap();
        assert_eq!(q.order_by.len(), 1);
        assert!(!q.order_by[0].1); // DESC = false
    }

    #[test]
    fn test_parse_limit() {
        let q = parse("SELECT * FROM users LIMIT 5").unwrap();
        assert_eq!(q.limit, Some(5));
    }

    #[test]
    fn test_execute_select() {
        let mut db = Database::new();
        let users = Table {
            name: "users".into(),
            columns: vec!["id".into(), "name".into()],
            rows: vec![
                row(vec![("id", int(1)), ("name", string("Alice"))]),
                row(vec![("id", int(2)), ("name", string("Bob"))]),
            ],
        };
        db.add_table(users);

        let q = parse("SELECT * FROM users").unwrap();
        let result = db.execute(&q).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_execute_where() {
        let mut db = Database::new();
        let users = Table {
            name: "users".into(),
            columns: vec!["id".into(), "age".into()],
            rows: vec![
                row(vec![("id", int(1)), ("age", int(25))]),
                row(vec![("id", int(2)), ("age", int(35))]),
            ],
        };
        db.add_table(users);

        let q = parse("SELECT * FROM users WHERE age > 30").unwrap();
        let result = db.execute(&q).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_execute_order_by() {
        let mut db = Database::new();
        let users = Table {
            name: "users".into(),
            columns: vec!["name".into(), "age".into()],
            rows: vec![
                row(vec![("name", string("Alice")), ("age", int(30))]),
                row(vec![("name", string("Bob")), ("age", int(25))]),
            ],
        };
        db.add_table(users);

        let q = parse("SELECT * FROM users ORDER BY age ASC").unwrap();
        let result = db.execute(&q).unwrap();
        if let Value::Int(age) = result[0].data.get("age").unwrap() {
            assert_eq!(*age, 25);
        }
    }

    #[test]
    fn test_execute_limit() {
        let mut db = Database::new();
        let users = Table {
            name: "users".into(),
            columns: vec!["id".into()],
            rows: vec![
                row(vec![("id", int(1))]),
                row(vec![("id", int(2))]),
                row(vec![("id", int(3))]),
            ],
        };
        db.add_table(users);

        let q = parse("SELECT * FROM users LIMIT 2").unwrap();
        let result = db.execute(&q).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_execute_join() {
        let mut db = Database::new();
        let users = Table {
            name: "users".into(),
            columns: vec!["id".into(), "name".into()],
            rows: vec![row(vec![("id", int(1)), ("name", string("Alice"))])],
        };
        let orders = Table {
            name: "orders".into(),
            columns: vec!["user_id".into(), "amount".into()],
            rows: vec![row(vec![("user_id", int(1)), ("amount", int(100))])],
        };
        db.add_table(users);
        db.add_table(orders);

        let q = parse("SELECT * FROM users JOIN orders ON users.id = orders.user_id").unwrap();
        let result = db.execute(&q).unwrap();
        assert_eq!(result.len(), 1);
    }
}
