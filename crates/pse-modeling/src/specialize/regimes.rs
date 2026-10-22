// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Alternative equations share their implicit owner's variables, physical score and criterion.
use super::*;
use crate::annotation::Annotation;
use crate::resolved::{Node, NodeIx, PredicateIx, Reference, ResolvedPredicate, Source};

/// One alternative residual set over the implicit block's shared unknowns.
#[derive(Clone, Debug, PartialEq)]
pub struct Regime {
    /// Regime identity within its instance.
    pub id: SemanticId,
    /// Declaration that authored the regime.
    pub declaration: DeclarationId,
    /// Equality residuals, one per shared unknown.
    pub equations: Vec<Row>,
    /// Regime-local starts, bounds, nominals and row scaling.
    pub annotations: Vec<Annotation>,
    /// Condition under which the regime may be selected; `true` when unauthored.
    pub eligibility: dsl::Predicate,
    /// Actual original selection guard.
    pub eligibility_root: PredicateIx,
}
/// Alternative regimes of one implicit block and the physical score that selects among them.
#[derive(Clone, Debug, PartialEq)]
pub struct RegimeSelection {
    /// Physical score minimized over eligible regimes.
    pub criterion: Expr,
    /// Sole executable criterion.
    pub criterion_root: NodeIx,
    /// Absolute score difference within which two regimes tie.
    pub tolerance: Expr,
    /// Sole executable tolerance.
    pub tolerance_root: NodeIx,
    /// Physical quantity type of the score.
    pub quantity: pse_quantity::QuantityTypeId,
    /// Regimes in authored order.
    pub alternatives: Vec<Regime>,
}
/// Authored single-residual selection, specialized to physical symbol identities.
#[derive(Clone, Debug, PartialEq)]
pub enum RootSelection {
    /// Explicit branch restriction, with equivalence established later from library mathematics.
    Branch {
        /// Independent original-source inspection.
        inspection: dsl::Predicate,
        /// Actual executable original restriction.
        predicate: PredicateIx,
    },
    /// Deterministic operational choice. A neighborhood predicate is not itself stability proof.
    Operational {
        /// Unknown targets and semantic anchors in canonical physical coordinates.
        anchors: BTreeMap<SemanticId, Expr>,
        /// Actual executable anchor roots in the same target order.
        anchor_roots: BTreeMap<SemanticId, NodeIx>,
        /// Checked realization/settings capability reference.
        settings: String,
        /// Optional authored branch neighborhood restriction.
        neighborhood: Option<dsl::Predicate>,
        /// Actual original neighborhood root.
        neighborhood_root: Option<PredicateIx>,
    },
}
impl Engine<'_, '_> {
    pub(super) fn regimes(
        &mut self,
        instance: InstanceId,
        scope: &Declaration,
        members: &BTreeMap<String, DeclarationId>,
        env: &Environment,
    ) -> Result<()> {
        if let Some(contract) = &scope.value.scope {
            if contract.branch.is_some() {
                let predicate = self.rewrite_predicate_field(
                    instance,
                    scope.declaration_id,
                    "scope.branch",
                    0,
                    env,
                    &[scope.declaration_id],
                )?;
                self.model.root_selections.insert(
                    instance,
                    RootSelection::Branch {
                        inspection: predicate.inspection,
                        predicate: predicate.root,
                    },
                );
            }
            if let Some(operation) = &contract.operational {
                if operation.settings != "native.kinsol.v1" || operation.anchors.is_empty() {
                    return Err(invalid(
                        scope.declaration_id,
                        "unknown implicit operational settings or empty anchor",
                    ));
                }
                let mut anchors = BTreeMap::new();
                let mut anchor_roots = BTreeMap::new();
                for (position, _anchor) in operation.anchors.iter().enumerate() {
                    let target = self.rewrite_field(
                        instance,
                        scope.declaration_id,
                        "scope.operational.anchors.target",
                        position,
                        env,
                        &[scope.declaration_id],
                    )?;
                    let Some(Node::Reference(Reference::Symbol(slot))) =
                        self.construction.get(target.root)
                    else {
                        return Err(invalid(
                            scope.declaration_id,
                            "operational anchor requires an unknown target",
                        ));
                    };
                    let symbol = self
                        .model
                        .symbols
                        .at(*slot)
                        .filter(|symbol| {
                            symbol.lineage.instance == instance
                                && symbol.role == Kind::Variable
                                && symbol.definition.is_none()
                        })
                        .ok_or_else(|| {
                            invalid(
                                scope.declaration_id,
                                "operational anchor target is not an implicit unknown",
                            )
                        })?;
                    let id = symbol.id;
                    let expression = self.rewrite_field(
                        instance,
                        scope.declaration_id,
                        "scope.operational.anchors.expression",
                        position,
                        env,
                        &[scope.declaration_id],
                    )?;
                    anchor_roots.insert(id, expression.root);
                    if anchors.insert(id, expression.inspection).is_some() {
                        return Err(invalid(
                            scope.declaration_id,
                            "duplicate operational anchor target",
                        ));
                    }
                }
                let neighborhood = operation
                    .neighborhood
                    .as_ref()
                    .map(|_| {
                        self.rewrite_predicate_field(
                            instance,
                            scope.declaration_id,
                            "scope.operational.neighborhood",
                            0,
                            env,
                            &[scope.declaration_id],
                        )
                    })
                    .transpose()?;
                let neighborhood_root = neighborhood.as_ref().map(|value| value.root);
                self.model.root_selections.insert(
                    instance,
                    RootSelection::Operational {
                        anchors,
                        anchor_roots,
                        settings: operation.settings.clone(),
                        neighborhood: neighborhood.map(|value| value.inspection),
                        neighborhood_root,
                    },
                );
            }
        }
        let rows = members
            .values()
            .filter(|id| self.p.declarations[id].value.kind == Kind::Regime)
            .map(|id| self.p.declarations[id].clone())
            .collect::<Vec<_>>();
        let selection = scope
            .value
            .scope
            .as_ref()
            .and_then(|s| s.selection.as_ref());
        if rows.is_empty() && selection.is_none() {
            return Ok(());
        }
        let Some(_selection) = selection else {
            return Err(invalid(
                scope.declaration_id,
                "implicit alternatives require a selection criterion",
            ));
        };
        if rows.is_empty()
            || self
                .model
                .equations
                .iter()
                .any(|r| r.lineage.instance == instance)
        {
            return Err(invalid(
                scope.declaration_id,
                "selected implicit blocks require alternatives and no common residual equations",
            ));
        }
        let criterion = self.rewrite_field(
            instance,
            scope.declaration_id,
            "scope.selection.criterion",
            0,
            env,
            &[scope.declaration_id],
        )?;
        let tolerance = self.rewrite_field(
            instance,
            scope.declaration_id,
            "scope.selection.tolerance",
            0,
            env,
            &[scope.declaration_id],
        )?;
        let Type::Quantity(quantity) = self.constructed_type(criterion.root)? else {
            return Err(invalid(
                scope.declaration_id,
                "physical regime score required",
            ));
        };
        let quantity = quantity
            .resolve_with_evidence(self.c.quantities, &BTreeMap::new(), self.c.preconditions)
            .map_err(|e| invalid(scope.declaration_id, e.to_string()))?;
        let mut alternatives = Vec::new();
        for regime in rows {
            self.checkpoint()?;
            let at = regime.declaration_id;
            let children = self.p.children.get(&at).cloned().unwrap_or_default();
            if children.iter().any(|id| {
                !matches!(
                    self.p.declarations[id].value.kind,
                    Kind::Equation | Kind::Annotation
                )
            }) {
                return Err(invalid(
                    at,
                    "regimes declare alternative equations and numerical annotations over shared unknowns",
                ));
            }
            let previous = self.states[&instance].members.clone();
            for child in &children {
                let row = &self.p.declarations[child];
                if let Some(shared) = previous.get(&row.name)
                    && !(row.value.kind == Kind::Annotation
                        && self.p.declarations[shared].value.kind == Kind::Annotation)
                {
                    return Err(invalid(
                        *child,
                        "regime declaration shadows a shared implicit member",
                    ));
                }
                self.states
                    .get_mut(&instance)
                    .ok_or_else(|| invalid(instance, "regime owner absent"))?
                    .members
                    .insert(row.name.clone(), *child);
            }
            let eligible = regime
                .value
                .scope
                .as_ref()
                .and_then(|s| s.eligibility.as_deref());
            let eligibility = if eligible.is_some() {
                self.rewrite_predicate_field(instance, at, "scope.eligibility", 0, env, &[at])?
            } else {
                let source = Source {
                    field: crate::expression::occurrences::OccurrenceKey {
                        declaration: at,
                        role: "derived.default_eligibility".into(),
                        position: 0,
                    },
                    span: Span::default(),
                    coordinate: None,
                };
                self.pair_predicate(
                    ResolvedPredicate::Bool(true),
                    dsl::PredicateKind::Bool(true),
                    source,
                )?
            };
            let mut equations = Vec::new();
            let annotation_start = self.model.annotations.len();
            for child in children {
                let row = self.p.declarations[&child].clone();
                if let Some(e) = &row.value.equation {
                    for coordinates in self.coordinates(
                        instance,
                        child,
                        env,
                        e.indices
                            .iter()
                            .map(|i| (i.name.as_str(), i.domain.as_str())),
                    )? {
                        let equation = self.rewrite_equation_field(
                            instance,
                            child,
                            "equation.expression",
                            0,
                            &coordinates_env(env, &coordinates),
                            &[child],
                        )?;
                        if equation.sense != EquationSense::Eq {
                            return Err(invalid(child, "regime residuals must be equalities"));
                        }
                        self.reserve(1)?;
                        let id = member_id(instance, child, &coordinates);
                        if self
                            .model
                            .equation_roots
                            .insert(id, equation.relation())
                            .is_some()
                        {
                            return Err(invalid(
                                child,
                                "one alternative row has competing constructed relations",
                            ));
                        }
                        equations.push(Row {
                            id,
                            equation: equation.inspection,
                            lineage: self.lineage(instance, &row, &[child]),
                        });
                    }
                } else {
                    self.annotation(instance, &row, env)?;
                }
            }
            let annotations = self.model.annotations.split_off(annotation_start);
            if annotations.iter().any(|a| {
                !matches!(
                    a.value,
                    crate::annotation::AnnotationValue::Start(_)
                        | crate::annotation::AnnotationValue::Bounds(..)
                        | crate::annotation::AnnotationValue::Nominal(_)
                        | crate::annotation::AnnotationValue::Scale(_)
                )
            }) {
                return Err(invalid(
                    at,
                    "regime annotations are starts, bounds, nominals and row scaling",
                ));
            }
            self.states
                .get_mut(&instance)
                .ok_or_else(|| invalid(instance, "regime owner absent"))?
                .members = previous;
            alternatives.push(Regime {
                id: member_id(instance, at, &[]),
                declaration: at,
                equations,
                annotations,
                eligibility_root: eligibility.root,
                eligibility: eligibility.inspection,
            });
        }
        let unknowns = self
            .model
            .symbols
            .values()
            .filter(|s| {
                s.lineage.instance == instance && s.role == Kind::Variable && s.expression.is_none()
            })
            .count();
        if unknowns == 0 || alternatives.iter().any(|r| r.equations.len() != unknowns) {
            return Err(invalid(
                scope.declaration_id,
                "each regime must define the same nonempty square unknown system",
            ));
        }
        self.model.regimes.insert(
            instance,
            RegimeSelection {
                criterion_root: criterion.root,
                criterion: criterion.inspection,
                tolerance_root: tolerance.root,
                tolerance: tolerance.inspection,
                quantity,
                alternatives,
            },
        );
        Ok(())
    }
}
