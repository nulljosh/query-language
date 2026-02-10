use std::io::{self, BufRead, Write};

#[derive(Debug)]
enum Token {
    Select, From, Where, OrderBy, Limit,
    Ident(String), Number(i64), String(String),
    Star, Comma, Gt, Lt, Eq, Semi,
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' => { chars.next(); }
            '*' => { chars.next(); tokens.push(Token::Star); }
            ',' => { chars.next(); tokens.push(Token::Comma); }
            '>' => { chars.next(); tokens.push(Token::Gt); }
            '<' => { chars.next(); tokens.push(Token::Lt); }
            '=' => { chars.next(); tokens.push(Token::Eq); }
            ';' => { chars.next(); tokens.push(Token::Semi); }
            '\'' | '"' => {
                let q = chars.next().unwrap();
                let s: String = chars.by_ref().take_while(|&c| c != q).collect();
                tokens.push(Token::String(s));
            }
            '0'..='9' => {
                let n: String = std::iter::from_fn(|| {
                    chars.peek().filter(|c| c.is_ascii_digit()).map(|_| chars.next().unwrap())
                }).collect();
                tokens.push(Token::Number(n.parse().unwrap()));
            }
            _ if ch.is_alphabetic() || ch == '_' => {
                let word: String = std::iter::from_fn(|| {
                    chars.peek().filter(|c| c.is_alphanumeric() || **c == '_' || **c == '.').map(|_| chars.next().unwrap())
                }).collect();
                match word.to_uppercase().as_str() {
                    "SELECT" => tokens.push(Token::Select),
                    "FROM" => tokens.push(Token::From),
                    "WHERE" => tokens.push(Token::Where),
                    "LIMIT" => tokens.push(Token::Limit),
                    _ => tokens.push(Token::Ident(word)),
                }
            }
            _ => { chars.next(); }
        }
    }
    tokens
}

fn main() {
    let stdin = io::stdin();
    print!("sql> ");
    io::stdout().flush().unwrap();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if line.trim().eq_ignore_ascii_case("quit") { break; }
        let tokens = tokenize(&line);
        println!("{tokens:?}");
        print!("sql> ");
        io::stdout().flush().unwrap();
    }
}
