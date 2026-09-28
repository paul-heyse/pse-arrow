// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Entity identity resolution (ADR-0115 Outcome 1).
//!
//! A column declares the identity it carries; a foreign-key column inherits the identity
//! of the column it references, transitively; a column that declares one identity and
//! inherits another is refused.
//!
//! Ownership is its own declaration ([`FieldContract::with_owned_identity`]): the owner
//! is the relation where the entity is minted, and its owning column is that relation's
//! single-column primary key. Carrying an identity never implies owning it, so a keyed
//! projection of an entity (a one-row export manifest, say) types its key without
//! becoming a second owner. An identity has at most one owner; identities without an
//! owner are allowed.
//!
//! A nested value (a struct field or list item inside a column) may also declare the
//! identity it carries, so a row that lists entities types them. Ownership and
//! foreign-key inheritance stay properties of top-level columns: a nested value never
//! owns an identity and never inherits one.

use std::collections::{BTreeMap, BTreeSet};

use crate::SchemaError;
use crate::model::{
    ExtensionUse, FieldContract, IdentityBase, IdentityDecl, IdentityOwner, IdentitySpec,
    RelationSpec,
};

/// Resolve every identity facet, fill inherited ones into `relations`, and assemble
/// the declared identities.
///
/// # Errors
/// A duplicate or unused declaration; a facet naming an undeclared identity or on a
/// non-identity field; conflicting identities; mixed bases; or two owners.
pub(super) fn resolve(
    relations: &mut [RelationSpec],
    decls: &[IdentityDecl],
) -> Result<Vec<IdentitySpec>, SchemaError> {
    let mut declared = BTreeMap::new();
    for decl in decls {
        if decl.name.is_empty()
            || !decl
                .name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return Err(crate::checks::invalid(
                format!("identity {}", decl.name),
                "identity names are snake-case identifiers",
            ));
        }
        if declared.insert(decl.name, decl).is_some() {
            return Err(SchemaError::DuplicateDeclaration {
                kind: "identity",
                name: decl.name.to_owned(),
            });
        }
    }
    let mut owners: BTreeMap<&str, IdentityOwner> = BTreeMap::new();
    for spec in relations.iter() {
        for column in &spec.columns {
            for (path, carrier) in carriers(column) {
                if path != column.name() && carrier.owns_identity() {
                    return Err(crate::checks::invalid(
                        format!("column {}.{path}", spec.key),
                        "a nested value never owns an identity",
                    ));
                }
                let Some(name) = carrier.identity() else {
                    continue;
                };
                if !declared.contains_key(name) {
                    return Err(SchemaError::UnknownReference {
                        context: format!("column {}.{path}", spec.key),
                        reference: format!("identity:{name}"),
                    });
                }
            }
            let Some(name) = column.identity() else {
                continue;
            };
            if column.owns_identity() {
                if spec.primary_key.as_slice() != [column.name()] {
                    return Err(crate::checks::invalid(
                        format!("column {}.{}", spec.key, column.name()),
                        format!(
                            "owns identity {name} but is not its relation's single-column primary key"
                        ),
                    ));
                }
                let owner = IdentityOwner {
                    relation_id: spec.id,
                    relation: spec.qualified_name(),
                    column: column.name().to_owned(),
                };
                if let Some(previous) = owners.insert(declared[name].name, owner) {
                    return Err(crate::checks::invalid(
                        format!("identity {name}"),
                        format!(
                            "owned by both {}.{} and {}",
                            previous.relation,
                            previous.column,
                            spec.qualified_name()
                        ),
                    ));
                }
            }
        }
    }
    inherit(relations)?;
    let mut bases: BTreeMap<&str, (IdentityBase, String)> = BTreeMap::new();
    for spec in relations.iter() {
        for (path, column) in spec.columns.iter().flat_map(carriers) {
            let Some(name) = column.identity() else {
                continue;
            };
            let context = format!("{}.{path}", spec.key);
            let base = match column.extension() {
                Some(ExtensionUse::SemanticId) => IdentityBase::SemanticId,
                Some(ExtensionUse::ContentHash) => IdentityBase::ContentHash,
                _ => {
                    return Err(crate::checks::invalid(
                        format!("column {context}"),
                        "an entity identity requires a semantic_id or content_hash column",
                    ));
                }
            };
            let name = declared[name].name;
            match bases.get(name) {
                Some((first, at)) if *first != base => {
                    return Err(crate::checks::invalid(
                        format!("identity {name}"),
                        format!("carried by both {at} and {context} over different bases"),
                    ));
                }
                Some(_) => {}
                None => {
                    bases.insert(name, (base, context));
                }
            }
        }
    }
    let mut out = Vec::with_capacity(declared.len());
    for (name, decl) in declared {
        let (base, _) = bases.get(name).ok_or_else(|| {
            crate::checks::invalid(
                format!("identity {name}"),
                "declared but carried by no column",
            )
        })?;
        out.push(IdentitySpec {
            id: super::registry_id(&format!("identity:{name}")),
            name,
            doc: decl.doc,
            base: *base,
            owner: owners.remove(name),
        });
    }
    Ok(out)
}

/// The column and every nested value inside it, each with its dotted field path.
fn carriers(column: &FieldContract) -> Vec<(String, FieldContract)> {
    fn visit(path: String, field: &FieldContract, out: &mut Vec<(String, FieldContract)>) {
        out.push((path.clone(), field.clone()));
        if field.extension().is_none() {
            for child in field.children() {
                visit(format!("{path}.{}", child.name()), &child, out);
            }
        }
    }
    let mut out = Vec::new();
    visit(column.name().to_owned(), column, &mut out);
    out
}

/// Propagate identities along column and table-level references to a fixed point.
fn inherit(relations: &mut [RelationSpec]) -> Result<(), SchemaError> {
    let index = super::index_relations(relations);
    loop {
        let mut inherited = Vec::new();
        for (position, spec) in relations.iter().enumerate() {
            let mut edges: Vec<(&str, &str, &str)> = spec
                .columns
                .iter()
                .filter_map(|column| column.fk().map(|fk| (column.name(), fk.relation, fk.column)))
                .collect();
            for reference in &spec.foreign_keys {
                for (local, remote) in reference.columns.iter().zip(&reference.target_columns) {
                    edges.push((local, reference.target, remote));
                }
            }
            for (local, relation, remote) in edges {
                let Some(identity) = index
                    .get(relation)
                    .and_then(|target| relations[*target].column(remote))
                    .and_then(FieldContract::identity)
                else {
                    continue;
                };
                let Some(column) = spec.column(local) else {
                    continue;
                };
                match column.identity() {
                    None => inherited.push((position, local.to_owned(), identity.to_owned())),
                    Some(own) if own == identity => {}
                    Some(own) => {
                        return Err(crate::checks::invalid(
                            format!("column {}.{local}", spec.key),
                            format!(
                                "conflicting identity: carries {own} but references {relation}.{remote}, which carries {identity}"
                            ),
                        ));
                    }
                }
            }
        }
        if inherited.is_empty() {
            return Ok(());
        }
        let mut seen = BTreeSet::new();
        for (position, local, identity) in inherited {
            if !seen.insert((position, local.clone())) {
                // Two references inherit into one column: the next pass checks agreement.
                continue;
            }
            if let Some(column) = relations[position]
                .columns
                .iter_mut()
                .find(|column| column.name() == local)
            {
                *column = column.clone().with_identity(&identity);
            }
        }
    }
}
