// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Occurrence paths resolve actual selected child and coordinate facts.
#![allow(clippy::unwrap_used, reason = "explicit declared fixture construction")]

use pse_ids::{CancellationToken, SemanticId};
use pse_relations::generated::{enums::SymbolRole, inferred as i, normalized as n};
use pse_schema::model::ExpressionPathSegmentKind as Segment;
use pse_templates::{
    identity::domain_product_id,
    paths::{Coordinate, PathInventory, PathRequest, ResolvedMember, resolve},
};

fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}

struct Fixture {
    instances: Vec<i::instances::Row>,
    prospective: Vec<n::instance_bindings::Row>,
    children: Vec<n::template_submodels::Row>,
    symbols: Vec<n::template_symbols::Row>,
    domains: Vec<n::instance_domain_bindings::Row>,
    members: Vec<n::domain_members::Row>,
    products: Vec<n::domain_products::Row>,
    tuples: Vec<i::valid_index_tuples::Row>,
}
impl Fixture {
    #[expect(
        clippy::too_many_lines,
        reason = "one fixture keeps its related path, axis and member declarations together"
    )]
    fn new() -> Self {
        let instances = [
            (1, None, 10, "plant"),
            (2, Some(1), 20, "plant.inlet"),
            (3, Some(1), 20, "plant.outlet"),
        ]
        .into_iter()
        .map(|(instance, parent, template, path)| i::instances::Row {
            instance_id: id(instance),
            parent_instance_id: parent.map(id),
            template_id: id(template),
            path: path.to_owned(),
            index: vec![],
            derivation_id: id(99),
        })
        .collect::<Vec<_>>();
        let prospective = instances
            .iter()
            .map(|row| n::instance_bindings::Row {
                instance_id: row.instance_id,
                parent_instance_id: row.parent_instance_id,
                template_id: row.template_id,
                submodel_template_id: row.parent_instance_id.map(|_| id(10)),
                submodel_name: row
                    .parent_instance_id
                    .map(|_| row.path.rsplit('.').next().unwrap().to_owned()),
                index: vec![],
                path: row.path.clone(),
                property_package_id: None,
                reaction_package_id: None,
                guard_source_id: None,
                guard_node_id: None,
                derivation_id: id(99),
            })
            .collect();
        let children = ["inlet", "outlet"]
            .into_iter()
            .map(|name| n::template_submodels::Row {
                template_id: id(10),
                name: name.to_owned(),
                child_template_id: Some(id(20)),
                child_from_param: None,
                multiplicity_domain: None,
                bindings: vec![],
                guard_id: None,
            })
            .collect();
        let symbols = vec![n::template_symbols::Row {
            template_id: id(20),
            symbol_decl_id: id(30),
            name: "flow".to_owned(),
            role: SymbolRole::Variable,
            quantity_type_id: id(40),
            indexed_by: vec!["j".to_owned()],
            default_lower: None,
            default_upper: None,
            default_initial: None,
            reference_to: None,
            wrt_domain: None,
            guard_id: None,
            idaes_name: None,
            doc: String::new(),
        }];
        let domains = [2, 3]
            .into_iter()
            .map(|instance| n::instance_domain_bindings::Row {
                instance_id: id(instance),
                domain_name: "j".to_owned(),
                domain_id: id(50),
            })
            .collect();
        let members = vec![n::domain_members::Row {
            domain_id: id(50),
            member_id: id(51),
            ordinal: 0,
            label: "member".to_owned(),
            coordinate: Some(42.0),
            ref_entity_id: None,
        }];
        let products = [vec![], vec![id(50)]]
            .into_iter()
            .map(|domain_ids| n::domain_products::Row {
                product_id: domain_product_id(&domain_ids),
                domain_ids,
            })
            .collect();
        let tuples = vec![
            i::valid_index_tuples::Row {
                product_id: domain_product_id(&[]),
                tuple: vec![],
                derivation_id: id(99),
            },
            i::valid_index_tuples::Row {
                product_id: domain_product_id(&[id(50)]),
                tuple: vec![id(51)],
                derivation_id: id(99),
            },
        ];
        Self {
            instances,
            prospective,
            children,
            symbols,
            domains,
            members,
            products,
            tuples,
        }
    }
    fn inventory(&self) -> PathInventory<'_> {
        PathInventory {
            instances: &self.instances,
            semantic_id_type: id(80),
            prospective: &self.prospective,
            submodels: &self.children,
            domains: &self.domains,
            members: &self.members,
            products: &self.products,
            valid_tuples: &self.tuples,
            symbols: &self.symbols,
            parameters: &[],
            features: &[],
            ports: &[],
            configuration: &[],
        }
    }
}

#[test]
fn same_template_siblings_keep_actual_owner_and_numeric_member() {
    let fixture = Fixture::new();
    for (name, owner) in [("inlet", 2), ("outlet", 3)] {
        let segments = [(Segment::Child, name, 0), (Segment::Member, "flow", 1)];
        let request = PathRequest {
            requester: id(1),
            root_instance: None,
            segments: &segments,
            coordinates: &[Coordinate::Integer(42)],
        };
        let target = resolve(fixture.inventory(), request, &CancellationToken::new()).unwrap();
        assert_eq!(target.owner, id(owner));
        assert_eq!(
            target.member,
            ResolvedMember::Symbol {
                declaration: id(30),
                index: vec![id(51)]
            }
        );
        assert_eq!(target.path_index, vec![id(51)]);
    }
}

#[test]
fn unchanged_id_cannot_hide_wrong_parent_or_missing_actual_tuple() {
    let mut fixture = Fixture::new();
    let segments = [(Segment::Child, "inlet", 0), (Segment::Member, "flow", 1)];
    let request = PathRequest {
        requester: id(1),
        root_instance: None,
        segments: &segments,
        coordinates: &[Coordinate::Integer(42)],
    };
    fixture.instances[1].parent_instance_id = Some(id(3));
    assert!(resolve(fixture.inventory(), request, &CancellationToken::new()).is_err());
    fixture.instances[1].parent_instance_id = Some(id(1));
    fixture.tuples.retain(|row| row.tuple.is_empty());
    assert!(resolve(fixture.inventory(), request, &CancellationToken::new()).is_err());
    let ordinal_request = PathRequest {
        coordinates: &[Coordinate::Integer(0)],
        ..request
    };
    assert!(
        resolve(
            fixture.inventory(),
            ordinal_request,
            &CancellationToken::new()
        )
        .is_err()
    );
}

#[test]
fn complete_enumeration_uses_actual_values_and_caller_resource_budget() {
    use pse_ids::{FixedBudget, MemoryReserver};
    let fixture = Fixture::new();
    let segments = [(Segment::Child, "inlet", 0), (Segment::Member, "flow", 1)];
    let budget = FixedBudget::new(64 * 1024);
    let mut work = budget.open("path fixture");
    let targets = pse_templates::paths::enumerate(
        fixture.inventory(),
        id(1),
        None,
        &segments,
        work.as_mut(),
        &CancellationToken::new(),
    )
    .unwrap();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].path_index, vec![id(51)]);
    assert!(work.size() > 0);
    let too_small = FixedBudget::new(1);
    let mut refused = too_small.open("refused path fixture");
    assert!(
        pse_templates::paths::enumerate(
            fixture.inventory(),
            id(1),
            None,
            &segments,
            refused.as_mut(),
            &CancellationToken::new()
        )
        .is_err()
    );
}
