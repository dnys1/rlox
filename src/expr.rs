use std::fmt;

use crate::token::*;

#[derive(Debug, Clone)]
pub enum Expr {
    Binary(BinaryExpr),
    Grouping(GroupingExpr),
    Literal(LiteralExpr),
    Unary(UnaryExpr),
    Variable(VariableExpr),
    Assign(AssignExpr),
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
    pub value: Literal,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct VariableExpr {
    pub name: Token,
}

#[derive(Debug, Clone)]
pub struct AssignExpr {
    pub name: Token,
    pub value: Box<Expr>,
}

impl Expr {
    pub fn accept<Visitor: ExpressionVisitor>(&self, visitor: &mut Visitor) -> Visitor::Output {
        visitor.visit(self)
    }
}

pub trait ExpressionVisitor {
    type Output;

    fn visit(&mut self, expr: &Expr) -> Self::Output {
        match expr {
            Expr::Binary(expr) => self.visit_binary(expr),
            Expr::Grouping(expr) => self.visit_grouping(expr),
            Expr::Literal(expr) => self.visit_literal(expr),
            Expr::Unary(expr) => self.visit_unary(expr),
            Expr::Variable(expr) => self.visit_variable(expr),
            Expr::Assign(expr) => self.visit_assign(expr),
        }
    }

    fn visit_binary(&mut self, expr: &BinaryExpr) -> Self::Output;
    fn visit_grouping(&mut self, expr: &GroupingExpr) -> Self::Output;
    fn visit_literal(&mut self, expr: &LiteralExpr) -> Self::Output;
    fn visit_unary(&mut self, expr: &UnaryExpr) -> Self::Output;
    fn visit_variable(&mut self, expr: &VariableExpr) -> Self::Output;
    fn visit_assign(&mut self, expr: &AssignExpr) -> Self::Output;
}

struct Printer;

impl Printer {
    fn parenthesize(&mut self, name: String, exprs: Vec<Expr>) -> String {
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

impl ExpressionVisitor for Printer {
    type Output = String;

    fn visit_binary(&mut self, expr: &BinaryExpr) -> Self::Output {
        self.parenthesize(
            expr.operator.lexeme.clone(),
            vec![*expr.left.clone(), *expr.right.clone()],
        )
    }

    fn visit_grouping(&mut self, expr: &GroupingExpr) -> Self::Output {
        self.parenthesize(String::from("group"), vec![*expr.expression.clone()])
    }

    fn visit_literal(&mut self, expr: &LiteralExpr) -> Self::Output {
        format!("{}", &expr.value)
    }

    fn visit_unary(&mut self, expr: &UnaryExpr) -> Self::Output {
        self.parenthesize(expr.operator.lexeme.clone(), vec![*expr.right.clone()])
    }

    fn visit_variable(&mut self, expr: &VariableExpr) -> Self::Output {
        expr.name.lexeme.clone()
    }

    fn visit_assign(&mut self, expr: &AssignExpr) -> Self::Output {
        self.parenthesize(expr.name.lexeme.clone(), vec![*expr.value.clone()])
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.accept(&mut Printer))
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
                    value: Literal::Number(123.0),
                })),
            })),
            operator: Token::new(TokenType::Star, "*".into(), None, 1),
            right: Box::new(Expr::Grouping(GroupingExpr {
                expression: Box::new(Expr::Literal(LiteralExpr {
                    value: Literal::Number(45.67),
                })),
            })),
        });
        assert_eq!("(* (- 123) (group 45.67))", format!("{}", exp));
    }
}
