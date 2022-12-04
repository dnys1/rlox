use core::fmt;
use std::{
    cell::RefCell,
    error::{self, Error},
    fs,
    io::{stdin, stdout, Write},
    rc::Rc,
};

use crate::{
    environment::Environment,
    expr::{self, ExpressionVisitor},
    parser::Parser,
    scanner::Scanner,
    stmt::{Stmt, StmtVisitor},
    token::{self, Literal, TokenType},
};

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn run_file(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
        let source = fs::read_to_string(filename)?;
        self.run(&source)
    }

    pub fn run_prompt(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let mut input = String::new();
            print!("> ");
            stdout().flush()?;
            stdin().read_line(&mut input)?;
            if input.trim().is_empty() {
                break;
            }
            self.run(&input)?;
        }
        Ok(())
    }

    fn run(&mut self, source: &str) -> Result<(), Box<dyn Error>> {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse()?;
        self.interpret(stmts)?;
        Ok(())
    }

    fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<(), RuntimeError> {
        for stmt in stmts {
            stmt.accept(self)?;
        }
        Ok(())
    }

    /// Executes a block of code with its own environment.
    fn execute_block(&mut self, block: &[Stmt]) -> Result<(), RuntimeError> {
        let environment = Rc::clone(&self.environment);
        self.environment = Rc::new(RefCell::new(Environment::new_enclosed(environment)));
        for stmt in block {
            stmt.accept(self)?;
        }
        self.environment = Rc::clone(&self.environment)
            .borrow()
            .enclosing
            .clone()
            .unwrap();
        Ok(())
    }
}

impl Interpreter {
    fn is_truthy(&self, literal: &Literal) -> bool {
        match literal {
            Literal::Nil => false,
            Literal::Boolean(b) => *b,
            _ => true,
        }
    }
}

impl ExpressionVisitor for Interpreter {
    type Output = Result<Literal, RuntimeError>;

    fn visit_binary(&mut self, expr: &expr::BinaryExpr) -> Self::Output {
        let left = expr.left.accept(self)?;
        let right = expr.right.accept(self)?;
        match expr.operator.typ {
            TokenType::Minus => {
                if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                    Ok(Literal::Number(left - right))
                } else {
                    Err(RuntimeError::Token(
                        expr.operator.clone(),
                        "Operands must be numbers.",
                    ))
                }
            }
            TokenType::Slash => {
                if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                    if right == 0.0 {
                        Err(RuntimeError::Token(
                            expr.operator.clone(),
                            "Cannot divide by zero.",
                        ))
                    } else {
                        Ok(Literal::Number(left / right))
                    }
                } else {
                    Err(RuntimeError::Token(
                        expr.operator.clone(),
                        "Operands must be numbers.",
                    ))
                }
            }
            TokenType::Star => {
                if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                    Ok(Literal::Number(left * right))
                } else {
                    Err(RuntimeError::Token(
                        expr.operator.clone(),
                        "Operands must be numbers.",
                    ))
                }
            }
            TokenType::Plus => {
                if let (Literal::Number(left), Literal::Number(right)) = (&left, &right) {
                    Ok(Literal::Number(left + right))
                } else if let (Literal::String(left), Literal::String(right)) = (&left, &right) {
                    Ok(Literal::String(left.clone() + right))
                } else {
                    Err(RuntimeError::Token(
                        expr.operator.clone(),
                        "Operands must be two numbers or two strings.",
                    ))
                }
            }
            TokenType::Greater => {
                if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                    Ok(Literal::Boolean(left > right))
                } else {
                    Err(RuntimeError::Token(
                        expr.operator.clone(),
                        "Operands must be numbers.",
                    ))
                }
            }
            TokenType::GreaterEqual => {
                if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                    Ok(Literal::Boolean(left >= right))
                } else {
                    Err(RuntimeError::Token(
                        expr.operator.clone(),
                        "Operands must be numbers.",
                    ))
                }
            }
            TokenType::Less => {
                if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                    Ok(Literal::Boolean(left < right))
                } else {
                    Err(RuntimeError::Token(
                        expr.operator.clone(),
                        "Operands must be numbers.",
                    ))
                }
            }
            TokenType::LessEqual => {
                if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                    Ok(Literal::Boolean(left <= right))
                } else {
                    Err(RuntimeError::Token(
                        expr.operator.clone(),
                        "Operands must be numbers.",
                    ))
                }
            }
            TokenType::BangEqual => Ok(Literal::Boolean(left != right)),
            TokenType::EqualEqual => Ok(Literal::Boolean(left == right)),
            _ => unreachable!(),
        }
    }

    fn visit_grouping(&mut self, expr: &expr::GroupingExpr) -> Self::Output {
        expr.expression.accept(self)
    }

    fn visit_literal(&mut self, expr: &expr::LiteralExpr) -> Self::Output {
        Ok(expr.value.clone())
    }

    fn visit_unary(&mut self, expr: &expr::UnaryExpr) -> Self::Output {
        let right = expr.right.accept(self)?;
        match expr.operator.typ {
            TokenType::Minus => {
                if let Literal::Number(right) = right {
                    Ok(Literal::Number(-right))
                } else {
                    Err(RuntimeError::Token(
                        expr.operator.clone(),
                        "Invalid operand for unary minus",
                    ))
                }
            }
            TokenType::Bang => Ok(Literal::Boolean(!self.is_truthy(&right))),
            _ => unreachable!(),
        }
    }

    fn visit_variable(&mut self, expr: &expr::VariableExpr) -> Self::Output {
        self.environment.borrow().get(&expr.name.lexeme)
    }

    fn visit_assign(&mut self, expr: &expr::AssignExpr) -> Self::Output {
        let value = expr.value.accept(self)?;
        self.environment
            .borrow_mut()
            .assign(&expr.name.lexeme, value.clone())?;
        Ok(value)
    }
}

impl StmtVisitor for Interpreter {
    type Output = Result<(), RuntimeError>;

    fn visit_expression(&mut self, expr: &expr::Expr) -> Self::Output {
        expr.accept(self).map(|_| ())
    }

    fn visit_print(&mut self, expr: &expr::Expr) -> Self::Output {
        let value = expr.accept(self)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_var(&mut self, name: &token::Token, initializer: &Option<expr::Expr>) -> Self::Output {
        let value = initializer
            .as_ref()
            .map(|expr| expr.accept(self))
            .transpose()?
            .unwrap_or(Literal::Nil);
        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), value);
        Ok(())
    }

    fn visit_block(&mut self, statements: &[Stmt]) -> Self::Output {
        self.execute_block(statements)
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    Token(token::Token, &'static str),
    UndefinedVariable(String),
}

impl error::Error for RuntimeError {}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            RuntimeError::Token(token, message) => format!("{}: {}", token, message),
            RuntimeError::UndefinedVariable(name) => format!("Undefined variable '{}'.", name),
        };
        write!(f, "{}", message)
    }
}
