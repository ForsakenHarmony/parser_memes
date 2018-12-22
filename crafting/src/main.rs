#![feature(nll)]

use std::env;
use std::fs::{
    self,
    File,
};
use std::io::{
    self,
    Read,
    BufRead,
};

fn main() {
    match env::args().collect::<Vec<_>>().as_slice() {
        ([_])=> {
            // repl
            Lox::new().run_prompt();
        }
        ([_, filename]) => {
            // file
            Lox::new().run_file(filename.clone());
        }
        _ => {
            println!("Usage: rlox [script]");
            std::process::exit(1);
        }
    }
}

#[derive(Debug)]
enum TokenType {
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
    Identifier(String),
    Str(String),
    Number(f32),

    // Keywords.                                     
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF     
}

#[derive(Debug)]
struct Lox {
    had_error: bool,
}

impl Lox {
    pub fn new() -> Self {
        Lox {
            had_error: false
        }
    }

    pub fn run_file(&mut self, filename: String) {
        let path = std::path::Path::new(&filename);
        match fs::read_to_string(&path) {
            Ok(content) => {
                self.run(content);
                if self.had_error {
                    std::process::exit(1);
                }
            }
            Err(err) => {
                println!("Error reading file {:?}: {:#?}", filename, err);
            }
        }
    }

    pub fn run_prompt(&mut self) {
        let input_reader = io::BufReader::new(io::stdin());
        for line in input_reader.lines() {
            if let Ok(line) = line {
                self.run(line);
                self.had_error = false;
                print!("> ");
            }
        }
    }

    pub fn run(&mut self, source: String) {
        let scanner = Scanner::new(source); 
        let tokens = scanner.scan_tokens();

        println!("{:?}", tokens);
    }

    pub fn error(pos: Pos, message: &str) {
        Lox::report(pos, "", message);
    }

    pub fn report(pos: Pos, cause: &str, message: &str) {
        println!("[at: {}] Error{}: {}", pos, cause, message);
    }
}

#[derive(Debug)]
enum InterpError {
    UnexpectedEOF,
}

#[derive(Debug, Copy, Clone)]
struct Pos {
    line: usize,
    ch: usize,
    idx: usize
}

impl std::fmt::Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.ch)
    }
}

#[derive(Debug)]
struct Token {
    ty: TokenType,
    raw: String,
    pos: Pos,
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
        if let Some('\n') = self.chars.get(self.pos.idx -2).map(|c| *c) {
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

    pub fn idx(&self) -> usize {
        self.pos.idx
    }

    pub fn is_eol(&self) -> bool { 
        Some('\n') == self.chars.get(self.pos.idx - 1).map(|c| *c)
    }

    pub fn is_eof(&self) -> bool {
        None == self.chars.get(self.pos.idx - 1)
    }

    pub fn str_from(&self, start: &Pos) -> String { 
       self.chars.iter().map(|c| *c).skip(start.idx).take(self.pos.idx - 1).collect() 
    }

    pub fn str_from_to(&self, start: &Pos, end: &Pos) -> String { 
       self.chars.iter().map(|c| *c).skip(start.idx).take(end.idx).collect() 
    }
}

#[derive(Debug)]
struct Scanner {
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

    pub fn scan_tokens(mut self) -> Vec<Token> {
        while let Some(c) = self.stream.next() {
            self.scan_token(c);
        }

        self.tokens.push(Token::new(TokenType::EOF, String::new(), self.stream.pos()));
        self.tokens
    }

    fn scan_token(&mut self, c: char) {
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
            },
            '=' => {
                let tt = if self.match_next('=') { EqualEqual } else { Equal };
                self.add_token(tt);
            },
            '<' => {
                let tt = if self.match_next('=') { LessEqual } else { Less };
                self.add_token(tt);
            },
            '>' => {
                let tt = if self.match_next('=') { GreaterEqual} else { Greater };
                self.add_token(tt);
            },
            '/' => {
                // eat comments
                if self.match_next('/') {
                    while Some('\n') != self.stream.peek() && !self.stream.is_eof() {
                        self.stream.next();
                    }
                } else {
                    self.add_token(Slash);
                }
            },
            '"' => self.string(),
            // ignore whitespace
            ' ' | '\r' | '\t' | '\n' => {},
            '0'...'9' => self.number(),
            c => {
                Lox::error(self.stream.pos(), &format!("Unexpected character: {:?}", c)); 
            }
        }
    }

    fn string(&mut self) {
        while !self.match_next('"') && !self.stream.is_eof() {
            self.stream.next();
        } 

        if self.stream.is_eof() {
            Lox::error(self.stream.pos(), "Unterminated string.");
            return;
        }

        let mut new_start = self.start;
        new_start.idx += 1;
        let mut new_end = self.stream.pos();
        new_end.idx -= 1;

        self.add_token(TokenType::Str(self.stream.str_from_to(&new_start, &new_end)));
    }

    fn number(&mut self) {
        if let '0'...'9' = self.stream.peek() { self.stream.next(); }

        if self.stream.peek() == '.' && (if let '0'...'9' = self.stream.peek_n(2) {true} else {false}) {
            self.stream.next();
            self.stream.next();
            if let '0'...'9' = self.stream.peek() { self.stream.next(); }
        }

        self.add_token(TokenType::Number(self.stream.str_from(&self.start).parse().unwrap()));
    }

    fn add_token(&mut self, tt: TokenType) {
        let text = self.stream.str_from(&self.start);
        self.tokens.push(Token::new(tt, text, self.stream.pos()));
        self.start = self.stream.pos();
    }

    fn match_next(&mut self, expected: char) -> bool {
        if let Some(actual) = self.stream.peek() {
            if actual == expected {
                self.stream.next();
                true
            } else {
                false
            }
        }  else {
            false
        }
    }
}

