// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact semantic entries use ordinary admitted local First artifacts.
use super::*;

#[derive(Clone, Debug)]
pub(super) struct EntryGroup {
    pub(super) instance: usize,
    pub(super) request: usize,
}
#[derive(Clone, Debug)]
struct EntryTerm {
    group: usize,
    offset: usize,
    destination: usize,
    scale: f64,
}
#[derive(Clone, Debug)]
pub(super) struct EntryPlan {
    pairs: Vec<(SemanticId, SemanticId)>,
    pub(super) groups: Vec<EntryGroup>,
    terms: Vec<EntryTerm>,
    providers: Vec<(SemanticId, SemanticId, pse_kernels::ProviderKey)>,
    pub(super) provider_visits: usize,
}
#[derive(Clone, Debug)]
pub(super) struct RepeatedProvider {
    instance: usize,
    source: SemanticId,
    key: pse_kernels::ProviderKey,
    ambiguous: bool,
}
impl EntryPlan {
    pub(super) fn bytes(&self) -> usize {
        size_of::<Self>()
            + self.pairs.capacity() * size_of::<(SemanticId, SemanticId)>()
            + self.groups.capacity() * size_of::<EntryGroup>()
            + self.terms.capacity() * size_of::<EntryTerm>()
            + self.providers.capacity()
                * size_of::<(SemanticId, SemanticId, pse_kernels::ProviderKey)>()
    }
}
#[derive(Debug, Default)]
pub(super) struct EntryScratch {
    inputs: Vec<Vec<f64>>,
    pub(super) trials: Vec<TrialEvaluation>,
    values: Vec<f64>,
    pub(super) visible: bool,
    contexts: Vec<Option<pse_kernels::ProviderLinearizationToken>>,
}
impl EntryScratch {
    pub(super) fn new(assembly: &CaseAssembly) -> Self {
        Self {
            inputs: assembly
                .entry_bindings
                .iter()
                .map(|b| vec![f64::NAN; b.len()])
                .collect(),
            trials: assembly
                .entry_bindings
                .iter()
                .map(|_| TrialEvaluation::default())
                .collect(),
            values: vec![f64::NAN; assembly.jacobian_entries().len()],
            visible: false,
            contexts: vec![None; assembly.repeated_entry_providers.len()],
        }
    }
    pub(super) fn clear(&mut self) {
        self.visible = false;
        for trial in &mut self.trials {
            trial.clear();
        }
        for inputs in &mut self.inputs {
            inputs.fill(f64::NAN);
        }
        self.values.fill(f64::NAN);
        self.contexts.fill(None);
    }
}
impl CasePlan {
    /// Exact semantic pairs in caller order, independent of complete structural support.
    pub fn jacobian_entries(&self) -> &[(SemanticId, SemanticId)] {
        self.entries.as_ref().map_or(&[], |entries| &entries.pairs)
    }
    /// Conservative provider-source projection of the exact selected support,
    /// including guard/control occurrences. Immutable registration alone does not
    /// establish an unseen exact product demand; built-in producers may select
    /// their ordinary full route before compilation when this list is nonempty.
    pub fn jacobian_entry_provider_sources(
        &self,
    ) -> &[(SemanticId, SemanticId, pse_kernels::ProviderKey)] {
        self.entries
            .as_ref()
            .map_or(&[], |entries| &entries.providers)
    }
    /// Select only Value, existing actions and these exact numerical First entries.
    /// The all-branch First pattern remains available for structural consumers.
    pub fn with_jacobian_entries(
        &self,
        pairs: &[(SemanticId, SemanticId)],
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self, MathError> {
        if cancel.load(Ordering::Acquire) {
            return Err(MathError::Cancelled);
        }
        if let Some(entries) = &self.entries {
            return if entries.pairs == pairs {
                Ok(self.clone())
            } else {
                Err(MathError::Contract(
                    "entry demand belongs to an immutable preparation".into(),
                ))
            };
        }
        if self.order != DerivativeOrder::First
            || pairs
                .iter()
                .any(|(row, column)| !self.rows.contains_key(row) || !self.columns.contains(column))
        {
            return Err(MathError::Contract(
                "entry demand requires admitted row and First coordinate membership".into(),
            ));
        }
        if pairs.len() > self.limits.contributions {
            return Err(MathError::Limit("selected entry request extent"));
        }
        // Charge the candidate/scatter population before collecting it. The source
        // contribution and slot bounds include aliases and repeated requested pairs.
        let candidates = pairs
            .iter()
            .try_fold(0usize, |count, (row, _)| {
                self.row_contributions(*row)
                    .iter()
                    .try_fold(count, |count, &(instance, _)| {
                        count.checked_add(self.structure.instances()[instance].slots.len().max(1))
                    })
            })
            .ok_or(MathError::Limit("selected entry population"))?;
        if candidates > self.limits.contributions {
            return Err(MathError::Limit("selected entry contributions"));
        }
        let bytes = candidates
            .checked_mul(1024)
            .and_then(|n| n.checked_add(pairs.len().checked_mul(128)?))
            .and_then(|n| n.checked_add(size_of::<Self>() + size_of::<EntryPlan>() + 4096))
            .and_then(|n| n.checked_add(self.requests.len().checked_mul(512)?))
            .and_then(|n| n.checked_add(self.bodies.len().checked_mul(192)?))
            .and_then(|n| {
                self.supports
                    .iter()
                    .try_fold(n, |n, support| n.checked_add(support.retained_bytes()))
            })
            .ok_or(MathError::Limit("selected entry metadata"))?;
        crate::construction::additional(bytes)?;
        // Preserve original binding/contribution order; formal aliases remain
        // separate derivative contributions even when they name one global column.
        let mut selected = BTreeMap::<(usize, Option<usize>), BTreeSet<usize>>::new();
        let mut pending = Vec::new();
        for (destination, &(row, column)) in pairs.iter().enumerate() {
            for &(instance, ordinal) in self.row_contributions(row) {
                let binding = &self.structure.instances()[instance];
                let contribution = &binding.contributions[ordinal];
                let support = self
                    .incidence_for(instance, contribution.output)
                    .ok_or_else(|| MathError::Contract("missing complete entry support".into()))?;
                let mut bound = false;
                for (formal, slot) in binding.slots.iter().enumerate() {
                    if slot.source() == column {
                        bound = true;
                        selected
                            .entry((instance, Some(formal)))
                            .or_default()
                            .insert(contribution.output);
                        if !support.contains(&formal) {
                            continue;
                        }
                        pending.push((
                            instance,
                            formal,
                            contribution.output,
                            destination,
                            contribution.scale * slot.scale(),
                        ));
                    }
                }
                if !bound {
                    selected
                        .entry((instance, None))
                        .or_default()
                        .insert(contribution.output);
                }
            }
        }
        let mut result = self.clone();
        let mut requests = self.requests.as_ref().clone();
        let mut supports = self.supports.as_ref().clone();
        let mut numerical = Vec::with_capacity(supports.len() + selected.len());
        for (request, support) in requests.iter_mut().zip(&supports) {
            if request.product == ProductKind::SelectedPartials {
                request.order = DerivativeOrder::Value;
            }
            numerical.push(Arc::new(support.at_order(request.order)?));
        }
        let mut groups = Vec::with_capacity(selected.len());
        let mut offsets = BTreeMap::new();
        for ((instance, formal), outputs) in selected {
            if cancel.load(Ordering::Acquire) {
                return Err(MathError::Cancelled);
            }
            let binding = &self.structure.instances()[instance];
            let outputs = outputs.into_iter().collect::<Vec<_>>();
            let demand = LocalDemand {
                body: binding.body,
                outputs,
                coordinates: formal.into_iter().collect(),
                order: DerivativeOrder::First,
                product: ProductKind::SelectedPartials,
            };
            let request =
                if let Some(index) = requests.iter().position(|request| request == &demand) {
                    index
                } else {
                    let support = Arc::new(self.bodies[&binding.body].prepare_support(
                        &demand.outputs,
                        &demand.coordinates,
                        demand.order,
                        cancel,
                    )?);
                    let index = requests.len();
                    requests.push(demand);
                    supports.push(support.clone());
                    numerical.push(support);
                    index
                };
            let layout = crate::execution::FirstLayout::from_support(&numerical[request]);
            let group = groups.len();
            for (row, output) in requests[request].outputs.iter().enumerate() {
                if let Some(formal) = formal
                    && let Some(offset) = layout.offset(row, 0)
                {
                    offsets.insert((instance, formal, *output), (group, offset));
                }
            }
            groups.push(EntryGroup { instance, request });
        }
        let terms = pending
            .into_iter()
            .map(|(instance, formal, output, destination, scale)| {
                let &(group, offset) = offsets
                    .get(&(instance, formal, output))
                    .ok_or_else(|| MathError::Contract("missing selected entry address".into()))?;
                Ok(EntryTerm {
                    group,
                    offset,
                    destination,
                    scale,
                })
            })
            .collect::<Result<Vec<_>, MathError>>()?;
        let provider_visits = groups
            .iter()
            .try_fold(0usize, |count, group| {
                let mut visits = 0usize;
                numerical[group.request].visit_provider_sources(|_, _| {
                    visits = visits.saturating_add(1);
                });
                count.checked_add(visits)
            })
            .ok_or(MathError::Limit("selected provider source population"))?;
        // Cover the source set and collected projection while both are live.
        crate::construction::additional(
            provider_visits
                .checked_mul(512)
                .ok_or(MathError::Limit("selected provider source metadata"))?,
        )?;
        let mut providers = BTreeSet::new();
        for group in &groups {
            numerical[group.request].visit_provider_sources(|source, key| {
                providers.insert((
                    self.structure.instances()[group.instance].instance,
                    source,
                    key,
                ));
            });
        }
        result.entries = Some(Arc::new(EntryPlan {
            pairs: pairs.to_vec(),
            groups,
            terms,
            providers: providers.into_iter().collect(),
            provider_visits,
        }));
        result.numerical_supports = Some(Arc::new(numerical));
        result.requests = Arc::new(requests);
        result.supports = Arc::new(supports);
        result.ownership = Arc::new(PlanOwnership::new(
            &result.structure,
            &result.instances,
            result.requests.len(),
        )?);
        Ok(result)
    }
    pub(super) fn refresh_numerical_supports(&mut self) -> Result<(), MathError> {
        if self.numerical_supports.is_some() {
            self.numerical_supports = Some(Arc::new(
                self.supports
                    .iter()
                    .zip(self.requests.iter())
                    .map(|(support, request)| support.at_order(request.order).map(Arc::new))
                    .collect::<Result<_, _>>()?,
            ));
        }
        Ok(())
    }
}
impl CaseAssembly {
    pub(super) fn repeated_entry_providers(
        plan: &CasePlan,
        programs: &[Arc<CompiledBody>],
    ) -> Vec<RepeatedProvider> {
        let mut sources = BTreeMap::<_, (usize, bool)>::new();
        for group in plan.entries.iter().flat_map(|entries| &entries.groups) {
            let mut local = BTreeMap::<_, usize>::new();
            programs[group.request].visit_provider_sources(
                DerivativeOrder::First,
                |source, key| {
                    *local.entry((group.instance, source, key)).or_default() += 1;
                },
            );
            for (source, visits) in local {
                let entry = sources.entry(source).or_insert((0, false));
                entry.0 += 1;
                entry.1 |= visits > 1;
            }
        }
        sources
            .into_iter()
            .filter_map(|((instance, source, key), (groups, ambiguous))| {
                (groups > 1).then_some(RepeatedProvider {
                    instance,
                    source,
                    key,
                    ambiguous,
                })
            })
            .collect()
    }
}
impl CaseWorker {
    pub(super) fn check_entry_providers(&self) -> Result<(), MathError> {
        for requirement in &self.assembly.repeated_entry_providers {
            if requirement.ambiguous || !self.providers.supports_linearization(requirement.key) {
                return Err(MathError::Scope(pse_kernels::ProviderError::Contract(
                    "repeated selected-entry provider requires unambiguous complete linearization support".into())));
            }
        }
        Ok(())
    }
    /// Fill only the immutable prepared semantic entries. Failure never publishes
    /// partial output or promotes selected completion to full Jacobian coverage.
    pub fn jacobian_entries<'a>(
        &mut self,
        values: impl Into<EvalInputs<'a>>,
        output: &mut [f64],
    ) -> Result<(), MathError> {
        let values = values.into();
        let result = (|| {
            self.scope.check().map_err(crate::error::scope_error)?;
            values.check(&self.assembly)?;
            self.check_entry_providers()?;
            let entries =
                self.assembly.entries.as_ref().ok_or_else(|| {
                    MathError::Contract("Jacobian entries were not prepared".into())
                })?;
            if output.len() != entries.pairs.len() {
                return Err(MathError::Contract("selected entry output extent".into()));
            }
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("math.selected_entry_calls", "calls", 1);
            }
            self.entries.visible = false;
            self.entries.values.fill(0.);
            self.entries.contexts.fill(None);
            for (index, group) in entries.groups.iter().enumerate() {
                let binding = &self.assembly.structure.instances()[group.instance];
                let context = |cause| MathError::Instance {
                    instance: binding.instance,
                    occurrences: binding.occurrences.as_ref().clone(),
                    checked_sources: binding.checked_sources.as_ref().clone(),
                    checked_members: binding.checked_members.as_ref().clone(),
                    cause: Box::new(cause),
                };
                values
                    .fill(
                        &self.assembly.entry_bindings[index],
                        &mut self.entries.inputs[index],
                    )
                    .map_err(context)?;
                self.workers[group.request]
                    .evaluate_trial_into(
                        &self.entries.inputs[index],
                        DerivativeOrder::First,
                        &mut self.providers,
                        &self.cancel,
                        &mut self.entries.trials[index],
                    )
                    .map_err(context)?;
                if let Some(context) = pse_diagnostics::observation::current() {
                    context.count("math.selected_entry_groups", "evaluations", 1);
                }
                self.scope.check().map_err(crate::error::scope_error)?;
                for visit in self.entries.trials[index].provider_visits() {
                    if let Some((position, _)) = self
                        .assembly
                        .repeated_entry_providers
                        .iter()
                        .enumerate()
                        .find(|(_, requirement)| {
                            requirement.instance == group.instance
                                && requirement.source == visit.source
                                && requirement.key == visit.provider
                        })
                    {
                        let token = visit.linearization.ok_or_else(|| {
                            MathError::Scope(pse_kernels::ProviderError::Contract(
                                "current selected-entry provider context unavailable".into(),
                            ))
                        })?;
                        if self.entries.contexts[position].is_some_and(|previous| previous != token)
                        {
                            return Err(MathError::Scope(
                                pse_kernels::ProviderError::LinearizationChanged,
                            ));
                        }
                        self.entries.contexts[position] = Some(token);
                    }
                }
            }
            for term in &entries.terms {
                let value = self.entries.trials[term.group].first[term.offset] * term.scale;
                finite(value)?;
                self.entries.values[term.destination] += value;
            }
            for value in &self.entries.values {
                finite(*value)?;
            }
            self.scope.check().map_err(crate::error::scope_error)?;
            output.copy_from_slice(&self.entries.values);
            self.entries.visible = true;
            Ok(())
        })();
        if let Err(cause) = &result {
            self.entries.clear();
            self.refuse_trial(cause);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug)]
    struct EntryProvider {
        qualified: bool,
        selected: bool,
        drift: bool,
        missing: bool,
        calls: Arc<std::sync::atomic::AtomicUsize>,
    }
    impl pse_kernels::Provider for EntryProvider {
        fn supports(&self, demand: &pse_kernels::ProviderDemand) -> bool {
            matches!(demand, pse_kernels::ProviderDemand::CompleteJets)
                || (self.selected
                    && matches!(demand, pse_kernels::ProviderDemand::SelectedColumns { .. }))
        }
        fn supports_linearization(&self) -> bool {
            self.qualified
        }
        fn completed_linearization(&self) -> Option<ContentHash> {
            (!self.missing).then(|| {
                ContentHash::from_bytes(
                    [if self.drift {
                        self.calls.load(Ordering::Acquire) as u8
                    } else {
                        1
                    }; 32],
                )
            })
        }
        fn fill(
            &mut self,
            x: &[f64],
            request: &pse_kernels::PreparedProviderRequest,
            _: pse_kernels::ProviderRuntime<'_>,
            output: pse_kernels::ProviderOutput<'_>,
            _: &pse_kernels::EvaluationContext<'_>,
        ) -> Result<(), pse_kernels::ProviderError> {
            self.calls.fetch_add(1, Ordering::AcqRel);
            let columns = match &request.demand {
                pse_kernels::ProviderDemand::SelectedColumns { columns } => columns.clone(),
                _ => vec![0, 1],
            };
            for (row, &original) in request.outputs.iter().enumerate() {
                output.values[row] = if original == 0 {
                    x[0] * x[0] + x[1]
                } else {
                    x[0] + x[1] * x[1]
                };
                if request.order >= DerivativeOrder::First {
                    for (axis, &column) in columns.iter().enumerate() {
                        output.jacobian[row * columns.len() + axis] = match (original, column) {
                            (0, 0) => 2. * x[0],
                            (1, 1) => 2. * x[1],
                            _ => 1.,
                        };
                    }
                }
            }
            Ok(())
        }
    }
    #[test]
    fn repeated_entry_provider_requires_current_complete_context_but_single_group_keeps_eligibility()
     {
        crate::initialize().unwrap();
        use crate::binding::{
            CaseLimits, Contribution, InstanceBinding, Row, SlotBinding, Variable,
        };
        let id = |n| SemanticId::from_bytes([n; 16]);
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let quantity = registry.neutral_dimensionless().unwrap();
        let port = |n| pse_kernels::Port {
            id: id(n),
            quantity,
            unit: registry.quantity_type(quantity).unwrap().canonical_unit,
        };
        let spec = pse_kernels::ProviderSpec {
            id: id(30),
            revision: ContentHash::from_bytes([30; 32]),
            data: ContentHash::from_bytes([31; 32]),
            inputs: vec![port(1), port(2)],
            outputs: vec![port(3), port(4)],
            shapes: Default::default(),
            derivative_source: pse_kernels::DerivativeSource::Analytic,
            derivatives: DerivativeOrder::First,
            smoothness: DerivativeOrder::First,
        };
        let mut body = PreparedBody::new(
            2,
            4,
            vec![2, 3],
            vec![crate::guarded::Stage::Provider {
                spec: spec.clone(),
                partial: vec![],
                inputs: vec![0, 1],
                outputs: vec![2, 3],
                source: id(31),
            }],
            DerivativeOrder::First,
        )
        .unwrap();
        body.set_quantities(vec![Some(quantity); 2], vec![quantity; 2]);
        let key = ContentHash::from_bytes([32; 32]);
        let structure = Arc::new(
            CaseStructure::new(
                (1..3)
                    .map(|n| Variable {
                        port: port(n),
                        fixed: false,
                        domain: pse_model::generated::enums::ModelingVariableDomain::Continuous,
                        lower: None,
                        upper: None,
                    })
                    .collect(),
                vec![],
                vec![InstanceBinding {
                    instance: id(20),
                    body: key,
                    occurrences: Default::default(),
                    checked_sources: Default::default(),
                    checked_members: Default::default(),
                    slots: (1..3)
                        .map(|n| SlotBinding::new(&port(n), &port(n), &registry).unwrap())
                        .collect::<Vec<_>>()
                        .into(),
                    contributions: (0..2)
                        .map(|output| Contribution {
                            output,
                            target: Target::Row(id(10 + output as u8)),
                            scale: 1.,
                        })
                        .collect(),
                }],
                (10..12)
                    .map(|n| Row {
                        id: id(n),
                        quantity,
                        lower: 0.,
                        upper: 0.,
                    })
                    .collect(),
                None,
                CaseLimits::default(),
            )
            .unwrap(),
        );
        let control = pse_diagnostics::Cancellation::default();
        let scope = pse_kernels::ExecutionScope::new(control.clone(), None);
        let full = CasePlan::prepare(
            structure,
            BTreeMap::from([(key, Arc::new(body))]),
            &registry,
            DerivativeOrder::First,
            AssemblyLimits::default(),
            &control.flag(),
        )
        .unwrap();
        let pairs = [(id(10), id(1)), (id(11), id(2))];
        let selected = full.with_jacobian_entries(&pairs, &control.flag()).unwrap();
        assert_eq!(
            selected.jacobian_entry_provider_sources(),
            [(id(20), id(31), spec.key())]
        );
        let selected = Arc::new(
            Arc::new(selected)
                .compile(Optimization::default(), EvaluationLimits::default(), &scope)
                .unwrap(),
        );
        assert_eq!(selected.repeated_entry_providers.len(), 1);
        let values = CaseValues {
            scalars: BTreeMap::from([(id(1), 2.), (id(2), 3.)]),
        };
        let bank = |qualified,
                    supports_selected,
                    drift,
                    missing,
                    calls: Arc<std::sync::atomic::AtomicUsize>| {
            pse_kernels::ProviderBindings::from([(
                spec.key(),
                pse_kernels::ProviderBinding::new(
                    spec.clone(),
                    Box::new(EntryProvider {
                        qualified,
                        selected: supports_selected,
                        drift,
                        missing,
                        calls,
                    }),
                ),
            )])
        };
        for (qualified, selected_support, drift, missing) in [
            (true, true, false, false),
            (false, true, false, false),
            (true, false, false, false),
            (true, true, true, false),
            (true, true, false, true),
        ] {
            let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let mut worker = selected.worker(
                bank(qualified, selected_support, drift, missing, calls.clone()),
                control.clone(),
            );
            let prepared = worker.prepare_providers();
            let mut output = [991., 992.];
            if !qualified || !selected_support {
                assert!(prepared.is_err());
                assert_eq!(calls.load(Ordering::Acquire), 0);
            } else {
                prepared.unwrap();
                let result = worker.jacobian_entries(&values, &mut output);
                if drift || missing {
                    assert!(result.is_err());
                    assert_eq!(output, [991., 992.]);
                } else {
                    result.unwrap();
                    assert_eq!(output, [4., 6.]);
                }
            }
        }
        let one = full
            .with_jacobian_entries(&pairs[..1], &control.flag())
            .unwrap();
        let one = Arc::new(
            Arc::new(one)
                .compile(Optimization::default(), EvaluationLimits::default(), &scope)
                .unwrap(),
        );
        assert!(one.repeated_entry_providers.is_empty());
        let mut worker = one.worker(
            bank(false, true, false, false, Arc::default()),
            control.clone(),
        );
        worker.prepare_providers().unwrap();
        let mut output = [999.];
        worker.jacobian_entries(&values, &mut output).unwrap();
        assert_eq!(output, [4.]);
        let full = Arc::new(
            Arc::new(full)
                .compile(Optimization::default(), EvaluationLimits::default(), &scope)
                .unwrap(),
        );
        let mut worker = full.worker(bank(false, false, false, false, Arc::default()), control);
        worker.prepare_providers().unwrap();
        assert_eq!(worker.jacobian(&values).unwrap().val(), [4., 1., 1., 6.]);
    }
    #[test]
    fn selected_entries_keep_physical_gathers_and_first_guards_for_structural_absence() {
        for sqrt in [false, true] {
            let (source, mut values, control) = instance_reuse_tests::fixture(sqrt);
            let rows = source.structure().rows();
            let columns = source.columns();
            let pairs = [
                (rows[1].id, columns[1]),
                (rows[0].id, columns[0]),
                (rows[0].id, columns[1]),
            ];
            let selected = source
                .plan
                .with_jacobian_entries(&pairs, &control.flag())
                .unwrap();
            let scope = pse_kernels::ExecutionScope::new(control.clone(), None);
            let assembly = Arc::new(
                Arc::new(selected)
                    .compile(Optimization::default(), EvaluationLimits::default(), &scope)
                    .unwrap(),
            );
            let mut worker = assembly.worker(Default::default(), control);
            let mut output = vec![123.; 3];
            worker.jacobian_entries(&values, &mut output).unwrap();
            if sqrt {
                let expected = 0.5 / 3_f64.sqrt();
                assert!((output[0] - expected).abs() < 1e-15);
                assert!((output[1] - 3. * expected).abs() < 1e-15);
                assert_eq!(output[2], 0.);
            } else {
                assert_eq!(output, [6., 18., 0.]);
            }
            values.scalars.insert(columns[0], -1.);
            let intact = output.clone();
            if sqrt {
                assert!(worker.jacobian_entries(&values, &mut output).is_err());
                assert_eq!(output, intact);
                // Ask only for a structurally absent derivative: the source's
                // original guard is still executed under its First request.
                let absent = source
                    .plan
                    .with_jacobian_entries(&[pairs[2]], scope.cancellation())
                    .unwrap();
                let absent = Arc::new(
                    Arc::new(absent)
                        .compile(Optimization::default(), EvaluationLimits::default(), &scope)
                        .unwrap(),
                );
                let mut absent_worker =
                    absent.worker(Default::default(), pse_diagnostics::Cancellation::default());
                let mut zero = [987.];
                assert!(absent_worker.jacobian_entries(&values, &mut zero).is_err());
                assert_eq!(zero, [987.]);
            } else {
                worker.jacobian_entries(&values, &mut output).unwrap();
                assert_eq!(output, [6., -18., 0.]);
            }
        }
    }
    #[test]
    fn exact_entries_preserve_aliases_duplicates_reordering_and_structural_zero_without_full_coverage()
     {
        for alias in [false, true] {
            let (source, values) = crate::assembly_tests::fixture(alias, false);
            let control = pse_diagnostics::Cancellation::default();
            let scope = pse_kernels::ExecutionScope::new(control.clone(), None);
            let rows = source
                .structure()
                .rows()
                .iter()
                .map(|row| row.id)
                .collect::<Vec<_>>();
            let coordinates = source.columns().iter().rev().copied().collect::<Vec<_>>();
            let plan = CasePlan::prepare_functions(
                source.structure(),
                |body| source.bodies().get(&body).cloned(),
                &rows,
                coordinates.clone(),
                &pse_quantity::standard::standard_registry().unwrap(),
                DerivativeOrder::First,
                source.limits(),
                &control.flag(),
            )
            .unwrap()
            .with_directional_actions(&control.flag())
            .unwrap();
            let full = Arc::new(
                Arc::new(plan.clone())
                    .compile(Optimization::default(), EvaluationLimits::default(), &scope)
                    .unwrap(),
            );
            let mut full_worker = full.worker(Default::default(), control.clone());
            let matrix = full_worker.jacobian(&values).unwrap();
            let pairs = rows
                .iter()
                .rev()
                .flat_map(|&row| coordinates.iter().map(move |&column| (row, column)))
                .chain([(rows[1], coordinates[0])])
                .collect::<Vec<_>>();
            let expected = pairs
                .iter()
                .map(|(row, column)| {
                    let row = rows.iter().position(|id| id == row).unwrap();
                    let column = coordinates.iter().position(|id| id == column).unwrap();
                    (matrix.col_ptr()[column]..matrix.col_ptr()[column + 1])
                        .find(|&entry| matrix.row_idx()[entry] == row)
                        .map_or(0., |entry| matrix.val()[entry])
                })
                .collect::<Vec<_>>();
            let selected = plan.with_jacobian_entries(&pairs, &control.flag()).unwrap();
            assert_eq!(selected.order(), DerivativeOrder::Value);
            assert_eq!(
                selected.jacobian_pattern().col_ptr(),
                plan.jacobian_pattern().col_ptr()
            );
            assert_eq!(
                selected.jacobian_pattern().row_idx(),
                plan.jacobian_pattern().row_idx()
            );
            for instance in selected.instances.iter() {
                for demand in [EvaluationDemand::All, EvaluationDemand::Constraints] {
                    if let Some(group) = &instance.groups[demand] {
                        assert_eq!(
                            selected.requests[group.request].order,
                            DerivativeOrder::Value
                        );
                        assert_eq!(
                            selected.supports[group.request].order(),
                            DerivativeOrder::First
                        );
                    }
                }
            }
            let assembly = Arc::new(
                Arc::new(selected.clone())
                    .compile(Optimization::default(), EvaluationLimits::default(), &scope)
                    .unwrap(),
            );
            assert_eq!(assembly.jacobian_entries(), pairs);
            let mut worker = assembly.worker(Default::default(), control.clone());
            assert!(
                worker.jacobian.is_none(),
                "no numerical full CSC is allocated for selected-only coverage"
            );
            let mut actual = vec![999.; pairs.len()];
            worker.jacobian_entries(&values, &mut actual).unwrap();
            assert_eq!(actual, expected);
            assert!(worker.jacobian(&values).is_err());
            assert!(worker.entries.visible);
            let mut direct = vec![0.; rows.len()];
            let direction = vec![2.; coordinates.len()];
            worker
                .jacobian_product(&values, &direction, &mut direct)
                .unwrap();
            let mut full_direct = vec![0.; rows.len()];
            full_worker
                .jacobian_product(&values, &direction, &mut full_direct)
                .unwrap();
            assert_eq!(direct, full_direct);
            let bound = assembly.bind_values(&values).unwrap();
            let point = coordinates
                .iter()
                .map(|id| values.scalars[id])
                .collect::<Vec<_>>();
            worker
                .jacobian_entries(bound.trial(&point).unwrap(), &mut actual)
                .unwrap();
            assert_eq!(actual, expected);
            let mut invalid = values.clone();
            invalid.scalars.insert(
                source.structure().instances()[0].slots[0].source(),
                f64::NAN,
            );
            assert!(worker.jacobian_entries(&invalid, &mut actual).is_err());
            assert_eq!(
                actual, expected,
                "failed stronger work preserves caller output"
            );
            worker.jacobian_entries(&values, &mut actual).unwrap();
            control.cancel();
            assert!(worker.jacobian_entries(&values, &mut actual).is_err());
            assert_eq!(actual, expected);
            control.flag().store(false, Ordering::Release);
            let restored = selected
                .prepare_order(DerivativeOrder::First, &control.flag())
                .unwrap();
            assert_eq!(restored.order(), DerivativeOrder::First);
            let restored = Arc::new(
                Arc::new(restored)
                    .compile(Optimization::default(), EvaluationLimits::default(), &scope)
                    .unwrap(),
            );
            assert_eq!(
                restored
                    .worker(Default::default(), control.clone())
                    .jacobian(&values)
                    .unwrap()
                    .val(),
                full_worker.jacobian(&values).unwrap().val()
            );
            assert!(
                plan.with_jacobian_entries(&[(SemanticId::NIL, coordinates[0])], &control.flag())
                    .is_err()
            );
        }
    }
}
