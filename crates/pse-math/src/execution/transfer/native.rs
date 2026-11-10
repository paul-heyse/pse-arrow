// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Receiver-local realization of checked numerical stages. No symbolic owner,
//! differentiator, expression normalizer or Horner optimizer is involved.
use super::*;

fn block_count(stages: &[CompiledStage]) -> Result<usize> {
    stages.iter().try_fold(0, |count, stage| {
        let extra = match stage {
            CompiledStage::Block { .. } => 1,
            CompiledStage::Domain { stages, .. } | CompiledStage::Applicability { stages, .. } => {
                block_count(stages)?
            }
            CompiledStage::Branch {
                then, otherwise, ..
            } => plus(block_count(then)?, block_count(otherwise)?)?,
            CompiledStage::Require { .. } => 0,
            CompiledStage::Provider { .. } => {
                return Err(TransferError::Unsupported("provider numerical compilation"));
            }
        };
        plus(count, extra)
    })
}

fn packed_worker_extra(stages: &[CompiledStage]) -> Result<usize> {
    stages.iter().try_fold(0usize, |bytes, stage| {
        let extra = match stage {
            CompiledStage::Block {
                evaluator: BlockEvaluator::Interpreted(evaluator),
                ..
            } => evaluator
                .f64_transfer_with_heads(crate::stable::transfer_heads()?)?
                .extents()
                .stack_bytes
                .checked_mul(4)
                .ok_or(TransferError::Overflow("packed native worker"))?,
            CompiledStage::Domain { stages, .. } | CompiledStage::Applicability { stages, .. } => {
                packed_worker_extra(stages)?
            }
            CompiledStage::Branch {
                then, otherwise, ..
            } => plus(packed_worker_extra(then)?, packed_worker_extra(otherwise)?)?,
            CompiledStage::Require { .. } => 0,
            _ => {
                return Err(TransferError::Unsupported(
                    "already compiled or provider numerical program",
                ));
            }
        };
        plus(bytes, extra)
    })
}

fn compile_stages(
    stages: Vec<CompiledStage>,
    batch: &mut crate::native_compiled::Batch,
    module_bytes: usize,
    scope: &pse_kernels::ExecutionScope,
) -> Result<Vec<CompiledStage>> {
    let mut compiled = reserve(stages.len())?;
    for stage in stages {
        scope.check().map_err(crate::error::scope_error)?;
        compiled.push(match stage {
            CompiledStage::Block {
                evaluator,
                resettable,
                components,
                inputs,
                outputs,
                gather,
                scatter,
                arguments,
                values,
                source,
                checks,
                checked_ends,
            } => {
                let original = match evaluator {
                    BlockEvaluator::Interpreted(original) => original,
                    BlockEvaluator::Native(_) => {
                        return Err(TransferError::Invalid(
                            "numerical template already compiled",
                        ));
                    }
                };
                let supplier = original.f64_transfer_with_heads(crate::stable::transfer_heads()?)?;
                let extent = supplier.extents();
                let storage = library::Storage {
                    numeric_entries: extent.stack,
                    instruction_bytes: plus(
                        plus(extent.instruction_bytes, extent.result_bytes)?,
                        extent.external_bytes,
                    )?,
                    instruction_count: extent.instructions,
                };
                let executable = batch.push(&original, &storage, module_bytes, scope)?;
                CompiledStage::Block {
                    evaluator: BlockEvaluator::Native(NativeBlock {
                        canonical: Arc::new(original),
                        executable,
                    }),
                    resettable,
                    components,
                    inputs,
                    outputs,
                    gather,
                    scatter,
                    arguments,
                    values,
                    source,
                    checks,
                    checked_ends,
                }
            }
            mut stage => {
                match &mut stage {
                    CompiledStage::Domain { stages, .. }
                    | CompiledStage::Applicability { stages, .. } => {
                        *stages =
                            compile_stages(std::mem::take(stages), batch, module_bytes, scope)?;
                    }
                    CompiledStage::Branch {
                        then, otherwise, ..
                    } => {
                        *then = compile_stages(std::mem::take(then), batch, module_bytes, scope)?;
                        *otherwise =
                            compile_stages(std::mem::take(otherwise), batch, module_bytes, scope)?;
                    }
                    CompiledStage::Provider { .. } => {
                        return Err(TransferError::Unsupported("provider numerical compilation"));
                    }
                    _ => {}
                }
                stage
            }
        });
    }
    Ok(compiled)
}

impl RestoredExecutionTemplate {
    /// Complete additional backing reserved before copying numerical stages or
    /// exporting source. The existing template remains independently live.
    pub fn scalar_compilation_bytes(&self, module_bytes: usize) -> Result<usize> {
        if module_bytes == 0 {
            return Err(TransferError::Invalid("zero compiled module allowance"));
        }
        let blocks = self
            .template
            .programs
            .values()
            .flatten()
            .try_fold(0, |count, stages| plus(count, block_count(stages)?))?;
        let per_block = plus(module_bytes, arc_bytes::<ExpressionEvaluator<f64>>()?)?;
        let modules = blocks
            .checked_mul(per_block)
            .ok_or(TransferError::Overflow("numerical native modules"))?;
        // An input stage vector and its consuming output vector can overlap;
        // two complete template extents cover this without relying on capacities.
        plus(
            plus(self.facts.template_bytes, self.facts.template_bytes)?,
            modules,
        )
    }

    /// Compile exactly the checked arithmetic stages under the current operation.
    /// Guards, selected layouts and original occurrences remain in the same schedule.
    /// The returned immutable template retains no producer or compiler control.
    pub fn compile_scalar<E: std::fmt::Debug + std::fmt::Display>(
        &self,
        module_bytes: usize,
        scope: &pse_kernels::ExecutionScope,
        admit: impl FnOnce(usize) -> std::result::Result<Arc<dyn AllocationOwner>, E>,
    ) -> std::result::Result<Self, TransferError<E>> {
        self.compile_native(module_bytes, false, scope, admit)
    }

    /// Realize the same portable stages as explicit AVX2 groups plus their
    /// conservative scalar entry. Packing never changes the original guard schedule.
    pub fn compile_x4<E: std::fmt::Debug + std::fmt::Display>(
        &self,
        module_bytes: usize,
        scope: &pse_kernels::ExecutionScope,
        admit: impl FnOnce(usize) -> std::result::Result<Arc<dyn AllocationOwner>, E>,
    ) -> std::result::Result<Self, TransferError<E>> {
        self.compile_native(module_bytes, true, scope, admit)
    }

    fn compile_native<E: std::fmt::Debug + std::fmt::Display>(
        &self,
        module_bytes: usize,
        packed: bool,
        scope: &pse_kernels::ExecutionScope,
        admit: impl FnOnce(usize) -> std::result::Result<Arc<dyn AllocationOwner>, E>,
    ) -> std::result::Result<Self, TransferError<E>> {
        scope.check().map_err(crate::error::scope_error)?;
        let backing = self
            .scalar_compilation_bytes(module_bytes)
            .map_err(widen::<E>)?;
        let mut extras = EnumMap::<DerivativeOrder, usize>::default();
        if packed {
            for (order, stages) in self.template.programs.iter() {
                if let Some(stages) = stages {
                    extras[order] = packed_worker_extra(stages).map_err(widen::<E>)?;
                }
            }
        }
        let extra = extras
            .values()
            .try_fold(0usize, |n, &bytes| plus(n, bytes))
            .map_err(widen::<E>)?;
        let scratch = self
            .template
            .scratch_bytes
            .checked_add(extra)
            .filter(|n| *n <= self.template.limits.scratch_bytes)
            .ok_or(MathError::Limit("packed numerical worker scratch"))?;
        let owner = admit(backing).map_err(TransferError::Admission)?;
        scope.check().map_err(crate::error::scope_error)?;
        let mut programs = self.template.programs.as_ref().clone();
        let mut batch = if packed {
            crate::native_compiled::Batch::packed()?
        } else {
            crate::native_compiled::Batch::default()
        };
        for stages in programs.values_mut().flatten() {
            *stages = compile_stages(std::mem::take(stages), &mut batch, module_bytes, scope)
                .map_err(widen::<E>)?;
        }
        batch.finish(scope)?;
        scope.check().map_err(crate::error::scope_error)?;
        let mut template = self.template.as_ref().clone();
        template.programs = Arc::new(programs);
        template.scratch_bytes = scratch;
        template.worker_bytes = template
            .worker_bytes
            .checked_add(extra)
            .ok_or(TransferError::Overflow("packed numerical worker"))?;
        for (order, census) in template.census.iter_mut() {
            template.order_worker_bytes[order] = template.order_worker_bytes[order]
                .checked_add(extras[order])
                .ok_or(TransferError::Overflow("packed numerical order worker"))?;
            if let Some(census) = census {
                census.native_arithmetic_blocks = census.arithmetic_blocks;
                census.worker_bytes = template.order_worker_bytes[order];
            }
        }
        let mut facts = self.facts.clone();
        facts.worker_bytes = template.worker_bytes;
        facts.template_bytes = facts
            .template_bytes
            .checked_add(backing)
            .ok_or(TransferError::Overflow("numerical native retained backing"))?;
        Ok(Self {
            template: Arc::new(template),
            facts,
            owner: Some(crate::retain_allocation_owner(self.owner.clone(), owner)),
        })
    }
}
