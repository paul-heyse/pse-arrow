// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::{IndexSelector, TargetPath};
use crate::{AuthoringError, SourceSpan, grammar};
use winnow::{
    Parser,
    error::ContextError,
    stream::{LocatingSlice, Location, Stream},
};

/// Parse the shared path grammar with target-only wildcard and final selector syntax.
/// # Errors
/// Malformed names, selectors, escapes, trailing input or excessive bytes.
pub fn parse(text: &str, at: SourceSpan) -> Result<TargetPath, AuthoringError> {
    if text.len() > 65_535 {
        return Err(AuthoringError::Budget {
            limit: "target bytes",
            allowed: 65_535,
            needed: u64::try_from(text.len()).unwrap_or(u64::MAX),
        });
    }
    let mut input = LocatingSlice::new(text);
    let result = target(&mut input);
    let (names, indices, instance_wildcard) = result.map_err(|_| {
        let offset = at
            .start
            .saturating_add(u32::try_from(input.current_token_start()).unwrap_or(u32::MAX));
        AuthoringError::Syntax {
            at,
            offset,
            expected: "a qualified target path with a final member tuple".into(),
            found: input.as_ref().to_string(),
        }
    })?;
    Ok(TargetPath {
        names,
        indices,
        instance_wildcard,
        at,
        text: text.to_owned(),
    })
}
fn ws(input: &mut grammar::Input<'_>) -> Result<(), ContextError> {
    winnow::ascii::multispace0.void().parse_next(input)
}
fn target(
    input: &mut grammar::Input<'_>,
) -> Result<(Vec<String>, Vec<IndexSelector>, bool), ContextError> {
    ws(input)?;
    let mut names = vec![grammar::name(input)?];
    let mut wildcard = false;
    loop {
        ws(input)?;
        if input.peek_token() != Some('.') {
            break;
        }
        '.'.parse_next(input)?;
        ws(input)?;
        if input.peek_token() == Some('*') {
            '*'.parse_next(input)?;
            wildcard = true;
            break;
        }
        names.push(grammar::name(input)?);
    }
    ws(input)?;
    let mut indices = Vec::new();
    if !wildcard && input.peek_token() == Some('[') {
        '['.parse_next(input)?;
        loop {
            ws(input)?;
            let selector = if input
                .peek_token()
                .is_some_and(|ch| matches!(ch, '\'' | '"'))
            {
                IndexSelector::Label(grammar::quoted(input)?)
            } else {
                let value: &str = winnow::token::take_while(1.., |ch: char| {
                    !ch.is_whitespace() && !matches!(ch, ',' | '[' | ']' | '\'' | '"' | '\\')
                })
                .parse_next(input)?;
                if matches!(value, "*" | ":") {
                    IndexSelector::Wildcard
                } else {
                    IndexSelector::Value(value.to_owned())
                }
            };
            indices.push(selector);
            ws(input)?;
            if input.peek_token() == Some(']') {
                ']'.parse_next(input)?;
                break;
            }
            ','.parse_next(input)?;
        }
    }
    ws(input)?;
    winnow::combinator::eof.parse_next(input)?;
    Ok((names, indices, wildcard))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_unicode_quoted_paths_and_target_only_selectors() {
        let at = SourceSpan::new(pse_ids::SemanticId::NIL, 10, 90);
        let source = "α.\"val.ve\".流量";
        let target = parse(source, at).unwrap();
        let expression = crate::dsl::parse_expr(source).unwrap();
        let crate::dsl::ExprKind::Path(path) = &expression.kind else {
            panic!("path");
        };
        assert_eq!(
            target.names,
            path.segments
                .iter()
                .map(|s| s.name.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            crate::dsl::parse_expr(&crate::dsl::render_expr(&expression))
                .unwrap()
                .kind,
            expression.kind
        );
        let target = parse("α.flow[\"a\\\"b,[]\", '*', *]", at).unwrap();
        assert_eq!(
            target.indices,
            vec![
                IndexSelector::Label("a\"b,[]".into()),
                IndexSelector::Label("*".into()),
                IndexSelector::Wildcard
            ]
        );
        assert!(parse("α.*[a]", at).is_err());
        assert!(parse("α..bad", at).is_err());
        assert!(parse("α[\"bad\\q\"]", at).is_err());
        let AuthoringError::Syntax { offset, .. } = parse("α..bad", at).unwrap_err() else {
            panic!("syntax");
        };
        assert_eq!(offset, 13);
    }
}
