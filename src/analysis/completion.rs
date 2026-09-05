//! Cursor-sensitive completion using tolerant command and word boundaries.

use std::ops::Range;

use super::Analysis;
use crate::catalog::{self, Dialect};

/// Option suggestions and the complete word they should replace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptionCompletions {
    /// Half-open UTF-8 range, including the already typed hyphen.
    pub span: Range<usize>,
    /// Documented option names matching the prefix before the cursor.
    pub labels: Vec<&'static str>,
}

impl Analysis {
    /// Suggests command options at a bare argument beginning with `-`.
    ///
    /// Nested substitutions use their own command's options. Strings, comments,
    /// numeric arguments, and command names do not form option contexts. An
    /// unknown command returns an empty list rather than unrelated options.
    #[must_use]
    pub fn option_completions(&self, dialect: Dialect, offset: usize) -> Option<OptionCompletions> {
        if dialect == Dialect::Tcl {
            return None;
        }
        let command = self
            .syntax
            .commands
            .iter()
            .filter(|command| command.span.start <= offset && offset <= command.span.end)
            .max_by_key(|command| command.nesting)?;
        let word = command
            .words
            .iter()
            .skip(1)
            .find(|word| word.span.start < offset && offset <= word.span.end)?;
        let text = word.plain_text()?;
        let prefix = text.get(..offset - word.span.start)?;
        if !prefix.starts_with('-') || text.parse::<f64>().is_ok() {
            return None;
        }
        Some(OptionCompletions {
            span: word.span.clone(),
            labels: catalog::command_options(dialect, command.name()?)
                .into_iter()
                .filter(|option| option.starts_with(prefix))
                .collect(),
        })
    }
}
