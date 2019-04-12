use crate::{
  lit::Lit,
  scanner::Token,
};

pub trait ExprVisitor<T> {
  fn visit(&mut self, expr: &Expr) -> T;
}

#[derive(Clone, PartialEq)]
pub enum Expr {
  Assign { name: Token, value: Box<Expr> },
  Binary { left: Box<Expr>, op: Token, right: Box<Expr> },
  Call { callee: Box<Expr>, paren: Token, arguments: Vec<Expr> },
  Grouping { expr: Box<Expr> },
  Literal { lit: Lit },
  Logical { left: Box<Expr>, op: Token, right: Box<Expr> },
  Unary { op: Token, right: Box<Expr> },
  Variable { name: Token },
}

impl Expr {
  pub fn accept<T, V: ExprVisitor<T>>(&self, visitor: &mut V) -> T {
    visitor.visit(self)
  }

  pub fn assign(name: Token, value: Expr) -> Self {
    Expr::Assign { name, value: Box::new(value) }
  }

  pub fn call(callee: Expr, paren: Token, arguments: Vec<Expr>) -> Self {
    Expr::Call { callee: Box::new(callee), paren, arguments }
  }

  pub fn binary(left: Expr, op: Token, right: Expr) -> Self {
    Expr::Binary { left: Box::new(left), op, right: Box::new(right) }
  }

  pub fn grouping(expr: Expr) -> Self {
    Expr::Grouping { expr: Box::new(expr) }
  }

  pub fn lit(lit: Lit) -> Self {
    Expr::Literal { lit }
  }

  pub fn logical(left: Expr, op: Token, right: Expr) -> Self {
    Expr::Logical { left: Box::new(left), op, right: Box::new(right) }
  }

  pub fn unary(op: Token, right: Expr) -> Self {
    Expr::Unary { op, right: Box::new(right) }
  }

  pub fn var(name: Token) -> Self {
    Expr::Variable { name }
  }
}

pub trait StmtVisitor<T> {
  fn visit(&mut self, expr: &Stmt) -> T;
}

#[derive(Clone, PartialEq)]
pub enum Stmt {
  Block { statements: Vec<Stmt> },
  Expression { expr: Expr },
  Function { name: Token, params: Vec<Token>, body: Vec<Stmt> },
  If { condition: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>> },
  Print { expr: Expr },
  Var { name: Token, init: Option<Expr> },
  While { condition: Expr, body: Box<Stmt> },
}

impl Stmt {
  pub fn accept<T, V: StmtVisitor<T>>(&self, visitor: &mut V) -> T {
    visitor.visit(self)
  }

  pub fn block(statements: Vec<Stmt>) -> Self {
    Stmt::Block { statements }
  }

  pub fn expression(expr: Expr) -> Self {
    Stmt::Expression { expr }
  }

  pub fn function(name: Token, params: Vec<Token>, body: Vec<Stmt>) -> Self {
    Stmt::Function { name, params, body }
  }

  pub fn if_stmt(condition: Expr, then_branch: Stmt, else_branch: Option<Stmt>) -> Self {
    Stmt::If { condition, then_branch: Box::new(then_branch), else_branch: else_branch.map(Box::new) }
  }

  pub fn print(expr: Expr) -> Self {
    Stmt::Print { expr }
  }

  pub fn var(name: Token, init: Option<Expr>) -> Self {
    Stmt::Var { name, init }
  }

  pub fn while_stmt(condition: Expr, body: Stmt) -> Self {
    Stmt::While { condition, body: Box::new(body) }
  }
}

//pub struct AstPrinter {}
//
//impl AstPrinter {
//  pub fn new() -> Self { AstPrinter {} }
//  pub fn print(&mut self, expr: Expr) -> String {
//    expr.accept(self)
//  }
//  fn parenthesize(&mut self, name: &str, exprs: &[Expr]) -> String {
//    format!("({} {})", name, exprs.iter().map(|expr| expr.accept(self)).collect::<Vec<_>>().join(" "))
//  }
//}
//
//impl ExprVisitor<String> for AstPrinter {
//  fn visit(&mut self, expr: &Expr) -> String {
//    use self::Expr::*;
//    match expr {
//      Binary { left, op, right } => {
//        self.parenthesize(&op.raw, &[*(*left).clone(), *(*right).clone()])
//      }
//      Grouping { expr } => {
//        self.parenthesize("group", &[*(*expr).clone()])
//      }
//      Literal { lit } => {
//        match lit {
//          Lit::Str(str) => format!("{:?}", str),
//          Lit::Bool(b) => b.to_string(),
//          Lit::Nil => "nil".to_string(),
//          Lit::Num(num) => num.to_string(),
//        }
//      }
//      Unary { op, right } => {
//        self.parenthesize(&op.raw, &[*(*right).clone()])
//      }
//      Variable { name } => {
//        name.raw.clone()
//      }
//      Assign { name, value } => {
//        self.parenthesize("=", &[Expr::var(name.clone()), *(*value).clone()])
//      }
//    }
//  }
//}
