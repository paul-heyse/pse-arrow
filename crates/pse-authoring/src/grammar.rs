// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared lexical rules for authored expression and target paths.
use winnow::{
    Parser,
    error::ContextError,
    stream::{LocatingSlice, Stream},
};
pub(crate) type Input<'a> = LocatingSlice<&'a str>;
pub(crate) fn start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_' || ch == '°'
}
pub(crate) fn continuation(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '°'
}
pub(crate) fn identifier<'a>(input: &mut Input<'a>) -> Result<&'a str, ContextError> {
    (
        winnow::token::one_of(start),
        winnow::token::take_while(0.., continuation),
    )
        .take()
        .parse_next(input)
}
pub(crate) fn quoted(input: &mut Input<'_>) -> Result<String, ContextError> {
    let quote = winnow::token::one_of(['\'', '"']).parse_next(input)?;
    let mut output = String::new();
    loop {
        let ch = winnow::token::any.parse_next(input)?;
        if ch == quote {
            return Ok(output);
        }
        if ch == '\\' {
            let escaped =
                winnow::token::one_of(['\\', '\'', '"', 'n', 'r', 't']).parse_next(input)?;
            output.push(match escaped {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
        } else {
            output.push(ch);
        }
    }
}
pub(crate) fn name(input: &mut Input<'_>) -> Result<String, ContextError> {
    if input
        .peek_token()
        .is_some_and(|ch| matches!(ch, '\'' | '"'))
    {
        quoted(input)
    } else {
        identifier(input).map(str::to_owned)
    }
}
pub(crate) fn render_name(value: &str) -> String {
    if value.chars().next().is_some_and(start)
        && value.chars().all(continuation)
        && !matches!(
            value,
            "if" | "then"
                | "else"
                | "where"
                | "in"
                | "and"
                | "or"
                | "not"
                | "for"
                | "NaN"
                | "nan"
                | "Inf"
                | "inf"
                | "infinity"
        )
    {
        value.to_owned()
    } else {
        let escaped = value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        format!("\"{escaped}\"")
    }
}
