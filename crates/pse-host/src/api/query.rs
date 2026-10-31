// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed declaration queries, usable public paths and authored examples over one checked artifact.
use super::{ApiError, Artifact};
use rustdoc_types::{Crate, Id, Item, ItemEnum, Type};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
mod definitions;
pub use definitions::QualificationContext;
/// Revalidate an explicit facade-to-definition qualification against its original
/// acquired dependency edge and source/build inputs; never select a newer artifact.
pub fn validate_definition(
    facade: &Artifact,
    surface_id: &str,
    evidence: &DefinitionEvidence,
    definition: &Artifact,
) -> Result<(), ApiError> {
    definitions::validate_definition(facade, surface_id, evidence, definition)
}

/// Structural kind selected without interpreting rendered documentation.
#[derive(
    Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, clap::ValueEnum, PartialEq, Eq,
)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// A free or associated function.
    Function,
    /// A struct declaration.
    Struct,
    /// An enum declaration.
    Enum,
    /// A union declaration.
    Union,
    /// A trait or trait alias.
    Trait,
    /// A type alias.
    TypeAlias,
    /// A constant or associated constant.
    Constant,
    /// A static declaration.
    Static,
    /// A module declaration.
    Module,
    /// A declarative or procedural macro.
    Macro,
    /// Another rustdoc declaration, retained without guessing its kind.
    Other,
}
fn kind(item: &Item) -> Kind {
    match &item.inner {
        ItemEnum::Function(_) => Kind::Function,
        ItemEnum::Struct(_) => Kind::Struct,
        ItemEnum::Enum(_) => Kind::Enum,
        ItemEnum::Union(_) => Kind::Union,
        ItemEnum::Trait(_) | ItemEnum::TraitAlias(_) => Kind::Trait,
        ItemEnum::TypeAlias(_) => Kind::TypeAlias,
        ItemEnum::Constant { .. } | ItemEnum::AssocConst { .. } => Kind::Constant,
        ItemEnum::Static(_) => Kind::Static,
        ItemEnum::Module(_) => Kind::Module,
        ItemEnum::Macro(_) | ItemEnum::ProcMacro(_) => Kind::Macro,
        _ => Kind::Other,
    }
}
/// One selected query; unspecified qualification filters impose no negative claim.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct Query {
    /// Exact public crate-relative path.
    pub path: Option<String>,
    /// Case-insensitive name/docs substring.
    pub search: Option<String>,
    /// Exact structural kind.
    pub kind: Option<Kind>,
    /// Input type spelling within structured function inputs.
    pub input: Option<String>,
    /// Output type spelling within a structured function output.
    pub output: Option<String>,
    /// Native structural input shape, distinct from the legacy spelling filter.
    pub input_shape: Option<TypePattern>,
    /// Native structural output shape, retaining generic argument structure.
    pub output_shape: Option<TypePattern>,
    /// A declared generic trait-bound path; this does not run a trait solver.
    pub declared_bound: Option<String>,
    /// Declared trait path substring, never a trait-solver query.
    pub trait_name: Option<String>,
    /// Require declared unsafe qualification.
    pub unsafe_only: bool,
    /// Require declared async qualification.
    pub async_only: bool,
    /// Require declared const qualification.
    pub const_only: bool,
    /// Expand authored docs on each returned candidate.
    pub docs: bool,
    /// Expand authored examples, retaining hidden setup.
    pub examples: bool,
    /// Expand this exact format-qualified upstream declaration instead of inferring missing bounds.
    pub declaration: bool,
    /// Facade artifact identities traversed before this declaration; binds continuations.
    pub surface_identity: Option<String>,
    /// Original public query identity, preserved while resolving facade paths.
    pub(crate) request_identity: Option<String>,
}
/// A structural declaration query is not semantic type equality or trait applicability.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TypePattern {
    /// Any type, explicitly written `_`.
    Any,
    /// Exact path components and optional native generic type arguments.
    Path {
        /// Written path, matched at component boundaries.
        name: String,
        /// Structural type arguments; None imposes no argument constraint.
        arguments: Option<Vec<TypePattern>>,
    },
    /// Explicit associated-type declaration shape, without trait applicability inference.
    Qualified {
        /// The associated item's declared name.
        name: String,
        /// Written self type, structurally matched.
        self_type: Box<TypePattern>,
        /// Explicit trait path; None selects an inherent association.
        trait_path: Option<String>,
    },
    /// Borrowed type with exact mutability.
    Reference {
        /// Mutable versus shared reference.
        mutable: bool,
        /// Explicit lifetime spelling, when the query constrains it.
        lifetime: Option<String>,
        /// Pointee type shape.
        inner: Box<TypePattern>,
    },
    /// Tuple shape, including unit.
    Tuple {
        /// Exact ordered elements.
        elements: Vec<TypePattern>,
    },
    /// Unsized slice shape.
    Slice {
        /// Element shape.
        inner: Box<TypePattern>,
    },
    /// Raw pointer shape.
    Pointer {
        /// Mutable versus const pointer.
        mutable: bool,
        /// Pointee shape.
        inner: Box<TypePattern>,
    },
}
impl TypePattern {
    /// Parse Rust type syntax through syn, never substring heuristics.
    pub fn parse(text: &str) -> Result<Self, ApiError> {
        let ty: syn::Type =
            syn::parse_str(text).map_err(|error| super::invalid(format!("type query: {error}")))?;
        Self::from_syn(&ty)
    }
    fn from_syn(ty: &syn::Type) -> Result<Self, ApiError> {
        Ok(match ty {
            syn::Type::Infer(_) => Self::Any,
            syn::Type::Path(path) if path.qself.is_none() => {
                let name = path
                    .path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::");
                let Some(last) = path.path.segments.last() else {
                    return Err(super::invalid("empty type path"));
                };
                let arguments = match &last.arguments {
                    syn::PathArguments::None => None,
                    syn::PathArguments::AngleBracketed(args) => {
                        let mut types = Vec::new();
                        for arg in &args.args {
                            match arg {
                                syn::GenericArgument::Type(ty) => types.push(Self::from_syn(ty)?),
                                _ => {
                                    return Err(super::invalid(
                                        "type query currently requires type-only generic arguments; lifetimes/const/bindings remain explicit unsupported shapes",
                                    ));
                                }
                            }
                        }
                        Some(types)
                    }
                    _ => {
                        return Err(super::invalid(
                            "parenthesized function-trait type queries require an explicit supported structural shape",
                        ));
                    }
                };
                Self::Path { name, arguments }
            }
            syn::Type::Path(path) => {
                let qself = path
                    .qself
                    .as_ref()
                    .ok_or_else(|| super::invalid("qualified type query has no self type"))?;
                if path.path.segments.len() != qself.position + 1 {
                    return Err(super::invalid(
                        "nested associated-type query needs an explicit supported shape",
                    ));
                }
                let associated = path
                    .path
                    .segments
                    .last()
                    .ok_or_else(|| super::invalid("associated type query has no item"))?;
                if !matches!(associated.arguments, syn::PathArguments::None) {
                    return Err(super::invalid(
                        "associated-type generic argument queries require an explicit supported shape",
                    ));
                }
                let trait_path = if qself.position == 0 {
                    None
                } else {
                    let parts = path.path.segments.iter().take(qself.position);
                    if parts
                        .clone()
                        .any(|segment| !matches!(segment.arguments, syn::PathArguments::None))
                    {
                        return Err(super::invalid(
                            "qualified trait generic arguments require an explicit supported shape",
                        ));
                    }
                    Some(
                        parts
                            .map(|segment| segment.ident.to_string())
                            .collect::<Vec<_>>()
                            .join("::"),
                    )
                };
                Self::Qualified {
                    name: associated.ident.to_string(),
                    self_type: Box::new(Self::from_syn(&qself.ty)?),
                    trait_path,
                }
            }
            syn::Type::Reference(reference) => Self::Reference {
                mutable: reference.mutability.is_some(),
                lifetime: reference.lifetime.as_ref().map(ToString::to_string),
                inner: Box::new(Self::from_syn(&reference.elem)?),
            },
            syn::Type::Tuple(tuple) => Self::Tuple {
                elements: tuple
                    .elems
                    .iter()
                    .map(Self::from_syn)
                    .collect::<Result<_, _>>()?,
            },
            syn::Type::Slice(slice) => Self::Slice {
                inner: Box::new(Self::from_syn(&slice.elem)?),
            },
            syn::Type::Ptr(pointer) => Self::Pointer {
                mutable: matches!(pointer.mutability, syn::PointerMutability::Mut(_)),
                inner: Box::new(Self::from_syn(&pointer.elem)?),
            },
            syn::Type::Paren(paren) => return Self::from_syn(&paren.elem),
            _ => {
                return Err(super::invalid(
                    "unsupported structural type query; no spelling fallback was used",
                ));
            }
        })
    }
    /// Compare the actual rustdoc type tree. Path spellings remain declared shape constraints.
    pub fn matches(&self, ty: &Type) -> bool {
        match (self, ty) {
            (Self::Any, _) => true,
            (Self::Path { name, arguments }, Type::ResolvedPath(path)) => {
                if path.path != *name && !path.path.ends_with(&format!("::{name}")) {
                    return false;
                }
                let Some(expected) = arguments else {
                    return true;
                };
                let Some(args) = path.args.as_deref() else {
                    return expected.is_empty();
                };
                let rustdoc_types::GenericArgs::AngleBracketed { args, .. } = args else {
                    return false;
                };
                let actual = args
                    .iter()
                    .filter_map(|arg| match arg {
                        rustdoc_types::GenericArg::Type(ty) => Some(ty),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                actual.len() == expected.len()
                    && expected
                        .iter()
                        .zip(actual)
                        .all(|(shape, ty)| shape.matches(ty))
            }
            (
                Self::Path {
                    name,
                    arguments: None,
                },
                Type::Primitive(value) | Type::Generic(value),
            ) => name == value,
            (
                Self::Qualified {
                    name,
                    self_type,
                    trait_path,
                },
                Type::QualifiedPath {
                    name: actual,
                    self_type: actual_self,
                    trait_,
                    ..
                },
            ) => {
                name == actual
                    && self_type.matches(actual_self)
                    && match (trait_path, trait_) {
                        (Some(expected), Some(actual)) => {
                            actual.path == *expected
                                || actual.path.ends_with(&format!("::{expected}"))
                        }
                        (None, None) => true,
                        _ => false,
                    }
            }
            (
                Self::Reference {
                    mutable,
                    lifetime,
                    inner,
                },
                Type::BorrowedRef {
                    is_mutable,
                    type_,
                    lifetime: actual,
                },
            ) => {
                mutable == is_mutable
                    && lifetime
                        .as_ref()
                        .is_none_or(|lifetime| actual.as_ref() == Some(lifetime))
                    && inner.matches(type_)
            }
            (Self::Pointer { mutable, inner }, Type::RawPointer { is_mutable, type_ }) => {
                mutable == is_mutable && inner.matches(type_)
            }
            (Self::Tuple { elements }, Type::Tuple(types)) => {
                elements.len() == types.len()
                    && elements
                        .iter()
                        .zip(types)
                        .all(|(shape, ty)| shape.matches(ty))
            }
            (Self::Slice { inner }, Type::Slice(ty)) => inner.matches(ty),
            _ => false,
        }
    }
    /// Rustdoc omits the return type for an implicit unit return.
    pub fn matches_return(&self, ty: Option<&Type>) -> bool {
        ty.map_or_else(
            || {
                matches!(self, Self::Any)
                    || matches!(self,Self::Tuple {elements} if elements.is_empty())
            },
            |ty| self.matches(ty),
        )
    }
}
/// Trait provenance is a declaration fact, independent of applicability.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct TraitProvenance {
    /// Trait declaration path when rustdoc exposes one.
    pub path: String,
    /// Required import when a public path is known.
    pub required_import: Option<String>,
    /// Negative impl declaration.
    pub negative: bool,
    /// Compiler-synthetic impl declaration.
    pub synthetic: bool,
    /// Blanket implementation type spelling when present.
    pub blanket: Option<String>,
}
/// Original source location; absent spans remain unknown.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Location {
    /// Original rustdoc source path.
    pub filename: std::path::PathBuf,
    /// Original one-based line and zero-based column.
    pub begin: (usize, usize),
    /// Original exclusive end location.
    pub end: (usize, usize),
}
/// Authored example expectation; reading never executes the example.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Expectation {
    /// Ordinary Rust example expectation.
    CompileAndRun,
    /// `no_run` example expectation.
    CompileOnly,
    /// `compile_fail` example expectation, without attributing a particular failure.
    CompileFail,
    /// `should_panic` example expectation.
    ShouldPanic,
    /// Globally or target-conditionally ignored example.
    Ignored,
}
/// One CommonMark code fence with independently preserved hidden setup and expectation.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Example {
    /// Authored rustdoc expectation, independent of any execution result.
    pub expectation: Expectation,
    /// Original fence flags, including edition and conditional ignore settings.
    pub attributes: Vec<String>,
    /// Original code-fence content.
    pub authored: String,
    /// Visible code with rustdoc hidden setup omitted.
    pub displayed: String,
    /// Code with hidden setup revealed and escaped hashes normalized.
    pub expanded: String,
    /// Hidden setup lines in original order.
    pub hidden_setup: Vec<String>,
    /// Fence start in original documentation UTF-8 bytes.
    pub documentation_byte_start: usize,
    /// Fence end in original documentation UTF-8 bytes.
    pub documentation_byte_end: usize,
    /// Reading this example always reports `not_run`.
    pub execution: &'static str,
}
/// One whole producer page unit. The original typed declaration remains in the addressed artifact.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Candidate {
    /// Reachable import or associated-item path in this addressed crate.
    pub public_path: String,
    /// Artifact-local rustdoc item identity; never compared across artifacts.
    pub item_id: u32,
    /// Typed declaration kind.
    pub kind: Kind,
    /// Original defining declaration evidence used for external reexport qualification.
    pub definition: Option<DefinitionEvidence>,
    /// Requested structural qualification remains unknown without exact defining evidence.
    pub qualification_unknown: Option<String>,
    /// Compact declaration presentation, with explicit omissions.
    pub signature: String,
    /// Whether this presentation includes every signature component.
    pub signature_complete: bool,
    /// Expanded authored docs only when requested.
    pub docs: Option<String>,
    /// Declared rustdoc attributes; absent cfg information stays unknown.
    pub attributes: Vec<String>,
    /// Authored deprecation release and note, when rustdoc supplies them.
    pub deprecation: Option<Deprecation>,
    /// Original definition location when supplied by rustdoc.
    pub source: Option<Location>,
    /// Original artifact-local public reexport declarations traversed to this definition.
    pub reexports: Vec<ReexportSource>,
    /// Required trait import and implementation facts where known.
    pub trait_provenance: Option<TraitProvenance>,
    /// Whether rustdoc explicitly stripped part of this declaration.
    pub incomplete_declaration: bool,
    /// Documentation extraction supplies no ordinary-build availability verdict.
    pub ordinary_build_availability: &'static str,
    /// Whole authored examples only when explicitly requested.
    pub examples: Vec<Example>,
    /// Explicit upstream rustdoc item, opaque outside its reported schema version.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    pub declaration: Option<Box<serde_json::value::RawValue>>,
}
/// Exact acquired definition, distinct from the facade's artifact-local use declaration.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct DefinitionEvidence {
    /// Source-qualified defining package.
    pub package_id: String,
    /// Original defining artifact context.
    pub context_key: String,
    /// Retained defining JSON.
    pub artifact: std::path::PathBuf,
    /// Original defining bytes.
    pub artifact_sha256: String,
    /// Defining artifact-local item ID.
    pub item_id: u32,
    /// Verified public route in that defining artifact.
    pub public_path: String,
}
/// Native rustdoc reexport provenance; these IDs are local to the same retained artifact.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ReexportSource {
    /// Original use declaration, distinct from the definition reached through it.
    pub item_id: u32,
    /// Exact original reexport syntax span when rustdoc retained it.
    pub source: Option<Location>,
}
/// Retained declaration deprecation, independent of its availability in a chosen build.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Deprecation {
    /// Upstream's declared release marker.
    pub since: Option<String>,
    /// Upstream's authored migration note.
    pub note: Option<String>,
}
/// Page cursor binds both immutable evidence and exact query/extraction policy.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cursor {
    version: u32,
    artifact: String,
    query: String,
    request: String,
    offset: usize,
}
fn read_cursor(cursor: &str) -> Result<Cursor, ApiError> {
    use base64::Engine as _;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| super::invalid("invalid API continuation encoding"))?;
    #[derive(Deserialize)]
    struct Header {
        version: u32,
    }
    let header: Header = serde_json::from_slice(&bytes)?;
    if header.version != 2 {
        return Err(super::invalid(
            "unsupported API continuation version; restart the query",
        ));
    }
    Ok(serde_json::from_slice(&bytes)?)
}
/// Bind a continuation to the original query before any reexport routing changes its path.
pub(crate) fn bind_request(query: &mut Query, cursor: Option<&str>) -> Result<(), ApiError> {
    let identity = crate::probe::digest(&serde_json::to_vec(query)?);
    if let Some(cursor) = cursor
        && read_cursor(cursor)?.request != identity
    {
        return Err(super::invalid(
            "API continuation query changed; restart against the selected public surface",
        ));
    }
    query.request_identity = Some(identity);
    Ok(())
}
/// Whole-candidate page with examined scope distinct from its returned size.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Page {
    /// Exact locked source-qualified package.
    pub package_id: String,
    /// Source/target/toolchain/features/extraction identity.
    pub context_key: String,
    /// Original immutable rustdoc bytes.
    pub artifact_sha256: String,
    /// Addressed retained original JSON.
    pub artifact: std::path::PathBuf,
    /// Actual documentation compilation target.
    pub target: String,
    /// Cargo's selected documentation-root unit features.
    pub features: BTreeSet<String>,
    /// Broader Cargo metadata package features, separate from selected doc units.
    pub metadata_features: BTreeSet<String>,
    /// Checked rustdoc JSON schema version.
    pub rustdoc_format: u32,
    /// All typed items examined, independently of this page size.
    pub examined_items: usize,
    /// All discovered declaration paths examined.
    pub examined_public_paths: usize,
    /// Exact number of matches within this artifact and query policy.
    pub matched_candidates: usize,
    /// Relevant candidates whose requested declaration qualification could not be proved.
    pub unknown_candidates: usize,
    /// External module/glob/associated descendants absent from this document's query universe.
    pub unexamined_external_routes: Vec<String>,
    /// Matches not returned on this page.
    pub omitted_candidates: usize,
    /// Complete selected candidate units.
    pub candidates: Vec<Candidate>,
    /// Bound continuation for this exact artifact and query.
    pub continuation: Option<String>,
    /// Known extraction/presentation limits retained on every page.
    pub limitations: Vec<&'static str>,
    /// Addressed facade artifacts traversed to the defining declaration artifact.
    pub facades: Vec<Facade>,
}
/// One native public reexport edge, preserving both immutable artifact contexts.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Facade {
    /// Source-qualified package containing the reexport.
    pub package_id: String,
    /// Exact source/feature/target/toolchain context of that facade.
    pub context_key: String,
    /// Original facade JSON location.
    pub artifact: std::path::PathBuf,
    /// Original facade JSON content identity.
    pub artifact_sha256: String,
    /// Public prefix selected through the facade.
    pub public_prefix: String,
    /// Defining artifact's corresponding usable public prefix.
    pub definition_prefix: String,
}
/// Typed external reexport lead; Cargo supplies its exact source-qualified package edge.
#[derive(Debug)]
pub(crate) struct ExternalUse {
    pub(crate) public_prefix: String,
    pub(crate) crate_name: String,
    pub(crate) definition_path: Vec<String>,
    pub(crate) suffix: String,
}
/// Typed availability shares one schema between unavailable preparation and addressed pages.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "availability", rename_all = "snake_case")]
pub enum Report {
    /// Native structural differences over two original acquired artifacts.
    Comparison {
        /// Whole differences selected by the comparison producer.
        page: super::comparison::Page,
    },
    /// A finalized artifact in the exact currently observed context was selected.
    Prepared {
        /// Whole producer-selected page over that artifact.
        page: Page,
    },
    /// No finalized artifact exists for the selected context; no build was attempted.
    Unprepared {
        /// Original locked package query.
        package_query: String,
        /// Explicit target request, or the observed host default when absent.
        requested_target: Option<String>,
    },
}
#[derive(Clone)]
struct Address {
    path: String,
    id: Id,
    trait_provenance: Option<TraitProvenance>,
    reexports: Vec<Id>,
}
fn public(item: &Item) -> bool {
    matches!(item.visibility, rustdoc_types::Visibility::Public)
}
fn walk(
    document: &Crate,
    id: Id,
    path: String,
    ancestors: &mut BTreeSet<Id>,
    reexports: &[Id],
    output: &mut BTreeMap<String, (Id, Vec<Id>)>,
) {
    if !ancestors.insert(id) {
        return;
    }
    if let Some(item) = document.index.get(&id) {
        if let ItemEnum::Use(import) = &item.inner {
            let mut route = reexports.to_vec();
            route.push(id);
            if let Some(target) = import.id
                && document.index.contains_key(&target)
            {
                walk(document, target, path, ancestors, &route, output);
            } else {
                output.insert(path, (id, route));
            }
        } else {
            output
                .entry(path.clone())
                .or_insert_with(|| (id, reexports.to_vec()));
            if let ItemEnum::Module(module) = &item.inner {
                for child in &module.items {
                    let Some(child_item) = document.index.get(child) else {
                        continue;
                    };
                    if !public(child_item) {
                        continue;
                    }
                    if let ItemEnum::Use(import) = &child_item.inner {
                        if import.is_glob {
                            if let Some(target) = import.id
                                && document.index.contains_key(&target)
                            {
                                let mut route = reexports.to_vec();
                                route.push(*child);
                                walk(document, target, path.clone(), ancestors, &route, output);
                            } else {
                                let mut route = reexports.to_vec();
                                route.push(*child);
                                output.insert(format!("{path}::*"), (*child, route));
                            }
                        } else {
                            walk(
                                document,
                                *child,
                                format!("{path}::{}", import.name),
                                ancestors,
                                reexports,
                                output,
                            );
                        }
                    } else if let Some(name) = &child_item.name {
                        walk(
                            document,
                            *child,
                            format!("{path}::{name}"),
                            ancestors,
                            reexports,
                            output,
                        );
                    }
                }
            }
        }
    }
    ancestors.remove(&id);
}
pub(crate) fn external_uses(document: &Crate, path: Option<&str>) -> Vec<ExternalUse> {
    let root = document
        .index
        .get(&document.root)
        .and_then(|item| item.name.as_deref())
        .unwrap_or("");
    let selected = path.map(|path| {
        if path == root || path.starts_with(&format!("{root}::")) {
            path.to_owned()
        } else {
            format!("{root}::{path}")
        }
    });
    let mut result = Vec::new();
    for address in addresses(document) {
        let Some(Item {
            inner: ItemEnum::Use(import),
            ..
        }) = document.index.get(&address.id)
        else {
            continue;
        };
        let Some(summary) = import.id.and_then(|id| document.paths.get(&id)) else {
            continue;
        };
        let Some(external) = document.external_crates.get(&summary.crate_id) else {
            continue;
        };
        let prefix = address.path.strip_suffix("::*").unwrap_or(&address.path);
        let suffix = match &selected {
            Some(selected) if selected == prefix => String::new(),
            Some(selected) => match selected.strip_prefix(&format!("{prefix}::")) {
                Some(suffix) => suffix.into(),
                None => continue,
            },
            None => String::new(),
        };
        result.push(ExternalUse {
            public_prefix: prefix.into(),
            crate_name: external.name.clone(),
            definition_path: summary.path.clone(),
            suffix,
        });
    }
    result.sort_by_key(|route| std::cmp::Reverse(route.public_prefix.len()));
    result
}
pub(crate) fn definition_prefix(document: &Crate, canonical: &[String]) -> Option<String> {
    let root = document
        .index
        .get(&document.root)
        .and_then(|item| item.name.as_deref())?;
    if canonical.len() == 1 {
        return Some(root.into());
    }
    let ids: BTreeSet<_> = document
        .paths
        .iter()
        .filter_map(|(id, summary)| (summary.path == canonical).then_some(*id))
        .collect();
    addresses(document)
        .into_iter()
        .filter(|address| ids.contains(&address.id))
        .map(|address| address.path)
        .min_by_key(String::len)
}
fn addresses(document: &Crate) -> Vec<Address> {
    let root = document
        .index
        .get(&document.root)
        .and_then(|item| item.name.clone())
        .unwrap_or_default();
    let mut paths = BTreeMap::new();
    walk(
        document,
        document.root,
        root,
        &mut BTreeSet::new(),
        &[],
        &mut paths,
    );
    let mut result = Vec::new();
    for (path, (id, reexports)) in &paths {
        result.push(Address {
            path: path.clone(),
            id: *id,
            trait_provenance: None,
            reexports: reexports.clone(),
        });
        let Some(item) = document.index.get(id) else {
            continue;
        };
        let impls = match &item.inner {
            ItemEnum::Struct(value) => &value.impls,
            ItemEnum::Enum(value) => &value.impls,
            ItemEnum::Union(value) => &value.impls,
            ItemEnum::Primitive(value) => &value.impls,
            ItemEnum::Trait(value) => {
                for associated in &value.items {
                    if let Some(associated_item) = document.index.get(associated)
                        && let Some(name) = &associated_item.name
                    {
                        result.push(Address {
                            path: format!("{path}::{name}"),
                            id: *associated,
                            trait_provenance: None,
                            reexports: reexports.clone(),
                        });
                    }
                }
                continue;
            }
            _ => continue,
        };
        for impl_id in impls {
            let Some(Item {
                inner: ItemEnum::Impl(implementation),
                ..
            }) = document.index.get(impl_id)
            else {
                continue;
            };
            let provenance = implementation.trait_.as_ref().map(|trait_| {
                let public_path = paths
                    .iter()
                    .find_map(|(path, (id, _))| (*id == trait_.id).then(|| path.clone()))
                    .or_else(|| {
                        document
                            .paths
                            .get(&trait_.id)
                            .map(|summary| summary.path.join("::"))
                    });
                TraitProvenance {
                    path: public_path.clone().unwrap_or_else(|| trait_.path.clone()),
                    required_import: public_path,
                    negative: implementation.is_negative,
                    synthetic: implementation.is_synthetic,
                    blanket: implementation.blanket_impl.as_ref().map(type_name),
                }
            });
            let mut associated = implementation.items.clone();
            if !implementation.is_negative
                && let Some(trait_) = &implementation.trait_
                && let Some(Item {
                    inner: ItemEnum::Trait(value),
                    ..
                }) = document.index.get(&trait_.id)
            {
                for default in &value.items {
                    if let Some(item) = document.index.get(default)
                        && matches!(&item.inner, ItemEnum::Function(function) if function.has_body)
                        && !associated
                            .iter()
                            .filter_map(|id| document.index.get(id))
                            .any(|override_| override_.name == item.name)
                    {
                        associated.push(*default);
                    }
                }
            }
            for associated_id in associated {
                if let Some(associated_item) = document.index.get(&associated_id)
                    && let Some(name) = &associated_item.name
                    && (provenance.is_some() || public(associated_item))
                {
                    result.push(Address {
                        path: format!("{path}::{name}"),
                        id: associated_id,
                        trait_provenance: provenance.clone(),
                        reexports: reexports.clone(),
                    });
                }
            }
        }
    }
    result.sort_by(|left, right| (&left.path, left.id.0).cmp(&(&right.path, right.id.0)));
    result.dedup_by(|left, right| {
        left.path == right.path
            && left.id == right.id
            && left.trait_provenance.as_ref().map(|value| &value.path)
                == right.trait_provenance.as_ref().map(|value| &value.path)
    });
    result
}

/// Request-local original public routes reuse the API owner's native address traversal.
#[derive(Debug)]
pub(crate) struct SourceRoutes {
    routes: BTreeMap<(u32, String), Vec<Id>>,
}
impl SourceRoutes {
    pub(crate) fn new(document: &Crate) -> Self {
        Self {
            routes: addresses(document)
                .into_iter()
                .map(|address| ((address.id.0, address.path), address.reexports))
                .collect(),
        }
    }
    pub(crate) fn validate(&self, id: Id, public_path: Option<&str>) -> Result<(), ApiError> {
        if public_path.is_some_and(|path| !self.routes.contains_key(&(id.0, path.into()))) {
            return Err(super::invalid(
                "API public path does not address this declaration in its original artifact",
            ));
        }
        Ok(())
    }
    /// A direct, nonglob original use can nominate the same declared target through native lookup.
    pub(crate) fn reexport_span<'a>(
        &self,
        document: &'a Crate,
        id: Id,
        public_path: Option<&str>,
    ) -> Result<Option<&'a rustdoc_types::Span>, ApiError> {
        let routes = if let Some(path) = public_path {
            vec![self.routes.get(&(id.0, path.into())).ok_or_else(|| {
                super::invalid(
                    "API public path does not address this declaration in its original artifact",
                )
            })?]
        } else {
            self.routes
                .iter()
                .filter_map(|((candidate, _), route)| (*candidate == id.0).then_some(route))
                .collect::<Vec<_>>()
        };
        let mut spans = Vec::new();
        for route in routes {
            for use_id in route {
                let Some(item) = document.index.get(use_id) else {
                    continue;
                };
                let ItemEnum::Use(import) = &item.inner else {
                    continue;
                };
                if import.is_glob || (*use_id != id && import.id != Some(id)) {
                    continue;
                }
                if let Some(span) = item.span.as_ref()
                    && !spans.contains(&span)
                {
                    spans.push(span);
                }
            }
        }
        match spans.as_slice() {
            [] => Ok(None),
            [span] => Ok(Some(*span)),
            _ => Err(super::invalid(
                "API declaration has multiple original reexport spans; select its exact public path",
            )),
        }
    }
}
#[cfg(test)]
pub(super) fn test_public_paths(document: &Crate) -> Vec<(String, Option<String>)> {
    addresses(document)
        .into_iter()
        .map(|address| {
            (
                address.path,
                address
                    .trait_provenance
                    .and_then(|value| value.required_import),
            )
        })
        .collect()
}
/// Presentation of a typed Rust type. Unhandled complexity is explicit, never a guessed type.
pub fn type_name(value: &Type) -> String {
    match value {
        Type::ResolvedPath(path) => format!(
            "{}{}",
            path.path,
            path.args.as_deref().map(arguments).unwrap_or_default()
        ),
        Type::Generic(value) | Type::Primitive(value) => value.clone(),
        Type::Infer => "_".into(),
        Type::BorrowedRef {
            lifetime,
            is_mutable,
            type_,
        } => format!(
            "&{}{}{}",
            lifetime
                .as_ref()
                .map(|value| format!("{value} "))
                .unwrap_or_default(),
            if *is_mutable { "mut " } else { "" },
            type_name(type_)
        ),
        Type::RawPointer { is_mutable, type_ } => format!(
            "*{} {}",
            if *is_mutable { "mut" } else { "const" },
            type_name(type_)
        ),
        Type::Slice(value) => format!("[{}]", type_name(value)),
        Type::Array { type_, len } => format!("[{}; {len}]", type_name(type_)),
        Type::Tuple(values) => format!(
            "({}{})",
            values.iter().map(type_name).collect::<Vec<_>>().join(", "),
            if values.len() == 1 { "," } else { "" }
        ),
        Type::QualifiedPath {
            name,
            args,
            self_type,
            trait_,
        } => format!(
            "<{}{}>::{name}{}",
            type_name(self_type),
            trait_
                .as_ref()
                .map(|value| format!(
                    " as {}{}",
                    value.path,
                    value.args.as_deref().map(arguments).unwrap_or_default()
                ))
                .unwrap_or_default(),
            args.as_deref().map(arguments).unwrap_or_default()
        ),
        Type::FunctionPointer(function) => format!(
            "fn({}){} [full function pointer declaration in artifact]",
            function
                .sig
                .inputs
                .iter()
                .map(|(_, value)| type_name(value))
                .collect::<Vec<_>>()
                .join(", "),
            function
                .sig
                .output
                .as_ref()
                .map(|value| format!(" -> {}", type_name(value)))
                .unwrap_or_default()
        ),
        Type::ImplTrait(_) => "impl <bounds in artifact>".into(),
        Type::DynTrait(_) => "dyn <bounds in artifact>".into(),
        Type::Pat { type_, .. } => format!("{} is <pattern in artifact>", type_name(type_)),
    }
}
fn arguments(value: &rustdoc_types::GenericArgs) -> String {
    match value {
        rustdoc_types::GenericArgs::AngleBracketed { args, constraints } => {
            let mut values: Vec<_> = args
                .iter()
                .map(|arg| match arg {
                    rustdoc_types::GenericArg::Lifetime(value) => value.clone(),
                    rustdoc_types::GenericArg::Type(value) => type_name(value),
                    rustdoc_types::GenericArg::Const(value) => value.expr.clone(),
                    rustdoc_types::GenericArg::Infer => "_".into(),
                })
                .collect();
            if !constraints.is_empty() {
                values.push("<associated constraints in artifact>".into());
            }
            if values.is_empty() {
                String::new()
            } else {
                format!("<{}>", values.join(", "))
            }
        }
        rustdoc_types::GenericArgs::Parenthesized { inputs, output } => format!(
            "({}){}",
            inputs.iter().map(type_name).collect::<Vec<_>>().join(", "),
            output
                .as_ref()
                .map(|value| format!(" -> {}", type_name(value)))
                .unwrap_or_default()
        ),
        _ => "<arguments in artifact>".into(),
    }
}
fn signature(item: &Item) -> (String, bool) {
    let name = item.name.as_deref().unwrap_or("<unnamed>");
    if let ItemEnum::Function(function) = &item.inner {
        let generics = if function.generics.params.is_empty() {
            String::new()
        } else {
            format!(
                "<{}>",
                function
                    .generics
                    .params
                    .iter()
                    .map(|param| param.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let mut signature = format!(
            "{}{}{}fn {name}{generics}({}){}",
            if function.header.is_const {
                "const "
            } else {
                ""
            },
            if function.header.is_async {
                "async "
            } else {
                ""
            },
            if function.header.is_unsafe {
                "unsafe "
            } else {
                ""
            },
            function
                .sig
                .inputs
                .iter()
                .map(|(name, type_)| format!("{name}: {}", type_name(type_)))
                .collect::<Vec<_>>()
                .join(", "),
            function
                .sig
                .output
                .as_ref()
                .map(|value| format!(" -> {}", type_name(value)))
                .unwrap_or_default()
        );
        let complete = function.generics.params.is_empty()
            && function.generics.where_predicates.is_empty()
            && !signature.contains("in artifact")
            && !function.sig.is_c_variadic
            && matches!(function.header.abi, rustdoc_types::Abi::Rust);
        if !complete {
            signature.push_str(" [full declaration and constraints in artifact]");
        }
        (signature, complete)
    } else {
        (format!("{:?} {name}", kind(item)), false)
    }
}
fn matches(item: &Item, address: &Address, query: &Query) -> bool {
    if let Some(kind_) = query.kind
        && kind(item) != kind_
    {
        return false;
    }
    if let Some(path) = &query.path
        && address.path != *path
        && !address.path.ends_with(&format!("::{path}"))
    {
        return false;
    }
    if let Some(search) = &query.search {
        let search = search.to_lowercase();
        if !address.path.to_lowercase().contains(&search)
            && !item
                .docs
                .as_ref()
                .is_some_and(|docs| docs.to_lowercase().contains(&search))
        {
            return false;
        }
    }
    if let Some(trait_) = &query.trait_name
        && !address
            .trait_provenance
            .as_ref()
            .is_some_and(|value| value.path.contains(trait_))
    {
        return false;
    }
    if let Some(bound) = &query.declared_bound {
        let ItemEnum::Function(function) = &item.inner else {
            return false;
        };
        let is_bound = |candidate: &rustdoc_types::GenericBound| matches!(candidate,rustdoc_types::GenericBound::TraitBound {trait_,..} if trait_.path==*bound||trait_.path.ends_with(&format!("::{bound}")));
        let parameter=function.generics.params.iter().any(|parameter|matches!(&parameter.kind,rustdoc_types::GenericParamDefKind::Type {bounds,..} if bounds.iter().any(is_bound)));
        let where_=function.generics.where_predicates.iter().any(|predicate|matches!(predicate,rustdoc_types::WherePredicate::BoundPredicate {bounds,..} if bounds.iter().any(is_bound)));
        if !parameter && !where_ {
            return false;
        }
    }
    if query.input.is_some()
        || query.output.is_some()
        || query.input_shape.is_some()
        || query.output_shape.is_some()
        || query.unsafe_only
        || query.async_only
        || query.const_only
    {
        let ItemEnum::Function(function) = &item.inner else {
            return false;
        };
        if query.unsafe_only && !function.header.is_unsafe
            || query.async_only && !function.header.is_async
            || query.const_only && !function.header.is_const
        {
            return false;
        }
        if let Some(input) = &query.input
            && !function
                .sig
                .inputs
                .iter()
                .any(|(_, value)| type_name(value).contains(input))
        {
            return false;
        }
        if let Some(output) = &query.output
            && !function
                .sig
                .output
                .as_ref()
                .is_some_and(|value| type_name(value).contains(output))
        {
            return false;
        }
        if query
            .input_shape
            .as_ref()
            .is_some_and(|shape| !function.sig.inputs.iter().any(|(_, ty)| shape.matches(ty)))
        {
            return false;
        }
        if query
            .output_shape
            .as_ref()
            .is_some_and(|shape| !shape.matches_return(function.sig.output.as_ref()))
        {
            return false;
        }
    }
    true
}
fn external_descendants_relevant(
    document: &Crate,
    item: &Item,
    _address: &Address,
    _query: &Query,
) -> bool {
    let ItemEnum::Use(import) = &item.inner else {
        return false;
    };
    if import.id.is_some_and(|id| document.index.contains_key(&id)) {
        return false;
    }
    let module = import
        .id
        .and_then(|id| document.paths.get(&id))
        .is_none_or(|summary| {
            matches!(
                summary.kind,
                rustdoc_types::ItemKind::Module
                    | rustdoc_types::ItemKind::Struct
                    | rustdoc_types::ItemKind::Enum
                    | rustdoc_types::ItemKind::Union
                    | rustdoc_types::ItemKind::Trait
                    | rustdoc_types::ItemKind::TraitAlias
                    | rustdoc_types::ItemKind::TypeAlias
                    | rustdoc_types::ItemKind::Primitive
                    | rustdoc_types::ItemKind::ExternType
            )
        });
    // matches() accepts suffix paths as well as exact routes. Any unseen descendant
    // can therefore satisfy a short or nested suffix beneath an external container.
    import.is_glob || module
}
/// Extract authored Rust fences without rustdoc transformation or code execution.
pub fn examples(docs: &str) -> Vec<Example> {
    use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};
    let mut result = Vec::new();
    let mut active: Option<(Vec<String>, usize, String)> = None;
    for (event, range) in pulldown_cmark::Parser::new(docs).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(block)) => {
                let info = match block {
                    CodeBlockKind::Fenced(info) => info.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                let attrs: Vec<String> = info
                    .split([',', ' ', '\t'])
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned)
                    .collect();
                let rust = attrs.first().is_none_or(|value| {
                    matches!(
                        value.as_str(),
                        "rust" | "no_run" | "compile_fail" | "should_panic" | "ignore"
                    ) || value.starts_with("edition")
                        || value.starts_with("ignore-")
                        || value.starts_with("E0")
                });
                if rust {
                    active = Some((attrs, range.start, String::new()));
                }
            }
            Event::Text(text) => {
                if let Some((_, _, text_)) = &mut active {
                    text_.push_str(&text);
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some((attributes, start, authored)) = active.take() {
                    let expectation = if attributes
                        .iter()
                        .any(|value| value == "ignore" || value.starts_with("ignore-"))
                    {
                        Expectation::Ignored
                    } else if attributes.iter().any(|value| value == "compile_fail") {
                        Expectation::CompileFail
                    } else if attributes.iter().any(|value| value == "no_run") {
                        Expectation::CompileOnly
                    } else if attributes.iter().any(|value| value == "should_panic") {
                        Expectation::ShouldPanic
                    } else {
                        Expectation::CompileAndRun
                    };
                    let mut displayed = String::new();
                    let mut expanded = String::new();
                    let mut hidden_setup = Vec::new();
                    for line in authored.lines() {
                        let trimmed = line.trim_start();
                        if trimmed == "#" || trimmed.starts_with("# ") {
                            let hidden = trimmed.strip_prefix("# ").unwrap_or("");
                            hidden_setup.push(hidden.into());
                            expanded.push_str(hidden);
                            expanded.push('\n');
                        } else {
                            let line = if trimmed.starts_with("##") {
                                line.replacen("##", "#", 1)
                            } else {
                                line.to_owned()
                            };
                            displayed.push_str(&line);
                            displayed.push('\n');
                            expanded.push_str(&line);
                            expanded.push('\n');
                        }
                    }
                    result.push(Example {
                        expectation,
                        attributes,
                        authored,
                        displayed,
                        expanded,
                        hidden_setup,
                        documentation_byte_start: start,
                        documentation_byte_end: range.end,
                        execution: "not_run",
                    });
                }
            }
            _ => {}
        }
    }
    result
}
/// Select whole candidates against one immutable artifact; never cut recursive declarations.
pub fn inspect(
    artifact: &Artifact,
    query: &Query,
    limit: usize,
    cursor: Option<&str>,
) -> Result<Page, ApiError> {
    use base64::Engine as _;
    if limit == 0 {
        return Err(super::invalid("API page limit must be positive"));
    }
    let addresses = addresses(&artifact.document);
    let examined_public_paths = addresses.len();
    let mut definitions = definitions::Definitions::default();
    let mut selected = Vec::new();
    let mut unexamined_external_routes = Vec::new();
    let surface = Query {
        path: query.path.clone(),
        search: query.search.clone(),
        ..Query::default()
    };
    let needs_definition = query.kind.is_some()
        || query.input.is_some()
        || query.output.is_some()
        || query.input_shape.is_some()
        || query.output_shape.is_some()
        || query.declared_bound.is_some()
        || query.trait_name.is_some()
        || query.unsafe_only
        || query.async_only
        || query.const_only;
    for address in addresses {
        let Some(item) = artifact.document.index.get(&address.id) else {
            continue;
        };
        if external_descendants_relevant(&artifact.document, item, &address, query) {
            unexamined_external_routes.push(address.path.clone());
        }
        // A relevant external use is a real authored surface candidate. Its absent target
        // cannot establish a negative declaration-shape answer.
        if matches!(&item.inner, ItemEnum::Use(_)) && matches(item, &address, &surface) {
            match definitions.resolve(artifact, address.id) {
                Ok((definition, id, evidence)) => {
                    let target = definition.document.index.get(&id).ok_or_else(|| {
                        super::invalid("verified defining declaration disappeared")
                    })?;
                    if matches(target, &address, query) {
                        selected.push((address, Some((definition, id, evidence)), None));
                    }
                }
                Err(error) => {
                    selected.push((address, None, needs_definition.then(|| error.to_string())))
                }
            }
        } else if matches(item, &address, query) {
            selected.push((address, None, None));
        }
    }
    // Additional defining evidence is part of the cursor, including missing-to-prepared changes.
    let evidence = selected
        .iter()
        .map(|(address, definition, unknown)| {
            (
                &address.path,
                definition.as_ref().map(|(_, _, proof)| proof),
                unknown,
            )
        })
        .collect::<Vec<_>>();
    let query_digest = crate::probe::digest(&serde_json::to_vec(&(
        query,
        evidence,
        &unexamined_external_routes,
    ))?);
    let offset = if let Some(cursor) = cursor {
        let cursor = read_cursor(cursor)?;
        if cursor.artifact != artifact.sha256 || cursor.query != query_digest {
            return Err(super::invalid(
                "API continuation source, defining evidence or query changed; restart",
            ));
        }
        cursor.offset
    } else {
        0
    };
    if offset > selected.len() {
        return Err(super::invalid(
            "API continuation offset is outside its selected artifact",
        ));
    }
    let total = selected.len();
    let unknown_candidates = selected
        .iter()
        .filter(|(_, _, unknown)| unknown.is_some())
        .count();
    let candidates = selected
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|(address, definition, qualification_unknown)| {
            let surface_item = artifact
                .document
                .index
                .get(&address.id)
                .ok_or_else(|| super::invalid("selected surface item disappeared"))?;
            let item = if let Some((definition, id, _)) = &definition {
                definition
                    .document
                    .index
                    .get(id)
                    .ok_or_else(|| super::invalid("selected defining item disappeared"))?
            } else {
                surface_item
            };
            let (signature, signature_complete) = signature(item);
            let incomplete_declaration = match &item.inner {
                ItemEnum::Module(value) => value.is_stripped,
                ItemEnum::Struct(value) => match &value.kind {
                    rustdoc_types::StructKind::Plain {
                        has_stripped_fields,
                        ..
                    } => *has_stripped_fields,
                    rustdoc_types::StructKind::Tuple(fields) => fields.iter().any(Option::is_none),
                    rustdoc_types::StructKind::Unit => false,
                },
                ItemEnum::Union(value) => value.has_stripped_fields,
                ItemEnum::Enum(value) => value.has_stripped_variants,
                ItemEnum::Use(_) => true,
                _ => false,
            };
            let declaration = query
                .declaration
                .then(|| serde_json::value::to_raw_value(item))
                .transpose()?;
            Ok::<_, ApiError>(Candidate {
                public_path: address.path,
                item_id: surface_item.id.0,
                kind: kind(item),
                definition: definition.as_ref().map(|(_, _, proof)| proof.clone()),
                qualification_unknown,
                signature,
                signature_complete,
                docs: query.docs.then(|| item.docs.clone()).flatten(),
                attributes: item.attrs.iter().map(|attr| format!("{attr:?}")).collect(),
                deprecation: item.deprecation.as_ref().map(|value| Deprecation {
                    since: value.since.clone(),
                    note: value.note.clone(),
                }),
                source: item.span.as_ref().map(|span| Location {
                    filename: span.filename.clone(),
                    begin: span.begin,
                    end: span.end,
                }),
                reexports: address
                    .reexports
                    .into_iter()
                    .filter_map(|id| artifact.document.index.get(&id))
                    .map(|reexport| ReexportSource {
                        item_id: reexport.id.0,
                        source: reexport.span.as_ref().map(|span| Location {
                            filename: span.filename.clone(),
                            begin: span.begin,
                            end: span.end,
                        }),
                    })
                    .collect(),
                trait_provenance: address.trait_provenance,
                incomplete_declaration,
                ordinary_build_availability: "unknown; rustdoc cfg(doc) is a documentation context",
                examples: if query.examples {
                    item.docs.as_deref().map(examples).unwrap_or_default()
                } else {
                    Vec::new()
                },
                declaration,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let next = offset + candidates.len();
    definitions.verify_unchanged()?;
    let request_identity = query
        .request_identity
        .clone()
        .unwrap_or_else(|| query_digest.clone());
    let continuation = if next < total {
        Some(
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_vec(&Cursor {
                version: 2,
                artifact: artifact.sha256.clone(),
                query: query_digest,
                request: request_identity,
                offset: next,
            })?),
        )
    } else {
        None
    };
    Ok(Page {
        package_id: artifact.context.package_id.clone(),
        context_key: artifact.context.key()?,
        artifact_sha256: artifact.sha256.clone(),
        artifact: artifact.path.clone(),
        target: artifact.context.target.clone(),
        features: artifact.context.features.clone(),
        metadata_features: artifact.context.metadata_features.clone(),
        rustdoc_format: artifact.context.rustdoc_format,
        examined_items: artifact.document.index.len(),
        examined_public_paths,
        matched_candidates: total - unknown_candidates,
        unknown_candidates,
        unexamined_external_routes,
        omitted_candidates: total.saturating_sub(candidates.len()),
        candidates,
        continuation,
        facades: Vec::new(),
        limitations: vec![
            "declarations and trait implementations do not establish arbitrary trait applicability or compiler success",
            "external definitions absent from this artifact require explicitly preparing the addressed crate",
            "signatures marked incomplete retain complete typed declarations in the original artifact",
            "known rustdoc attributes are declaration facts; absent ordinary-build cfg gates remain unknown",
        ],
    })
}

#[cfg(test)]
mod cursor_tests {
    use super::*;
    use base64::Engine as _;

    #[test]
    fn native_type_shapes_preserve_structure_and_component_boundaries() {
        let borrowed = Type::BorrowedRef {
            lifetime: Some("'a".into()),
            is_mutable: true,
            type_: Box::new(Type::Slice(Box::new(Type::Primitive("u8".into())))),
        };
        assert!(TypePattern::parse("&mut [u8]").unwrap().matches(&borrowed));
        assert!(!TypePattern::parse("&[u8]").unwrap().matches(&borrowed));
        assert!(!TypePattern::parse("&mut [u16]").unwrap().matches(&borrowed));
        assert!(
            !TypePattern::parse("&'static mut [u8]")
                .unwrap()
                .matches(&borrowed)
        );
        assert!(TypePattern::parse("()").unwrap().matches_return(None));
        assert!(!TypePattern::parse("u8").unwrap().matches_return(None));
        let path = Type::ResolvedPath(rustdoc_types::Path {
            path: "alloc::vec::Vec".into(),
            id: Id(7),
            args: Some(Box::new(rustdoc_types::GenericArgs::AngleBracketed {
                args: vec![rustdoc_types::GenericArg::Type(Type::Primitive(
                    "u8".into(),
                ))],
                constraints: Vec::new(),
            })),
        });
        assert!(TypePattern::parse("Vec<u8>").unwrap().matches(&path));
        assert!(TypePattern::parse("Vec<_>").unwrap().matches(&path));
        assert!(!TypePattern::parse("Vec<u16>").unwrap().matches(&path));
        assert!(!TypePattern::parse("ec<u8>").unwrap().matches(&path));
        let associated = Type::QualifiedPath {
            name: "Item".into(),
            args: None,
            self_type: Box::new(Type::Generic("T".into())),
            trait_: Some(rustdoc_types::Path {
                path: "core::iter::Iterator".into(),
                id: Id(8),
                args: None,
            }),
        };
        assert!(
            TypePattern::parse("<T as Iterator>::Item")
                .unwrap()
                .matches(&associated)
        );
        assert!(
            !TypePattern::parse("<T as OtherIterator>::Item")
                .unwrap()
                .matches(&associated)
        );
        assert!(
            !TypePattern::parse("<U as Iterator>::Item")
                .unwrap()
                .matches(&associated)
        );
        assert!(
            !TypePattern::parse("<T>::Item")
                .unwrap()
                .matches(&associated)
        );
        assert!(TypePattern::parse("dyn Trait").is_err());
    }

    #[test]
    fn original_query_is_checked_before_reexport_routing_and_keeps_its_binding() {
        let mut original = Query {
            search: Some("spawn".into()),
            ..Query::default()
        };
        bind_request(&mut original, None).unwrap();
        let request = original.request_identity.clone().unwrap();
        let cursor = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(&Cursor {
                version: 2,
                artifact: "defining-artifact".into(),
                query: "routed-query".into(),
                request: request.clone(),
                offset: 1,
            })
            .unwrap(),
        );
        let mut unchanged = Query {
            search: Some("spawn".into()),
            ..Query::default()
        };
        bind_request(&mut unchanged, Some(&cursor)).unwrap();
        unchanged.path = Some("defining::method".into());
        unchanged.surface_identity = Some("facade-artifact".into());
        assert_eq!(unchanged.request_identity, Some(request));
        let mut changed = Query {
            search: Some("changed-query-with-no-matches".into()),
            ..Query::default()
        };
        assert!(
            bind_request(&mut changed, Some(&cursor))
                .unwrap_err()
                .to_string()
                .contains("continuation query changed")
        );
    }

    #[test]
    fn retired_cursor_version_refuses_before_new_shape_decoding() {
        let cursor = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(br#"{"version":1}"#);
        assert!(
            read_cursor(&cursor)
                .unwrap_err()
                .to_string()
                .contains("unsupported API continuation version")
        );
    }
}
