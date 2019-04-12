#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Pos {
  pub line: usize,
  pub ch: usize,
  pub idx: usize,
}

impl std::fmt::Display for Pos {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}:{}", self.line, self.ch)
  }
}
