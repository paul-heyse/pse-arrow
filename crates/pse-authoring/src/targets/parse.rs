// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{IndexSelector, TargetPath};
use crate::{AuthoringError, SourceSpan};

/// Parse a qualified instance/member path and an optional final index tuple.
/// # Errors
/// Empty/malformed segments, unclosed quotes/brackets, nested tuples and excessive bytes.
pub fn parse(text: &str, at: SourceSpan) -> Result<TargetPath, AuthoringError> {
    if text.len() > 65_535 {
        return Err(AuthoringError::Budget {
            limit: "target bytes",
            allowed: 65_535,
            needed: u64::try_from(text.len()).unwrap_or(u64::MAX),
        });
    }
    let trimmed = text.trim();
    let (path, indices) = if let Some((path, tail)) = trimmed.split_once('[') {
        let body = tail
            .strip_suffix(']')
            .filter(|body| !body.contains(['[', ']']))
            .ok_or_else(|| invalid(at, text))?;
        (path, selectors(body, at)?)
    } else {
        (trimmed, Vec::new())
    };
    let wildcard = path.ends_with(".*");
    let path = if wildcard {
        &path[..path.len() - 2]
    } else {
        path
    };
    if wildcard && !indices.is_empty() {
        return Err(invalid(at, text));
    }
    let names = path.split('.').map(str::to_owned).collect::<Vec<_>>();
    if names.iter().any(|name| {
        name.is_empty()
            || !name.chars().enumerate().all(|(index, ch)| {
                ch == '_' || ch.is_alphabetic() || (index > 0 && ch.is_ascii_digit())
            })
    }) {
        return Err(invalid(at, text));
    }
    Ok(TargetPath {
        names,
        indices,
        instance_wildcard: wildcard,
        at,
        text: text.to_owned(),
    })
}

fn selectors(text: &str, at: SourceSpan) -> Result<Vec<IndexSelector>, AuthoringError> {
    let mut quote = None;
    let mut start = 0;
    let mut values = Vec::new();
    for (offset, ch) in text.char_indices() {
        match (quote, ch) {
            (Some(current), ch) if current == ch => quote = None,
            (None, '\'' | '"') => quote = Some(ch),
            (None, ',') => {
                values.push(selector(&text[start..offset], at)?);
                start = offset + 1;
            }
            (_, '\\') => return Err(invalid(at, text)),
            _ => {}
        }
    }
    if quote.is_some() {
        return Err(invalid(at, text));
    }
    values.push(selector(&text[start..], at)?);
    Ok(values)
}
fn selector(text: &str, at: SourceSpan) -> Result<IndexSelector, AuthoringError> {
    let text = text.trim();
    if matches!(text, "*" | ":") {
        return Ok(IndexSelector::Wildcard);
    }
    if text.is_empty() {
        return Err(invalid(at, text));
    }
    let unquoted = text
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .or_else(|| {
            text.strip_prefix('\'')
                .and_then(|text| text.strip_suffix('\''))
        })
        .unwrap_or(text);
    if unquoted.is_empty() || (unquoted == text && text.chars().any(char::is_whitespace)) {
        return Err(invalid(at, text));
    }
    if unquoted.contains(['\u{27}', '"']) {
        return Err(invalid(at, text));
    }
    Ok(if unquoted == text {
        IndexSelector::Value(unquoted.to_owned())
    } else {
        IndexSelector::Label(unquoted.to_owned())
    })
}
fn invalid(at: SourceSpan, found: &str) -> AuthoringError {
    AuthoringError::Syntax {
        at,
        offset: at.start,
        expected: "a qualified target path with a final member tuple".to_owned(),
        found: found.to_owned(),
    }
}
