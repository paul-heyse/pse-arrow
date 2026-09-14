// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P3 preserves complete syntax and checks actual source and unit facts.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "fixture assertions"
)]

use pse_authoring::{
    ParseBudget,
    document::{DocumentBundle, load_package_texts},
};
use pse_compiler::passes::p3::normalize;
use pse_ids::{CancellationToken, FixedBudget, SemanticId};
use pse_relations::RecordBatch;
use pse_schema::{
    Registry,
    model::{Cell, RelationKey},
};
use std::{collections::BTreeMap, fmt::Write};

fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
fn bundle(expression: &str) -> DocumentBundle {
    let header = format!(
        "[package]\nid='{}'\nname='example'\nversion='1.0.0'\nkind='model'\nid_policy='explicit'\ndependencies=[]\ndoc=''\n",
        id(90)
    );
    let template = format!(
        "templates:\n  - id: '{}'\n    name: model\n    version: '1.0.0'\n    kind: unit\n    doc: ''\ntemplate_symbols:\n  - id: '{}'\n    template_id: '{}'\n    name: x\n    role: variable\n    quantity_type_id: '{}'\n    indexed_by: []\n    doc: ''\ntemplate_equations:\n  - id: '{}'\n    template_id: '{}'\n    name: equation\n    indexed_by: []\n    expression: '{expression}'\n    sense: eq\n    doc: ''\n",
        id(91),
        id(92),
        id(91),
        id(93),
        id(94),
        id(91)
    );
    load_package_texts(
        BTreeMap::from([
            ("package.toml".to_owned(), header),
            ("templates/model.yaml".to_owned(), template),
        ]),
        pse_schema::registry().unwrap(),
        ParseBudget::default(),
    )
    .unwrap()
}
fn inputs(bundle: &DocumentBundle, registry: &Registry) -> BTreeMap<RelationKey, RecordBatch> {
    registry
        .relations()
        .iter()
        .filter(|spec| {
            spec.key.namespace == pse_schema::model::Namespace::Authored
                || matches!(spec.key.name, "units" | "unit_sets" | "reference_states")
        })
        .map(|spec| {
            (
                spec.key,
                pse_relations::cells::batch_from_cells(
                    registry,
                    spec,
                    bundle.rows.get(&spec.id).map_or(&[], Vec::as_slice),
                )
                .unwrap(),
            )
        })
        .collect()
}
fn cells(rows: &BTreeMap<RelationKey, RecordBatch>, name: &str) -> Vec<Vec<Cell>> {
    let registry = pse_schema::registry().unwrap();
    let spec = registry.relation(name).unwrap();
    pse_relations::cells::cells_from_batch(registry, spec, &rows[&spec.key]).unwrap()
}
fn run(
    bundle: &DocumentBundle,
    rows: &BTreeMap<RelationKey, RecordBatch>,
) -> BTreeMap<RelationKey, RecordBatch> {
    normalize(
        rows,
        std::slice::from_ref(bundle),
        pse_schema::registry().unwrap(),
        FixedBudget::new(512 * 1024 * 1024).as_ref(),
        &CancellationToken::new(),
    )
    .unwrap()
}
#[test]
fn conditional_equations_keep_senses_predicates_branches_and_source_keys() {
    let registry = pse_schema::registry().unwrap();
    let bundle = bundle("if x > 0 then x <= 3 else x >= -2");
    let output = run(&bundle, &inputs(&bundle, registry));
    let sources = cells(&output, "normalized.expression_sources");
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0][6], Cell::Enum("equation"));
    let Cell::List(key) = &sources[0][3] else {
        panic!("typed key absent")
    };
    assert_eq!(key.len(), 1);
    let equations = cells(&output, "normalized.equation_nodes");
    assert_eq!(equations.len(), 3);
    assert!(equations.iter().any(|row| row[3] == Cell::Enum("le")));
    assert!(equations.iter().any(|row| row[3] == Cell::Enum("ge")));
    assert!(
        equations
            .iter()
            .any(|row| row[2] == Cell::Enum("conditional")
                && row[6] != Cell::Null
                && row[7] != row[8])
    );
    let predicates = cells(&output, "normalized.predicate_nodes");
    assert_eq!(predicates.len(), 1);
    assert_eq!(predicates[0][4], Cell::Enum("gt"));
}
#[test]
fn edited_source_bytes_cannot_reuse_stale_parsed_rows_or_metadata() {
    let registry = pse_schema::registry().unwrap();
    let mut bundle = bundle("x == 3");
    let rows = inputs(&bundle, registry);
    let document = bundle
        .documents
        .iter_mut()
        .find(|document| document.path.starts_with("templates/"))
        .unwrap();
    document.text = document.text.replace("x == 3", "x == 4");
    assert!(
        normalize(
            &rows,
            &[bundle],
            registry,
            FixedBudget::new(512 * 1024 * 1024).as_ref(),
            &CancellationToken::new()
        )
        .is_err()
    );
}
#[test]
fn lexical_values_are_lowered_by_binding_and_actual_row_mutations_are_refused() {
    let registry = pse_schema::registry().unwrap();
    let bundle = bundle("x == (x + 1 where x = 3)");
    let mut rows = inputs(&bundle, registry);
    let output = run(&bundle, &rows);
    assert_eq!(
        cells(&output, "normalized.template_expr_symbol_refs").len(),
        1
    );
    let spec = registry.relation("authored.template_equations").unwrap();
    let mut changed = cells(&rows, "authored.template_equations");
    let position = spec
        .columns
        .iter()
        .position(|column| column.name == "expression")
        .unwrap();
    changed[0][position] = Cell::text("x == 99");
    rows.insert(
        spec.key,
        pse_relations::cells::batch_from_cells(registry, spec, &changed).unwrap(),
    );
    assert!(
        normalize(
            &rows,
            std::slice::from_ref(&bundle),
            registry,
            FixedBudget::new(512 * 1024 * 1024).as_ref(),
            &CancellationToken::new()
        )
        .is_err()
    );
}
#[test]
fn cancellation_and_small_budget_return_no_successful_output() {
    let registry = pse_schema::registry().unwrap();
    let bundle = bundle("x == 1");
    let rows = inputs(&bundle, registry);
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert!(
        normalize(
            &rows,
            std::slice::from_ref(&bundle),
            registry,
            FixedBudget::new(512 * 1024 * 1024).as_ref(),
            &cancel
        )
        .is_err()
    );
    let budget = FixedBudget::new(1);
    assert!(
        normalize(
            &rows,
            &[bundle],
            registry,
            budget.as_ref(),
            &CancellationToken::new()
        )
        .is_err()
    );
    assert_eq!(budget.reserved(), 0);
}

fn indexed_bundle(expression: &str, equation_indices: bool) -> DocumentBundle {
    let initial = bundle(expression);
    let mut texts = initial
        .documents
        .iter()
        .map(|document| (document.path.clone(), document.text.clone()))
        .collect::<BTreeMap<_, _>>();
    let text = texts.get_mut("templates/model.yaml").unwrap();
    *text = text.replacen("indexed_by: []", "indexed_by: [i, j]", 1);
    if equation_indices {
        *text = text.replace("indexed_by: []", "indexed_by: [i, j]");
    }
    write!(text, "template_domains:\n  - template_id: '{}'\n    name: i\n    kind: custom\n    continuous: false\n  - template_id: '{}'\n    name: j\n    kind: custom\n    continuous: false\n", id(91), id(91)).unwrap();
    load_package_texts(
        texts,
        pse_schema::registry().unwrap(),
        ParseBudget::default(),
    )
    .unwrap()
}

#[test]
fn pending_indices_preserve_order_and_do_not_invent_lexical_bindings() {
    let registry = pse_schema::registry().unwrap();
    let bundle = indexed_bundle("x[1, 2] == x[2, 1]", false);
    let output = run(&bundle, &inputs(&bundle, registry));
    let gathers = cells(&output, "normalized.template_expr_gathers");
    assert_eq!(gathers.len(), 2);
    assert!(
        gathers
            .iter()
            .all(|row| row[2] == Cell::Null && row[3] == Cell::Enum("pending"))
    );
    let (Cell::List(first), Cell::List(second)) = (&gathers[0][4], &gathers[1][4]) else {
        panic!("ordered index nodes absent");
    };
    assert_eq!(
        first.iter().rev().collect::<Vec<_>>(),
        second.iter().collect::<Vec<_>>()
    );
    assert!(cells(&output, "normalized.expression_index_bindings").is_empty());
    let constants = cells(&output, "normalized.template_expr_int_constants");
    assert!(constants.iter().any(|row| row[1] == Cell::I64(1)));
    assert!(constants.iter().any(|row| row[1] == Cell::I64(2)));
}

#[test]
fn pending_compound_indices_retain_actual_binders_and_refuse_wrong_axis_or_arity() {
    let registry = pse_schema::registry().unwrap();
    let bundle = indexed_bundle("x[i + 1, j] == 2", true);
    let output = run(&bundle, &inputs(&bundle, registry));
    assert_eq!(
        cells(&output, "normalized.expression_index_bindings").len(),
        2
    );
    assert_eq!(
        cells(&output, "normalized.template_expr_gathers")[0][3],
        Cell::Enum("pending")
    );
    for expression in ["x[j + 1, i] == 2", "x[1] == 2"] {
        let bundle = indexed_bundle(expression, true);
        assert!(
            normalize(
                &inputs(&bundle, registry),
                &[bundle],
                registry,
                FixedBudget::new(512 << 20).as_ref(),
                &CancellationToken::new()
            )
            .is_err()
        );
    }
}

fn dimension(value: pse_quantity::DimensionVector) -> Cell {
    Cell::List(
        value
            .exponents()
            .iter()
            .map(|ratio| {
                Cell::Struct(vec![
                    Cell::I64(i64::from(ratio.num())),
                    Cell::I64(i64::from(ratio.den())),
                ])
            })
            .collect(),
    )
}
fn with_units(expression: &str) -> (DocumentBundle, BTreeMap<RelationKey, RecordBatch>) {
    let registry = pse_schema::registry().unwrap();
    let initial = bundle(expression);
    let mut texts = initial
        .documents
        .iter()
        .map(|document| (document.path.clone(), document.text.clone()))
        .collect::<BTreeMap<_, _>>();
    write!(
        texts.get_mut("package.toml").unwrap(),
        "\n[[unit_sets]]\nunit_set_id='{}'\n",
        id(9)
    )
    .unwrap();
    let bundle = load_package_texts(texts, registry, ParseBudget::default()).unwrap();
    let mut rows = inputs(&bundle, registry);
    let mut units = pse_quantity::BaseDimension::ALL
        .iter()
        .map(|axis| {
            vec![
                Cell::Id(id(axis.ordinal() + 1)),
                Cell::text(axis.as_str()),
                Cell::text(axis.as_str()),
                dimension(pse_quantity::DimensionVector::base(*axis)),
                Cell::F64(1.0),
                Cell::F64(0.0),
                Cell::Bool(false),
                Cell::Null,
                Cell::text("SI"),
                Cell::text("base"),
            ]
        })
        .collect::<Vec<_>>();
    units.push(vec![
        Cell::Id(id(20)),
        Cell::text("cm"),
        Cell::text("centimetre"),
        dimension(pse_quantity::DimensionVector::base(
            pse_quantity::BaseDimension::Length,
        )),
        Cell::F64(0.01),
        Cell::F64(0.0),
        Cell::Bool(false),
        Cell::Null,
        Cell::text("SI"),
        Cell::text("scaled length"),
    ]);
    units.push(vec![
        Cell::Id(id(21)),
        Cell::text("degC"),
        Cell::text("Celsius"),
        dimension(pse_quantity::DimensionVector::base(
            pse_quantity::BaseDimension::Temperature,
        )),
        Cell::F64(1.0),
        Cell::F64(273.15),
        Cell::Bool(true),
        Cell::Null,
        Cell::text("SI"),
        Cell::text("affine temperature"),
    ]);
    let spec = registry.relation("reference.units").unwrap();
    rows.insert(
        spec.key,
        pse_relations::cells::batch_from_cells(registry, spec, &units).unwrap(),
    );
    let spec = registry.relation("reference.unit_sets").unwrap();
    rows.insert(
        spec.key,
        pse_relations::cells::batch_from_cells(
            registry,
            spec,
            &[vec![
                Cell::Id(id(9)),
                Cell::text("base"),
                Cell::Id(id(3)),
                Cell::Id(id(1)),
                Cell::Id(id(2)),
                Cell::Id(id(5)),
                Cell::Id(id(4)),
                Cell::Id(id(6)),
                Cell::Id(id(7)),
                Cell::Id(id(8)),
            ]],
        )
        .unwrap(),
    );
    (bundle, rows)
}
#[test]
fn compound_unit_values_recompute_from_changed_component_definitions_with_same_ids() {
    let registry = pse_schema::registry().unwrap();
    let (bundle, mut rows) = with_units("x == 2{cm/time}");
    let output = run(&bundle, &rows);
    let values = cells(&output, "normalized.template_expr_float_constants");
    assert_eq!(values[0][1], Cell::F64(0.02));
    assert_eq!(cells(&output, "normalized.units").len(), 2);
    let spec = registry.relation("reference.units").unwrap();
    let mut units = cells(&rows, "reference.units");
    let cm = units
        .iter_mut()
        .find(|row| row[0] == Cell::Id(id(20)))
        .unwrap();
    cm[4] = Cell::F64(0.02);
    rows.insert(
        spec.key,
        pse_relations::cells::batch_from_cells(registry, spec, &units).unwrap(),
    );
    let output = run(&bundle, &rows);
    assert_eq!(
        cells(&output, "normalized.template_expr_float_constants")[0][1],
        Cell::F64(0.04)
    );
}
#[test]
fn affine_literals_keep_authored_values_and_requests_do_not_guess_coefficients() {
    let (bundle, rows) = with_units("x == 2{degC}");
    let output = run(&bundle, &rows);
    let constants = cells(&output, "normalized.template_expr_float_constants");
    assert_eq!(constants[0][1], Cell::F64(2.0));
    assert_eq!(constants[0][2], Cell::Id(id(21)));
    let (bundle, rows) = with_units("convert(x, cm) == 2{cm}");
    let output = run(&bundle, &rows);
    let converts = cells(&output, "normalized.template_expr_unit_converts");
    assert_eq!(converts.len(), 1);
    assert_eq!(&converts[0][1..4], &[Cell::Null, Cell::Null, Cell::Null]);
    assert_eq!(converts[0][4], Cell::Id(id(20)));
    assert_eq!(converts[0][5], Cell::Enum("pending"));
}

#[test]
fn conditional_property_demands_preserve_both_branch_guards() {
    let registry = pse_schema::registry().unwrap();
    let initial = bundle("if true then x == 3 else x == 4");
    let mut texts = initial
        .documents
        .iter()
        .map(|document| (document.path.clone(), document.text.clone()))
        .collect::<BTreeMap<_, _>>();
    let document = texts.get_mut("templates/model.yaml").unwrap();
    *document = document.replace(
        "    sense: eq",
        &format!("    sense: eq\n    guard_id: '{}'", id(84)),
    );
    write!(document, "template_guards:\n  - id: '{}'\n    template_id: '{}'\n    predicate: 'false'\n    doc: ''\n",id(84),id(91)).unwrap();
    write!(texts.get_mut("templates/model.yaml").unwrap(), "template_symbol_properties:\n  - symbol_decl_id: '{}'\n    property_kind_id: '{}'\n    scope_selector_id: '{}'\n",id(92),id(80),id(81)).unwrap();
    texts.insert("cases/scopes.yaml".to_owned(),format!("scopes:\n  - id: '{}'\n    root_term_id: '{}'\nselector_terms:\n  - id: '{}'\n    scope_id: '{}'\n    ordinal: 0\n    op: kind_is\n    entity_kind: template\n",id(81),id(82),id(82),id(81)));
    let bundle = load_package_texts(texts, registry, ParseBudget::default()).unwrap();
    let mut rows = inputs(&bundle, registry);
    let spec = registry.relation("reference.property_kinds").unwrap();
    rows.insert(
        spec.key,
        pse_relations::cells::batch_from_cells(
            registry,
            spec,
            &[vec![
                Cell::Id(id(80)),
                Cell::text("fixture"),
                Cell::Id(id(83)),
                Cell::Null,
                Cell::List(vec![]),
                Cell::Enum("state"),
                Cell::text("fixture"),
            ]],
        )
        .unwrap(),
    );
    let output = run(&bundle, &rows);
    let seeds = cells(&output, "normalized.property_demand_seeds");
    assert_eq!(seeds.len(), 2);
    assert!(seeds.iter().all(|row| row[5] == Cell::List(vec![])
        && row[6] != Cell::Null
        && row[8] != Cell::Null
        && row[9] == Cell::Id(id(92))));
    assert_ne!(seeds[0][6], seeds[1][6]);
    let predicates = cells(&output, "normalized.predicate_nodes");
    assert!(predicates.iter().any(|row| row[2] == Cell::Enum("not")));
    let sources = cells(&output, "normalized.expression_sources");
    let equation_relation = registry.relation("authored.template_equations").unwrap();
    for seed in &seeds {
        let source = sources.iter().find(|row| row[0] == seed[1]).unwrap();
        assert_eq!(source[2], Cell::Id(equation_relation.id));
    }
    let equations = cells(&output, "normalized.template_equations");
    let guard_column = equation_relation
        .columns
        .iter()
        .position(|column| column.name == "guard_id")
        .unwrap();
    assert_eq!(equations[0][guard_column], Cell::Id(id(84)));
    let guard_relation = registry.relation("authored.template_guards").unwrap();
    let guard_source = sources
        .iter()
        .find(|row| row[2] == Cell::Id(guard_relation.id))
        .unwrap();
    let guard_root = predicates
        .iter()
        .find(|row| row[0] == guard_source[0] && row[1] == guard_source[7])
        .unwrap();
    assert_eq!(guard_root[3], Cell::Bool(false));
}

#[test]
fn opaque_requirement_guards_bind_declared_indices_and_keep_concrete_members() {
    let registry = pse_schema::registry().unwrap();
    let initial = indexed_bundle("x[1, 2] == 2", false);
    let mut texts = initial
        .documents
        .iter()
        .map(|document| (document.path.clone(), document.text.clone()))
        .collect::<BTreeMap<_, _>>();
    write!(texts.get_mut("templates/model.yaml").unwrap(),
        "template_property_requirements:\n  - id: '{}'\n    template_id: '{}'\n    operation_id: '{}'\n    scope_selector_id: '{}'\n    property_kind_id: '{}'\n    guard: 'i > 0'\n    index_domain_bindings:\n      - index_name: i\n        domain_id: '{}'\n", id(85), id(91), id(86), id(81), id(80), id(71)).unwrap();
    texts.insert("cases/scopes.yaml".to_owned(),format!("scopes:\n  - id: '{}'\n    root_term_id: '{}'\nselector_terms:\n  - id: '{}'\n    scope_id: '{}'\n    ordinal: 0\n    op: kind_is\n    entity_kind: template\n",id(81),id(82),id(82),id(81)));
    let bundle = load_package_texts(texts, registry, ParseBudget::default()).unwrap();
    let mut rows = inputs(&bundle, registry);
    for (name, values) in [
        (
            "reference.property_kinds",
            vec![vec![
                Cell::Id(id(80)),
                Cell::text("fixture"),
                Cell::Id(id(83)),
                Cell::Null,
                Cell::List(vec![]),
                Cell::Enum("state"),
                Cell::text("fixture"),
            ]],
        ),
        (
            "authored.domains",
            vec![vec![
                Cell::Id(id(71)),
                Cell::Id(id(91)),
                Cell::Enum("custom"),
                Cell::Bool(false),
                Cell::Null,
                Cell::Null,
                Cell::text(""),
            ]],
        ),
        (
            "authored.domain_members",
            vec![vec![
                Cell::Id(id(71)),
                Cell::Id(id(72)),
                Cell::U64(0),
                Cell::text("first"),
                Cell::Null,
                Cell::Null,
            ]],
        ),
    ] {
        let spec = registry.relation(name).unwrap();
        rows.insert(
            spec.key,
            pse_relations::cells::batch_from_cells(registry, spec, &values).unwrap(),
        );
    }
    let output = run(&bundle, &rows);
    let seeds = cells(&output, "normalized.property_demand_seeds");
    assert_eq!(seeds.len(), 1);
    assert_eq!(seeds[0][5], Cell::List(vec![Cell::Id(id(72))]));
    assert_ne!(seeds[0][6], Cell::Null);
    let bindings = cells(&output, "normalized.expression_index_bindings");
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0][2], Cell::text("i"));
    let domain = registry.relation("authored.domains").unwrap();
    rows.insert(
        domain.key,
        pse_relations::cells::batch_from_cells(registry, domain, &[]).unwrap(),
    );
    assert!(
        normalize(
            &rows,
            &[bundle],
            registry,
            FixedBudget::new(512 << 20).as_ref(),
            &CancellationToken::new()
        )
        .is_err()
    );
}

fn enum_bundle(
    expression: &str,
    enum_id: SemanticId,
    parameter: bool,
    extra: &str,
) -> DocumentBundle {
    let initial = bundle(expression);
    let mut texts = initial
        .documents
        .iter()
        .map(|document| (document.path.clone(), document.text.clone()))
        .collect::<BTreeMap<_, _>>();
    let declaration = if parameter {
        format!(
            "template_params:\n  - template_id: '{}'\n    name: energy_balance_type\n    logical_type_id: '{}'\n    enum_id: '{enum_id}'\n    required: true\n    doc: ''\n",
            id(91),
            id(95)
        )
    } else {
        format!(
            "template_features:\n  - template_id: '{}'\n    name: energy_balance_type\n    kind: enum\n    enum_id: '{enum_id}'\n    doc: ''\n{extra}",
            id(91)
        )
    };
    texts
        .get_mut("templates/model.yaml")
        .unwrap()
        .push_str(&declaration);
    load_package_texts(
        texts,
        pse_schema::registry().unwrap(),
        ParseBudget::default(),
    )
    .unwrap()
}

#[test]
fn enum_guard_literals_retain_actual_declaration_identity_in_both_operand_positions() {
    let registry = pse_schema::registry().unwrap();
    let energy = registry.enum_spec("EnergyBalanceType").unwrap().id;
    let material = registry.enum_spec("MaterialBalanceType").unwrap().id;
    for parameter in [false, true] {
        for (predicate, member_column, enum_column, expression_column) in [
            ("energy_balance_type != none", 17, 16, 6),
            ("none == energy_balance_type", 14, 13, 5),
        ] {
            let bundle = enum_bundle(
                &format!("if {predicate} then x == 1 else x == 2"),
                energy,
                parameter,
                "",
            );
            let output = run(&bundle, &inputs(&bundle, registry));
            let predicates = cells(&output, "normalized.predicate_nodes");
            assert_eq!(predicates[0][member_column], Cell::text("none"));
            assert_eq!(predicates[0][enum_column], Cell::Id(energy));
            assert_eq!(predicates[0][expression_column], Cell::Null);
            let references = cells(&output, "normalized.template_expr_symbol_refs");
            assert_eq!(
                references.len(),
                2,
                "enum member must not create a numerical symbol reference"
            );
        }
    }
    let expression = "if energy_balance_type != none then x == 1 else x == 2";
    let before = enum_bundle(expression, energy, false, "");
    let after = enum_bundle(expression, material, false, "");
    assert_ne!(
        cells(
            &run(&before, &inputs(&before, registry)),
            "normalized.predicate_nodes"
        ),
        cells(
            &run(&after, &inputs(&after, registry)),
            "normalized.predicate_nodes"
        ),
        "identical template keys/member spellings cannot conceal changed enum context"
    );
}

#[test]
fn enum_guards_refuse_unknown_wrong_ambiguous_or_untyped_literal_context() {
    let registry = pse_schema::registry().unwrap();
    let energy = registry.enum_spec("EnergyBalanceType").unwrap().id;
    let material = registry.enum_spec("MaterialBalanceType").unwrap().id;
    let other = format!(
        "  - template_id: '{}'\n    name: other\n    kind: enum\n    enum_id: '{material}'\n    doc: ''\n",
        id(91)
    );
    let collision = format!(
        "  - template_id: '{}'\n    name: none\n    kind: bool\n    doc: ''\n",
        id(91)
    );
    for (predicate, identity, extra) in [
        ("energy_balance_type == componentTotal", energy, ""),
        ("energy_balance_type == unknown_member", energy, ""),
        ("energy_balance_type == none", id(99), ""),
        ("energy_balance_type == other", energy, other.as_str()),
        ("energy_balance_type == none", energy, collision.as_str()),
        ("energy_balance_type < none", energy, ""),
        ("none == none", energy, ""),
        ("none in energy_balance_type", energy, ""),
        ("energy_balance_type + none > 0", energy, ""),
    ] {
        let bundle = enum_bundle(
            &format!("if {predicate} then x == 1 else x == 2"),
            identity,
            false,
            extra,
        );
        assert!(
            normalize(
                &inputs(&bundle, registry),
                &[bundle],
                registry,
                FixedBudget::new(512 << 20).as_ref(),
                &CancellationToken::new()
            )
            .is_err(),
            "{predicate}"
        );
    }
}

#[test]
fn smoothing_epsilon_keeps_declared_unit_and_value_until_physical_admission() {
    for (unit, identity) in [("cm", id(20)), ("degC", id(21))] {
        let (bundle, rows) = with_units(&format!("smooth_min(x, x, eps=2{{{unit}}}) == x"));
        let output = run(&bundle, &rows);
        let values = cells(&output, "normalized.template_expr_smooth_ops");
        assert_eq!(
            &values[0][1..4],
            &[
                Cell::F64(2.0),
                Cell::Enum("pending_unit"),
                Cell::Id(identity)
            ]
        );
    }
    let (bundle, rows) = with_units("smooth_min(x, x, eps=2) == x");
    let output = run(&bundle, &rows);
    assert_eq!(
        &cells(&output, "normalized.template_expr_smooth_ops")[0][1..4],
        &[Cell::F64(2.0), Cell::Enum("coordinate"), Cell::Null]
    );
}
