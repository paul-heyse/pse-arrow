// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The modeling source grammar inventory, declared at exact native field paths.
use crate::model::{FieldContract, ExpressionGrammar as G, expression::AnnotationSyntax, field_facets as F};
use arrow_schema::DataType;

pub(super) struct Annotation {
    pub kind: &'static str,
    shape: &'static str,
    count: usize,
    grammar: Option<G>,
    member: Option<&'static str>,
    target: bool,
}
pub(super) const ANNOTATIONS: &[Annotation] = &[
    Annotation { kind: "start", shape: "Expressions", count: 1, grammar: Some(G::Expression), member: None, target: true },
    Annotation { kind: "nominal", shape: "Expressions", count: 1, grammar: Some(G::Expression), member: None, target: true },
    Annotation { kind: "bounds", shape: "Expressions", count: 2, grammar: Some(G::Expression), member: None, target: true },
    Annotation { kind: "scale", shape: "Scheme", count: 0, grammar: None, member: Some("scheme"), target: true },
    Annotation { kind: "report", shape: "Label", count: 1, grammar: Some(G::Static), member: None, target: true },
    Annotation { kind: "valid", shape: "Expressions", count: 2, grammar: Some(G::Expression), member: None, target: true },
    Annotation { kind: "check", shape: "Predicate", count: 1, grammar: Some(G::Predicate), member: None, target: true },
    Annotation { kind: "objective", shape: "Objective", count: 0, grammar: None, member: Some("objective"), target: true },
    Annotation { kind: "connectivity", shape: "Connectivity", count: 0, grammar: None, member: Some("connectivity"), target: true },
    Annotation { kind: "accuracy_goal", shape: "AccuracyGoal", count: 0, grammar: None, member: Some("accuracy_goal"), target: true },
    Annotation { kind: "engineering_scale", shape: "EngineeringScale", count: 0, grammar: None, member: Some("engineering_scale"), target: true },
    Annotation { kind: "engineering_default", shape: "EngineeringDefault", count: 0, grammar: None, member: Some("engineering_default"), target: true },
    Annotation { kind: "engineering_rule", shape: "EngineeringRule", count: 0, grammar: None, member: None, target: false },
];

/// One explicit leaf is grammar-bearing; ordinary text and scientific cells stay native.
fn grammar(path: &str) -> Option<G> {
    Some(match path {
        "binding.expression" | "scope.parameters.default_value"
            | "scope.fixture.expected_failure.applicability.sets"
            | "scope.fixture.expected_failure.validity.sets" => G::Static,
        "binding.defined_by" | "equation.expression"
            | "state_specification.reconstructions.equation" => G::Equation,
        "function.validity" | "scope.branch" | "scope.operational.neighborhood"
            | "scope.eligibility" | "guard.predicate" | "requirement.predicate"
            | "coordinate_map.validity" | "table.requirements" | "applicability.predicate" => G::Predicate,
        "logic.proposition" => G::Proposition,
        "annotation.arguments" => G::AnnotationArguments,
        "annotation.target" => G::AnnotationTarget,
        "equation.condition.variable" | "function.body" | "function.applicability"
            | "function.external.output" | "scope.fixture.modes.events.guard"
            | "scope.fixture.modes.events.tolerance" | "scope.fixture.modes.events.reset.target"
            | "scope.fixture.modes.events.reset.expression" | "scope.fixture.integration.samples"
            | "scope.fixture.integration.initial_step" | "scope.fixture.integration.quadratures.target"
            | "scope.fixture.integration.quadratures.absolute_tolerance"
            | "scope.fixture.integration.schedules.target" | "scope.fixture.integration.schedules.times"
            | "scope.fixture.integration.schedules.values" | "scope.fixture.integration.schedules.lower"
            | "scope.fixture.integration.schedules.upper" | "scope.fixture.shooting.nodes"
            | "scope.fixture.specifications.target" | "scope.fixture.specifications.expression"
            | "scope.fixture.diagnostics.members" | "scope.fixture.expected_failure.members"
            | "scope.fixture.expected_failure.applicability.variables"
            | "scope.fixture.expected_failure.validity.variables" | "scope.operational.anchors.target"
            | "scope.operational.anchors.expression" | "scope.selection.criterion"
            | "scope.selection.tolerance" | "accumulator.tolerance" | "accumulator.boundary"
            | "contribution.target" | "contribution.expression" | "connection.from" | "connection.to"
            | "state_specification.supplied" | "state_specification.extends"
            | "state_specification.coordinates.target" | "state_specification.reconstructions.tolerance"
            | "state_specification.transports.expression" | "state_specification.transports.tolerance"
            | "state_port.specification" | "inventory_balance.axis" | "inventory_balance.inventory"
            | "inventory_balance.flux" | "inventory_balance.tolerance"
            | "inventory_balance.transfers.event" | "inventory_balance.transfers.expression"
            | "exchange.from" | "exchange.to" | "coordinate_slot.expression" | "response.body"
            | "continuous.lower" | "continuous.upper" | "discretization.elements" | "discretization.order"
            | "relaxation.target" | "relaxation.nominal" | "continuation.target" | "continuation.start"
            | "continuation.end" | "realization.function" | "realization.argument" | "ordered_set.member"
            | "ordered_set.weight" | "cardinality.member" | "cardinality.count" | "piecewise.input"
            | "piecewise.output" | "piecewise.abscissa" | "piecewise.ordinate" | "complementarity.first"
            | "complementarity.second" | "annotation.objective.weight"
            | "annotation.objective.normalization" | "annotation.objective.absolute_tolerance"
            | "annotation.objective.relative_tolerance" | "annotation.accuracy_goal.time"
            | "annotation.accuracy_goal.resolution" | "annotation.accuracy_goal.criterion_lower"
            | "annotation.accuracy_goal.criterion_upper" | "annotation.engineering_scale.value"
            | "attribute.derived" | "table.columns.derived" | "permission.targets" | "applicability.evidence"
            | "applicability.axis" | "applicability.lower" | "applicability.upper"
            | "applicability.alternatives" | "applicability.dependencies" | "reference_translation.temperature"
            | "reference_translation.pressure" | "reconstruction.normalization" | "expectation.actual"
            | "expectation.expected" | "expectation.tolerance" | "expectation.relative_tolerance" => G::Expression,
        _ => return None,
    })
}

fn role(path: &str) -> (&str, &str) {
    match path {
        "scope.fixture.modes.events.guard" | "scope.fixture.modes.events.tolerance" => (path, "flatten"),
        "scope.fixture.modes.events.reset.target" => ("scope.fixture.modes.{0}.events.{1}.reset.{2}.target", "zero"),
        "scope.fixture.modes.events.reset.expression" => ("scope.fixture.modes.{0}.events.{1}.reset.{2}.expression", "zero"),
        "scope.fixture.integration.schedules.target" => ("scope.fixture.integration.schedules.{0}.target", "zero"),
        "scope.fixture.integration.schedules.times" => ("scope.fixture.integration.schedules.{0}.times", "list"),
        "scope.fixture.integration.schedules.values" => ("scope.fixture.integration.schedules.{0}.values", "list"),
        "scope.fixture.integration.schedules.lower" => ("scope.fixture.integration.schedules.{0}.lower", "zero"),
        "scope.fixture.integration.schedules.upper" => ("scope.fixture.integration.schedules.{0}.upper", "zero"),
        "scope.fixture.diagnostics.members" => ("scope.fixture.diagnostics.{0}.members", "list"),
        "state_specification.coordinates.indices.domain" => ("state_specification.coordinates.{0}.indices.domain", "list"),
        "state_specification.reconstructions.indices.domain" => ("state_specification.reconstructions.{0}.indices.domain", "list"),
        "state_specification.transports.indices.domain" => ("state_specification.transports.{0}.indices.domain", "list"),
        _ => (path, "list"),
    }
}

pub(super) fn apply(field: FieldContract, path: &str) -> FieldContract {
    let native = field.field().clone();
    let native = match native.data_type() {
        DataType::Struct(children) => native.clone().with_data_type(DataType::Struct(children.iter()
            .map(|child| apply(FieldContract::from_field(child.as_ref().clone()), &format!("{path}.{}", child.name())).into_field()).collect())),
        DataType::List(child) => native.clone().with_data_type(DataType::List(apply(FieldContract::from_field(child.as_ref().clone()), path).into_field().into())),
        DataType::FixedSizeList(child, width) => native.clone().with_data_type(DataType::FixedSizeList(apply(FieldContract::from_field(child.as_ref().clone()), path).into_field().into(), *width)),
        _ => native,
    };
    let mut native = native;
    if native.data_type() == &DataType::Utf8 {
        let grammar = grammar(path).or_else(|| match field.extension() {
            Some(crate::model::ExtensionUse::ExprDsl(grammar)) => Some(grammar), _ => None,
        });
        if let Some(grammar) = grammar {
            native.metadata_mut().insert(F::EXTENSION.into(), "pse.expr_dsl".into());
            native.metadata_mut().insert(F::PARAMETER.into(), grammar.as_str().into());
            let (role, ordinal) = role(path);
            native.metadata_mut().insert(F::EXPRESSION_ROLE.into(), role.into());
            native.metadata_mut().insert(F::EXPRESSION_ORDINAL.into(), ordinal.into());
        }
    }
    if path == "annotation" {
        let cases = ANNOTATIONS.iter().map(|a| AnnotationSyntax {kind:a.kind.into(),shape:a.shape.into(),count:a.count,
            grammar:a.grammar.map(|g|g.as_str().into()),member:a.member.map(str::to_owned),target:a.target}).collect::<Vec<_>>();
        native.metadata_mut().insert(F::EXPRESSION_CASES.into(), serde_json::json!(cases).to_string());
    }
    FieldContract::from_field(native)
}
