# Build Your Own Query Language

A SQL-like query language with lexer, parser, planner, and executor — query CSV/JSON data.

## Scope
- Lexer → Parser → AST
- SELECT, WHERE, ORDER BY, LIMIT
- JOIN support (stretch goal)
- Aggregate functions (COUNT, SUM, AVG)
- CSV and JSON as backing "tables"
- REPL interface

## Learning Goals
- Lexical analysis and tokenization
- Recursive descent parsing
- Abstract syntax trees
- Query planning and optimization basics
- Iterator/volcano execution model
- How databases actually work under the hood
