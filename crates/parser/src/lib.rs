//! SQL Parser Module
//!
//! Converts token stream from Lexer into Abstract Syntax Tree (AST).
//! Supports SQL-92 subset: SELECT, INSERT, UPDATE, DELETE, CREATE/DROP TABLE.

use serde::{Deserialize, Serialize};
use sqlrustgo_lexer::{Lexer, Token};

/// SQL Statement types
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Select(SelectStatement),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),
    CreateTable(CreateTableStatement),
    DropTable(DropTableStatement),
}

/// Aggregate function type
#[derive(Debug, Clone, PartialEq)]
pub enum AggregateFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

/// Aggregate function call
#[derive(Debug, Clone, PartialEq)]
pub struct AggregateCall {
    pub func: AggregateFunction,
    pub column: Option<String>,
}

/// SELECT statement
#[derive(Debug, Clone, PartialEq)]
pub struct SelectStatement {
    pub columns: Vec<SelectColumn>,
    pub table: String,
    pub where_clause: Option<Expression>,
    pub aggregates: Vec<AggregateCall>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

/// Column in SELECT
#[derive(Debug, Clone, PartialEq)]
pub struct SelectColumn {
    pub name: String,
    pub alias: Option<String>,
}

/// INSERT statement
#[derive(Debug, Clone, PartialEq)]
pub struct InsertStatement {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<Vec<Expression>>,
}

/// UPDATE statement  
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateStatement {
    pub table: String,
    pub set_clauses: Vec<(String, Expression)>,
    pub where_clause: Option<Expression>,
}

/// DELETE statement
#[derive(Debug, Clone, PartialEq)]
pub struct DeleteStatement {
    pub table: String,
    pub where_clause: Option<Expression>,
}

/// CREATE TABLE statement
#[derive(Debug, Clone, PartialEq)]
pub struct CreateTableStatement {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
}

/// DROP TABLE statement
#[derive(Debug, Clone, PartialEq)]
pub struct DropTableStatement {
    pub name: String,
}

/// Column definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

/// SQL Expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(String),
    Identifier(String),
    BinaryOp(Box<Expression>, String, Box<Expression>),
}

/// SQL Parser
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    /// Create a parser from tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    /// Get current token
    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    /// Advance to next token
    fn next(&mut self) -> Option<Token> {
        self.position += 1;
        self.tokens.get(self.position - 1).cloned()
    }

    /// Expect a specific token
    fn expect(&mut self, expected: Token) -> Result<Token, String> {
        match self.current() {
            Some(t) if t == &expected => self
                .next()
                .ok_or_else(|| "Unexpected end of input".to_string()),
            Some(t) => Err(format!("Expected {:?}, got {:?}", expected, t)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    /// Parse a complete SQL statement
    pub fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current() {
            Some(Token::Select) => self.parse_select(),
            Some(Token::Insert) => self.parse_insert(),
            Some(Token::Update) => self.parse_update(),
            Some(Token::Delete) => self.parse_delete(),
            Some(Token::Create) => self.parse_create_table(),
            Some(Token::Drop) => self.parse_drop_table(),
            Some(t) => Err(format!("Unexpected token: {:?}", t)),
            None => Err("Empty input".to_string()),
        }
    }

    fn parse_select(&mut self) -> Result<Statement, String> {
        self.expect(Token::Select)?;

        let mut columns = Vec::new();
        let mut aggregates = Vec::new();

        loop {
            match self.current() {
                Some(Token::From) => break,
                Some(Token::Count) | Some(Token::Sum) | Some(Token::Avg) | Some(Token::Min)
                | Some(Token::Max) => {
                    let agg = self.parse_aggregate()?;
                    aggregates.push(agg);
                }
                Some(Token::Star) => {
                    columns.push(SelectColumn {
                        name: "*".to_string(),
                        alias: None,
                    });
                    self.next();
                }
                Some(Token::Identifier(_)) => {
                    if let Some(Token::Identifier(name)) = self.next() {
                        columns.push(SelectColumn { name, alias: None });
                    }
                }
                Some(Token::Comma) => {
                    self.next();
                }
                _ => {
                    return Err("Expected FROM or column name".to_string());
                }
            }
        }

        self.expect(Token::From)?;

        let table = match self.next() {
            Some(Token::Identifier(name)) => name,
            Some(t) => return Err(format!("Expected table name, got {:?}", t)),
            None => return Err("Expected table name".to_string()),
        };

        let where_clause = if matches!(self.current(), Some(Token::Where)) {
            self.next();
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Parse LIMIT and OFFSET
        let mut limit = None;
        let mut offset = None;

        if matches!(self.current(), Some(Token::Limit)) {
            self.next();
            if let Some(Token::NumberLiteral(n)) = self.next() {
                limit = Some(n.parse::<u64>().map_err(|_| "Invalid LIMIT value")?);
            } else {
                return Err("Expected number after LIMIT".to_string());
            }

            // Parse optional OFFSET
            if matches!(self.current(), Some(Token::Offset)) {
                self.next();
                if let Some(Token::NumberLiteral(n)) = self.next() {
                    offset = Some(n.parse::<u64>().map_err(|_| "Invalid OFFSET value")?);
                } else {
                    return Err("Expected number after OFFSET".to_string());
                }
            }
        }

        Ok(Statement::Select(SelectStatement {
            columns,
            table,
            where_clause,
            aggregates,
            limit,
            offset,
        }))
    }

    fn parse_aggregate(&mut self) -> Result<AggregateCall, String> {
        let func_token = self
            .current()
            .cloned()
            .ok_or("Expected aggregate function")?;
        let func = match func_token {
            Token::Count => AggregateFunction::Count,
            Token::Sum => AggregateFunction::Sum,
            Token::Avg => AggregateFunction::Avg,
            Token::Min => AggregateFunction::Min,
            Token::Max => AggregateFunction::Max,
            _ => return Err("Not an aggregate function".to_string()),
        };

        self.next();
        self.expect(Token::LParen)?;

        let column = match self.next() {
            Some(Token::Star) => None,
            Some(Token::Identifier(name)) => Some(name),
            _ => return Err("Expected column name or *".to_string()),
        };

        self.expect(Token::RParen)?;

        Ok(AggregateCall { func, column })
    }

    fn parse_insert(&mut self) -> Result<Statement, String> {
        self.expect(Token::Insert)?;
        self.expect(Token::Into)?;

        let table = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected table name".to_string()),
        };

        let columns = if matches!(self.current(), Some(Token::LParen)) {
            self.next();
            let mut cols = Vec::new();
            loop {
                match self.current() {
                    Some(Token::Identifier(name)) => {
                        cols.push(name.clone());
                        self.next();
                    }
                    Some(Token::RParen) => {
                        self.next();
                        break;
                    }
                    Some(Token::Comma) => {
                        self.next();
                    }
                    _ => return Err("Expected column name".to_string()),
                }
            }
            cols
        } else {
            Vec::new()
        };

        if !matches!(self.current(), Some(Token::Values)) {
            return Err("Expected VALUES".to_string());
        }
        self.next();

        let mut values = Vec::new();

        if !matches!(self.current(), Some(Token::LParen)) {
            return Err("Expected ( after VALUES".to_string());
        }

        loop {
            if !matches!(self.current(), Some(Token::LParen)) {
                break;
            }

            self.next();
            let mut row = Vec::new();
            loop {
                match self.current() {
                    Some(Token::RParen) => {
                        self.next();
                        break;
                    }
                    Some(Token::Identifier(name)) => {
                        row.push(Expression::Identifier(name.clone()));
                        self.next();
                    }
                    Some(Token::NumberLiteral(n)) => {
                        row.push(Expression::Literal(n.clone()));
                        self.next();
                    }
                    Some(Token::StringLiteral(s)) => {
                        row.push(Expression::Literal(format!("'{}'", s)));
                        self.next();
                    }
                    Some(Token::Comma) => {
                        self.next();
                    }
                    Some(Token::Null) => {
                        row.push(Expression::Literal("NULL".to_string()));
                        self.next();
                    }
                    Some(Token::Minus) => {
                        self.next();
                        if let Some(Token::NumberLiteral(n)) = self.current() {
                            row.push(Expression::Literal(format!("-{}", n)));
                            self.next();
                        } else {
                            return Err("Expected number after -".to_string());
                        }
                    }
                    _ => return Err("Expected value".to_string()),
                }
            }
            values.push(row);

            match self.current() {
                Some(Token::Comma) => {
                    self.next();
                }
                _ => break,
            }
        }

        if values.is_empty() {
            return Err("Expected at least one row of values".to_string());
        }

        Ok(Statement::Insert(InsertStatement {
            table,
            columns,
            values,
        }))
    }

    fn parse_update(&mut self) -> Result<Statement, String> {
        self.expect(Token::Update)?;
        let table = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected table name".to_string()),
        };

        if !matches!(self.current(), Some(Token::Set)) {
            return Err("Expected SET".to_string());
        }
        self.next();

        let mut set_clauses = Vec::new();
        loop {
            let column = match self.current() {
                Some(Token::Identifier(name)) => name.clone(),
                _ => return Err("Expected column name in SET".to_string()),
            };
            self.next();

            match self.current() {
                Some(Token::Equal) => {}
                _ => return Err("Expected = in SET clause".to_string()),
            }
            self.next();

            let value = match self.current() {
                Some(Token::Identifier(name)) => Expression::Identifier(name.clone()),
                Some(Token::NumberLiteral(n)) => Expression::Literal(n.clone()),
                Some(Token::StringLiteral(s)) => Expression::Literal(format!("'{}'", s)),
                Some(Token::Null) => Expression::Literal("NULL".to_string()),
                Some(Token::Minus) => {
                    self.next();
                    if let Some(Token::NumberLiteral(n)) = self.current() {
                        Expression::Literal(format!("-{}", n))
                    } else {
                        return Err("Expected number after -".to_string());
                    }
                }
                _ => return Err("Expected value in SET clause".to_string()),
            };
            self.next();

            set_clauses.push((column, value));

            match self.current() {
                Some(Token::Comma) => {
                    self.next();
                }
                Some(Token::Where) | None | Some(Token::Eof) => break,
                _ => return Err("Expected , or WHERE".to_string()),
            }
        }

        let where_clause = if matches!(self.current(), Some(Token::Where)) {
            self.next();
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(Statement::Update(UpdateStatement {
            table,
            set_clauses,
            where_clause,
        }))
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        let left = match self.current() {
            Some(Token::Identifier(name)) => Expression::Identifier(name.clone()),
            Some(Token::NumberLiteral(n)) => Expression::Literal(n.clone()),
            Some(Token::StringLiteral(s)) => Expression::Literal(format!("'{}'", s)),
            Some(Token::Null) => Expression::Literal("NULL".to_string()),
            Some(Token::Minus) => {
                self.next();
                if let Some(Token::NumberLiteral(n)) = self.current() {
                    Expression::Literal(format!("-{}", n))
                } else {
                    return Err("Expected number after -".to_string());
                }
            }
            _ => return Err("Expected expression".to_string()),
        };
        self.next();

        let op = match self.current() {
            Some(Token::Equal) => "=",
            Some(Token::NotEqual) => "!=",
            Some(Token::Greater) => ">",
            Some(Token::Less) => "<",
            Some(Token::GreaterEqual) => ">=",
            Some(Token::LessEqual) => "<=",
            _ => return Ok(left),
        };
        self.next();

        let right = match self.current() {
            Some(Token::Identifier(name)) => Expression::Identifier(name.clone()),
            Some(Token::NumberLiteral(n)) => Expression::Literal(n.clone()),
            Some(Token::StringLiteral(s)) => Expression::Literal(format!("'{}'", s)),
            Some(Token::Null) => Expression::Literal("NULL".to_string()),
            Some(Token::Minus) => {
                self.next();
                if let Some(Token::NumberLiteral(n)) = self.current() {
                    Expression::Literal(format!("-{}", n))
                } else {
                    return Err("Expected number after -".to_string());
                }
            }
            _ => return Err("Expected value in expression".to_string()),
        };
        self.next();

        Ok(Expression::BinaryOp(
            Box::new(left),
            op.to_string(),
            Box::new(right),
        ))
    }

    fn parse_delete(&mut self) -> Result<Statement, String> {
        self.expect(Token::Delete)?;
        self.expect(Token::From)?;
        let table = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected table name".to_string()),
        };

        let where_clause = if matches!(self.current(), Some(Token::Where)) {
            self.next();
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(Statement::Delete(DeleteStatement {
            table,
            where_clause,
        }))
    }

    fn parse_create_table(&mut self) -> Result<Statement, String> {
        self.expect(Token::Create)?;
        self.expect(Token::Table)?;
        let name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected table name".to_string()),
        };

        let mut columns = Vec::new();
        if matches!(self.current(), Some(Token::LParen)) {
            self.next();
            loop {
                match self.current() {
                    Some(Token::Identifier(name)) => {
                        let col_name = name.clone();
                        self.next();
                        let data_type = match self.current() {
                            Some(Token::Identifier(type_name)) => {
                                let t = type_name.to_uppercase();
                                self.next();
                                t
                            }
                            Some(Token::Integer) => {
                                self.next();
                                "INTEGER".to_string()
                            }
                            Some(Token::Text) => {
                                self.next();
                                "TEXT".to_string()
                            }
                            _ => "INTEGER".to_string(),
                        };
                        columns.push(ColumnDefinition {
                            name: col_name,
                            data_type,
                            nullable: true,
                        });
                    }
                    Some(Token::RParen) => {
                        self.next();
                        break;
                    }
                    Some(Token::Comma) => {
                        self.next();
                    }
                    _ => break,
                }
            }
        }

        Ok(Statement::CreateTable(CreateTableStatement {
            name,
            columns,
        }))
    }

    fn parse_drop_table(&mut self) -> Result<Statement, String> {
        self.expect(Token::Drop)?;
        self.expect(Token::Table)?;
        let name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected table name".to_string()),
        };

        Ok(Statement::DropTable(DropTableStatement { name }))
    }
}

/// Parse a SQL string into statements
pub fn parse(sql: &str) -> Result<Statement, String> {
    let tokens = Lexer::new(sql).tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse_statement()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_select() {
        let result = parse("SELECT id FROM users");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.table, "users");
                assert_eq!(s.columns.len(), 1);
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_insert() {
        let result = parse("INSERT INTO users VALUES (1)");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Insert(i) => {
                assert_eq!(i.table, "users");
                assert_eq!(i.values.len(), 1);
            }
            _ => panic!("Expected INSERT statement"),
        }
    }

    #[test]
    fn test_parse_update() {
        let result = parse("UPDATE users SET name = 'Bob'");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Update(u) => {
                assert_eq!(u.table, "users");
                assert_eq!(u.set_clauses.len(), 1);
            }
            _ => panic!("Expected UPDATE statement"),
        }
    }

    #[test]
    fn test_parse_delete() {
        let result = parse("DELETE FROM users");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Delete(d) => {
                assert_eq!(d.table, "users");
            }
            _ => panic!("Expected DELETE statement"),
        }
    }

    #[test]
    fn test_parse_create_table() {
        let result = parse("CREATE TABLE users");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::CreateTable(c) => {
                assert_eq!(c.name, "users");
            }
            _ => panic!("Expected CREATE TABLE statement"),
        }
    }

    #[test]
    fn test_parse_drop_table() {
        let result = parse("DROP TABLE users");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::DropTable(d) => {
                assert_eq!(d.name, "users");
            }
            _ => panic!("Expected DROP TABLE statement"),
        }
    }

    #[test]
    fn test_parse_select_limit() {
        let result = parse("SELECT * FROM users LIMIT 10");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.table, "users");
                assert_eq!(s.limit, Some(10));
                assert_eq!(s.offset, None);
            }
            _ => panic!("Expected SELECT statement with LIMIT"),
        }
    }

    #[test]
    fn test_parse_select_limit_offset() {
        let result = parse("SELECT * FROM users LIMIT 10 OFFSET 20");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.table, "users");
                assert_eq!(s.limit, Some(10));
                assert_eq!(s.offset, Some(20));
            }
            _ => panic!("Expected SELECT statement with LIMIT and OFFSET"),
        }
    }
}
