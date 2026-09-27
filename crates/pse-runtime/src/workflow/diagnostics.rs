// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Attach only source evidence supplied by the compiler or native boundary.
use super::{RunReport, RunResult};
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

pub(super) fn observed(
    error: &(dyn std::error::Error + 'static),
    stage: &str,
) -> BoundaryDiagnostic {
    let mut result = BoundaryDiagnostic::new(Class::Internal, stage, [], "workflow.unclassified");
    result
        .observations
        .insert("detail".into(), Observation::Text(error.to_string()));
    let mut current = Some(error);
    while let Some(error) = current {
        if let Some(boundary) = error.downcast_ref::<BoundaryDiagnostic>() {
            return boundary.clone();
        }
        if let Some(modeling) = error.downcast_ref::<pse_modeling::ModelingError>() {
            return modeling.boundary_diagnostic();
        }
        if let Some(authoring) = error.downcast_ref::<pse_authoring::AuthoringError>() {
            use pse_authoring::AuthoringError as E;
            result.class = Class::InvalidModel;
            let (rule, span) = match authoring {
                E::Syntax { at, .. } => ("authoring.syntax", Some(*at)),
                E::Contract { at, .. } => ("authoring.contract", *at),
                E::UnknownKey { at, .. } => ("authoring.unknown_key", Some(*at)),
                E::MissingId { at, .. } => ("authoring.missing_id", Some(*at)),
                E::UnresolvedTarget { at, .. } => ("authoring.unresolved_target", Some(*at)),
                E::Budget { .. } => {
                    result.class = Class::ResourceLimit;
                    ("authoring.budget", None)
                }
                E::DocumentIo { .. } => ("authoring.document_io", None),
                E::DerivedWrite { .. } => ("authoring.derived_write", None),
                E::RenameNamed { .. } => ("authoring.rename_named", None),
                E::UnknownRowKey { .. } => ("authoring.unknown_row_key", None),
                E::PackageUnresolved { .. } => ("authoring.package_unresolved", None),
                E::PackageVersionConflict { .. } => ("authoring.package_version_conflict", None),
                E::SchemaVersionMismatch { .. } => ("authoring.schema_version", None),
            };
            result.rule = rule.into();
            if let Some(span) = span {
                result.sources.push(span.document_id);
                result.locations.push(SourceLocation {
                    source: span.document_id,
                    path: format!("document/{}", span.document_id),
                    name: None,
                    start: Some(span.start),
                    end: Some(span.end),
                });
            }
        }
        if let Some(super::WorkflowError::Contract(_)) =
            error.downcast_ref::<super::WorkflowError>()
        {
            result.class = Class::InvalidModel;
            result.rule = "workflow.contract".into();
        }
        if let Some(error) = error.downcast_ref::<pse_math::MathError>() {
            use pse_math::MathError as E;
            result.rule = match error {
                E::Instance { .. } => result.rule.as_str(),
                E::Domain { .. } => "math.domain",
                E::OutsideRange { .. } => "math.range",
                E::Evaluation { .. } => "math.evaluation",
                E::Provider { .. } => "math.provider",
                E::Cancelled => "math.cancelled",
                E::Limit(_) | E::WorkLimit { .. } => "math.limit",
                E::Contract(_) => "math.contract",
                E::Quantity(_) => "math.quantity",
                E::Library(_) => "math.library",
                E::CoefficientRange => "math.coefficient_range",
            }
            .into();
            match error {
                E::Instance { instance, .. } => result.sources.push(*instance),
                E::Domain { source_id, .. }
                | E::OutsideRange { source_id, .. }
                | E::Evaluation { source_id, .. } => {
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
                E::Limit(_) | E::WorkLimit { .. } => result.class = Class::ResourceLimit,
                E::Contract(_) | E::Quantity(_) => result.class = Class::InvalidModel,
                E::Library(_) => result.class = Class::Infrastructure,
                E::CoefficientRange => result.class = Class::Infrastructure,
            }
            if let E::OutsideRange {
                target,
                value,
                lower,
                upper,
                ..
            } = error
            {
                result.sources.push(*target);
                for (name, value) in [
                    ("value", Some(*value)),
                    ("lower", *lower),
                    ("upper", *upper),
                ] {
                    if let Some(value) = value {
                        result
                            .observations
                            .insert(name.into(), Observation::Real(value));
                    }
                }
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
            if let E::WorkLimit {
                source_id,
                resource,
                required,
                available,
                components,
            } = error
            {
                result.sources.push(*source_id);
                result
                    .observations
                    .insert("resource".into(), Observation::Text((*resource).into()));
                for (name, value) in [
                    ("required_operations", required),
                    ("available_operations", available),
                    ("taylor_components", components),
                ] {
                    result.observations.insert(
                        name.into(),
                        i64::try_from(*value).map_or_else(
                            |_| Observation::Text(value.to_string()),
                            Observation::Integer,
                        ),
                    );
                }
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
            result.rule = match error {
                E::Missing(_) => "compiler.missing",
                E::Syntax { .. } => "compiler.syntax",
                E::Math(_) => "compiler.math",
                E::Modeling(_) => "compiler.modeling",
                E::Structure(_) => "compiler.structure",
                E::Cancelled => "compiler.cancelled",
                E::Limit(_) => "compiler.limit",
            }
            .into();
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
        } else if let Some(driver) = error.downcast_ref::<crate::authoring_driver::DriverError>() {
            use crate::authoring_driver::DriverError as E;
            match driver {
                E::Authoring(e) => Some(e),
                E::Catalog(e) => Some(e),
                E::Resource(e) => Some(e),
                E::Allocation(e) => Some(e),
                E::Relation(e) => Some(e),
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
        } else if let Some(pse_compiler::workspace::CompileError::Modeling(e)) =
            error.downcast_ref::<pse_compiler::workspace::CompileError>()
        {
            Some(e)
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
        match &self.report {
            Err(error) => vec![error.boundary_diagnostic()],
            Ok(RunReport::Modeling(r)) => r.iter().filter_map(|r| r.diagnostic()).collect(),
            Ok(RunReport::Simulation(r)) => r.diagnostic().into_iter().collect(),
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
                .chain(r.validation_error.clone())
                .collect(),

        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn modeling_failure_retains_class_rule_and_source_through_wrappers() {
        let declaration = pse_ids::SemanticId::from_bytes([9; 16]);
        let error = pse_modeling::ModelingError::Unsupported {
            declaration,
            capability: "synthetic capability".into(),
        };
        let wrapped =
            super::super::WorkflowError::Math(crate::math::MathRuntimeError::Compile(error.into()));
        let diagnostic = wrapped.boundary_diagnostic();
        assert_eq!(diagnostic.class, Class::Unsupported);
        assert_eq!(diagnostic.rule, "modeling.capability");
        assert_eq!(diagnostic.sources, vec![declaration]);
        assert!(
            matches!(&diagnostic.observations["detail"], Observation::Text(value) if value == "synthetic capability")
        );
    }
    #[test]
    fn unattributed_failure_never_acquires_a_guessed_source() {
        let e = pse_backend_native::ProblemError::Contract("presolve evaluation failed".into());
        assert!(observed(&e, "presolve").sources.is_empty());
    }
    #[test]
    fn derivative_work_limit_retains_quantitative_witness_through_wrappers() {
        let source_id = pse_ids::SemanticId::from_bytes([8; 16]);
        let error = pse_math::MathError::WorkLimit {
            source_id,
            resource: "derivative expansion",
            required: 12,
            available: 3,
            components: 6,
        };
        let wrapped =
            super::super::WorkflowError::Math(crate::math::MathRuntimeError::Compile(error.into()));
        let diagnostic = wrapped.boundary_diagnostic();
        assert_eq!(diagnostic.class, Class::ResourceLimit);
        assert_eq!(diagnostic.rule, "math.limit");
        assert_eq!(diagnostic.sources, vec![source_id]);
        assert!(matches!(
            diagnostic.observations["required_operations"],
            Observation::Integer(12)
        ));
        assert!(matches!(
            diagnostic.observations["available_operations"],
            Observation::Integer(3)
        ));
        assert!(matches!(
            diagnostic.observations["taylor_components"],
            Observation::Integer(6)
        ));
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
