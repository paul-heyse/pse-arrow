// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Attach only source evidence supplied by the compiler or native boundary.
use super::{RunReport, RunRequest, RunResult};
use pse_model::diagnostic::{
    BoundaryClass as Class, BoundaryDiagnostic, Observation, SourceLocation,
};
use std::error::Error as _;

impl super::WorkflowError {
    /// Structured preparation/execution failure, retaining native source identities.
    pub fn boundary_diagnostic(&self) -> BoundaryDiagnostic {
        observed(self, "workflow")
    }
}

fn observed(error: &(dyn std::error::Error + 'static), stage: &str) -> BoundaryDiagnostic {
    let mut result = BoundaryDiagnostic::new(Class::Internal, stage, [], error.to_string());
    let mut current = Some(error);
    while let Some(error) = current {
        if let Some(boundary) = error.downcast_ref::<BoundaryDiagnostic>() {
            return boundary.clone();
        }
        if let Some(super::WorkflowError::Contract(_)) =
            error.downcast_ref::<super::WorkflowError>()
        {
            result.class = Class::InvalidModel;
        }
        if let Some(error) = error.downcast_ref::<pse_math::MathError>() {
            use pse_math::MathError as E;
            match error {
                E::Instance { instance, .. } => result.sources.push(*instance),
                E::Domain { source_id, .. } | E::Evaluation { source_id, .. } => {
                    result.sources.push(*source_id);
                    result.class = Class::TrialRejected;
                }
                E::Provider {
                    source_id,
                    provider,
                    ..
                } => {
                    result.sources.extend([*source_id, *provider]);
                    result
                        .observations
                        .insert("provider".into(), Observation::Text(provider.to_hex()));
                    result.class = Class::TrialRejected;
                }
                E::Cancelled => result.class = Class::Cancelled,
                E::Limit(_) => result.class = Class::ResourceLimit,
                E::Contract(_) | E::Quantity(_) => result.class = Class::InvalidModel,
                E::Library(_) => result.class = Class::Infrastructure,
            }
            if let E::Evaluation { order, .. } = error {
                result.observations.insert(
                    "derivative_order".into(),
                    Observation::Integer(match order {
                        pse_kernels::DerivativeOrder::Value => 0,
                        pse_kernels::DerivativeOrder::First => 1,
                        pse_kernels::DerivativeOrder::Second => 2,
                    }),
                );
            }
        }
        if let Some(error) = error.downcast_ref::<pse_backend_native::ProblemError>() {
            use pse_backend_native::ProblemError as E;
            match error {
                E::Unavailable { .. } => result.class = Class::Unsupported,
                E::Contract(_) => result.class = Class::InvalidModel,
                E::Structural { rows, columns, .. } => {
                    result.class = Class::InvalidModel;
                    result.sources.extend(rows);
                    result.sources.extend(columns);
                }
                E::Math(_) => {}
            }
        }
        if let Some(error) = error.downcast_ref::<crate::math::MathRuntimeError>() {
            use crate::math::MathRuntimeError as E;
            match error {
                E::Cancelled => result.class = Class::Cancelled,
                E::Limit(_) | E::Retiring | E::Pool(_) => result.class = Class::ResourceLimit,
                E::Infrastructure(_) => result.class = Class::Infrastructure,
                _ => {}
            }
        }
        if let Some(error) = error.downcast_ref::<pse_kernels::ProviderError>() {
            use pse_kernels::ProviderError as E;
            result.class = match error {
                E::Cancelled => Class::Cancelled,
                E::Limit(_) => Class::ResourceLimit,
                E::Contract(_) => Class::InvalidModel,
                E::Terminal(_) => Class::Infrastructure,
                E::Trial(_) | E::OutsideEnvelope { .. } | E::Singular(_) => Class::TrialRejected,
            };
            result.observations.insert(
                "provider_recoverable".into(),
                Observation::Boolean(matches!(
                    error,
                    E::Trial(_) | E::OutsideEnvelope { .. } | E::Singular(_)
                )),
            );
            if let E::OutsideEnvelope {
                axis,
                value,
                lower,
                upper,
            } = error
            {
                result
                    .observations
                    .insert("provider_axis".into(), Observation::Text(axis.clone()));
                for (name, value) in [
                    ("provider_value", *value),
                    ("provider_lower", *lower),
                    ("provider_upper", *upper),
                ] {
                    if value.is_finite() {
                        result
                            .observations
                            .insert(name.into(), Observation::Real(value));
                    }
                }
            }
        }
        if let Some(error) = error.downcast_ref::<pse_compiler::workspace::CompileError>() {
            use pse_compiler::workspace::CompileError as E;
            result.class = match error {
                E::Cancelled => Class::Cancelled,
                E::Limit(_) => Class::ResourceLimit,
                _ => Class::InvalidModel,
            };
            if let E::Syntax {
                definition,
                source_index,
                error,
            } = error
            {
                let (start, end) = match error.as_ref() {
                    pse_authoring::dsl::DslError::Syntax { span, .. } => {
                        (Some(span.start), Some(span.end))
                    }
                    pse_authoring::dsl::DslError::AmbiguousUnaryPower { offset }
                    | pse_authoring::dsl::DslError::NonFiniteNumber { offset } => {
                        (Some(*offset), Some(*offset))
                    }
                    pse_authoring::dsl::DslError::Budget { .. } => {
                        result.class = Class::ResourceLimit;
                        (None, None)
                    }
                };
                result.sources.push(*definition);
                result.locations.push(SourceLocation {
                    source: *definition,
                    path: format!("definition/{definition}/source/{source_index}"),
                    name: None,
                    start,
                    end,
                });
            }
        }
        current = if let Some(workflow) = error.downcast_ref::<super::WorkflowError>() {
            match workflow {
                super::WorkflowError::Boundary(error) => Some(error.as_ref()),
                super::WorkflowError::Math(error) => Some(error),
                super::WorkflowError::Engine(error) => Some(error),
                super::WorkflowError::Authoring(error) => Some(error),
                super::WorkflowError::Shared(error) => Some(error.as_ref()),
                super::WorkflowError::Contract(_) => None,
            }
        } else if let Some(pse_math::MathError::Instance { cause, .. }) =
            error.downcast_ref::<pse_math::MathError>()
        {
            Some(cause.as_ref())
        } else if let Some(error) = error.downcast_ref::<crate::math::MathRuntimeError>() {
            use crate::math::MathRuntimeError as E;
            match error {
                E::Math(e) => Some(e),
                E::Solve(e) => Some(e),
                E::Shared(e) => Some(e.as_ref()),
                E::Compile(e) => Some(e),
                _ => error.source(),
            }
        } else if let Some(pse_backend_native::ProblemError::Math(e)) =
            error.downcast_ref::<pse_backend_native::ProblemError>()
        {
            Some(e)
        } else if let Some(pse_compiler::workspace::CompileError::Math(e)) =
            error.downcast_ref::<pse_compiler::workspace::CompileError>()
        {
            Some(e.as_ref())
        } else {
            error.source()
        };
    }
    result.sources.sort_unstable();
    result.sources.dedup();
    result
}
impl RunResult {
    pub(super) fn capture_diagnostics(&self) -> Vec<BoundaryDiagnostic> {
        let mut diagnostics = match &self.report {
            Err(error) => vec![observed(error.as_ref(), "run")],
            Ok(RunReport::Solves(report)) => report
                .outcomes
                .iter()
                .filter_map(|outcome| match outcome {
                    crate::math::solves::Outcome::Rejected(e) => {
                        Some(observed(e.as_ref(), "solve"))
                    }
                    crate::math::solves::Outcome::Native(r) => {
                        r.validation_error.as_ref().map(|message| {
                            BoundaryDiagnostic::new(
                                Class::TrialRejected,
                                "solve.final_evaluation",
                                [],
                                message.clone(),
                            )
                        })
                    }
                    _ => None,
                })
                .collect(),
            Ok(RunReport::Simulation(r)) => r
                .error
                .as_ref()
                .map(|e| observed(e, "simulation"))
                .into_iter()
                .collect(),
            Ok(RunReport::Fit(r)) => r
                .diagnostic
                .iter()
                .map(|message| {
                    BoundaryDiagnostic::new(
                        Class::TrialRejected,
                        "fit.final_evaluation",
                        [],
                        message.clone(),
                    )
                })
                .chain(
                    r.solve
                        .iter()
                        .filter_map(|s| s.validation_error.as_ref())
                        .map(|message| {
                            BoundaryDiagnostic::new(
                                Class::TrialRejected,
                                "fit.candidate_validation",
                                [],
                                message.clone(),
                            )
                        }),
                )
                .collect(),
        };
        for diagnostic in &mut diagnostics {
            for revision in self.request.revisions() {
                enrich(diagnostic, revision);
            }
            if let RunRequest::Solves(steps) = &self.request {
                for step in steps {
                    for occurrences in step.compiled().occurrences.values() {
                        for occurrence in occurrences
                            .iter()
                            .filter(|o| diagnostic.sources.contains(&o.id))
                        {
                            diagnostic.locations.push(SourceLocation {
                                source: occurrence.id,
                                path: format!(
                                    "model/{}/case/{}/definition/{}",
                                    step.revision.0.row.model_id, step.case, occurrence.definition
                                ),
                                name: Some(step.revision.0.row.name.clone()),
                                start: Some(occurrence.span.start),
                                end: Some(occurrence.span.end),
                            });
                        }
                    }
                }
            }
        }
        diagnostics
    }
}

fn enrich(diagnostic: &mut BoundaryDiagnostic, revision: &super::ModelRevision) {
    for case in &revision.0.row.cases {
        for (symbol_id, quantity_id, unit_id) in case
            .variables
            .iter()
            .map(|variable| {
                (
                    variable.port.symbol_id,
                    variable.port.quantity_id,
                    variable.port.unit_id,
                )
            })
            .chain(
                case.parameters
                    .iter()
                    .map(|port| (port.symbol_id, port.quantity_id, port.unit_id)),
            )
        {
            if !diagnostic.sources.contains(&symbol_id) {
                continue;
            }
            let name =
                case.instances.iter().find_map(|instance| {
                    let index = instance
                        .slots
                        .iter()
                        .position(|slot| slot.source_id == symbol_id)?;
                    let definition =
                        revision.0.row.definitions.iter().find(|definition| {
                            definition.definition_id == instance.definition_id
                        })?;
                    definition
                        .formals
                        .get(index)
                        .map(|formal| format!("{}.{}", case.name, formal.path))
                });
            diagnostic.locations.push(SourceLocation {
                source: symbol_id,
                path: format!(
                    "model/{}/case/{}/port/{}",
                    revision.0.row.model_id, case.case_id, symbol_id
                ),
                name,
                start: None,
                end: None,
            });
            for (field, value) in [("unit", unit_id), ("quantity", quantity_id)] {
                diagnostic.observations.insert(
                    format!("port/{}/{field}", symbol_id),
                    Observation::Text(value.to_hex()),
                );
            }
            if let Some(unit) = revision
                .0
                .physical
                .quantities
                .units()
                .find(|unit| unit.id.as_id() == unit_id)
            {
                diagnostic.observations.insert(
                    format!("port/{}/unit_symbol", symbol_id),
                    Observation::Text(unit.symbol.clone()),
                );
            }
        }
        for instance in &case.instances {
            if diagnostic.sources.contains(&instance.instance_id) {
                diagnostic.locations.push(SourceLocation {
                    source: instance.instance_id,
                    path: format!(
                        "model/{}/case/{}/instance/{}",
                        revision.0.row.model_id, case.case_id, instance.instance_id
                    ),
                    name: Some(case.name.clone()),
                    start: None,
                    end: None,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unattributed_failure_never_acquires_a_guessed_source() {
        let e = pse_backend_native::ProblemError::Contract("presolve evaluation failed".into());
        assert!(observed(&e, "presolve").sources.is_empty());
    }
    #[test]
    fn instance_and_expression_are_retained_independently() {
        let instance = pse_ids::SemanticId::from_bytes([1; 16]);
        let source_id = pse_ids::SemanticId::from_bytes([2; 16]);
        let e = pse_math::MathError::Instance {
            instance,
            cause: Box::new(pse_math::MathError::Domain {
                source_id,
                requirement: "positive",
            }),
        };
        let d = observed(&e, "evaluation");
        assert_eq!(d.sources, vec![instance, source_id]);
        assert_eq!(d.class, Class::TrialRejected);
    }
    #[test]
    fn compiler_syntax_retains_definition_and_byte_span() {
        let definition = pse_ids::SemanticId::from_bytes([3; 16]);
        let error = pse_compiler::workspace::CompileError::Syntax {
            definition,
            source_index: 2,
            error: std::sync::Arc::new(pse_authoring::dsl::parse_expr("x + )").unwrap_err()),
        };
        let d = super::super::WorkflowError::Math(crate::math::MathRuntimeError::Compile(error))
            .boundary_diagnostic();
        assert_eq!(d.sources, vec![definition]);
        assert_eq!(d.locations[0].start, Some(4));
        assert_eq!(d.locations[0].end, Some(5));
    }
    #[test]
    fn exact_port_attribution_uses_authored_names_and_units() {
        let revision = super::super::dynamics::tests::source().freeze().unwrap();
        let id = super::super::tests::id(1);
        let mut diagnostic =
            BoundaryDiagnostic::new(Class::InvalidModel, "control", [id], "known offending port");
        enrich(&mut diagnostic, &revision);
        assert_eq!(
            diagnostic.locations[0].name.as_deref(),
            Some("functions.state")
        );
        assert!(
            diagnostic.locations[0]
                .path
                .ends_with(&format!("/port/{id}"))
        );
        assert!(
            matches!(&diagnostic.observations[&format!("port/{id}/unit_symbol")], Observation::Text(symbol) if symbol == "fixture_minute")
        );
    }
    #[test]
    fn provider_recoverability_is_preserved_with_its_witness() {
        let d = observed(
            &pse_math::MathError::Provider {
                source_id: pse_ids::SemanticId::from_bytes([4; 16]),
                provider: pse_ids::SemanticId::from_bytes([5; 16]),
                cause: pse_kernels::ProviderError::OutsideEnvelope {
                    axis: "temperature".into(),
                    value: 100.,
                    lower: 200.,
                    upper: 400.,
                },
            },
            "property",
        );
        assert_eq!(d.class, Class::TrialRejected);
        assert_eq!(d.sources.len(), 2);
        assert!(matches!(
            d.observations["provider_recoverable"],
            Observation::Boolean(true)
        ));
        let d = observed(
            &pse_kernels::ProviderError::Terminal("native failure".into()),
            "property",
        );
        assert_eq!(d.class, Class::Infrastructure);
        assert!(matches!(
            d.observations["provider_recoverable"],
            Observation::Boolean(false)
        ));
    }
}
