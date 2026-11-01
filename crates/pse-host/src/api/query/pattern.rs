// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One typed schema61 pattern program for Rust syntax and structured query input.
use super::{ApiError, contracts as c};
use quote::ToTokens as _;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Original structured query parse failure, retaining its source coordinate.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("structural API query: {message}")]
#[diagnostic(code(pse::host::ownership))]
pub struct QueryError {
    /// Original parser diagnostic.
    pub message: String,
    /// Original query source.
    #[source_code]
    pub query_text: String,
    /// Source byte location of the parser diagnostic.
    #[label("invalid query syntax")]
    pub span: miette::SourceSpan,
    /// One-based parser line.
    pub line: usize,
    /// Zero-based parser column.
    pub column: usize,
}
impl pse_diagnostics::TypedDiagnostic for QueryError {
    fn diagnostic_code(&self) -> Option<pse_diagnostics::DiagnosticCode> {
        Some(pse_diagnostics::DiagnosticCode::HostOwnership)
    }
}
fn parse_error(source: &str, error: syn::Error) -> ApiError {
    let start = error.span().start();
    let offset = source
        .split_inclusive('\n')
        .take(start.line.saturating_sub(1))
        .map(str::len)
        .sum::<usize>()
        + start.column;
    QueryError {
        message: error.to_string(),
        query_text: source.into(),
        span: (offset.min(source.len()), 0).into(),
        line: start.line,
        column: start.column,
    }
    .into()
}

/// A declared structural shape, not semantic equality or a trait obligation.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TypePattern {
    /// Schema61 structure. Infer denotes an independent wildcard at each occurrence.
    pub shape: c::Type,
}
/// One declared generic predicate; this does not solve trait obligations.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WherePattern {
    /// Structured schema61 predicate.
    pub predicate: c::Predicate,
}
impl WherePattern {
    /// Reject unsupported structured predicates before executing providers.
    pub fn validate(&self) -> Result<(), ApiError> {
        validate_predicate(&self.predicate)
    }
    /// Parse a single Rust where-clause predicate.
    pub fn parse(text: &str) -> Result<Self, ApiError> {
        let predicate: syn::WherePredicate =
            syn::parse_str(text).map_err(|e| parse_error(text, e))?;
        let pattern = Self {
            predicate: match predicate {
                syn::WherePredicate::Type(v) => c::Predicate::Bound {
                    type_: ty_syn(&v.bounded_ty)?,
                    bounds: v.bounds.iter().map(bound).collect::<Result<_, _>>()?,
                    generic_params: params(v.lifetimes.as_ref())?,
                },
                syn::WherePredicate::Lifetime(v) => c::Predicate::Lifetime {
                    lifetime: v.lifetime.to_string(),
                    outlives: v.bounds.iter().map(ToString::to_string).collect(),
                },
                _ => return Err(invalid("unsupported where predicate syntax")),
            },
        };
        pattern.validate()?;
        Ok(pattern)
    }
    pub(super) fn evaluate(&self, item: &rustdoc_types::Item) -> ShapeMatch {
        let Some(generics) = c::generics(item) else {
            return ShapeMatch::Different;
        };
        let Ok(mut actual) = c::project::<_, c::Generics>(generics) else {
            return ShapeMatch::Unknown("declared generics projection unavailable".into());
        };
        for parameter in actual.params {
            match parameter.kind {
                c::ParameterKind::Type { bounds, .. } => {
                    actual.where_predicates.push(c::Predicate::Bound {
                        type_: c::Type::Generic(parameter.name),
                        bounds,
                        generic_params: Vec::new(),
                    })
                }
                c::ParameterKind::Lifetime { outlives } => {
                    actual.where_predicates.push(c::Predicate::Lifetime {
                        lifetime: parameter.name,
                        outlives,
                    })
                }
                _ => {}
            }
        }
        any(actual
            .where_predicates
            .iter()
            .map(|p| predicate(&self.predicate, p, &mut Lifetimes::default())))
    }
}
fn predicate(a: &c::Predicate, b: &c::Predicate, env: &mut Lifetimes) -> ShapeMatch {
    match (a, b) {
        (
            c::Predicate::Bound {
                type_: a,
                bounds: ab,
                generic_params: ap,
            },
            c::Predicate::Bound {
                type_: b,
                bounds: bb,
                generic_params: bp,
            },
        ) => env.scoped(ap, bp, |env| all([compare(a, b, env), bounds(ab, bb, env)])),
        (
            c::Predicate::Lifetime {
                lifetime: a,
                outlives: ao,
            },
            c::Predicate::Lifetime {
                lifetime: b,
                outlives: bo,
            },
        ) => yes(env.equal(a, b)
            && ao.len() == bo.len()
            && ao.iter().all(|a| bo.iter().any(|b| env.equal(a, b)))),
        (
            c::Predicate::Equality {
                lhs: a,
                rhs: c::Term::Type(ar),
            },
            c::Predicate::Equality {
                lhs: b,
                rhs: c::Term::Type(br),
            },
        ) => all([compare(a, b, env), compare(ar, br, env)]),
        (
            c::Predicate::Equality {
                lhs: a,
                rhs: c::Term::Constant(ar),
            },
            c::Predicate::Equality {
                lhs: b,
                rhs: c::Term::Constant(br),
            },
        ) => all([compare(a, b, env), const_equal(ar, br)]),
        _ => ShapeMatch::Different,
    }
}
/// Shape evaluation preserves unavailable producer structure as unknown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShapeMatch {
    /// Every requested structural constraint was observed.
    Matches,
    /// A known constraint differs.
    Different,
    /// Producer structure is not available for this comparison.
    Unknown(String),
}
impl TypePattern {
    /// Reject unsupported structured query nodes before providers start.
    pub fn validate(&self) -> Result<(), ApiError> {
        validate_type(&self.shape)
    }
    /// Parse through the pinned syn syntax owner once, without spelling fallbacks.
    pub fn parse(text: &str) -> Result<Self, ApiError> {
        let ty: syn::Type = syn::parse_str(text).map_err(|e| parse_error(text, e))?;
        let pattern = Self {
            shape: ty_syn(&ty)?,
        };
        pattern.validate()?;
        Ok(pattern)
    }
    /// Evaluate the actual typed producer tree through the existing contract projection.
    pub fn evaluate(&self, ty: &rustdoc_types::Type) -> ShapeMatch {
        match c::project(ty) {
            Ok(actual) => compare(&self.shape, &actual, &mut Lifetimes::default()),
            Err(e) => ShapeMatch::Unknown(e.to_string()),
        }
    }
    /// Positive-witness convenience; public queries also retain unknown outcomes.
    pub fn matches(&self, ty: &rustdoc_types::Type) -> bool {
        self.evaluate(ty) == ShapeMatch::Matches
    }
    /// Rustdoc omits implicit unit returns.
    pub fn matches_return(&self, ty: Option<&rustdoc_types::Type>) -> bool {
        self.evaluate_return(ty) == ShapeMatch::Matches
    }
    /// Evaluate an omitted unit return without inventing a producer node.
    pub fn evaluate_return(&self, ty: Option<&rustdoc_types::Type>) -> ShapeMatch {
        ty.map_or_else(
            || {
                yes(matches!(&self.shape, c::Type::Infer)
                    || matches!(&self.shape,c::Type::Tuple(v)if v.is_empty()))
            },
            |ty| self.evaluate(ty),
        )
    }
}
fn invalid(message: impl Into<String>) -> ApiError {
    super::super::invalid(message)
}
fn validate_params(values: &[c::Parameter]) -> Result<(), ApiError> {
    let mut names = std::collections::BTreeSet::new();
    for value in values {
        if !names.insert(&value.name) {
            return Err(invalid("duplicate higher-ranked lifetime binder"));
        }
        match &value.kind {
            c::ParameterKind::Lifetime { outlives } => {
                validate_lifetime(&value.name)?;
                for value in outlives {
                    validate_lifetime(value)?;
                }
            }
            _ => return Err(invalid("higher-ranked binders require lifetime parameters")),
        }
    }
    Ok(())
}
fn validate_lifetime(value: &str) -> Result<(), ApiError> {
    syn::parse_str::<syn::Lifetime>(value)
        .map(|_| ())
        .map_err(|e| parse_error(value, e))
}
fn validate_bounds(values: &[c::Bound]) -> Result<(), ApiError> {
    for value in values {
        match value {
            c::Bound::Trait {
                trait_,
                generic_params,
                ..
            } => {
                validate_path(trait_)?;
                validate_params(generic_params)?;
            }
            c::Bound::Outlives(value) => validate_lifetime(value)?,
            c::Bound::Use(_) => {
                return Err(invalid("precise-capture bound patterns are unsupported"));
            }
        }
    }
    Ok(())
}
fn validate_constant(value: &c::Constant) -> Result<(), ApiError> {
    if value.expr.trim().is_empty() {
        Err(invalid(
            "constant pattern requires original producer expression text",
        ))
    } else {
        Ok(())
    }
}
fn validate_path(value: &c::Path) -> Result<(), ApiError> {
    if value.path.is_empty() {
        return Err(invalid("path pattern must not be empty"));
    }
    validate_args(value.args.as_deref())
}
fn validate_args(value: Option<&c::Arguments>) -> Result<(), ApiError> {
    match value {
        None => {}
        Some(c::Arguments::ReturnTypeNotation) => {
            return Err(invalid("return-type notation patterns are unsupported"));
        }
        Some(c::Arguments::Parenthesized { inputs, output }) => {
            for value in inputs {
                validate_type(value)?;
            }
            if let Some(value) = output {
                validate_type(value)?;
            }
        }
        Some(c::Arguments::AngleBracketed { args, constraints }) => {
            for value in args {
                match value {
                    c::Argument::Type(value) => validate_type(value)?,
                    c::Argument::Lifetime(value) => validate_lifetime(value)?,
                    c::Argument::Const(value) => validate_constant(value)?,
                    c::Argument::Infer => {}
                }
            }
            for value in constraints {
                validate_args(value.args.as_deref())?;
                match &value.binding {
                    c::Binding::Equality(c::Term::Type(value)) => validate_type(value)?,
                    c::Binding::Equality(c::Term::Constant(value)) => validate_constant(value)?,
                    c::Binding::Constraint(value) => validate_bounds(value)?,
                }
            }
        }
    }
    Ok(())
}
fn validate_type(value: &c::Type) -> Result<(), ApiError> {
    match value {
        c::Type::ResolvedPath(value) => validate_path(value)?,
        c::Type::DynTrait(value) => {
            for value in &value.traits {
                validate_path(&value.trait_)?;
                validate_params(&value.generic_params)?;
            }
            if let Some(value) = &value.lifetime {
                validate_lifetime(value)?;
            }
        }
        c::Type::FunctionPointer(value) => {
            validate_params(&value.generic_params)?;
            if value.header.is_const || value.header.is_async {
                return Err(invalid(
                    "const/async function-pointer patterns are unsupported",
                ));
            }
            for (_, value) in &value.sig.inputs {
                validate_type(value)?;
            }
            if let Some(value) = &value.sig.output {
                validate_type(value)?;
            }
        }
        c::Type::Tuple(values) => {
            for value in values {
                validate_type(value)?;
            }
        }
        c::Type::Slice(value)
        | c::Type::Array { type_: value, .. }
        | c::Type::RawPointer { type_: value, .. } => validate_type(value)?,
        c::Type::BorrowedRef {
            lifetime, type_, ..
        } => {
            if let Some(value) = lifetime {
                validate_lifetime(value)?;
            }
            validate_type(type_)?;
        }
        c::Type::QualifiedPath {
            args,
            self_type,
            trait_,
            ..
        } => {
            validate_args(args.as_deref())?;
            validate_type(self_type)?;
            if let Some(value) = trait_ {
                validate_path(value)?;
            }
        }
        c::Type::ImplTrait(values) => validate_bounds(values)?,
        c::Type::Pat { .. } => return Err(invalid("type-pattern constraints are unsupported")),
        c::Type::Generic(_) | c::Type::Primitive(_) | c::Type::Infer => {}
    }
    Ok(())
}
fn validate_predicate(value: &c::Predicate) -> Result<(), ApiError> {
    match value {
        c::Predicate::Bound {
            type_,
            bounds,
            generic_params,
        } => {
            validate_type(type_)?;
            validate_bounds(bounds)?;
            validate_params(generic_params)?;
        }
        c::Predicate::Lifetime { lifetime, outlives } => {
            validate_lifetime(lifetime)?;
            for value in outlives {
                validate_lifetime(value)?;
            }
        }
        c::Predicate::Equality { lhs, rhs } => {
            validate_type(lhs)?;
            match rhs {
                c::Term::Type(value) => validate_type(value)?,
                c::Term::Constant(value) => validate_constant(value)?,
            }
        }
    }
    Ok(())
}
fn params(value: Option<&syn::BoundLifetimes>) -> Result<Vec<c::Parameter>, ApiError> {
    value
        .map(|v| {
            v.lifetimes
                .iter()
                .map(|p| match p {
                    syn::GenericParam::Lifetime(v) => Ok(c::Parameter {
                        name: v.lifetime.to_string(),
                        kind: c::ParameterKind::Lifetime {
                            outlives: v.bounds.iter().map(ToString::to_string).collect(),
                        },
                    }),
                    _ => Err(invalid("higher-ranked binders require lifetime parameters")),
                })
                .collect()
        })
        .unwrap_or_else(|| Ok(Vec::new()))
}
fn constant(expr: &syn::Expr) -> c::Constant {
    let expr = match expr {
        syn::Expr::Block(block) if block.block.stmts.len() == 1 => match &block.block.stmts[0] {
            syn::Stmt::Expr(expr, None) => expr,
            _ => expr,
        },
        _ => expr,
    };
    c::Constant {
        expr: expr.to_token_stream().to_string(),
        value: None,
        is_literal: matches!(expr, syn::Expr::Lit(_)),
    }
}
fn output(value: &syn::ReturnType) -> Result<Option<c::Type>, ApiError> {
    match value {
        syn::ReturnType::Default => Ok(None),
        syn::ReturnType::Type(_, ty) => Ok(Some(ty_syn(ty)?)),
    }
}
fn abi(value: Option<&syn::Abi>) -> c::Abi {
    let Some(value) = value else {
        return c::Abi::Rust;
    };
    let name = value
        .name
        .as_ref()
        .map_or_else(|| "C".to_owned(), syn::LitStr::value);
    let (base, unwind) = name
        .strip_suffix("-unwind")
        .map_or((name.as_str(), false), |name| (name, true));
    match base {
        "Rust" if !unwind => c::Abi::Rust,
        "C" => c::Abi::C { unwind },
        "cdecl" => c::Abi::Cdecl { unwind },
        "stdcall" => c::Abi::Stdcall { unwind },
        "fastcall" => c::Abi::Fastcall { unwind },
        "aapcs" => c::Abi::Aapcs { unwind },
        "win64" => c::Abi::Win64 { unwind },
        "sysv64" => c::Abi::SysV64 { unwind },
        "system" => c::Abi::System { unwind },
        _ => c::Abi::Other(name),
    }
}
fn arguments(value: &syn::PathArguments) -> Result<Option<Box<c::Arguments>>, ApiError> {
    Ok(match value {
        syn::PathArguments::None => None,
        syn::PathArguments::Parenthesized(v) => Some(Box::new(c::Arguments::Parenthesized {
            inputs: v
                .inputs
                .iter()
                .map(|v| ty_syn(&v.ty))
                .collect::<Result<_, _>>()?,
            output: output(&v.output)?,
        })),
        syn::PathArguments::AngleBracketed(v) => Some(Box::new(angle(v)?)),
    })
}
fn angle(value: &syn::AngleBracketedGenericArguments) -> Result<c::Arguments, ApiError> {
    let mut args = Vec::new();
    let mut constraints = Vec::new();
    for argument in &value.args {
        match argument {
            syn::GenericArgument::Lifetime(v) => args.push(c::Argument::Lifetime(v.to_string())),
            syn::GenericArgument::Type(v) => args.push(c::Argument::Type(ty_syn(v)?)),
            syn::GenericArgument::Const(v) => args.push(c::Argument::Const(constant(v))),
            syn::GenericArgument::AssocType(v) => constraints.push(c::Constraint {
                name: v.ident.to_string(),
                args: v.generics.as_ref().map(angle).transpose()?.map(Box::new),
                binding: c::Binding::Equality(c::Term::Type(ty_syn(&v.ty)?)),
            }),
            syn::GenericArgument::AssocConst(v) => constraints.push(c::Constraint {
                name: v.ident.to_string(),
                args: v.generics.as_ref().map(angle).transpose()?.map(Box::new),
                binding: c::Binding::Equality(c::Term::Constant(constant(&v.value))),
            }),
            syn::GenericArgument::Constraint(v) => constraints.push(c::Constraint {
                name: v.ident.to_string(),
                args: v.generics.as_ref().map(angle).transpose()?.map(Box::new),
                binding: c::Binding::Constraint(
                    v.bounds.iter().map(bound).collect::<Result<_, _>>()?,
                ),
            }),
            _ => {
                return Err(invalid(
                    "unsupported generic argument; no argument kinds were discarded",
                ));
            }
        }
    }
    Ok(c::Arguments::AngleBracketed { args, constraints })
}
fn path(value: &syn::Path) -> Result<c::Path, ApiError> {
    if value
        .segments
        .iter()
        .take(value.segments.len().saturating_sub(1))
        .any(|s| !matches!(s.arguments, syn::PathArguments::None))
    {
        return Err(invalid(
            "generic arguments on intermediate path components are unsupported",
        ));
    }
    Ok(c::Path {
        path: value
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
        id: 0,
        args: value
            .segments
            .last()
            .map(|s| arguments(&s.arguments))
            .transpose()?
            .flatten(),
    })
}
fn bound(value: &syn::TypeParamBound) -> Result<c::Bound, ApiError> {
    match value {
        syn::TypeParamBound::Lifetime(v) => Ok(c::Bound::Outlives(v.to_string())),
        syn::TypeParamBound::Trait(v) => {
            v.modifiers
                .require_empty()
                .map_err(|e| invalid(e.to_string()))?;
            Ok(c::Bound::Trait {
                trait_: path(&v.path)?,
                generic_params: params(v.lifetimes.as_ref())?,
                modifier: if v.maybe.is_some() {
                    c::Modifier::Maybe
                } else {
                    c::Modifier::None
                },
            })
        }
        _ => Err(invalid(
            "unsupported bound syntax; no trait solving was attempted",
        )),
    }
}
fn ty_syn(value: &syn::Type) -> Result<c::Type, ApiError> {
    Ok(match value {
        syn::Type::Infer(_) => c::Type::Infer,
        syn::Type::Never(_) => c::Type::Primitive("!".into()),
        syn::Type::Path(v) if v.qself.is_none() => c::Type::ResolvedPath(path(&v.path)?),
        syn::Type::Path(v) => {
            let q = v
                .qself
                .as_ref()
                .ok_or_else(|| invalid("missing qualified self"))?;
            if v.path.segments.len() != q.position + 1 {
                return Err(invalid("nested qualified associated paths are unsupported"));
            }
            let associated = v
                .path
                .segments
                .last()
                .ok_or_else(|| invalid("missing associated item"))?;
            let trait_ = if q.position == 0 {
                None
            } else {
                let mut p = v.path.clone();
                p.segments = p.segments.into_iter().take(q.position).collect();
                Some(path(&p)?)
            };
            c::Type::QualifiedPath {
                name: associated.ident.to_string(),
                args: arguments(&associated.arguments)?,
                self_type: Box::new(ty_syn(&q.ty)?),
                trait_,
            }
        }
        syn::Type::Reference(v) => c::Type::BorrowedRef {
            lifetime: v.lifetime.as_ref().map(ToString::to_string),
            is_mutable: v.mutability.is_some(),
            type_: Box::new(ty_syn(&v.elem)?),
        },
        syn::Type::Ptr(v) => c::Type::RawPointer {
            is_mutable: matches!(v.mutability, syn::PointerMutability::Mut(_)),
            type_: Box::new(ty_syn(&v.elem)?),
        },
        syn::Type::Tuple(v) => {
            c::Type::Tuple(v.elems.iter().map(ty_syn).collect::<Result<_, _>>()?)
        }
        syn::Type::Slice(v) => c::Type::Slice(Box::new(ty_syn(&v.elem)?)),
        syn::Type::Array(v) => c::Type::Array {
            type_: Box::new(ty_syn(&v.elem)?),
            len: constant(&v.len).expr,
        },
        syn::Type::FnPtr(v) => c::Type::FunctionPointer(Box::new(c::FunctionPointer {
            sig: c::Signature {
                inputs: v
                    .inputs
                    .iter()
                    .map(|a| Ok((String::new(), ty_syn(&a.ty)?)))
                    .collect::<Result<_, ApiError>>()?,
                output: output(&v.output)?,
                is_c_variadic: v.variadic.is_some(),
            },
            generic_params: params(v.lifetimes.as_ref())?,
            header: c::Header {
                is_const: false,
                is_async: false,
                is_unsafe: v.unsafety.is_some(),
                abi: abi(v.abi.as_ref()),
            },
        })),
        syn::Type::ImplTrait(v) => {
            c::Type::ImplTrait(v.bounds.iter().map(bound).collect::<Result<_, _>>()?)
        }
        syn::Type::TraitObject(v) => {
            let mut traits = Vec::new();
            let mut lifetime = None;
            for b in &v.bounds {
                match bound(b)? {
                    c::Bound::Trait {
                        trait_,
                        generic_params,
                        modifier: c::Modifier::None,
                    } => traits.push(c::PolyTrait {
                        trait_,
                        generic_params,
                    }),
                    c::Bound::Outlives(v) => {
                        if lifetime.replace(v).is_some() {
                            return Err(invalid(
                                "multiple trait-object lifetime bounds are unsupported",
                            ));
                        }
                    }
                    _ => return Err(invalid("unsupported trait-object bound")),
                }
            }
            c::Type::DynTrait(c::DynTrait { traits, lifetime })
        }
        syn::Type::Paren(v) => return ty_syn(&v.elem),
        syn::Type::Group(v) => return ty_syn(&v.elem),
        _ => {
            return Err(invalid(
                "unsupported structural type query; no spelling fallback was used",
            ));
        }
    })
}

#[derive(Default)]
struct Lifetimes {
    scopes: Vec<(Vec<String>, Vec<String>)>,
}
impl Lifetimes {
    fn equal(&self, a: &str, b: &str) -> bool {
        let resolve = |value: &str, expected: bool| {
            self.scopes
                .iter()
                .rev()
                .enumerate()
                .find_map(|(depth, (a, b))| {
                    let v = if expected { a } else { b };
                    v.iter()
                        .position(|name| name == value)
                        .map(|index| (depth, index))
                })
        };
        match (resolve(a, true), resolve(b, false)) {
            (Some(a), Some(b)) => a == b,
            (None, None) => a == b,
            _ => false,
        }
    }
    fn scoped(
        &mut self,
        a: &[c::Parameter],
        b: &[c::Parameter],
        f: impl FnOnce(&mut Self) -> ShapeMatch,
    ) -> ShapeMatch {
        if a.len() != b.len() {
            return ShapeMatch::Different;
        }
        if a.iter()
            .chain(b)
            .any(|p| !matches!(p.kind, c::ParameterKind::Lifetime { .. }))
        {
            return ShapeMatch::Unknown("non-lifetime higher-ranked binder".into());
        }
        self.scopes.push((
            a.iter().map(|p| p.name.clone()).collect(),
            b.iter().map(|p| p.name.clone()).collect(),
        ));
        let constraints = yes(a.iter().zip(b).all(|(a, b)| match (&a.kind, &b.kind) {
            (
                c::ParameterKind::Lifetime { outlives: a },
                c::ParameterKind::Lifetime { outlives: b },
            ) => a.len() == b.len() && a.iter().all(|a| b.iter().any(|b| self.equal(a, b))),
            _ => false,
        }));
        let result = all([constraints, f(self)]);
        self.scopes.pop();
        result
    }
}
fn yes(value: bool) -> ShapeMatch {
    if value {
        ShapeMatch::Matches
    } else {
        ShapeMatch::Different
    }
}
pub(super) fn all(values: impl IntoIterator<Item = ShapeMatch>) -> ShapeMatch {
    let mut unknown = None;
    for v in values {
        match v {
            ShapeMatch::Different => return ShapeMatch::Different,
            ShapeMatch::Unknown(e) => unknown = Some(e),
            ShapeMatch::Matches => {}
        }
    }
    unknown.map_or(ShapeMatch::Matches, ShapeMatch::Unknown)
}
fn same_path(a: &str, b: &str) -> bool {
    a == b || b.ends_with(&format!("::{a}"))
}
fn const_equal(a: &c::Constant, b: &c::Constant) -> ShapeMatch {
    // Rustdoc expressions are producer text, not an invitation to evaluate Rust expressions.
    if a.expr == b.expr {
        ShapeMatch::Matches
    } else if a.is_literal && b.is_literal {
        ShapeMatch::Different
    } else {
        ShapeMatch::Unknown("opaque producer constant expressions differ; no evaluation or source equivalence was inferred".into())
    }
}
fn array_length(a: &str, b: &str) -> ShapeMatch {
    if a == b {
        ShapeMatch::Matches
    } else if syn::parse_str::<syn::LitInt>(a).is_ok() && syn::parse_str::<syn::LitInt>(b).is_ok() {
        ShapeMatch::Different
    } else {
        ShapeMatch::Unknown(
            "opaque array length expressions differ; no evaluation was attempted".into(),
        )
    }
}
fn args(a: Option<&c::Arguments>, b: Option<&c::Arguments>, env: &mut Lifetimes) -> ShapeMatch {
    let Some(a) = a else {
        return ShapeMatch::Matches;
    };
    let Some(b) = b else {
        return ShapeMatch::Different;
    };
    match (a, b) {
        (
            c::Arguments::AngleBracketed {
                args: a,
                constraints: ac,
            },
            c::Arguments::AngleBracketed {
                args: b,
                constraints: bc,
            },
        ) => {
            if a.len() != b.len() || ac.len() != bc.len() {
                return ShapeMatch::Different;
            }
            let mut outcomes = Vec::new();
            for (a, b) in a.iter().zip(b) {
                outcomes.push(match (a, b) {
                    (c::Argument::Type(a), c::Argument::Type(b)) => compare(a, b, env),
                    (c::Argument::Infer, _) => ShapeMatch::Matches,
                    (c::Argument::Lifetime(a), c::Argument::Lifetime(b)) => yes(env.equal(a, b)),
                    (c::Argument::Const(a), c::Argument::Const(b)) => const_equal(a, b),
                    _ => ShapeMatch::Different,
                });
            }
            outcomes.push(multiset(ac, bc, |a, b| {
                all([
                    yes(a.name == b.name),
                    args(a.args.as_deref(), b.args.as_deref(), env),
                    match (&a.binding, &b.binding) {
                        (
                            c::Binding::Equality(c::Term::Type(a)),
                            c::Binding::Equality(c::Term::Type(b)),
                        ) => compare(a, b, env),
                        (
                            c::Binding::Equality(c::Term::Constant(a)),
                            c::Binding::Equality(c::Term::Constant(b)),
                        ) => const_equal(a, b),
                        (c::Binding::Constraint(a), c::Binding::Constraint(b)) => bounds(a, b, env),
                        _ => ShapeMatch::Different,
                    },
                ])
            }));
            all(outcomes)
        }
        (
            c::Arguments::Parenthesized {
                inputs: a,
                output: ao,
            },
            c::Arguments::Parenthesized {
                inputs: b,
                output: bo,
            },
        ) => all([types(a, b, env), returns(ao.as_ref(), bo.as_ref(), env)]),
        (c::Arguments::ReturnTypeNotation, _) => {
            ShapeMatch::Unknown("return-type notation patterns are unsupported".into())
        }
        _ => ShapeMatch::Different,
    }
}
pub(super) fn any(values: impl IntoIterator<Item = ShapeMatch>) -> ShapeMatch {
    let mut unknown = None;
    for v in values {
        match v {
            ShapeMatch::Matches => return ShapeMatch::Matches,
            ShapeMatch::Unknown(e) => unknown = Some(e),
            ShapeMatch::Different => {}
        }
    }
    unknown.map_or(ShapeMatch::Different, ShapeMatch::Unknown)
}
fn paths(a: &c::Path, b: &c::Path, env: &mut Lifetimes) -> ShapeMatch {
    all([
        yes(same_path(&a.path, &b.path)),
        args(a.args.as_deref(), b.args.as_deref(), env),
    ])
}
fn types(a: &[c::Type], b: &[c::Type], env: &mut Lifetimes) -> ShapeMatch {
    if a.len() != b.len() {
        return ShapeMatch::Different;
    }
    all(a.iter().zip(b).map(|(a, b)| compare(a, b, env)))
}
fn returns(a: Option<&c::Type>, b: Option<&c::Type>, env: &mut Lifetimes) -> ShapeMatch {
    match (a, b) {
        (None, None) => ShapeMatch::Matches,
        (Some(c::Type::Tuple(v)), None) | (None, Some(c::Type::Tuple(v))) if v.is_empty() => {
            ShapeMatch::Matches
        }
        (Some(a), Some(b)) => compare(a, b, env),
        (Some(c::Type::Infer), _) => ShapeMatch::Matches,
        _ => ShapeMatch::Different,
    }
}
fn modifier(a: &c::Modifier, b: &c::Modifier) -> bool {
    matches!(
        (a, b),
        (c::Modifier::None, c::Modifier::None)
            | (c::Modifier::Maybe, c::Modifier::Maybe)
            | (c::Modifier::MaybeConst, c::Modifier::MaybeConst)
    )
}
fn bounds(a: &[c::Bound], b: &[c::Bound], env: &mut Lifetimes) -> ShapeMatch {
    if a.len() != b.len() {
        return ShapeMatch::Different;
    }
    multiset(a, b, |a, b| match (a, b) {
        (c::Bound::Outlives(a), c::Bound::Outlives(b)) => yes(env.equal(a, b)),
        (
            c::Bound::Trait {
                trait_: a,
                generic_params: ap,
                modifier: am,
            },
            c::Bound::Trait {
                trait_: b,
                generic_params: bp,
                modifier: bm,
            },
        ) => {
            if modifier(am, bm) {
                env.scoped(ap, bp, |env| paths(a, b, env))
            } else {
                ShapeMatch::Different
            }
        }
        (c::Bound::Use(_), _) | (_, c::Bound::Use(_)) => {
            ShapeMatch::Unknown("unsupported precise-capturing bound comparison".into())
        }
        _ => ShapeMatch::Different,
    })
}
fn multiset<T>(a: &[T], b: &[T], mut compare: impl FnMut(&T, &T) -> ShapeMatch) -> ShapeMatch {
    if a.len() != b.len() {
        return ShapeMatch::Different;
    }
    let matrix = a
        .iter()
        .map(|a| b.iter().map(|b| compare(a, b)).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    fn augment(
        row: usize,
        matrix: &[Vec<ShapeMatch>],
        allow_unknown: bool,
        seen: &mut [bool],
        assigned: &mut [Option<usize>],
    ) -> bool {
        for column in 0..assigned.len() {
            if seen[column]
                || matches!(matrix[row][column], ShapeMatch::Different)
                || !allow_unknown && matches!(matrix[row][column], ShapeMatch::Unknown(_))
            {
                continue;
            }
            seen[column] = true;
            if assigned[column]
                .is_none_or(|previous| augment(previous, matrix, allow_unknown, seen, assigned))
            {
                assigned[column] = Some(row);
                return true;
            }
        }
        false
    }
    let perfect = |allow_unknown| {
        let mut assigned = vec![None; b.len()];
        (0..a.len()).all(|row| {
            augment(
                row,
                &matrix,
                allow_unknown,
                &mut vec![false; b.len()],
                &mut assigned,
            )
        })
    };
    if perfect(false) {
        ShapeMatch::Matches
    } else if perfect(true) {
        ShapeMatch::Unknown(
            matrix
                .into_iter()
                .flatten()
                .find_map(|v| {
                    if let ShapeMatch::Unknown(e) = v {
                        Some(e)
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "incomplete unordered comparison".into()),
        )
    } else {
        ShapeMatch::Different
    }
}
fn compare(a: &c::Type, b: &c::Type, env: &mut Lifetimes) -> ShapeMatch {
    match (a, b) {
        (c::Type::Infer, _) => ShapeMatch::Matches,
        (_, c::Type::Infer | c::Type::Pat { .. }) => ShapeMatch::Unknown(
            "producer type is inferred or carries unsupported pattern constraints".into(),
        ),
        (c::Type::ResolvedPath(a), c::Type::ResolvedPath(b)) => paths(a, b, env),
        (c::Type::ResolvedPath(a), c::Type::Primitive(b) | c::Type::Generic(b))
            if a.args.is_none() =>
        {
            yes(a.path == *b)
        }
        (c::Type::Primitive(a), c::Type::Primitive(b))
        | (c::Type::Generic(a), c::Type::Generic(b)) => yes(a == b),
        (
            c::Type::BorrowedRef {
                lifetime: a,
                is_mutable: am,
                type_: at,
            },
            c::Type::BorrowedRef {
                lifetime: b,
                is_mutable: bm,
                type_: bt,
            },
        ) => all([
            yes(am == bm
                && a.as_ref()
                    .is_none_or(|a| b.as_ref().is_some_and(|b| env.equal(a, b)))),
            compare(at, bt, env),
        ]),
        (
            c::Type::RawPointer {
                is_mutable: a,
                type_: at,
            },
            c::Type::RawPointer {
                is_mutable: b,
                type_: bt,
            },
        ) => all([yes(a == b), compare(at, bt, env)]),
        (c::Type::Tuple(a), c::Type::Tuple(b)) => types(a, b, env),
        (c::Type::Slice(a), c::Type::Slice(b)) => compare(a, b, env),
        (c::Type::Array { type_: a, len: al }, c::Type::Array { type_: b, len: bl }) => {
            all([array_length(al, bl), compare(a, b, env)])
        }
        (
            c::Type::QualifiedPath {
                name: a,
                args: aa,
                self_type: at,
                trait_: atr,
            },
            c::Type::QualifiedPath {
                name: b,
                args: ba,
                self_type: bt,
                trait_: btr,
            },
        ) => {
            let tr = match (atr, btr) {
                (Some(a), Some(b)) => paths(a, b, env),
                (None, None) => ShapeMatch::Matches,
                _ => ShapeMatch::Different,
            };
            all([
                yes(a == b),
                args(aa.as_deref(), ba.as_deref(), env),
                compare(at, bt, env),
                tr,
            ])
        }
        (c::Type::FunctionPointer(a), c::Type::FunctionPointer(b)) => {
            env.scoped(&a.generic_params, &b.generic_params, |env| {
                let inputs_a = a
                    .sig
                    .inputs
                    .iter()
                    .map(|(_, t)| t.clone())
                    .collect::<Vec<_>>();
                let inputs_b = b
                    .sig
                    .inputs
                    .iter()
                    .map(|(_, t)| t.clone())
                    .collect::<Vec<_>>();
                all([
                    yes(a.header.abi == b.header.abi
                        && a.header.is_unsafe == b.header.is_unsafe
                        && a.sig.is_c_variadic == b.sig.is_c_variadic),
                    types(&inputs_a, &inputs_b, env),
                    returns(a.sig.output.as_ref(), b.sig.output.as_ref(), env),
                ])
            })
        }
        (c::Type::ImplTrait(a), c::Type::ImplTrait(b)) => bounds(a, b, env),
        (c::Type::DynTrait(a), c::Type::DynTrait(b)) => {
            if a.traits.len() != b.traits.len() {
                return ShapeMatch::Different;
            }
            let mut values = Vec::new();
            values.push(multiset(&a.traits, &b.traits, |a, b| {
                env.scoped(&a.generic_params, &b.generic_params, |env| {
                    paths(&a.trait_, &b.trait_, env)
                })
            }));
            values.push(yes(a.lifetime.as_ref().is_none_or(|a| {
                b.lifetime.as_ref().is_some_and(|b| env.equal(a, b))
            })));
            all(values)
        }
        _ => ShapeMatch::Different,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn actual(value: serde_json::Value) -> rustdoc_types::Type {
        serde_json::from_value(value).unwrap()
    }
    fn primitive() -> serde_json::Value {
        json!({"primitive":"u8"})
    }
    fn reference(lifetime: &str) -> serde_json::Value {
        json!({"borrowed_ref":{"lifetime":lifetime,"is_mutable":false,"type":primitive()}})
    }
    fn callback(
        names: &[&str],
        inputs: Vec<serde_json::Value>,
        output: Option<serde_json::Value>,
    ) -> serde_json::Value {
        json!({"function_pointer":{"sig":{"inputs":inputs.into_iter().map(|t|json!(["value",t])).collect::<Vec<_>>(),"output":output,"is_c_variadic":false},"generic_params":names.iter().map(|name|json!({"name":name,"kind":{"lifetime":{"outlives":[]}}})).collect::<Vec<_>>(),"header":{"abi":"Rust","is_const":false,"is_async":false,"is_unsafe":false}}})
    }
    #[test]
    fn arrays_constants_and_argument_kinds_remain_distinct() {
        let array = actual(json!({"array":{"type":primitive(),"len":"3"}}));
        assert!(TypePattern::parse("[u8; 3]").unwrap().matches(&array));
        assert_eq!(
            TypePattern::parse("[u8; 4]").unwrap().evaluate(&array),
            ShapeMatch::Different
        );
        assert!(matches!(
            TypePattern::parse("[u8; N]").unwrap().evaluate(&array),
            ShapeMatch::Unknown(_)
        ));
        let generic = actual(
            json!({"resolved_path":{"path":"owner::Container","id":91,"args":{"angle_bracketed":{"args":[{"lifetime":"'a"},{"type":primitive()},{"const":{"expr":"N","value":null,"is_literal":false}}],"constraints":[]}}}}),
        );
        assert!(
            TypePattern::parse("Container<'a, u8, {N}>")
                .unwrap()
                .matches(&generic)
        );
        assert_eq!(
            TypePattern::parse("Container<'b, u8, {N}>")
                .unwrap()
                .evaluate(&generic),
            ShapeMatch::Different
        );
        assert_eq!(
            TypePattern::parse("Container<'a, u8, N>")
                .unwrap()
                .evaluate(&generic),
            ShapeMatch::Different
        );
        assert!(matches!(
            TypePattern::parse("Container<'a, u8, {N + 1}>")
                .unwrap()
                .evaluate(&generic),
            ShapeMatch::Unknown(_)
        ));
    }
    #[test]
    fn callback_binders_alpha_match_without_capturing_an_outer_lifetime() {
        let pointer = actual(callback(
            &["'x"],
            vec![reference("'x")],
            Some(reference("'x")),
        ));
        assert!(
            TypePattern::parse("for<'a> fn(&'a u8) -> &'a u8")
                .unwrap()
                .matches(&pointer)
        );
        assert_eq!(
            TypePattern::parse("fn(&'a u8) -> &'a u8")
                .unwrap()
                .evaluate(&pointer),
            ShapeMatch::Different
        );
        assert_eq!(
            TypePattern::parse("for<'a> unsafe fn(&'a u8) -> &'a u8")
                .unwrap()
                .evaluate(&pointer),
            ShapeMatch::Different
        );
        let nested = actual(callback(
            &["'x"],
            vec![callback(
                &["'y"],
                vec![reference("'x"), reference("'y")],
                None,
            )],
            None,
        ));
        assert!(
            TypePattern::parse("for<'a> fn(for<'b> fn(&'a u8, &'b u8))")
                .unwrap()
                .matches(&nested)
        );
        assert_eq!(
            TypePattern::parse("for<'a> fn(for<'a> fn(&'a u8, &'a u8))")
                .unwrap()
                .evaluate(&nested),
            ShapeMatch::Different
        );
    }
    #[test]
    fn associated_equality_gat_arguments_and_bounds_do_not_collapse() {
        let iterator = actual(
            json!({"impl_trait":[{"trait_bound":{"trait":{"path":"Iterator","id":2,"args":{"angle_bracketed":{"args":[],"constraints":[{"name":"Item","args":{"angle_bracketed":{"args":[{"lifetime":"'a"}],"constraints":[]}},"binding":{"equality":{"type":primitive()}}}]}}},"generic_params":[],"modifier":"none"}}]}),
        );
        assert!(
            TypePattern::parse("impl Iterator<Item<'a> = u8>")
                .unwrap()
                .matches(&iterator)
        );
        assert_eq!(
            TypePattern::parse("impl Iterator<Item<'b> = u8>")
                .unwrap()
                .evaluate(&iterator),
            ShapeMatch::Different
        );
        assert_eq!(
            TypePattern::parse("impl Iterator<Item<'a>: Clone>")
                .unwrap()
                .evaluate(&iterator),
            ShapeMatch::Different
        );
        let callable = actual(
            json!({"dyn_trait":{"traits":[{"trait":{"path":"Fn","id":3,"args":{"parenthesized":{"inputs":[primitive()],"output":primitive()}}},"generic_params":[]}],"lifetime":null}}),
        );
        assert!(
            TypePattern::parse("dyn Fn(u8) -> u8")
                .unwrap()
                .matches(&callable)
        );
        assert_eq!(
            TypePattern::parse("dyn Fn(u16) -> u8")
                .unwrap()
                .evaluate(&callable),
            ShapeMatch::Different
        );
        assert_eq!(
            TypePattern::parse("Fn").unwrap().evaluate(&callable),
            ShapeMatch::Different
        );
    }
    #[test]
    fn query_errors_retain_source_and_wildcards_are_independent() {
        let error = TypePattern::parse("fn(\n &)").unwrap_err();
        assert!(matches!(error, ApiError::Query(QueryError { line: 2, .. })));
        let tuple = actual(json!({"tuple":[{"primitive":"u8"},{"primitive":"u16"}]}));
        assert!(TypePattern::parse("(_, _)").unwrap().matches(&tuple));
        assert_eq!(
            TypePattern::parse("(u8, u8)").unwrap().evaluate(&tuple),
            ShapeMatch::Different
        );
        assert!(TypePattern::parse("dyn A + 'a + 'b").is_err());
    }
}
