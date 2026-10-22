// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source syntax contracts. These select existing parsers, never define arithmetic.
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
