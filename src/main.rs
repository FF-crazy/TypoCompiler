mod cli;

use crate::cli::{Cli, Input};
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

    let inputs = match cli.resolve_input() {
        Ok(Input::Raw(content)) => vec![content],
        Ok(Input::Divided(parts)) => parts,
        Err(error) => {
            eprintln!("failed to read input: {error}");
            std::process::exit(1);
        }
    };

    for (index, sentence) in inputs.iter().enumerate() {
        if sentence.trim().is_empty() {
            continue;
        }

        let response = match service.post(sentence).await {
            Ok(response) => response,
            Err(error) => {
                eprintln!("failed to process sentence {}: {error}", index + 1);
                std::process::exit(1);
            }
        };

        let mistakes = parser.parse_items(&response);
        let compiler_output = CompilerOutput::new(sentence.clone(), mistakes);

        if let Some(rendered) = renderer.render_compiler_output(&compiler_output) {
            println!("{rendered}");
        } else {
            // Fallback when AI output format is unexpected.
            println!("{response}");
        }

        if index + 1 < inputs.len() {
            println!();
        }
    }
}
