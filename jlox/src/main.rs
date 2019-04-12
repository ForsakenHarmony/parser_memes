#![feature(try_trait, bind_by_move_pattern_guards, duration_as_u128)]

mod lox;
mod scanner;
mod pos;
mod expr;
mod parser;
mod err;
mod interpreter;
mod lit;

use std::env;

use crate::lox::Lox;

fn main() {
  match env::args().collect::<Vec<_>>().as_slice() {
    [_] => {
      // repl
      if let Err(err) = Lox::new().run_prompt() {
        println!("{}", err);
        ::std::process::exit(1);
      }
    }
    [_, filename] => {
      // file
      if let Err(err) = Lox::new().run_file(filename.clone()) {
        println!("{}", err);
        ::std::process::exit(1);
      }
    }
    _ => {
      println!("Usage: rlox [script]");
      std::process::exit(1);
    }
  }
}
