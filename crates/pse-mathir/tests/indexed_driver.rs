// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual domain members, equation residuals and kernel contracts in the P10 driver.
use pse_ids::SemanticId;
use pse_mathir::canonicalize::{CanonicalizeInput, Policy, canonicalize};
use pse_mathir::equation::{EquationRecord, FreeIndex, Sense};
use pse_mathir::index::DomainFacts;
use pse_mathir::infer::{GroupFacts, KernelContract, KernelPort, SymbolTypeSource};
use pse_mathir::relations::vec_sink::KernelBinding;
use pse_mathir::{ExprGraph, NodeId, Opcode, Payload};
use pse_quantity::{BoundIndexId, DomainId, DomainKind, QuantityTypeId, UnitId};
use std::collections::BTreeMap;
fn sid(byte: u8) -> SemanticId {
    SemanticId::from_bytes([byte; 16])
}
fn qid(byte: u8) -> QuantityTypeId {
    QuantityTypeId::from_id(sid(byte))
}
fn domain() -> DomainId {
    DomainId::from_id(sid(9))
}
fn binder() -> BoundIndexId {
    BoundIndexId::from_id(sid(10))
}
#[derive(Default)]
struct Source {
    symbols: BTreeMap<SemanticId, QuantityTypeId>,
    domains: BTreeMap<DomainId, DomainFacts>,
    groups: BTreeMap<SemanticId, GroupFacts>,
    kernels: BTreeMap<SemanticId, KernelContract>,
}
impl SymbolTypeSource for Source {
    fn symbol_type(&self, symbol: SemanticId) -> Option<QuantityTypeId> {
        self.symbols.get(&symbol).copied()
    }
    fn domain(&self, domain: DomainId) -> Option<&DomainFacts> {
        self.domains.get(&domain)
    }
    fn group(&self, group: SemanticId) -> Option<&GroupFacts> {
        self.groups.get(&group)
    }
    fn kernel_contract(&self, kernel: SemanticId) -> Option<&KernelContract> {
        self.kernels.get(&kernel)
    }
}
#[expect(
    clippy::unwrap_used,
    reason = "fixture helper asserts registry admission outside individual test bodies"
)]
fn indexed_fixture() -> (pse_quantity::QuantityRegistry, Source) {
    use pse_quantity::{
        DimensionVector, QuantityAdditionKind, QuantityKind, QuantityRegistryBuilder, QuantityType,
        QuantityTypeKey, ScaleKind, Unit,
    };
    let unit = UnitId::from_id(sid(1));
    let kind = pse_quantity::QuantityKindId::from_id(sid(2));
    let mut builder = QuantityRegistryBuilder::new();
    builder.unit(Unit {
        id: unit,
        symbol: "1".into(),
        dimension: DimensionVector::DIMENSIONLESS,
        scale_to_canonical: 1.,
        offset_to_canonical: 0.,
        is_affine: false,
        reference_state: None,
    });
    builder.kind(QuantityKind {
        id: kind,
        extensive: false,
        dimension: DimensionVector::DIMENSIONLESS,
        addition_kind: QuantityAdditionKind::Additive,
    });
    for (id, shape) in [(qid(3), vec![]), (qid(4), vec![DomainKind::Species])] {
        builder.quantity_type(QuantityType {
            id,
            key: QuantityTypeKey {
                kind,
                basis: None,
                reference_state: None,
                scale_kind: ScaleKind::Point,
                shape,
                subject_kind: None,
            },
            canonical_unit: unit,
            nominal_magnitude: None,
        });
    }
    builder.neutral_dimensionless(qid(3));
    let source = Source {
        symbols: BTreeMap::from([(sid(21), qid(3)), (sid(22), qid(3))]),
        domains: BTreeMap::from([(
            domain(),
            DomainFacts {
                kind: DomainKind::Species,
                continuous: false,
                unit: None,
                members: vec![sid(11), sid(12)],
            },
        )]),
        groups: BTreeMap::from([(
            sid(8),
            GroupFacts {
                quantity_type: qid(4),
                domains: vec![domain()],
                valid_tuples: vec![vec![sid(11)], vec![sid(12)]],
                members: BTreeMap::from([(vec![sid(11)], sid(21)), (vec![sid(12)], sid(22))]),
            },
        )]),
        ..Source::default()
    };
    (builder.build().unwrap(), source)
}
#[expect(
    clippy::unwrap_used,
    reason = "fixture helper asserts graph construction outside individual test bodies"
)]
fn gather(graph: &mut ExprGraph) -> NodeId {
    graph
        .insert(
            Opcode::Gather,
            Payload::Gather {
                group: sid(8),
                coordinate_map: vec![(binder(), 0)],
            },
            &[],
            None,
        )
        .unwrap()
}
#[test]
fn explicit_broadcast_transfers_the_scalar_contract_and_retains_the_actual_axis() {
    use pse_mathir::canonicalize::{RootEnvironment, canonicalize_with_environments};
    let (original, source) = indexed_fixture();
    let mut builder = original.to_builder();
    let mut alternative = original.quantity_type(qid(3)).unwrap().clone();
    alternative.id = qid(5);
    let mut other_kind = original.kind(alternative.key.kind).unwrap().clone();
    other_kind.id = pse_quantity::QuantityKindId::from_id(sid(6));
    alternative.key.kind = other_kind.id;
    builder.kind(other_kind);
    builder.quantity_type(alternative);
    let registry = builder.build().unwrap();
    let mut graph = ExprGraph::new();
    let literal = graph.float_const(0., UnitId::from_id(sid(1))).unwrap();
    let broadcast = graph
        .insert(
            Opcode::Broadcast,
            Payload::Broadcast {
                domain: domain().into(),
                bound_index: binder(),
            },
            &[literal],
            None,
        )
        .unwrap();
    let indices = pse_quantity::IndexSet::try_from_iter([pse_quantity::BoundIndexRef::new(
        binder(),
        domain(),
        DomainKind::Species,
    )])
    .unwrap();
    let environment = RootEnvironment {
        indices,
        expected: Some(qid(4)),
    };
    let result = canonicalize_with_environments(
        CanonicalizeInput::new(&graph, &[broadcast], &source, &registry),
        std::slice::from_ref(&environment),
        Policy::Strict,
    )
    .unwrap();
    let root = result.node(result.roots()[0]).unwrap();
    assert_eq!(root.opcode, Opcode::Broadcast);
    assert_eq!(root.quantity_type, qid(4));
    assert_eq!(root.free_indices, environment.indices);
    assert_eq!(result.node(root.children[0]).unwrap().quantity_type, qid(3));
    // No broadcast is inferred for the same raw literal in an indexed consumer.
    assert!(
        canonicalize_with_environments(
            CanonicalizeInput::new(&graph, &[literal], &source, &registry),
            &[environment],
            Policy::Strict,
        )
        .is_err()
    );
    let gathered = gather(&mut graph);
    let difference = graph.sub(gathered, broadcast).unwrap();
    let sum = graph
        .insert(
            Opcode::SumOver,
            Payload::Reduction {
                kind: pse_quantity::ReductionKind::Sum,
                domain: domain().into(),
                bound_index: binder(),
                filter: None,
            },
            &[difference],
            None,
        )
        .unwrap();
    let result = canonicalize(
        CanonicalizeInput::new(&graph, &[sum], &source, &registry),
        Policy::Strict,
    )
    .unwrap();
    assert_eq!(
        result.node(result.roots()[0]).unwrap().quantity_type,
        qid(3)
    );
}

#[test]
fn reduction_binds_gather_using_actual_member_identities() {
    let (registry, mut source) = indexed_fixture();
    let mut graph = ExprGraph::new();
    let read = gather(&mut graph);
    assert!(
        canonicalize(
            CanonicalizeInput::new(&graph, &[read], &source, &registry),
            Policy::Strict
        )
        .is_err()
    );
    let sum = graph
        .insert(
            Opcode::SumOver,
            Payload::Reduction {
                kind: pse_quantity::ReductionKind::Sum,
                domain: domain().into(),
                bound_index: binder(),
                filter: None,
            },
            &[read],
            None,
        )
        .unwrap();
    let result = canonicalize(
        CanonicalizeInput::new(&graph, &[sum], &source, &registry),
        Policy::Strict,
    )
    .unwrap();
    assert_eq!(
        result.node(result.roots()[0]).unwrap().quantity_type,
        qid(3)
    );
    let gathered = result
        .iter()
        .find(|(_, node)| node.opcode == Opcode::Gather)
        .unwrap()
        .1;
    assert_eq!(
        gathered.free_indices.iter().next().unwrap().bound_index,
        binder()
    );
    source.groups.get_mut(&sid(8)).unwrap().valid_tuples[0][0] = sid(99);
    assert!(
        canonicalize(
            CanonicalizeInput::new(&graph, &[sum], &source, &registry),
            Policy::Strict
        )
        .is_err()
    );
}
#[test]
fn indexed_equation_binds_axes_and_derives_residual_before_persistence() {
    let (registry, source) = indexed_fixture();
    let mut graph = ExprGraph::new();
    let body = gather(&mut graph);
    let zero = graph.int_const(0).unwrap();
    let upper = graph
        .insert(
            Opcode::Broadcast,
            Payload::Broadcast {
                domain: domain().into(),
                bound_index: binder(),
            },
            &[zero],
            None,
        )
        .unwrap();
    let equations = [EquationRecord {
        indexed_equation_id: sid(30),
        owner_instance: sid(31),
        equation_decl: None,
        qualified_name: "indexed.balance".into(),
        product: Some(sid(32)),
        filter: None,
        body,
        sense: Sense::Eq,
        lower: None,
        upper: Some(upper),
        free_indices: vec![FreeIndex {
            bound_index: binder(),
            domain: domain(),
            position: 0,
        }],
        residual_quantity_type: None,
        law_instance: None,
        derivation: sid(33),
    }];
    let mut input = CanonicalizeInput::new(&graph, &[], &source, &registry);
    input.equations = &equations;
    let result = canonicalize(input, Policy::Strict).unwrap();
    assert_eq!(result.equations()[0].residual_quantity_type, Some(qid(4)));
    assert_eq!(
        result.equations()[0].free_indices,
        equations[0].free_indices
    );
    assert_eq!(
        result.node(result.equations()[0].body).unwrap().opcode,
        Opcode::Gather
    );
    let mut rows = pse_mathir::relations::VecSink::new();
    pse_mathir::relations::emit(&result, &mut rows).unwrap();
    let loaded = pse_mathir::relations::load_untyped(&rows, result.roots()).unwrap();
    assert_eq!(loaded.equations[0].residual_quantity_type, Some(qid(4)));
    assert_eq!(loaded.equations[0].derivation, sid(33));
}
#[test]
fn kernel_inputs_and_outputs_have_visible_natural_unit_edges() {
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let temperature = pse_quantity::standard::ids::quantity("temperature.point");
    let kelvin = pse_quantity::standard::ids::unit("K");
    let celsius = pse_quantity::standard::ids::unit("degC");
    let port = |name: &str| KernelPort {
        name: name.into(),
        quantity_type: temperature,
        unit: celsius,
        shape: vec![],
    };
    let source = Source {
        symbols: BTreeMap::from([(sid(1), temperature)]),
        kernels: BTreeMap::from([(
            sid(2),
            KernelContract {
                inputs: vec![port("T")],
                outputs: vec![port("result")],
                parameters: vec![],
            },
        )]),
        ..Source::default()
    };
    let mut graph = ExprGraph::new();
    let symbol = graph.symbol(sid(1)).unwrap();
    let call = graph
        .insert(
            Opcode::KernelCall,
            Payload::KernelCall {
                kernel_binding: sid(3),
                output_ordinal: 0,
            },
            &[],
            None,
        )
        .unwrap();
    let bindings = BTreeMap::from([(
        sid(3),
        KernelBinding {
            binding: sid(3),
            kernel: sid(2),
            scope: sid(4),
            parameters: vec![],
            inputs: vec![("T".into(), symbol)],
        },
    )]);
    let roots = [call];
    let mut input = CanonicalizeInput::new(&graph, &roots, &source, &registry);
    input.kernel_bindings = &bindings;
    let result = canonicalize(input, Policy::Strict).unwrap();
    let edge = result.node(result.roots()[0]).unwrap();
    assert!(
        matches!(edge.payload,Payload::UnitConvert(spec) if spec.from==celsius && spec.to==kelvin && spec.offset.to_bits()==273.15f64.to_bits())
    );
    let bound_input = result.kernel_bindings()[&sid(3)].inputs[0].1;
    assert!(
        matches!(result.node(bound_input).unwrap().payload,Payload::UnitConvert(spec) if spec.from==kelvin && spec.to==celsius && spec.offset.to_bits() == (-273.15f64).to_bits())
    );
    let mut wrong = bindings.clone();
    wrong.get_mut(&sid(3)).unwrap().inputs[0].0 = "wrong".into();
    input.kernel_bindings = &wrong;
    assert!(canonicalize(input, Policy::Strict).is_err());
}

#[test]
fn unresolved_template_domain_is_refused_before_any_quantity_or_index_facts() {
    let (registry, source) = indexed_fixture();
    let mut graph = ExprGraph::new();
    let body = graph.int_const(1).unwrap();
    let unresolved = graph
        .insert(
            Opcode::Broadcast,
            Payload::Broadcast {
                domain: pse_mathir::DomainRef::Template {
                    template_id: sid(9),
                    domain_name: "species".into(),
                },
                bound_index: binder(),
            },
            &[body],
            None,
        )
        .unwrap();
    // Even though an actual domain shares these bytes, the explicit alternative cannot alias it.
    // The preflight includes unreachable rows rather than checking only the requested root.
    for root in [body, unresolved] {
        assert!(
            matches!(canonicalize(CanonicalizeInput::new(&graph, &[root], &source, &registry), Policy::Strict),
            Err(pse_mathir::MathIrError::UnresolvedDomain { template_id, domain_name, .. })
                if template_id == sid(9) && domain_name == "species")
        );
    }
}

#[test]
fn normalized_values_and_predicates_require_explicit_lowering_before_p10() {
    use pse_mathir::{GuardRef, TemplateValueKind, ValueRef};
    let (registry, source) = indexed_fixture();
    let mut graph = ExprGraph::new();
    let reference = graph
        .insert(
            Opcode::SymbolRef,
            Payload::SymbolRef {
                symbol: ValueRef::Template {
                    template_id: sid(21),
                    kind: TemplateValueKind::Parameter,
                    name: "x".into(),
                },
            },
            &[],
            None,
        )
        .unwrap();
    assert!(matches!(
        canonicalize(
            CanonicalizeInput::new(&graph, &[reference], &source, &registry),
            Policy::Strict
        ),
        Err(pse_mathir::MathIrError::UnresolvedValue {
            kind: "parameter",
            ..
        })
    ));
    let mut graph = ExprGraph::new();
    let body = graph.int_const(1).unwrap();
    let guarded = graph
        .insert(
            Opcode::Conditional,
            Payload::Conditional {
                guard: GuardRef::Predicate {
                    source_id: sid(21),
                    predicate_id: body.0,
                },
            },
            &[body, body],
            None,
        )
        .unwrap();
    assert!(matches!(
        canonicalize(
            CanonicalizeInput::new(&graph, &[guarded], &source, &registry),
            Policy::Strict
        ),
        Err(pse_mathir::MathIrError::UnresolvedValue {
            kind: "predicate",
            ..
        })
    ));
}

#[expect(
    clippy::unwrap_used,
    reason = "fixture helper asserts exact registered Integral inputs"
)]
fn integral_fixture() -> (pse_quantity::QuantityRegistry, Source) {
    use pse_quantity::{
        BasisRule, OperationId, QuantityOperation, QuantityScaleRule, QuantityShapeRule,
        ReferenceRule, SubjectRule,
    };
    let (original, mut source) = indexed_fixture();
    let mut builder = original.to_builder();
    let mut body_type = original.quantity_type(qid(4)).unwrap().clone();
    body_type.id = qid(5);
    body_type.key.shape = vec![DomainKind::Custom];
    let kind = body_type.key.kind;
    builder.quantity_type(body_type);
    let integral_rule = QuantityOperation {
        id: OperationId::from_id(sid(70)),
        opcode: Opcode::Integral,
        input_kinds: vec![kind],
        result_kind: kind,
        basis_rule: BasisRule::RequireEqual,
        reference_rule: ReferenceRule::RequireEqual,
        scale_rule: QuantityScaleRule::Point,
        shape_rule: QuantityShapeRule::ReduceBoundIndex,
        basis_source: None,
        reference_source: None,
        scale_source: None,
        shape_source: None,
        subject_rule: SubjectRule::RequireEqual,
        subject_source: None,
        result_subject_kind: None,
        result_basis: None,
        result_reference_state: None,
        input_conversions: vec![],
        precondition_invariants: vec![],
    };
    let mut log_rule = integral_rule.clone();
    log_rule.id = OperationId::from_id(sid(71));
    log_rule.opcode = Opcode::Log;
    log_rule.shape_rule = QuantityShapeRule::SameIndices;
    builder.operation(integral_rule).operation(log_rule);
    let registry = builder.build().unwrap();
    source.groups.get_mut(&sid(8)).unwrap().quantity_type = qid(5);
    let facts = source.domains.get_mut(&domain()).unwrap();
    facts.kind = DomainKind::Custom;
    facts.continuous = true;
    facts.unit = Some(UnitId::from_id(sid(1)));
    (registry, source)
}

#[test]
fn integral_preserves_its_owning_binder_and_resolves_registered_reduction_type() {
    let (registry, mut source) = integral_fixture();
    let mut graph = ExprGraph::new();
    let body = gather(&mut graph);
    let integral = graph
        .insert(
            Opcode::Integral,
            Payload::Integral {
                domain: domain().into(),
                bound_index: binder(),
                quadrature_policy: None,
                filter: None,
            },
            &[body],
            None,
        )
        .unwrap();
    let result = canonicalize(
        CanonicalizeInput::new(&graph, &[integral], &source, &registry),
        Policy::Strict,
    )
    .unwrap();
    let node = result.node(result.roots()[0]).unwrap();
    assert_eq!(node.quantity_type, qid(3));
    assert!(node.free_indices.is_empty());
    assert!(
        matches!(node.payload, Payload::Integral { bound_index, .. } if bound_index == binder())
    );
    source.domains.get_mut(&domain()).unwrap().continuous = false;
    assert!(
        canonicalize(
            CanonicalizeInput::new(&graph, &[integral], &source, &registry),
            Policy::Strict
        )
        .is_err()
    );
}

#[test]
fn excluded_integral_body_defers_domain_failure_but_direct_execution_still_refuses() {
    let (registry, source) = integral_fixture();
    let mut graph = ExprGraph::new();
    let zero = graph.int_const(0).unwrap();
    let bad = graph
        .insert(Opcode::Log, Payload::None, &[zero], None)
        .unwrap();
    let body = graph
        .insert(
            Opcode::Broadcast,
            Payload::Broadcast {
                domain: domain().into(),
                bound_index: binder(),
            },
            &[bad],
            None,
        )
        .unwrap();
    let integral = graph
        .insert(
            Opcode::Integral,
            Payload::Integral {
                domain: domain().into(),
                bound_index: binder(),
                quadrature_policy: None,
                filter: Some(zero.into()),
            },
            &[body],
            None,
        )
        .unwrap();
    let result = canonicalize(
        CanonicalizeInput::new(&graph, &[integral], &source, &registry),
        Policy::Strict,
    )
    .unwrap();
    let logarithm = result
        .iter()
        .find(|(_, node)| node.opcode == Opcode::Log)
        .unwrap()
        .0;
    assert!(
        result
            .selections()
            .iter()
            .any(|selection| selection.node == logarithm && selection.deferred_static_check)
    );
    assert!(matches!(
        canonicalize(
            CanonicalizeInput::new(&graph, &[integral, bad], &source, &registry),
            Policy::Strict
        ),
        Err(pse_mathir::MathIrError::StaticDomain {
            opcode: Opcode::Log,
            ..
        })
    ));
}
