use std::{error, fmt};

use crate::stmt::Stmt;
use crate::Result;
use crate::{expr::*, token::*};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt> {
        if self.matches_token(TokenType::Var) {
            self.var_declaration().or_else(|_| {
                self.synchronize();
                Ok(Stmt::Expression(Expr::Literal(LiteralExpr {
                    value: Literal::Nil,
                })))
            })
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expected variable name.")?;
        let initializer = if self.matches_token(TokenType::Equal) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(
            TokenType::Semicolon,
            "Expected ';' after variable declaration.",
        )?;
        Ok(Stmt::Var(name, initializer))
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.matches_token(TokenType::Print) {
            self.print_statement()
        } else if self.matches_token(TokenType::LeftBrace) {
            self.block()
        } else {
            self.expression_statement()
        }
    }

    fn block(&mut self) -> Result<Stmt> {
        let mut statements = Vec::new();
        while !self.check_token(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace, "Expected '}' after block.")?;
        Ok(Stmt::Block(statements))
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after print statement.")?;
        Ok(Stmt::Print(value))
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression.")?;
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.equality()?;
        if self.check_token(TokenType::Equal) {
            let equals = self.advance();
            let value = self.assignment()?;
            if let Expr::Variable(var) = expr {
                return Ok(Expr::Assign(AssignExpr {
                    name: var.name,
                    value: Box::new(value),
                }));
            }
            eprintln!("{}", ParseError::new(equals, "Invalid assignment target"));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;

        while let TokenType::BangEqual | TokenType::EqualEqual = self.peek().typ {
            let operator = self.advance();
            let right = self.comparison()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;

        while let TokenType::Greater
        | TokenType::GreaterEqual
        | TokenType::Less
        | TokenType::LessEqual = self.peek().typ
        {
            let operator = self.advance();
            let right = self.term()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;

        while let TokenType::Minus | TokenType::Plus = self.peek().typ {
            let operator = self.advance();
            let right = self.factor()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while let TokenType::Slash | TokenType::Star = self.peek().typ {
            let operator = self.advance();
            let right = self.unary()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if let TokenType::Bang | TokenType::Minus = self.peek().typ {
            let operator = self.advance();
            let right = self.unary()?;
            Ok(Expr::Unary(UnaryExpr {
                operator,
                right: Box::new(right),
            }))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr> {
        let token = self.advance();
        let expr = match token.typ {
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "expected ')' after expression")?;
                Expr::Grouping(GroupingExpr {
                    expression: Box::new(expr),
                })
            }
            TokenType::String
            | TokenType::Number
            | TokenType::True
            | TokenType::False
            | TokenType::Nil => Expr::Literal(LiteralExpr {
                value: token.literal.unwrap_or(Literal::Nil),
            }),
            TokenType::Identifier => Expr::Variable(VariableExpr { name: token }),
            _ => return Err(ParseError::new(token, "expected expression").into()),
        };
        Ok(expr)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn check_token(&self, typ: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().typ == typ
        }
    }

    fn matches_token(&mut self, typ: TokenType) -> bool {
        if self.check_token(typ) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn consume(&mut self, typ: TokenType, msg: &'static str) -> Result<Token> {
        if self.check_token(typ) {
            Ok(self.advance())
        } else {
            Err(ParseError::new(self.peek().clone(), msg).into())
        }
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn is_at_end(&self) -> bool {
        self.peek().typ == TokenType::EOF
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().typ == TokenType::Semicolon {
                return;
            }

            match self.peek().typ {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => {}
            }

            self.advance();
        }
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub token: Token,
    pub message: &'static str,
}

impl ParseError {
    pub fn new(token: Token, message: &'static str) -> Self {
        ParseError { token, message }
    }
}

impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.token.typ == TokenType::EOF {
            write!(f, "{} at end: {}", self.token.line, self.message)
        } else {
            write!(
                f,
                "{} at {}: {}",
                self.token.line, self.token.lexeme, self.message
            )
        }
    }
}
