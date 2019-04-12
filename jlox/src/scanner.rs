use crate::{
  err::LoxError,
  err::LoxResult,
  lit::Lit,
  pos::Pos,
};

#[derive(PartialEq, Clone)]
pub enum TokenType {
  // Single-character tokens.
  LeftParen,
  RightParen,
  LeftBrace,
  RightBrace,
  Comma,
  Dot,
  Minus,
  Plus,
  Semicolon,
  Slash,
  Star,

  // One or two character tokens.
  Bang,
  BangEqual,
  Equal,
  EqualEqual,
  Greater,
  GreaterEqual,
  Less,
  LessEqual,

  // Literals.
  Literal(Lit),
  Ident(String),

  // Keywords.
  And,
  Class,
  Else,
  Fun,
  For,
  If,
  Or,
  Print,
  Return,
  Super,
  This,
  Var,
  While,

  EOF,
}

#[derive(Clone, PartialEq)]
pub struct Token {
  pub ty: TokenType,
  pub raw: String,
  pub pos: Pos,
}

impl Token {
  pub fn new(ty: TokenType, raw: String, pos: Pos) -> Self {
    Token {
      ty,
      raw,
      pos,
    }
  }
}

#[derive(Debug)]
struct CharStream {
  chars: Vec<char>,
  pos: Pos,
}

impl CharStream {
  pub fn new(source: &String) -> Self {
    CharStream {
      chars: source.chars().collect(),
      pos: Pos { line: 1, ch: 0, idx: 0 },
    }
  }

  pub fn next(&mut self) -> Option<char> {
    self.pos.idx += 1;
    self.pos.ch += 1;
    if self.pos.idx > 1 && Some('\n') == self.chars.get(self.pos.idx - 2).map(|c| *c) {
      self.pos.line += 1;
      self.pos.ch = 0;
    }
    Some(*self.chars.get(self.pos.idx - 1)?)
  }

  pub fn peek(&self) -> char {
    *self.chars.get(self.pos.idx).unwrap_or(&'\0')
  }

  pub fn peek_n(&self, n: usize) -> char {
    *self.chars.get(self.pos.idx + n).unwrap_or(&'\0')
  }

  pub fn pos(&self) -> Pos {
    self.pos
  }

  pub fn is_eof(&self) -> bool {
    None == self.chars.get(self.pos.idx - 1)
  }

  pub fn str_from(&self, start: &Pos) -> String {
    self.str_from_to(start, &self.pos)
  }

  pub fn str_from_to(&self, start: &Pos, end: &Pos) -> String {
    self.chars.iter().map(|c| *c).skip(start.idx).take(end.idx - start.idx).collect()
  }
}

pub struct Scanner {
  source: String,
  tokens: Vec<Token>,
  stream: CharStream,
  start: Pos,
}

impl Scanner {
  pub fn new(source: String) -> Self {
    let stream = CharStream::new(&source);

    Scanner {
      source,
      tokens: Vec::new(),
      start: stream.pos(),
      stream,
    }
  }

  pub fn scan_tokens(mut self) -> LoxResult<Vec<Token>> {
    while let Some(c) = self.stream.next() {
      self.scan_token(c)?;
    }

    self.tokens.push(Token::new(TokenType::EOF, String::new(), self.stream.pos()));
    Ok(self.tokens)
  }

  fn scan_token(&mut self, c: char) -> LoxResult<()> {
    use self::TokenType::*;
    match c {
      '(' => self.add_token(LeftParen),
      ')' => self.add_token(RightParen),
      '{' => self.add_token(LeftBrace),
      '}' => self.add_token(RightBrace),
      ',' => self.add_token(Comma),
      '.' => self.add_token(Dot),
      '-' => self.add_token(Minus),
      '+' => self.add_token(Plus),
      ';' => self.add_token(Semicolon),
      '*' => self.add_token(Star),
      '!' => {
        let tt = if self.match_next('=') { BangEqual } else { Bang };
        self.add_token(tt);
      }
      '=' => {
        let tt = if self.match_next('=') { EqualEqual } else { Equal };
        self.add_token(tt);
      }
      '<' => {
        let tt = if self.match_next('=') { LessEqual } else { Less };
        self.add_token(tt);
      }
      '>' => {
        let tt = if self.match_next('=') { GreaterEqual } else { Greater };
        self.add_token(tt);
      }
      '/' => {
        // eat comments
        if self.match_next('/') {
          while self.stream.peek() != '\n' && !self.stream.is_eof() {
            self.stream.next();
          }
        } else {
          self.add_token(Slash);
        }
      }
      '"' => self.string()?,
      // ignore whitespace
      ' ' | '\r' | '\t' | '\n' => {
        self.start = self.stream.pos();
      }
      c if c.is_digit(10) => self.number()?,
      c if c.is_alphanumeric() => self.identifier()?,
      c => {
        return Err(LoxError::lex(self.stream.pos(), format!("Unexpected character: {:?}", c)));
      }
    }
    Ok(())
  }

  fn string(&mut self) -> LoxResult<()> {
    while !self.match_next('"') && !self.stream.is_eof() {
      self.stream.next();
    }

    if self.stream.is_eof() {
      return Err(LoxError::lex(self.stream.pos(), format!("Unterminated string.")));
    }

    let mut new_start = self.start;
    new_start.idx += 1;
    let mut new_end = self.stream.pos();
    new_end.idx -= 1;

    self.add_token(TokenType::Literal(Lit::Str(self.stream.str_from_to(&new_start, &new_end))));

    Ok(())
  }

  fn number(&mut self) -> LoxResult<()> {
    while self.stream.peek().is_digit(10) { self.stream.next(); }

    if self.stream.peek() == '.' && (if self.stream.peek_n(2).is_digit(10) { true } else { false }) {
      self.stream.next();
      self.stream.next();
      while self.stream.peek().is_digit(10) { self.stream.next(); }
    }

    self.add_token(TokenType::Literal(Lit::Num(self.stream.str_from(&self.start).parse().unwrap())));

    Ok(())
  }

  fn identifier(&mut self) -> LoxResult<()> {
    while self.stream.peek().is_alphanumeric() {
      self.stream.next();
    }

    let ident = self.stream.str_from(&self.start);

    use self::TokenType::*;

    self.add_token(match ident.as_ref() {
      "and" => And,
      "class" => Class,
      "else" => Else,
      "false" => Literal(Lit::Bool(false)),
      "for" => For,
      "fun" => Fun,
      "if" => If,
      "nil" => Literal(Lit::Nil),
      "or" => Or,
      "print" => Print,
      "return" => Return,
      "super" => Super,
      "this" => This,
      "true" => Literal(Lit::Bool(true)),
      "var" => Var,
      "while" => While,
      _ => Ident(ident),
    });

    Ok(())
  }

  fn add_token(&mut self, tt: TokenType) {
    let text = self.stream.str_from(&self.start);
    self.tokens.push(Token::new(tt, text, self.start));
    self.start = self.stream.pos();
  }

  fn match_next(&mut self, expected: char) -> bool {
    if self.stream.peek() == expected {
      self.stream.next();
      true
    } else {
      false
    }
  }
}
