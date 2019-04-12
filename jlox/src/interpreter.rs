use crate::{
  expr::StmtVisitor,
  scanner::Token,
  expr::{
    ExprVisitor,
    Expr,
  },
  scanner::{TokenType::*},
  err::LoxError,
  lit::Lit,
  expr::Stmt,
  err::LoxResult,
  lit::Function
};
use std::{
  mem,
  collections::HashMap,
};

pub struct Environment {
  values: HashMap<String, Lit>,
  enclosing: Option<Box<Environment>>,
}

impl Environment {
  pub fn new(enclosing: Option<Environment>) -> Self {
    Environment {
      values: HashMap::new(),
      enclosing: enclosing.map(Box::new),
    }
  }

  pub fn define(&mut self, name: String, value: Lit) {
    self.values.insert(name, value);
  }

  pub fn assign(&mut self, name: &Token, value: Lit) -> LoxResult<()> {
    if let Some(val) = self.values.get_mut(&name.raw) {
      *val = value;
    } else if let Some(ref mut enclosing) = self.enclosing {
      enclosing.assign(name, value)?;
    } else {
      return Err(LoxError::runtime(name.clone(), format!("Undefined variable '{}'.", &name.raw)));
    }

    Ok(())
  }

  pub fn get(&self, name: &Token) -> LoxResult<Lit> {
    if let Some(lit) = self.values.get(&name.raw) {
      Ok(lit.clone())
    } else if let Some(ref enclosing) = self.enclosing {
      enclosing.get(name)
    } else {
      Err(LoxError::runtime(
        name.clone(),
        format!("Undefined variable '{}'.", &name.raw),
      ))
    }
  }

  pub fn set_enclosing(&mut self, enclosing: Environment) {
    self.enclosing = Some(Box::new(enclosing));
  }

  pub fn take_enclosing(&mut self) -> Option<Environment> {
    let mut enclosing = None;
    mem::swap(&mut enclosing, &mut self.enclosing);
    enclosing.map(|env| *env)
  }
}

pub struct Interpreter {
  environment: Environment,
}

impl Interpreter {
  pub fn new() -> Self {
    let mut environment = Environment::new(None);

    environment.define(
      "clock".to_string(),
      Lit::Func(Function::new_native(0, |_, _| {
        use std::time::{SystemTime, UNIX_EPOCH};

        Ok(Lit::Num(SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis() as f64))
      }))
    );

    Interpreter {
      environment,
    }
  }

  pub fn interpret(&mut self, statements: &Vec<Stmt>) -> LoxResult<()> {
    for statement in statements {
      self.execute(statement)?;
    }
    Ok(())
  }

  fn execute(&mut self, stmt: &Stmt) -> LoxResult<()> {
    stmt.accept(self)
  }
  pub fn execute_block(&mut self, statements: &Vec<Stmt>, mut environment: Environment) -> LoxResult<()> {
    mem::swap(&mut self.environment, &mut environment);
    self.environment.set_enclosing(environment);

    let mut iter = statements.iter();

    let res = loop {
      if let Some(stmt) = iter.next() {
        if let Err(err) = self.execute(stmt) {
          break Err(err);
        }
      } else {
        break Ok(());
      }
    };

    self.environment = self.environment.take_enclosing()?;

    res
  }

  fn evaluate(&mut self, expr: &Expr) -> LoxResult<Lit> {
    expr.accept(self)
  }

  fn is_truthy(&self, lit: &Lit) -> bool {
    match lit {
      Lit::Nil => false,
      Lit::Bool(b) => *b,
      _ => true,
    }
  }

  fn is_equal(&self, a: &Lit, b: &Lit) -> bool {
    match (a, b) {
      (Lit::Nil, Lit::Nil) => true,
      (Lit::Nil, _) => false,
      (_, Lit::Nil) => false,
      (a, b) => a == b,
    }
  }

  fn check_number_operand<F>(&self, op: &Token, a: &Lit, f: F)
    -> LoxResult<Lit>
    where F: Fn(f64) -> Lit
  {
    match a {
      Lit::Num(num) => Ok(f(*num)),
      _ => Err(LoxError::runtime(op.clone(), format!("Operand must be a number")))
    }
  }

  fn check_number_operands<F>(&self, op: &Token, a: &Lit, b: &Lit, f: F)
    -> LoxResult<Lit>
    where F: Fn(f64, f64) -> Lit
  {
    match (a, b) {
      (Lit::Num(a), Lit::Num(b)) => Ok(f(*a, *b)),
      _ => Err(LoxError::runtime(op.clone(), format!("Operands must be a numbers")))
    }
  }
}

impl ExprVisitor<LoxResult<Lit>> for Interpreter {
  fn visit(&mut self, expr: &Expr) -> LoxResult<Lit> {
    use self::Expr::*;
    use self::Lit::*;

    match *expr {
      Binary { ref left, ref op, ref right } => {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match op.ty {
          Greater => self.check_number_operands(op, &left, &right, |a, b| Bool(a > b)),
          GreaterEqual => self.check_number_operands(op, &left, &right, |a, b| Bool(a >= b)),
          Less => self.check_number_operands(op, &left, &right, |a, b| Bool(a < b)),
          LessEqual => self.check_number_operands(op, &left, &right, |a, b| Bool(a <= b)),
          BangEqual => Ok(Lit::Bool(!self.is_equal(&left, &right))),
          EqualEqual => Ok(Lit::Bool(self.is_equal(&left, &right))),
          Minus => self.check_number_operands(op, &left, &right, |a, b| Num(a - b)),
          Plus => {
            self.check_number_operands(op, &left, &right, |a, b| Num(a + b))
                .or_else(|_| match (left, right) {
                  (Str(a), Str(b)) => Ok(Str(a + &b)),
                  _ => Err(())
                })
                .or(Err(LoxError::runtime(op.clone(), format!("Operands must be numbers or strings"))))
          }
          Slash => self.check_number_operands(op, &left, &right, |a, b| Num(a / b)),
          Star => self.check_number_operands(op, &left, &right, |a, b| Num(a * b)),
          _ => Err(LoxError::runtime(op.clone(), format!("Unreachable")))
        }
      }
      Call { ref callee, ref arguments, ref paren } => {
        let callee = self.evaluate(callee)?;

        let mut args = Vec::new();
        for arg in arguments {
          args.push(self.evaluate(arg)?);
        }

        match callee {
          Func(function) => {
            if args.len() != function.arity() {
              return Err(LoxError::runtime(paren.clone(), format!("Expected {} arguments but got {}.", function.arity(), args.len())));
            }
            function.call(self, args)
          }
          _ => Err(LoxError::runtime(paren.clone(), format!("Can only call functions and classes.")))
        }
      }
      Grouping { ref expr } => {
        expr.accept(self)
      }
      Literal { ref lit } => {
        Ok(lit.clone())
      }
      Logical { ref left, ref op, ref right } => {
        let left = self.evaluate(left)?;

        if op.ty == Or && self.is_truthy(&left) {
          return Ok(left);
        } else if op.ty == And && !self.is_truthy(&left) {
          return Ok(left);
        }

        self.evaluate(right)
      }
      Unary { ref op, ref right } => {
        let right = self.evaluate(&right)?;
        match op.ty {
          Bang => Ok(Lit::Bool(!self.is_truthy(&right))),
          Minus => self.check_number_operand(op, &right, |a| Num(-a)),
          _ => Err(LoxError::runtime(op.clone(), format!("Unreachable")))
        }
      }
      Variable { ref name } => {
        self.environment.get(name)
      }
      Assign { ref name, ref value } => {
        let value = self.evaluate(value)?;
        self.environment.assign(name, value.clone())?;
        Ok(value)
      }
    }
  }
}

impl StmtVisitor<LoxResult<()>> for Interpreter {
  fn visit(&mut self, expr: &Stmt) -> LoxResult<()> {

    match expr {
      Stmt::Block { ref statements } => {
        self.execute_block(statements, Environment::new(None))?;
      }
      Stmt::Expression { ref expr } => {
        self.evaluate(expr)?;
      }
      Stmt::If { ref condition, ref then_branch, ref else_branch } => {
        let condition = self.evaluate(condition)?;
        if self.is_truthy(&condition) {
          self.execute(then_branch)?
        } else if let Some(else_branch) = else_branch {
          self.execute(else_branch)?
        }
      }
      Stmt::Print { ref expr } => {
        println!("{}", self.evaluate(expr)?);
      }
      Stmt::Var { ref name, ref init } => {
        let value = if let Some(init) = init {
          self.evaluate(init)?
        } else {
          Lit::Nil
        };
        self.environment.define(name.raw.clone(), value);
      }
      Stmt::While { ref condition, ref body } => {
        while {
          let condition = self.evaluate(condition)?;
          self.is_truthy(&condition)
        } {
          self.execute(body)?;
        }
      },
      Stmt::Function { ref name, ref params, ref body } => {
        self.environment.define(name.raw.clone(), Lit::Func(Function::new(name.raw.clone(), params.clone(), body.clone())))
      }
    }

    Ok(())
  }
}
