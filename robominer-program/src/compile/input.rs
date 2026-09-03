use std::collections::BTreeMap;

use crate::types::{CompileError, Operator, SourceSpan, ValueType};

pub(super) fn expect_empty_call(input: &mut CompileInput) -> Result<(), CompileError> {
    if !input.eat_char('(', false) || !input.eat_char(')', false) {
        return Err(CompileError::new(format!(
            "Syntax error at line {}. '()' expected",
            input.current_line
        )));
    }

    Ok(())
}
pub(super) fn expect_char(
    input: &mut CompileInput,
    expected: char,
    message: &str,
) -> Result<(), CompileError> {
    if input.eat_char(expected, false) {
        Ok(())
    } else {
        Err(CompileError::new(format!(
            "Syntax error at line {}. {}",
            input.current_line, message
        )))
    }
}
pub(super) fn robot_property_mutation_error(line: usize) -> CompileError {
    CompileError::new(format!(
        "Error at line {line}: Robot properties cannot be changed."
    ))
}

pub(super) fn area_property_mutation_error(line: usize) -> CompileError {
    CompileError::new(format!(
        "Error at line {line}: Area properties cannot be changed."
    ))
}
pub(super) fn parse_operator_token(input: &mut CompileInput) -> Operator {
    if input.eat_char('+', false) {
        Operator::Addition
    } else if input.eat_char('-', false) {
        Operator::Subtraction
    } else if input.eat_char('*', false) {
        Operator::Multiply
    } else if input.eat_char('/', false) {
        Operator::Division
    } else if input.eat_char('%', false) {
        Operator::Mod
    } else if input.eat_sequence(">=") {
        Operator::LargerEqual
    } else if input.eat_sequence("<=") {
        Operator::SmallerEqual
    } else if input.eat_sequence("==") {
        Operator::Equal
    } else if input.eat_sequence("!=") {
        Operator::NotEqual
    } else if input.eat_char('>', false) {
        Operator::Larger
    } else if input.eat_char('<', false) {
        Operator::Smaller
    } else if input.eat_sequence("&&") {
        Operator::And
    } else if input.eat_sequence("||") {
        Operator::Or
    } else {
        Operator::Undefined
    }
}

/// Start of a construct, captured before parsing it so [`CompileInput::span_from`] can
/// turn the consumed range into a [`SourceSpan`].
#[derive(Debug, Clone, Copy)]
pub(super) struct SourceMark {
    line: usize,
    display_col: usize,
    raw_pos: usize,
}

pub(super) struct CompileInput {
    pub(super) source: Vec<char>,
    pub(super) pos: usize,
    pub(super) next_word: String,
    pub(super) current_line: usize,
    pub(super) variables: VariableStorage,
    unterminated_block_comment_line: Option<usize>,
}

impl CompileInput {
    /// Record where the construct about to be parsed starts.
    pub(super) fn mark_pos(&mut self) -> SourceMark {
        if self.next_word.is_empty() {
            self.skip_whitespace();
        }
        let raw_pos = self.consumed_pos();
        SourceMark {
            line: self.current_line,
            display_col: self.display_column_at(raw_pos),
            raw_pos,
        }
    }

    /// Span from `mark` to everything consumed since, in displayed-source columns.
    ///
    /// Constructs that span several lines cannot be represented, so they are reported as
    /// running to the end of their first line.
    pub(super) fn span_from(&self, mark: SourceMark) -> SourceSpan {
        // Lookahead via `peek` skips trailing whitespace, so walk back to the last
        // character that actually belongs to the construct.
        let end_pos = self
            .trim_whitespace_before(self.consumed_pos())
            .max(mark.raw_pos);
        let end_col = if self.line_of(end_pos) == self.line_of(mark.raw_pos) {
            self.display_column_at(end_pos)
        } else {
            self.display_column_at(self.end_of_line(mark.raw_pos))
        };

        SourceSpan {
            line: clamp_u16(mark.line),
            start_col: clamp_u16(mark.display_col),
            end_col: clamp_u16(end_col.max(mark.display_col + 1)),
        }
    }

    /// Position of the first not-yet-consumed character, ignoring the [`Self::next_word`]
    /// lookahead buffer that `pos` has already been advanced past.
    fn consumed_pos(&self) -> usize {
        self.pos.saturating_sub(self.next_word.chars().count())
    }

    /// 1-based column in the source the player sees.
    ///
    /// The compiler parses `{<source>\n}`, so on the first line the wrapping `{` occupies
    /// raw column 1 and displayed columns are one lower.
    fn display_column_at(&self, pos: usize) -> usize {
        let line_start = self.line_start(pos);
        let raw_col = pos.min(self.source.len()) - line_start + 1;
        if line_start == 0 {
            raw_col.saturating_sub(1).max(1)
        } else {
            raw_col
        }
    }

    fn line_start(&self, pos: usize) -> usize {
        self.source[..pos.min(self.source.len())]
            .iter()
            .rposition(|character| *character == '\n')
            .map_or(0, |index| index + 1)
    }

    fn end_of_line(&self, pos: usize) -> usize {
        let start = pos.min(self.source.len());
        self.source[start..]
            .iter()
            .position(|character| *character == '\n')
            .map_or(self.source.len(), |offset| start + offset)
    }

    fn trim_whitespace_before(&self, mut pos: usize) -> usize {
        pos = pos.min(self.source.len());
        while pos > 0 && self.source[pos - 1].is_whitespace() {
            pos -= 1;
        }
        pos
    }

    fn line_of(&self, pos: usize) -> usize {
        self.source[..pos.min(self.source.len())]
            .iter()
            .filter(|character| **character == '\n')
            .count()
    }

    pub(super) fn new(source: &str) -> Self {
        let mut input = Self {
            source: format!("{{{}\n}}", source).chars().collect(),
            pos: 0,
            next_word: String::new(),
            current_line: 1,
            variables: VariableStorage::default(),
            unterminated_block_comment_line: None,
        };
        input.extract_next_word();
        input
    }

    pub(super) fn use_next_word(&mut self, word: &str) -> bool {
        if self.get_next_word() == word {
            self.next_word.clear();
            true
        } else {
            false
        }
    }

    pub(super) fn use_next_word_any(&mut self) -> String {
        let word = self.get_next_word().to_owned();
        self.next_word.clear();
        word
    }

    pub(super) fn return_next_word(&mut self, word: String) {
        debug_assert!(self.next_word.is_empty());
        self.next_word = word;
    }

    pub(super) fn get_next_word(&mut self) -> &str {
        if self.next_word.is_empty() {
            self.extract_next_word();
        }
        &self.next_word
    }

    pub(super) fn eat_char(&mut self, next_char: char, repeatedly: bool) -> bool {
        let mut result = false;
        while self.peek() == Some(next_char) && (repeatedly || !result) {
            result = true;
            if !self.next_word.is_empty() {
                self.next_word.remove(0);
            } else {
                self.pos += 1;
            }
        }
        result
    }

    pub(super) fn eat_sequence(&mut self, sequence: &str) -> bool {
        if !self.next_word.is_empty() {
            if self.next_word.starts_with(sequence) {
                self.next_word.drain(..sequence.len());
                true
            } else {
                false
            }
        } else {
            self.skip_whitespace();
            let mut matched = 0;
            for expected in sequence.chars() {
                if self.source.get(self.pos + matched) == Some(&expected) {
                    matched += 1;
                } else {
                    return false;
                }
            }
            self.pos += matched;
            true
        }
    }

    pub(super) fn peek(&mut self) -> Option<char> {
        if self.next_word.is_empty() {
            self.skip_whitespace();
            self.source.get(self.pos).copied()
        } else {
            self.next_word.chars().next()
        }
    }

    pub(super) fn peek_nth(&mut self, n: usize) -> Option<char> {
        if !self.next_word.is_empty() {
            self.next_word.chars().nth(n)
        } else {
            self.skip_whitespace();
            self.source.get(self.pos + n).copied()
        }
    }

    pub(super) fn eof(&mut self) -> bool {
        self.skip_whitespace();
        self.pos >= self.source.len()
    }

    pub(super) fn extract_next_word(&mut self) {
        self.skip_whitespace();
        self.next_word.clear();

        while let Some(next) = self.source.get(self.pos).copied() {
            let can_start = next.is_ascii_alphabetic() || next == '_';
            let can_continue = can_start || (!self.next_word.is_empty() && next.is_ascii_digit());

            if can_continue {
                self.next_word.push(next);
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    pub(super) fn extract_number_literal(
        &mut self,
    ) -> Option<crate::ast::ExecutableExpressionKind> {
        self.get_next_word();

        if !self.next_word.is_empty() {
            return None;
        }

        let start = self.pos;
        let mut has_digits = false;
        let mut has_decimal = false;

        if self.source.get(self.pos) == Some(&'-') {
            self.pos += 1;
        }

        while let Some(next) = self.source.get(self.pos).copied() {
            if next.is_ascii_digit() {
                has_digits = true;
                self.pos += 1;
            } else if next == '.' && !has_decimal {
                has_decimal = true;
                self.pos += 1;
            } else {
                break;
            }
        }

        if has_digits || has_decimal {
            let value: String = self.source[start..self.pos].iter().collect();
            self.extract_next_word();
            if has_decimal {
                value
                    .parse::<f64>()
                    .ok()
                    .map(crate::ast::ExecutableExpressionKind::Float)
            } else {
                value
                    .parse::<i64>()
                    .ok()
                    .map(crate::ast::ExecutableExpressionKind::Int)
                    .or_else(|| {
                        value
                            .parse::<f64>()
                            .ok()
                            .map(crate::ast::ExecutableExpressionKind::Float)
                    })
            }
        } else {
            self.pos = start;
            None
        }
    }

    pub(super) fn skip_whitespace(&mut self) {
        loop {
            match self.source.get(self.pos).copied() {
                Some(' ' | '\t' | '\r') => self.pos += 1,
                Some('\n') => {
                    self.current_line += 1;
                    self.pos += 1;
                }
                Some('/') if self.source.get(self.pos + 1) == Some(&'/') => {
                    self.pos += 2;
                    while let Some(ch) = self.source.get(self.pos).copied() {
                        self.pos += 1;
                        if ch == '\n' {
                            self.current_line += 1;
                            break;
                        }
                    }
                }
                Some('/') if self.source.get(self.pos + 1) == Some(&'*') => {
                    self.skip_block_comment();
                }
                _ => break,
            }
        }
    }

    /// C-style block comment: `/* ... */`, including across lines. Does not nest.
    fn skip_block_comment(&mut self) {
        let start_line = self.current_line;
        self.pos += 2;
        while let Some(ch) = self.source.get(self.pos).copied() {
            if ch == '*' && self.source.get(self.pos + 1) == Some(&'/') {
                self.pos += 2;
                return;
            }
            if ch == '\n' {
                self.current_line += 1;
            }
            self.pos += 1;
        }
        if self.unterminated_block_comment_line.is_none() {
            self.unterminated_block_comment_line = Some(start_line);
        }
    }

    pub(super) fn unterminated_comment_error(&self) -> Option<CompileError> {
        self.unterminated_block_comment_line.map(|line| {
            CompileError::new(format!("Syntax error at line {line}. Unterminated comment"))
        })
    }
}

fn clamp_u16(value: usize) -> u16 {
    value.min(u16::MAX as usize) as u16
}

#[derive(Debug, Clone)]
struct Variable {
    _value_type: ValueType,
    is_const: bool,
}

#[derive(Debug, Clone, Default)]
pub(super) struct VariableStorage {
    pub(super) scope_depth: i32,
    variables: BTreeMap<i32, BTreeMap<String, Variable>>,
}

impl VariableStorage {
    pub(super) fn set_scope_depth(&mut self, depth: i32) {
        self.scope_depth = depth;
        self.variables.remove(&(self.scope_depth + 1));
    }

    pub(super) fn declare(&mut self, name: String, value_type: ValueType, is_const: bool) {
        self.variables.entry(self.scope_depth).or_default().insert(
            name,
            Variable {
                _value_type: value_type,
                is_const,
            },
        );
    }

    pub(super) fn exists_at_current_level(&self, name: &str) -> bool {
        self.variables
            .get(&self.scope_depth)
            .is_some_and(|scope| scope.contains_key(name))
    }

    pub(super) fn contains(&self, name: &str) -> bool {
        self.variables
            .values()
            .any(|scope| scope.contains_key(name))
    }

    pub(super) fn is_const(&self, name: &str) -> bool {
        self.variables
            .values()
            .filter_map(|scope| scope.get(name))
            .next_back()
            .is_some_and(|variable| variable.is_const)
    }
}
