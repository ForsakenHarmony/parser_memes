use std::fmt::Formatter;
use std::fmt::Error;
use crate::{
  expr::Stmt,
  interpreter::Interpreter,
  err::LoxResult,
  scanner::Token
};
use crate::interpreter::Environment;
use std::cell::RefCell;

pub trait Callable {
  fn arity(&self) -> usize;
  fn call(&self, interpreter: &Interpreter, arguments: Vec<Lit>) -> LoxResult<Lit>;
  fn to_string(&self) -> String;
}

thread_local!(static NATIVE_FN_ID: RefCell<usize> = RefCell::new(0));

pub type NativeFn = fn(&Interpreter, Vec<Lit>) -> LoxResult<Lit>;

#[derive(Clone)]
struct NativeFuntion {
  body: NativeFn,
  id: usize,
}

impl NativeFuntion {
  pub fn new(body: NativeFn) -> Self {
    NativeFuntion {
      body,
      id: NATIVE_FN_ID.with(|fn_id| {
        *fn_id.borrow_mut() += 1;
        *fn_id.borrow()
      }),
    }
  }

  pub fn call(&self, interpreter: &mut Interpreter, args: Vec<Lit>) -> LoxResult<Lit> {
    (self.body)(interpreter, args)
  }
}

impl PartialEq for NativeFuntion {
  fn eq(&self, other: &NativeFuntion) -> bool {
    self.id == other.id
  }
}

#[derive(PartialEq, Clone)]
enum InternalFunc {
  Native(NativeFuntion),
  User {
    params: Vec<Token>,
    body: Vec<Stmt>,
  },
}

#[derive( PartialEq, Clone)]
pub struct Function {
  arity: usize,
  body: InternalFunc,
  name: String,
}

impl Function {
  pub fn new(
    name: String,
    params: Vec<Token>,
    body: Vec<Stmt>,
  ) -> Self {
    Function {
      arity: params.len(),
      body: InternalFunc::User {
        params,
        body,
      },
      name,
    }
  }

  pub fn new_native(arity: usize, body: NativeFn) -> Self {
    Function {
      arity,
      body: InternalFunc::Native(NativeFuntion::new(body)),
      name: "native".to_string(),
    }
  }

  pub fn arity(&self) -> usize {
    self.arity
  }

  pub fn call(&self, interpreter: &mut Interpreter, args: Vec<Lit>) -> LoxResult<Lit> {
    match self.body {
      InternalFunc::Native(ref func) => func.call(interpreter, args),
      InternalFunc::User { ref body, ref params } => {
        let mut environment = Environment::new(None);

        for (i, arg) in args.into_iter().enumerate() {
          environment.define(params.get(i)?.raw.clone(), arg)
        }

        interpreter.execute_block(body, environment);
        Ok(Lit::Nil)
      }
    }
  }

  pub fn to_string(&self) -> String {
    format!("<fn {}>", self.name)
  }
}

#[derive(PartialEq, Clone)]
pub enum Lit {
  Str(String),
  Num(f64),
  Bool(bool),
  Func(Function),
  Nil,
}

impl ::std::fmt::Display for Lit {
  fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
    match self {
      Lit::Nil => write!(f, "{}", "nil"),
      Lit::Num(num) => write!(f, "{}", num),
      Lit::Bool(b) => write!(f, "{}", b),
      Lit::Str(st) => write!(f, "{:?}", st),
      Lit::Func(func) => write!(f, "{}", func.to_string()),
    }
  }
}
