// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Schema61 declaration contracts. Display strings are never parsed for these facts.
// These selected native schema shapes form the public consumer contract. Keeping them
// typed also gives command-schema clients the recursive constraints hidden by signatures.
#![allow(
    missing_docs,
    reason = "fields preserve the named schema61 declaration protocol"
)]
use rustdoc_types::{Item, ItemEnum};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Generics {
    pub params: Vec<Parameter>,
    pub where_predicates: Vec<Predicate>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Parameter {
    pub name: String,
    pub kind: ParameterKind,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParameterKind {
    Lifetime {
        outlives: Vec<String>,
    },
    Type {
        bounds: Vec<Bound>,
        default: Option<Type>,
        is_synthetic: bool,
    },
    Const {
        #[serde(rename = "type")]
        type_: Type,
        default: Option<String>,
    },
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Predicate {
    #[serde(rename = "bound_predicate")]
    Bound {
        #[serde(rename = "type")]
        type_: Type,
        bounds: Vec<Bound>,
        generic_params: Vec<Parameter>,
    },
    #[serde(rename = "lifetime_predicate")]
    Lifetime {
        lifetime: String,
        outlives: Vec<String>,
    },
    #[serde(rename = "eq_predicate")]
    Equality { lhs: Type, rhs: Term },
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Bound {
    #[serde(rename = "trait_bound")]
    Trait {
        #[serde(rename = "trait")]
        trait_: Path,
        generic_params: Vec<Parameter>,
        modifier: Modifier,
    },
    Outlives(String),
    Use(Vec<Capture>),
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Modifier {
    None,
    Maybe,
    MaybeConst,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Capture {
    Lifetime(String),
    Param(String),
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Path {
    pub path: String,
    pub id: u32,
    pub args: Option<Box<Arguments>>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Arguments {
    AngleBracketed {
        args: Vec<Argument>,
        constraints: Vec<Constraint>,
    },
    Parenthesized {
        inputs: Vec<Type>,
        output: Option<Type>,
    },
    ReturnTypeNotation,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Argument {
    Lifetime(String),
    Type(Type),
    Const(Constant),
    Infer,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Constant {
    pub expr: String,
    pub value: Option<String>,
    pub is_literal: bool,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Constraint {
    pub name: String,
    pub args: Option<Box<Arguments>>,
    pub binding: Binding,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Binding {
    Equality(Term),
    Constraint(Vec<Bound>),
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Term {
    Type(Type),
    Constant(Constant),
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    ResolvedPath(Path),
    DynTrait(DynTrait),
    Generic(String),
    Primitive(String),
    FunctionPointer(Box<FunctionPointer>),
    Tuple(Vec<Type>),
    Slice(Box<Type>),
    Array {
        #[serde(rename = "type")]
        type_: Box<Type>,
        len: String,
    },
    Pat {
        #[serde(rename = "type")]
        type_: Box<Type>,
        __pat_unstable_do_not_use: String,
    },
    ImplTrait(Vec<Bound>),
    Infer,
    RawPointer {
        is_mutable: bool,
        #[serde(rename = "type")]
        type_: Box<Type>,
    },
    BorrowedRef {
        lifetime: Option<String>,
        is_mutable: bool,
        #[serde(rename = "type")]
        type_: Box<Type>,
    },
    QualifiedPath {
        name: String,
        args: Option<Box<Arguments>>,
        self_type: Box<Type>,
        #[serde(rename = "trait")]
        trait_: Option<Path>,
    },
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DynTrait {
    pub traits: Vec<PolyTrait>,
    pub lifetime: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PolyTrait {
    #[serde(rename = "trait")]
    pub trait_: Path,
    pub generic_params: Vec<Parameter>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct FunctionPointer {
    pub sig: Signature,
    pub generic_params: Vec<Parameter>,
    pub header: Header,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Signature {
    pub inputs: Vec<(String, Type)>,
    pub output: Option<Type>,
    pub is_c_variadic: bool,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Header {
    pub is_const: bool,
    pub is_unsafe: bool,
    pub is_async: bool,
    pub abi: Abi,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum Abi {
    Rust,
    C { unwind: bool },
    Cdecl { unwind: bool },
    Stdcall { unwind: bool },
    Fastcall { unwind: bool },
    Aapcs { unwind: bool },
    Win64 { unwind: bool },
    SysV64 { unwind: bool },
    System { unwind: bool },
    Other(String),
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Attribute {
    NonExhaustive,
    MustUse { reason: Option<String> },
    MacroExport,
    ExportName(String),
    LinkSection(String),
    AutomaticallyDerived,
    Repr(Repr),
    NoMangle,
    TargetFeature { enable: Vec<String> },
    Other(String),
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Repr {
    pub kind: ReprKind,
    pub align: Option<u64>,
    pub packed: Option<u64>,
    pub int: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReprKind {
    Rust,
    C,
    Transparent,
    Simd,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Stability {
    pub feature: String,
    pub level: StabilityLevel,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StabilityLevel {
    Stable { since: Option<String> },
    Unstable,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TraitProperties {
    pub is_auto: bool,
    pub is_unsafe: bool,
    pub is_dyn_compatible: bool,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CaptureCategory {
    Lifetime,
    Type,
    Const,
    Unresolved,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CaptureResolution {
    pub argument: String,
    pub category: CaptureCategory,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EnclosingGenerics {
    pub item_id: u32,
    pub generics: Generics,
}

/// Structural declaration contract, not inferred layout, solver results or availability.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Contract {
    pub generics: Option<Generics>,
    pub enclosing_generics: Vec<EnclosingGenerics>,
    pub signature: Option<Signature>,
    pub bounds: Vec<Bound>,
    pub trait_properties: Option<TraitProperties>,
    pub precise_captures: Vec<CaptureResolution>,
    pub stability: Option<Stability>,
    pub const_stability: Option<Stability>,
    /// A recorded unstable default gate; None supplies no stability verdict.
    pub provided_default_unstable: Option<String>,
}
pub(super) fn project<T: Serialize, U: serde::de::DeserializeOwned>(
    value: &T,
) -> Result<U, super::ApiError> {
    Ok(serde_json::from_value(serde_json::to_value(value)?)?)
}
fn generics(item: &Item) -> Option<&rustdoc_types::Generics> {
    Some(match &item.inner {
        ItemEnum::Function(v) => &v.generics,
        ItemEnum::Struct(v) => &v.generics,
        ItemEnum::Enum(v) => &v.generics,
        ItemEnum::Union(v) => &v.generics,
        ItemEnum::Trait(v) => &v.generics,
        ItemEnum::TraitAlias(v) => &v.generics,
        ItemEnum::Impl(v) => &v.generics,
        ItemEnum::TypeAlias(v) => &v.generics,
        ItemEnum::AssocType { generics, .. } => generics,
        _ => return None,
    })
}
pub(super) fn contract(
    document: &rustdoc_types::Crate,
    item: &Item,
) -> Result<Contract, super::ApiError> {
    let mut enclosing = Vec::new();
    // Parent declarations are artifact-local membership edges, never span/name matches.
    let mut selected = item.id;
    let mut seen = std::collections::BTreeSet::new();
    while seen.insert(selected) {
        let parents: Vec<_> = document
            .index
            .values()
            .filter(|parent| match &parent.inner {
                ItemEnum::Trait(v) => v.items.contains(&selected),
                ItemEnum::Impl(v) => v.items.contains(&selected),
                _ => false,
            })
            .collect();
        let [parent] = parents.as_slice() else { break };
        if let Some(generics) = generics(parent) {
            enclosing.push(EnclosingGenerics {
                item_id: parent.id.0,
                generics: project(generics)?,
            });
        }
        selected = parent.id;
    }
    let own_generics = generics(item).map(project).transpose()?;
    let mut categories = BTreeMap::new();
    if matches!(item.inner, ItemEnum::Trait(_) | ItemEnum::Impl(_)) || !enclosing.is_empty() {
        categories.insert("Self".into(), CaptureCategory::Type);
    }
    for generics in enclosing
        .iter()
        .rev()
        .map(|v| &v.generics)
        .chain(own_generics.iter())
    {
        for param in &generics.params {
            categories.insert(
                param.name.clone(),
                match param.kind {
                    ParameterKind::Lifetime { .. } => CaptureCategory::Lifetime,
                    ParameterKind::Type { .. } => CaptureCategory::Type,
                    ParameterKind::Const { .. } => CaptureCategory::Const,
                },
            );
        }
    }
    // Inspect schema61 Use nodes, retaining the enclosing parameter categories above.
    // This walks serde's typed schema representation, never arbitrary attribute text.
    fn captures(value: &serde_json::Value, out: &mut Vec<Capture>) -> Result<(), super::ApiError> {
        match value {
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    if key == "use" && value.is_array() {
                        out.extend(serde_json::from_value::<Vec<Capture>>(value.clone())?);
                    } else {
                        captures(value, out)?;
                    }
                }
            }
            serde_json::Value::Array(values) => {
                for value in values {
                    captures(value, out)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    let mut captures_found = Vec::new();
    captures(&serde_json::to_value(&item.inner)?, &mut captures_found)?;
    let precise_captures = captures_found
        .into_iter()
        .map(|capture| {
            let (argument, category) = match capture {
                Capture::Lifetime(v) => (v, CaptureCategory::Lifetime),
                Capture::Param(v) => {
                    let category = categories
                        .get(&v)
                        .copied()
                        .unwrap_or(CaptureCategory::Unresolved);
                    (v, category)
                }
            };
            CaptureResolution { argument, category }
        })
        .collect();
    let bounds = match &item.inner {
        ItemEnum::Trait(v) => project(&v.bounds)?,
        ItemEnum::TraitAlias(v) => project(&v.params)?,
        ItemEnum::AssocType { bounds, .. } => project(bounds)?,
        _ => Vec::new(),
    };
    let default = match &item.inner {
        ItemEnum::Function(v) => v.default_unstable.as_deref(),
        ItemEnum::AssocConst {
            default_unstable, ..
        }
        | ItemEnum::AssocType {
            default_unstable, ..
        } => default_unstable.as_deref(),
        _ => None,
    };
    Ok(Contract {
        generics: own_generics,
        enclosing_generics: enclosing,
        signature: match &item.inner {
            ItemEnum::Function(v) => Some(project(&v.sig)?),
            _ => None,
        },
        bounds,
        trait_properties: match &item.inner {
            ItemEnum::Trait(v) => Some(TraitProperties {
                is_auto: v.is_auto,
                is_unsafe: v.is_unsafe,
                is_dyn_compatible: v.is_dyn_compatible,
            }),
            _ => None,
        },
        precise_captures,
        stability: item.stability.as_ref().map(project).transpose()?,
        const_stability: item.const_stability.as_ref().map(project).transpose()?,
        provided_default_unstable: default.map(|v| v.feature.clone()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn schema61_contract_keeps_association_hrtb_capture_and_stability_separate() {
        let mut value: serde_json::Value = serde_json::from_str(r##"{"id":1,"crate_id":0,"name":"choose","span":null,"visibility":"public","docs":null,"links":{},"attrs":[{"repr":{"kind":"c","align":16,"packed":null,"int":null}},{"other":"#[unknown(text)]"}],"deprecation":null,"stability":{"feature":"ordinary","level":{"stable":{"since":"1.0"}}},"const_stability":{"feature":"const_gate","level":"unstable"},"inner":{"function":{"sig":{"inputs":[],"output":{"impl_trait":[{"use":[{"lifetime":"'a"},{"param":"T"},{"param":"N"}]}]},"is_c_variadic":false},"generics":{"params":[{"name":"'a","kind":{"lifetime":{"outlives":[]}}},{"name":"T","kind":{"type":{"bounds":[],"default":null,"is_synthetic":false}}},{"name":"N","kind":{"const":{"type":{"primitive":"usize"},"default":null}}}],"where_predicates":[{"bound_predicate":{"type":{"generic":"T"},"bounds":[{"trait_bound":{"trait":{"path":"Iterator","id":2,"args":{"angle_bracketed":{"args":[],"constraints":[{"name":"Item","args":null,"binding":{"equality":{"type":{"generic":"T"}}}}]}}},"generic_params":[{"name":"'b","kind":{"lifetime":{"outlives":[]}}}],"modifier":"none"}}],"generic_params":[]}}]},"header":{"is_const":true,"is_unsafe":false,"is_async":false,"abi":"Rust"},"has_body":true,"default_unstable":{"feature":"body_gate"}}}}"##).unwrap();
        let item: Item = serde_json::from_value(value.clone()).unwrap();
        let document = rustdoc_types::Crate {
            root: item.id,
            crate_version: None,
            includes_private: false,
            index: [(item.id, item.clone())].into(),
            paths: Default::default(),
            external_crates: Default::default(),
            target: rustdoc_types::Target {
                triple: "test-target".into(),
                target_features: vec![],
            },
            format_version: 61,
        };
        let facts = contract(&document, &item).unwrap();
        assert!(matches!(
            facts.precise_captures[0].category,
            CaptureCategory::Lifetime
        ));
        assert!(matches!(
            facts.precise_captures[1].category,
            CaptureCategory::Type
        ));
        assert!(matches!(
            facts.precise_captures[2].category,
            CaptureCategory::Const
        ));
        assert_eq!(
            facts.provided_default_unstable.as_deref(),
            Some("body_gate")
        );
        assert!(matches!(
            facts.stability.unwrap().level,
            StabilityLevel::Stable { .. }
        ));
        assert!(matches!(
            facts.const_stability.unwrap().level,
            StabilityLevel::Unstable
        ));
        let generics = facts.generics.unwrap();
        let Predicate::Bound { bounds, .. } = &generics.where_predicates[0] else {
            panic!("missing predicate")
        };
        let Bound::Trait {
            trait_,
            generic_params,
            ..
        } = &bounds[0]
        else {
            panic!("missing trait bound")
        };
        assert_eq!(generic_params[0].name, "'b");
        let Arguments::AngleBracketed { constraints, .. } = trait_.args.as_deref().unwrap() else {
            panic!("missing arguments")
        };
        assert!(matches!(constraints[0].binding, Binding::Equality(_)));
        value["inner"]["function"]["generics"]["where_predicates"][0]["bound_predicate"]["bounds"]
            [0]["trait_bound"]["trait"]["args"]["angle_bracketed"]["constraints"][0]["binding"] =
            json!({"constraint":[{"outlives":"'a"}]});
        let constrained: Item = serde_json::from_value(value).unwrap();
        let bound = serde_json::to_value(contract(&document, &constrained).unwrap()).unwrap();
        assert!(bound.to_string().contains("constraint"));
        let attrs: Vec<Attribute> = project(&item.attrs).unwrap();
        assert!(matches!(
            attrs[0],
            Attribute::Repr(Repr {
                align: Some(16),
                ..
            })
        ));
        assert!(matches!(attrs[1], Attribute::Other(_)));
    }
}
