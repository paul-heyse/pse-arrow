// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Data-layer guards at specialization (ADR-0123 Outcome 4).
//!
//! A function's data-layer guards are specialized in its frame, like its form-layer
//! predicate, under the policy the calling instance's consumer selected. A rejecting guard
//! remains a domain predicate of the specialized function. An extrapolating guard is
//! observed instead: its guarded argument is followed out to the call that makes it a model
//! member or a static value, through arguments that enclosing functions pass on unchanged.
//! An argument of any other shape, or a guarded call inside a conditional branch, cannot be
//! observed where it is evaluated and is refused, so an extrapolation is never silent.
use super::*;
use crate::envelope::{Guard, StaticObservation};
use pse_model::generated::enums::{
    ExtrapolationPolicy as Policy, ModelingValidityLayer as Layer,
};

/// What an extrapolating guard observes: the envelope, its bounds over the rows guarding
/// the argument, and where the observation was demanded.
#[derive(Clone, Debug)]
pub(super) struct Observed {
    /// The table or envelope declaration declaring the envelope.
    owner: DeclarationId,
    /// The axis quantity; the bounds are in its canonical unit.
    quantity: pse_quantity::QuantityTypeId,
    lower: f64,
    upper: f64,
    /// The declaration that selected extrapolation.
    selection: Option<DeclarationId>,
    /// The guarding function.
    function: DeclarationId,
}
impl Observed {
    /// Several rows guarding one argument: the argument lies within all of them.
    fn intersect(&mut self, other: &Self) {
        self.lower = self.lower.max(other.lower);
        self.upper = self.upper.min(other.upper);
    }
}
/// An extrapolating guard's observation on its way out to the call that resolves it.
#[derive(Clone, Debug)]
pub(super) struct Lift {
    /// The guarded formal of the function whose call resolves the lift.
    formal: usize,
    observed: Observed,
}
/// An observation resolved at a member or a static value, with the instance and demand
/// chain of the call that resolved it.
#[derive(Clone, Debug)]
pub(super) struct Resolved {
    observed: Observed,
    instance: InstanceId,
    demand: Vec<DeclarationId>,
}

impl Engine<'_, '_> {
    /// The data layer's policy for `instance` and the declaration that selected it; none
    /// selects reject.
    fn data_policy(&self, instance: InstanceId) -> (Policy, Option<DeclarationId>) {
        self.states
            .get(&instance)
            .and_then(|state| state.extrapolation)
            .map_or((Policy::Reject, None), |(policy, at)| (policy, Some(at)))
    }
    /// Specialize `function`'s guards in its frame, whose lexical formals the caller has
    /// installed, under the policy selected for `instance`; an extrapolating guard lifts one
    /// observation per guarded formal (`formals` maps argument names to formal positions).
    pub(super) fn specialize_guards(
        &mut self,
        instance: InstanceId,
        function: DeclarationId,
        guards: &[Guard],
        statics: &Environment,
        formals: &BTreeMap<String, usize>,
    ) -> Result<(Vec<Guard>, Vec<Lift>)> {
        let (policy, selection) = self.data_policy(instance);
        let mut specialized = Vec::with_capacity(guards.len());
        let mut lifts = Vec::new();
        for guard in guards {
            let mut predicate = self.rewrite_predicate(instance, &guard.predicate, statics, &[function])?;
            predicate.strip_spans();
            if policy == Policy::Extrapolate {
                let (lower, upper, quantity) = self.bounds(function, guard, statics)?;
                for argument in &guard.arguments {
                    let formal = *formals.get(argument).ok_or_else(|| {
                        invalid(function, format!("guarded argument {argument} has no scalar formal"))
                    })?;
                    lifts.push(Lift {
                        formal,
                        observed: Observed {
                            owner: guard.envelope.owner,
                            quantity,
                            lower,
                            upper,
                            selection,
                            function,
                        },
                    });
                }
            }
            specialized.push(Guard {
                predicate,
                policy,
                selection,
                ..guard.clone()
            });
        }
        Ok((specialized, lifts))
    }
    /// The bounds of `guard`'s envelope on its static carrier: the row's columns or the
    /// entity's attributes, in the axis's canonical unit.
    fn bounds(
        &self,
        function: DeclarationId,
        guard: &Guard,
        statics: &Environment,
    ) -> Result<(f64, f64, pse_quantity::QuantityTypeId)> {
        let carrier = statics.get(&guard.carrier);
        let bound = |name: &str| match carrier {
            Some(Value::Row { names, fields, .. }) => names
                .iter()
                .position(|n| n == name)
                .and_then(|i| fields.get(i).cloned()),
            Some(Value::Entity { id, .. }) => self
                .p
                .entities
                .get(id)
                .and_then(|record| record.values.get(name).cloned()),
            _ => None,
        };
        match (bound(&guard.envelope.lower), bound(&guard.envelope.upper)) {
            (
                Some(Value::Number { bits: lower, quantity }),
                Some(Value::Number { bits: upper, .. }),
            ) => Ok((f64::from_bits(lower), f64::from_bits(upper), quantity)),
            _ => Err(invalid(
                function,
                format!(
                    "envelope {} of {} is bounded by the numbers of its static row or entity",
                    guard.envelope.axis, guard.carrier
                ),
            )),
        }
    }
    /// Open the frame collecting the observations a function body passes on to its call.
    pub(super) fn open_lifts(&mut self) {
        self.lifted.push(Vec::new());
    }
    /// Close the innermost frame, returning its observations.
    pub(super) fn close_lifts(&mut self) -> Vec<Lift> {
        self.lifted.pop().unwrap_or_default()
    }
    /// Resolve `lifts` at one call of `function`: each guarded formal's actual is a model
    /// member, a static value, or, inside another function's body, that function's own
    /// argument passed on unchanged. `actual` holds the call's formal actuals and `sources`
    /// the source argument of each scalar formal.
    #[expect(
        clippy::too_many_arguments,
        reason = "one call site's actuals, sources, environment and demand chain"
    )]
    pub(super) fn resolve_lifts(
        &mut self,
        instance: InstanceId,
        function: DeclarationId,
        lifts: Vec<Lift>,
        actual: &[Expr],
        sources: &[Option<(&Expr, Type)>],
        env: &Environment,
        chain: &[DeclarationId],
    ) -> Result<()> {
        let name = self
            .p
            .declarations
            .get(&function)
            .map_or_else(|| function.to_string(), |row| row.name.clone());
        let refused = |reason: &str| {
            invalid(
                function,
                format!(
                    "a data envelope {name} guards is selected to extrapolate, and its extrapolation is recorded where the guarded argument is observed, but {reason}"
                ),
            )
        };
        for lift in lifts {
            if self.branches > 0 {
                return Err(refused(
                    "the guarded call is inside a conditional branch; select reject or call it unconditionally",
                ));
            }
            let expression = actual
                .get(lift.formal)
                .ok_or_else(|| invalid(function, "guarded formal outside the call"))?;
            let mut demand = chain.to_vec();
            demand.push(lift.observed.function);
            if let Some(member) = symbol_reference(expression) {
                let resolved = Resolved {
                    observed: lift.observed,
                    instance,
                    demand,
                };
                self.observed
                    .entry((member, resolved.observed.owner))
                    .and_modify(|previous| previous.observed.intersect(&resolved.observed))
                    .or_insert(resolved);
                continue;
            }
            if let Some(frame) = self.lifted.last_mut()
                && let ExprKind::Path(path) = &expression.kind
                && let [segment] = path.segments.as_slice()
                && segment.indices.is_empty()
                && let Some(formal) = segment
                    .name
                    .strip_prefix("arg_")
                    .and_then(|n| n.parse::<usize>().ok())
            {
                frame.push(Lift {
                    formal,
                    observed: lift.observed,
                });
                continue;
            }
            let at = chain.last().copied().unwrap_or(function);
            let value = match sources.get(lift.formal).and_then(Option::as_ref) {
                Some((source, ty)) => self.eval(at, env, &dsl::render_expr(source), Some(ty)).ok(),
                None => None,
            };
            if let Some(Value::Number { bits, .. }) = value {
                let resolved = Resolved {
                    observed: lift.observed,
                    instance,
                    demand,
                };
                self.observed_static
                    .entry((resolved.observed.owner, bits))
                    .and_modify(|previous| previous.observed.intersect(&resolved.observed))
                    .or_insert(resolved);
                continue;
            }
            return Err(refused(
                "its guarded argument is neither a model member, a static value nor an argument passed on unchanged; bind it to a member",
            ));
        }
        Ok(())
    }
    /// Publish the observations: at a member as a data-layer validity range, observed and
    /// not enforced; at a static value as its decided membership.
    pub(super) fn publish_observations(&mut self) -> Result<()> {
        for ((target, owner), resolved) in std::mem::take(&mut self.observed) {
            let observed = &resolved.observed;
            let ty = Type::Quantity(pse_quantity::scheme::Scheme::Concrete(observed.quantity));
            let lower = self.constant(observed.lower, &ty, owner)?;
            let upper = self.constant(observed.upper, &ty, owner)?;
            self.reserve(1)?;
            let row = self.p.declarations[&owner].clone();
            let lineage = self.lineage(resolved.instance, &row, &resolved.demand);
            self.model.annotations.push(crate::annotation::Annotation {
                target,
                value: crate::annotation::AnnotationValue::Valid {
                    lower,
                    upper,
                    policy: Policy::Extrapolate,
                    layer: Layer::Data,
                    selection: observed.selection,
                },
                lineage,
            });
        }
        for ((owner, bits), resolved) in std::mem::take(&mut self.observed_static) {
            self.reserve(1)?;
            let row = self.p.declarations[&owner].clone();
            self.model.observations.push(StaticObservation {
                id: pse_ids::named_id(owner.as_id(), &format!("static-observation:{bits:016x}")),
                value: f64::from_bits(bits),
                lower: resolved.observed.lower,
                upper: resolved.observed.upper,
                selection: resolved.observed.selection,
                lineage: self.lineage(resolved.instance, &row, &resolved.demand),
            });
        }
        Ok(())
    }
}
