// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-only syntax for compile-time collections and named definition applications.
use crate::dsl::{self, Expr};
use crate::{AuthoringError, ParseBudget};
use winnow::stream::LocatingSlice;

/// Static syntax embeds the existing expression language rather than duplicating arithmetic.
#[derive(Clone, Debug, PartialEq)]
pub enum StaticValue {
    /// Quoted text.
    Text(String),
    /// Explicit finite membership.
    Set(Vec<StaticValue>),
    /// Ordered compound key.
    Tuple(Vec<StaticValue>),
    /// A definition reference with named partial bindings.
    Apply {
        /// Referenced definition name.
        name: String,
        /// Named bindings in source order.
        arguments: Vec<(String, StaticValue)>,
    },
    /// Finite projection and optional selection over ordered, potentially ragged bindings.
    Comprehension {
        /// Projected value for each binding combination.
        body: Box<StaticValue>,
        /// Binder names and their domains, outermost first.
        bindings: Vec<(String, StaticValue)>,
        /// Optional selection predicate over the binders.
        filter: Option<dsl::Predicate>,
    },
    /// An ordinary expression or reference.
    Expression(Expr),
}
fn error(reason: impl Into<String>) -> AuthoringError {
    AuthoringError::Contract {
        at: None,
        reason: reason.into(),
    }
}
/// Split at a top-level token, preserving nested expression bytes.
/// # Errors
/// Unbalanced syntax or invalid expression tokens.
pub fn split(text: &str, separator: &str) -> Result<Vec<String>, AuthoringError> {
    let tokens = dsl::lexer::tokenize(text).map_err(|e| error(e.to_string()))?;
    let mut stack = Vec::new();
    let mut begin = 0;
    let mut out = Vec::new();
    for token in tokens {
        if stack.is_empty() && token.text == separator {
            out.push(text[begin..token.span.start as usize].trim().into());
            begin = token.span.end as usize;
            continue;
        }
        match token.text {
            "(" => stack.push(")"),
            "[" => stack.push("]"),
            "{" => stack.push("}"),
            ")" | "]" | "}" if stack.pop() != Some(token.text) => {
                return Err(error("unbalanced static expression"));
            }
            _ => {}
        }
        if stack.len() > ParseBudget::DEFAULT_MAX_DEPTH as usize {
            return Err(error("static expression depth"));
        }
    }
    if !stack.is_empty() {
        return Err(error("unclosed static expression"));
    }
    if begin < text.len() {
        out.push(text[begin..].trim().into());
    }
    Ok(out)
}
/// Parse bounded static syntax, including collections and partial applications.
/// # Errors
/// Invalid or over-deep syntax.
pub fn parse_static(text: &str) -> Result<StaticValue, AuthoringError> {
    parse_at(text, 0)
}
fn parse_at(text: &str, depth: u32) -> Result<StaticValue, AuthoringError> {
    if depth > ParseBudget::DEFAULT_MAX_DEPTH {
        return Err(error("static nesting limit"));
    }
    let text = text.trim();
    if text.starts_with('"') || text.starts_with('\'') {
        let mut stream = LocatingSlice::new(text);
        let value =
            crate::grammar::quoted(&mut stream).map_err(|_| error("quoted static value"))?;
        if !stream.as_ref().is_empty() {
            return Err(error("trailing quoted value"));
        }
        return Ok(StaticValue::Text(value));
    }
    if text.starts_with('{') && text.ends_with('}') {
        let parts = split(&text[1..text.len() - 1], "for")?;
        if parts.len() == 2 {
            let body = Box::new(parse_at(&parts[0], depth + 1)?);
            let clause = split(&parts[1], "where")?;
            if clause.len() > 2 {
                return Err(error("one comprehension filter required"));
            }
            let filter = clause
                .get(1)
                .map(|s| dsl::parse_predicate(s).map_err(|e| error(e.to_string())))
                .transpose()?;
            let bindings = split(&clause[0], ",")?
                .iter()
                .map(|s| {
                    let parts = split(s, "in")?;
                    if parts.len() != 2 {
                        return Err(error("set binding requires name in domain"));
                    }
                    let name = dsl::parse_expr(&parts[0]).map_err(|e| error(e.to_string()))?;
                    let dsl::ExprKind::Path(path) = name.kind else {
                        return Err(error("set binder requires a name"));
                    };
                    if path.segments.len() != 1 || !path.segments[0].indices.is_empty() {
                        return Err(error("set binder requires one name"));
                    }
                    Ok((
                        path.segments[0].name.clone(),
                        parse_at(&parts[1], depth + 1)?,
                    ))
                })
                .collect::<Result<Vec<_>, AuthoringError>>()?;
            if bindings.is_empty() {
                return Err(error("set comprehension requires a binding"));
            }
            return Ok(StaticValue::Comprehension {
                body,
                bindings,
                filter,
            });
        }
        if parts.len() > 1 {
            return Err(error("invalid set comprehension"));
        }
    }
    for (open, close, tuple) in [('{', '}', false), ('[', ']', true)] {
        if text.starts_with(open) && text.ends_with(close) {
            let values = split(&text[1..text.len() - 1], ",")?
                .iter()
                .map(|s| parse_at(s, depth + 1))
                .collect::<Result<_, _>>()?;
            return Ok(if tuple {
                StaticValue::Tuple(values)
            } else {
                StaticValue::Set(values)
            });
        }
    }
    // The expression parser owns chained calls such as partial(f,x)(value).
    // A named constructor application is considered only after ordinary syntax refuses it.
    if let Ok(expression) = dsl::parse_expr(text) {
        return Ok(StaticValue::Expression(expression));
    }
    if let Some(open) = text.find('(')
        && text.ends_with(')')
    {
        let items = split(&text[open + 1..text.len() - 1], ",")?;
        if items
            .iter()
            .any(|s| split(s, "=").is_ok_and(|v| v.len() > 1))
        {
            let arguments = items
                .iter()
                .map(|s| {
                    let parts = split(s, "=")?;
                    if parts.len() != 2 {
                        return Err(error("named binding required"));
                    }
                    Ok((parts[0].clone(), parse_at(&parts[1], depth + 1)?))
                })
                .collect::<Result<_, _>>()?;
            return Ok(StaticValue::Apply {
                name: text[..open].trim().into(),
                arguments,
            });
        }
    }
    dsl::parse_expr(text)
        .map(StaticValue::Expression)
        .map_err(|e| error(e.to_string()))
}
