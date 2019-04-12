use crate::{
  err::LoxError,
  err::LoxResult,
  expr::Expr,
  expr::Stmt,
  lox::Lox,
  scanner::{Token, TokenType::{self, *}},
};
use crate::lit::Lit;

/*

expression     → equality ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → addition ( ( ">" | ">=" | "<" | "<=" ) addition )* ;
addition       → multiplication ( ( "-" | "+" ) multiplication )* ;
multiplication → unary ( ( "/" | "*" ) unary )* ;
unary          → ( "!" | "-" ) unary
               | primary ;
primary        → NUMBER | STRING | "false" | "true" | "nil"
               | "(" expression ")" ;

*/

pub struct Parser {
  tokens: Vec<Token>,
  current: usize,
}

impl Parser {
  pub fn new(tokens: Vec<Token>) -> Self {
    Parser {
      tokens,
      current: 0,
    }
  }

  pub fn parse(mut self) -> LoxResult<Vec<Stmt>> {
    let mut statements = Vec::new();

    while !self.at_end() {
      if let Some(stmt) = self.declaration()? {
        statements.push(stmt);
      }
    }

    Ok(statements)
  }

  fn declaration(&mut self) -> LoxResult<Option<Stmt>> {
    match {
      if self.eat(Var) {
        self.var_declaration()
      } else {
        self.statement()
      }
    } {
      Ok(stmt) => Ok(Some(stmt)),
      Err(_) => {
        self.synchronize()?;
        Ok(None)
      }
    }
  }

  fn var_declaration(&mut self) -> LoxResult<Stmt> {
    let name = match self.peek()?.ty.clone() {
      Ident(_) => self.advance()?.clone(),
      _ => return Err(LoxError::parse(self.peek()?.clone(), format!("Expected variable name."))),
    };

    let init = if self.eat(Equal) {
      Some(self.expression()?)
    } else {
      None
    };

    self.eat_or(Semicolon, format!("Expected ';' after variable declaration"))?;
    Ok(Stmt::var(name, init))
  }

  fn statement(&mut self) -> LoxResult<Stmt> {
    match () {
      _ if self.eat(For) => self.if_statement(),
      _ if self.eat(If) => self.if_statement(),
      _ if self.eat(Print) => self.print_statement(),
      _ if self.eat(While) => self.while_statement(),
      _ if self.eat(LeftBrace) => Ok(Stmt::block(self.block()?)),
      _ => self.expression_statement(),
    }
  }

  fn for_statement(&mut self) -> LoxResult<Stmt> {
    self.eat_or(LeftParen, format!("Expect '(' after 'for'."));

    let initializer = if self.eat(Semicolon) {
      None
    } else if self.eat(Var) {
      Some(self.var_declaration()?)
    } else {
      Some(self.expression_statement()?)
    };

    let condition = if self.check(&Semicolon) {
      // no condition = true
      Expr::lit(Lit::Bool(true))
    } else {
      self.expression()?
    };
    self.eat_or(Semicolon, format!("Expect ';' after loop condition."));

    let increment = if self.check(&RightParen) {
      None
    } else {
      Some(self.expression()?)
    };
    self.eat_or(Semicolon, format!("Expect ')' after for clauses."));

    let mut body = self.statement()?;

    if let Some(increment) = increment {
      body = Stmt::block(vec![
        body,
        Stmt::expression(increment)
      ]);
    }

    body = Stmt::while_stmt(condition, body);

    if let Some(init) = initializer {
      body = Stmt::block(vec![init, body])
    }

    Ok(body)
  }

  fn if_statement(&mut self) -> LoxResult<Stmt> {
    self.eat_or(LeftParen, format!("Expect '(' after 'if'."));
    let condition = self.expression()?;
    self.eat_or(RightParen, format!("Expect ')' after if condition."));

    let then_branch = self.statement()?;
    let else_branch = if self.eat(Else) {
      Some(self.statement()?)
    } else {
      None
    };

    Ok(Stmt::if_stmt(condition, then_branch, else_branch))
  }

  fn while_statement(&mut self) -> LoxResult<Stmt> {
    self.eat_or(LeftParen, format!("Expect '(' after 'while'."));
    let condition = self.expression()?;
    self.eat_or(RightParen, format!("Expect ')' after while condition."));
    let body = self.statement()?;

    Ok(Stmt::while_stmt(condition, body))
  }

  fn print_statement(&mut self) -> LoxResult<Stmt> {
    let value = self.expression()?;
    self.eat_or(Semicolon, format!("Expect ';' after value."))?;
    Ok(Stmt::print(value))
  }

  fn block(&mut self) -> LoxResult<Vec<Stmt>> {
    let mut statements = Vec::new();

    while !self.check(&RightBrace) && !self.at_end() {
      statements.push(self.declaration()??);
    }

    self.eat_or(RightBrace, format!("Expected '}}' after block."))?;
    Ok(statements)
  }

  fn expression_statement(&mut self) -> LoxResult<Stmt> {
    let expr = self.expression()?;
    self.eat_or(Semicolon, format!("Expect ';' after expression"))?;
    Ok(Stmt::expression(expr))
  }

  fn expression(&mut self) -> LoxResult<Expr> {
    self.assignment()
  }

  fn assignment(&mut self) -> LoxResult<Expr> {
    let expr = self.or()?;

    if self.eat(Equal) {
      let equals = self.previous()?.clone();
      let value = self.assignment()?;

      match expr {
        Expr::Variable { name } => {
          return Ok(Expr::assign(name, value));
        }
        _ => self.error(equals.clone(), format!("Invalid assignment target."))
      };
    }

    Ok(expr)
  }

  fn or(&mut self) -> LoxResult<Expr> {
    let mut expr = self.and()?;

    while self.eat(Or) {
      let operator = self.previous()?.clone();
      let right = self.or()?;
      expr = Expr::logical(expr, operator, right);
    }

    Ok(expr)
  }

  fn and(&mut self) -> LoxResult<Expr> {
    let mut expr = self.equality()?;

    while self.eat(And) {
      let operator = self.previous()?.clone();
      let right = self.and()?;
      expr = Expr::logical(expr, operator, right);
    }

    Ok(expr)
  }

  fn equality(&mut self) -> LoxResult<Expr> {
    let mut expr = self.comparison()?;

    while self.eat_m(&[Bang, BangEqual]) {
      let operator = self.previous()?.clone();
      let right = self.comparison()?;
      expr = Expr::binary(expr, operator, right);
    }

    Ok(expr)
  }

  fn comparison(&mut self) -> LoxResult<Expr> {
    let mut expr = self.addition()?;

    while self.eat_m(&[Greater, GreaterEqual, Less, LessEqual]) {
      let operator = self.previous()?.clone();
      let right = self.addition()?;
      expr = Expr::binary(expr, operator.clone(), right);
    }

    Ok(expr)
  }

  fn addition(&mut self) -> LoxResult<Expr> {
    let mut expr = self.multiplication()?;

    while self.eat_m(&[Minus, Plus]) {
      let operator = self.previous()?.clone();
      let right = self.multiplication()?;
      expr = Expr::binary(expr, operator, right);
    }

    Ok(expr)
  }

  fn multiplication(&mut self) -> LoxResult<Expr> {
    let mut expr = self.unary()?;

    while self.eat_m(&[Slash, Star]) {
      let operator = self.previous()?.clone();
      let right = self.unary()?;
      expr = Expr::binary(expr, operator, right);
    }

    Ok(expr)
  }

  fn unary(&mut self) -> LoxResult<Expr> {
    if self.eat_m(&[Bang, Minus]) {
      let operator = self.previous()?.clone();
      let right = self.unary()?;
      Ok(Expr::unary(operator, right))
    } else {
      self.call()
    }
  }

  fn call(&mut self) -> LoxResult<Expr> {
    let mut expr = self.primary()?;

    loop {
      if self.eat(LeftParen) {
        expr = self.finish_call(expr)?;
      } else {
        break
      }
    }

    Ok(expr)
  }

  fn finish_call(&mut self, callee: Expr) -> LoxResult<Expr> {
    let mut arguments = Vec::new();
    if !self.check(&RightParen) {
      while {
        if arguments.len() >= 8 {
          let token = self.peek()?.clone();
          return Err(self.error(token, format!("Cannot have more than 8 arguments.")))
        }
        arguments.push(self.expression()?);
        self.eat(Comma)
      } {}
    }

    self.eat_or(RightParen, format!("Expect ')' after arguments"))?;
    let paren = self.previous()?.clone();

    Ok(Expr::call(callee, paren, arguments))
  }

  fn primary(&mut self) -> LoxResult<Expr> {
    match self.advance()?.ty {
      Ident(_) => Ok(Expr::var(self.previous()?.clone())),
      Literal(ref lit) => Ok(Expr::lit(lit.clone())),
      LeftParen => {
        let expr = self.expression()?;
        self.eat_or(RightParen, format!("Expected ')' after expression."))?;
        Ok(Expr::grouping(expr))
      }
      _ => {
        let tok = self.peek()?.clone();
        Err(self.error(tok, format!("Expected expression.")))
      }
    }
  }

  fn eat(&mut self, tt: TokenType) -> bool {
    if self.check(&tt) {
      self.advance();
      true
    } else {
      false
    }
  }

  fn eat_m(&mut self, tts: &[TokenType]) -> bool {
    for tt in tts {
      if self.check(tt) {
        self.advance();
        return true;
      }
    }
    false
  }

  fn eat_or(&mut self, tt: TokenType, message: String) -> Result<(), LoxError> {
    if self.eat(tt) {
      Ok(())
    } else {
      let tok = self.peek()?.clone();
      Err(self.error(tok, message))
    }
  }

  fn error(&mut self, token: Token, message: String) -> LoxError {
    let err = LoxError::parse(token, message);
    Lox::report(err.clone());
    err
  }

  fn check(&mut self, tt: &TokenType) -> bool {
    !self.at_end() && self.peek().map_or(false, |token| &token.ty == tt)
  }

  fn advance(&mut self) -> Option<Token> {
    let token = self.peek().map(|tok| tok.clone());
    if !self.at_end() {
      self.current += 1;
    }
    token
  }

  fn at_end(&mut self) -> bool {
    self.peek().map_or(false, |token| token.ty == TokenType::EOF)
  }

  fn peek(&mut self) -> Option<&Token> {
    self.tokens.get(self.current)
  }

  fn previous(&mut self) -> Option<&Token> {
    self.tokens.get(self.current.checked_sub(1)?)
  }

  fn synchronize(&mut self) -> Result<(), LoxError> {
    self.advance();

    while !self.at_end() {
      if self.previous()?.ty == Semicolon {
        return Ok(());
      }

      match self.peek()?.ty {
        Class | Fun | Var | For | If | While | Print | Return => {
          return Ok(());
        }
        _ => {
          self.advance();
        }
      }
    }

    return Ok(());
  }
}
