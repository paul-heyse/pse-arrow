// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded declared adjacency in acquired artifacts; never a native applicability graph.
use super::{ApiError, Artifact, Id, ItemEnum, definitions};
use crate::investigation::{Locator, ResultReference};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Declared artifact relation selected independently of native relationships.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    JsonSchema,
    clap::ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    /// Declared child/associated item.
    Contains,
    /// Explicit supertrait or structured declared bound.
    Bound,
    /// An impl record, without a use-site applicability verdict.
    Implementation,
    /// Default trait method supplied by an impl's recorded provided-method list.
    ProvidedDefault,
    /// Public facade to its definition.
    Reexport,
    /// Compiler-resolved intra-documentation link.
    DocumentationLink,
}
/// One qualified declaration or explicitly incomplete original path summary.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Node {
    /// Response-local graph address.
    pub id: usize,
    /// Exact acquired artifact and context-qualified action.
    pub reference: ResultReference,
    /// Original declaration name or summary path; never an identity join.
    pub name: Option<String>,
    /// A summary is not a full defining declaration.
    pub declaration_available: bool,
}
/// Original item witness and explicit impl/default/constraint qualifiers.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Witness {
    /// Response-local witness address.
    pub id: usize,
    /// Original item supplying this adjacency.
    pub reference: ResultReference,
    /// Original resolved-link label, when applicable.
    pub label: Option<String>,
    /// Native declaration projection of relevant generics/bounds and metadata.
    pub contract: super::contracts::Contract,
    /// Native impl flags remain declaration facts, not solver results.
    pub implementation: Option<ImplProperties>,
}
/// Native rustdoc impl distinctions.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ImplProperties {
    /// Original implemented trait, when this is a trait impl.
    pub trait_: Option<super::contracts::Path>,
    /// Original implemented-for type; no receiver adjustment is inferred.
    pub for_: super::contracts::Type,
    /// Explicit negative impl.
    pub negative: bool,
    /// Compiler-synthetic impl.
    pub synthetic: bool,
    /// Original blanket implementation type, when supplied.
    pub blanket: Option<super::contracts::Type>,
}
/// Canonical declared edge with an original artifact witness.
pub type Edge = crate::investigation::graph::Edge<Relation>;
/// Settled compact artifact graph, separate from working-source native traversal.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Graph {
    /// Fixed provider domain.
    pub domain: &'static str,
    /// Original selected item index.
    pub seed: usize,
    /// Requested relation kinds.
    pub relations: Vec<Relation>,
    /// Qualified shared nodes.
    pub nodes: Vec<Node>,
    /// Original item witnesses.
    pub witnesses: Vec<Witness>,
    /// Canonical selected edges.
    pub edges: Vec<Edge>,
    /// Within-inquiry declared adjacency completed, not effective callable availability.
    pub complete: bool,
    /// Provider omissions and missing definitions, without invented totals.
    pub limitations: Vec<String>,
}
struct Catalog {
    graph: Graph,
    keys: BTreeMap<(String, String, u32), usize>,
    definitions: definitions::Definitions,
    contracts: BTreeMap<(String, String, u32), super::contracts::Contract>,
    parents: BTreeMap<(String, String), super::contracts::ParentIndex>,
    limit: usize,
}
impl Catalog {
    fn reference(artifact: &Artifact, id: Id) -> Result<ResultReference, ApiError> {
        Ok(ResultReference::new(Locator::Api {
            artifact: artifact.path.clone(),
            item: id.0.to_string(),
            artifact_sha256: Some(artifact.sha256.clone()),
            context_key: Some(artifact.context.key()?),
            public_path: artifact.document.paths.get(&id).map(|p| p.path.join("::")),
            definition: None,
        }))
    }
    fn node(&mut self, artifact: &Artifact, id: Id) -> Result<usize, ApiError> {
        let key = (artifact.sha256.clone(), artifact.context.key()?, id.0);
        if let Some(index) = self.keys.get(&key) {
            return Ok(*index);
        }
        let index = self.graph.nodes.len();
        let item = artifact.document.index.get(&id);
        let name = item
            .and_then(|i| i.name.clone())
            .or_else(|| artifact.document.paths.get(&id).map(|p| p.path.join("::")));
        if item.is_none() {
            self.unknown(format!(
                "artifact-local item {} is only an external summary or unavailable definition",
                id.0
            ));
        }
        self.graph.nodes.push(Node {
            id: index,
            reference: Self::reference(artifact, id)?,
            name,
            declaration_available: item.is_some(),
        });
        self.keys.insert(key, index);
        Ok(index)
    }
    fn unknown(&mut self, message: String) {
        self.graph.complete = false;
        if !self.graph.limitations.contains(&message) {
            self.graph.limitations.push(message);
        }
    }
    fn edge(
        &mut self,
        artifact: &Artifact,
        source: Id,
        target: Id,
        relation: Relation,
        label: Option<String>,
    ) -> Result<bool, ApiError> {
        if !artifact.document.index.contains_key(&target) {
            match self.definitions.resolve(artifact, target) {
                Ok((definition, id, _)) => {
                    return self.cross_edge(artifact, source, &definition, id, relation, label);
                }
                Err(error) => self.unknown(format!(
                    "qualified target {} unavailable: {error}",
                    target.0
                )),
            }
        }
        self.cross_edge(artifact, source, artifact, target, relation, label)
    }
    fn cross_edge(
        &mut self,
        from: &Artifact,
        source: Id,
        to: &Artifact,
        target: Id,
        relation: Relation,
        label: Option<String>,
    ) -> Result<bool, ApiError> {
        if self.graph.edges.len() >= self.limit {
            self.unknown(
                "declared adjacency edge allowance reached; omitted total is unknown".into(),
            );
            return Ok(false);
        }
        let (owner, witness_id) = if relation == Relation::Implementation {
            (to, target)
        } else {
            (from, source)
        };
        let Some(item) = owner.document.index.get(&witness_id) else {
            self.unknown("original witness item is unavailable".into());
            return Ok(true);
        };
        let source = self.node(from, source)?;
        let target = self.node(to, target)?;
        let witness = self.graph.witnesses.len();
        let implementation = if let ItemEnum::Impl(v) = &item.inner {
            Some(ImplProperties {
                trait_: v
                    .trait_
                    .as_ref()
                    .map(super::contracts::project)
                    .transpose()?,
                for_: super::contracts::project(&v.for_)?,
                negative: v.is_negative,
                synthetic: v.is_synthetic,
                blanket: v
                    .blanket_impl
                    .as_ref()
                    .map(super::contracts::project)
                    .transpose()?,
            })
        } else {
            None
        };
        let key = (owner.sha256.clone(), owner.context.key()?, item.id.0);
        if !self.contracts.contains_key(&key) {
            let parents = self
                .parents
                .entry((key.0.clone(), key.1.clone()))
                .or_insert_with(|| super::contracts::ParentIndex::new(&owner.document));
            self.contracts.insert(
                key.clone(),
                super::contracts::contract_with_parents(&owner.document, item, parents)?,
            );
        }
        let contract = self.contracts[&key].clone();
        self.graph.witnesses.push(Witness {
            id: witness,
            reference: Self::reference(owner, item.id)?,
            label,
            contract,
            implementation,
        });
        self.graph.edges.push(Edge {
            source,
            target,
            relation,
            witness,
        });
        Ok(true)
    }
}
/// Examine only the selected item's demanded declared relationships, loading each retained
/// defining artifact through the existing request-owned qualification cache.
pub fn inspect(
    artifact: &Artifact,
    item: Id,
    relations: &[Relation],
    limit: usize,
) -> Result<Graph, ApiError> {
    if relations.is_empty() || limit == 0 {
        return Err(super::super::invalid(
            "API adjacency requires nonempty relations and a positive edge limit",
        ));
    }
    if !artifact.document.index.contains_key(&item) {
        return Err(super::super::invalid(
            "API adjacency seed is not a complete declaration in this artifact",
        ));
    }
    let mut catalog = Catalog {
        graph: Graph {
            domain: "acquired_api",
            seed: 0,
            relations: relations.to_vec(),
            nodes: Vec::new(),
            witnesses: Vec::new(),
            edges: Vec::new(),
            complete: true,
            limitations: Vec::new(),
        },
        keys: BTreeMap::new(),
        definitions: definitions::Definitions::default(),
        contracts: BTreeMap::new(),
        parents: BTreeMap::new(),
        limit,
    };
    catalog.node(artifact, item)?;
    let original = &artifact.document.index[&item];
    let mut children = Vec::new();
    let mut impls = Vec::new();
    let mut bounds = Vec::new();
    if let Some(generics) = super::contracts::generics(original) {
        for parameter in &generics.params {
            if let rustdoc_types::GenericParamDefKind::Type { bounds: values, .. } = &parameter.kind
            {
                bounds.extend(values.iter());
            }
        }
        for predicate in &generics.where_predicates {
            if let rustdoc_types::WherePredicate::BoundPredicate { bounds: values, .. } = predicate
            {
                bounds.extend(values.iter());
            }
        }
    }
    match &original.inner {
        ItemEnum::Module(v) => {
            children.extend(v.items.iter().copied());
            if v.is_stripped {
                catalog.unknown("selected module has stripped items".into());
            }
        }
        ItemEnum::Trait(v) => {
            children.extend(v.items.iter().copied());
            impls.extend(v.implementations.iter().copied());
            bounds.extend(v.bounds.iter());
        }
        ItemEnum::TraitAlias(v) => bounds.extend(v.params.iter()),
        ItemEnum::AssocType { bounds: values, .. } => bounds.extend(values.iter()),
        ItemEnum::Impl(v) => {
            children.extend(v.items.iter().copied());
            if relations.contains(&Relation::ProvidedDefault)
                && !v.provided_trait_methods.is_empty()
                && let Some(trait_) = &v.trait_
            {
                if let Some(trait_item) = artifact.document.index.get(&trait_.id) {
                    if let ItemEnum::Trait(data) = &trait_item.inner {
                        for id in &data.items {
                            if let Some(default) = artifact.document.index.get(id)
                                && matches!(&default.inner, ItemEnum::Function(_))
                                && default
                                    .name
                                    .as_ref()
                                    .is_some_and(|name| v.provided_trait_methods.contains(name))
                                && !catalog.edge(
                                    artifact,
                                    item,
                                    *id,
                                    Relation::ProvidedDefault,
                                    default.name.clone(),
                                )?
                            {
                                break;
                            }
                        }
                    }
                } else {
                    match catalog.definitions.resolve(artifact, trait_.id) {
                        Ok((definition, trait_id, _)) => {
                            if let Some(rustdoc_types::Item {
                                inner: ItemEnum::Trait(data),
                                ..
                            }) = definition.document.index.get(&trait_id)
                            {
                                for id in &data.items {
                                    if let Some(default) = definition.document.index.get(id)
                                        && matches!(&default.inner, ItemEnum::Function(_))
                                        && default.name.as_ref().is_some_and(|name| {
                                            v.provided_trait_methods.contains(name)
                                        })
                                        && !catalog.cross_edge(
                                            artifact,
                                            item,
                                            &definition,
                                            *id,
                                            Relation::ProvidedDefault,
                                            default.name.clone(),
                                        )?
                                    {
                                        break;
                                    }
                                }
                            } else {
                                catalog.unknown(
                                    "acquired default owner is not a complete trait".into(),
                                );
                            }
                        }
                        Err(error) => catalog.unknown(format!(
                            "provided defaults require a qualified acquired trait: {error}"
                        )),
                    }
                }
            }
        }
        ItemEnum::Struct(v) => {
            impls.extend(v.impls.iter().copied());
            match &v.kind {
                rustdoc_types::StructKind::Plain {
                    fields,
                    has_stripped_fields,
                } => {
                    children.extend(fields.iter().copied());
                    if *has_stripped_fields {
                        catalog.unknown("selected struct has stripped fields".into());
                    }
                }
                rustdoc_types::StructKind::Tuple(v) => {
                    children.extend(v.iter().flatten().copied());
                    if v.iter().any(Option::is_none) {
                        catalog.unknown("selected tuple struct has stripped fields".into());
                    }
                }
                _ => {}
            }
        }
        ItemEnum::Enum(v) => {
            impls.extend(v.impls.iter().copied());
            children.extend(v.variants.iter().copied());
            if v.has_stripped_variants {
                catalog.unknown("selected enum has stripped variants".into());
            }
        }
        ItemEnum::Union(v) => {
            impls.extend(v.impls.iter().copied());
            children.extend(v.fields.iter().copied());
            if v.has_stripped_fields {
                catalog.unknown("selected union has stripped fields".into());
            }
        }
        ItemEnum::Primitive(v) => impls.extend(v.impls.iter().copied()),
        _ => {}
    }
    if relations.contains(&Relation::Contains) {
        for child in children {
            if !catalog.edge(artifact, item, child, Relation::Contains, None)? {
                break;
            }
        }
    }
    if relations.contains(&Relation::Implementation) {
        for implementation in impls {
            if !catalog.edge(
                artifact,
                item,
                implementation,
                Relation::Implementation,
                None,
            )? {
                break;
            }
        }
    }
    if relations.contains(&Relation::Bound) {
        for bound in bounds {
            if let rustdoc_types::GenericBound::TraitBound { trait_, .. } = bound
                && !catalog.edge(artifact, item, trait_.id, Relation::Bound, None)?
            {
                break;
            }
        }
    }
    if relations.contains(&Relation::DocumentationLink) {
        for (label, target) in &original.links {
            if !catalog.edge(
                artifact,
                item,
                *target,
                Relation::DocumentationLink,
                Some(label.clone()),
            )? {
                break;
            }
        }
    }
    if relations.contains(&Relation::Reexport)
        && let ItemEnum::Use(v) = &original.inner
    {
        if let Some(target) = v.id {
            if artifact.document.index.contains_key(&target) {
                catalog.edge(
                    artifact,
                    item,
                    target,
                    Relation::Reexport,
                    Some(v.source.clone()),
                )?;
            } else {
                match catalog.definitions.resolve(artifact, item) {
                    Ok((definition, target, proof)) => {
                        let witness_index = catalog.graph.witnesses.len();
                        catalog.cross_edge(
                            artifact,
                            item,
                            &definition,
                            target,
                            Relation::Reexport,
                            Some(v.source.clone()),
                        )?;
                        if let Some(witness) = catalog.graph.witnesses.get_mut(witness_index)
                            && let Locator::Api { definition, .. } = &mut witness.reference.locator
                        {
                            *definition = Some(proof);
                        }
                    }
                    Err(error) => {
                        catalog.edge(
                            artifact,
                            item,
                            target,
                            Relation::Reexport,
                            Some(v.source.clone()),
                        )?;
                        catalog.unknown(error.to_string());
                    }
                }
            }
        } else {
            catalog.unknown("reexport has no resolved target".into());
        }
    }
    catalog.definitions.verify_unchanged()?;
    // Even a complete declared graph does not establish use-site receiver/import/obligation fit.
    catalog.graph.limitations.push("declared artifact adjacency only; effective callable surface and destination applicability require selected native evidence".into());
    Ok(catalog.graph)
}
