mod cli;

use crate::cli::{Cli, Input};

fn main() {
    let cli = Cli::parse_args();

    match cli.resolve_input() {
        Ok(Input::Raw(content)) => println!("{content}"),
        Ok(Input::Divided(parts)) => println!("{parts:?}"),
        Err(error) => {
            eprintln!("failed to read input: {error}");
            std::process::exit(1);
        }
    }
}
