// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    reason = "independent numerical publication assertions"
)]

use datafusion::{
    arrow::array::RecordBatch, common::TableReference, execution::context::SessionContext,
};
use pse_ids::SemanticId;
use pse_relations::generated::{
    enums::{PublicationKind, SolverTermination},
    runtime::{
        jacobian_coordinates, numerical_evaluations, numerical_programs, publications,
        solver_outcomes,
    },
};

fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}

struct Fixture {
    program: numerical_programs::Row,
    coordinates: Vec<(i64, i64, i64)>,
    evaluation: Option<numerical_evaluations::Row>,
    outcome: Option<solver_outcomes::Row>,
}
impl Default for Fixture {
    fn default() -> Self {
        Self {
            program: numerical_programs::Row {
                program_id: id(1),
                residual_dimension: 2,
                variable_columns: vec!["x".into(), "y".into()],
                jacobian_dimension: 2,
            },
            coordinates: vec![(0, 0, 0), (1, 1, 1)],
            evaluation: Some(numerical_evaluations::Row {
                program_id: id(1),
                scenario_id: id(2),
                residual_dimension: 2,
                variable_dimension: 2,
                jacobian_dimension: 2,
                residuals: vec![3.0, 4.0],
                jacobian: vec![1.0, 1.0],
            }),
            outcome: None,
        }
    }
}
impl Fixture {
    fn bind(self) -> (SessionContext, publications::Row) {
        let context = SessionContext::new();
        let mut record = publications::Row {
            workspace_id: id(10),
            publication_id: id(11),
            parent_publication_id: None,
            attempt_id: id(12),
            kind: PublicationKind::Relations,
            inputs: vec![],
            members: vec![],
        };
        let mut programs = numerical_programs::Builder::new().unwrap();
        programs.push(self.program).unwrap();
        bind(
            &context,
            &mut record,
            "runtime.numerical_programs",
            programs.finish().unwrap().into_batch(),
        );
        let mut coordinates = jacobian_coordinates::Builder::new().unwrap();
        for (ordinal, residual, variable) in self.coordinates {
            coordinates
                .push(jacobian_coordinates::Row {
                    program_id: id(1),
                    ordinal,
                    residual,
                    variable,
                })
                .unwrap();
        }
        bind(
            &context,
            &mut record,
            "runtime.jacobian_coordinates",
            coordinates.finish().unwrap().into_batch(),
        );
        if let Some(evaluation) = self.evaluation {
            let mut values = numerical_evaluations::Builder::new().unwrap();
            values.push(evaluation).unwrap();
            bind(
                &context,
                &mut record,
                "runtime.numerical_evaluations",
                values.finish().unwrap().into_batch(),
            );
        }
        if let Some(outcome) = self.outcome {
            let mut values = solver_outcomes::Builder::new().unwrap();
            values.push(outcome).unwrap();
            bind(
                &context,
                &mut record,
                "runtime.solver_outcomes",
                values.finish().unwrap().into_batch(),
            );
        }
        (context, record)
    }
    async fn valid(self) -> bool {
        let (context, record) = self.bind();
        admit(&context, &record).await
    }
}
fn bind(context: &SessionContext, record: &mut publications::Row, name: &str, batch: RecordBatch) {
    let spec = pse_schema::registry().unwrap().relation(name).unwrap();
    context
        .register_batch(
            TableReference::full("datafusion", "public", spec.key.name),
            batch,
        )
        .unwrap();
    record
        .members
        .push(publications::RuntimePublicationsFieldMembersItem {
            catalog_name: "datafusion".into(),
            schema_name: "public".into(),
            table_name: spec.key.name.into(),
            relation_id: spec.id,
            relation_version: i64::from(spec.key.version),
            contract_fingerprint: spec.fingerprint,
            table_uri: format!("memory://{name}"),
            delta_version: 1,
            selection: publications::RuntimePublicationsFieldMembersItemSelection::from_full(),
        });
}
async fn admit(context: &SessionContext, record: &publications::Row) -> bool {
    super::super::admission::admit(
        record,
        pse_schema::shared_registry().unwrap(),
        &context.state(),
    )
    .await
    .is_ok()
}

#[tokio::test]
async fn complete_program_and_empty_dimensions_are_admitted() {
    assert!(Fixture::default().valid().await);
    let mut fixture = Fixture::default();
    fixture.program.residual_dimension = 0;
    fixture.program.variable_columns.clear();
    fixture.program.jacobian_dimension = 0;
    fixture.coordinates.clear();
    fixture.evaluation = Some(numerical_evaluations::Row {
        program_id: id(1),
        scenario_id: id(2),
        residual_dimension: 0,
        variable_dimension: 0,
        jacobian_dimension: 0,
        residuals: vec![],
        jacobian: vec![],
    });
    assert!(fixture.valid().await);
}

#[tokio::test]
async fn locally_consistent_vectors_cannot_forge_program_dimensions() {
    for dimension in 0..3 {
        let mut fixture = Fixture::default();
        let result = fixture.evaluation.as_mut().unwrap();
        match dimension {
            0 => {
                result.residual_dimension = 3;
                result.residuals.push(0.0);
            }
            1 => result.variable_dimension = 3,
            _ => {
                result.jacobian_dimension = 3;
                result.jacobian.push(0.0);
            }
        }
        assert!(!fixture.valid().await, "forged dimension {dimension}");
    }
}

#[tokio::test]
async fn sparse_coordinates_require_complete_unique_ordinals_and_bounded_unique_pairs() {
    for coordinates in [
        vec![],
        vec![(0, 0, 0)],
        vec![(0, 0, 0), (2, 1, 1)],
        vec![(0, 0, 0), (0, 1, 1)],
        vec![(0, 0, 0), (1, 0, 0)],
        vec![(0, 2, 0), (1, 1, 1)],
        vec![(0, 0, 2), (1, 1, 1)],
    ] {
        assert!(
            !Fixture {
                coordinates,
                ..Fixture::default()
            }
            .valid()
            .await
        );
    }
}

#[tokio::test]
async fn only_the_exact_selected_program_and_coordinates_can_satisfy_results() {
    let (context, mut record) = Fixture::default().bind();
    let program = record.members.remove(0);
    assert!(
        !admit(&context, &record).await,
        "ambient program is not selected"
    );
    record.members.push(program.clone());
    let coordinates = record.members.remove(0);
    assert!(
        !admit(&context, &record).await,
        "ambient coordinates are not selected"
    );
    record.members.push(coordinates);
    let mut conflicting = program;
    conflicting.delta_version += 1;
    record.members.push(conflicting);
    assert!(
        !admit(&context, &record).await,
        "conflicting selected revisions"
    );
}

#[tokio::test]
async fn solver_outcomes_preserve_program_variable_order_and_objective_constraint_split() {
    for (columns, constraints, valid) in [
        (vec!["x", "y"], 1, true),
        (vec!["y", "x"], 1, false),
        (vec!["x", "y"], 2, false),
    ] {
        let fixture = Fixture {
            outcome: Some(solver_outcomes::Row {
                program_id: id(1),
                scenario_id: id(2),
                variable_columns: columns.into_iter().map(str::to_owned).collect(),
                constraint_dimension: constraints,
                ipopt_status: 0,
                termination: SolverTermination::Success,
                objective: Some(3.0),
                values: vec![Some(1.0); 2],
                constraints: vec![Some(0.0); usize::try_from(constraints).unwrap()],
                constraint_duals: vec![Some(0.0); usize::try_from(constraints).unwrap()],
                lower_duals: vec![Some(0.0); 2],
                upper_duals: vec![Some(0.0); 2],
                diagnostic: None,
                iterations: vec![],
            }),
            ..Fixture::default()
        };
        assert_eq!(fixture.valid().await, valid);
    }
}

fn successful_outcome() -> solver_outcomes::Row {
    solver_outcomes::Row {
        program_id: id(1),
        scenario_id: id(2),
        variable_columns: vec!["x".into(), "y".into()],
        constraint_dimension: 1,
        ipopt_status: 0,
        termination: SolverTermination::Success,
        objective: Some(3.0),
        values: vec![Some(1.0); 2],
        constraints: vec![Some(0.0)],
        constraint_duals: vec![Some(0.0)],
        lower_duals: vec![Some(0.0); 2],
        upper_duals: vec![Some(0.0); 2],
        diagnostic: None,
        iterations: vec![],
    }
}

#[tokio::test]
async fn a_success_claim_requires_actual_available_values_and_a_successful_status() {
    for missing in [
        "objective",
        "values",
        "constraints",
        "constraint_duals",
        "lower_duals",
        "upper_duals",
        "status",
        "diagnostic",
    ] {
        let mut outcome = successful_outcome();
        match missing {
            "objective" => outcome.objective = None,
            "values" => outcome.values[0] = None,
            "constraints" => outcome.constraints[0] = None,
            "constraint_duals" => outcome.constraint_duals[0] = None,
            "lower_duals" => outcome.lower_duals[0] = None,
            "upper_duals" => outcome.upper_duals[0] = None,
            "status" => outcome.ipopt_status = -1,
            _ => {
                outcome.diagnostic = Some(solver_outcomes::RuntimeSolverOutcomesFieldDiagnostic {
                    code: Some("evaluation.failure".into()),
                    message: "failed callback".into(),
                });
            }
        }
        assert!(
            !Fixture {
                outcome: Some(outcome.clone()),
                ..Fixture::default()
            }
            .valid()
            .await,
            "{missing}"
        );
        outcome.termination = SolverTermination::EvaluationFailure;
        assert!(
            Fixture {
                outcome: Some(outcome),
                ..Fixture::default()
            }
            .valid()
            .await,
            "unavailable values must remain representable for {missing}"
        );
    }
}
