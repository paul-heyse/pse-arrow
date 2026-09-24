// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Owned source binding values shared by rename and pure expression lowering.
use pse_authoring::dsl;
use pse_ids::SemanticId;
use pse_ids::source_path::ExpressionPathSegmentKind;

/// A declared or lexical path meaning; composite declaration keys stay composite.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathMeaning {
    /// Exact instance-relative source path, resolved independently in every actual context.
    InstancePath(BoundInstancePath),
    /// A bound symbol declaration and its template.
    Symbol {
        /// Owning template.
        template_id: SemanticId,
        /// Symbol declaration.
        symbol_id: SemanticId,
    },
    /// A bound equation declaration.
    Equation {
        /// Owning template.
        template_id: SemanticId,
        /// Equation declaration.
        equation_id: SemanticId,
    },
    /// A port's actual composite primary key.
    Port {
        /// Owning template.
        template_id: SemanticId,
        /// Port key.
        name: String,
    },
    /// An explicit registered entity.
    Entity(SemanticId),
    /// A template-local domain declaration, before instance realization.
    Domain {
        /// Owning template.
        template_id: SemanticId,
        /// Domain key.
        name: String,
    },
    /// A declared template parameter.
    Parameter {
        /// Owning template.
        template_id: SemanticId,
        /// Parameter key.
        name: String,
    },
    /// A declared template feature.
    Feature {
        /// Owning template.
        template_id: SemanticId,
        /// Feature key.
        name: String,
    },
    /// An actual declared unit in a conversion target expression.
    Unit {
        /// Exact admitted unit identity.
        unit_id: SemanticId,
    },
    /// A predicate literal admitted under the actual compared declaration's enum.
    EnumLiteral {
        /// Exact registered enum identity.
        enum_id: SemanticId,
        /// Exact declared member spelling.
        member: String,
    },
    /// A Boolean configuration literal, distinct from a numerical graph constant.
    BooleanLiteral(bool),
    /// A lexically bound reduction/let/index variable.
    Local(String),
}
/// Every parsed path is accounted for, including lexical variables and nested indices.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundPath {
    /// Byte range relative to the decoded DSL string.
    pub span: dsl::Span,
    /// Resolved declared or lexical meaning.
    pub meaning: PathMeaning,
    /// Identity of each named path segment where one exists; used by rename.
    pub segment_entities: Vec<Option<SemanticId>>,
}
/// The grammar admitted for one source field.
#[derive(Clone, Debug, PartialEq)]
pub enum ParsedExpression {
    /// Arithmetic expression.
    Expr(dsl::Expr),
    /// Equation, including conditional equations.
    Equation(dsl::Equation),
    /// Predicate.
    Predicate(dsl::Predicate),
}
/// One exact declared step with its original index-expression partition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstancePathSegment {
    /// Child, parameter-selected root, or final member.
    pub kind: ExpressionPathSegmentKind,
    /// Exact declared key within the selected owning template.
    pub name: String,
    /// Number of authored index expressions attached to this step.
    pub index_count: u16,
}
/// A path remains relative until its actual occurrence and selected templates are known.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundInstancePath {
    /// A globally referenced actual instance, otherwise the source occurrence's owner.
    pub root_instance_id: Option<SemanticId>,
    /// Exact ordered steps; no display-path parsing is needed by binding.
    pub segments: Vec<InstancePathSegment>,
    /// Actual known leaf declarations across selected contexts, used to prevent partial rename.
    pub member_entities: Vec<SemanticId>,
    /// Exact enum shared by every resolved leaf declaration, when enum-valued.
    pub member_enum: Option<SemanticId>,
    /// The actual selected owner is required before admitting the leaf's type.
    pub deferred: bool,
}
