// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Tokenization with offsets measured by winnow's original input stream.

use winnow::Parser;
use winnow::error::ContextError;
use winnow::stream::{LocatingSlice, Location, Stream};

use super::{DslError, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Identifier,
    Number,
    Symbol,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Token<'a> {
    pub text: &'a str,
    pub span: Span,
    pub kind: Kind,
}

pub(super) fn tokenize(text: &str) -> Result<Vec<Token<'_>>, DslError> {
    if text.len() > usize::from(u16::MAX) {
        return Err(DslError::Budget {
            limit: "bytes",
            allowed: u64::from(u16::MAX),
            needed: u64::try_from(text.len()).unwrap_or(u64::MAX),
        });
    }
    let mut input = LocatingSlice::new(text);
    let mut tokens = Vec::new();
    loop {
        winnow::ascii::multispace0::<_, ContextError>
            .parse_next(&mut input)
            .map_err(|_| syntax(offset(&input), "whitespace", "invalid whitespace"))?;
        let Some(first) = input.peek_token() else {
            break;
        };
        let start = offset(&input);
        let (token, kind) = if first.is_ascii_digit()
            || (first == '.'
                && input
                    .as_ref()
                    .chars()
                    .nth(1)
                    .is_some_and(|next| next.is_ascii_digit()))
        {
            let token = winnow::ascii::float::<_, f64, ContextError>
                .take()
                .parse_next(&mut input)
                .map_err(|_| syntax(start, "finite decimal number", input.as_ref()))?;
            if !token.parse::<f64>().is_ok_and(f64::is_finite) {
                return Err(DslError::NonFiniteNumber { offset: start });
            }
            (token, Kind::Number)
        } else if first.is_alphabetic() || first == '_' || first == '°' {
            let token = winnow::token::take_while::<_, _, ContextError>(1.., |ch: char| {
                ch.is_alphanumeric() || ch == '_' || ch == '°'
            })
            .parse_next(&mut input)
            .map_err(|_| syntax(start, "identifier", input.as_ref()))?;
            if matches!(token, "NaN" | "nan" | "Inf" | "inf" | "infinity") {
                return Err(DslError::NonFiniteNumber { offset: start });
            }
            (token, Kind::Identifier)
        } else {
            let width = if ["==", "!=", "<=", ">="]
                .iter()
                .any(|value| input.as_ref().starts_with(value))
            {
                2
            } else if "+-*/^()[]{},.|=<>".contains(first) {
                first.len_utf8()
            } else {
                return Err(syntax(start, "expression token", &first.to_string()));
            };
            (input.next_slice(width), Kind::Symbol)
        };
        tokens.push(Token {
            text: token,
            span: Span {
                start,
                end: offset(&input),
            },
            kind,
        });
    }
    Ok(tokens)
}

fn offset(input: &LocatingSlice<&str>) -> u32 {
    u32::try_from(input.current_token_start()).unwrap_or(u32::MAX)
}

pub(super) fn syntax(offset: u32, expected: &str, found: &str) -> DslError {
    DslError::Syntax {
        offset,
        span: Span {
            start: offset,
            end: offset.saturating_add(u32::try_from(found.len()).unwrap_or(u32::MAX)),
        },
        expected: expected.to_owned(),
        found: found.to_owned(),
    }
}
