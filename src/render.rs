use regex::Regex;

// TODO: support file input

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SentenceStatus {
    AllRight,
    SpellingError,
    GrammarError,
    SyntaxError,
    TenseError,
    WordChoiceError,
}

impl SentenceStatus {
    fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "AllRight" => Some(Self::AllRight),
            "SpellingError" => Some(Self::SpellingError),
            "GrammarError" => Some(Self::GrammarError),
            "SyntaxError" => Some(Self::SyntaxError),
            "TenseError" => Some(Self::TenseError),
            "WordChoiceError" => Some(Self::WordChoiceError),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::AllRight => "AllRight",
            Self::SpellingError => "SpellingError",
            Self::GrammarError => "GrammarError",
            Self::SyntaxError => "SyntaxError",
            Self::TenseError => "TenseError",
            Self::WordChoiceError => "WordChoiceError",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputItem {
    pub status: SentenceStatus,
    pub fix: Option<String>,
    pub explain: Option<String>,
}

pub struct CompilerOutput {
    pub sentence: String,
    pub mistakes: Vec<OutputItem>,
}

impl CompilerOutput {
    pub fn new(sentence: impl Into<String>, mistakes: Vec<OutputItem>) -> Self {
        Self {
            sentence: sentence.into(),
            mistakes,
        }
    }
}

pub struct OutputRegex {
    error_re: Regex,
    fix_re: Regex,
    explain_re: Regex,
}

pub struct Render;

impl Render {
    const BRIGHT_GREEN: &str = "\x1b[92m";
    const BRIGHT_YELLOW: &str = "\x1b[93m";
    const BRIGHT_CYAN: &str = "\x1b[96m";
    const BRIGHT_RED: &str = "\x1b[91m";
    const RESET: &str = "\x1b[0m";

    pub fn new() -> Self {
        Self
    }

    pub fn render_item(&self, item: &OutputItem) -> Option<String> {
        match item.status {
            SentenceStatus::AllRight => Some(self.render_all_right()),
            _ => None,
        }
    }

    pub fn render_compiler_output(&self, output: &CompilerOutput) -> Option<String> {
        if output
            .mistakes
            .iter()
            .any(|item| item.status == SentenceStatus::AllRight)
        {
            return Some(self.render_all_right());
        }

        let blocks: Vec<String> = output
            .mistakes
            .iter()
            .filter_map(|item| match item.status {
                SentenceStatus::SpellingError => self.render_spelling_error(output, item),
                SentenceStatus::GrammarError => self.render_sentence_level_error(output, item),
                SentenceStatus::SyntaxError => self.render_sentence_level_error(output, item),
                SentenceStatus::TenseError => self.render_sentence_level_error(output, item),
                SentenceStatus::WordChoiceError => self.render_word_choice_error(output, item),
                _ => None,
            })
            .collect();

        if blocks.is_empty() {
            None
        } else {
            Some(blocks.join("\n\n"))
        }
    }

    pub fn render_all_right(&self) -> String {
        format!(
            "{}Congratulations, all right!{}",
            Self::BRIGHT_GREEN,
            Self::RESET
        )
    }

    fn render_spelling_error(
        &self,
        output: &CompilerOutput,
        spelling_item: &OutputItem,
    ) -> Option<String> {
        let fix = spelling_item.fix.as_deref()?;
        let (wrong, correct) = parse_spelling_fix(fix)?;

        let hit = output.sentence.find(wrong)?;
        let end = hit + wrong.len();

        let highlighted_sentence = format!(
            "{}{}{}{}{}",
            &output.sentence[..hit],
            Self::BRIGHT_RED,
            wrong,
            Self::RESET,
            &output.sentence[end..]
        );

        let caret_line = format!("{}{}^^^{}", " ".repeat(hit), Self::BRIGHT_RED, Self::RESET);

        let hint_line = format!(
            "Do you mean \"{}{}{}\"?  Not \"{}{}{}\"!",
            Self::BRIGHT_GREEN,
            correct,
            Self::RESET,
            Self::BRIGHT_RED,
            wrong,
            Self::RESET
        );

        Some(format!("{highlighted_sentence}\n{caret_line}\n{hint_line}"))
    }

    fn render_word_choice_error(
        &self,
        output: &CompilerOutput,
        word_choice_item: &OutputItem,
    ) -> Option<String> {
        let correct_sentence = word_choice_item.fix.as_deref()?.trim();
        if correct_sentence.is_empty() {
            return None;
        }

        let original_line = format!(
            "{}: {}{}{}",
            word_choice_item.status.as_str(),
            Self::BRIGHT_YELLOW,
            output.sentence,
            Self::RESET
        );
        let correct_line = format!(
            "Correct : {}{}{}",
            Self::BRIGHT_GREEN,
            correct_sentence,
            Self::RESET
        );

        let explain_line = match word_choice_item.explain.as_deref().map(str::trim) {
            Some(text) if !text.is_empty() => {
                format!("Explain : {}{}{}", Self::BRIGHT_CYAN, text, Self::RESET)
            }
            _ => format!(
                "Explain : {}(no explanation){}",
                Self::BRIGHT_CYAN,
                Self::RESET
            ),
        };

        Some(format!("{original_line}\n{correct_line}\n{explain_line}"))
    }

    fn render_sentence_level_error(
        &self,
        output: &CompilerOutput,
        item: &OutputItem,
    ) -> Option<String> {
        let correct_sentence = item.fix.as_deref()?.trim();
        if correct_sentence.is_empty() {
            return None;
        }

        let original_line = format!(
            "{}: {}{}{}",
            item.status.as_str(),
            Self::BRIGHT_RED,
            output.sentence,
            Self::RESET
        );
        let correct_line = format!(
            "Correct : {}{}{}",
            Self::BRIGHT_GREEN,
            correct_sentence,
            Self::RESET
        );
        let explain_line = match item.explain.as_deref().map(str::trim) {
            Some(text) if !text.is_empty() => {
                format!("Explain : {}{}{}", Self::BRIGHT_CYAN, text, Self::RESET)
            }
            _ => format!(
                "Explain : {}(no explanation){}",
                Self::BRIGHT_CYAN,
                Self::RESET
            ),
        };

        Some(format!("{original_line}\n{correct_line}\n{explain_line}"))
    }
}

fn parse_spelling_fix(fix: &str) -> Option<(&str, &str)> {
    let (wrong, correct) = fix.split_once("->")?;
    let wrong = wrong.trim();
    let correct = correct.trim();

    if wrong.is_empty() || correct.is_empty() {
        return None;
    }

    Some((wrong, correct))
}

impl OutputRegex {
    pub fn new() -> Result<Self, regex::Error> {
        Ok(Self {
            error_re: Regex::new(r"^Error<(?P<status>[^>]+)>$")?,
            fix_re: Regex::new(r"^Fix<(?P<fix>.*)>$")?,
            explain_re: Regex::new(r"^Explain<(?P<explain>.*)>$")?,
        })
    }

    pub fn collect_errors(&self, text: &str) -> Vec<String> {
        text.lines()
            .filter_map(|line| {
                self.error_re
                    .captures(line.trim())
                    .and_then(|caps| caps.name("status").map(|m| m.as_str().to_string()))
            })
            .collect()
    }

    pub fn collect_fixes(&self, text: &str) -> Vec<String> {
        text.lines()
            .filter_map(|line| {
                self.fix_re
                    .captures(line.trim())
                    .and_then(|caps| caps.name("fix").map(|m| m.as_str().to_string()))
            })
            .collect()
    }

    pub fn collect_explains(&self, text: &str) -> Vec<String> {
        text.lines()
            .filter_map(|line| {
                self.explain_re
                    .captures(line.trim())
                    .and_then(|caps| caps.name("explain").map(|m| m.as_str().to_string()))
            })
            .collect()
    }

    pub fn parse_items(&self, text: &str) -> Vec<OutputItem> {
        let mut items: Vec<OutputItem> = Vec::new();
        let lines: Vec<&str> = text.lines().map(str::trim).collect();
        let mut i = 0usize;

        while i < lines.len() {
            let line = lines[i];
            let Some(error_caps) = self.error_re.captures(line) else {
                i += 1;
                continue;
            };

            let status_tag = error_caps
                .name("status")
                .map(|m| m.as_str())
                .unwrap_or_default();

            if let Some(status) = SentenceStatus::from_tag(status_tag) {
                if status == SentenceStatus::AllRight {
                    items.push(OutputItem {
                        status,
                        fix: None,
                        explain: None,
                    });
                    i += 1;
                    continue;
                }

                let fix = lines
                    .get(i + 1)
                    .and_then(|line| self.fix_re.captures(line))
                    .and_then(|caps| caps.name("fix").map(|m| m.as_str().to_string()));

                let explain = lines
                    .get(i + 2)
                    .and_then(|line| self.explain_re.captures(line))
                    .and_then(|caps| caps.name("explain").map(|m| m.as_str().to_string()));

                items.push(OutputItem {
                    status,
                    fix,
                    explain,
                });
            }

            i += 3;
        }

        items
    }
}

#[cfg(test)]
mod tests {
    use super::{CompilerOutput, OutputItem, OutputRegex, Render, SentenceStatus};

    #[test]
    fn collect_fields_from_ai_output() {
        let text = r#"Error<SpellingError>
Fix<vrey -> very>
Explain<Spelling mistake>
Error<WordChoiceError>
Fix<I hate apples very much>
Explain<\"apple\" should be plural \"apples\" when referring to the fruit in general>"#;

        let parser = OutputRegex::new().expect("regex must compile");

        let errors = parser.collect_errors(text);
        let fixes = parser.collect_fixes(text);
        let explains = parser.collect_explains(text);

        assert_eq!(errors, vec!["SpellingError", "WordChoiceError"]);
        assert_eq!(fixes, vec!["vrey -> very", "I hate apples very much"]);
        assert_eq!(
            explains,
            vec![
                "Spelling mistake",
                "\\\"apple\\\" should be plural \\\"apples\\\" when referring to the fruit in general"
            ]
        );
    }

    #[test]
    fn parse_structured_items() {
        let text = r#"Error<SpellingError>
Fix<vrey -> very>
Explain<Spelling mistake>
Error<WordChoiceError>
Fix<I hate apples very much>
Explain<Use plural noun in this context>"#;

        let parser = OutputRegex::new().expect("regex must compile");
        let items = parser.parse_items(text);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].status, SentenceStatus::SpellingError);
        assert_eq!(items[0].fix.as_deref(), Some("vrey -> very"));
        assert_eq!(items[1].status, SentenceStatus::WordChoiceError);
        assert_eq!(
            items[1].explain.as_deref(),
            Some("Use plural noun in this context")
        );
    }

    #[test]
    fn render_all_right_in_bright_green() {
        let renderer = Render::new();
        let item = OutputItem {
            status: SentenceStatus::AllRight,
            fix: None,
            explain: None,
        };

        let output = renderer.render_item(&item).expect("must render all-right");

        assert_eq!(output, "\x1b[92mCongratulations, all right!\x1b[0m");
    }

    #[test]
    fn render_spelling_error_compiler_style() {
        let renderer = Render::new();
        let output = CompilerOutput::new(
            "I hate apple vrey much",
            vec![OutputItem {
                status: SentenceStatus::SpellingError,
                fix: Some("vrey -> very".to_string()),
                explain: Some("Spelling mistake".to_string()),
            }],
        );

        let rendered = renderer
            .render_compiler_output(&output)
            .expect("must render spelling error");

        assert!(rendered.contains("I hate apple \x1b[91mvrey\x1b[0m much"));
        assert!(rendered.contains("\x1b[91m^^^\x1b[0m"));
        assert!(
            rendered.contains("Do you mean \"\x1b[92mvery\x1b[0m\"?  Not \"\x1b[91mvrey\x1b[0m\"!")
        );
    }

    #[test]
    fn render_multiple_spelling_errors() {
        let renderer = Render::new();
        let output = CompilerOutput::new(
            "vrey good and hapy",
            vec![
                OutputItem {
                    status: SentenceStatus::SpellingError,
                    fix: Some("vrey -> very".to_string()),
                    explain: Some("Spelling mistake".to_string()),
                },
                OutputItem {
                    status: SentenceStatus::SpellingError,
                    fix: Some("hapy -> happy".to_string()),
                    explain: Some("Spelling mistake".to_string()),
                },
            ],
        );

        let rendered = renderer
            .render_compiler_output(&output)
            .expect("must render spelling errors");

        assert!(rendered.contains("\x1b[91mvrey\x1b[0m good and hapy"));
        assert!(rendered.contains("vrey good and \x1b[91mhapy\x1b[0m"));
        assert!(
            rendered.contains("Do you mean \"\x1b[92mvery\x1b[0m\"?  Not \"\x1b[91mvrey\x1b[0m\"!")
        );
        assert!(
            rendered
                .contains("Do you mean \"\x1b[92mhappy\x1b[0m\"?  Not \"\x1b[91mhapy\x1b[0m\"!")
        );
        assert!(rendered.contains("\n\n"));
    }

    #[test]
    fn render_word_choice_error_with_colored_sentences() {
        let renderer = Render::new();
        let output = CompilerOutput::new(
            "I hate apple very much",
            vec![OutputItem {
                status: SentenceStatus::WordChoiceError,
                fix: Some("I hate apples very much".to_string()),
                explain: Some("Use plural noun for fruit in general".to_string()),
            }],
        );

        let rendered = renderer
            .render_compiler_output(&output)
            .expect("must render word choice error");

        assert!(rendered.contains("WordChoiceError: \x1b[93mI hate apple very much\x1b[0m"));
        assert!(rendered.contains("Correct : \x1b[92mI hate apples very much\x1b[0m"));
        assert!(rendered.contains("Explain : \x1b[96mUse plural noun for fruit in general\x1b[0m"));
    }

    #[test]
    fn render_grammar_error_with_red_green_cyan() {
        let renderer = Render::new();
        let output = CompilerOutput::new(
            "He like apples",
            vec![OutputItem {
                status: SentenceStatus::GrammarError,
                fix: Some("He likes apples".to_string()),
                explain: Some("Subject-verb agreement".to_string()),
            }],
        );

        let rendered = renderer
            .render_compiler_output(&output)
            .expect("must render grammar error");

        assert!(rendered.contains("GrammarError: \x1b[91mHe like apples\x1b[0m"));
        assert!(rendered.contains("Correct : \x1b[92mHe likes apples\x1b[0m"));
        assert!(rendered.contains("Explain : \x1b[96mSubject-verb agreement\x1b[0m"));
    }
}
