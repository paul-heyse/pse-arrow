// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit source registration and finite parent closure shared by admission boundaries.
use pse_ids::SemanticId;
use pse_relations::generated::authored::entities;
use std::collections::{BTreeMap, BTreeSet};

/// Actual registration facts projected from a declared source relation.
#[derive(Clone, Debug)]
pub struct EntityRegistration {
    /// Relation owning this registration.
    pub relation: String,
    /// Actual declared identity.
    pub entity: SemanticId,
    /// Registered entity kind from the source section declaration.
    pub kind: String,
    /// Actual declared name, when this source carries one.
    pub name: Option<String>,
    /// Actual declaring package, when present directly.
    pub package: Option<SemanticId>,
    /// Actual declared containing identity, or an explicit root.
    pub parent: Option<SemanticId>,
}
/// The semantic obligation violated by a concrete identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityViolationKind {
    /// Two entity rows claim the same identity.
    DuplicateIdentity,
    /// Two entity rows claim the same qualified name.
    DuplicateName,
    /// A registration has no entity of its declared kind.
    Registration,
    /// Name, parent or declaring package differs from the actual source.
    Fields,
    /// A parent identity is not in the complete inventory.
    MissingParent,
    /// Parent edges return to a member of their own chain.
    ParentCycle,
}
/// Typed affected identity and its owning source relation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityViolation {
    /// Affected source identity.
    pub entity: SemanticId,
    /// Owning source relation.
    pub relation: String,
    /// Concrete violated obligation.
    pub kind: EntityViolationKind,
}
/// Compare explicit source registration facts and admit the complete finite parent inventory.
/// No prior boundary validation or global registry/session installation is assumed.
/// # Errors
/// Actual identities violate registration, reference, field agreement or acyclicity.
pub fn admit_entities(
    rows: &[entities::Row],
    registrations: &[EntityRegistration],
) -> Result<(), crate::RuleError> {
    let mut entities = BTreeMap::new();
    let mut names = BTreeSet::new();
    let mut violations = Vec::new();
    let mut add = |entity, relation: &str, kind| {
        violations.push(EntityViolation {
            entity,
            relation: relation.into(),
            kind,
        })
    };
    for row in rows {
        if entities.insert(row.entity_id, row).is_some() {
            add(
                row.entity_id,
                "authored.entities",
                EntityViolationKind::DuplicateIdentity,
            );
        }
        if !names.insert(&row.qualified_name) {
            add(
                row.entity_id,
                "authored.entities",
                EntityViolationKind::DuplicateName,
            );
        }
    }
    for registration in registrations {
        let Some(entity) = entities
            .get(&registration.entity)
            .filter(|entity| entity.kind.as_str() == registration.kind)
        else {
            add(
                registration.entity,
                &registration.relation,
                EntityViolationKind::Registration,
            );
            continue;
        };
        let package = registration.package.or_else(|| {
            registration.parent.and_then(|parent| {
                entities
                    .get(&parent)
                    .map(|entity| entity.package_id.as_id())
            })
        });
        if registration
            .name
            .as_ref()
            .is_some_and(|name| *name != entity.name)
            || registration.parent != entity.parent_entity_id
            || package.is_some_and(|package| package != entity.package_id.as_id())
        {
            add(
                registration.entity,
                &registration.relation,
                EntityViolationKind::Fields,
            );
        }
    }
    let mut complete = BTreeSet::new();
    for id in entities.keys() {
        if complete.contains(id) {
            continue;
        }
        let mut chain = Vec::new();
        let mut positions = BTreeMap::new();
        let mut current = Some(*id);
        while let Some(id) = current {
            if complete.contains(&id) {
                break;
            }
            if let Some(position) = positions.get(&id).copied() {
                for id in &chain[position..] {
                    add(*id, "authored.entities", EntityViolationKind::ParentCycle);
                }
                break;
            }
            let Some(entity) = entities.get(&id) else {
                if let Some(source) = chain.last() {
                    add(
                        *source,
                        "authored.entities",
                        EntityViolationKind::MissingParent,
                    );
                }
                break;
            };
            positions.insert(id, chain.len());
            chain.push(id);
            current = entity.parent_entity_id;
        }
        complete.extend(chain);
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(crate::RuleError::References { violations })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entity(n: u8, parent: Option<u8>) -> entities::Row {
        entities::Row {
            entity_id: SemanticId::from_bytes([n; 16]),
            package_id: pse_relations::generated::identities::PackageId::from_bytes([9; 16]),
            kind: "unit".parse().unwrap(),
            name: format!("n{n}"),
            qualified_name: format!("p.n{n}"),
            parent_entity_id: parent.map(|n| SemanticId::from_bytes([n; 16])),
            source_span: None,
        }
    }
    #[test]
    fn explicit_registration_and_parent_closure_refuse_affected_identities() {
        let rows = [entity(1, None), entity(2, Some(1))];
        let registration = EntityRegistration {
            relation: "authored.units".into(),
            entity: rows[1].entity_id,
            kind: rows[1].kind.as_str().into(),
            name: Some(rows[1].name.clone()),
            package: None,
            parent: rows[1].parent_entity_id,
        };
        admit_entities(&rows, std::slice::from_ref(&registration)).unwrap();
        let mut changed = registration.clone();
        changed.name = Some("different".into());
        let crate::RuleError::References { violations } =
            admit_entities(&rows, &[changed]).unwrap_err()
        else {
            panic!("typed violations required")
        };
        assert_eq!(
            violations,
            [EntityViolation {
                entity: rows[1].entity_id,
                relation: registration.relation,
                kind: EntityViolationKind::Fields
            }]
        );
        for (rows, kind) in [
            (vec![entity(1, Some(2))], EntityViolationKind::MissingParent),
            (
                vec![entity(1, Some(2)), entity(2, Some(1))],
                EntityViolationKind::ParentCycle,
            ),
        ] {
            let crate::RuleError::References { violations } =
                admit_entities(&rows, &[]).unwrap_err()
            else {
                panic!("typed violations required")
            };
            assert!(violations.iter().all(|violation| violation.kind == kind));
        }
    }
}
