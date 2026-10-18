// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Prepared compiler dependency witnesses and finite issued-iteration permission.
use super::*;

#[derive(Clone, Debug)]
pub(in crate::math) struct Dependencies {
    predecessors: Vec<usize>,
    successors: Vec<Vec<usize>>,
    consumed: Vec<Vec<SemanticId>>,
    #[cfg(test)]
    completion: Option<Arc<CompletionFence>>,
}
impl Dependencies {
    pub(in crate::math) fn allocation_bound<'a>(
        blocks: impl Iterator<
            Item = Result<
                (
                    &'a pse_structural::initialization::Block,
                    &'a pse_math::assembly::CasePlan,
                    &'a [SemanticId],
                ),
                native::ProblemError,
            >,
        >,
    ) -> Result<usize, native::ProblemError> {
        let mut total = 4096usize;
        for block in blocks {
            let (block, plan, inputs) = block?;
            total = (|| {
                let slots = plan
                    .structure()
                    .instances()
                    .iter()
                    .try_fold(0usize, |n, instance| n.checked_add(instance.slots.len()))?;
                let ids = slots
                    .checked_add(inputs.len())?
                    .checked_add(plan.columns().len())?
                    .checked_add(plan.structure().source_frozen_ids().count())?
                    .checked_add(block.inputs.len())?;
                total.checked_add(ids.checked_mul(128)?)?.checked_add(1024)
            })()
            .ok_or_else(|| native::ProblemError::memory("block dependency descriptor extent"))?;
        }
        Ok(total)
    }
    pub(in crate::math) fn prepare<'a>(
        blocks: impl Iterator<Item = &'a pse_structural::initialization::Block>,
        consumed: Vec<Vec<SemanticId>>,
    ) -> Result<Self, native::ProblemError> {
        let blocks: Vec<_> = blocks.collect();
        if consumed.len() != blocks.len() {
            return Err(native::ProblemError::Contract(
                "block binding inventory".into(),
            ));
        }
        let mut producers = BTreeMap::new();
        for (index, block) in blocks.iter().enumerate() {
            for column in &block.members.columns {
                if producers.insert(*column, index).is_some() {
                    return Err(native::ProblemError::Contract(
                        "overlapping block output coordinates".into(),
                    ));
                }
            }
        }
        let mut predecessors = vec![0; blocks.len()];
        let mut successors = vec![Vec::new(); blocks.len()];
        for (index, block) in blocks.iter().enumerate() {
            // Several consumed coordinates from one producer are one prerequisite.
            let parents: BTreeSet<_> = block
                .inputs
                .iter()
                .filter_map(|input| producers.get(input).copied())
                .filter(|&parent| parent != index)
                .collect();
            predecessors[index] = parents.len();
            for parent in parents {
                successors[parent].push(index);
            }
        }
        let result = Self {
            predecessors,
            successors,
            consumed,
            #[cfg(test)]
            completion: None,
        };
        let mut check = result.start();
        while let Some(index) = check.take() {
            check.commit(index)?;
        }
        if check.remaining != 0 {
            return Err(native::ProblemError::Contract(
                "incomplete or cyclic block dependency witness".into(),
            ));
        }
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("math.block_dependency_preparations", "count", 1);
        }
        Ok(result)
    }
    #[cfg(test)]
    pub(in crate::math) fn independent_chain(&self) -> (usize, usize, usize) {
        let fast = (0..self.predecessors.len())
            .find(|&i| {
                self.predecessors[i] == 0
                    && self.successors[i]
                        .iter()
                        .any(|&j| self.predecessors[j] == 1)
            })
            .expect("fixture fast predecessor");
        let child = *self.successors[fast]
            .iter()
            .find(|&&j| self.predecessors[j] == 1)
            .expect("fixture child");
        let slow = (0..self.predecessors.len())
            .find(|&i| i != fast && self.predecessors[i] == 0)
            .expect("fixture independent sibling");
        (fast, slow, child)
    }
    #[cfg(test)]
    pub(in crate::math) fn fence(
        &mut self,
        slow: usize,
    ) -> (Arc<tokio::sync::Notify>, tokio::sync::mpsc::Receiver<usize>) {
        let (events, received) = tokio::sync::mpsc::channel(self.predecessors.len().max(1));
        let release = Arc::new(tokio::sync::Notify::new());
        self.completion = Some(Arc::new(CompletionFence {
            slow,
            release: release.clone(),
            events,
        }));
        (release, received)
    }
    #[cfg(test)]
    pub(in crate::math) async fn delivered(
        &self,
        index: usize,
        scope: &pse_kernels::ExecutionScope,
        cancel: &crate::CancelSource,
    ) {
        if let Some(fence) = &self.completion {
            if index == fence.slow {
                tokio::select! {
                    () = fence.release.notified() => {},
                    () = cancel.cancelled() => {},
                    () = scope.cancellation_control().cancelled() => {},
                    () = async { match scope.deadline() { Some(deadline) => tokio::time::sleep_until(deadline.into()).await, None => std::future::pending().await } } => {},
                }
            }
            let _ = fence.events.send(index).await;
        }
    }
    pub(in crate::math) fn start(&self) -> Readiness<'_> {
        Readiness {
            graph: self,
            predecessors: self.predecessors.clone(),
            ready: self
                .predecessors
                .iter()
                .enumerate()
                .filter_map(|(i, &n)| (n == 0).then_some(i))
                .collect(),
            committed: vec![false; self.predecessors.len()],
            remaining: self.predecessors.len(),
        }
    }
    pub(in crate::math) fn binding_bytes(&self, index: usize) -> usize {
        self.consumed[index]
            .len()
            .saturating_mul(96)
            .saturating_add(256)
    }
    pub(in crate::math) fn bindings_match(
        &self,
        index: usize,
        inputs: &CaseValues,
        current: &CaseValues,
    ) -> Result<bool, native::ProblemError> {
        let mut matches = true;
        for id in &self.consumed[index] {
            let value = current.scalars.get(id).ok_or_else(|| {
                native::ProblemError::Contract(format!("missing conditional binding {id}"))
            })?;
            matches &= inputs
                .scalars
                .get(id)
                .is_some_and(|bound| bound.to_bits() == value.to_bits());
        }
        Ok(matches)
    }
    /// Refresh an exclusive request-local base after all previous child aliases drain.
    /// A nonexclusive base must be replaced by a newly admitted compact allocation.
    pub(in crate::math) fn refresh_inputs(
        &self,
        index: usize,
        inputs: &mut pse_math::SharedAllocation<CaseValues>,
        current: &CaseValues,
    ) -> Result<bool, native::ProblemError> {
        if self.bindings_match(index, inputs, current)? {
            return Ok(true);
        }
        let Some(inputs) = inputs.get_mut() else {
            return Ok(false);
        };
        let mut copied = 0;
        for id in &self.consumed[index] {
            let value = current.scalars[id];
            let bound = inputs.scalars.get_mut(id).ok_or_else(|| {
                native::ProblemError::Contract(format!("missing prepared conditional binding {id}"))
            })?;
            if bound.to_bits() != value.to_bits() {
                *bound = value;
                copied += 1;
            }
        }
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("math.block_bound_input_copies", "count", copied);
        }
        Ok(true)
    }
    pub(in crate::math) fn inputs(
        &self,
        index: usize,
        values: &CaseValues,
    ) -> Result<CaseValues, native::ProblemError> {
        let scalars = self.consumed[index]
            .iter()
            .map(|id| {
                values
                    .scalars
                    .get(id)
                    .copied()
                    .map(|value| (*id, value))
                    .ok_or_else(|| {
                        native::ProblemError::Contract(format!("missing conditional binding {id}"))
                    })
            })
            .collect::<Result<_, _>>()?;
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count(
                "math.block_bound_input_copies",
                "count",
                self.consumed[index].len() as u64,
            );
        }
        Ok(CaseValues { scalars })
    }
}
#[cfg(test)]
#[derive(Debug)]
struct CompletionFence {
    slow: usize,
    release: Arc<tokio::sync::Notify>,
    events: tokio::sync::mpsc::Sender<usize>,
}
pub(in crate::math) struct Readiness<'a> {
    graph: &'a Dependencies,
    predecessors: Vec<usize>,
    ready: BTreeSet<usize>,
    committed: Vec<bool>,
    pub(in crate::math) remaining: usize,
}
impl Readiness<'_> {
    pub(in crate::math) fn take(&mut self) -> Option<usize> {
        self.ready.pop_first()
    }
    pub(in crate::math) fn defer(&mut self, index: usize) {
        self.ready.insert(index);
    }
    pub(in crate::math) fn commit(&mut self, index: usize) -> Result<(), native::ProblemError> {
        if self.committed[index] || self.predecessors[index] != 0 {
            return Err(native::ProblemError::Contract(
                "duplicate or premature block commit".into(),
            ));
        }
        self.committed[index] = true;
        self.remaining -= 1;
        for &next in &self.graph.successors[index] {
            self.predecessors[next] -= 1;
            if self.predecessors[next] == 0 {
                self.ready.insert(next);
            }
        }
        Ok(())
    }
}
/// Issued children own their complete allowance before dispatch. Only known actual
/// work returns unused permission; a failed/unknown child consumes its whole grant.
pub(in crate::math) struct Iterations {
    available: u32,
    issued: BTreeMap<usize, u32>,
}
impl Iterations {
    pub(in crate::math) fn new(available: u32) -> Self {
        Self {
            available,
            issued: BTreeMap::new(),
        }
    }
    pub(in crate::math) fn reserve(
        &mut self,
        index: usize,
        cap: u32,
        slots: usize,
    ) -> Result<u32, native::ProblemError> {
        let fair = self.available
            / u32::try_from(slots.max(1))
                .map_err(|_| native::ProblemError::memory("block reservation width"))?;
        let grant = cap.min(self.available).min(fair.max(1));
        if grant == 0 || self.issued.contains_key(&index) {
            return Err(native::ProblemError::Limit {
                kind: native::LimitKind::Work,
                detail: "aggregate block iteration allowance exhausted".into(),
            });
        }
        self.available -= grant;
        self.issued.insert(index, grant);
        Ok(grant)
    }
    pub(in crate::math) fn complete(
        &mut self,
        index: usize,
        actual: Option<u64>,
    ) -> Result<(), native::ProblemError> {
        let grant = self
            .issued
            .remove(&index)
            .ok_or_else(|| native::ProblemError::Contract("unissued block completion".into()))?;
        let actual = actual
            .and_then(|n| u32::try_from(n).ok())
            .filter(|&n| n <= grant)
            .ok_or_else(|| {
                native::ProblemError::Unsupported(
                    "block aggregate iteration coverage unavailable or exceeded".into(),
                )
            })?;
        self.available += grant - actual;
        Ok(())
    }
    #[cfg(test)]
    pub(in crate::math) fn available(&self) -> u32 {
        self.available
    }
}

/// Frozen binding admission still requires the original parameter/fixed inventory.
/// Local slot sources additionally satisfy the compiler proof projection, which
/// currently reads inactive instance slots. They do not invent execution edges.
/// Remaining inputs come from compiled execution and selected unknowns.
pub(in crate::math) fn consumed_inputs(
    plan: &pse_math::assembly::CasePlan,
    inputs: &[SemanticId],
) -> Vec<SemanticId> {
    inputs
        .iter()
        .copied()
        .chain(plan.columns().iter().copied())
        .chain(
            plan.structure()
                .instances()
                .iter()
                .flat_map(|instance| instance.slots.iter().map(|slot| slot.source())),
        )
        .chain(plan.structure().source_frozen_ids())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_structural::{
        incidence::{BlockId, Part},
        initialization::Block,
    };
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    fn block(n: u8, inputs: Vec<SemanticId>) -> Block {
        Block {
            id: BlockId(pse_ids::ContentHash::from_bytes([n; 32])),
            members: Part {
                rows: vec![id(n + 10)],
                columns: vec![id(n)],
            },
            inputs,
        }
    }
    #[test]
    fn prepared_readiness_unlocks_fast_chain_without_sibling_barrier() {
        let blocks = [
            block(1, vec![]),
            block(2, vec![]),
            block(3, vec![id(1), id(1)]),
            block(4, vec![id(2), id(3)]),
        ];
        let graph = Dependencies::prepare(blocks.iter(), vec![vec![]; 4]).unwrap();
        let mut ready = graph.start();
        assert_eq!(ready.take(), Some(0));
        assert_eq!(ready.take(), Some(1));
        ready.commit(0).unwrap();
        assert_eq!(ready.take(), Some(2));
        ready.commit(2).unwrap();
        assert_eq!(ready.take(), None);
        ready.commit(1).unwrap();
        assert_eq!(ready.take(), Some(3));
        ready.commit(3).unwrap();
        assert_eq!(ready.remaining, 0);
        assert!(ready.commit(0).is_err());
        let cycle = [block(1, vec![id(2)]), block(2, vec![id(1)])];
        assert!(Dependencies::prepare(cycle.iter(), vec![vec![]; 2]).is_err());
    }
    #[test]
    fn binding_map_survives_value_rebind_and_excludes_unconsumed_coordinates() {
        let blocks = [block(1, vec![id(9)])];
        let graph = Dependencies::prepare(blocks.iter(), vec![vec![id(1), id(9)]]).unwrap();
        let mut values = CaseValues {
            scalars: BTreeMap::from([(id(1), 1.0), (id(9), 2.0), (id(8), 99.0)]),
        };
        assert_eq!(graph.inputs(0, &values).unwrap().scalars.len(), 2);
        values.scalars.insert(id(9), 3.0);
        assert_eq!(graph.inputs(0, &values).unwrap().scalars[&id(9)], 3.0);
        values.scalars.remove(&id(9));
        assert!(graph.inputs(0, &values).is_err());
    }
    #[test]
    fn retained_binding_base_updates_only_after_aliases_drain() {
        let blocks = [block(1, vec![id(9)])];
        let graph = Dependencies::prepare(blocks.iter(), vec![vec![id(1), id(9)]]).unwrap();
        let mut current = CaseValues {
            scalars: BTreeMap::from([(id(1), 1.0), (id(9), 2.0), (id(8), 99.0)]),
        };
        let mut inputs: pse_math::SharedAllocation<_> =
            Arc::new(graph.inputs(0, &current).unwrap()).into();
        let identity = inputs.allocation_identity();
        let alias = inputs.clone();
        assert!(graph.refresh_inputs(0, &mut inputs, &current).unwrap());
        current.scalars.insert(id(9), 3.0);
        assert!(!graph.refresh_inputs(0, &mut inputs, &current).unwrap());
        assert_eq!(alias.scalars[&id(9)], 2.0);
        drop(alias);
        assert!(graph.refresh_inputs(0, &mut inputs, &current).unwrap());
        assert_eq!(inputs.allocation_identity(), identity);
        assert_eq!(inputs.scalars[&id(9)], 3.0);
        assert_eq!(inputs.scalars.len(), 2);
        current.scalars.remove(&id(9));
        assert!(graph.refresh_inputs(0, &mut inputs, &current).is_err());
        assert_eq!(inputs.scalars[&id(9)], 3.0);
    }
    #[test]
    fn aggregate_issued_budget_cannot_double_spend_or_refund_unknown_work() {
        let mut budget = Iterations::new(10);
        assert_eq!(budget.reserve(0, 10, 2).unwrap(), 5);
        assert_eq!(budget.reserve(1, 10, 1).unwrap(), 5);
        assert!(budget.reserve(2, 10, 1).is_err());
        budget.complete(0, Some(3)).unwrap();
        assert_eq!(budget.reserve(2, 10, 1).unwrap(), 2);
        assert!(budget.complete(1, None).is_err());
        assert_eq!(budget.available(), 0);
        assert!(budget.complete(2, Some(3)).is_err());
    }
    #[test]
    fn late_dependency_chain_can_spend_last_iteration_with_vacant_lanes() {
        let blocks = [block(1, vec![]), block(2, vec![id(1)])];
        let graph = Dependencies::prepare(blocks.iter(), vec![vec![]; 2]).unwrap();
        let mut ready = graph.start();
        let mut budget = Iterations::new(2);
        assert_eq!(ready.take(), Some(0));
        assert_eq!(budget.reserve(0, 10, 4).unwrap(), 1);
        budget.complete(0, Some(1)).unwrap();
        ready.commit(0).unwrap();
        assert_eq!(budget.available(), 1);
        assert_eq!(ready.take(), Some(1));
        assert_eq!(budget.reserve(1, 10, 4).unwrap(), 1);
        budget.complete(1, Some(1)).unwrap();
        ready.commit(1).unwrap();
        assert_eq!(ready.remaining, 0);
        assert_eq!(budget.available(), 0);
        assert!(budget.reserve(2, 1, 4).is_err());
    }
}
