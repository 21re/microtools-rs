#[derive(Clone, Debug)]
pub struct Done;

#[derive(Clone, Debug)]
pub struct Lines(Vec<String>);

impl Lines {
  pub fn new(lines: Vec<String>) -> Lines {
    Lines(lines)
  }
}
