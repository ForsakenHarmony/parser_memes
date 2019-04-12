use std::option::NoneError;

use crate::{
  pos::Pos,
  scanner::Token,
};

#[derive(Clone)]
pub enum LoxError {
  ParseError {
    token: Token,
    message: String,
  },
  LexError {
    pos: Pos,
    message: String,
  },
  RuntimeError {
    token: Token,
    message: String,
  },
  Other {
    message: String
  },
}

pub type LoxResult<T> = Result<T, LoxError>;

impl LoxError {
  pub fn parse(token: Token, message: String) -> Self {
    LoxError::ParseError { token, message }
  }
  pub fn lex(pos: Pos, message: String) -> Self {
    LoxError::LexError { pos, message }
  }
  pub fn other(message: String) -> Self {
    LoxError::Other { message }
  }
  pub fn runtime(token: Token, message: String) -> Self {
    LoxError::RuntimeError { token, message }
  }
}

impl From<NoneError> for LoxError {
  fn from(_: NoneError) -> Self {
    LoxError::other(format!("Unexpected None"))
  }
}
