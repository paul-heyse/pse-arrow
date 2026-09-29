// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Case/test data binds existing symbols; it never adds equations or changes model fixedness.
use super::*;
use pse_model::generated::{
    authored::modeling_declarations::{
        AuthoredModelingDeclarationsFieldValueScopeFixture as Contract,
        AuthoredModelingDeclarationsFieldValueScopeOracle as Oracle,
    },
    enums::ModelingFixtureBinding as Binding,
};
/// Bindings of one independent symbol, in canonical units; unset parts keep the model's.
#[derive(Clone, Debug, PartialEq)]
pub struct FixtureValue {
    /// Bound variable or parameter.
    pub target: SemanticId,
    /// Specified value.
    pub value: Option<f64>,
    /// Whether the fixture fixes (`true`) or frees (`false`) the variable.
    pub fixed: Option<bool>,
    /// Lower bound override.
    pub lower: Option<f64>,
    /// Upper bound override.
    pub upper: Option<f64>,
}
/// One authored case or test fixture of a model instance.
#[derive(Clone, Debug, PartialEq)]
pub struct Fixture {
    /// Declaration that authored the fixture.
    pub declaration: DeclarationId,
    /// Degrees of freedom the fixture expects after its bindings.
    pub expected_degrees_of_freedom: i64,
    /// Bindings keyed by their model path.
    pub specifications: BTreeMap<String, FixtureValue>,
    /// Reference oracle the fixture's results are compared with.
    pub oracle: Option<Oracle>,
    /// Analysis route the fixture runs under; steady when unauthored.
    pub execution: pse_model::generated::enums::ModelingFixtureExecution,
    /// Declared solve intent (ADR-0119 Outcome 1); `None` leaves it to the runtime policy.
    pub intent: Option<pse_model::generated::enums::NativeSolveIntent>,
    /// Initialization stages to run, in order.
    pub stages: Vec<String>,
    /// Authored initialization settings.
    pub initialization: Option<pse_model::generated::authored::modeling_declarations::AuthoredModelingDeclarationsFieldValueScopeFixtureInitialization>,
    /// Integration samples and tolerances for the integrated route.
    pub integration: Option<IntegrationFixture>,
    /// Failure the fixture expects instead of a result.
    pub expected_failure: Option<pse_model::generated::authored::modeling_declarations::AuthoredModelingDeclarationsFieldValueScopeFixtureExpectedFailure>,
}
/// Authored integration samples expressed in the admitted axis's canonical unit.
#[derive(Clone, Debug, PartialEq)]
pub struct IntegrationFixture {
    pub samples: Vec<f64>,
    pub initial_step: f64,
    pub relative_tolerance: f64,
    pub normalized_absolute_tolerance: f64,
    pub quadrature_relative_tolerance: Option<f64>,
    pub quadratures: BTreeMap<SemanticId, f64>,
}
impl Engine<'_, '_> {
    pub(super) fn fixture(
        &mut self,
        instance: InstanceId,
        row: &Declaration,
        contract: &Contract,
        oracle: Option<Oracle>,
        env: &Environment,
    ) -> Result<()> {
        let at = row.declaration_id;
        let integration = if let Some(data) = &contract.integration {
            let axis = self.model.integrated.values().next().ok_or_else(|| {
                invalid(
                    at,
                    "integrated fixture requires the integrated analysis route and a time axis",
                )
            })?;
            let ty = Type::Quantity(pse_quantity::scheme::Scheme::Concrete(axis.quantity));
            let lower = axis.lower;
            let upper = axis.upper;
            let samples = data
                .samples
                .iter()
                .map(|s| self.eval(at, env, s, Some(&ty))?.scalar(at))
                .collect::<Result<Vec<_>>>()?;
            let initial_step = self
                .eval(at, env, &data.initial_step, Some(&ty))?
                .scalar(at)?;
            if !initial_step.is_finite()
                || initial_step <= 0.
                || samples
                    .iter()
                    .any(|s| !s.is_finite() || *s < lower || *s > upper)
                || samples.windows(2).any(|s| s[0] >= s[1])
            {
                return Err(invalid(
                    at,
                    "integration samples must increase within the declared axis and the step must be positive",
                ));
            }
            let mut quadratures = BTreeMap::new();
            for entry in &data.quadratures {
                let targets = self.annotation_targets(instance, &entry.target, env, at, false)?;
                let [(target, ty, local)] = targets.as_slice() else {
                    return Err(invalid(at, "one terminal integral per tolerance"));
                };
                // Authored lets retain their identity for diagnostics. Follow only
                // exact symbol aliases to the integrated coordinate; a sum or scaled
                // expression cannot define the integrator's absolute tolerance.
                let mut target = *target;
                let mut visited = BTreeSet::new();
                while !self.model.integrals.contains_key(&target) {
                    if !visited.insert(target) {
                        return Err(invalid(at, "cyclic quadrature tolerance target"));
                    }
                    target = self
                        .model
                        .symbols
                        .get(&target)
                        .and_then(|symbol| symbol.expression.as_ref())
                        .and_then(symbol_reference)
                        .ok_or_else(|| {
                            invalid(at, "quadrature tolerance target is not an integral")
                        })?;
                }
                let value = self
                    .eval(at, local, &entry.absolute_tolerance, Some(ty))?
                    .scalar(at)?;
                if !value.is_finite() || value <= 0. || quadratures.insert(target, value).is_some()
                {
                    return Err(invalid(
                        at,
                        "quadrature tolerances must be unique positive physical values",
                    ));
                }
            }
            if quadratures.keys().ne(self.model.integrals.keys()) {
                return Err(invalid(
                    at,
                    "each integrated quadrature requires an explicit physical tolerance",
                ));
            }
            Some(IntegrationFixture {
                samples,
                quadratures,
                quadrature_relative_tolerance: data.quadrature_relative_tolerance,
                initial_step,
                relative_tolerance: data.relative_tolerance,
                normalized_absolute_tolerance: data.normalized_absolute_tolerance,
            })
        } else {
            None
        };
        let mut specifications = BTreeMap::<String, FixtureValue>::new();
        let mut seen = BTreeSet::new();
        for s in &contract.specifications {
            let targets = self.annotation_targets(instance, &s.target, env, at, false)?;
            if targets.len() != 1 {
                return Err(invalid(
                    at,
                    "fixture specifications require one indexed scalar path",
                ));
            }
            let (target, ty, local) = targets
                .into_iter()
                .next()
                .ok_or_else(|| invalid(at, "empty fixture target"))?;
            let symbol = self.model.symbols.get(&target).ok_or_else(|| {
                invalid(at, "fixture target must be an independent physical symbol")
            })?;
            if symbol.expression.is_some()
                || !matches!(symbol.role, Kind::Variable | Kind::Parameter)
            {
                return Err(invalid(
                    at,
                    "fixture target must be an independent variable or parameter",
                ));
            }
            if s.kind != Binding::Value && symbol.role != Kind::Variable {
                return Err(invalid(at, "fixture fixedness/bounds require a variable"));
            }
            if !matches!(ty, Type::Quantity(_)) {
                return Err(invalid(at, "fixture requires a physical target"));
            }
            let key = if s.kind == Binding::Free {
                Binding::Fix
            } else {
                s.kind
            };
            if !seen.insert((target, key)) {
                return Err(invalid(at, "duplicate fixture specification"));
            }
            if s.kind == Binding::Free && s.expression.is_some() {
                return Err(invalid(at, "free fixture binding takes no value"));
            }
            let value = s
                .expression
                .as_ref()
                .map(|expr| self.eval(at, &local, expr, Some(&ty))?.scalar(at))
                .transpose()?;
            if matches!(s.kind, Binding::Value | Binding::Lower | Binding::Upper) && value.is_none()
            {
                return Err(invalid(at, "fixture binding requires a physical value"));
            }
            if value.is_some_and(|v| !v.is_finite()) {
                return Err(invalid(at, "nonfinite fixture value"));
            }
            if s.kind == Binding::Fix && value.is_some() && !seen.insert((target, Binding::Value)) {
                return Err(invalid(at, "duplicate fixture value through aliased paths"));
            }
            let path = if self.states[&instance].parent.is_none() {
                s.target.clone()
            } else {
                format!("{}.{}", self.states[&instance].path, s.target)
            };
            self.model.paths.insert(path.clone(), target);
            let entry = specifications.entry(path).or_insert(FixtureValue {
                target,
                value: None,
                fixed: None,
                lower: None,
                upper: None,
            });
            match s.kind {
                Binding::Value => {
                    if entry.value.is_some() {
                        return Err(invalid(at, "duplicate fixture value"));
                    }
                    entry.value = value;
                }
                Binding::Fix => {
                    entry.fixed = Some(true);
                    if value.is_some() {
                        if entry.value.is_some() {
                            return Err(invalid(at, "duplicate fixture value"));
                        }
                        entry.value = value;
                    }
                }
                Binding::Free => entry.fixed = Some(false),
                Binding::Lower => entry.lower = value,
                Binding::Upper => entry.upper = value,
            }
        }
        for s in specifications.values() {
            if s.lower.zip(s.upper).is_some_and(|(a, b)| a > b) {
                return Err(invalid(at, "reversed fixture bounds"));
            }
        }
        self.reserve(specifications.len() + 1)?;
        self.model.fixtures.insert(
            instance,
            Fixture {
                declaration: at,
                expected_degrees_of_freedom: contract.degrees_of_freedom,
                specifications,
                oracle,
                execution: contract
                    .execution
                    .unwrap_or(pse_model::generated::enums::ModelingFixtureExecution::Steady),
                intent: contract.intent,
                stages: contract.stages.clone(),
                initialization: contract.initialization.clone(),
                integration,
                expected_failure: contract.expected_failure.clone(),
            },
        );
        Ok(())
    }
}
