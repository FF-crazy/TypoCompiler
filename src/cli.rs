use clap::Parser;
use std::fs;
use std::io;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, help = "Treat INPUT as a file path")]
    pub file: bool,

    #[arg(short, long, help = "Divide into sentences")]
    pub divide: bool,

    #[arg(help = "Raw text, or a file path when -f is set")]
    pub input: String,
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
        let content = if self.file {
            fs::read_to_string(&self.input)?
        } else {
            self.input.clone()
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
