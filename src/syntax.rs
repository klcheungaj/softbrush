//! Tolerant Tcl source model layered on the generated ANTLR recognizer.

use std::ops::Range;

use antlr4_runtime::{CommonTokenStream, InputStream, Parser as _};

use crate::generated::tcl_lexer::TclLexer;
use crate::generated::tcl_parser::TclParser;

/// Maximum command-substitution depth parsed recursively by the tolerant scanner.
///
/// At the limit, the scanner reports the next `[` and skips that substitution
/// iteratively. This keeps editor input from exhausting the call stack while
/// allowing parsing to resume after the matching `]`.
const MAX_COMMAND_SUBSTITUTION_DEPTH: usize = 64;

/// A Tcl word and its UTF-8 byte range in the source document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Word {
    /// Source text including any Tcl quoting delimiters.
    pub text: String,
    /// Half-open UTF-8 byte range in the original source.
    pub span: Range<usize>,
}

impl Word {
    /// Returns the text when no quoting or Tcl substitutions are present.
    #[must_use]
    pub fn plain_text(&self) -> Option<&str> {
        let text = self.text.as_str();
        let is_plain =
            !text.is_empty() && !text.starts_with(['{', '"']) && !text.contains(['$', '[', '\\']);
        is_plain.then_some(text)
    }

    /// Removes one balanced outer brace or quote pair when present.
    #[must_use]
    pub fn unquoted_text(&self) -> &str {
        self.text
            .strip_prefix('{')
            .and_then(|text| text.strip_suffix('}'))
            .or_else(|| {
                self.text
                    .strip_prefix('"')
                    .and_then(|text| text.strip_suffix('"'))
            })
            .unwrap_or(&self.text)
    }
}

/// A parsed Tcl command, including nested command substitutions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Command {
    /// Words in source order.
    pub words: Vec<Word>,
    /// Half-open UTF-8 byte range of the complete command.
    pub span: Range<usize>,
    /// Command-substitution nesting depth, where zero is top level.
    pub nesting: usize,
}

impl Command {
    /// Returns a statically recognizable command name.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.words.first()?.plain_text()
    }
}

/// Categories of structural Tcl syntax failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyntaxErrorKind {
    /// A braced word has no closing brace.
    UnclosedBrace,
    /// A braced variable substitution has no closing brace.
    UnclosedVariableBrace,
    /// A command substitution has no closing bracket.
    UnclosedBracket,
    /// A command substitution exceeds the scanner's bounded recursion depth.
    CommandSubstitutionDepthExceeded,
    /// A quoted word has no closing quote.
    UnclosedQuote,
    /// A closing delimiter appears without a matching opener.
    UnexpectedCloser(char),
    /// Non-separator text follows a word where Tcl requires a separator.
    TrailingCharacters,
}

/// A recoverable syntax error associated with a source range.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyntaxError {
    /// Stable machine-readable category.
    pub kind: SyntaxErrorKind,
    /// Half-open UTF-8 byte range associated with the error.
    pub span: Range<usize>,
    /// Human-readable diagnostic message.
    pub message: String,
}

/// Tolerant syntax data retained even when the document is incomplete.
#[derive(Clone, Debug, Default)]
pub struct SyntaxModel {
    /// Top-level and nested commands in source order.
    pub commands: Vec<Command>,
    /// Comment byte ranges.
    pub comments: Vec<Range<usize>>,
    /// Quoted and braced string byte ranges.
    pub strings: Vec<Range<usize>>,
    /// Variable substitution byte ranges.
    pub variables: Vec<Range<usize>>,
    /// Structural errors emitted by the tolerant scanner.
    pub errors: Vec<SyntaxError>,
    /// Number of syntax errors reported independently by ANTLR.
    ///
    /// ANTLR is skipped, leaving this at zero, when the tolerant scanner reaches
    /// `MAX_COMMAND_SUBSTITUTION_DEPTH` so generated-parser recursion stays bounded.
    pub antlr_syntax_errors: usize,
}

/// Parses Tcl source with ANTLR and a tolerant scanner suitable for editors.
///
/// The returned model preserves usable commands and spans after syntax errors.
#[must_use]
pub fn parse(source: &str) -> SyntaxModel {
    let mut scanner = Scanner {
        source,
        position: 0,
        model: SyntaxModel::default(),
    };
    scanner.parse_script(None, 0);
    if !scanner
        .model
        .errors
        .iter()
        .any(|error| error.kind == SyntaxErrorKind::CommandSubstitutionDepthExceeded)
    {
        scanner.model.antlr_syntax_errors = antlr_error_count(source);
    }
    scanner.model
}

fn antlr_error_count(source: &str) -> usize {
    let lexer = TclLexer::new(InputStream::new(source));
    let tokens = CommonTokenStream::new(lexer);
    let mut parser = TclParser::new(tokens);
    parser.remove_error_listeners();
    let _ = parser.script();
    parser.number_of_syntax_errors()
}

struct Scanner<'source> {
    source: &'source str,
    position: usize,
    model: SyntaxModel,
}

#[derive(Clone, Copy)]
enum RecoveryMode {
    BetweenWords { at_command_start: bool },
    BareWord,
    QuotedWord,
    BracedWord { depth: usize },
}

impl Scanner<'_> {
    fn parse_script(&mut self, terminator: Option<char>, nesting: usize) -> bool {
        loop {
            self.skip_horizontal_space();
            if self.position >= self.source.len() {
                return terminator.is_none();
            }

            let current = self.current_char().expect("position is in source");
            if Some(current) == terminator {
                self.position += current.len_utf8();
                return true;
            }
            if current == '\n' || current == '\r' || current == ';' {
                self.consume_separator();
                continue;
            }
            if current == '#' {
                self.scan_comment();
                continue;
            }
            if matches!(current, ']' | '}') {
                let start = self.position;
                self.position += current.len_utf8();
                self.model.errors.push(SyntaxError {
                    kind: SyntaxErrorKind::UnexpectedCloser(current),
                    span: start..self.position,
                    message: format!("unexpected closing `{current}`"),
                });
                continue;
            }

            self.scan_command(terminator, nesting);
        }
    }

    fn scan_command(&mut self, terminator: Option<char>, nesting: usize) {
        let command_start = self.position;
        let mut words = Vec::new();
        loop {
            self.skip_horizontal_space();
            let Some(current) = self.current_char() else {
                break;
            };
            if Some(current) == terminator || matches!(current, '\n' | '\r' | ';') {
                break;
            }

            let word_start = self.position;
            match current {
                '{' => self.scan_braced_word(),
                '"' => self.scan_quoted_word(nesting),
                _ => self.scan_bare_word(terminator, nesting),
            }
            if self.position == word_start {
                self.position += current.len_utf8();
            }
            words.push(Word {
                text: self.source[word_start..self.position].to_owned(),
                span: word_start..self.position,
            });

            if matches!(current, '{' | '"')
                && self
                    .current_char()
                    .is_some_and(|next| Some(next) != terminator && !is_word_separator(next))
            {
                let start = self.position;
                let end = start + self.current_char().map_or(0, char::len_utf8);
                self.model.errors.push(SyntaxError {
                    kind: SyntaxErrorKind::TrailingCharacters,
                    span: start..end,
                    message:
                        "characters after a braced or quoted word must be separated by whitespace"
                            .to_owned(),
                });
            }
        }

        if !words.is_empty() {
            let end = words.last().map_or(command_start, |word| word.span.end);
            self.model.commands.push(Command {
                words,
                span: command_start..end,
                nesting,
            });
        }
        if self
            .current_char()
            .is_some_and(|ch| matches!(ch, '\n' | '\r' | ';'))
        {
            self.consume_separator();
        }
    }

    fn scan_braced_word(&mut self) {
        let start = self.position;
        self.position += 1;
        let mut depth = 1usize;
        while let Some(current) = self.current_char() {
            match current {
                '\\' => self.consume_escape(),
                '{' => {
                    depth += 1;
                    self.position += 1;
                }
                '}' => {
                    depth -= 1;
                    self.position += 1;
                    if depth == 0 {
                        self.model.strings.push(start..self.position);
                        return;
                    }
                }
                _ => self.position += current.len_utf8(),
            }
        }
        self.model.strings.push(start..self.position);
        self.model.errors.push(SyntaxError {
            kind: SyntaxErrorKind::UnclosedBrace,
            span: start..(start + 1),
            message: "unclosed braced word".to_owned(),
        });
    }

    fn scan_quoted_word(&mut self, nesting: usize) {
        let start = self.position;
        self.position += 1;
        while let Some(current) = self.current_char() {
            match current {
                '"' => {
                    self.position += 1;
                    self.model.strings.push(start..self.position);
                    return;
                }
                '[' => {
                    if !self.scan_command_substitution(nesting) {
                        break;
                    }
                }
                '$' => self.scan_variable(),
                '\\' => self.consume_escape(),
                _ => self.position += current.len_utf8(),
            }
        }
        self.model.strings.push(start..self.position);
        self.model.errors.push(SyntaxError {
            kind: SyntaxErrorKind::UnclosedQuote,
            span: start..(start + 1),
            message: "unclosed quoted word".to_owned(),
        });
    }

    fn scan_bare_word(&mut self, terminator: Option<char>, nesting: usize) {
        while let Some(current) = self.current_char() {
            if Some(current) == terminator || is_word_separator(current) {
                return;
            }
            match current {
                '[' => {
                    if !self.scan_command_substitution(nesting) {
                        return;
                    }
                }
                '$' => self.scan_variable(),
                '\\' => self.consume_escape(),
                _ => self.position += current.len_utf8(),
            }
        }
    }

    fn scan_variable(&mut self) {
        let start = self.position;
        self.position += 1;
        if self.current_char() == Some('{') {
            self.position += 1;
            let mut closed = false;
            while let Some(current) = self.current_char() {
                if matches!(current, '\n' | '\r') {
                    break;
                }
                self.position += current.len_utf8();
                if current == '}' {
                    closed = true;
                    break;
                }
            }
            if !closed {
                self.model.errors.push(SyntaxError {
                    kind: SyntaxErrorKind::UnclosedVariableBrace,
                    span: start..(start + 2),
                    message: "unclosed braced variable substitution".to_owned(),
                });
            }
        } else {
            while let Some(current) = self.current_char() {
                if current.is_ascii_alphanumeric() || matches!(current, '_' | ':') {
                    self.position += current.len_utf8();
                } else {
                    break;
                }
            }
            if self.current_char() == Some('(') {
                self.position += 1;
                while let Some(current) = self.current_char() {
                    match current {
                        ')' => {
                            self.position += 1;
                            break;
                        }
                        '\\' => self.consume_escape(),
                        _ => self.position += current.len_utf8(),
                    }
                }
            }
        }
        if self.position > start + 1 {
            self.model.variables.push(start..self.position);
        }
    }

    fn scan_command_substitution(&mut self, nesting: usize) -> bool {
        let bracket_start = self.position;
        self.position += 1;
        let closed = if nesting >= MAX_COMMAND_SUBSTITUTION_DEPTH {
            self.model.errors.push(SyntaxError {
                kind: SyntaxErrorKind::CommandSubstitutionDepthExceeded,
                span: bracket_start..(bracket_start + 1),
                message: format!(
                    "command substitution nesting exceeds the recovery limit of \
                     {MAX_COMMAND_SUBSTITUTION_DEPTH}"
                ),
            });
            self.skip_excessive_command_substitution()
        } else {
            self.parse_script(Some(']'), nesting + 1)
        };

        if !closed {
            self.model.errors.push(SyntaxError {
                kind: SyntaxErrorKind::UnclosedBracket,
                span: bracket_start..(bracket_start + 1),
                message: "unclosed command substitution".to_owned(),
            });
        }
        closed
    }

    /// Iteratively finds the end of a substitution that exceeded the depth limit.
    ///
    /// The recovery state mirrors the scanner's brace, quote, comment, variable,
    /// and escape handling, but deliberately omits commands from the skipped
    /// region. Heap-backed return modes replace recursive calls for deeper `[`s.
    fn skip_excessive_command_substitution(&mut self) -> bool {
        let mut mode = RecoveryMode::BetweenWords {
            at_command_start: true,
        };
        let mut return_modes = Vec::new();

        while let Some(current) = self.current_char() {
            match mode {
                RecoveryMode::BetweenWords { at_command_start } => {
                    if self.skip_recovery_between_words(
                        current,
                        at_command_start,
                        &mut mode,
                        &mut return_modes,
                    ) {
                        return true;
                    }
                }
                RecoveryMode::BareWord => {
                    if is_word_separator(current) {
                        mode = RecoveryMode::BetweenWords {
                            at_command_start: false,
                        };
                        continue;
                    }
                    match current {
                        ']' => {
                            let Some(parent_mode) =
                                self.close_recovery_substitution(&mut return_modes)
                            else {
                                return true;
                            };
                            mode = parent_mode;
                        }
                        '[' => {
                            mode = self.open_recovery_substitution(
                                &mut return_modes,
                                RecoveryMode::BareWord,
                            );
                        }
                        '$' => self.scan_variable(),
                        '\\' => self.consume_escape(),
                        _ => self.position += current.len_utf8(),
                    }
                }
                RecoveryMode::QuotedWord => match current {
                    '"' => {
                        self.position += 1;
                        mode = RecoveryMode::BetweenWords {
                            at_command_start: false,
                        };
                    }
                    '[' => {
                        mode = self.open_recovery_substitution(
                            &mut return_modes,
                            RecoveryMode::QuotedWord,
                        );
                    }
                    '$' => self.scan_variable(),
                    '\\' => self.consume_escape(),
                    _ => self.position += current.len_utf8(),
                },
                RecoveryMode::BracedWord { depth } => match current {
                    '\\' => self.consume_escape(),
                    '{' => {
                        self.position += 1;
                        mode = RecoveryMode::BracedWord { depth: depth + 1 };
                    }
                    '}' => {
                        self.position += 1;
                        mode = if depth == 1 {
                            RecoveryMode::BetweenWords {
                                at_command_start: false,
                            }
                        } else {
                            RecoveryMode::BracedWord { depth: depth - 1 }
                        };
                    }
                    _ => self.position += current.len_utf8(),
                },
            }
        }
        false
    }

    fn skip_recovery_between_words(
        &mut self,
        current: char,
        at_command_start: bool,
        mode: &mut RecoveryMode,
        return_modes: &mut Vec<RecoveryMode>,
    ) -> bool {
        match current {
            ' ' | '\t' | '\u{000c}' => self.position += 1,
            '\n' | '\r' | ';' => {
                self.consume_separator();
                *mode = RecoveryMode::BetweenWords {
                    at_command_start: true,
                };
            }
            '#' if at_command_start => self.scan_comment(),
            ']' => {
                let Some(parent_mode) = self.close_recovery_substitution(return_modes) else {
                    return true;
                };
                *mode = parent_mode;
            }
            '[' => {
                *mode = self.open_recovery_substitution(return_modes, RecoveryMode::BareWord);
            }
            '{' => {
                self.position += 1;
                *mode = RecoveryMode::BracedWord { depth: 1 };
            }
            '"' => {
                self.position += 1;
                *mode = RecoveryMode::QuotedWord;
            }
            '$' => {
                self.scan_variable();
                *mode = RecoveryMode::BareWord;
            }
            '\\' => {
                let is_continuation = self.is_line_continuation();
                self.consume_escape();
                if !is_continuation {
                    *mode = RecoveryMode::BareWord;
                }
            }
            _ => {
                self.position += current.len_utf8();
                *mode = RecoveryMode::BareWord;
            }
        }
        false
    }

    fn open_recovery_substitution(
        &mut self,
        return_modes: &mut Vec<RecoveryMode>,
        return_mode: RecoveryMode,
    ) -> RecoveryMode {
        self.position += 1;
        return_modes.push(return_mode);
        RecoveryMode::BetweenWords {
            at_command_start: true,
        }
    }

    fn close_recovery_substitution(
        &mut self,
        return_modes: &mut Vec<RecoveryMode>,
    ) -> Option<RecoveryMode> {
        self.position += 1;
        return_modes.pop()
    }

    fn scan_comment(&mut self) {
        let start = self.position;
        while let Some(current) = self.current_char() {
            if matches!(current, '\n' | '\r') {
                break;
            }
            self.position += current.len_utf8();
        }
        self.model.comments.push(start..self.position);
    }

    fn skip_horizontal_space(&mut self) {
        loop {
            match self.current_char() {
                Some(' ' | '\t' | '\u{000c}') => self.position += 1,
                Some('\\') if self.is_line_continuation() => self.consume_escape(),
                _ => return,
            }
        }
    }

    fn consume_separator(&mut self) {
        match self.current_char() {
            Some('\r') => {
                self.position += 1;
                if self.current_char() == Some('\n') {
                    self.position += 1;
                }
            }
            Some(current) => self.position += current.len_utf8(),
            None => {}
        }
    }

    fn consume_escape(&mut self) {
        self.position += 1;
        if self.current_char() == Some('\r') {
            self.position += 1;
            if self.current_char() == Some('\n') {
                self.position += 1;
            }
            while self
                .current_char()
                .is_some_and(|ch| matches!(ch, ' ' | '\t'))
            {
                self.position += 1;
            }
        } else if self.current_char() == Some('\n') {
            self.position += 1;
            while self
                .current_char()
                .is_some_and(|ch| matches!(ch, ' ' | '\t'))
            {
                self.position += 1;
            }
        } else if let Some(current) = self.current_char() {
            self.position += current.len_utf8();
        }
    }

    fn is_line_continuation(&self) -> bool {
        self.source[self.position + 1..].starts_with(['\n', '\r'])
    }

    fn current_char(&self) -> Option<char> {
        self.source[self.position..].chars().next()
    }
}

fn is_word_separator(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\u{000c}' | '\n' | '\r' | ';')
}

#[cfg(test)]
mod tests {
    use super::{MAX_COMMAND_SUBSTITUTION_DEPTH, SyntaxErrorKind, parse};

    #[test]
    fn extracts_top_level_and_nested_commands() {
        let model = parse("set clk [get_clocks $name]\nputs {hello; world}\n");
        let names = model
            .commands
            .iter()
            .filter_map(|command| command.name())
            .collect::<Vec<_>>();
        assert_eq!(names, ["get_clocks", "set", "puts"]);
        assert_eq!(model.variables.len(), 1);
        assert!(model.errors.is_empty());
        assert_eq!(model.antlr_syntax_errors, 0);
    }

    #[test]
    fn reports_unclosed_delimiters() {
        let model = parse("create_clock -period 10 [get_ports clk\n");
        assert!(
            model
                .errors
                .iter()
                .any(|error| error.kind == SyntaxErrorKind::UnclosedBracket)
        );
        assert!(model.antlr_syntax_errors > 0);
    }

    #[test]
    fn reports_unclosed_braced_variable_and_recovers_at_newline() {
        let source = "set value ${foo\nset retained yes\n";
        let variable_start = source.find("${").expect("test input has a variable");

        let model = parse(source);

        let error = model
            .errors
            .iter()
            .find(|error| error.kind == SyntaxErrorKind::UnclosedVariableBrace)
            .expect("unclosed braced variable should be reported");
        assert_eq!(error.span, variable_start..(variable_start + 2));
        assert_eq!(error.message, "unclosed braced variable substitution");
        assert!(model.antlr_syntax_errors > 0);
        assert!(model.commands.iter().any(|command| {
            command.nesting == 0
                && command.name() == Some("set")
                && command
                    .words
                    .get(1)
                    .is_some_and(|word| word.text == "retained")
        }));
    }

    #[test]
    fn bounds_deep_command_substitution_and_retains_following_command() {
        let excessive_depth = MAX_COMMAND_SUBSTITUTION_DEPTH + 4_096;
        let prefix = "set value ";
        let source = format!(
            "{prefix}{}leaf{}\nset retained yes\n",
            "[".repeat(excessive_depth),
            "]".repeat(excessive_depth)
        );

        let model = parse(&source);

        let error = model
            .errors
            .iter()
            .find(|error| error.kind == SyntaxErrorKind::CommandSubstitutionDepthExceeded)
            .expect("excessive nesting should be reported");
        let excessive_opener = prefix.len() + MAX_COMMAND_SUBSTITUTION_DEPTH;
        assert_eq!(error.span, excessive_opener..(excessive_opener + 1));
        assert_eq!(
            error.message,
            format!(
                "command substitution nesting exceeds the recovery limit of \
                 {MAX_COMMAND_SUBSTITUTION_DEPTH}"
            )
        );
        assert_eq!(model.antlr_syntax_errors, 0, "ANTLR should be skipped");
        assert!(model.commands.iter().any(|command| {
            command.nesting == 0
                && command.name() == Some("set")
                && command
                    .words
                    .get(1)
                    .is_some_and(|word| word.text == "retained")
        }));
    }

    #[test]
    fn hash_is_a_comment_only_at_command_start() {
        let model = parse("set value foo#bar\n  # real comment\n");
        assert_eq!(model.comments.len(), 1);
        assert_eq!(model.commands.len(), 1);
        assert_eq!(model.commands[0].words[2].text, "foo#bar");
    }
}
