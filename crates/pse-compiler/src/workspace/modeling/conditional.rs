// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Admission of a unit's conditional original-equation problem, before iteration.
use super::*;
use std::collections::BTreeSet;

/// Compiler-owned source inventory for an explicitly selected conditional unit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConditionalUnitInventory {
    /// Original active owned equality identities, excluding connection equalities.
    pub residuals: BTreeSet<SemanticId>,
    /// Remaining free local symbols after direct boundary coordinates are supplied.
    pub unknowns: BTreeSet<SemanticId>,
}
/// Complete causal direction derived from authored connections and expression ports.
#[derive(Clone, Debug)]
pub struct AutomaticCausalUnit {
    /// Original instance owner.
    pub node: SemanticId,
    /// Authored scalar input port identities.
    pub inputs: BTreeSet<SemanticId>,
    /// Authored scalar output port identities.
    pub outputs: BTreeSet<SemanticId>,
    /// Actual equality inventory; empty means an explicit authored function map.
    pub local: ConditionalUnitInventory,
}

impl PreparedModeling {
    /// Inventory existing authored selected suppliers without creating regimes or
    /// inferring a regular sheet from equality matching. Descriptors that are already
    /// provider projections, rather than free original coordinates, stay explicit refusals.
    /// Fixed descriptor inputs are supplied by the bound original case.
    pub fn automatic_reduced_suppliers(
        &self,
        source: &CasePlan,
    ) -> Result<Vec<Alternative<pse_math::SharedAllocation<AdmittedImplicit>>>> {
        let rows: BTreeSet<_> = source.structure().rows().iter().map(|r| r.id).collect();
        self.implicit_order()?.into_iter().map(|supplier| {
            if matches!(supplier.selection.meaning, ImplicitMeaning::Relation)
                || supplier.selection.neighborhood_evidence == SelectionNeighborhood::Unestablished {
                Ok(Alternative::Unavailable("selected regular supplier correspondence is unestablished".into()))
            } else if supplier.unknowns.iter().any(|id| !source.columns().contains(id))
                || supplier.residuals.iter().any(|r| r.rows.iter().any(|id| !rows.contains(id))) {
                Ok(Alternative::Unavailable("authored implicit supplier is not a free-coordinate original equality region".into()))
            } else {
                Ok(Alternative::Available(supplier))
            }
        }).collect()
    }
    /// Derive candidate unit boundaries from authored directed connections and
    /// expression ports. Unknown unconnected directions remain inputs; actual
    /// conditional locality and capability admission still owns eligibility.
    pub fn automatic_causal_units(
        &self,
        source: &PreparedCase,
        nodes: &BTreeSet<InstanceId>,
    ) -> Result<Vec<AutomaticCausalUnit>> {
        let from: BTreeSet<_> = self
            .model
            .connections
            .values()
            .flat_map(|c| c.bindings.iter().map(|(from, _)| *from))
            .collect();
        let to: BTreeSet<_> = self
            .model
            .connections
            .values()
            .flat_map(|c| c.bindings.iter().map(|(_, to)| *to))
            .collect();
        let mut result = Vec::new();
        for node in nodes {
            if !self.model.instances.contains_key(node) {
                return Err(CompileError::Missing("automatic causal node absent".into()));
            }
            let mut inputs = BTreeSet::new();
            let mut outputs = BTreeSet::new();
            let mut input_symbols = BTreeSet::new();
            for port in self
                .model
                .ports
                .values()
                .filter(|p| p.lineage.instance == *node)
            {
                if from.contains(&port.id) && to.contains(&port.id) {
                    return Err(CompileError::ConditionalUnavailable(
                        "automatic causal port has conflicting authored directions".into(),
                    ));
                }
                let expression = self
                    .model
                    .symbols
                    .get(&port.symbol)
                    .is_some_and(|s| s.expression.is_some());
                if from.contains(&port.id) || !to.contains(&port.id) && expression {
                    outputs.insert(port.id);
                } else {
                    inputs.insert(port.id);
                    input_symbols.insert(port.symbol);
                }
            }
            let local = CompilerWorkspace::modeling_conditional_unit_inventory(
                self,
                source.plan.structure(),
                node.as_id(),
                &input_symbols,
            )?;
            result.push(AutomaticCausalUnit {
                node: node.as_id(),
                inputs,
                outputs,
                local,
            });
        }
        Ok(result)
    }
}

impl CompilerWorkspace {
    /// Discover the complete source inventory; admission still proves locality,
    /// matching and the selected mathematical capability before execution.
    pub fn modeling_conditional_unit_inventory(
        model: &PreparedModeling,
        source: &CaseStructure,
        node: SemanticId,
        inputs: &BTreeSet<SemanticId>,
    ) -> Result<ConditionalUnitInventory> {
        if !model.model.instances.keys().any(|id| id.as_id() == node)
            || inputs
                .iter()
                .any(|id| !model.model.symbols.contains_key(id))
        {
            return Err(CompileError::Missing(
                "conditional unit inventory requires an original node and boundary symbols".into(),
            ));
        }
        let local = |mut instance: InstanceId| {
            loop {
                if instance.as_id() == node {
                    return true;
                }
                let Some(parent) = model.model.instances.get(&instance).and_then(|i| i.parent)
                else {
                    return false;
                };
                instance = parent;
            }
        };
        let free: BTreeSet<_> = source
            .variables()
            .iter()
            .filter(|v| !v.fixed)
            .map(|v| v.port.id)
            .collect();
        let active: BTreeSet<_> = source.rows().iter().map(|r| r.id).collect();
        let connections: BTreeSet<_> = model
            .model
            .connections
            .values()
            .flat_map(|c| c.rows.iter().copied())
            .collect();
        Ok(ConditionalUnitInventory {
            unknowns: model
                .model
                .symbols
                .values()
                .filter(|s| {
                    local(s.lineage.instance) && free.contains(&s.id) && !inputs.contains(&s.id)
                })
                .map(|s| s.id)
                .collect(),
            residuals: model
                .model
                .equations
                .iter()
                .filter(|r| {
                    local(r.lineage.instance)
                        && active.contains(&r.id)
                        && !connections.contains(&r.id)
                })
                .map(|r| r.id)
                .collect(),
        })
    }
    /// Prepare an owned square unit problem with explicit fixed boundary coordinates.
    /// Connections are evaluated by the admitted causal graph; every other equation
    /// touching a local unknown must belong to this unit problem.
    #[expect(
        clippy::too_many_arguments,
        reason = "a conditional unit binds model ownership, selected case, boundary, original equations and compiler policy"
    )]
    pub fn prepare_modeling_conditional_unit(
        &self,
        model: &PreparedModeling,
        source: &PreparedCase,
        node: SemanticId,
        inputs: &BTreeSet<SemanticId>,
        outputs: &BTreeSet<SemanticId>,
        rows: &BTreeSet<SemanticId>,
        unknowns: &BTreeSet<SemanticId>,
        profile: Profile,
        cancel: &Arc<AtomicBool>,
    ) -> Result<PreparedBlock> {
        if !model.model.native.is_empty() {
            return Err(CompileError::ConditionalUnavailable("conditional unit cannot establish locality for native constraint handlers; request an explicit simultaneous strategy".into()));
        }
        let owned = Self::modeling_conditional_unit_inventory(
            model,
            source.plan.structure(),
            node,
            inputs,
        )?;
        let free: BTreeSet<_> = source.plan.columns().iter().copied().collect();
        let connections: BTreeSet<_> = model
            .model
            .connections
            .values()
            .flat_map(|c| c.rows.iter().copied())
            .collect();
        if owned.unknowns != *unknowns
            || owned.residuals != *rows
            || inputs
                .iter()
                .any(|id| !model.model.symbols.contains_key(id))
            || outputs
                .iter()
                .any(|id| !model.model.symbols.contains_key(id))
        {
            return Err(CompileError::Missing(format!(
                "conditional unit {node} must select its complete owned residuals and remaining local unknowns"
            )));
        }
        let direct: BTreeSet<_> = inputs
            .iter()
            .filter(|id| {
                source
                    .plan
                    .structure()
                    .variables()
                    .iter()
                    .any(|v| v.port.id == **id)
                    || source
                        .plan
                        .structure()
                        .parameters()
                        .iter()
                        .any(|p| p.id == **id)
            })
            .copied()
            .collect();
        let derived: BTreeSet<_> = inputs.difference(&direct).copied().collect();
        let mut augmented = source.plan.as_ref().clone();
        if !derived.is_empty() {
            let boundary = self.prepare_conditional_boundary_functions(
                model,
                &derived,
                unknowns.iter().copied().collect(),
                profile,
                cancel,
            )?;
            let mut instances = augmented.structure().instances().to_vec();
            let incidence = boundary.plan.incidence(cancel)?;
            for (index, binding) in boundary.plan.structure().instances().iter().enumerate() {
                for contribution in &binding.contributions {
                    for slot in incidence[index]
                        .first_for_output(contribution.output)
                        .ok_or_else(|| {
                            CompileError::Missing("conditional boundary incidence".into())
                        })?
                        .iter()
                        .chain(&incidence[index].support().controls)
                    {
                        let id = binding.slots[*slot].source();
                        if free.contains(&id) && !unknowns.contains(&id) && !direct.contains(&id) {
                            return Err(CompileError::ConditionalUnavailable(format!(
                                "conditional boundary has undeclared external dependency {id}"
                            )));
                        }
                    }
                }
                if let Some(existing) = instances
                    .iter_mut()
                    .find(|i| i.instance == binding.instance && i.body == binding.body)
                {
                    existing.contributions.extend(binding.contributions.clone());
                } else {
                    let mut projected = binding.clone();
                    if instances.iter().any(|i| i.instance == projected.instance) {
                        projected.instance = pse_ids::named_id(
                            projected.instance,
                            "conditional-boundary-observation",
                        );
                    }
                    instances.push(projected);
                }
            }
            let mut augmented_rows = augmented.structure().rows().to_vec();
            augmented_rows.extend(boundary.plan.structure().rows().iter().cloned().map(
                |mut row| {
                    row.lower = 0.0;
                    row.upper = 0.0;
                    row
                },
            ));
            let mut parameters = augmented.structure().parameters().to_vec();
            for symbol in &derived {
                let parameter = pse_ids::named_id(*symbol, "conditional-boundary-value");
                parameters.push(
                    boundary
                        .plan
                        .structure()
                        .parameters()
                        .iter()
                        .find(|p| p.id == parameter)
                        .ok_or_else(|| {
                            CompileError::Missing("conditional boundary parameter absent".into())
                        })?
                        .clone(),
                );
            }
            let structure = CaseStructure::new(
                augmented.structure().variables().to_vec(),
                parameters,
                instances,
                augmented_rows,
                None,
                CaseLimits::default(),
            )?;
            let mut bodies = augmented.bodies().clone();
            bodies.extend(boundary.plan.bodies().clone());
            augmented = CasePlan::prepare(
                Arc::new(structure),
                bodies,
                &self.inputs.quantities,
                DerivativeOrder::First,
                profile.assembly,
                cancel,
            )?;
        }
        let selected: BTreeSet<_> = rows
            .iter()
            .copied()
            .chain(
                derived
                    .iter()
                    .map(|id| ModelingOutput::ConditionalBoundary(*id).row_id()),
            )
            .collect();
        admit_boundary(
            &augmented,
            &direct,
            outputs,
            &selected,
            unknowns,
            &connections,
            cancel,
        )?;
        let observed = self.prepare_modeling_functions(
            model,
            outputs
                .iter()
                .map(|id| ModelingOutput::Member(*id).row_id())
                .collect(),
            Vec::new(),
            DerivativeOrder::Value,
            profile,
            cancel,
        )?;
        let incidence = observed.plan.incidence(cancel)?;
        for (index, instance) in observed.plan.structure().instances().iter().enumerate() {
            for contribution in &instance.contributions {
                for slot in incidence[index]
                    .first_for_output(contribution.output)
                    .ok_or_else(|| {
                        CompileError::Missing("conditional observation incidence".into())
                    })?
                    .iter()
                    .chain(&incidence[index].support().controls)
                {
                    let id = instance.slots[*slot].source();
                    if free.contains(&id) && !unknowns.contains(&id) && !inputs.contains(&id) {
                        return Err(CompileError::ConditionalUnavailable(format!(
                            "conditional output has undeclared external free dependency {id}"
                        )));
                    }
                }
            }
        }
        let plan = Arc::new(augmented.conditional(
            &selected,
            unknowns,
            &self.inputs.quantities,
            cancel,
        )?);
        let structure = structural_plan(node, &plan, cancel)?;
        let artifacts = artifact_requests(&plan, profile, self.inventory.environment(&self.db));
        Ok(PreparedBlock {
            class_proof_work: profile.class_proof_work,
            boundary: pse_structural::initialization::Block {
                id: pse_structural::incidence::BlockId(plan.structure().key()),
                members: pse_structural::incidence::Part {
                    rows: plan.structure().rows().iter().map(|r| r.id).collect(),
                    columns: plan.columns().to_vec(),
                },
                inputs: inputs.iter().copied().collect(),
            },
            plan,
            structure,
            artifacts,
        })
    }
}

fn admit_boundary(
    source: &CasePlan,
    inputs: &BTreeSet<SemanticId>,
    outputs: &BTreeSet<SemanticId>,
    rows: &BTreeSet<SemanticId>,
    unknowns: &BTreeSet<SemanticId>,
    connection_rows: &BTreeSet<SemanticId>,
    cancel: &Arc<AtomicBool>,
) -> Result<()> {
    let refuse = |message: &str| {
        CompileError::ConditionalUnavailable(format!(
            "conditional unit admission: {message}; request an explicit simultaneous strategy"
        ))
    };
    if !inputs.is_disjoint(unknowns)
        || rows
            .iter()
            .any(|id| !source.structure().rows().iter().any(|r| r.id == *id))
        || unknowns
            .iter()
            .any(|id| source.columns().binary_search(id).is_err())
    {
        return Err(CompileError::Missing(
            "conditional unit boundary names overlapping or absent source identities".into(),
        ));
    }
    if rows.is_empty()
        || rows.len() != unknowns.len()
        || source
            .structure()
            .rows()
            .iter()
            .any(|r| rows.contains(&r.id) && (!r.lower.is_finite() || r.lower != r.upper))
    {
        return Err(refuse(
            "expected disjoint boundary inputs and a square original equality inventory",
        ));
    }
    // Original support, including dependencies through demanded expression members,
    // is compiler-issued. Counting rows alone cannot establish this boundary.
    for dependency in source.dependencies(cancel)? {
        match dependency.target {
            Target::Objective(_) => {
                if dependency
                    .execution
                    .iter()
                    .any(|symbol| unknowns.contains(symbol))
                {
                    return Err(refuse("an original objective couples a local unknown"));
                }
            }
            Target::Row(row) => {
                for symbol in dependency.execution {
                    if rows.contains(&row)
                        && source.columns().binary_search(&symbol).is_ok()
                        && !unknowns.contains(&symbol)
                        && !inputs.contains(&symbol)
                    {
                        return Err(refuse(&format!(
                            "row {row} has undeclared external free dependency {symbol}"
                        )));
                    }
                    if !rows.contains(&row)
                        && !connection_rows.contains(&row)
                        && unknowns.contains(&symbol)
                    {
                        return Err(refuse(&format!(
                            "external row {row} couples local unknown {symbol}"
                        )));
                    }
                }
            }
        }
    }
    if outputs.iter().any(|id| {
        source.columns().binary_search(id).is_ok() && !unknowns.contains(id) && !inputs.contains(id)
    }) {
        return Err(refuse("a free output must be a solved local unknown"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn id(v: u8) -> SemanticId {
        SemanticId::from_bytes([v; 16])
    }
    fn source(external_row: bool) -> (PreparedCase, [SemanticId; 4]) {
        let connection = if external_row {
            "eq external:x==2;"
        } else {
            ""
        };
        source_text(
            &format!(
                "package p {{ def Root {{ var x:Scalar; var p:Scalar; eq local:x*p==1; {connection} }} }}"
            ),
            DerivativeOrder::First,
        )
    }
    fn source_text(text: &str, order: DerivativeOrder) -> (PreparedCase, [SemanticId; 4]) {
        let declarations = crate::authored_transfer_tests::rows(text);
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let mut workspace = CompilerWorkspace::new(
            crate::authored_transfer_tests::context(),
            WorkspaceLimits::default(),
        )
        .unwrap();
        workspace
            .publish_modeling(declarations, PhysicalScope::default())
            .unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let model = workspace
            .prepare_modeling_cancellable(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                cancel.clone(),
            )
            .unwrap();
        let symbol = |name: &str| {
            model
                .model
                .symbols
                .values()
                .find(|symbol| symbol.lineage.path.ends_with(&format!(".{name}")))
                .unwrap()
                .id
        };
        let row = |name: &str| {
            model
                .model
                .equations
                .iter()
                .find(|row| row.lineage.path.ends_with(&format!(".{name}")))
                .map_or(SemanticId::NIL, |row| row.id)
        };
        let ids = [symbol("x"), symbol("p"), row("local"), row("external")];
        let bound = model.bound_structure(&BTreeMap::new()).unwrap();
        let values = model
            .case_values(&ModelingCaseBindings {
                members: BTreeMap::from([(ids[0], 1.0), (ids[1], 1.0)]),
                ..Default::default()
            })
            .unwrap();
        let case = workspace
            .prepare_modeling_view(
                &model,
                bound.structure,
                &values,
                order,
                Profile::default(),
                &cancel,
            )
            .unwrap();
        (case, ids)
    }
    #[test]
    fn conditional_source_branch_cycle_is_execution_coupling_without_numerical_rank() {
        let (source, [x, p, local, external]) = source_text(
            "package p { def Root { var x:Scalar; var p:Scalar; eq local:x==if p>0 then 1 else 2; eq external:p==x; } }",
            DerivativeOrder::Value,
        );
        let cancel = Arc::new(AtomicBool::new(false));
        let dependencies = source.plan.dependencies(&cancel).unwrap();
        let branch = dependencies
            .iter()
            .find(|d| d.target == Target::Row(local))
            .unwrap();
        assert_eq!(branch.numerical, BTreeSet::from([x]));
        assert_eq!(branch.execution, BTreeSet::from([x, p]));
        assert_eq!(source.structure.blocks.len(), 2);
        let edges: Vec<_> = dependencies
            .into_iter()
            .flat_map(|d| match d.target {
                Target::Row(row) => d
                    .execution
                    .into_iter()
                    .map(move |column| (row, column))
                    .collect(),
                _ => Vec::new(),
            })
            .collect();
        let schedule = pse_structural::initialization::Plan::with_execution_dependencies(
            &source.structure,
            &edges,
            &cancel,
        )
        .unwrap();
        assert_eq!(schedule.blocks.len(), 1);
        assert_eq!(
            schedule.blocks[0]
                .members
                .rows
                .iter()
                .copied()
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([local, external])
        );
        let alternatives = source.automatic_alternatives(&cancel).unwrap();
        assert!(
            matches!(alternatives.initialization, Alternative::Available(ref plan) if plan.blocks.len() == 1)
        );
        let Alternative::Available(blocks) = source.automatic_blocks(&cancel).unwrap() else {
            panic!("the complete original equality schedule must remain preparable")
        };
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0]
                .plan
                .columns()
                .iter()
                .copied()
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([x, p])
        );
        // Existing conditional root blocks compile their actual First residual
        // minimum independently of the source's value-only preparation.
        assert_eq!(blocks[0].plan.order(), DerivativeOrder::First);
        let separator = source.automatic_separator(&cancel).unwrap().unwrap();
        assert!(separator.columns.contains(&p));
        assert!(separator.rows.contains(&local));
    }
    #[test]
    fn fixed_parameter_branch_is_retained_in_execution_projection() {
        let (source, [x, p, local, _]) = source_text(
            "package p { def Root { var x:Scalar; param p:Scalar=1; eq local:x==if p>0 then 1 else 2; } }",
            DerivativeOrder::Value,
        );
        assert!(!source.plan.columns().contains(&p));
        let dependencies = source
            .plan
            .dependencies(&Arc::new(AtomicBool::new(false)))
            .unwrap();
        let branch = dependencies
            .iter()
            .find(|d| d.target == Target::Row(local))
            .unwrap();
        assert_eq!(branch.numerical, BTreeSet::from([x]));
        assert_eq!(branch.execution, BTreeSet::from([x, p]));
    }
    #[test]
    fn conditional_locality_refuses_objective_coupling() {
        let (source, [x, p, local, _]) = source_text(
            "package p { def Root { var x:Scalar; var p:Scalar; eq local:x*p==1; annotation objective x(minimize); } }",
            DerivativeOrder::Value,
        );
        let error = admit_boundary(
            &source.plan,
            &BTreeSet::from([p]),
            &BTreeSet::from([x]),
            &BTreeSet::from([local]),
            &BTreeSet::from([x]),
            &BTreeSet::new(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("objective couples a local unknown")
        );
        assert!(matches!(error, CompileError::ConditionalUnavailable(_)));
    }
    #[test]
    fn conditional_unit_boundary_requires_declared_external_inputs() {
        let (source, [unknown, external, residual, _]) = source(false);
        let rows = BTreeSet::from([residual]);
        let unknowns = BTreeSet::from([unknown]);
        let outputs = unknowns.clone();
        let error = admit_boundary(
            &source.plan,
            &BTreeSet::new(),
            &outputs,
            &rows,
            &unknowns,
            &BTreeSet::new(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("undeclared external free dependency")
        );
        assert!(matches!(error, CompileError::ConditionalUnavailable(_)));
        let missing = admit_boundary(
            &source.plan,
            &BTreeSet::from([external]),
            &outputs,
            &BTreeSet::from([id(99)]),
            &unknowns,
            &BTreeSet::new(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap_err();
        assert!(matches!(missing, CompileError::Missing(_)));
        admit_boundary(
            &source.plan,
            &BTreeSet::from([external]),
            &outputs,
            &rows,
            &unknowns,
            &BTreeSet::new(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        assert!(
            admit_boundary(
                &source.plan,
                &unknowns,
                &outputs,
                &rows,
                &unknowns,
                &BTreeSet::new(),
                &Arc::new(AtomicBool::new(false)),
            )
            .is_err()
        );
    }
    #[test]
    fn conditional_unit_boundary_refuses_parent_coupling_before_iteration() {
        let (source, [unknown, external, residual, connection]) = source(true);
        let inputs = BTreeSet::from([external]);
        let rows = BTreeSet::from([residual]);
        let unknowns = BTreeSet::from([unknown]);
        let error = admit_boundary(
            &source.plan,
            &inputs,
            &unknowns,
            &rows,
            &unknowns,
            &BTreeSet::new(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap_err();
        assert!(error.to_string().contains("external row"));
        assert!(matches!(error, CompileError::ConditionalUnavailable(_)));
        // Only the declared connection occurrence may replace its equation with
        // physical propagation and the original tear residual validator.
        admit_boundary(
            &source.plan,
            &inputs,
            &unknowns,
            &rows,
            &unknowns,
            &BTreeSet::from([connection]),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        // A distinct output port may project an already supplied scalar, such as
        // shared pressure. It is a known boundary value, not a fabricated solve.
        admit_boundary(
            &source.plan,
            &inputs,
            &inputs,
            &rows,
            &unknowns,
            &BTreeSet::from([connection]),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    }
    #[test]
    fn conditional_unit_recycle_reference_declares_material_connections_and_stages() {
        let source =
            include_str!("../../../../../packages/reference/campaign/models/recycle-flash.pse");
        let declarations = pse_authoring::language::parse(
            source,
            id(73),
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let connections: Vec<_> = declarations
            .iter()
            .filter(|d| d.value.connection.is_some())
            .collect();
        assert_eq!(connections.len(), 7);
        assert_eq!(connections.iter().filter(|d| d.is_override).count(), 2);
        assert!(
            connections
                .iter()
                .filter(|d| d.is_override)
                .all(|d| d.name == "recycle")
        );
        assert!(!declarations.iter().any(|d| d.name == "recycle_flow"
            || d.name == "recycle_temperature"
            || d.name == "recycle_composition"));
        assert!(declarations.iter().any(|d| d.name == "tear_state"));
        assert!(declarations.iter().any(|d| d.name == "overheated_state"));
    }
    #[test]
    fn conditional_unit_recycle_reference_specializes_actual_unit_boundaries() {
        use crate::authored_transfer_tests::{context, reference_sources, root, rows};
        let source = format!(
            "{}\n{}\n{}\n{}\n{}",
            reference_sources(),
            include_str!("../../../../../packages/reference/process/models/mixer.pse"),
            include_str!("../../../../../packages/reference/process/models/flash.pse"),
            include_str!("../../../../../packages/reference/campaign/models/references.pse"),
            include_str!("../../../../../packages/reference/campaign/models/recycle-flash.pse")
        );
        let declarations = rows(&source);
        let root = root(
            &declarations,
            "recycle_flash",
            "recycle_flash_conditional_handoff",
        );
        let mut workspace = CompilerWorkspace::new(context(), WorkspaceLimits::default()).unwrap();
        workspace
            .publish_modeling(declarations, PhysicalScope::default())
            .unwrap();
        let model = workspace
            .specialize_modeling(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
            )
            .unwrap();
        assert_eq!(model.connections.len(), 5);
        assert!(
            model
                .connections
                .values()
                .all(|c| c.bindings.len() == 4 && c.rows.len() == 4)
        );
        assert!(
            model
                .connections
                .values()
                .any(|c| c.lineage.path.ends_with(".recycle"))
        );
        assert!(
            model
                .material_ports
                .values()
                .any(|p| p.lineage.path.ends_with(".mixer.outlet_port"))
        );
        assert!(
            model
                .equations
                .iter()
                .any(|r| r.lineage.path.ends_with(".heater.target_temperature"))
        );
        assert!(
            !model
                .equations
                .iter()
                .any(|r| r.lineage.path.ends_with(".recycle_temperature"))
        );
        let mut staged = Bindings::default();
        staged.facts.insert(
            pse_modeling::analysis::Fact::Stage("tear".into()),
            pse_modeling::specialize::Value::Boolean(true),
        );
        let staged = workspace
            .specialize_modeling(
                root,
                pse_modeling::specialize::root_instance(root),
                staged,
                Limits::default(),
            )
            .unwrap();
        let tear = staged
            .material_ports
            .values()
            .find(|p| p.lineage.path.ends_with(".tear_port"))
            .unwrap();
        let recycle = staged
            .connections
            .values()
            .find(|c| c.lineage.path.ends_with(".recycle"))
            .unwrap();
        assert_eq!(recycle.from, tear.id);
        assert_eq!(
            recycle.id,
            model
                .connections
                .values()
                .find(|c| c.lineage.path.ends_with(".recycle"))
                .unwrap()
                .id
        );
        assert_eq!(recycle.bindings.len(), 4);
        assert_eq!(staged.connections.len(), 5);
    }
}
