use std::collections::{HashMap, HashSet};
use regex::Regex;

// ============================================================================
// AST & Types
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone)]
pub struct Row {
    pub data: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub columns: Vec<String>,
    pub rows: Vec<Row>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Column(String),
    Literal(Value),
    BinOp(Box<Expr>, String, Box<Expr>), // expr, op, expr
    FuncCall(String, Vec<Expr>),           // func_name, args
}

#[derive(Debug, Clone)]
pub struct Join {
    pub table: String,
    pub on: Expr,
}

#[derive(Debug, Clone)]
pub struct Query {
    pub select_cols: Vec<String>,
    pub from_table: String,
    pub joins: Vec<Join>,
    pub where_clause: Option<Expr>,
    pub group_by: Vec<String>,
    pub order_by: Vec<(String, bool)>, // (col, is_asc)
    pub limit: Option<usize>,
}

// ============================================================================
// Lexer
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Select,
    From,
    Where,
    Join,
    On,
    GroupBy,
    OrderBy,
    Limit,
    And,
    Or,
    Asc,
    Desc,
    Comma,
    Star,
    LParen,
    RParen,
    Ident(String),
    Number(String),
    String(String),
    Op(String),
}

fn tokenize(input: &str) -> Vec<Token> {
    let input = input.to_uppercase();
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
        } else if ch == ',' {
            tokens.push(Token::Comma);
            chars.next();
        } else if ch == '*' {
            tokens.push(Token::Star);
            chars.next();
        } else if ch == '(' {
            tokens.push(Token::LParen);
            chars.next();
        } else if ch == ')' {
            tokens.push(Token::RParen);
            chars.next();
        } else if ch == '\'' {
            chars.next();
            let s: String = chars.by_ref().take_while(|&c| c != '\'').collect();
            chars.next();
            tokens.push(Token::String(s));
        } else if ch.is_numeric() {
            let num: String = chars.by_ref().take_while(|c| c.is_numeric() || *c == '.').collect();
            tokens.push(Token::Number(num));
        } else if ch.is_alphabetic() || ch == '_' {
            let word: String = chars.by_ref().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            let token = match word.as_str() {
                "SELECT" => Token::Select,
                "FROM" => Token::From,
                "WHERE" => Token::Where,
                "JOIN" => Token::Join,
                "ON" => Token::On,
                "GROUP" => Token::GroupBy,
                "ORDER" => Token::OrderBy,
                "LIMIT" => Token::Limit,
                "AND" => Token::And,
                "OR" => Token::Or,
                "ASC" => Token::Asc,
                "DESC" => Token::Desc,
                "BY" => Token::Ident("BY".into()),
                _ => Token::Ident(word),
            };
            tokens.push(token);
        } else if "=<>!".contains(ch) {
            let op: String = chars.by_ref().take_while(|&c| "=<>!".contains(c)).collect();
            tokens.push(Token::Op(op));
        } else {
            chars.next();
        }
    }
    tokens
}

// ============================================================================
// Parser
// ============================================================================

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.pos + offset)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if self.current() == Some(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}", expected))
        }
    }

    fn parse_query(&mut self) -> Result<Query, String> {
        self.expect(Token::Select)?;

        let select_cols = self.parse_select_list()?;
        self.expect(Token::From)?;

        let from_table = match self.current() {
            Some(Token::Ident(name)) => {
                let n = name.clone();
                self.advance();
                n
            }
            _ => return Err("Expected table name".into()),
        };

        let mut joins = Vec::new();
        while matches!(self.current(), Some(Token::Join)) {
            self.advance();
            let join_table = match self.current() {
                Some(Token::Ident(name)) => {
                    let n = name.clone();
                    self.advance();
                    n
                }
                _ => return Err("Expected table name in JOIN".into()),
            };
            self.expect(Token::On)?;
            let on_expr = self.parse_expr()?;
            joins.push(Join { table: join_table, on: on_expr });
        }

        let where_clause = if matches!(self.current(), Some(Token::Where)) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };

        let group_by = if matches!(self.current(), Some(Token::GroupBy)) {
            self.advance();
            if matches!(self.current(), Some(Token::Ident(s)) if s == "BY") {
                self.advance();
            }
            self.parse_column_list()?
        } else {
            Vec::new()
        };

        let order_by = if matches!(self.current(), Some(Token::OrderBy)) {
            self.advance();
            if matches!(self.current(), Some(Token::Ident(s)) if s == "BY") {
                self.advance();
            }
            let mut cols = Vec::new();
            loop {
                let col = match self.current() {
                    Some(Token::Ident(name)) => {
                        let n = name.clone();
                        self.advance();
                        n
                    }
                    _ => return Err("Expected column name".into()),
                };
                let is_asc = match self.current() {
                    Some(Token::Asc) => {
                        self.advance();
                        true
                    }
                    Some(Token::Desc) => {
                        self.advance();
                        false
                    }
                    _ => true,
                };
                cols.push((col, is_asc));
                if !matches!(self.current(), Some(Token::Comma)) {
                    break;
                }
                self.advance();
            }
            cols
        } else {
            Vec::new()
        };

        let limit = if matches!(self.current(), Some(Token::Limit)) {
            self.advance();
            match self.current() {
                Some(Token::Number(n)) => {
                    let l = n.parse().ok();
                    self.advance();
                    l
                }
                _ => return Err("Expected limit number".into()),
            }
        } else {
            None
        };

        Ok(Query { select_cols, from_table, joins, where_clause, group_by, order_by, limit })
    }

    fn parse_select_list(&mut self) -> Result<Vec<String>, String> {
        let mut cols = Vec::new();
        if matches!(self.current(), Some(Token::Star)) {
            self.advance();
            cols.push("*".into());
        } else {
            loop {
                match self.current() {
                    Some(Token::Ident(name)) => {
                        cols.push(name.clone());
                        self.advance();
                    }
                    _ => return Err("Expected column name".into()),
                }
                if !matches!(self.current(), Some(Token::Comma)) {
                    break;
                }
                self.advance();
            }
        }
        Ok(cols)
    }

    fn parse_column_list(&mut self) -> Result<Vec<String>, String> {
        let mut cols = Vec::new();
        loop {
            match self.current() {
                Some(Token::Ident(name)) => {
                    cols.push(name.clone());
                    self.advance();
                }
                _ => return Err("Expected column name".into()),
            }
            if !matches!(self.current(), Some(Token::Comma)) {
                break;
            }
            self.advance();
        }
        Ok(cols)
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_or_expr()
    }

    fn parse_or_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and_expr()?;
        while matches!(self.current(), Some(Token::Or)) {
            self.advance();
            let right = self.parse_and_expr()?;
            left = Expr::BinOp(Box::new(left), "OR".into(), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        while matches!(self.current(), Some(Token::And)) {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinOp(Box::new(left), "AND".into(), Box::new(right));
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let left = self.parse_primary()?;
        if let Some(Token::Op(op)) = self.current() {
            let op = op.clone();
            self.advance();
            let right = self.parse_primary()?;
            Ok(Expr::BinOp(Box::new(left), op, Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.current() {
            Some(Token::Ident(name)) => {
                let n = name.clone();
                self.advance();
                Ok(Expr::Column(n))
            }
            Some(Token::Number(n)) => {
                let num = n.clone();
                self.advance();
                if num.contains('.') {
                    Ok(Expr::Literal(Value::Float(num.parse().unwrap())))
                } else {
                    Ok(Expr::Literal(Value::Int(num.parse().unwrap())))
                }
            }
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Literal(Value::String(s)))
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            _ => Err("Expected expression".into()),
        }
    }
}

pub fn parse(sql: &str) -> Result<Query, String> {
    let tokens = tokenize(sql);
    let mut parser = Parser::new(tokens);
    parser.parse_query()
}

// ============================================================================
// Query Executor
// ============================================================================

pub struct Database {
    pub tables: HashMap<String, Table>,
}

impl Database {
    pub fn new() -> Self {
        Database { tables: HashMap::new() }
    }

    pub fn add_table(&mut self, table: Table) {
        self.tables.insert(table.name.clone(), table);
    }

    pub fn execute(&self, query: &Query) -> Result<Vec<Row>, String> {
        let mut rows = self.tables
            .get(&query.from_table)
            .ok_or(format!("Table not found: {}", query.from_table))?
            .rows
            .clone();

        // Apply WHERE clause
        if let Some(ref where_expr) = query.where_clause {
            rows.retain(|row| self.eval_expr(where_expr, row).is_true());
        }

        // Apply JOINs
        for join in &query.joins {
            let join_table = self.tables.get(&join.table).ok_or(format!("Table not found: {}", join.table))?;
            let mut new_rows = Vec::new();
            for left in &rows {
                for right in &join_table.rows {
                    let mut merged = left.clone();
                    merged.data.extend(right.data.clone());
                    if self.eval_expr(&join.on, &merged).is_true() {
                        new_rows.push(merged);
                    }
                }
            }
            rows = new_rows;
        }

        // Apply GROUP BY
        if !query.group_by.is_empty() {
            rows = self.apply_group_by(&rows, &query.group_by);
        }

        // Apply ORDER BY
        for (col, is_asc) in query.order_by.iter().rev() {
            rows.sort_by(|a, b| {
                let av = a.data.get(col).unwrap_or(&Value::Null);
                let bv = b.data.get(col).unwrap_or(&Value::Null);
                let cmp = self.compare_values(av, bv);
                if *is_asc { cmp } else { cmp.reverse() }
            });
        }

        // Apply SELECT projection
        let selected_cols: Vec<String> = if query.select_cols.contains(&"*".to_string()) {
            rows.get(0).map(|r| r.data.keys().cloned().collect()).unwrap_or_default()
        } else {
            query.select_cols.clone()
        };

        rows = rows
            .into_iter()
            .map(|row| {
                let mut new_row = Row { data: HashMap::new() };
                for col in &selected_cols {
                    new_row.data.insert(col.clone(), row.data.get(col).cloned().unwrap_or(Value::Null));
                }
                new_row
            })
            .collect();

        // Apply LIMIT
        if let Some(l) = query.limit {
            rows.truncate(l);
        }

        Ok(rows)
    }

    fn apply_group_by(&self, rows: &[Row], group_cols: &[String]) -> Vec<Row> {
        let mut groups: HashMap<Vec<Value>, Vec<Row>> = HashMap::new();
        for row in rows {
            let key: Vec<Value> = group_cols
                .iter()
                .map(|col| row.data.get(col).cloned().unwrap_or(Value::Null))
                .collect();
            groups.entry(key).or_insert_with(Vec::new).push(row.clone());
        }
        groups
            .into_iter()
            .map(|(key, group)| {
                let mut result = Row { data: HashMap::new() };
                for (i, col) in group_cols.iter().enumerate() {
                    result.data.insert(col.clone(), key[i].clone());
                }
                result
            })
            .collect()
    }

    fn eval_expr(&self, expr: &Expr, row: &Row) -> Value {
        match expr {
            Expr::Column(name) => row.data.get(name).cloned().unwrap_or(Value::Null),
            Expr::Literal(v) => v.clone(),
            Expr::BinOp(left, op, right) => {
                let lv = self.eval_expr(left, row);
                let rv = self.eval_expr(right, row);
                self.apply_binop(&lv, op, &rv)
            }
            Expr::FuncCall(_, _) => Value::Null,
        }
    }

    fn apply_binop(&self, left: &Value, op: &str, right: &Value) -> Value {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => match op {
                "=" => Value::Bool(a == b),
                "!=" => Value::Bool(a != b),
                "<" => Value::Bool(a < b),
                ">" => Value::Bool(a > b),
                "<=" => Value::Bool(a <= b),
                ">=" => Value::Bool(a >= b),
                _ => Value::Null,
            },
            (Value::String(a), Value::String(b)) => match op {
                "=" => Value::Bool(a == b),
                "!=" => Value::Bool(a != b),
                _ => Value::Null,
            },
            (Value::Bool(a), Value::Bool(b)) => match op {
                "AND" => Value::Bool(*a && *b),
                "OR" => Value::Bool(*a || *b),
                _ => Value::Null,
            },
            _ => Value::Null,
        }
    }

    fn compare_values(&self, a: &Value, b: &Value) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => x.cmp(y),
            (Value::String(x), Value::String(y)) => x.cmp(y),
            (Value::Float(x), Value::Float(y)) => {
                if x < y { Ordering::Less } else if x > y { Ordering::Greater } else { Ordering::Equal }
            }
            _ => Ordering::Equal,
        }
    }
}

impl Value {
    fn is_true(&self) -> bool {
        matches!(self, Value::Bool(true))
    }
}
