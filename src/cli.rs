use clap::Parser;
use std::fs;
use std::io;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "Read input content from file"
    )]
    pub file: Option<String>,

    #[arg(short, long, help = "Divide into sentences")]
    pub divide: bool,

    #[arg(help = "Raw input text", required_unless_present = "file")]
    pub input: Option<String>,
}

pub enum Input {
    Raw(String),
    Divided(Vec<String>),
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn resolve_input(&self) -> io::Result<Input> {
        let content = if let Some(path) = &self.file {
            fs::read_to_string(path)?
        } else {
            self.input
                .clone()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing input text"))?
        };

        if self.divide {
            let parts = content
                .split('.')
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .map(ToOwned::to_owned)
                .collect();
            Ok(Input::Divided(parts))
        } else {
            Ok(Input::Raw(content))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_file_option_with_path() {
        let cli = Cli::try_parse_from(["typo_compiler", "-f", "./sample.txt"])
            .expect("-f should accept a file path");

        assert_eq!(cli.file.as_deref(), Some("./sample.txt"));
        assert_eq!(cli.input, None);
    }

    #[test]
    fn parse_raw_input_without_file() {
        let cli = Cli::try_parse_from(["typo_compiler", "I has an apple"])
            .expect("raw input should parse");

        assert_eq!(cli.file, None);
        assert_eq!(cli.input.as_deref(), Some("I has an apple"));
    }
}
