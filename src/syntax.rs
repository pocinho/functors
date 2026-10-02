use std::path::Path;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SyntaxLanguage {
    #[default]
    Auto,
    PlainText,
    Rust,
    Markdown,
}

impl SyntaxLanguage {
    pub fn from_extension(extension: Option<&str>) -> Self {
        match extension {
            Some("rs") => Self::Rust,
            Some("md" | "markdown") => Self::Markdown,
            None => Self::Auto,
            _ => Self::PlainText,
        }
    }

    pub fn from_path(path: &Path) -> Self {
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase);
        Self::from_extension(extension.as_deref())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HighlightKind {
    Keyword,
    String,
    Comment,
    Number,
    Punctuation,
    Heading,
    Emphasis,
    Code,
    Link,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HighlightSpan {
    pub start: usize,
    pub end: usize,
    pub kind: HighlightKind,
}

pub fn highlight_line(text: &str, language: SyntaxLanguage) -> Vec<HighlightSpan> {
    match language {
        SyntaxLanguage::Auto => highlight_line(text, detect_language(text)),
        SyntaxLanguage::PlainText => Vec::new(),
        SyntaxLanguage::Rust => highlight_rust(text),
        SyntaxLanguage::Markdown => highlight_markdown(text),
    }
}

fn detect_language(text: &str) -> SyntaxLanguage {
    let trimmed = text.trim_start();
    if trimmed.starts_with('#') || text.contains('`') || text.contains("](") {
        SyntaxLanguage::Markdown
    } else if text
        .split(|character: char| !is_identifier_continue(character))
        .any(|identifier| {
            !identifier.is_empty() && is_rust_keyword(&identifier.chars().collect::<Vec<_>>())
        })
    {
        SyntaxLanguage::Rust
    } else {
        SyntaxLanguage::PlainText
    }
}

fn highlight_rust(text: &str) -> Vec<HighlightSpan> {
    let characters = text.chars().collect::<Vec<_>>();
    let mut spans = Vec::new();
    let mut index = 0;

    while index < characters.len() {
        if characters[index] == '/' && characters.get(index + 1) == Some(&'/') {
            spans.push(HighlightSpan {
                start: index,
                end: characters.len(),
                kind: HighlightKind::Comment,
            });
            break;
        }

        if characters[index] == '"' {
            let start = index;
            index += 1;
            while index < characters.len() {
                let escaped = index > start && characters[index - 1] == '\\';
                if characters[index] == '"' && !escaped {
                    index += 1;
                    break;
                }
                index += 1;
            }
            spans.push(HighlightSpan {
                start,
                end: index,
                kind: HighlightKind::String,
            });
            continue;
        }

        if characters[index].is_ascii_digit() {
            let start = index;
            index += 1;
            while index < characters.len()
                && (characters[index].is_ascii_digit() || characters[index] == '.')
            {
                index += 1;
            }
            spans.push(HighlightSpan {
                start,
                end: index,
                kind: HighlightKind::Number,
            });
            continue;
        }

        if is_identifier_start(characters[index]) {
            let start = index;
            index += 1;
            while index < characters.len() && is_identifier_continue(characters[index]) {
                index += 1;
            }
            if is_rust_keyword(&characters[start..index]) {
                spans.push(HighlightSpan {
                    start,
                    end: index,
                    kind: HighlightKind::Keyword,
                });
            }
            continue;
        }

        if characters[index].is_ascii_punctuation() && characters[index] != '_' {
            spans.push(HighlightSpan {
                start: index,
                end: index + 1,
                kind: HighlightKind::Punctuation,
            });
        }
        index += 1;
    }

    spans
}

fn highlight_markdown(text: &str) -> Vec<HighlightSpan> {
    let characters = text.chars().collect::<Vec<_>>();
    let trimmed_start = characters
        .iter()
        .position(|character| !character.is_whitespace())
        .unwrap_or(characters.len());
    let mut spans = Vec::new();

    if characters[trimmed_start..].starts_with(&['#']) {
        spans.push(HighlightSpan {
            start: trimmed_start,
            end: characters.len(),
            kind: HighlightKind::Heading,
        });
        return spans;
    }

    let mut index = 0;
    while index < characters.len() {
        if characters[index] == '`' {
            let start = index;
            index += 1;
            while index < characters.len() && characters[index] != '`' {
                index += 1;
            }
            if index < characters.len() {
                index += 1;
            }
            spans.push(HighlightSpan {
                start,
                end: index,
                kind: HighlightKind::Code,
            });
        } else if characters[index] == '[' {
            if let Some(close) = characters[index + 1..]
                .iter()
                .position(|character| *character == ']')
            {
                let close = index + 1 + close;
                if characters.get(close + 1) == Some(&'(')
                    && let Some(end) = characters[close + 2..]
                        .iter()
                        .position(|character| *character == ')')
                {
                    spans.push(HighlightSpan {
                        start: index,
                        end: close + 3 + end,
                        kind: HighlightKind::Link,
                    });
                    index = close + 3 + end;
                    continue;
                }
            }
            index += 1;
        } else if characters[index] == '*' || characters[index] == '_' {
            let marker = characters[index];
            let start = index;
            index += 1;
            while index < characters.len() && characters[index] != marker {
                index += 1;
            }
            if index < characters.len() {
                index += 1;
                spans.push(HighlightSpan {
                    start,
                    end: index,
                    kind: HighlightKind::Emphasis,
                });
            }
        } else {
            index += 1;
        }
    }

    spans
}

fn is_identifier_start(character: char) -> bool {
    character == '_' || character.is_ascii_alphabetic()
}

fn is_identifier_continue(character: char) -> bool {
    character == '_' || character.is_ascii_alphanumeric()
}

fn is_rust_keyword(identifier: &[char]) -> bool {
    matches!(
        identifier,
        ['a', 's']
            | ['a', 's', 'y', 'n', 'c']
            | ['a', 'w', 'a', 'i', 't']
            | ['b', 'r', 'e', 'a', 'k']
            | ['c', 'o', 'n', 's', 't']
            | ['c', 'r', 'a', 't', 'e']
            | ['d', 'y', 'n']
            | ['e', 'l', 's', 'e']
            | ['e', 'n', 'u', 'm']
            | ['e', 'x', 't', 'e', 'r', 'n']
            | ['f', 'n']
            | ['f', 'o', 'r']
            | ['i', 'f']
            | ['i', 'm', 'p', 'l']
            | ['i', 'n']
            | ['l', 'e', 't']
            | ['l', 'o', 'o', 'p']
            | ['m', 'a', 't', 'c', 'h']
            | ['m', 'o', 'd']
            | ['m', 'o', 'v', 'e']
            | ['m', 'u', 't']
            | ['p', 'u', 'b']
            | ['r', 'e', 'f']
            | ['r', 'e', 't', 'u', 'r', 'n']
            | ['s', 't', 'r', 'u', 'c', 't']
            | ['s', 'u', 'p', 'e', 'r']
            | ['t', 'r', 'a', 'i', 't']
            | ['t', 'y', 'p', 'e']
            | ['u', 'n', 's', 'a', 'f', 'e']
            | ['u', 's', 'e']
            | ['w', 'h', 'e', 'r', 'e']
            | ['w', 'h', 'i', 'l', 'e']
    )
}

#[cfg(test)]
mod tests {
    use super::{HighlightKind, SyntaxLanguage, highlight_line};

    #[test]
    fn rust_highlighting_marks_keywords_strings_numbers_and_comments() {
        let spans = highlight_line("fn main() { let value = 42; // note", SyntaxLanguage::Rust);

        assert!(spans.iter().any(|span| span.kind == HighlightKind::Keyword));
        assert!(spans.iter().any(|span| span.kind == HighlightKind::Number));
        assert!(spans.iter().any(|span| span.kind == HighlightKind::Comment));
    }

    #[test]
    fn markdown_highlighting_marks_headings_code_and_links() {
        assert_eq!(
            highlight_line("# Title", SyntaxLanguage::Markdown)[0].kind,
            HighlightKind::Heading
        );
        let spans = highlight_line(
            "See `code` [link](https://example.com)",
            SyntaxLanguage::Markdown,
        );
        assert!(spans.iter().any(|span| span.kind == HighlightKind::Code));
        assert!(spans.iter().any(|span| span.kind == HighlightKind::Link));
    }

    #[test]
    fn plain_text_has_no_highlight_spans() {
        assert!(highlight_line("plain", SyntaxLanguage::PlainText).is_empty());
    }

    #[test]
    fn auto_language_detects_rust_and_markdown_examples() {
        assert!(!highlight_line("fn main() {}", SyntaxLanguage::Auto).is_empty());
        assert!(!highlight_line("# Heading", SyntaxLanguage::Auto).is_empty());
        assert!(highlight_line("ordinary prose", SyntaxLanguage::Auto).is_empty());
    }
}
