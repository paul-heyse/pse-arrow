// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Attributable mathematical admission and evaluation failures.
use pse_ids::SemanticId;
/// A failure at an authored expression occurrence, not a library-local node number.
#[derive(Debug, thiserror::Error)]
pub enum MathError {
    /// Local source failure attributed to the actual bound process instance.
    #[error("instance {instance}: {cause}")]
    Instance {
        /// Selected instance, distinct from the reusable definition occurrence.
        instance: SemanticId,
        /// Body-local checked-member tokens bound to actual semantic members.
        checked_members: std::collections::BTreeMap<SemanticId, SemanticId>,
        /// Original typed domain/provider/library cause and source occurrence.
        #[source]
        cause: Box<MathError>,
    },
    /// Invalid input, binding or execution profile.
    #[error("math contract: {0}")]
    Contract(String),
    /// Optional symbolic coefficient projection is outside finite f64 representation.
    #[error("symbolic coefficient is not representable as a finite f64")]
    CoefficientRange,
    /// Library construction or evaluation failed.
    #[error("math library: {0}")]
    Library(String),
    /// Runtime library failure attributed to its admitted source and derivative product.
    #[error("expression {source_id}, {order:?}: {detail}")]
    Evaluation {
        /// Original authored occurrence.
        source_id: SemanticId,
        /// Requested derivative order.
        order: pse_kernels::DerivativeOrder,
        /// Library diagnostic.
        detail: String,
    },
    /// An authored validity predicate of a form rejected the evaluation: the form's own
    /// domain or a data envelope it guards (ADR-0123 Outcome 4), named by its lineage.
    #[error(
        "{}-layer validity stated by {} rejected the evaluation",
        .0.layer.as_str(),
        .0.source
    )]
    Validity(Box<pse_model::diagnostic::ValidityLineage>),
    /// Required evidence was outside or unknown without a matching named permission.
    #[error("scientific applicability refused {} required claims", .0.refused.len())]
    Applicability(Box<pse_model::applicability::Assessment>),
    /// A required authored domain condition failed.
    #[error("expression {source_id}: {requirement}")]
    Domain {
        /// Original expression identity.
        source_id: SemanticId,
        /// The unmet condition.
        requirement: &'static str,
    },
    /// An observed canonical value lies outside an explicitly declared range.
    #[error("range {source_id}, target {target}: value {value} outside [{lower:?}, {upper:?}]")]
    OutsideRange {
        /// Authored bound or validity annotation.
        source_id: SemanticId,
        /// Actual specialized member, distinct from a reusable annotation.
        target: SemanticId,
        /// Observed value in the target's canonical unit.
        value: f64,
        /// Inclusive lower bound, when declared.
        lower: Option<f64>,
        /// Inclusive upper bound, when declared.
        upper: Option<f64>,
    },
    /// A configured finite admission bound was exceeded.
    #[error("math limit exceeded: {0}")]
    Limit(&'static str),
    /// A body needs more formal slots (inputs plus stage results) than its explicit
    /// allowance. The formal pool extends up to the allowance, never beyond it.
    #[error("math limit exceeded: body slots: {required} required, allowance {available}")]
    SlotLimit {
        /// Slots the refused allocation needs, counting every slot the body holds so far.
        required: usize,
        /// The body's formal-slot allowance (`BodyLimits::slots`).
        available: usize,
    },
    /// A construction demand exceeds its explicit work allowance.
    #[error(
        "expression {source_id}: {resource} requires {required} operations, allowance {available}, Taylor width {components}"
    )]
    WorkLimit {
        /// Authored occurrence of the mathematical block.
        source_id: SemanticId,
        /// The admitted operation category.
        resource: &'static str,
        /// Conservative required operation count.
        required: usize,
        /// Remaining operation allowance.
        available: usize,
        /// Number of Taylor coefficients per scalar.
        components: usize,
    },
    /// Requested cancellation.
    #[error("math evaluation cancelled")]
    Cancelled,
    /// Physical inference failed before normalization.
    #[error(transparent)]
    Quantity(#[from] pse_quantity::QuantityError),
    /// Provider failure, retaining its typed recoverability.
    #[error("provider {provider}: {cause}")]
    Provider {
        /// Authored call occurrence.
        source_id: SemanticId,
        /// Actual provider identity.
        provider: SemanticId,
        /// Original typed failure.
        #[source]
        cause: pse_kernels::ProviderError,
    },
    /// A retained typed boundary cause without a fabricated authored occurrence.
    #[error("{cause}")]
    Typed {
        /// Original source failure, projected by its owner.
        #[source]
        cause: pse_model::diagnostic::DiagnosticCause,
        /// Owned cause extent supplied by its producer.
        retained: usize,
    },
    /// An injected native inner solve failed. Its typed cause, including structural
    /// row and column identities, is retained for the owner that classifies it.
    #[error("native inner solve {source_id}: {cause}")]
    Native {
        /// Authored implicit block whose inner solve failed.
        source_id: SemanticId,
        /// Original typed native failure.
        #[source]
        cause: pse_model::diagnostic::DiagnosticCause,
        /// Owned extent of `cause`, reported by the producer that knows its type.
        retained: usize,
    },
}
impl MathError {
    /// Conservative owned extent of the complete source/provider error chain.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(match self {
            Self::Instance {
                cause,
                checked_members,
                ..
            } => cause.retained_bytes().saturating_add(
                checked_members
                    .len()
                    .saturating_mul(size_of::<(SemanticId, SemanticId)>() + 96),
            ),
            Self::Contract(s) | Self::Library(s) | Self::Evaluation { detail: s, .. } => {
                s.capacity()
            }
            Self::Quantity(e) => e.retained_bytes(),
            Self::Provider { cause, .. } => cause.retained_bytes(),
            Self::Native { retained, .. } | Self::Typed { retained, .. } => *retained,
            Self::Applicability(assessment) => assessment.retained_bytes(),
            Self::Validity(lineage) => size_of_val(lineage.as_ref()) + lineage.heap_bytes(),
            Self::Domain { .. }
            | Self::OutsideRange { .. }
            | Self::Limit(_)
            | Self::SlotLimit { .. }
            | Self::WorkLimit { .. }
            | Self::Cancelled
            | Self::CoefficientRange => 0,
        })
    }
}

pse_diagnostics::impl_diagnostic! {
    MathError,
    code(this) { match this {
        Self::Applicability(_) => Some(pse_diagnostics::DiagnosticCode::MathApplicability),
        Self::Validity(_) => Some(pse_diagnostics::DiagnosticCode::MathValidity),
        Self::Domain {..} => Some(pse_diagnostics::DiagnosticCode::MathDomain),
        Self::OutsideRange {..} => Some(pse_diagnostics::DiagnosticCode::MathRange),
        Self::Evaluation {..} => Some(pse_diagnostics::DiagnosticCode::MathEvaluation),
        Self::Cancelled => Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled),
        Self::Limit(_) | Self::SlotLimit {..} | Self::WorkLimit {..} => Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),
        Self::Quantity(_) | Self::Provider {..} | Self::Instance {..} | Self::Native {..} | Self::Typed {..} => None,
        Self::Contract(_) => Some(pse_diagnostics::DiagnosticCode::MathContract),
        Self::Library(_) => Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
        Self::CoefficientRange => Some(pse_diagnostics::DiagnosticCode::MathCoefficientRange),
    } },
    forward(this) { match this { Self::Quantity(e) => Some(e), Self::Instance {cause,..} => Some(cause.as_ref()), Self::Provider {cause,..} => Some(cause), Self::Native {cause,..} | Self::Typed {cause,..} => Some(cause.as_ref()), _ => None } },
    help(_this) { None }, related(_this) { None }, source(_this) { None }
}

impl pse_model::diagnostic::DiagnosticProjection for MathError {
    fn boundary_diagnostic(
        &self,
        stage: pse_diagnostics::DiagnosticStage,
    ) -> pse_model::diagnostic::BoundaryDiagnostic {
        self.boundary_diagnostic_with_members(stage, &std::collections::BTreeMap::new())
    }
    fn boundary_diagnostic_with_members(
        &self,
        stage: pse_diagnostics::DiagnosticStage,
        bindings: &std::collections::BTreeMap<SemanticId, SemanticId>,
    ) -> pse_model::diagnostic::BoundaryDiagnostic {
        use pse_diagnostics::{DiagnosticRule as R, TypedDiagnostic};
        use pse_model::diagnostic::{Observation as O, project_facts};
        // The closest instance supplies the actual member meaning. Each local token
        // is looked up once at its owner; actual identities are never mapped transitively.
        let effective = match self {
            Self::Instance {
                checked_members, ..
            } if !checked_members.is_empty() => {
                let mut effective = bindings.clone();
                effective.extend(checked_members);
                Some(effective)
            }
            _ => None,
        };
        let bindings = effective.as_ref().unwrap_or(bindings);
        let mut d = match self {
            Self::Instance { cause, .. } => cause.boundary_diagnostic_with_members(stage, bindings),
            Self::Provider { cause, .. } => cause.boundary_diagnostic_with_members(stage, bindings),
            Self::Native { cause, .. } | Self::Typed { cause, .. } => {
                cause.boundary_diagnostic_with_members(stage, bindings)
            }
            Self::Quantity(cause) => {
                project_facts(cause.diagnostic_code(), cause.diagnostic_facts(), stage)
            }
            Self::Contract(_)
            | Self::CoefficientRange
            | Self::Library(_)
            | Self::Evaluation { .. }
            | Self::Validity(_)
            | Self::Applicability(_)
            | Self::Domain { .. }
            | Self::OutsideRange { .. }
            | Self::Limit(_)
            | Self::SlotLimit { .. }
            | Self::WorkLimit { .. }
            | Self::Cancelled => {
                project_facts(self.diagnostic_code(), self.diagnostic_facts(), stage)
            }
        };
        match self {
            Self::Instance { instance, .. } => {
                d.causes = vec![d.clone()];
                d.sources.push(*instance);
                for observation in &mut d.applicability {
                    observation.instance = Some(*instance);
                }
            }
            Self::Validity(lineage) => {
                let mut lineage = lineage.as_ref().clone();
                if lineage.layer == pse_model::generated::enums::ModelingValidityLayer::Closure {
                    for member in &mut lineage.members {
                        if let Some(actual) = bindings.get(member) {
                            *member = *actual;
                        }
                    }
                }
                d.rule = R::MathValidity;
                d.sources.push(lineage.source);
                d.sources.extend(lineage.form);
                d.sources.extend(&lineage.sets);
                d.sources.extend(&lineage.members);
                d.validity = Some(lineage);
            }
            Self::Applicability(assessment) => {
                d.rule = R::MathApplicability;
                d.applicability = assessment.observations.clone();
                for o in &d.applicability {
                    d.sources.extend(o.claim.id);
                    d.sources.push(o.claim.form);
                    d.sources.extend(&o.claim.records);
                }
            }
            Self::Domain {
                source_id,
                requirement,
            } => {
                d.rule = R::MathDomain;
                d.sources.push(*source_id);
                d.observations
                    .insert("requirement".into(), O::Text((*requirement).into()));
            }
            Self::Evaluation {
                source_id, order, ..
            } => {
                d.rule = R::MathEvaluation;
                d.sources.push(*source_id);
                d.observations.insert(
                    "derivative_order".into(),
                    O::Integer(match order {
                        pse_kernels::DerivativeOrder::Value => 0,
                        pse_kernels::DerivativeOrder::First => 1,
                        pse_kernels::DerivativeOrder::Second => 2,
                    }),
                );
            }
            Self::OutsideRange {
                source_id,
                target,
                value,
                lower,
                upper,
            } => {
                d.rule = R::MathRange;
                d.sources.extend([*source_id, *target]);
                for (key, value) in [
                    ("value", Some(*value)),
                    ("lower", *lower),
                    ("upper", *upper),
                ] {
                    d.observations
                        .insert(key.into(), value.map_or(O::Missing, O::number));
                }
            }
            Self::Provider {
                source_id,
                provider,
                ..
            } => {
                d.causes = vec![d.clone()];
                d.sources.extend([*source_id, *provider]);
                d.observations
                    .insert("provider".into(), O::Text(provider.to_hex()));
            }
            Self::Native { source_id, .. } => {
                d.causes = vec![d.clone()];
                d.sources.push(*source_id);
            }
            Self::WorkLimit {
                source_id,
                resource,
                required,
                available,
                components,
            } => {
                d.rule = R::MathLimit;
                d.sources.push(*source_id);
                d.observations
                    .insert("resource".into(), O::Text((*resource).into()));
                for (key, value) in [
                    ("required_operations", required),
                    ("available_operations", available),
                    ("taylor_components", components),
                ] {
                    d.observations.insert(
                        key.into(),
                        i64::try_from(*value)
                            .map_or_else(|_| O::Text(value.to_string()), O::Integer),
                    );
                }
            }
            Self::SlotLimit {
                required,
                available,
            } => {
                d.rule = R::MathLimit;
                for (key, value) in [("required_slots", required), ("available_slots", available)] {
                    d.observations.insert(
                        key.into(),
                        i64::try_from(*value)
                            .map_or_else(|_| O::Text(value.to_string()), O::Integer),
                    );
                }
            }
            Self::Limit(_) => d.rule = R::MathLimit,
            Self::Cancelled => d.rule = R::MathCancelled,
            Self::Contract(_) => d.rule = R::MathContract,
            Self::CoefficientRange => d.rule = R::MathCoefficientRange,
            Self::Library(_) => d.rule = R::MathLibrary,
            Self::Quantity(_) | Self::Typed { .. } => {}
        }
        d.observations
            .insert("detail".into(), O::Text(self.to_string()));
        d.sources.sort_unstable();
        d.sources.dedup();
        d
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retained_failure_counts_owned_capacity_through_instance_and_provider() {
        let mut message = String::with_capacity(1 << 20);
        message.push('x');
        let capacity = message.capacity();
        let error = MathError::Instance {
            checked_members: Default::default(),
            instance: SemanticId::NIL,
            cause: Box::new(MathError::Provider {
                source_id: SemanticId::NIL,
                provider: SemanticId::NIL,
                cause: pse_kernels::ProviderError::Terminal(message),
            }),
        };
        assert!(error.retained_bytes() >= capacity + 2 * size_of::<MathError>());
    }
    #[test]
    fn checked_member_rebinding_preserves_nested_typed_envelopes_and_disposition() {
        use pse_diagnostics::{DiagnosticRule, DiagnosticStage};
        use pse_model::diagnostic::{
            BoundaryClass, BoundaryDiagnostic, DiagnosticCause, DiagnosticProjection, Observation,
            ValidityLineage,
        };
        use pse_model::generated::enums::ModelingValidityLayer;
        use std::collections::BTreeMap;
        let id = |n| SemanticId::from_bytes([n; 16]);
        let token = id(1);
        let actual = id(2);
        let lineage = ValidityLineage {
            layer: ModelingValidityLayer::Closure,
            source: id(3),
            form: Some(id(4)),
            sets: vec![id(5)],
            variables: vec![7],
            members: vec![token],
        };
        for stage in [
            DiagnosticStage::ModelingAdmission,
            DiagnosticStage::Evaluation,
        ] {
            let mut original = BoundaryDiagnostic::new(
                BoundaryClass::TrialRejected,
                stage,
                [id(3), id(4), id(5), token],
                DiagnosticRule::MathValidity,
            );
            original.validity = Some(lineage.clone());
            original
                .locations
                .push(pse_model::diagnostic::SourceLocation {
                    source: token,
                    revision: Some(pse_ids::ContentHash::from_bytes([11; 32])),
                    path: "authored/model.pse".into(),
                    name: Some("original declaration".into()),
                    start: Some(3),
                    end: Some(11),
                });
            original.observations.insert(
                "operands".into(),
                Observation::Contracts(vec![pse_diagnostics::OperandContract {
                    quantity: *token.as_bytes(),
                    indices: vec![[*token.as_bytes(), *id(7).as_bytes(), *id(8).as_bytes()]],
                }]),
            );
            original
                .observations
                .insert("provider_recoverable".into(), Observation::Boolean(true));
            original.observations.insert(
                "revision".into(),
                Observation::Text("original-revision".into()),
            );
            let mut nested = original.clone();
            nested.causes = vec![original.clone()];
            original.causes = vec![nested];
            let retained = pse_model::HeapUsage::heap_bytes(&original);
            let cause = MathError::Typed {
                cause: DiagnosticCause::new(original),
                retained,
            };
            let expected = cause.boundary_diagnostic(stage);
            let error = MathError::Instance {
                instance: id(6),
                checked_members: BTreeMap::from([
                    (token, actual),
                    (id(3), id(9)),
                    (id(4), id(9)),
                    (id(5), id(9)),
                ]),
                cause: Box::new(cause),
            };
            let projected = error.boundary_diagnostic(stage);
            assert_eq!(projected.class, BoundaryClass::TrialRejected);
            assert_eq!(projected.validity.as_ref().unwrap().members, vec![actual]);
            // The extra instance envelope retains the fully rebound original cause.
            let mut rebound = projected.causes[0].clone();
            fn restore(d: &mut BoundaryDiagnostic, token: SemanticId, actual: SemanticId) {
                if let Some(lineage) = &mut d.validity {
                    lineage.members = vec![token];
                }
                d.sources.retain(|source| *source != actual);
                for cause in &mut d.causes {
                    restore(cause, token, actual);
                }
            }
            for cause in &rebound.causes {
                assert_eq!(cause.validity.as_ref().unwrap().members, vec![actual]);
                assert_eq!(
                    cause.causes[0].validity.as_ref().unwrap().members,
                    vec![actual]
                );
            }
            assert!(
                rebound.sources.contains(&token),
                "independent declaration/quantity source survives a token collision"
            );
            assert!(rebound.sources.contains(&actual));
            restore(&mut rebound, token, actual);
            assert_eq!(
                serde_json::to_value(rebound).unwrap(),
                serde_json::to_value(expected).unwrap()
            );
            assert!(
                error.retained_bytes()
                    >= retained + 4 * (size_of::<(SemanticId, SemanticId)>() + 96)
            );
        }
        for (cause, recoverable) in [
            (
                pse_kernels::ProviderError::OutsideEnvelope {
                    axis: "temperature".into(),
                    value: 100.0,
                    lower: 200.0,
                    upper: 400.0,
                },
                true,
            ),
            (
                pse_kernels::ProviderError::Terminal("original infrastructure".into()),
                false,
            ),
        ] {
            let original = cause.boundary_diagnostic(DiagnosticStage::Property);
            let error = MathError::Instance {
                instance: id(6),
                checked_members: BTreeMap::from([(token, actual)]),
                cause: Box::new(MathError::Provider {
                    source_id: id(3),
                    provider: id(4),
                    cause,
                }),
            };
            let projected = error.boundary_diagnostic(DiagnosticStage::Property);
            assert_eq!(projected.class, original.class);
            assert!(
                matches!(projected.observations["provider_recoverable"], Observation::Boolean(value) if value == recoverable)
            );
        }
    }

    #[test]
    fn opaque_sources_without_a_typed_member_role_are_never_rewritten() {
        use pse_diagnostics::{DiagnosticRule, DiagnosticStage};
        use pse_model::diagnostic::{
            BoundaryClass, BoundaryDiagnostic, DiagnosticCause, DiagnosticProjection, Observation,
        };
        let token = SemanticId::from_bytes([1; 16]);
        let actual = SemanticId::from_bytes([2; 16]);
        let source = SemanticId::from_bytes([3; 16]);
        let mut original = BoundaryDiagnostic::new(
            BoundaryClass::Infrastructure,
            DiagnosticStage::ModelingAdmission,
            [token, source],
            DiagnosticRule::MathLibrary,
        );
        original.observations.insert(
            "opaque provenance".into(),
            Observation::Text(token.to_hex()),
        );
        original.causes = vec![original.clone()];
        let retained = pse_model::HeapUsage::heap_bytes(&original);
        let cause = MathError::Typed {
            cause: DiagnosticCause::new(original),
            retained,
        };
        let expected = cause.boundary_diagnostic(DiagnosticStage::ModelingAdmission);
        let error = MathError::Instance {
            instance: source,
            checked_members: std::collections::BTreeMap::from([(token, actual)]),
            cause: Box::new(cause),
        };
        let projected = error.boundary_diagnostic(DiagnosticStage::ModelingAdmission);
        assert!(projected.validity.is_none());
        assert!(projected.sources.contains(&token));
        assert!(!projected.sources.contains(&actual));
        assert_eq!(
            serde_json::to_value(&projected.causes[0]).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
    }

    #[test]
    fn checked_member_collision_preserves_native_and_inner_instance_source_roles() {
        use pse_diagnostics::DiagnosticStage;
        use pse_model::diagnostic::{DiagnosticCause, DiagnosticProjection, ValidityLineage};
        use pse_model::generated::enums::ModelingValidityLayer;
        use std::collections::BTreeMap;
        let token = SemanticId::from_bytes([1; 16]);
        let actual = SemanticId::from_bytes([2; 16]);
        let annotation = SemanticId::from_bytes([3; 16]);
        let outer = SemanticId::from_bytes([4; 16]);
        let validity = || {
            MathError::Validity(Box::new(ValidityLineage {
                layer: ModelingValidityLayer::Closure,
                source: annotation,
                form: None,
                sets: vec![],
                variables: vec![],
                members: vec![token],
            }))
        };
        for native in [false, true] {
            let cause = if native {
                let inner = validity();
                MathError::Native {
                    source_id: token,
                    retained: inner.retained_bytes(),
                    cause: DiagnosticCause::new(inner),
                }
            } else {
                MathError::Instance {
                    instance: token,
                    checked_members: BTreeMap::new(),
                    cause: Box::new(validity()),
                }
            };
            let expected_class = cause.boundary_diagnostic(DiagnosticStage::Evaluation).class;
            let error = MathError::Instance {
                instance: outer,
                checked_members: BTreeMap::from([(token, actual)]),
                cause: Box::new(cause),
            };
            let projected = error.boundary_diagnostic(DiagnosticStage::Evaluation);
            assert_eq!(projected.class, expected_class);
            assert_eq!(projected.validity.as_ref().unwrap().members, vec![actual]);
            assert!(
                projected.sources.contains(&token),
                "independent typed wrapper source survives"
            );
            let wrapper = &projected.causes[0];
            assert!(wrapper.sources.contains(&token));
            assert!(wrapper.sources.contains(&actual));
            let member_only = &wrapper.causes[0];
            assert_eq!(member_only.validity.as_ref().unwrap().members, vec![actual]);
            assert!(member_only.sources.contains(&actual));
            assert!(
                !member_only.sources.contains(&token),
                "member-only child has no independent wrapper source"
            );
            assert!(member_only.sources.contains(&annotation));
        }
        // An inner Instance also inherits its Native child's source role, while neither
        // identity is transplanted into the final member-only leaf.
        let inner = validity();
        let native = MathError::Native {
            source_id: token,
            retained: inner.retained_bytes(),
            cause: DiagnosticCause::new(inner),
        };
        let inner = MathError::Instance {
            instance: annotation,
            checked_members: BTreeMap::new(),
            cause: Box::new(native),
        };
        let error = MathError::Instance {
            instance: outer,
            checked_members: BTreeMap::from([(token, actual)]),
            cause: Box::new(inner),
        };
        let projected = error.boundary_diagnostic(DiagnosticStage::Evaluation);
        assert!(projected.sources.contains(&token));
        assert!(projected.causes[0].sources.contains(&token));
        assert!(projected.causes[0].causes[0].sources.contains(&token));
        assert!(
            !projected.causes[0].causes[0].causes[0]
                .sources
                .contains(&token)
        );
    }

    #[test]
    fn checked_member_context_crosses_typed_and_nested_native_adapters() {
        use pse_diagnostics::DiagnosticStage;
        use pse_model::diagnostic::{DiagnosticCause, DiagnosticProjection, ValidityLineage};
        use pse_model::generated::enums::ModelingValidityLayer;
        use std::collections::BTreeMap;
        let token = SemanticId::from_bytes([1; 16]);
        let actual = SemanticId::from_bytes([2; 16]);
        let annotation = SemanticId::from_bytes([3; 16]);
        let outer = SemanticId::from_bytes([4; 16]);
        for typed in [false, true] {
            let inner = MathError::Validity(Box::new(ValidityLineage {
                layer: ModelingValidityLayer::Closure,
                source: annotation,
                form: None,
                sets: vec![],
                variables: vec![],
                members: vec![token],
            }));
            let native = MathError::Native {
                source_id: token,
                retained: inner.retained_bytes(),
                cause: DiagnosticCause::new(inner),
            };
            let retained = native.retained_bytes();
            let cause = if typed {
                MathError::Typed {
                    retained,
                    cause: DiagnosticCause::new(native),
                }
            } else {
                MathError::Native {
                    source_id: token,
                    retained,
                    cause: DiagnosticCause::new(native),
                }
            };
            let expected_class = cause.boundary_diagnostic(DiagnosticStage::Evaluation).class;
            let error = MathError::Instance {
                instance: outer,
                checked_members: BTreeMap::from([(token, actual)]),
                cause: Box::new(cause),
            };
            let projected = error.boundary_diagnostic(DiagnosticStage::Evaluation);
            assert_eq!(projected.class, expected_class);
            let mut envelope = &projected;
            let mut depth = 0;
            loop {
                assert_eq!(envelope.validity.as_ref().unwrap().members, vec![actual]);
                assert!(envelope.sources.contains(&actual));
                assert!(envelope.sources.contains(&annotation));
                if envelope.causes.is_empty() {
                    assert!(
                        !envelope.sources.contains(&token),
                        "member-only leaf removes its local token"
                    );
                    break;
                }
                assert!(
                    envelope.sources.contains(&token),
                    "each enclosing Native retains its independent source role"
                );
                assert_eq!(envelope.causes.len(), 1);
                envelope = &envelope.causes[0];
                depth += 1;
            }
            assert_eq!(depth, if typed { 2 } else { 3 });
        }
    }

    #[test]
    fn nearest_instance_member_binding_is_not_transitively_rewritten() {
        use pse_diagnostics::DiagnosticStage;
        use pse_model::diagnostic::{DiagnosticProjection, ValidityLineage};
        use pse_model::generated::enums::ModelingValidityLayer;
        use std::collections::BTreeMap;
        let local = SemanticId::from_bytes([1; 16]);
        let inner_actual = SemanticId::from_bytes([2; 16]);
        let unrelated_outer_actual = SemanticId::from_bytes([3; 16]);
        let inner = MathError::Instance {
            instance: SemanticId::from_bytes([4; 16]),
            checked_members: BTreeMap::from([(local, inner_actual)]),
            cause: Box::new(MathError::Validity(Box::new(ValidityLineage {
                layer: ModelingValidityLayer::Closure,
                source: SemanticId::from_bytes([5; 16]),
                form: None,
                sets: vec![],
                variables: vec![],
                members: vec![local],
            }))),
        };
        let outer = MathError::Instance {
            instance: SemanticId::from_bytes([6; 16]),
            checked_members: BTreeMap::from([
                (local, unrelated_outer_actual),
                (inner_actual, unrelated_outer_actual),
            ]),
            cause: Box::new(inner),
        };
        let projected = outer.boundary_diagnostic(DiagnosticStage::Evaluation);
        assert_eq!(
            projected.validity.as_ref().unwrap().members,
            vec![inner_actual]
        );
        assert_eq!(
            projected.causes[0].validity.as_ref().unwrap().members,
            vec![inner_actual]
        );
        assert_eq!(
            projected.causes[0].causes[0]
                .validity
                .as_ref()
                .unwrap()
                .members,
            vec![inner_actual]
        );
        assert!(projected.sources.contains(&inner_actual));
        assert!(!projected.sources.contains(&unrelated_outer_actual));
    }
}
