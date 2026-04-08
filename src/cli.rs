use clap::Parser;
use regex::Regex;
use std::fs;
use std::io;
use std::sync::LazyLock;

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

pub struct InputEntry {
    pub sentence: String,
    pub line: usize,
}

pub enum Input {
    Raw(InputEntry),
    Divided(Vec<InputEntry>),
}

impl Input {
    pub fn into_entries(self) -> Vec<InputEntry> {
        match self {
            Input::Raw(entry) => vec![entry],
            Input::Divided(entries) => entries,
        }
    }
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn source_name(&self) -> &str {
        self.file.as_deref().unwrap_or("<input>")
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
            let entries = split_sentences_with_lines(&content);
            Ok(Input::Divided(entries))
        } else {
            Ok(Input::Raw(InputEntry {
                sentence: content,
                line: 1,
            }))
        }
    }
}

static ABBREV_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(Mr|Mrs|Ms|Dr|Prof|Sr|Jr|St|Gen|Gov|Sgt|Cpl|Pvt|e\.g|i\.e|vs|etc|al|approx|dept|est|inc|ltd|co|corp|no|vol|rev|Jan|Feb|Mar|Apr|Jun|Jul|Aug|Sep|Oct|Nov|Dec)\."
    ).unwrap()
});

static NUM_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d)\.(\d)").unwrap());

static SPLIT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[.!?]+\s*").unwrap());

fn split_sentences(text: &str) -> Vec<String> {
    split_sentences_with_lines(text)
        .into_iter()
        .map(|e| e.sentence)
        .collect()
}

fn split_sentences_with_lines(text: &str) -> Vec<InputEntry> {
    const PLACEHOLDER: char = '\u{FFFC}';

    let protected = ABBREV_RE.replace_all(text, |caps: &regex::Captures| {
        caps[0].replace('.', &PLACEHOLDER.to_string())
    });

    let protected = NUM_RE.replace_all(&protected, |caps: &regex::Captures| {
        format!("{}{}{}", &caps[1], PLACEHOLDER, &caps[2])
    });

    let mut entries = Vec::new();
    let mut last_end = 0;

    for mat in SPLIT_RE.find_iter(&protected) {
        let chunk = protected[last_end..mat.end()].trim();
        if !chunk.is_empty() {
            let line = protected[..last_end].matches('\n').count() + 1;
            entries.push(InputEntry {
                sentence: chunk.replace(PLACEHOLDER, "."),
                line,
            });
        }
        last_end = mat.end();
    }

    let remainder = protected[last_end..].trim();
    if !remainder.is_empty() {
        let line = protected[..last_end].matches('\n').count() + 1;
        entries.push(InputEntry {
            sentence: remainder.replace(PLACEHOLDER, "."),
            line,
        });
    }

    entries
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
    fn split_basic_sentences() {
        let result = split_sentences("I am happy. You are sad. We are fine.");
        assert_eq!(result, vec!["I am happy.", "You are sad.", "We are fine."]);
    }

    #[test]
    fn split_preserves_question_and_exclamation() {
        let result = split_sentences("Are you okay? Yes! I am fine.");
        assert_eq!(result, vec!["Are you okay?", "Yes!", "I am fine."]);
    }

    #[test]
    fn split_handles_abbreviations() {
        let result = split_sentences("Mr. Smith went to Washington. He was happy.");
        assert_eq!(
            result,
            vec!["Mr. Smith went to Washington.", "He was happy."]
        );
    }

    #[test]
    fn split_handles_eg() {
        let result =
            split_sentences("Use connectors, e.g. however and therefore. They help.");
        assert_eq!(
            result,
            vec![
                "Use connectors, e.g. however and therefore.",
                "They help."
            ]
        );
    }

    #[test]
    fn split_handles_decimals() {
        let result = split_sentences("Pi is about 3.14. That is approximate.");
        assert_eq!(
            result,
            vec!["Pi is about 3.14.", "That is approximate."]
        );
    }

    #[test]
    fn split_no_trailing_punctuation() {
        let result = split_sentences("Hello world. No period here");
        assert_eq!(result, vec!["Hello world.", "No period here"]);
    }

    #[test]
    fn split_ellipsis() {
        let result = split_sentences("Well... I guess so. Fine.");
        assert_eq!(result, vec!["Well...", "I guess so.", "Fine."]);
    }

    #[test]
    fn parse_raw_input_without_file() {
        let cli = Cli::try_parse_from(["typo_compiler", "I has an apple"])
            .expect("raw input should parse");

        assert_eq!(cli.file, None);
        assert_eq!(cli.input.as_deref(), Some("I has an apple"));
    }
}
