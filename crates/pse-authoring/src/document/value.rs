// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Transient parser values preserve spans; generated DTOs remain the row contracts.

use pse_ids::SemanticId;
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_saphyr::{Location, Spanned};
use std::fmt;

use super::spans::{SpanIndex, child_path};
use crate::{AuthoringError, ParseBudget, SourceSpan};

#[derive(Clone, Debug)]
pub(super) enum Value {
    Null,
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    Text(String),
    List(Vec<Spanned<Self>>),
    Map(Vec<(Spanned<String>, Spanned<Self>)>),
}

impl Serialize for Value {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Null => serializer.serialize_none(),
            Self::Bool(value) => serializer.serialize_bool(*value),
            Self::I64(value) => serializer.serialize_i64(*value),
            Self::U64(value) => serializer.serialize_u64(*value),
            Self::F64(value) => serializer.serialize_f64(*value),
            Self::Text(value) => serializer.serialize_str(value),
            Self::List(values) => {
                let mut sequence = serializer.serialize_seq(Some(values.len()))?;
                for value in values {
                    sequence.serialize_element(&value.value)?;
                }
                sequence.end()
            }
            Self::Map(values) => {
                let mut map = serializer.serialize_map(Some(values.len()))?;
                for (key, value) in values {
                    map.serialize_entry(&key.value, &value.value)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(ValueVisitor)
    }
}

struct ValueVisitor;
impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a bounded authoring scalar, list or mapping")
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_none<E: serde::de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }
    fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Value, E> {
        Ok(Value::I64(value))
    }
    fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Value, E> {
        Ok(Value::U64(value))
    }
    fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Value, E> {
        if value.is_finite() {
            Ok(Value::F64(value))
        } else {
            Err(E::custom("nonfinite authored numeric value"))
        }
    }
    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Value, E> {
        Ok(Value::Text(value.to_owned()))
    }
    fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Value, E> {
        Ok(Value::Text(value))
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut access: A) -> Result<Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = access.next_element()? {
            values.push(value);
        }
        Ok(Value::List(values))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Value, A::Error> {
        let mut values = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        while let Some((key, value)) = access.next_entry::<Spanned<String>, Spanned<Value>>()? {
            if !seen.insert(key.value.clone()) {
                return Err(serde::de::Error::custom(format!(
                    "duplicate key {}",
                    key.value
                )));
            }
            values.push((key, value));
        }
        Ok(Value::Map(values))
    }
}

pub(super) fn parse_yaml(
    text: &str,
    document: SemanticId,
    budget: &ParseBudget,
) -> Result<(Spanned<Value>, SpanIndex), AuthoringError> {
    parse_yaml_accounted(text, document, budget, None)
}

pub(super) fn parse_yaml_accounted(
    text: &str,
    document: SemanticId,
    budget: &ParseBudget,
    mut allocation: Option<&mut super::allocation::Allocation<'_>>,
) -> Result<(Spanned<Value>, SpanIndex), AuthoringError> {
    let start = allocation.as_ref().map_or(0, |funds| funds.size());
    if let Some(funds) = allocation.as_deref_mut() {
        funds.grow(super::allocation::parser_extent(text, budget, true)?)?;
    }
    let parsed = parse_yaml_value(text, document, budget)?;
    let paths = super::allocation::span_extent(&parsed.value, 0)?;
    if let Some(funds) = allocation.as_deref_mut() {
        funds.grow(paths)?;
    }
    let mut spans = SpanIndex::default();
    collect_spans(&parsed, "", document, &mut spans)?;
    if let Some(funds) = allocation {
        funds.retain(
            start,
            super::allocation::add(super::allocation::value_retained(&parsed.value)?, paths)?,
        )?;
    }
    Ok((parsed, spans))
}

fn parse_yaml_value(
    text: &str,
    document: SemanticId,
    budget: &ParseBudget,
) -> Result<Spanned<Value>, AuthoringError> {
    let size = u64::try_from(text.len()).unwrap_or(u64::MAX);
    if size > budget.max_bytes {
        return Err(AuthoringError::Budget {
            limit: "bytes",
            allowed: budget.max_bytes,
            needed: size,
        });
    }
    let limit = usize::try_from(budget.max_bytes).unwrap_or(usize::MAX);
    let options = serde_saphyr::options! {
        budget:serde_saphyr::budget! {
            max_depth:usize::try_from(budget.max_depth).unwrap_or(usize::MAX),
            max_aliases:usize::try_from(budget.max_aliases).unwrap_or(usize::MAX),
            max_anchors:usize::try_from(budget.max_aliases).unwrap_or(usize::MAX),
            max_total_scalar_bytes:limit,
            max_nodes:super::allocation::yaml_node_limit(text, budget)?,
            max_events:limit.saturating_mul(4).min(1_000_000),
        },
        strict_booleans:true,
        with_snippet:false,
    };
    let parsed: Spanned<Value> = serde_saphyr::from_str_with_options(text, options)
        .map_err(|error| yaml_error(&error, document))?;
    Ok(parsed)
}

pub(super) fn yaml_error(error: &serde_saphyr::Error, document: SemanticId) -> AuthoringError {
    let start = error
        .location()
        .and_then(|location| location.span().byte_offset())
        .and_then(|offset| u32::try_from(offset).ok())
        .unwrap_or(0);
    AuthoringError::Syntax {
        at: SourceSpan::new(document, start, start),
        offset: start,
        expected: "a schema-valid bounded YAML document".to_owned(),
        found: error.to_string(),
    }
}

fn source_span(location: Location, document: SemanticId) -> Result<SourceSpan, AuthoringError> {
    let span = location.span();
    let (Some(start), Some(length)) = (span.byte_offset(), span.byte_len()) else {
        return Err(AuthoringError::Syntax {
            at: SourceSpan::head(document),
            offset: 0,
            expected: "parser byte offsets for a string document".to_owned(),
            found: "location without byte bounds".to_owned(),
        });
    };
    let end = start.saturating_add(length);
    let range = |value| {
        u32::try_from(value).map_err(|_| AuthoringError::Budget {
            limit: "source span",
            allowed: u64::from(u32::MAX),
            needed: value,
        })
    };
    Ok(SourceSpan::new(document, range(start)?, range(end)?))
}

fn node_span(node: &Spanned<Value>, document: SemanticId) -> Result<SourceSpan, AuthoringError> {
    let mut span = source_span(node.referenced, document)?;
    // Container events identify their opening position. Their extent is the union
    // of original parser token ranges; alias uses retain the reference token range.
    if node.defined != Location::UNKNOWN && node.defined != node.referenced {
        return Ok(span);
    }
    let mut include = |child: SourceSpan| {
        span.start = span.start.min(child.start);
        span.end = span.end.max(child.end);
    };
    match &node.value {
        Value::List(values) => {
            for value in values {
                include(node_span(value, document)?);
            }
        }
        Value::Map(values) => {
            for (key, value) in values {
                include(source_span(key.referenced, document)?);
                include(node_span(value, document)?);
            }
        }
        _ => {}
    }
    Ok(span)
}

fn collect_spans(
    node: &Spanned<Value>,
    path: &str,
    document: SemanticId,
    spans: &mut SpanIndex,
) -> Result<(), AuthoringError> {
    spans.insert(path.to_owned(), node_span(node, document)?);
    match &node.value {
        Value::List(values) => {
            for (index, value) in values.iter().enumerate() {
                collect_spans(
                    value,
                    &child_path(path, &index.to_string()),
                    document,
                    spans,
                )?;
            }
        }
        Value::Map(values) => {
            for (key, value) in values {
                let path = child_path(path, &key.value);
                spans.insert(
                    format!("{path}/@key"),
                    source_span(key.referenced, document)?,
                );
                collect_spans(value, &path, document, spans)?;
            }
        }
        _ => {}
    }
    Ok(())
}

impl Value {
    pub(super) fn get(&self, key: &str) -> Option<&Self> {
        match self {
            Self::Map(values) => values
                .iter()
                .find(|(name, _)| name.value == key)
                .map(|(_, value)| &value.value),
            _ => None,
        }
    }
    pub(super) fn get_mut(&mut self, key: &str) -> Option<&mut Self> {
        match self {
            Self::Map(values) => values
                .iter_mut()
                .find(|(name, _)| name.value == key)
                .map(|(_, value)| &mut value.value),
            _ => None,
        }
    }
    pub(super) fn text(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value),
            _ => None,
        }
    }
    pub(super) fn set(&mut self, key: &str, value: Self) {
        if let Some(existing) = self.get_mut(key) {
            *existing = value;
        } else if let Self::Map(values) = self {
            values.push((synthetic(key.to_owned()), synthetic(value)));
        }
    }
    pub(super) fn remove(&mut self, key: &str) -> Option<Self> {
        if let Self::Map(values) = self {
            values
                .iter()
                .position(|(name, _)| name.value == key)
                .map(|index| values.remove(index).1.value)
        } else {
            None
        }
    }
}

pub(super) fn synthetic<T>(value: T) -> Spanned<T> {
    Spanned {
        value,
        referenced: Location::UNKNOWN,
        defined: Location::UNKNOWN,
    }
}

pub(super) fn from_toml(value: toml::Value) -> Value {
    match value {
        toml::Value::String(value) => Value::Text(value),
        toml::Value::Integer(value) => Value::I64(value),
        toml::Value::Float(value) => Value::F64(value),
        toml::Value::Boolean(value) => Value::Bool(value),
        toml::Value::Datetime(value) => Value::Text(value.to_string()),
        toml::Value::Array(values) => Value::List(
            values
                .into_iter()
                .map(|value| synthetic(from_toml(value)))
                .collect(),
        ),
        toml::Value::Table(values) => Value::Map(
            values
                .into_iter()
                .map(|(key, value)| (synthetic(key), synthetic(from_toml(value))))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_are_parser_byte_ranges_even_after_unicode_and_aliases() {
        let text = "title: α\nitems:\n  - name: beta\n    x: 4\n";
        let (value, spans) = parse_yaml(
            text,
            SemanticId::from_bytes([2; 16]),
            &ParseBudget::default(),
        )
        .unwrap();
        let span = spans.span("/items/0/name").unwrap();
        assert_eq!(
            &text[usize::try_from(span.start).unwrap()..usize::try_from(span.end).unwrap()],
            "beta"
        );
        let serialized = serde_saphyr::to_string(&value.value).unwrap();
        assert!(serialized.contains("beta"));
    }

    #[test]
    fn hostile_yaml_is_refused_by_explicit_parser_budgets() {
        let budget = ParseBudget {
            max_depth: 4,
            max_aliases: 1,
            max_bytes: 1000,
        };
        for text in [
            "a: 1\na: 2",
            "a: .nan",
            "x: [[[[[[0]]]]]]",
            "a: &x [1,2]\nb: [*x,*x,*x]",
        ] {
            assert!(
                parse_yaml(text, SemanticId::NIL, &budget).is_err(),
                "{text}"
            );
        }
    }
}
