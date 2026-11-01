// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Owned observations of lossless Rust tokens; no cfg evaluation or semantic authority.
use super::*;
use ra_ap_syntax::AstToken as _;

/// Interpreted literal contents. Bytes preserve C/byte strings without UTF-8 coercion.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum InterpretedValue {
    /// Unicode string or documentation text.
    Text(String),
    /// Native decoded byte contents (C strings exclude the implicit terminator).
    Bytes(Vec<u8>),
    /// Unicode scalar.
    Character(char),
    /// Byte literal.
    Byte(u8),
    /// Native integer value and authored suffix, without inferring a resolved Rust type.
    Integer {
        /// Decimal magnitude interpreted by RA.
        value: String,
        /// Written numeric suffix.
        suffix: Option<String>,
    },
    /// Native underscore-free float spelling; no binary rounding or type inference is imposed.
    Float {
        /// Exact numeric spelling without suffix or underscores.
        value: String,
        /// Written numeric suffix.
        suffix: Option<String>,
    },
    /// Written boolean token.
    Boolean(bool),
}
/// Decoded UTF-8 byte interval maps to the complete original character/escape spelling.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ValueMapping {
    /// Byte range in the interpreted string.
    pub interpreted: Range<usize>,
    /// Absolute original source range, including every byte of an escape.
    pub source: SourceRange,
}
/// Authored meta tree, including unevaluated nested cfg_attr predicates.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct AttributeMeta {
    /// Native name/path when available.
    pub name: Option<String>,
    /// Exact native written meta representation.
    pub written: String,
    /// Exact source range.
    pub range: SourceRange,
    /// Written cfg_attr predicate, never an effective-build assertion.
    pub predicate: Option<String>,
    /// Nested native meta items.
    pub children: Vec<AttributeMeta>,
}
/// One token or authored attribute, associated once through its native syntax parent.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SourceFact {
    /// Native token/node kind, without a resolved symbol claim.
    pub kind: String,
    /// Exact written representation, retaining raw delimiters and trivia.
    pub written: String,
    /// Exact source extent.
    pub range: SourceRange,
    /// Closest native declaration, absent for crate-level facts.
    pub owner: Option<SourceRange>,
    /// Inner/outer attribute or doc attachment; literals have none.
    pub attachment: Option<String>,
    /// Native interpreted value, absent when invalid or unavailable.
    pub value: Option<InterpretedValue>,
    /// Native interpretation error, without discarding the written fact.
    pub interpretation_error: Option<String>,
    /// Exact decoded character mapping. C strings expose bytes but no invented char mapping.
    pub mapping: Vec<ValueMapping>,
    /// Native authored attribute meta tree.
    pub meta: Option<AttributeMeta>,
}
impl SourceFact {
    /// Map a complete decoded UTF-8 character interval to its full original spelling.
    /// Invalid/unavailable interpretations, split characters and unmapped byte strings refuse.
    pub fn map_interpreted_range(&self, selected: Range<usize>) -> Option<SourceRange> {
        if self.value.is_none() || selected.start >= selected.end {
            return None;
        }
        let first = self
            .mapping
            .iter()
            .find(|mapping| mapping.interpreted.start == selected.start)?;
        let last = self
            .mapping
            .iter()
            .find(|mapping| mapping.interpreted.end == selected.end)?;
        (first.source.byte_start <= last.source.byte_end).then_some(SourceRange {
            byte_start: first.source.byte_start,
            byte_end: last.source.byte_end,
            line_start: first.source.line_start,
            line_end: last.source.line_end,
        })
    }
}
fn range(range: ra_ap_syntax::TextRange, lines: &[usize]) -> SourceRange {
    location(lines, usize::from(range.start())..usize::from(range.end()))
}
fn meta(value: ast::Meta, lines: &[usize]) -> AttributeMeta {
    let (predicate, children) = match &value {
        ast::Meta::CfgAttrMeta(cfg) => (
            cfg.cfg_predicate()
                .map(|node| node.syntax().text().to_string()),
            cfg.metas().map(|child| meta(child, lines)).collect(),
        ),
        ast::Meta::UnsafeMeta(value) => (
            None,
            value
                .meta()
                .into_iter()
                .map(|child| meta(child, lines))
                .collect(),
        ),
        _ => (None, Vec::new()),
    };
    AttributeMeta {
        name: value
            .simple_name()
            .map(|name| name.to_string())
            .or_else(|| value.path().map(|path| path.syntax().text().to_string())),
        written: value.syntax().text().to_string(),
        range: range(value.syntax().text_range(), lines),
        predicate,
        children,
    }
}
fn mappings(value: &impl ast::IsString, lines: &[usize]) -> Vec<ValueMapping> {
    let mut output = Vec::new();
    let mut offset = 0;
    value.escaped_char_ranges(&mut |native, decoded| {
        if let Ok(character) = decoded {
            let end = offset + character.len_utf8();
            output.push(ValueMapping {
                interpreted: offset..end,
                source: range(native + value.syntax().text_range().start(), lines),
            });
            offset = end;
        }
    });
    output
}
pub(super) fn collect(root: &SyntaxNode, lines: &[usize]) -> Vec<SourceFact> {
    let mut facts = Vec::new();
    for node in root.descendants() {
        let owner = node
            .ancestors()
            .skip(1)
            .find(|node| declaration(node).is_some())
            .map(|node| range(node.text_range(), lines));
        if let Some(attribute) = ast::Attr::cast(node.clone()) {
            facts.push(SourceFact {
                kind: "attribute".into(),
                written: node.text().to_string(),
                range: range(node.text_range(), lines),
                owner,
                attachment: Some(
                    if attribute.kind().is_inner() {
                        "inner"
                    } else {
                        "outer"
                    }
                    .into(),
                ),
                value: None,
                interpretation_error: None,
                mapping: Vec::new(),
                meta: attribute.meta().map(|value| meta(value, lines)),
            });
            continue;
        }
        if let Some(doc) = ast::DocComment::cast(node.clone()) {
            let start = usize::from(node.text_range().start()) + 3;
            let mapping = doc
                .text()
                .char_indices()
                .map(|(offset, character)| ValueMapping {
                    interpreted: offset..offset + character.len_utf8(),
                    source: location(lines, start + offset..start + offset + character.len_utf8()),
                })
                .collect();
            facts.push(SourceFact {
                kind: "documentation".into(),
                written: node.text().to_string(),
                range: range(node.text_range(), lines),
                owner,
                attachment: Some(
                    if doc.kind().is_inner() {
                        "inner"
                    } else {
                        "outer"
                    }
                    .into(),
                ),
                value: Some(InterpretedValue::Text(doc.text().to_string())),
                interpretation_error: None,
                mapping,
                meta: None,
            });
            continue;
        }
        let Some(literal) = ast::Literal::cast(node.clone()) else {
            continue;
        };
        let Some(token) = literal.syntax().first_token() else {
            continue;
        };
        let mut mapping = Vec::new();
        if let Some(number) = ast::IntNumber::cast(token.clone()) {
            let (value, interpretation_error) = match number.value() {
                Ok(value) => (
                    Some(InterpretedValue::Integer {
                        value: value.to_string(),
                        suffix: number.suffix().map(str::to_owned),
                    }),
                    None,
                ),
                Err(error) => (None, Some(error.to_string())),
            };
            facts.push(SourceFact {
                kind: "integer".into(),
                written: token.text().into(),
                range: range(token.text_range(), lines),
                owner,
                attachment: None,
                value,
                interpretation_error,
                mapping,
                meta: None,
            });
            continue;
        }
        if let Some(number) = ast::FloatNumber::cast(token.clone()) {
            facts.push(SourceFact {
                kind: "float".into(),
                written: token.text().into(),
                range: range(token.text_range(), lines),
                owner,
                attachment: None,
                value: Some(InterpretedValue::Float {
                    value: number.value_string(),
                    suffix: number.suffix().map(str::to_owned),
                }),
                interpretation_error: None,
                mapping,
                meta: None,
            });
            continue;
        }
        if matches!(token.kind(), SyntaxKind::TRUE_KW | SyntaxKind::FALSE_KW) {
            facts.push(SourceFact {
                kind: "boolean".into(),
                written: token.text().into(),
                range: range(token.text_range(), lines),
                owner,
                attachment: None,
                value: Some(InterpretedValue::Boolean(
                    token.kind() == SyntaxKind::TRUE_KW,
                )),
                interpretation_error: None,
                mapping,
                meta: None,
            });
            continue;
        }
        let value = if let Some(value) = ast::String::cast(token.clone()) {
            mapping = mappings(&value, lines);
            value
                .value()
                .map(|value| InterpretedValue::Text(value.into_owned()))
        } else if let Some(value) = ast::ByteString::cast(token.clone()) {
            value
                .value()
                .map(|value| InterpretedValue::Bytes(value.into_owned()))
        } else if let Some(value) = ast::CString::cast(token.clone()) {
            value
                .value()
                .map(|value| InterpretedValue::Bytes(value.into_owned()))
        } else if let Some(value) = ast::Char::cast(token.clone()) {
            value.value().map(InterpretedValue::Character)
        } else if let Some(value) = ast::Byte::cast(token.clone()) {
            value.value().map(InterpretedValue::Byte)
        } else {
            continue;
        };
        let (value, interpretation_error) = match value {
            Ok(value) => (Some(value), None),
            Err(error) => (None, Some(format!("{error:?}"))),
        };
        if value.is_none() {
            mapping.clear();
        }
        facts.push(SourceFact {
            kind: format!("{:?}", token.kind()),
            written: token.text().to_string(),
            range: range(token.text_range(), lines),
            owner,
            attachment: None,
            value,
            interpretation_error,
            mapping,
            meta: None,
        });
    }
    facts.sort_by_key(|fact| (fact.range.byte_start, fact.range.byte_end));
    facts
}
