// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Read-only projection of the actual native routing and eligibility decision.
use pse_backend_native::routing::{Eligibility, Ineligible, Route};
use pse_model::generated::enums::*;

/// A selected native backend, or direct evaluation without free coordinates.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RouteDocument {
    /// Selected backend; absent for direct constant evaluation.
    pub backend: Option<NativeBackend>,
    /// Whether the selected model is evaluated directly.
    pub constant: bool,
}
impl From<Route> for RouteDocument {
    fn from(route: Route) -> Self {
        Self {
            backend: match route {
                Route::Native(backend) => Some(backend),
                Route::Constant => None,
            },
            constant: route == Route::Constant,
        }
    }
}
/// An adapter's retained contextual representation assessment.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EligibilityDocument {
    /// Backend assessed by the native routing owner.
    pub backend: NativeBackend,
    /// Whether no ineligibility reasons were retained.
    pub eligible: bool,
    /// Every retained applicable typed reason.
    pub reasons: Vec<IneligibleDocument>,
}
impl From<&Eligibility> for EligibilityDocument {
    fn from(eligibility: &Eligibility) -> Self {
        Self {
            backend: eligibility.backend,
            eligible: eligibility.reasons.is_empty(),
            reasons: eligibility.reasons.iter().map(Into::into).collect(),
        }
    }
}
/// One reason retained by the native eligibility owner, with its exact typed detail.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IneligibleDocument {
    /// Canonical native reason code.
    pub code: NativeIneligibility,
    /// Classes established by the prepared facts and requested intent.
    pub problem_classes: Vec<NativeProblemClass>,
    /// Required derivative order, with value, first and second encoded as 0, 1 and 2.
    pub derivative_order: Option<u8>,
    /// Whether shifted one-sided sign constraints were representable.
    pub sign_bounds: Option<bool>,
    /// Constraint forms outside this adapter's capability.
    pub forms: Vec<NativeConstraintForm>,
    /// Structural requirements this adapter or method cannot honour.
    pub requirements: Vec<ModelingStructuralRequirement>,
}
impl From<&Ineligible> for IneligibleDocument {
    fn from(reason: &Ineligible) -> Self {
        let mut document = Self {
            code: reason.code(),
            problem_classes: Vec::new(),
            derivative_order: None,
            sign_bounds: None,
            forms: Vec::new(),
            requirements: Vec::new(),
        };
        match reason {
            Ineligible::Class { problem } => document.problem_classes = problem.to_vec(),
            Ineligible::Derivatives { required } => {
                document.derivative_order = Some(match required {
                    pse_kernels::DerivativeOrder::Value => 0,
                    pse_kernels::DerivativeOrder::First => 1,
                    pse_kernels::DerivativeOrder::Second => 2,
                })
            }
            Ineligible::Bounds { signs } => document.sign_bounds = Some(*signs),
            Ineligible::NativeForms { missing } => document.forms = missing.to_vec(),
            Ineligible::Method { requirements } => document.requirements = requirements.to_vec(),
            _ => {}
        }
        document
    }
}
