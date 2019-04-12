use std::{
  fs::{
    self,
  },
  path::{
    Path
  },
  io::{
    BufRead,
    Write,
    Error,
    BufReader,
    stdin,
    stdout
  }
};

use crate::{
  err::LoxError,
  err::LoxResult,
  interpreter::Interpreter,
  parser::Parser,
  scanner::{
    Scanner,
    TokenType,
  },
};

pub struct Lox {
  interpreter: Interpreter,
}

impl Lox {
  pub fn new() -> Self {
    Lox {
      interpreter: Interpreter::new(),
    }
  }

  pub fn run_file(&mut self, filename: String) -> Result<(), Error> {
//    let dir = env::current_dir()?;
//    Path::
    let path = Path::new(&filename);
    let content = fs::read_to_string(&path)?;
    match self.run(content) {
      Ok(_) => {}
      Err(err) => {
        Lox::report(err);
        std::process::exit(1);
      }
    }

    Ok(())
  }

  pub fn run_prompt(&mut self) -> Result<(), Error> {
    let mut stdout = stdout();
    print!("> ");
    stdout.flush()?;
    let input_reader = BufReader::new(stdin());
    for line in input_reader.lines() {
      ;
      match self.run(line?) {
        Ok(_) => {}
        Err(err) => {
          Lox::report(err);
        }
      }
      print!("> ");
      stdout.flush()?;
    }
    Ok(())
  }

  pub fn run(&mut self, source: String) -> LoxResult<()> {
    let scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;
    let parser = Parser::new(tokens);
    let statements = parser.parse()?;
    self.interpreter.interpret(&statements)?;
    Ok(())
  }

  pub fn report(err: LoxError) {
    match err {
      LoxError::ParseError { token, message } => {
        let cause = if token.ty == TokenType::EOF {
          " at end".to_string()
        } else {
          format!(" at '{}'", token.raw)
        };

        println!("[Line: {}] Error{}: {}", token.pos, cause, message);
      }
      LoxError::LexError { pos, message } => {
        println!("[Line: {}] Error: {}", pos, message);
      }
      LoxError::Other { message } => {
        println!("[??] Unexpected Error: {}", message);
      }
      LoxError::RuntimeError { token, message } => {
        println!("[Line: {}] RuntimeError: {}", token.pos, message);
      }
    }
  }
}
