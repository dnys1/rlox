use core::fmt;
use std::error;

use crate::{
    expr::{self, ExpressionVisitor},
    token::{self, Literal, TokenType},
};

pub fn evaluate(expr: &expr::Expr) -> Result<Literal, RuntimeError> {
    expr.accept(&Interpreter {})
}

struct Interpreter;

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

    fn visit_binary(&self, expr: &expr::BinaryExpr) -> Self::Output {
        let left = expr.left.accept(self)?;
        let right = expr.right.accept(self)?;
        match expr.operator.typ {
            TokenType::Minus => {
                if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                    Ok(Literal::Number(left - right))
                } else {
                    Err(RuntimeError::new(
                        expr.operator.clone(),
                        "Operands must be numbers.",
                    ))
                }
            }
            TokenType::Slash => {
                if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                    if right == 0.0 {
                        Err(RuntimeError::new(
                            expr.operator.clone(),
                            "Cannot divide by zero.",
                        ))
                    } else {
                        Ok(Literal::Number(left / right))
                    }
                } else {
                    Err(RuntimeError::new(
                        expr.operator.clone(),
                        "Operands must be numbers.",
                    ))
                }
            }
            TokenType::Star => {
                if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                    Ok(Literal::Number(left * right))
                } else {
                    Err(RuntimeError::new(
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
                    Err(RuntimeError::new(
                        expr.operator.clone(),
                        "Operands must be two numbers or two strings.",
                    ))
                }
            }
            TokenType::Greater => {
                if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                    Ok(Literal::Boolean(left > right))
                } else {
                    Err(RuntimeError::new(
                        expr.operator.clone(),
                        "Operands must be numbers.",
                    ))
                }
            }
            TokenType::GreaterEqual => {
                if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                    Ok(Literal::Boolean(left >= right))
                } else {
                    Err(RuntimeError::new(
                        expr.operator.clone(),
                        "Operands must be numbers.",
                    ))
                }
            }
            TokenType::Less => {
                if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                    Ok(Literal::Boolean(left < right))
                } else {
                    Err(RuntimeError::new(
                        expr.operator.clone(),
                        "Operands must be numbers.",
                    ))
                }
            }
            TokenType::LessEqual => {
                if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                    Ok(Literal::Boolean(left <= right))
                } else {
                    Err(RuntimeError::new(
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

    fn visit_grouping(&self, expr: &expr::GroupingExpr) -> Self::Output {
        expr.expression.accept(self)
    }

    fn visit_literal(&self, expr: &expr::LiteralExpr) -> Self::Output {
        Ok(expr.value.clone())
    }

    fn visit_unary(&self, expr: &expr::UnaryExpr) -> Self::Output {
        let right = expr.right.accept(self)?;
        match expr.operator.typ {
            TokenType::Minus => {
                if let Literal::Number(right) = right {
                    Ok(Literal::Number(-right))
                } else {
                    Err(RuntimeError::new(
                        expr.operator.clone(),
                        "Invalid operand for unary minus",
                    ))
                }
            }
            TokenType::Bang => Ok(Literal::Boolean(!self.is_truthy(&right))),
            _ => Err(RuntimeError::new(
                expr.operator.clone(),
                "Invalid operator for unary expression",
            )),
        }
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    token: token::Token,
    message: &'static str,
}

impl RuntimeError {
    fn new(token: token::Token, message: &'static str) -> Self {
        Self { token, message }
    }
}

impl error::Error for RuntimeError {}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Runtime error: {}\n[line {}]",
            self.message, self.token.line
        )
    }
}
