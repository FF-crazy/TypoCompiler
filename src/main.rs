mod cli;

use crate::cli::Cli;
use typo_compiler::provider;
use typo_compiler::render::{CompilerOutput, OutputRegex, Render};
use typo_compiler::service::Service;

#[tokio::main]
async fn main() {
    let cli = Cli::parse_args();

    let provider = match provider::read_provider() {
        Ok(provider) => provider,
        Err(error) => {
            eprintln!("failed to load provider config: {error}");
            std::process::exit(1);
        }
    };

    let service = Service::new(provider);
    let parser = match OutputRegex::new() {
        Ok(parser) => parser,
        Err(error) => {
            eprintln!("failed to build output parser: {error}");
            std::process::exit(1);
        }
    };
    let renderer = Render::new();

    let source = cli.source_name().to_string();
    let entries = match cli.resolve_input() {
        Ok(input) => input.into_entries(),
        Err(error) => {
            eprintln!("failed to read input: {error}");
            std::process::exit(1);
        }
    };

    let mut total_errors = 0usize;

    for (index, entry) in entries.iter().enumerate() {
        if entry.sentence.trim().is_empty() {
            continue;
        }

        let response = match service.post(&entry.sentence).await {
            Ok(response) => response,
            Err(error) => {
                eprintln!("failed to process sentence {}: {error}", index + 1);
                std::process::exit(1);
            }
        };

        let mistakes = parser.parse_items(&response);
        let compiler_output =
            CompilerOutput::new(&source, entry.line, entry.sentence.clone(), mistakes);

        let blocks = renderer.render_error_blocks(&compiler_output);
        total_errors += blocks.len();

        for block in &blocks {
            println!("{block}\n");
        }
    }

    if total_errors == 0 {
        println!("{}", renderer.render_success_summary());
    } else {
        println!("{}", renderer.render_error_summary(total_errors));
    }
}
