use crate::{expr, token::Token};

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(expr::Expr),
    Print(expr::Expr),
    Var(Token, Option<expr::Expr>),
    Block(Vec<Stmt>),
}

impl Stmt {
    pub fn accept<Visitor: StmtVisitor>(&self, visitor: &mut Visitor) -> Visitor::Output {
        match self {
            Stmt::Expression(expr) => visitor.visit_expression(expr),
            Stmt::Print(expr) => visitor.visit_print(expr),
            Stmt::Var(name, initializer) => visitor.visit_var(name, initializer),
            Stmt::Block(statements) => visitor.visit_block(statements),
        }
    }
}

pub trait StmtVisitor {
    type Output;

    fn visit(&mut self, stmt: &Stmt) -> Self::Output {
        match stmt {
            Stmt::Expression(stmt) => self.visit_expression(stmt),
            Stmt::Print(stmt) => self.visit_print(stmt),
            Stmt::Var(name, initializer) => self.visit_var(name, initializer),
            Stmt::Block(statements) => self.visit_block(statements),
        }
    }

    fn visit_expression(&mut self, stmt: &expr::Expr) -> Self::Output;
    fn visit_print(&mut self, stmt: &expr::Expr) -> Self::Output;
    fn visit_var(&mut self, name: &Token, initializer: &Option<expr::Expr>) -> Self::Output;
    fn visit_block(&mut self, statements: &[Stmt]) -> Self::Output;
}
