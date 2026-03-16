mod cli;

use crate::cli::Cli;

fn main() {
    let cli = Cli::parse_args();

    match cli.resolve_input() {
        Ok(content) => println!("{content}"),
        Err(error) => {
            eprintln!("failed to read input: {error}");
            std::process::exit(1);
        }
    }
}
