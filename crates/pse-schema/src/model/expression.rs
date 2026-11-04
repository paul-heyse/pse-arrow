// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source syntax contracts. These select existing parsers, never define arithmetic.

/// Native indexing convention for a selected source-expression occurrence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExpressionOrdinal {
    /// Enumerate source values in their local list.
    List,
    /// The occurrence has an explicit zero ordinal.
    Zero,
    /// Enumerate through the owner's flattened containing population.
    Flatten,
}
impl ExpressionOrdinal {
    /// Exact native facet spelling.
    pub const fn as_str(self) -> &'static str { match self { Self::List => "list", Self::Zero => "zero", Self::Flatten => "flatten" } }
    /// Closed source lookup.
    pub fn parse(value: &str) -> Option<Self> { [Self::List, Self::Zero, Self::Flatten].into_iter().find(|kind| kind.as_str() == value) }
}

/// The source ordering used when declaring local index binders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExpressionBinder {
    /// Evaluate an index's domain before admitting its local name.
    DomainBeforeName,
}
impl ExpressionBinder {
    /// Exact native facet spelling.
    pub const fn as_str(self) -> &'static str { match self { Self::DomainBeforeName => "domain-before-name" } }
    /// Closed source lookup.
    pub fn parse(value: &str) -> Option<Self> { (value == Self::DomainBeforeName.as_str()).then_some(Self::DomainBeforeName) }
}

/// Source path association and traversal policy; it does not parse or execute an expression.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExpressionSourceContract<'a> {
    /// Owner-declared path template; placeholders retain their owner's local meaning.
    pub role: Option<&'a str>,
    /// The corresponding occurrence indexing convention.
    pub ordinal: Option<ExpressionOrdinal>,
    /// Binder evaluation ordering when declared independently on a collection.
    pub binder: Option<ExpressionBinder>,
}
impl<'a> ExpressionSourceContract<'a> {
    /// Parse source associations with closed traversal vocabularies.
    /// # Errors
    /// Incomplete role/ordinal pairs, empty paths or unknown traversal values refuse.
    pub fn parse_metadata(role: Option<&'a str>, ordinal: Option<&str>, binder: Option<&str>, path: &str) -> Result<Option<Self>, crate::SchemaError> {
        if role.is_none() && ordinal.is_none() && binder.is_none() { return Ok(None); }
        let invalid = || crate::checks::invalid(path, "incomplete expression source association or unknown traversal policy");
        let (role, ordinal) = match (role, ordinal) {
            (None, None) => (None, None),
            (Some(role), Some(ordinal)) if !role.is_empty() => (Some(role), Some(ExpressionOrdinal::parse(ordinal).ok_or_else(invalid)?)),
            _ => return Err(invalid()),
        };
        let binder = binder.map(|value| ExpressionBinder::parse(value).ok_or_else(invalid)).transpose()?;
        Ok(Some(Self { role, ordinal, binder }))
    }
}
/// Closed source grammars, including the declared annotation-kind selector.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExpressionGrammar {
    /// Arithmetic expressions and references.
    Expression,
    /// Conditional or unconditional equations.
    Equation,
    /// Boolean physical predicates.
    Predicate,
    /// Logical propositions with cardinalities.
    Proposition,
    /// Static values and collections.
    Static,
    /// Arguments whose grammar/count is selected by the annotation declaration.
    AnnotationArguments,
    /// Targets selected as source expressions by the annotation kind; engineering-rule markers are names.
    AnnotationTarget,
}
impl ExpressionGrammar {
    /// Every supported spelling; grammar generation is independent of extension metadata generation.
    pub const ALL: [Self; 7] = [
        Self::Expression,
        Self::Equation,
        Self::Predicate,
        Self::Proposition,
        Self::Static,
        Self::AnnotationArguments,
        Self::AnnotationTarget,
    ];
    /// Exact wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Expression => "expression",
            Self::Equation => "equation",
            Self::Predicate => "predicate",
            Self::Proposition => "proposition",
            Self::Static => "static",
            Self::AnnotationArguments => "annotation_arguments",
            Self::AnnotationTarget => "annotation_target",
        }
    }
    /// Closed spelling lookup; unknown grammars are never interpreted as arithmetic.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|grammar| grammar.as_str() == name)
    }
}
/// The grammar generation understood by the parser dispatch.
pub const GRAMMAR_VERSION: u32 = 1;

/// One exhaustive annotation-kind selection. Stored on the declared annotation field.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnnotationSyntax {
    /// Declared enum spelling.
    pub kind: String,
    /// Typed shape variant projected into the semantic model.
    pub shape: String,
    /// Exact argument cardinality.
    pub count: usize,
    /// Argument grammar, absent when there are no positional arguments.
    pub grammar: Option<String>,
    /// The one required typed payload field, if any.
    pub member: Option<String>,
    /// Whether the annotation target is source syntax.
    pub target: bool,
}
