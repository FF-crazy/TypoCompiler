use clap::Parser;
use std::fs;
use std::io;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
  #[arg(short, long, help = "Treat INPUT as a file path")]
  pub file: bool,

  #[arg(help = "Raw text, or a file path when -f is set")]
  pub input: String,
}

impl Cli {
  pub fn parse_args() -> Self {
    Self::parse()
  }

  pub fn resolve_input(&self) -> io::Result<String> {
    if self.file {
      fs::read_to_string(&self.input)
    } else {
      Ok(self.input.clone())
    }
  }
}