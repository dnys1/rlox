use std::fmt;

use crate::token::*;

#[derive(Debug, Clone)]
pub enum Expr {
    Binary(BinaryExpr),
    Grouping(GroupingExpr),
    Literal(LiteralExpr),
    Unary(UnaryExpr),
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct GroupingExpr {
    pub expression: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct LiteralExpr {
    pub value: Option<Literal>,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub operator: Token,
    pub right: Box<Expr>,
}

impl Expr {
    pub fn accept<Visitor: ExpressionVisitor>(&self, visitor: Visitor) -> Visitor::Output {
        visitor.visit(self)
    }
}

pub trait ExpressionVisitor {
    type Output;

    fn visit(&self, expr: &Expr) -> Self::Output {
        match expr {
            Expr::Binary(expr) => self.visit_binary(expr),
            Expr::Grouping(expr) => self.visit_grouping(expr),
            Expr::Literal(expr) => self.visit_literal(expr),
            Expr::Unary(expr) => self.visit_unary(expr),
        }
    }

    fn visit_binary(&self, expr: &BinaryExpr) -> Self::Output;
    fn visit_grouping(&self, expr: &GroupingExpr) -> Self::Output;
    fn visit_literal(&self, expr: &LiteralExpr) -> Self::Output;
    fn visit_unary(&self, expr: &UnaryExpr) -> Self::Output;
}

struct Printer;

impl Printer {
    fn parenthesize(&self, name: String, exprs: Vec<Expr>) -> String {
        let mut builder = String::new();
        builder += "(";
        builder += name.as_str();
        for expr in exprs {
            builder += " ";
            builder += expr.accept(self).as_str();
        }
        builder += ")";
        builder
    }
}

impl ExpressionVisitor for &Printer {
    type Output = String;

    fn visit_binary(&self, expr: &BinaryExpr) -> Self::Output {
        self.parenthesize(
            expr.operator.lexeme.clone(),
            vec![*expr.left.clone(), *expr.right.clone()],
        )
    }

    fn visit_grouping(&self, expr: &GroupingExpr) -> Self::Output {
        self.parenthesize(String::from("group"), vec![*expr.expression.clone()])
    }

    fn visit_literal(&self, expr: &LiteralExpr) -> Self::Output {
        match &expr.value {
            Some(literal) => format!("{}", literal),
            None => String::from("nil"),
        }
    }

    fn visit_unary(&self, expr: &UnaryExpr) -> Self::Output {
        self.parenthesize(expr.operator.lexeme.clone(), vec![*expr.right.clone()])
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.accept(&Printer {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_printer() {
        let exp = Expr::Binary(BinaryExpr {
            left: Box::new(Expr::Unary(UnaryExpr {
                operator: Token::new(TokenType::Minus, "-".into(), None, 1),
                right: Box::new(Expr::Literal(LiteralExpr {
                    value: Some(Literal::Number(123.0)),
                })),
            })),
            operator: Token::new(TokenType::Star, "*".into(), None, 1),
            right: Box::new(Expr::Grouping(GroupingExpr {
                expression: Box::new(Expr::Literal(LiteralExpr {
                    value: Some(Literal::Number(45.67)),
                })),
            })),
        });
        assert_eq!("(* (- 123) (group 45.67))", format!("{}", exp));
    }
}
