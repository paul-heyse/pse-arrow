// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable preparation, effect-owned compilation and worker-local library evaluation.
use crate::{
    MathError,
    guarded::{Comparison, Condition, Stage, validate_dependencies},
    jets::{EvaluationLimits, JetLayout, LiftInput, ProductKind, ProductValues, ProviderLift},
    library::{self, Optimization},
};
use enum_map::EnumMap;
use pse_ids::SemanticId;
use pse_kernels::{
    DerivativeOrder, EvaluationContext, ProviderKey, ProviderRequest, ProviderSpec,
    PreparedProviderRequest, ProviderBindings, ProviderBuffer, ProviderRuntime,
};
#[cfg(test)]
use pse_kernels::Provider;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::{
        Arc, Weak,
        atomic::{AtomicBool, Ordering},
    },
};
use symbolica::{
    atom::{Atom, AtomCore, Indeterminate, Symbol},
    evaluate::ExpressionEvaluator,
};

#[path = "products.rs"]
mod products;

fn order_name(order: DerivativeOrder) -> &'static str {
    match order {
        DerivativeOrder::Value => "value",
        DerivativeOrder::First => "first",
        DerivativeOrder::Second => "second",
    }
}

/// Conservative complete mathematical support and separate control dependencies.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Support {
    /// Nonzero-possible first derivative input columns for each output.
    pub first: Vec<BTreeSet<usize>>,
    /// Nonzero-possible symmetric second derivative input pairs for each output.
    pub second: Vec<BTreeSet<(usize, usize)>>,
    /// Inputs needed for obligations/branch decisions even when numerical derivatives vanish.
    pub controls: BTreeSet<usize>,
}

/// Immutable demand-indexed mathematical support, separate from the value program.
#[derive(Clone, Debug)]
pub struct PreparedSupport {
    program: Arc<PreparedStages>,
    data: Arc<PreparedSupportData>,
    owner: Option<Arc<dyn crate::AllocationOwner>>,
}
#[derive(Clone, Debug)]
struct PreparedSupportData {
    body: PreparedBody,
    source: Weak<PreparedBodyData>,
    source_inputs: usize,
    outputs: Vec<usize>,
    coordinates: Vec<usize>,
    order: DerivativeOrder,
    support: Support,
    remaining_occurrences: usize,
    derivative_operations: usize,
}
impl PartialEq for PreparedSupport {
    fn eq(&self, other: &Self) -> bool {
        // Allocation leases and construction receipts do not change mathematical demand identity.
        self.data.body == other.data.body
            && self.outputs() == other.outputs()
            && self.coordinates() == other.coordinates()
            && self.order() == other.order()
            && self.support() == other.support()
    }
}
impl PreparedSupport {
    fn from_data(
        data: PreparedSupportData,
        owner: Option<Arc<dyn crate::AllocationOwner>>,
        cancel: &Arc<AtomicBool>,
        retained: Option<Arc<PreparedStages>>,
    ) -> Result<Self, MathError> {
        if cancel.load(Ordering::Acquire) {
            return Err(MathError::Cancelled);
        }
        let program = if let Some(program) = retained {
            program
        } else {
            let parameters = data.body.parameters()?;
            let symbols = symbol_map(&parameters);
            let stages = data
                .body
                .demand(&(0..data.outputs.len()).collect::<Vec<_>>(), &symbols);
            Arc::new(PreparedStages::new(&stages, &parameters, &symbols, cancel)?)
        };
        if cancel.load(Ordering::Acquire) {
            return Err(MathError::Cancelled);
        }
        Ok(Self {
            data: Arc::new(data),
            program,
            owner,
        })
    }
    /// Original body output ordinals; support rows use this ordered map.
    pub fn outputs(&self) -> &[usize] {
        &self.data.outputs
    }
    /// Original formal input positions; never renumbered by selection.
    pub fn coordinates(&self) -> &[usize] {
        &self.data.coordinates
    }
    /// Constructed support ceiling, independent of numerical capability/readiness.
    pub fn order(&self) -> DerivativeOrder {
        self.data.order
    }
    /// Requested all-branch support. Value products have no derivative support rows.
    pub fn support(&self) -> &Support {
        &self.data.support
    }
    /// First support by original body output ordinal, through the explicit selected map.
    pub fn first_for_output(&self, output: usize) -> Option<&BTreeSet<usize>> {
        self.outputs()
            .iter()
            .position(|&i| i == output)
            .and_then(|row| self.support().first.get(row))
    }
    /// Second support by original body output ordinal, through the selected map.
    pub fn second_for_output(&self, output: usize) -> Option<&BTreeSet<(usize, usize)>> {
        self.outputs()
            .iter()
            .position(|&i| i == output)
            .and_then(|row| self.support().second.get(row))
    }
    /// Whether selected programs came from the same admitted original semantic body.
    /// Weak allocation identity survives release; independent live equal admissions also match.
    pub fn same_source(&self, other: &Self) -> bool {
        self.data.source_inputs == other.data.source_inputs
            && (self.data.source.ptr_eq(&other.data.source)
                || self
                    .data
                    .source
                    .upgrade()
                    .zip(other.data.source.upgrade())
                    .is_some_and(|(a, b)| {
                        PreparedBody {
                            data: a,
                            owner: None,
                        } == PreparedBody {
                            data: b,
                            owner: None,
                        }
                    }))
    }
    /// Unspent finite authored construction allowance.
    pub fn remaining_occurrences(&self) -> usize {
        self.data.remaining_occurrences
    }
    /// Library derivative calls performed while constructing this immutable support.
    pub fn derivative_operations(&self) -> usize {
        self.data.derivative_operations
    }
    /// Selected immutable value program; no complete source body is retained.
    fn body(&self) -> &PreparedBody {
        &self.data.body
    }
    /// Match the original admitted body without retaining its allocation or lease.
    /// A separately admitted equal body matches while the source still exists. A dropped
    /// source conservatively prevents reuse rather than inventing semantic identity.
    pub fn matches_body(&self, body: &PreparedBody) -> bool {
        self.data.source.ptr_eq(&Arc::downgrade(&body.data))
            || self
                .data
                .source
                .upgrade()
                .is_some_and(|data| PreparedBody { data, owner: None } == *body)
    }
    /// Authored formal IDs in the compact numerical input signature.
    pub fn input_formals(&self) -> &[usize] {
        &self.body().formal_slots[..self.body().inputs]
    }
    fn dense_coordinates(&self) -> Result<Vec<usize>, MathError> {
        self.coordinates()
            .iter()
            .map(|formal| {
                self.input_formals().binary_search(formal).map_err(|_| {
                    MathError::Contract("selected derivative coordinate absent".into())
                })
            })
            .collect()
    }
    fn check_selection(&self, outputs: &[usize], coordinates: &[usize]) -> Result<(), MathError> {
        if outputs.is_empty()
            || outputs.iter().any(|i| !self.outputs().contains(i))
            || outputs.iter().collect::<BTreeSet<_>>().len() != outputs.len()
            || coordinates.iter().any(|&i| i >= self.data.source_inputs)
            || coordinates.iter().collect::<BTreeSet<_>>().len() != coordinates.len()
        {
            return Err(MathError::Contract(
                "compiled output/coordinate demand".into(),
            ));
        }
        Ok(())
    }
    fn restore_selection(
        &mut self,
        source: Weak<PreparedBodyData>,
        source_inputs: usize,
        outputs: &[usize],
        coordinates: &[usize],
    ) {
        let data = Arc::make_mut(&mut self.data);
        let formals = &data.body.formal_slots;
        for row in &mut data.support.first {
            *row = row.iter().map(|&offset| formals[offset]).collect();
        }
        for row in &mut data.support.second {
            *row = row
                .iter()
                .map(|&(left, right)| {
                    let (left, right) = (formals[left], formals[right]);
                    (left.min(right), left.max(right))
                })
                .collect();
        }
        data.support.controls = data
            .support
            .controls
            .iter()
            .map(|&offset| formals[offset])
            .collect();
        data.source = source;
        data.source_inputs = source_inputs;
        data.outputs = outputs.to_vec();
        data.coordinates = coordinates.to_vec();
    }
    fn reconstruct_support(
        &self,
        coordinates: &[usize],
        order: DerivativeOrder,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self, MathError> {
        self.check_selection(self.outputs(), coordinates)?;
        let body = self.body().compact(
            &(0..self.outputs().len()).collect::<Vec<_>>(),
            coordinates,
            self.data.source_inputs,
        )?;
        let dense = coordinates
            .iter()
            .map(|formal| {
                body.formal_slots.binary_search(formal).map_err(|_| {
                    MathError::Contract("selected derivative coordinate absent".into())
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut support = body.support_with_allowance(
            &(0..self.outputs().len()).collect::<Vec<_>>(),
            &dense,
            order,
            self.remaining_occurrences(),
            cancel,
            (body == *self.body()).then(|| self.program.clone()),
        )?;
        support.restore_selection(
            self.data.source.clone(),
            self.data.source_inputs,
            self.outputs(),
            coordinates,
        );
        support.owner = self.owner.clone();
        Ok(support)
    }
    /// Owned selected-support identity, including its immutable compact value payload.
    pub fn allocation_identity(&self) -> usize {
        Arc::as_ptr(&self.data) as usize
    }
    /// Known owned demand payload, including the selected compact value program.
    pub fn retained_bytes(&self) -> usize {
        size_of::<PreparedSupportData>()
            + self.body().retained_bytes()
            + self.program.retained_bytes()
            + (self.data.outputs.capacity() + self.data.coordinates.capacity()) * size_of::<usize>()
            + (self.data.support.first.capacity() + self.data.support.second.capacity())
                * size_of::<BTreeSet<usize>>()
            + self
                .data
                .support
                .first
                .iter()
                .map(|s| s.len() * (size_of::<usize>() + 96))
                .sum::<usize>()
            + self
                .data
                .support
                .second
                .iter()
                .map(|s| s.len() * (size_of::<(usize, usize)>() + 96))
                .sum::<usize>()
            + self.data.support.controls.len() * (size_of::<usize>() + 96)
    }
    /// Retain the runtime owner's lease with every escaping support clone.
    pub fn with_owner(mut self, owner: Arc<dyn crate::AllocationOwner>) -> Self {
        self.owner = Some(crate::retain_allocation_owner(self.owner.take(), owner));
        self
    }
    /// Add stronger support under the remaining allowance; failure preserves this product.
    pub fn upgrade(
        &self,
        order: DerivativeOrder,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self, MathError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        if order <= self.order() {
            let _reuse = tracing::info_span!(
                "pse.case.support_reuse",
                product = "support",
                derivative_order = order_name(order),
                reused = true
            )
            .entered();
            return Ok(self.clone());
        }
        let support = self.reconstruct_support(self.coordinates(), order, cancel)?;
        if self.order() >= DerivativeOrder::First && support.support().first != self.support().first
        {
            return Err(MathError::Contract(
                "support upgrade changed incidence".into(),
            ));
        }
        Ok(support)
    }
    /// Admit structural First support on explicit original formal coordinates.
    /// Selected outputs and the unspent construction allowance remain authoritative;
    /// changing coordinates constructs new incidence without promoting numerical capability.
    pub fn incidence(
        &self,
        coordinates: &[usize],
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self, MathError> {
        let mut incidence = if coordinates == self.coordinates() {
            self.upgrade(DerivativeOrder::First, cancel)?
        } else {
            self.reconstruct_support(coordinates, DerivativeOrder::First, cancel)?
        };
        incidence.owner = self.owner.clone();
        Ok(incidence)
    }
    /// Narrow a known support product to the actual numerical demand ceiling.
    pub fn at_order(&self, order: DerivativeOrder) -> Result<Self, MathError> {
        if order > self.order() {
            return Err(MathError::Contract(
                "support ceiling exceeds admission".into(),
            ));
        }
        if order == self.order() {
            return Ok(self.clone());
        }
        let mut result = self.clone();
        let data = Arc::make_mut(&mut result.data);
        data.order = order;
        if order < DerivativeOrder::Second {
            data.support.second.clear();
        }
        if order < DerivativeOrder::First {
            data.support.first.clear();
        }
        Ok(result)
    }
    /// Restrict outputs without repeating symbolic support construction.
    pub fn select_outputs(
        &self,
        outputs: &[usize],
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self, MathError> {
        let _phase = pse_diagnostics::observation::current().map(|context| {
            context.count("math.support_output_selections", "count", 1);
            context.phase("math.support_output_selection")
        });
        let construction = tracing::info_span!(
            "pse.case.support_selection",
            product = "support_selection",
            derivative_order = order_name(self.order()),
            success = false
        );
        let _construction = construction.enter();
        self.check_selection(outputs, self.coordinates())?;
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        let rows = outputs
            .iter()
            .map(|output| {
                self.outputs()
                    .iter()
                    .position(|i| i == output)
                    .ok_or_else(|| {
                        MathError::Contract("support output selection is not a subset".into())
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut remaining = self.remaining_occurrences();
        let mut allowance = SupportAllowance {
            remaining: &mut remaining,
            derivatives: 0,
        };
        allowance.consume(
            outputs
                .len()
                .checked_add(self.coordinates().len())
                .ok_or(MathError::Limit("support maps"))?,
            SemanticId::NIL,
        )?;
        let mut support = Support::default();
        if self.order() > DerivativeOrder::Value {
            allowance.consume(self.support().controls.len(), SemanticId::NIL)?;
            support.controls = self.support().controls.clone();
            for &row in &rows {
                allowance.consume(self.support().first[row].len(), SemanticId::NIL)?;
                support.first.push(self.support().first[row].clone());
                if self.order() >= DerivativeOrder::Second {
                    allowance.consume(self.support().second[row].len(), SemanticId::NIL)?;
                    support.second.push(self.support().second[row].clone());
                }
            }
        }
        construction.record("success", true);
        Self::from_data(
            PreparedSupportData {
                body: self
                    .body()
                    .compact(&rows, self.coordinates(), self.data.source_inputs)?,
                source: self.data.source.clone(),
                source_inputs: self.data.source_inputs,
                outputs: outputs.to_vec(),
                coordinates: self.coordinates().to_vec(),
                order: self.order(),
                support,
                remaining_occurrences: remaining,
                derivative_operations: 0,
            },
            self.owner.clone(),
            cancel,
            None,
        )
    }
    /// Mechanical conditional projection of already established derivative support.
    /// No symbolic construction is repeated and no exhausted allowance is reset.
    /// The containing case admits and retains these restricted metadata buffers.
    pub(crate) fn conditional_support(
        &self,
        outputs: &[usize],
        coordinates: &[usize],
        order: DerivativeOrder,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self, MathError> {
        self.check_selection(outputs, coordinates)?;
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        if order < DerivativeOrder::First
            || self.order() < order
            || coordinates
                .iter()
                .any(|coordinate| !self.coordinates().contains(coordinate))
        {
            return Err(MathError::Contract(
                "conditional support is not an established order and coordinate restriction".into(),
            ));
        }
        let first = outputs
            .iter()
            .map(|output| {
                let support = self.first_for_output(*output).ok_or_else(|| {
                    MathError::Contract(
                        "conditional First support output is not established".into(),
                    )
                })?;
                Ok(support
                    .iter()
                    .filter(|coordinate| coordinates.contains(coordinate))
                    .copied()
                    .collect())
            })
            .collect::<Result<Vec<BTreeSet<usize>>, MathError>>()?;
        let second = if order >= DerivativeOrder::Second {
            outputs
                .iter()
                .map(|output| {
                    let support = self.second_for_output(*output).ok_or_else(|| {
                        MathError::Contract(
                            "conditional Second support output is not established".into(),
                        )
                    })?;
                    Ok(support
                        .iter()
                        .filter(|(left, right)| {
                            coordinates.contains(left) && coordinates.contains(right)
                        })
                        .copied()
                        .collect())
                })
                .collect::<Result<Vec<BTreeSet<(usize, usize)>>, MathError>>()?
        } else {
            Vec::new()
        };
        Self::from_data(
            PreparedSupportData {
                body: self.body().compact(
                    &outputs
                        .iter()
                        .map(|output| {
                            self.outputs()
                                .iter()
                                .position(|i| i == output)
                                .ok_or_else(|| {
                                    MathError::Contract("conditional support output absent".into())
                                })
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    coordinates,
                    self.data.source_inputs,
                )?,
                source: self.data.source.clone(),
                source_inputs: self.data.source_inputs,
                outputs: outputs.to_vec(),
                coordinates: coordinates.to_vec(),
                order,
                support: Support {
                    first,
                    second,
                    controls: self
                        .support()
                        .controls
                        .iter()
                        .filter(|coordinate| coordinates.contains(coordinate))
                        .copied()
                        .collect(),
                },
                remaining_occurrences: self.remaining_occurrences(),
                derivative_operations: 0,
            },
            self.owner.clone(),
            cancel,
            None,
        )
    }
    /// Initial selected numeric construction population. Fixed inputs remain value
    /// parameters, while only the demanded coordinates contribute Taylor axes.
    /// Each arithmetic block conservatively uses the compact slot population and
    /// selected coordinate count. This preflight constructs no selection metadata.
    /// Positive optimized excess grows the installed runtime owner before floats.
    pub fn compilation_allocation_bound(
        &self,
        limits: EvaluationLimits,
        product: ProductKind,
        options: Optimization,
    ) -> Result<Option<usize>, MathError> {
        if matches!(
            product,
            ProductKind::WeightedHessian | ProductKind::WeightedHessianAction | ProductKind::TransposeJacobianAction
        ) {
            if self.order() != (if product == ProductKind::TransposeJacobianAction { DerivativeOrder::First } else { DerivativeOrder::Second }) {
                return Err(MathError::Contract(
                    "weighted construction requires Second support".into(),
                ));
            }
            // Temporary flattened expressions and library derivative optimization have
            // no sound source-only exact shape envelope. The installed finite foreign
            // owner admits them and reserves known positive excess before allocation.
            return Ok(None);
        }
        let directional = product == ProductKind::JacobianAction;
        let bound =
            self.selected_compilation_allocation_bound(limits, self.order(), directional)?;
        bound
            .map(|bytes| {
                self.body()
                    .stages
                    .iter()
                    .filter(|stage| matches!(stage, Stage::Block { .. }))
                    .count()
                    .checked_mul(options.evaluator.module_bytes())
                    .and_then(|bytes| {
                        bytes.checked_mul(if directional {
                            1
                        } else {
                            self.order() as usize + 1
                        })
                    })
                    .and_then(|modules| bytes.checked_add(modules))
                    .ok_or(MathError::Limit("compiled module construction storage"))
            })
            .transpose()
    }
    fn selected_compilation_allocation_bound(
        &self,
        limits: EvaluationLimits,
        requested: DerivativeOrder,
        directional: bool,
    ) -> Result<Option<usize>, MathError> {
        if directional && requested != DerivativeOrder::First {
            return Err(MathError::Contract(
                "directional support must admit exactly First".into(),
            ));
        }
        if !self.body().is_flat_arithmetic() {
            return Ok(None);
        }
        let overflow = || MathError::Limit("selected compilation construction extent");
        let add = |a: usize, b: usize| a.checked_add(b).ok_or_else(overflow);
        let mul = |a: usize, b: usize| a.checked_mul(b).ok_or_else(overflow);
        let body = self.body();
        let coordinates = self.coordinates().len();
        let axes = if directional { 1 } else { coordinates };
        // A sum bounds the selected union without allocating a duplicate support map.
        let second_pairs = self
            .support()
            .second
            .iter()
            .try_fold(0usize, |n, row| n.checked_add(row.len()))
            .ok_or_else(overflow)?;
        // Construction admission precedes every coordinate/read/reachability map.
        // The compact slot population and global selected axes upper-bound every
        // block-local selection without allocating those selections in preflight.
        let mut numeric_bytes = 0usize;
        let mut instructions = 0usize;
        let mut descriptors = 0usize;
        for order in [
            DerivativeOrder::Value,
            DerivativeOrder::First,
            DerivativeOrder::Second,
        ]
        .into_iter()
        .filter(|order| *order <= requested && (!directional || *order == DerivativeOrder::First))
        {
            let pair_count = if order >= DerivativeOrder::Second {
                second_pairs
            } else {
                0
            };
            let width = add(
                add(
                    1,
                    if order >= DerivativeOrder::First {
                        axes
                    } else {
                        0
                    },
                )?,
                pair_count,
            )?;
            let output_width = add(
                add(
                    1,
                    if order >= DerivativeOrder::First {
                        axes
                    } else {
                        0
                    },
                )?,
                pair_count,
            )?;
            let mut entries = add(
                mul(body.slots, width)?,
                mul(body.outputs.len(), output_width)?,
            )?;
            // Global layouts persist in the immutable program. Growing shape and
            // pair vectors, plus a construction copy, retain separate headroom.
            descriptors = add(descriptors, construction_layout_shape_bytes(axes, width)?)?;
            for stage in &body.stages {
                let Stage::Block {
                    expressions,
                    outputs,
                    ..
                } = stage
                else {
                    descriptors = add(descriptors, 2 * size_of::<CompiledStage>())?;
                    continue;
                };
                let local_axes = if order > DerivativeOrder::Value {
                    axes
                } else {
                    0
                };
                let local_width = width;
                let products = add(
                    add(
                        1,
                        if order >= DerivativeOrder::First {
                            mul(2, axes)?
                        } else {
                            0
                        },
                    )?,
                    mul(4, pair_count)?,
                )?;
                let block = library::evaluator_construction_shape(
                    expressions,
                    body.slots,
                    order,
                    local_width,
                    products,
                )?;
                entries = add(
                    entries,
                    add(
                        block.entries,
                        mul(add(body.slots, outputs.len())?, local_width)?,
                    )?,
                )?;
                entries = add(entries, local_width)?;
                instructions = add(instructions, block.instructions)?;
                // A block's local layout/zero-component list exists only while it
                // compiles; summing them also covers their temporary coexistence
                // with every prior retained stage and the full outer layout.
                descriptors = add(
                    descriptors,
                    construction_layout_shape_bytes(local_axes, local_width)?,
                )?;
                descriptors = add(
                    descriptors,
                    mul(
                        mul(body.slots, local_width)?,
                        2 * size_of::<(usize, usize)>(),
                    )?,
                )?;
                descriptors = add(
                    descriptors,
                    mul(
                        add(add(body.slots, outputs.len())?, local_width)?,
                        2 * size_of::<usize>(),
                    )?,
                )?;
                descriptors = add(descriptors, 2 * size_of::<CompiledStage>())?;
            }
            numeric_bytes = add(numeric_bytes, mul(entries, size_of::<f64>())?)?;
        }
        // The scientific scratch guard limits the combined known numeric programs,
        // frames and output buffers. Instructions and descriptors are independent.
        numeric_bytes = numeric_bytes.min(limits.scratch_bytes);
        let metadata = add(
            mul(add(body.inputs, coordinates)?, 4 * size_of::<usize>())?,
            mul(mul(body.slots, add(axes, 3)?)?, size_of::<usize>())?,
        )?;
        let bytes = add(
            add(add(mul(numeric_bytes, 2)?, instructions)?, descriptors)?,
            add(add(metadata, mul(self.retained_bytes(), 4)?)?, 4096)?,
        )?;
        Ok(Some(bytes))
    }
    /// Support/order upgrades retain the selected outputs and coordinates. Bound
    /// their requested order and finite intermediate support facts, including the
    /// immutable source and current support beside the new products.
    pub fn support_upgrade_allocation_bound(
        &self,
        limits: EvaluationLimits,
        order: DerivativeOrder,
    ) -> Result<Option<usize>, MathError> {
        let Some(compilation) = self.selected_compilation_allocation_bound(limits, order, false)?
        else {
            return Ok(None);
        };
        let axes = self.coordinates().len();
        let support_width = axes.checked_add(1).and_then(|width| {
            if order >= DerivativeOrder::Second {
                axes.checked_mul(axes)
                    .and_then(|pairs| width.checked_add(pairs))
            } else {
                Some(width)
            }
        });
        let population = self
            .body()
            .slots
            .checked_add(self.body().inputs)
            .and_then(|slots| slots.checked_add(self.outputs().len()))
            .and_then(|slots| slots.checked_mul(support_width?))
            .and_then(|slots| slots.checked_mul(4))
            .ok_or(MathError::Limit("selected support construction extent"))?;
        Ok(Some(
            population
                .min(self.remaining_occurrences())
                .checked_mul(256)
                .and_then(|support| compilation.checked_add(support))
                .ok_or(MathError::Limit("selected support construction extent"))?,
        ))
    }
    /// Prepare the selected original scalar blocks once, before any Taylor conversion.
    /// Actual expression/parameter/optimizer identity is checked inside the bundle.
    pub fn prepare_scalars(
        &self,
        options: Optimization,
        limits: EvaluationLimits,
        scope: &pse_kernels::ExecutionScope,
    ) -> Result<library::ScalarProducts, MathError> {
        scope.check().map_err(crate::error::scope_error)?;
        let _phase = pse_diagnostics::observation::current().map(|context| {
            context.count("math.scalar_preparation_calls", "count", 1);
            context.phase("math.scalar_preparation")
        });
        limits.check()?;
        fn collect(
            program: &PreparedStages,
            options: Optimization,
            limits: EvaluationLimits,
            scope: &pse_kernels::ExecutionScope,
            cache: &mut library::ScalarCache,
        ) -> Result<(), MathError> {
            for prepared in &program.stages {
                scope.check().map_err(crate::error::scope_error)?;
                if let (Stage::Block { expressions, .. }, Some(region)) =
                    (&prepared.stage, &prepared.region)
                {
                    cache.admit_prepare(
                        expressions,
                        &region.parameters,
                        options,
                        scope.cancellation(),
                        limits,
                    )?;
                }
                for child in &prepared.children {
                    collect(child, options, limits, scope, cache)?;
                }
            }
            Ok(())
        }
        let mut cache = library::ScalarCache::default();
        collect(&self.program, options, limits, scope, &mut cache)?;
        scope.check().map_err(crate::error::scope_error)?;
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("math.scalar_preparation_completed", "count", 1);
        }
        Ok(cache.freeze())
    }
    /// Compile from retained exact scalar products, checking the current limits anew.
    pub fn compile_reusing(
        &self,
        options: Optimization,
        limits: EvaluationLimits,
        scope: &pse_kernels::ExecutionScope,
        scalars: &library::ScalarProducts,
    ) -> Result<CompiledBody, MathError> {
        scope.check().map_err(crate::error::scope_error)?;
        self.body().compile_scope(
            &(0..self.outputs().len()).collect::<Vec<_>>(),
            &self.dense_coordinates()?,
            self.order(),
            options,
            limits,
            scope,
            false,
            ProductKind::SelectedPartials,
            self,
            Some(scalars),
        )
    }
    /// Compile only a first directional action; no Value template is constructed.
    pub fn compile_directional_reusing(
        &self,
        options: Optimization,
        limits: EvaluationLimits,
        scope: &pse_kernels::ExecutionScope,
        scalars: &library::ScalarProducts,
    ) -> Result<CompiledBody, MathError> {
        scope.check().map_err(crate::error::scope_error)?;
        if self.order() != DerivativeOrder::First {
            return Err(MathError::Contract(
                "directional support must admit exactly First".into(),
            ));
        }
        self.body().compile_scope(
            &(0..self.outputs().len()).collect::<Vec<_>>(),
            &self.dense_coordinates()?,
            self.order(),
            options,
            limits,
            scope,
            false,
            ProductKind::JacobianAction,
            self,
            Some(scalars),
        )
    }
    /// Compile numerical products after independent selected-closure capability admission.
    pub fn compile(
        &self,
        options: Optimization,
        limits: EvaluationLimits,
        scope: &pse_kernels::ExecutionScope,
    ) -> Result<CompiledBody, MathError> {
        scope.check().map_err(crate::error::scope_error)?;
        self.body().compile_scope(
            &(0..self.outputs().len()).collect::<Vec<_>>(),
            &self.dense_coordinates()?,
            self.order(),
            options,
            limits,
            scope,
            false,
            ProductKind::SelectedPartials,
            self,
            None,
        )
    }
    /// Compile strict-interior derivatives from retained exact scalar preparations.
    /// Guard-boundary admission remains part of every numerical trial.
    pub fn compile_branch_local_reusing(
        &self,
        options: Optimization,
        limits: EvaluationLimits,
        scope: &pse_kernels::ExecutionScope,
        scalars: &library::ScalarProducts,
    ) -> Result<CompiledBody, MathError> {
        scope.check().map_err(crate::error::scope_error)?;
        self.body().compile_scope(
            &(0..self.outputs().len()).collect::<Vec<_>>(),
            &self.dense_coordinates()?,
            self.order(),
            options,
            limits,
            scope,
            true,
            ProductKind::SelectedPartials,
            self,
            Some(scalars),
        )
    }
    /// Compile a first directional action using one Taylor axis. Structural support
    /// still names every admitted formal coordinate; the axis is a computational layout.
    pub fn compile_directional(
        &self,
        options: Optimization,
        limits: EvaluationLimits,
        scope: &pse_kernels::ExecutionScope,
    ) -> Result<CompiledBody, MathError> {
        scope.check().map_err(crate::error::scope_error)?;
        if self.order() != DerivativeOrder::First {
            return Err(MathError::Contract(
                "directional support must admit exactly First".into(),
            ));
        }
        self.body().compile_scope(
            &(0..self.outputs().len()).collect::<Vec<_>>(),
            &self.dense_coordinates()?,
            DerivativeOrder::First,
            options,
            limits,
            scope,
            false,
            ProductKind::JacobianAction,
            self,
            None,
        )
    }
    /// Compile strict-interior derivatives; every trial retains boundary admission.
    pub fn compile_branch_local(
        &self,
        options: Optimization,
        limits: EvaluationLimits,
        scope: &pse_kernels::ExecutionScope,
    ) -> Result<CompiledBody, MathError> {
        scope.check().map_err(crate::error::scope_error)?;
        self.body().compile_scope(
            &(0..self.outputs().len()).collect::<Vec<_>>(),
            &self.dense_coordinates()?,
            self.order(),
            options,
            limits,
            scope,
            true,
            ProductKind::SelectedPartials,
            self,
            None,
        )
    }
}

/// Physically admitted symbolic regions without mutable evaluators or foreign workers.
#[derive(Clone, Debug)]
pub struct PreparedBody {
    data: Arc<PreparedBodyData>,
    owner: Option<Arc<dyn crate::AllocationOwner>>,
}
/// Shared immutable body data; construction and mutation stay inside admission.
#[derive(Clone, Debug)]
pub struct PreparedBodyData {
    pub(crate) inputs: usize,
    pub(crate) slots: usize,
    /// Dense offsets select stable authored symbols; empty means the authored layout.
    formal_slots: Vec<usize>,
    symbols: HashMap<Symbol, usize>,
    pub(crate) outputs: Vec<usize>,
    output_effects: Option<Vec<BTreeSet<usize>>>,
    pub(crate) stages: Vec<Stage>,
    /// Unspent authored construction allowance, continued by each demand.
    remaining_occurrences: usize,
    pub(crate) obligations: Vec<(Option<Atom>, Condition)>,
    input_quantities: Vec<Option<pse_quantity::QuantityTypeId>>,
    output_quantities: Vec<pse_quantity::QuantityTypeId>,
    expressions: Vec<Option<Atom>>,
    providers: Vec<ProviderSpec>,
    /// Local checked-member tokens whose actual attribution belongs to each instance.
    checked_members: BTreeSet<SemanticId>,
    /// Construction occurrences its builder counted; accounting, not mathematical identity.
    occurrences: usize,
}
impl std::ops::Deref for PreparedBody {
    type Target = PreparedBodyData;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl PartialEq for PreparedBody {
    fn eq(&self, other: &Self) -> bool {
        // Allocation lifetime is runtime metadata, not mathematical identity.
        self.inputs == other.inputs
            && self.slots == other.slots
            && self.formal_slots == other.formal_slots
            && self.outputs == other.outputs
            && self.output_effects == other.output_effects
            && self.stages == other.stages
            && self.obligations == other.obligations
            && self.input_quantities == other.input_quantities
            && self.output_quantities == other.output_quantities
            && self.expressions == other.expressions
            && self.providers == other.providers
            && self.checked_members == other.checked_members
    }
}
impl PreparedBody {
    fn parameters(&self) -> Result<Vec<Atom>, MathError> {
        if self.formal_slots.is_empty() {
            (0..self.slots).map(library::formal).collect()
        } else {
            self.formal_slots
                .iter()
                .copied()
                .map(library::formal)
                .collect()
        }
    }
    fn formal_id(&self, slot: usize) -> usize {
        self.formal_slots.get(slot).copied().unwrap_or(slot)
    }
    /// Project only executable selected stages and their captures. Symbol identity remains
    /// authored; every storage index is lowered through this single explicit layout.
    fn compact(
        &self,
        outputs: &[usize],
        coordinates: &[usize],
        source_inputs: usize,
    ) -> Result<Self, MathError> {
        let mut stages = self.demand(outputs, &self.symbols);
        let mut slots = BTreeSet::new();
        stage_slots(&stages, &self.symbols, &mut slots)?;
        slots.extend(outputs.iter().map(|&output| self.outputs[output]));
        let mut formals = slots
            .iter()
            .map(|&slot| self.formal_id(slot))
            .collect::<BTreeSet<_>>();
        formals.extend(coordinates);
        let formal_slots = formals.into_iter().collect::<Vec<_>>();
        let offsets = formal_slots
            .iter()
            .enumerate()
            .map(|(offset, &formal)| (formal, offset))
            .collect::<BTreeMap<_, _>>();
        let lower = slots
            .into_iter()
            .map(|slot| (slot, offsets[&self.formal_id(slot)]))
            .collect::<BTreeMap<_, _>>();
        lower_stages(&mut stages, &lower)?;
        let parameters = formal_slots
            .iter()
            .copied()
            .map(library::formal)
            .collect::<Result<Vec<_>, _>>()?;
        let symbols = symbol_map(&parameters);
        let inputs = formal_slots.partition_point(|&formal| formal < source_inputs);
        let input_quantities = formal_slots[..inputs]
            .iter()
            .map(|formal| {
                if self.formal_slots.is_empty() {
                    self.input_quantities.get(*formal).copied().flatten()
                } else {
                    self.formal_slots
                        .binary_search(formal)
                        .ok()
                        .and_then(|i| self.input_quantities.get(i).copied().flatten())
                }
            })
            .collect();
        let providers = stage_providers(&stages).into_values().collect();
        Ok(Self {
            data: Arc::new(PreparedBodyData {
                inputs,
                slots: formal_slots.len(),
                formal_slots,
                symbols,
                outputs: outputs.iter().map(|&i| lower[&self.outputs[i]]).collect(),
                output_effects: self.output_effects.as_ref().map(|effects| {
                    outputs
                        .iter()
                        .map(|&i| {
                            effects[i]
                                .iter()
                                .filter_map(|slot| lower.get(slot).copied())
                                .collect()
                        })
                        .collect()
                }),
                stages,
                remaining_occurrences: self.remaining_occurrences,
                // The selected stage program carries every actual obligation. Optional
                // whole-body flattening views are not needed by support or compilation.
                obligations: vec![],
                input_quantities,
                output_quantities: outputs
                    .iter()
                    .filter_map(|&i| self.output_quantities.get(i).copied())
                    .collect(),
                expressions: vec![None; outputs.len()],
                providers,
                checked_members: BTreeSet::new(),
                occurrences: self.occurrences,
            }),
            owner: None,
        })
    }
    /// Process-local immutable allocation identity used only for unique live accounting.
    pub fn allocation_identity(&self) -> usize {
        Arc::as_ptr(&self.data) as usize
    }
    /// Known symbolic payload and container contents. The library's global symbol interner
    /// is process state that no product owns or releases, so it is not counted here.
    pub fn retained_bytes(&self) -> usize {
        size_of::<PreparedBodyData>()
            + self.formal_slots.capacity() * size_of::<usize>()
            + self.symbols.capacity() * (size_of::<Symbol>() + size_of::<usize>() + 16)
            + self.checked_members.len() * (size_of::<SemanticId>() + 96)
            + stage_retained_bytes(&self.stages, &mut BTreeSet::new())
            + self.outputs.capacity() * size_of::<usize>()
            + self.input_quantities.capacity() * size_of::<Option<pse_quantity::QuantityTypeId>>()
            + self.output_quantities.capacity() * size_of::<pse_quantity::QuantityTypeId>()
            + self
                .expressions
                .iter()
                .flatten()
                .chain(self.obligations.iter().filter_map(|(a, _)| a.as_ref()))
                .map(|a| a.as_view().get_byte_size())
                .sum::<usize>()
    }
    /// Declare body-local checked-member attribution after typed closure admission.
    /// Every declared token must occur in an emitted closure validity stage; form/data
    /// lineages keep their original authored identities and are not instance tokens.
    pub fn with_checked_members(mut self, tokens: BTreeSet<SemanticId>) -> Result<Self, MathError> {
        fn collect(stages: &[Stage], members: &mut BTreeSet<SemanticId>) {
            use pse_model::generated::enums::ModelingValidityLayer;
            for stage in stages {
                match stage {
                    Stage::Require {
                        lineage: Some(lineage),
                        ..
                    } => {
                        if lineage.layer == ModelingValidityLayer::Closure {
                            members.extend(&lineage.members);
                        }
                    }
                    Stage::Domain {
                        stages, lineage, ..
                    } => {
                        if lineage.layer == ModelingValidityLayer::Closure {
                            members.extend(&lineage.members);
                        }
                        collect(stages, members);
                    }
                    Stage::Applicability { stages, .. } => collect(stages, members),
                    Stage::Branch {
                        then, otherwise, ..
                    } => {
                        collect(then, members);
                        collect(otherwise, members);
                    }
                    _ => {}
                }
            }
        }
        let mut emitted = BTreeSet::new();
        collect(&self.stages, &mut emitted);
        if emitted != tokens {
            return Err(MathError::Contract(
                "checked-member tokens do not cover emitted closure validity".into(),
            ));
        }
        Arc::make_mut(&mut self.data).checked_members = tokens;
        Ok(self)
    }
    /// Complete attribution demand of this normalized reusable body.
    pub fn checked_members(&self) -> &BTreeSet<SemanticId> {
        &self.checked_members
    }
    /// Retain accounting when a body clone outlives its runtime case plan.
    pub fn with_owner(mut self, owner: Arc<dyn crate::AllocationOwner>) -> Self {
        self.owner = Some(crate::retain_allocation_owner(self.owner.take(), owner));
        self
    }
    #[cfg(test)]
    pub(crate) fn new(
        inputs: usize,
        slots: usize,
        outputs: Vec<usize>,
        stages: Vec<Stage>,
        _smooth: DerivativeOrder,
    ) -> Result<Self, MathError> {
        Self::new_with_allowance(
            inputs,
            slots,
            outputs,
            stages,
            &mut crate::typed::BodyLimits::default().occurrences,
        )
    }
    /// Value admission continues the caller's finite authored construction allowance.
    pub(crate) fn new_with_allowance(
        inputs: usize,
        slots: usize,
        outputs: Vec<usize>,
        stages: Vec<Stage>,
        remaining: &mut usize,
    ) -> Result<Self, MathError> {
        let _phase = pse_diagnostics::observation::current().map(|context| {
            context.count("math.prepared_body_constructions", "count", 1);
            context.phase("math.prepared_body_construction")
        });
        if slots == 0 || inputs > slots || outputs.is_empty() || outputs.iter().any(|&i| i >= slots)
        {
            return Err(MathError::Contract("prepared body layout".into()));
        }
        let mut allowance = SupportAllowance {
            remaining,
            derivatives: 0,
        };
        allowance.consume(slots, SemanticId::NIL)?;
        let parameters = (0..slots)
            .map(library::formal)
            .collect::<Result<Vec<_>, _>>()?;
        let symbols = symbol_map(&parameters);
        let mut assigned = (0..inputs).collect::<BTreeSet<_>>();
        validate_dependencies(&stages, &symbols, &mut assigned, 0, allowance.remaining)?;
        if outputs.iter().any(|i| !assigned.contains(i)) {
            return Err(MathError::Contract(
                "output unassigned on some branch".into(),
            ));
        }
        allowance.consume(slots, SemanticId::NIL)?;
        allowance.consume(1, SemanticId::NIL)?;
        let empty_fact = Arc::new(Fact::default());
        let mut facts = Facts::Dense(vec![empty_fact; slots]);
        for (i, parameter) in parameters.iter().enumerate().take(inputs) {
            allowance.consume(1, SemanticId::NIL)?;
            facts.set(
                i,
                Arc::new(Fact {
                    expression: Some(parameter.clone()),
                    ..Fact::default()
                }),
            );
        }
        let mut providers = BTreeMap::new();
        let mut obligations = vec![];
        analyze(
            &stages,
            &parameters,
            &symbols,
            &mut facts,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
            &mut providers,
            &mut obligations,
            &mut allowance,
            DerivativeOrder::Value,
            None,
            None,
            None,
        )?;
        let expressions = outputs
            .iter()
            .map(|&i| facts[i].expression.clone())
            .collect();
        Ok(Self {
            data: Arc::new(PreparedBodyData {
                inputs,
                slots,
                formal_slots: Vec::new(),
                symbols,
                outputs,
                output_effects: None,
                stages,
                remaining_occurrences: *allowance.remaining,
                obligations,
                input_quantities: vec![None; inputs],
                output_quantities: vec![],
                expressions,
                providers: providers.into_values().collect(),
                occurrences: 0,
                checked_members: BTreeSet::new(),
            }),
            owner: None,
        })
    }
    /// Smooth derivative capability of all admitted outputs and formal coordinates.
    pub fn available_order(&self) -> DerivativeOrder {
        self.available_order_for(&(0..self.inputs).collect::<Vec<_>>())
    }
    /// Order available inside selected active branches; boundaries require trial checks.
    pub fn branch_local_order(&self) -> DerivativeOrder {
        self.selected_order(
            &(0..self.output_count()).collect::<Vec<_>>(),
            &(0..self.inputs).collect::<Vec<_>>(),
            true,
        )
        .unwrap_or(DerivativeOrder::Value)
    }
    /// Capability of all outputs with respect to selected formal coordinates.
    pub fn available_order_for(&self, coordinates: &[usize]) -> DerivativeOrder {
        self.available_order_for_outputs(&(0..self.output_count()).collect::<Vec<_>>(), coordinates)
            .unwrap_or(DerivativeOrder::Value)
    }
    /// Capability of precisely the selected output/effect closure, independent of readiness.
    pub fn available_order_for_outputs(
        &self,
        outputs: &[usize],
        coordinates: &[usize],
    ) -> Result<DerivativeOrder, MathError> {
        self.selected_order(outputs, coordinates, false)
    }
    fn selected_order(
        &self,
        outputs: &[usize],
        coordinates: &[usize],
        local_branches: bool,
    ) -> Result<DerivativeOrder, MathError> {
        self.check_selection(outputs, coordinates)?;
        if self.formal_slots.is_empty() {
            let body = self.compact(outputs, coordinates, self.inputs)?;
            let dense = coordinates
                .iter()
                .map(|formal| {
                    body.formal_slots.binary_search(formal).map_err(|_| {
                        MathError::Contract("selected derivative coordinate absent".into())
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            return body.selected_order(
                &(0..outputs.len()).collect::<Vec<_>>(),
                &dense,
                local_branches,
            );
        }
        let stages = self.demanded_stages(outputs)?;
        let parameters = self.parameters()?;
        let symbols = symbol_map(&parameters);
        let mut depends = vec![false; self.slots];
        for &i in coordinates {
            depends[i] = true;
        }
        let numeric = numeric_slots(
            &stages,
            &outputs.iter().map(|&i| self.outputs[i]).collect::<Vec<_>>(),
            &symbols,
        )?;
        selected_capability(&stages, &symbols, &mut depends, local_branches, &numeric)
    }
    fn check_selection(&self, outputs: &[usize], coordinates: &[usize]) -> Result<(), MathError> {
        if outputs.is_empty()
            || outputs.iter().any(|&i| i >= self.output_count())
            || outputs.iter().collect::<BTreeSet<_>>().len() != outputs.len()
            || coordinates.iter().any(|&i| i >= self.inputs)
            || coordinates.iter().collect::<BTreeSet<_>>().len() != coordinates.len()
        {
            return Err(MathError::Contract(
                "compiled output/coordinate demand".into(),
            ));
        }
        Ok(())
    }
    /// Construct selected immutable support without constructing an evaluator.
    pub fn prepare_support(
        &self,
        outputs: &[usize],
        coordinates: &[usize],
        order: DerivativeOrder,
        cancel: &Arc<AtomicBool>,
    ) -> Result<PreparedSupport, MathError> {
        let _phase = pse_diagnostics::observation::current().map(|context| {
            context.count("math.support_preparation_calls", "count", 1);
            context.gauge(
                "math.support_selected_outputs",
                "count",
                outputs.len() as f64,
            );
            context.gauge(
                "math.support_selected_coordinates",
                "count",
                coordinates.len() as f64,
            );
            context.phase("math.support_preparation")
        });
        self.check_selection(outputs, coordinates)?;
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        let body = self.compact(outputs, coordinates, self.inputs)?;
        let dense = coordinates
            .iter()
            .map(|formal| {
                body.formal_slots.binary_search(formal).map_err(|_| {
                    MathError::Contract("selected derivative coordinate absent".into())
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut support = body.support_with_allowance(
            &(0..outputs.len()).collect::<Vec<_>>(),
            &dense,
            order,
            self.remaining_occurrences,
            cancel,
            None,
        )?;
        support.restore_selection(
            Arc::downgrade(&self.data),
            self.inputs,
            outputs,
            coordinates,
        );
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("math.support_preparation_completed", "count", 1);
        }
        Ok(support)
    }
    /// Conservative all-branch incidence, including opaque value-only providers.
    pub fn incidence(
        &self,
        outputs: &[usize],
        coordinates: &[usize],
        cancel: &Arc<AtomicBool>,
    ) -> Result<PreparedSupport, MathError> {
        self.prepare_support(outputs, coordinates, DerivativeOrder::First, cancel)
    }
    fn support_with_allowance(
        &self,
        outputs: &[usize],
        coordinates: &[usize],
        order: DerivativeOrder,
        mut remaining: usize,
        cancel: &Arc<AtomicBool>,
        retained_program: Option<Arc<PreparedStages>>,
    ) -> Result<PreparedSupport, MathError> {
        let _phase = pse_diagnostics::observation::current().map(|context| {
            context.count("math.support_constructions", "count", 1);
            context.phase("math.support_construction")
        });
        let construction = tracing::info_span!(
            "pse.case.support_construction",
            product = "support",
            derivative_order = order_name(order),
            success = false,
            derivative_operations = tracing::field::Empty
        );
        let _construction = construction.enter();
        self.check_selection(outputs, coordinates)?;
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        let mut allowance = SupportAllowance {
            remaining: &mut remaining,
            derivatives: 0,
        };
        allowance.consume(
            outputs
                .len()
                .checked_add(coordinates.len())
                .ok_or(MathError::Limit("support demand maps"))?,
            SemanticId::NIL,
        )?;
        let mut support = Support::default();
        if order > DerivativeOrder::Value {
            allowance.consume(self.slots, SemanticId::NIL)?;
            let parameters = self.parameters()?;
            let symbols = symbol_map(&parameters);
            let stages = self.demand(outputs, &symbols);
            allowance.consume(self.slots, SemanticId::NIL)?;
            allowance.consume(1, SemanticId::NIL)?;
            let empty_fact = Arc::new(Fact::default());
            let mut facts = Facts::Dense(vec![empty_fact; self.slots]);
            for &i in coordinates {
                allowance.consume(2, SemanticId::NIL)?;
                facts.set(
                    i,
                    Arc::new(Fact {
                        expression: Some(parameters[i].clone()),
                        first: BTreeSet::from([i]),
                        ..Fact::default()
                    }),
                );
            }
            // Non-coordinate inputs remain symbolic parameters for exact local composition.
            for i in (0..self.inputs).filter(|i| !coordinates.contains(i)) {
                allowance.consume(1, SemanticId::NIL)?;
                facts.set(
                    i,
                    Arc::new(Fact {
                        expression: Some(parameters[i].clone()),
                        ..Fact::default()
                    }),
                );
            }
            let selected = coordinates.iter().copied().collect::<BTreeSet<_>>();
            let numeric = numeric_slots(
                &stages,
                &outputs.iter().map(|&i| self.outputs[i]).collect::<Vec<_>>(),
                &symbols,
            )?;
            analyze(
                &stages,
                &parameters,
                &symbols,
                &mut facts,
                &mut support.controls,
                &mut BTreeSet::new(),
                &mut BTreeMap::new(),
                &mut vec![],
                &mut allowance,
                order,
                Some(&selected),
                Some(cancel),
                Some(&numeric),
            )?;
            allowance.consume(
                outputs
                    .len()
                    .checked_mul(if order >= DerivativeOrder::Second {
                        2
                    } else {
                        1
                    })
                    .ok_or(MathError::Limit("support output extent"))?,
                SemanticId::NIL,
            )?;
            for &output in outputs {
                let fact = &facts[self.outputs[output]];
                let source = fact.source.unwrap_or(SemanticId::NIL);
                allowance.consume(fact.first.len(), source)?;
                support.first.push(fact.first.clone());
                if order >= DerivativeOrder::Second {
                    allowance.consume(fact.second.len(), source)?;
                    support.second.push(fact.second.clone());
                }
            }
        }
        let derivative_operations = allowance.derivatives;
        let prepared = PreparedSupport::from_data(
            PreparedSupportData {
                body: self.clone(),
                source: Arc::downgrade(&self.data),
                source_inputs: self.inputs,
                outputs: outputs.to_vec(),
                coordinates: coordinates.to_vec(),
                order,
                support,
                remaining_occurrences: remaining,
                derivative_operations,
            },
            None,
            cancel,
            retained_program,
        )?;
        construction.record("success", true);
        construction.record("derivative_operations", derivative_operations as u64);
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("math.support_constructions_completed", "count", 1);
            context.count(
                "math.completed_support_derivative_calls",
                "calls",
                derivative_operations as u64,
            );
        }
        Ok(prepared)
    }
    /// Whether projection visits only arithmetic blocks and scalar domain requirements.
    /// Requirements inspect already computed slots; branch/provider regions introduce
    /// additional populations and retain conservative construction admission.
    pub fn is_flat_arithmetic(&self) -> bool {
        self.stages
            .iter()
            .all(|stage| matches!(stage, Stage::Block { .. } | Stage::Require { .. }))
    }
    /// Known Rust working population for capability/dependency traversal without
    /// constructing numerical evaluators. An optional leading coordinate count adds
    /// the second-order support used to prove an affine residual; it does not request
    /// second-order output products. Control/provider regions remain opaque here.
    pub fn arithmetic_analysis_allocation_bound(
        &self,
        second_order_support_axes: Option<usize>,
    ) -> Result<Option<usize>, MathError> {
        if !self.is_flat_arithmetic() {
            return Ok(None);
        }
        let overflow = || MathError::Limit("arithmetic analysis extent");
        let add = |a: usize, b: usize| a.checked_add(b).ok_or_else(overflow);
        let mul = |a: usize, b: usize| a.checked_mul(b).ok_or_else(overflow);
        let slots = add(add(self.slots, self.inputs)?, self.outputs.len())?;
        // Compact stage/source copies, formal/slot maps, selected-output sets,
        // parameters/symbol maps and scalar numeric/reachability maps coexist.
        let source = mul(self.retained_bytes(), 4)?;
        let traversal = mul(mul(slots, 8)?, 256)?;
        let support = if let Some(axes) = second_order_support_axes {
            if axes > self.inputs {
                return Err(MathError::Contract(
                    "analysis support coordinate extent".into(),
                ));
            }
            let width = add(add(1, axes)?, mul(axes, axes)?)?;
            let population = mul(mul(slots, width)?, 4)?;
            mul(population.min(self.remaining_occurrences), 256)?
        } else {
            0
        };
        Ok(Some(add(add(add(source, traversal)?, support)?, 4096)?))
    }
    /// Initial construction reservation for compiling every output at all input axes
    /// through the actual requested order, including its lower-order products.
    /// The runtime owner reserves positive optimized payload excess before allocation.
    /// Control/provider lowering retains its separately admitted opaque allowance.
    pub fn arithmetic_compilation_allocation_bound(
        &self,
        requested: DerivativeOrder,
        limits: EvaluationLimits,
    ) -> Result<Option<usize>, MathError> {
        self.arithmetic_compilation_bound(requested, limits, self.inputs, self.outputs.len())
    }
    /// Source-issued initial reservation for selected derivative axes and outputs.
    /// Conservative scalar/source populations remain borrowed; unselected input values
    /// never become derivative axes. No support map or numerical program is constructed.
    pub fn arithmetic_selected_compilation_allocation_bound(
        &self,
        outputs: &[usize],
        coordinates: &[usize],
        requested: DerivativeOrder,
        limits: EvaluationLimits,
    ) -> Result<Option<usize>, MathError> {
        if outputs.is_empty()
            || outputs
                .iter()
                .enumerate()
                .any(|(i, &v)| v >= self.outputs.len() || outputs[..i].contains(&v))
            || coordinates
                .iter()
                .enumerate()
                .any(|(i, &v)| v >= self.inputs || coordinates[..i].contains(&v))
        {
            return Err(MathError::Contract(
                "selected construction output/coordinate extent".into(),
            ));
        }
        self.arithmetic_compilation_bound(requested, limits, coordinates.len(), outputs.len())
    }
    fn arithmetic_compilation_bound(
        &self,
        requested: DerivativeOrder,
        limits: EvaluationLimits,
        coordinate_count: usize,
        output_count: usize,
    ) -> Result<Option<usize>, MathError> {
        if !self.is_flat_arithmetic() {
            return Ok(None);
        }
        let overflow = || MathError::Limit("arithmetic compilation extent");
        let add = |a: usize, b: usize| a.checked_add(b).ok_or_else(overflow);
        let mul = |a: usize, b: usize| a.checked_mul(b).ok_or_else(overflow);
        let n = if requested > DerivativeOrder::Value {
            coordinate_count
        } else {
            0
        };
        let mut numeric = 0usize;
        let mut instructions = 0usize;
        let mut descriptors = 0usize;
        for order in [
            DerivativeOrder::Value,
            DerivativeOrder::First,
            DerivativeOrder::Second,
        ] {
            if order > requested {
                break;
            }
            let first = if order >= DerivativeOrder::First {
                n
            } else {
                0
            };
            let second = if order >= DerivativeOrder::Second {
                mul(n, add(n, 1)?)? / 2
            } else {
                0
            };
            let width = add(add(1, first)?, second)?;
            let dense = if order >= DerivativeOrder::Second {
                mul(n, n)?
            } else {
                0
            };
            let mut entries = add(
                mul(self.slots, width)?,
                mul(output_count, add(add(1, first)?, dense)?)?,
            )?;
            // Full/local layouts, coordinate maps, component maps and the sparse
            // zero-component (parameter,component) list coexist during a block build.
            let layout = mul(
                width,
                add(
                    mul(n, size_of::<usize>())?,
                    2 * size_of::<Vec<usize>>() + 4 * size_of::<usize>(),
                )?,
            )?;
            descriptors = add(descriptors, layout)?;
            for stage in &self.stages {
                let Stage::Block {
                    expressions,
                    outputs,
                    ..
                } = stage
                else {
                    // Require compiles one descriptor for an existing slot; no numeric
                    // evaluator or Taylor expansion is constructed at this stage.
                    descriptors = add(descriptors, size_of::<CompiledStage>())?;
                    continue;
                };
                let block = library::evaluator_construction(expressions, self.slots, n, order)?;
                entries = add(
                    entries,
                    add(
                        block.entries,
                        mul(add(add(self.slots, outputs.len())?, 1)?, width)?,
                    )?,
                )?;
                instructions = add(instructions, block.instructions)?;
                descriptors = add(
                    descriptors,
                    add(
                        layout,
                        mul(mul(self.slots, width)?, 4 * size_of::<usize>())?,
                    )?,
                )?;
            }
            numeric = add(numeric, mul(entries, size_of::<f64>())?)?;
        }
        // The existing scratch guard independently bounds retained numeric programs
        // and worker/output buffers. Use it only as a checked upper alternative to
        // the source population, never as the initial construction demand.
        numeric = numeric.min(limits.scratch_bytes);
        let second = if requested >= DerivativeOrder::Second {
            mul(n, n)?
        } else {
            0
        };
        let support_width = add(add(1, n)?, second)?;
        let support_population = mul(
            mul(
                add(add(self.slots, self.inputs)?, output_count)?,
                support_width,
            )?,
            4,
        )?;
        let support = mul(support_population.min(self.remaining_occurrences), 256)?;
        let source = mul(self.retained_bytes(), 4)?;
        // Two numeric populations cover retained programs plus construction/cache
        // copies; source container copies and known instruction indices are separate.
        let bytes = add(
            add(add(mul(numeric, 2)?, instructions)?, descriptors)?,
            add(add(source, support)?, 4096)?,
        )?;
        Ok(Some(bytes))
    }
    /// Number of formal inputs.
    pub fn input_count(&self) -> usize {
        self.inputs
    }
    /// Formal slots the body holds: its inputs plus stage results, within its
    /// `BodyLimits::slots` allowance.
    pub fn slot_count(&self) -> usize {
        self.slots
    }
    /// Authored construction occurrences counted by its typed builder, before support
    /// preparation consumes the rest of `BodyLimits::occurrences`; zero without a builder.
    pub fn occurrence_count(&self) -> usize {
        self.occurrences
    }
    pub(crate) fn set_occurrences(&mut self, occurrences: usize) {
        Arc::make_mut(&mut self.data).occurrences = occurrences;
    }
    pub(crate) fn set_effects(&mut self, effects: Vec<BTreeSet<usize>>) {
        Arc::make_mut(&mut self.data).output_effects = Some(effects);
    }
    pub(crate) fn set_quantities(
        &mut self,
        inputs: Vec<Option<pse_quantity::QuantityTypeId>>,
        outputs: Vec<pse_quantity::QuantityTypeId>,
    ) {
        let data = Arc::make_mut(&mut self.data);
        data.input_quantities = inputs;
        data.output_quantities = outputs;
    }
    /// Scalar physical contracts established by the builder; unused inputs may be absent.
    pub fn input_quantities(&self) -> &[Option<pse_quantity::QuantityTypeId>] {
        &self.input_quantities
    }
    /// Physical output contracts in declaration order.
    pub fn output_quantities(&self) -> &[pse_quantity::QuantityTypeId] {
        &self.output_quantities
    }
    /// Number of ordered body outputs.
    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }
    /// Consumed executable contracts, keyed by full physical/data interpretation.
    pub fn providers(&self) -> &[ProviderSpec] {
        &self.providers
    }
    /// Actual provider orders required by compilation, including authored local partials.
    /// Domain and applicability evidence is evaluated at value order independently of outer jets.
    pub fn provider_demands(
        &self,
        order: DerivativeOrder,
    ) -> Result<BTreeMap<ProviderKey, DerivativeOrder>, MathError> {
        self.provider_demands_for_outputs(&(0..self.output_count()).collect::<Vec<_>>(), order)
    }
    /// Actual provider requirements for selected outputs and their mandatory effects.
    pub fn provider_demands_for_outputs(
        &self,
        outputs: &[usize],
        order: DerivativeOrder,
    ) -> Result<BTreeMap<ProviderKey, DerivativeOrder>, MathError> {
        self.provider_demands_for_selection(outputs, &(0..self.inputs).collect::<Vec<_>>(), order)
    }
    /// Provider requirements on the actual numerical derivative coordinates. Mandatory
    /// effects retain Value demand, and explicitly authored partials retain their order.
    /// # Errors
    /// Invalid output/coordinate selection or unsupported authored provider partials.
    pub fn provider_demands_for_selection(
        &self,
        outputs: &[usize],
        coordinates: &[usize],
        order: DerivativeOrder,
    ) -> Result<BTreeMap<ProviderKey, DerivativeOrder>, MathError> {
        self.check_selection(outputs, coordinates)?;
        if self.formal_slots.is_empty() {
            let body = self.compact(outputs, coordinates, self.inputs)?;
            let dense = coordinates
                .iter()
                .map(|formal| {
                    body.formal_slots.binary_search(formal).map_err(|_| {
                        MathError::Contract("selected derivative coordinate absent".into())
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            return body.provider_demands_for_selection(
                &(0..outputs.len()).collect::<Vec<_>>(),
                &dense,
                order,
            );
        }
        fn collect(
            stages: &[Stage],
            order: DerivativeOrder,
            demands: &mut BTreeMap<ProviderKey, DerivativeOrder>,
            numeric: &BTreeSet<usize>,
            coordinates: &[Vec<bool>],
        ) -> Result<(), MathError> {
            for stage in stages {
                match stage {
                    Stage::Provider {
                        spec,
                        partial,
                        inputs,
                        outputs,
                        ..
                    } => {
                        let requested = if outputs.iter().any(|i| numeric.contains(i))
                            && inputs.iter().any(|&i| coordinates[i][0])
                        {
                            order
                        } else {
                            DerivativeOrder::Value
                        };
                        let required = provider_request_order(requested, partial.len())?;
                        demands
                            .entry(spec.key())
                            .and_modify(|o| *o = (*o).max(required))
                            .or_insert(required);
                    }
                    Stage::Domain { stages, .. } | Stage::Applicability { stages, .. } => collect(
                        stages,
                        DerivativeOrder::Value,
                        demands,
                        numeric,
                        coordinates,
                    )?,
                    Stage::Branch {
                        then, otherwise, ..
                    } => {
                        collect(then, order, demands, numeric, coordinates)?;
                        collect(otherwise, order, demands, numeric, coordinates)?;
                    }
                    Stage::Block { .. } | Stage::Require { .. } => {}
                }
            }
            Ok(())
        }
        self.check_selection(outputs, coordinates)?;
        let mut demands = BTreeMap::new();
        let stages = self.demanded_stages(outputs)?;
        let parameters = self.parameters()?;
        let symbols = symbol_map(&parameters);
        let numeric = numeric_slots(
            &stages,
            &outputs.iter().map(|&i| self.outputs[i]).collect::<Vec<_>>(),
            &symbols,
        )?;
        let mut reachability = vec![vec![false]; self.slots];
        if order > DerivativeOrder::Value {
            for &coordinate in coordinates {
                reachability[coordinate][0] = true;
            }
            coordinate_reachability(&stages, &symbols, &mut reachability)?;
        }
        collect(&stages, order, &mut demands, &numeric, &reachability)?;
        Ok(demands)
    }
    /// Optional flattened library expression of one output. It is absent when a provider or
    /// branch prevents flattening, and also when substituting the shared stage results
    /// would exceed the flattening bound (large but factorable bodies, such as Helmholtz
    /// derivatives). Projections that must not lose such outputs read the stage program
    /// instead ([`crate::factorable`]).
    pub fn expression(&self, output: usize) -> Option<&Atom> {
        self.expressions.get(output).and_then(Option::as_ref)
    }
    /// Resolve a retained expression symbol through the admitted input layout. This is
    /// a storage offset, including compact layouts, never a library registration ordinal.
    pub(crate) fn input_slot(&self, symbol: Symbol) -> Option<usize> {
        self.symbols
            .get(&symbol)
            .copied()
            .filter(|&slot| slot < self.inputs)
    }
    /// Whether coefficient views must retain a domain/control obligation.
    pub fn has_obligations(&self) -> bool {
        has_obligations(&self.stages)
    }
    /// The stage program an evaluation of `outputs` executes: every retained obligation and
    /// only the arithmetic those outputs and their effects need. Projections read this
    /// program so that they carry exactly the obligations the evaluator enforces.
    /// # Errors
    /// An output ordinal outside the body, or an exhausted formal vocabulary.
    pub(crate) fn demanded_stages(&self, outputs: &[usize]) -> Result<Vec<Stage>, MathError> {
        if outputs.iter().any(|&i| i >= self.outputs.len()) {
            return Err(MathError::Contract("demanded body output".into()));
        }
        Ok(self.demand(outputs, &self.symbols))
    }
    fn demand(&self, outputs: &[usize], symbols: &HashMap<Symbol, usize>) -> Vec<Stage> {
        let mut needed: BTreeSet<_> = outputs.iter().map(|&i| self.outputs[i]).collect();
        if let Some(effects) = &self.output_effects {
            needed.extend(outputs.iter().flat_map(|&i| effects[i].iter().copied()));
        }
        prune(
            &self.stages,
            &mut needed,
            symbols,
            self.output_effects.is_none(),
        )
    }
    /// Compile an exact output/coordinate demand under explicit limits.
    pub fn compile(
        &self,
        outputs: &[usize],
        coordinates: &[usize],
        order: DerivativeOrder,
        options: Optimization,
        limits: EvaluationLimits,
        scope: &pse_kernels::ExecutionScope,
    ) -> Result<CompiledBody, MathError> {
        self.prepare_support(outputs, coordinates, order, scope.cancellation())?
            .compile(options, limits, scope)
    }
    /// Compile derivatives restricted to the strict interior of the selected control path.
    /// Each trial checks separation from every unproved guard boundary, including domain
    /// predicates. This establishes only a local neighborhood, never transition smoothness.
    /// # Errors
    /// Unsupported provider derivatives, resource limits, or invalid demands. A worker
    /// additionally refuses a trial on an unproved control boundary.
    pub fn compile_branch_local(
        &self,
        outputs: &[usize],
        coordinates: &[usize],
        order: DerivativeOrder,
        options: Optimization,
        limits: EvaluationLimits,
        scope: &pse_kernels::ExecutionScope,
    ) -> Result<CompiledBody, MathError> {
        self.prepare_support(outputs, coordinates, order, scope.cancellation())?
            .compile_branch_local(options, limits, scope)
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "The local scope is explicit and the public compilation contracts stay unchanged"
    )]
    fn compile_scope(
        &self,
        outputs: &[usize],
        coordinates: &[usize],
        order: DerivativeOrder,
        options: Optimization,
        limits: EvaluationLimits,
        scope: &pse_kernels::ExecutionScope,
        local_branches: bool,
        product: ProductKind,
        prepared_support: &PreparedSupport,
        retained_scalars: Option<&library::ScalarProducts>,
    ) -> Result<CompiledBody, MathError> {
        scope.check().map_err(crate::error::scope_error)?;
        let cancelled = scope.cancellation();
        let directional = product == ProductKind::JacobianAction;
        let hessian_action = product == ProductKind::WeightedHessianAction;
        let _phase = pse_diagnostics::observation::current().map(|context| {
            context.count("math.compiled_body_constructions", "count", 1);
            context.gauge(
                "math.compiled_selected_outputs",
                "count",
                outputs.len() as f64,
            );
            context.gauge(
                "math.compiled_selected_coordinates",
                "count",
                coordinates.len() as f64,
            );
            context.phase("math.compiled_body_construction")
        });
        let construction = tracing::info_span!(
            "pse.case.compiled_body_construction",
            product = "compiled_artifact",
            derivative_order = order_name(order),
            success = false
        );
        let _construction = construction.enter();
        limits.check()?;
        if cancelled.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        self.check_selection(outputs, coordinates)?;
        let available = self.selected_order(outputs, coordinates, local_branches)?;
        if order > available {
            return Err(MathError::DerivativeDemand {
                source_id: None,
                body: None,
                outputs: prepared_support.outputs().to_vec(),
                coordinates: prepared_support.coordinates().to_vec(),
                requested: order,
                available,
            });
        }
        let selected: Vec<_> = outputs.iter().map(|&i| self.outputs[i]).collect();
        let parameters = self.parameters()?;
        let symbols = symbol_map(&parameters);
        let stages = self.demand(outputs, &symbols);
        let numeric = numeric_slots(&stages, &selected, &symbols)?;
        // Conservative coordinate reachability includes every branch alternative. It
        // selects Taylor coefficients; Symbolica still owns every derivative operation.
        let reachability_coordinates = if order > DerivativeOrder::Value {
            if directional {
                1
            } else {
                coordinates.len() + usize::from(hessian_action)
            }
        } else {
            0
        };
        let support_entries = self
            .slots
            .checked_mul(reachability_coordinates.saturating_add(3))
            .ok_or(MathError::Limit("coordinate reachability"))?;
        limits.allocation(support_entries)?;
        let mut coordinate_support = vec![vec![false; reachability_coordinates]; self.slots];
        if order > DerivativeOrder::Value {
            for (coordinate, &slot) in coordinates.iter().enumerate() {
                coordinate_support[slot][if directional { 0 } else { coordinate }] = true;
                if hessian_action {
                    coordinate_support[slot][coordinates.len()] = true;
                }
            }
            coordinate_reachability(&stages, &symbols, &mut coordinate_support)?;
        }
        let frame_support = coordinate_support.iter().enumerate().map(|(slot, axes)| {
            if numeric.contains(&slot) { axes.clone() } else { vec![false; axes.len()] }
        }).collect::<Vec<_>>();
        // Ordinary artifacts retain their ceiling contract. Directional consumers
        // request only First and never construct an unused Value template.
        let mut layouts = EnumMap::<DerivativeOrder, Option<JetLayout>>::default();
        let mut programs = EnumMap::<DerivativeOrder, Option<Vec<CompiledStage>>>::default();
        let mut used = 0usize;
        let mut retained_numeric = 0usize;
        let mut retained_instructions = 0usize;
        let mut retained_modules = 0usize;
        let mut order_worker_bytes = EnumMap::<DerivativeOrder, usize>::default();
        let mut census = EnumMap::<DerivativeOrder, Option<CompiledOrderCensus>>::default();
        let mut remaining_operations = limits.operations;
        let mut remaining_providers = limits.provider_calls;
        let mut scalars = retained_scalars
            .map(library::ScalarCache::from_products)
            .unwrap_or_default();
        let first_layout = Arc::new(if directional || hessian_action {
            FirstLayout::from_axes(selected.iter().map(|&slot| {
                coordinate_support[slot]
                    .iter()
                    .enumerate()
                    .filter_map(|(axis, &supported)| supported.then_some(axis))
                    .collect()
            }))
        } else {
            FirstLayout::from_support(prepared_support)
        });
        for requested in [
            DerivativeOrder::Value,
            DerivativeOrder::First,
            DerivativeOrder::Second,
        ]
        .into_iter()
        .filter(|requested| {
            *requested <= order
                && match product {
                    ProductKind::SelectedPartials => true,
                    ProductKind::JacobianAction | ProductKind::TransposeJacobianAction => *requested == DerivativeOrder::First,
                    ProductKind::WeightedHessian | ProductKind::WeightedHessianAction => {
                        *requested == DerivativeOrder::Second
                    }
                }
        }) {
            let _order_phase = pse_diagnostics::observation::current().map(|context| {
                context.count("math.compiled_order_constructions", "count", 1);
                context.gauge(
                    "math.compiled_derivative_order",
                    "order",
                    requested as u8 as f64,
                );
                context.phase("math.compiled_order_construction")
            });
            let order_construction = tracing::info_span!(
                "pse.case.compiled_order_construction",
                product = "compiled_order",
                derivative_order = order_name(requested),
                success = false
            );
            let _order_construction = order_construction.enter();
            let required_pairs = if hessian_action {
                (0..coordinates.len())
                    .map(|axis| (axis, coordinates.len()))
                    .collect()
            } else if requested >= DerivativeOrder::Second {
                let axes = prepared_support
                    .coordinates()
                    .iter()
                    .enumerate()
                    .map(|(axis, &formal)| (formal, axis))
                    .collect::<BTreeMap<_, _>>();
                prepared_support
                    .support()
                    .second
                    .iter()
                    .flatten()
                    .filter_map(|&(i, j)| Some((*axes.get(&i)?, *axes.get(&j)?)))
                    .map(|(i, j)| (i.min(j), i.max(j)))
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect()
            } else {
                Vec::new()
            };
            let layout = JetLayout::with_pairs(
                if directional {
                    vec![0]
                } else {
                    let mut axes = coordinates.to_vec();
                    if hessian_action {
                        axes.push(usize::MAX);
                    }
                    axes
                },
                requested,
                required_pairs,
                limits,
            )?.with_slot_support(&frame_support, limits)?;
            let frame = layout.frame_entries(self.slots)?;
            limits.allocation(frame)?;
            let remaining_operations_before_order = remaining_operations;
            let mut allowance = BuildAllowance {
                operations: remaining_operations,
                providers: remaining_providers,
                entries: frame,
                instructions: 0,
                instruction_count: 0,
                modules: 0,
                selected_first: self.stages.iter().all(|stage| matches!(stage, Stage::Block { .. }))
                    && requested == DerivativeOrder::First
                    && product == ProductKind::SelectedPartials,
                first_variables: coordinates.iter().map(|&slot| parameters[slot].clone()).collect(),
                scalar_inputs: self.inputs,
                selected_partial_calls: limits.operations,
                #[cfg(feature = "compiled-evaluator")]
                native_prepaid: self.is_flat_arithmetic(),
                local_order: if local_branches {
                    requested
                } else {
                    DerivativeOrder::Value
                },
            };
            let program = compile_stages(
                &prepared_support.program,
                &coordinate_support,
                &numeric,
                &layout,
                options,
                limits,
                scope,
                &mut allowance,
                &mut scalars,
            )?;
            retained_modules = retained_modules
                .checked_add(allowance.modules)
                .ok_or(MathError::Limit("compiled module storage"))?;
            remaining_operations = allowance.operations;
            remaining_providers = allowance.providers;
            retained_numeric = retained_numeric
                .checked_add(allowance.entries - frame)
                .ok_or(MathError::Limit("retained numeric storage"))?;
            retained_instructions = retained_instructions
                .checked_add(allowance.instructions)
                .ok_or(MathError::Limit("retained instruction storage"))?;
            used = used
                .checked_add(allowance.entries)
                .ok_or(MathError::Limit("compiled demand scratch"))?;
            let result_entries = selected.len()
                .checked_add(if requested >= DerivativeOrder::First { first_layout.axes.len() } else { 0 })
                .and_then(|n| n.checked_add(if requested >= DerivativeOrder::Second { selected.len().checked_mul(layout.pairs.len())? } else { 0 }))
                .ok_or(MathError::Limit("output derivative buffers"))?;
            used = used.checked_add(result_entries)
                .ok_or(MathError::Limit("output derivative buffers"))?;
            limits.allocation(used)?;
            order_worker_bytes[requested] = (allowance.entries - frame)
                .checked_mul(size_of::<f64>())
                .and_then(|n| n.checked_add(allowance.instructions))
                .and_then(|n| n.checked_add(cloned_stage_descriptors(&program)?))
                .ok_or(MathError::Limit("order worker storage"))?;
            let mut counts = StageCensus::default();
            counts.add(&program);
            census[requested] = Some(CompiledOrderCensus {
                order: requested,
                directional,
                product,
                direct_weighted: false,
                stages: counts.stages,
                arithmetic_blocks: counts.arithmetic_blocks,
                arithmetic_input_entries: counts.arithmetic_input_entries,
                arithmetic_output_entries: counts.arithmetic_output_entries,
                native_arithmetic_blocks: counts.native_arithmetic_blocks,
                provider_stages: counts.provider_stages,
                control_stages: counts.control_stages,
                library_operations: remaining_operations_before_order - allowance.operations,
                library_instructions: allowance.instruction_count,
                instruction_storage_bytes: allowance.instructions,
                frame_entries: frame,
                derivative_components: layout.width(),
                second_pairs: layout.pairs.len(),
                result_entries,
                worker_bytes: order_worker_bytes[requested],
            });
            layouts[requested] = Some(layout);
            programs[requested] = Some(program);
            order_construction.record("success", true);
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("math.compiled_order_constructions_completed", "count", 1);
            }
        }
        let evidence_bytes = applicability_extent(&stages)
            .saturating_mul(4)
            .checked_add(
                applicability_capture_extent(&stages)
                    .ok_or(MathError::Limit("applicability capture storage"))?,
            )
            .ok_or(MathError::Limit("applicability capture storage"))?;
        let scratch_bytes = used
            .checked_mul(size_of::<f64>())
            .and_then(|b| b.checked_add(evidence_bytes))
            .ok_or(MathError::Limit("applicability observation storage"))?;
        if scratch_bytes > limits.scratch_bytes {
            return Err(MathError::Limit("applicability observation storage"));
        }
        let ceiling_layout = layouts[order]
            .as_ref()
            .ok_or_else(|| MathError::Contract("compiled ceiling layout".into()))?;
        let result_entries = selected.len()
            .checked_add(if order >= DerivativeOrder::First { first_layout.axes.len() } else { 0 })
            .and_then(|n| n.checked_add(if order >= DerivativeOrder::Second { selected.len().checked_mul(ceiling_layout.pairs.len())? } else { 0 }))
            .ok_or(MathError::Limit("owning evaluation extent"))?;
        let mut evaluation_cache_bytes = result_entries.checked_add(self.inputs)
            .and_then(|n| n.checked_mul(2 * size_of::<f64>()))
            .and_then(|n| n.checked_add(evidence_bytes))
            .and_then(|n| n.checked_add(size_of::<TrialEvaluation>() + size_of::<Vec<u64>>() + 64))
            .ok_or(MathError::Limit("evaluation cache storage"))?;
        if product != ProductKind::SelectedPartials {
            let entries = match product {
                ProductKind::SelectedPartials => 0,
                ProductKind::JacobianAction | ProductKind::TransposeJacobianAction => selected.len() + coordinates.len(),
                ProductKind::WeightedHessian => {
                    layouts[order]
                        .as_ref()
                        .map_or(0, |layout| layout.pairs.len())
                        + selected.len()
                }
                ProductKind::WeightedHessianAction => 2 * coordinates.len() + selected.len(),
            };
            evaluation_cache_bytes = evaluation_cache_bytes
                .checked_add(
                    entries
                        .checked_mul(2 * size_of::<f64>())
                        .ok_or(MathError::Limit("product point cache"))?,
                )
                .ok_or(MathError::Limit("product point cache"))?;
        }
        let descriptor_bytes = programs
            .values()
            .flatten()
            .try_fold(0usize, |bytes, stages| {
                bytes.checked_add(cloned_stage_descriptors(stages)?)
            })
            .ok_or(MathError::Limit("worker descriptor storage"))?;
        let worker_bytes = scratch_bytes
            .checked_add(retained_instructions)
            .and_then(|n| n.checked_add(descriptor_bytes))
            .ok_or(MathError::Limit("worker owned storage"))?;
        #[cfg(test)]
        let scalar_builds = scalars.builds();
        let scalars = scalars.freeze();
        let scalar_new_bytes = scalars.unowned_bytes();
        let compiled = CompiledBody {
            owner: None,
            scalar_products: scalars.clone(),
            directional,
            product,
            direct_product: None,
            transpose_product: None,
            inputs: self.inputs,
            input_formals: Arc::new(self.formal_slots[..self.inputs].to_vec()),
            coordinate_offsets: Arc::new(coordinates.to_vec()),
            slots: self.slots,
            scratch_bytes,
            evaluation_cache_bytes,
            worker_bytes,
            order_worker_bytes,
            census,
            retained_bytes: retained_numeric
                .checked_mul(size_of::<f64>())
                .and_then(|n| {
                    n.checked_add(
                        (self.inputs + coordinates.len()) * size_of::<usize>()
                            + 2 * size_of::<Vec<usize>>(),
                    )
                })
                .and_then(|n| n.checked_add(retained_instructions))
                .and_then(|n| n.checked_add(retained_modules))
                .and_then(|n| n.checked_add(descriptor_bytes))
                .and_then(|n| n.checked_add(first_layout.retained_bytes()))
                .and_then(|n| n.checked_add(layouts.values().flatten().map(JetLayout::frame_layout_bytes).sum::<usize>()))
                .and_then(|n| {
                    n.checked_add(size_of::<
                        EnumMap<DerivativeOrder, Option<CompiledOrderCensus>>,
                    >())
                })
                .and_then(|n| n.checked_add(evidence_bytes))
                .and_then(|n| n.checked_add(prepared_support.retained_bytes()))
                .and_then(|n| n.checked_add(scalar_new_bytes))
                .and_then(|n| n.checked_add(scalars.descriptor_bytes()))
                .ok_or(MathError::Limit("retained program storage"))?,
            outputs: Arc::new(selected),
            layouts: Arc::new(layouts),
            first_layout,
            programs: Arc::new(programs),
            limits,
            support: prepared_support.clone(),
            #[cfg(test)]
            scalar_builds,
        };
        construction.record("success", true);
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("math.compiled_body_constructions_completed", "count", 1);
        }
        scope.check().map_err(crate::error::scope_error)?;
        Ok(compiled)
    }
}

#[derive(Clone)]
#[allow(
    clippy::large_enum_variant,
    reason = "Keep the existing interpreter inline rather than allocating every block"
)]
enum BlockEvaluator {
    Interpreted(ExpressionEvaluator<f64>),
    #[cfg(feature = "compiled-evaluator")]
    Native(crate::native_compiled::Evaluator),
}
impl BlockEvaluator {
    fn instantiate(&mut self) -> Result<(), MathError> {
        match self {
            Self::Interpreted(_) => Ok(()),
            #[cfg(feature = "compiled-evaluator")]
            Self::Native(evaluator) => evaluator.instantiate(),
        }
    }
    fn evaluate(&mut self, arguments: &[f64], values: &mut [f64]) -> Result<(), String> {
        match self {
            Self::Interpreted(evaluator) => evaluator
                .try_evaluate(arguments, values)
                .map_err(|e| e.to_string()),
            #[cfg(feature = "compiled-evaluator")]
            Self::Native(evaluator) => evaluator.evaluate(arguments, values),
        }
    }
    fn reset(self) -> Result<Self, MathError> {
        match self {
            Self::Interpreted(evaluator) => {
                let first = evaluator.get_input_len();
                let end = first
                    .checked_add(evaluator.get_constants().len())
                    .ok_or(MathError::Limit("evaluator reset constants"))?;
                let index = std::cell::Cell::new(0usize);
                Ok(Self::Interpreted(evaluator.map_coeff(&|value| {
                    let slot = index.get();
                    index.set(slot + 1);
                    if (first..end).contains(&slot) {
                        *value
                    } else {
                        f64::NAN
                    }
                })))
            }
            #[cfg(feature = "compiled-evaluator")]
            Self::Native(mut evaluator) => {
                evaluator.reset();
                Ok(Self::Native(evaluator))
            }
        }
    }
}
#[cfg(test)]
#[derive(Default)]
struct InstantiationProbe {
    visits: usize,
    evaluator_calls: usize,
    template_clones: usize,
    fail_at: Option<usize>,
    cancel_at: Option<usize>,
}
#[cfg(test)]
thread_local! {
    static INSTANTIATION_PROBE: std::cell::RefCell<InstantiationProbe> =
        std::cell::RefCell::new(InstantiationProbe::default());
    static FORCE_TAYLOR_FIRST: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn instantiate_stages(
    stages: &mut [CompiledStage],
    scope: &pse_kernels::ExecutionScope,
) -> Result<(), MathError> {
    for stage in stages {
        scope.check().map_err(crate::error::scope_error)?;
        #[cfg(test)]
        INSTANTIATION_PROBE.with_borrow_mut(|probe| {
            probe.visits += 1;
            if probe.cancel_at == Some(probe.visits) {
                scope.cancellation_control().cancel();
            }
            if probe.fail_at == Some(probe.visits) {
                return Err(MathError::Library("injected instantiation failure".into()));
            }
            Ok(())
        })?;
        match stage {
            CompiledStage::Block { evaluator, .. } => {
                evaluator.instantiate()?;
                #[cfg(test)]
                INSTANTIATION_PROBE.with_borrow_mut(|probe| probe.evaluator_calls += 1);
            }
            CompiledStage::Applicability { stages, .. } | CompiledStage::Domain { stages, .. } => {
                instantiate_stages(stages, scope)?
            }
            CompiledStage::Branch {
                then, otherwise, ..
            } => {
                instantiate_stages(then, scope)?;
                instantiate_stages(otherwise, scope)?;
            }
            _ => {}
        }
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct SourceCheck {
    end_output: usize,
    source: SemanticId,
}

#[derive(Clone)]
#[allow(
    clippy::large_enum_variant,
    reason = "Evaluator stages stay inline to avoid an allocation for each compiled stage"
)]
enum CompiledStage {
    Applicability {
        stages: Vec<Self>,
        frame: Vec<f64>,
        reads: Vec<usize>,
        layout: JetLayout,
        predicates: Vec<usize>,
        inputs: Vec<usize>,
        token: usize,
        plan: Arc<pse_model::applicability::Node>,
    },
    Domain {
        stages: Vec<Self>,
        frame: Vec<f64>,
        reads: Vec<usize>,
        layout: JetLayout,
        argument: usize,
        token: usize,
        lineage: Arc<pse_model::diagnostic::ValidityLineage>,
    },
    Block {
        evaluator: BlockEvaluator,
        resettable: bool,
        components: Vec<usize>,
        inputs: Vec<usize>,
        outputs: Vec<usize>,
        gather: Vec<(usize, usize)>,
        scatter: Vec<(usize, Option<usize>)>,
        arguments: Vec<f64>,
        values: Vec<f64>,
        source: SemanticId,
        checks: Vec<SourceCheck>,
        checked_ends: Vec<usize>,
    },
    Require {
        argument: usize,
        condition: Condition,
        order: DerivativeOrder,
        source: SemanticId,
        lineage: Option<Arc<pse_model::diagnostic::ValidityLineage>>,
    },
    Branch {
        require_separation: bool,
        comparison: Comparison,
        left: usize,
        right: usize,
        then: Vec<Self>,
        otherwise: Vec<Self>,
    },
    Provider {
        partial: Vec<usize>,
        inputs: Vec<usize>,
        outputs: Vec<usize>,
        request: PreparedProviderRequest,
        binding: Option<(u64, usize)>,
        buffer: ProviderBuffer,
        arguments: Vec<f64>,
        lift: Option<ProviderLift>,
        source: SemanticId,
    },
}
/// Populations and admitted capacities of one actual immutable evaluator order.
/// These are construction facts, not allocations measured during execution or RSS.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompiledOrderCensus {
    /// Derivative product actually compiled.
    pub order: DerivativeOrder,
    /// Whether the product carries one runtime directional axis.
    pub directional: bool,
    /// Mathematical product whose actual execution work is counted.
    pub product: ProductKind,
    /// The library emits the weighted product directly rather than per-output jets.
    pub direct_weighted: bool,
    /// All compiled host stages, including every branch alternative.
    pub stages: usize,
    /// Library arithmetic blocks, including nested control scopes.
    pub arithmetic_blocks: usize,
    /// Caller-owned scalar entries copied into the library across all arithmetic regions.
    pub arithmetic_input_entries: usize,
    /// Caller-owned scalar entries copied out of the library across all arithmetic regions.
    pub arithmetic_output_entries: usize,
    /// Arithmetic blocks whose actual implementation is compiled native code.
    pub native_arithmetic_blocks: usize,
    /// Fallible scientific provider stages.
    pub provider_stages: usize,
    /// Branch, validity, applicability and requirement barriers.
    pub control_stages: usize,
    /// Library-reported operations across arithmetic blocks and provider lifts.
    pub library_operations: usize,
    /// Actual exported instruction population, including exported sub-evaluator occurrences.
    pub library_instructions: usize,
    /// Conservative retained instruction storage, separate from instruction count.
    pub instruction_storage_bytes: usize,
    /// Admitted main numeric frame capacity in scalar entries.
    pub frame_entries: usize,
    /// Actual ancestor-closed Taylor shape width sent to the library.
    pub derivative_components: usize,
    /// Actual symmetric second components, excluding absent mathematical support.
    pub second_pairs: usize,
    /// Admitted result entries: selected values, supported firsts and packed seconds.
    pub result_entries: usize,
    /// Admitted per-order cloned stages and numeric scratch, excluding shared templates.
    pub worker_bytes: usize,
}
#[derive(Default)]
struct StageCensus {
    stages: usize,
    arithmetic_blocks: usize,
    arithmetic_input_entries: usize,
    arithmetic_output_entries: usize,
    native_arithmetic_blocks: usize,
    provider_stages: usize,
    control_stages: usize,
}
impl StageCensus {
    fn add(&mut self, stages: &[CompiledStage]) {
        for stage in stages {
            self.stages += 1;
            match stage {
                CompiledStage::Block { evaluator, arguments, values, .. } => {
                    self.arithmetic_blocks += 1;
                    self.arithmetic_input_entries += arguments.len();
                    self.arithmetic_output_entries += values.len();
                    match evaluator {
                        BlockEvaluator::Interpreted(_) => {}
                        #[cfg(feature = "compiled-evaluator")]
                        BlockEvaluator::Native(_) => self.native_arithmetic_blocks += 1,
                    }
                }
                CompiledStage::Provider { .. } => self.provider_stages += 1,
                CompiledStage::Applicability { stages, .. }
                | CompiledStage::Domain { stages, .. } => {
                    self.control_stages += 1;
                    self.add(stages);
                }
                CompiledStage::Branch {
                    then, otherwise, ..
                } => {
                    self.control_stages += 1;
                    self.add(then);
                    self.add(otherwise);
                }
                CompiledStage::Require { .. } => self.control_stages += 1,
            }
        }
    }
}
/// One original materialized arithmetic source represented by a compiled region.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArithmeticOrigin {
    /// Region ordinal in deterministic selected-program traversal, including both branch alternatives.
    pub region: usize,
    /// Original authored occurrence, never an optimizer-generated node.
    pub source: SemanticId,
    /// First original materialized output within this shared region.
    pub start_output: usize,
    /// Exclusive end of this occurrence's original materialized outputs.
    pub end_output: usize,
    /// Actual order of the retained materialized check. Direct weighted products retain Value checks.
    pub check_order: DerivativeOrder,
}
/// Immutable library artifact. Clone workers to obtain independent evaluator scratch.
#[derive(Clone)]
pub struct CompiledBody {
    scalar_products: library::ScalarProducts,
    directional: bool,
    product: ProductKind,
    direct_product: Option<Arc<products::DirectProduct>>,
    transpose_product: Option<Arc<products::TransposeProduct>>,
    owner: Option<Arc<dyn crate::AllocationOwner>>,
    scratch_bytes: usize,
    evaluation_cache_bytes: usize,
    worker_bytes: usize,
    order_worker_bytes: EnumMap<DerivativeOrder, usize>,
    census: EnumMap<DerivativeOrder, Option<CompiledOrderCensus>>,
    retained_bytes: usize,
    inputs: usize,
    input_formals: Arc<Vec<usize>>,
    coordinate_offsets: Arc<Vec<usize>>,
    slots: usize,
    outputs: Arc<Vec<usize>>,
    layouts: Arc<EnumMap<DerivativeOrder, Option<JetLayout>>>,
    first_layout: Arc<FirstLayout>,
    programs: Arc<EnumMap<DerivativeOrder, Option<Vec<CompiledStage>>>>,
    limits: EvaluationLimits,
    support: PreparedSupport,
    #[cfg(test)]
    scalar_builds: usize,
}
impl std::fmt::Debug for CompiledBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledBody")
            .field("inputs", &self.inputs)
            .field("outputs", &self.outputs.len())
            .field("profiles", &self.layouts.values().flatten().count())
            .finish()
    }
}
impl CompiledBody {
    pub(crate) fn evaluation_limits(&self) -> EvaluationLimits {
        self.limits
    }
    /// Compact facts from the actual admitted compilation, without support copies or
    /// library exports at inspection time. Scalar and dualized orders remain separate.
    pub fn census(&self) -> impl Iterator<Item = &CompiledOrderCensus> {
        self.census.values().flatten()
    }
    /// Count original arithmetic source checks without allocating diagnostic storage.
    pub fn arithmetic_origin_count(&self, order: DerivativeOrder) -> usize {
        let mut count = 0;
        self.visit_arithmetic_origins(order, |_| count += 1);
        count
    }
    /// Visit compact source ranges from existing selected program checks without allocation.
    /// A direct weighted derivative product names its original Value-check program;
    /// its shared weighted realization is in census. Both static branch arms are visited.
    pub fn visit_arithmetic_origins(
        &self,
        order: DerivativeOrder,
        mut visitor: impl FnMut(ArithmeticOrigin),
    ) {
        fn collect(
            stages: &[CompiledStage],
            order: DerivativeOrder,
            region: &mut usize,
            visitor: &mut impl FnMut(ArithmeticOrigin),
        ) {
            for stage in stages {
                match stage {
                    CompiledStage::Block { checks, .. } => {
                        let mut start = 0;
                        for check in checks {
                            visitor(ArithmeticOrigin {
                                region: *region,
                                source: check.source,
                                start_output: start,
                                end_output: check.end_output,
                                check_order: order,
                            });
                            start = check.end_output;
                        }
                        *region += 1;
                    }
                    CompiledStage::Applicability { stages, .. }
                    | CompiledStage::Domain { stages, .. } => {
                        collect(stages, DerivativeOrder::Value, region, visitor)
                    }
                    CompiledStage::Branch {
                        then, otherwise, ..
                    } => {
                        collect(then, order, region, visitor);
                        collect(otherwise, order, region, visitor);
                    }
                    _ => {}
                }
            }
        }
        let check_order = if self.direct_product.is_some() {
            DerivativeOrder::Value
        } else {
            order
        };
        if let Some(stages) = &self.programs[check_order] {
            collect(stages, check_order, &mut 0, &mut visitor);
        }
    }
    /// Exact immutable scalar preparations borrowed by this artifact.
    pub fn scalar_products(&self) -> &library::ScalarProducts {
        &self.scalar_products
    }
    /// Authored formal IDs in numerical input order. Evaluate accepts this compact signature.
    pub fn input_formals(&self) -> &[usize] {
        &self.input_formals
    }
    /// Individually shared evaluator component identities and known payload estimates.
    /// Body, support and shallow evaluator wrappers are observed separately.
    pub fn allocation_components(&self) -> Vec<(&'static str, usize, usize)> {
        let layout_bytes = size_of::<EnumMap<DerivativeOrder, Option<JetLayout>>>()
            + self
                .layouts
                .values()
                .flatten()
                .map(|layout| {
                    layout.coordinates.capacity() * size_of::<usize>()
                        + layout.shape.capacity() * size_of::<Vec<usize>>()
                        + layout
                            .shape
                            .iter()
                            .map(|s| s.capacity() * size_of::<usize>())
                            .sum::<usize>()
                        + layout.pairs.capacity() * size_of::<(usize, usize)>()
                })
                .sum::<usize>();
        let mut components = vec![
            (
                "input_formals",
                Arc::as_ptr(&self.input_formals) as usize,
                size_of::<Vec<usize>>() + self.input_formals.capacity() * size_of::<usize>(),
            ),
            (
                "coordinate_offsets",
                Arc::as_ptr(&self.coordinate_offsets) as usize,
                size_of::<Vec<usize>>() + self.coordinate_offsets.capacity() * size_of::<usize>(),
            ),
            (
                "outputs",
                Arc::as_ptr(&self.outputs) as usize,
                size_of::<Vec<usize>>() + self.outputs.capacity() * size_of::<usize>(),
            ),
            ("layouts", Arc::as_ptr(&self.layouts) as usize, layout_bytes),
            (
                "programs",
                Arc::as_ptr(&self.programs) as usize,
                self.retained_bytes
                    .saturating_sub(self.support.retained_bytes())
                    .saturating_sub(
                        self.direct_product
                            .as_ref()
                            .map_or(0, |product| product.retained_bytes),
                    ),
            ),
        ];
        if let Some(product) = &self.direct_product {
            components.push((
                "direct_product",
                Arc::as_ptr(product) as usize,
                product.retained_bytes,
            ));
        }
        components
    }
    /// Whether this product carries one directional Taylor axis instead of a full Jacobian.
    pub fn is_directional(&self) -> bool {
        self.directional
    }
    /// Highest residual order actually compiled, distinct from symbolic availability.
    pub fn compiled_order(&self) -> DerivativeOrder {
        if let Some(product) = &self.direct_product {
            product.order
        } else if self.transpose_product.is_some() {
            DerivativeOrder::First
        } else if self.layouts[DerivativeOrder::Second].is_some() {
            DerivativeOrder::Second
        } else if self.layouts[DerivativeOrder::First].is_some() {
            DerivativeOrder::First
        } else {
            DerivativeOrder::Value
        }
    }
    /// Fresh body wrapper and all possible accounting joins when attaching another
    /// owner. Evaluator, scalar-map and support payloads remain shared. Arc control
    /// blocks carry strong/weak counts; each join retains the prior and new owners.
    pub fn owner_attachment_bytes(&self) -> usize {
        let controls = 2 * size_of::<usize>();
        let join = size_of::<(
            Arc<dyn crate::AllocationOwner>,
            Arc<dyn crate::AllocationOwner>,
        )>() + controls;
        size_of::<Self>() + controls + 3 * join
    }
    /// Attach accounting to the allocation itself so evaluator/worker clones retain it.
    pub fn with_owner(self, owner: Arc<dyn crate::AllocationOwner>) -> Self {
        self.with_owner_and_scalar_foreign_allowance(owner, 0)
    }
    /// Transfer the constructor's admitted foreign allowance only for new exact scalars.
    pub fn with_owner_and_scalar_foreign_allowance(
        mut self,
        owner: Arc<dyn crate::AllocationOwner>,
        foreign_bytes: usize,
    ) -> Self {
        if self.scalar_products.has_unowned() {
            self.retained_bytes = self.retained_bytes.saturating_add(foreign_bytes);
        }
        self.scalar_products = self
            .scalar_products
            .with_owner_and_foreign_allowance(owner.clone(), foreign_bytes);
        self.support = self.support.with_owner(owner.clone());
        self.owner = Some(crate::retain_allocation_owner(self.owner.take(), owner));
        self
    }
    /// Upper bound on owned numeric frame and stage scratch.
    pub fn scratch_bytes(&self) -> usize {
        self.scratch_bytes
    }
    /// Conservative complete attempt-owned evaluator extent: numeric scratch plus
    /// cloned library instructions and stage descriptors; shared Arc payloads excluded.
    pub fn worker_bytes(&self) -> usize {
        self.worker_bytes
    }
    /// Conservative retained result and input-signature extent of one occurrence's
    /// cache at the compiled derivative ceiling, including applicability evidence.
    pub(crate) fn evaluation_cache_bytes(&self) -> usize {
        self.evaluation_cache_bytes
    }
    /// Storage retained by the immutable evaluator templates: their numeric buffers and
    /// library instruction streams, excluding attempt frames and returned derivative
    /// buffers.
    pub fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }
    /// Independent mutable scratch; caller/attempt owns its provider worker map.
    pub fn worker(&self) -> Worker {
        self.worker_scoped(pse_kernels::ExecutionScope::new(
            pse_diagnostics::Cancellation::default(),
            None,
        ))
    }
    /// Independent scratch attached to the enclosing execution's immutable controls.
    pub fn worker_scoped(&self, scope: pse_kernels::ExecutionScope) -> Worker {
        Worker {
            body: self.clone(),
            programs: EnumMap::default(),
            frame: Vec::new(),
            direct_product: None,
            transpose_worker: None,
            product_scratch: Vec::new(),
            prepaid_bytes: self.worker_bytes,
            used_bytes: 0,
            scope: Some(scope),
        }
    }
    /// Exact selected admitted demand retained by this evaluator product.
    pub fn prepared_support(&self) -> &PreparedSupport {
        &self.support
    }
    pub(crate) fn projection_second_pairs(&self) -> Arc<Vec<(usize, usize)>> {
        self.layouts[DerivativeOrder::Second]
            .as_ref()
            .map_or_else(|| Arc::new(Vec::new()), |layout| layout.pairs.clone())
    }
    pub(crate) fn projection_first_layout(&self) -> Arc<FirstLayout> {
        self.first_layout.clone()
    }
    pub(crate) fn first_result_entries(&self) -> usize {
        self.first_layout.axes.len()
    }
    pub(crate) fn projection_witnesses(
        &self,
        order: DerivativeOrder,
    ) -> Vec<Arc<pse_model::applicability::Node>> {
        fn collect(stages: &[CompiledStage], out: &mut Vec<Arc<pse_model::applicability::Node>>) {
            for stage in stages {
                match stage {
                    CompiledStage::Applicability { plan, stages, .. } => {
                        out.push(plan.clone());
                        collect(stages, out);
                    }
                    CompiledStage::Domain { stages, .. } => collect(stages, out),
                    CompiledStage::Branch {
                        then, otherwise, ..
                    } => {
                        collect(then, out);
                        collect(otherwise, out);
                    }
                    _ => {}
                }
            }
        }
        let mut out = Vec::new();
        if let Some(stages) = &self.programs[order] {
            collect(stages, &mut out);
        }
        out
    }
    /// Structural support survives trials and numerical zeros.
    pub fn support(&self) -> &Support {
        self.support.support()
    }
    /// Explicit structural incidence admission from this retained output demand.
    pub fn incidence(&self, cancel: &Arc<AtomicBool>) -> Result<PreparedSupport, MathError> {
        if self.support.order() >= DerivativeOrder::First {
            return Ok(self.support.clone());
        }
        self.support.upgrade(DerivativeOrder::First, cancel)
    }
    /// Differentiation coordinates in formal input order.
    pub fn coordinates(&self) -> &[usize] {
        self.support.coordinates()
    }
}
/// Atomic result, with raw derivatives in output-major row-major order.
#[derive(Clone, Debug, PartialEq)]
pub struct Evaluation {
    /// Evidence observed only on active demanded numerical paths.
    pub applicability: Vec<pse_model::applicability::Observation>,
    /// Requested values.
    pub values: Vec<f64>,
    /// Output × coordinate first partials.
    pub jacobian: Vec<f64>,
    /// Output × coordinate × coordinate full symmetric Hessians.
    pub hessians: Vec<f64>,
}
/// Successful active-path evidence, retaining numerical inputs and immutable attribution.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ApplicabilityWitness {
    plan: Arc<pse_model::applicability::Node>,
    predicates: Vec<bool>,
    inputs: Vec<f64>,
}
impl ApplicabilityWitness {
    pub(crate) fn hydrate(&self) -> Vec<pse_model::applicability::Observation> {
        self.plan
            .assess(&self.predicates, &self.inputs)
            .observations
    }
}
/// Immutable support-indexed addresses, in output order and then coordinate order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FirstLayout {
    rows: Vec<std::ops::Range<usize>>,
    axes: Vec<usize>,
}
impl FirstLayout {
    fn retained_bytes(&self) -> usize {
        size_of::<Self>() + self.rows.capacity() * size_of::<std::ops::Range<usize>>()
            + self.axes.capacity() * size_of::<usize>()
    }
    fn from_axes(rows: impl Iterator<Item = Vec<usize>>) -> Self {
        let mut result = Self::default();
        for axes in rows {
            let start = result.axes.len();
            result.axes.extend(axes);
            result.rows.push(start..result.axes.len());
        }
        result
    }
    pub(crate) fn from_support(support: &PreparedSupport) -> Self {
        Self::from_axes(support.support().first.iter().map(|first| {
            support.coordinates().iter().enumerate()
                .filter_map(|(axis, formal)| first.contains(formal).then_some(axis))
                .collect()
        }))
    }
    pub(crate) fn offset(&self, row: usize, axis: usize) -> Option<usize> {
        let range = self.rows.get(row)?;
        self.axes[range.clone()].binary_search(&axis).ok().map(|i| range.start + i)
    }
    pub(crate) fn entries(&self, row: usize) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.rows.get(row).cloned().into_iter().flatten()
            .map(|offset| (self.axes[offset], offset))
    }
}

/// Operation-local result. First and second partials retain support-indexed layouts.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TrialEvaluation {
    pub applicability: Vec<ApplicabilityWitness>,
    spare_witnesses: Vec<ApplicabilityWitness>,
    pub values: Vec<f64>,
    pub first: Vec<f64>,
    first_layout: Arc<FirstLayout>,
    first_requested: bool,
    second: Vec<f64>,
    coordinates: usize,
    second_pairs: Arc<Vec<(usize, usize)>>,
    second_requested: bool,
}
impl TrialEvaluation {
    pub(crate) fn clear(&mut self) {
        for mut witness in self.applicability.drain(..) {
            witness.inputs.fill(f64::NAN);
            witness.inputs.clear();
            witness.predicates.clear();
            self.spare_witnesses.push(witness);
        }
        self.values.fill(f64::NAN);
        self.values.clear();
        self.first.fill(f64::NAN);
        self.first.clear();
        self.second.fill(f64::NAN);
        self.second.clear();
        self.coordinates = 0;
        self.first_requested = false;
        self.second_requested = false;
    }
    pub(crate) fn first(&self, row: usize, axis: usize) -> f64 {
        self.first_layout.offset(row, axis).and_then(|offset| self.first.get(offset)).copied().unwrap_or(0.0)
    }
    pub(crate) fn first_entries(&self, row: usize) -> impl Iterator<Item = (usize, f64)> + '_ {
        self.first_layout.entries(row).map(|(axis, offset)| (axis, self.first[offset]))
    }
    pub(crate) fn second(&self, row: usize, i: usize, j: usize) -> f64 {
        let (i, j) = (i.min(j), i.max(j));
        self.second_pairs
            .binary_search(&(i, j))
            .ok()
            .map_or(0.0, |offset| {
                self.second[row * self.second_pairs.len() + offset]
            })
    }
    pub(crate) fn project_into(
        &self,
        rows: &[usize],
        axes: &[usize],
        order: DerivativeOrder,
        witnesses: &[Arc<pse_model::applicability::Node>],
        pairs: &Arc<Vec<(usize, usize)>>,
        first: &Arc<FirstLayout>,
        first_sources: &[Option<usize>],
        target: &mut Self,
    ) -> Result<(), MathError> {
        target.clear();
        target
            .values
            .try_reserve_exact(rows.len())
            .map_err(|_| MathError::Limit("projected value cache"))?;
        if order >= DerivativeOrder::First {
            target
                .first
                .try_reserve_exact(first.axes.len())
                .map_err(|_| MathError::Limit("projected first cache"))?;
        }
        if order >= DerivativeOrder::Second {
            target
                .second
                .try_reserve_exact(
                    rows.len()
                        .checked_mul(pairs.len())
                        .ok_or(MathError::Limit("projected second cache"))?,
                )
                .map_err(|_| MathError::Limit("projected second cache"))?;
        }
        for &row in rows {
            target.values.push(
                *self
                    .values
                    .get(row)
                    .ok_or_else(|| MathError::Contract("projected output ordinal".into()))?,
            );
            if order >= DerivativeOrder::Second {
                for &(i, j) in pairs.iter() {
                    target.second.push(self.second(row, axes[i], axes[j]));
                }
            }
        }
        if order >= DerivativeOrder::First {
            for offset in first_sources {
                target.first.push(offset.map_or(0.0, |offset| self.first[offset]));
            }
        }
        target.first_layout = first.clone();
        target.first_requested = order >= DerivativeOrder::First;
        target.second_pairs = pairs.clone();
        target.second_requested = order >= DerivativeOrder::Second;
        for witness in &self.applicability {
            if witnesses
                .iter()
                .any(|plan| Arc::ptr_eq(plan, &witness.plan) || plan == &witness.plan)
            {
                let mut projected = target
                    .spare_witnesses
                    .iter()
                    .position(|candidate| {
                        Arc::ptr_eq(&candidate.plan, &witness.plan)
                            || candidate.plan == witness.plan
                    })
                    .map(|index| target.spare_witnesses.swap_remove(index))
                    .unwrap_or_else(|| ApplicabilityWitness {
                        plan: witness.plan.clone(),
                        predicates: Vec::new(),
                        inputs: Vec::new(),
                    });
                projected.predicates.clear();
                projected.inputs.clear();
                projected
                    .predicates
                    .try_reserve_exact(witness.predicates.len())
                    .map_err(|_| MathError::Limit("projected applicability predicates"))?;
                projected
                    .inputs
                    .try_reserve_exact(witness.inputs.len())
                    .map_err(|_| MathError::Limit("projected applicability inputs"))?;
                projected.predicates.extend_from_slice(&witness.predicates);
                projected.inputs.extend_from_slice(&witness.inputs);
                target
                    .applicability
                    .try_reserve(1)
                    .map_err(|_| MathError::Limit("projected applicability witnesses"))?;
                target.applicability.push(projected);
            }
        }
        target.coordinates = axes.len();
        Ok(())
    }
    pub(crate) fn second_entries(
        &self,
        row: usize,
    ) -> impl Iterator<Item = (usize, usize, f64)> + '_ {
        self.second_pairs
            .iter()
            .enumerate()
            .map(move |(offset, &(i, j))| {
                (i, j, self.second[row * self.second_pairs.len() + offset])
            })
    }
    pub(crate) fn into_evaluation(self) -> Evaluation {
        let n = self.coordinates;
        let jacobian = if !self.first_requested {
            Vec::new()
        } else {
            let mut dense = vec![0.0; self.values.len() * n];
            for row in 0..self.values.len() {
                for (axis, value) in self.first_entries(row) {
                    dense[row * n + axis] = value;
                }
            }
            dense
        };
        let hessians = if !self.second_requested {
            Vec::new()
        } else {
            let mut dense = vec![0.0; self.values.len() * n * n];
            for row in 0..self.values.len() {
                for i in 0..n {
                    for j in 0..n {
                        dense[(row * n + i) * n + j] = self.second(row, i, j);
                    }
                }
            }
            dense
        };
        Evaluation {
            applicability: self
                .applicability
                .iter()
                .flat_map(ApplicabilityWitness::hydrate)
                .collect(),
            values: self.values,
            jacobian,
            hessians,
        }
    }
}
/// Current attempt-owned evaluator realization, distinct from the prepared capacity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkerCensus {
    /// Numeric frame entries currently initialized for admitted orders.
    pub frame_entries: usize,
    /// Actual owned vector capacity, in scalar entries.
    pub frame_capacity: usize,
    /// Orders whose mutable templates have actually been cloned for this attempt.
    pub instantiated_orders: [bool; 3],
    /// Complete prepaid numeric and template-clone allowance.
    pub admitted_bytes: usize,
    /// Allowance consumed by current frame and instantiated order populations.
    /// This is conservative owner accounting, not allocator-observed bytes or RSS.
    pub used_allowance_bytes: usize,
}
/// One order's private executable scratch and its completed instantiation state.
struct WorkerProgram {
    stages: Vec<CompiledStage>,
    // This records executable readiness, never numerical result validity.
    instantiated: bool,
}

/// Worker-local numeric frame and Symbolica scratch.
pub struct Worker {
    // Private native evaluators and numeric frames must drain before the shared
    // body/support allocation owners and execution scope are released.
    programs: EnumMap<DerivativeOrder, Option<WorkerProgram>>,
    frame: Vec<f64>,
    body: CompiledBody,
    scope: Option<pse_kernels::ExecutionScope>,
    direct_product: Option<products::DirectWorker>,
    transpose_worker: Option<Box<products::TransposeWorker>>,
    product_scratch: Vec<f64>,
    prepaid_bytes: usize,
    used_bytes: usize,
}
impl Clone for Worker {
    fn clone(&self) -> Self {
        // A new mutable evaluator never inherits previous values or eagerly copies
        // warmed orders. Instruction templates remain immutable and shared.
        match &self.scope {
            Some(scope) => self.body.worker_scoped(scope.clone()),
            None => self.body.worker(),
        }
    }
}
impl std::fmt::Debug for Worker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Worker")
            .field("body", &self.body)
            .field("frame_entries", &self.frame.len())
            .finish_non_exhaustive()
    }
}
impl Worker {
    /// Copy current owner facts without instantiating an unused order or exporting code.
    pub fn census(&self) -> WorkerCensus {
        WorkerCensus {
            frame_entries: self.frame.len(),
            frame_capacity: self.frame.capacity(),
            instantiated_orders: [
                self.programs[DerivativeOrder::Value].is_some(),
                self.programs[DerivativeOrder::First].is_some(),
                self.programs[DerivativeOrder::Second].is_some() || self.direct_product.is_some(),
            ],
            admitted_bytes: self.prepaid_bytes,
            used_allowance_bytes: self.used_bytes,
        }
    }
    #[cfg(test)]
    pub(crate) fn instantiated_orders(&self) -> Vec<DerivativeOrder> {
        [
            DerivativeOrder::Value,
            DerivativeOrder::First,
            DerivativeOrder::Second,
        ]
        .into_iter()
        .filter(|order| self.census().instantiated_orders[*order as usize])
        .collect()
    }
    /// Required authored input IDs, ordered by their compact numerical offsets.
    pub fn input_formals(&self) -> &[usize] {
        self.body.input_formals()
    }
    /// Attach original execution controls while constructing a nested problem worker.
    pub(crate) fn set_scope(&mut self, scope: pse_kernels::ExecutionScope) {
        self.scope = Some(scope);
    }
    fn check_scope(&self) -> Result<(), MathError> {
        self.scope
            .as_ref()
            .ok_or_else(|| MathError::Contract("detached numeric scratch".into()))?
            .check()
            .map_err(crate::error::scope_error)
    }
    pub(crate) fn detach_controls(&mut self) {
        self.scope = None;
    }
    /// Caller reserves the complete old/new stack overlap before this operation.
    pub(crate) fn reset_scratch(&mut self) -> Result<(), MathError> {
        if self.scope.is_some() {
            return Err(MathError::Contract(
                "numeric reset requires detached controls".into(),
            ));
        }
        self.frame.fill(f64::NAN);
        self.product_scratch.fill(f64::NAN);
        if let Some(direct) = self.direct_product.take() {
            self.direct_product = Some(direct.reset()?);
        }
        if let Some(transpose) = self.transpose_worker.take() {
            self.transpose_worker = Some(Box::new(transpose.reset()?));
        }
        for program in self.programs.values_mut().flatten() {
            program.instantiated = false;
            program.stages = reset_stages(std::mem::take(&mut program.stages))?;
        }
        Ok(())
    }
    /// Evaluate only the requested order, publishing no result on failure.
    pub fn evaluate(
        &mut self,
        inputs: &[f64],
        order: DerivativeOrder,
        providers: &mut ProviderBindings,
        cancelled: &Arc<AtomicBool>,
    ) -> Result<Evaluation, MathError> {
        self.evaluate_trial(inputs, order, providers, cancelled)
            .map(TrialEvaluation::into_evaluation)
    }
    pub(crate) fn evaluate_trial(
        &mut self,
        inputs: &[f64],
        order: DerivativeOrder,
        providers: &mut ProviderBindings,
        cancelled: &Arc<AtomicBool>,
    ) -> Result<TrialEvaluation, MathError> {
        let mut result = TrialEvaluation::default();
        self.evaluate_trial_into(inputs, order, providers, cancelled, &mut result)?;
        Ok(result)
    }
    pub(crate) fn evaluate_trial_into(
        &mut self,
        inputs: &[f64],
        order: DerivativeOrder,
        providers: &mut ProviderBindings,
        cancelled: &Arc<AtomicBool>,
        result: &mut TrialEvaluation,
    ) -> Result<(), MathError> {
        result.clear();
        if self.body.product != ProductKind::SelectedPartials {
            return Err(MathError::Contract(
                "product artifact requires its runtime bindings".into(),
            ));
        }
        let evaluated = self.evaluate_seeded(inputs, order, None, providers, cancelled, result);
        if evaluated.is_err() {
            result.clear();
        }
        evaluated
    }
    /// Evaluate DF(x)v from seeds in requested derivative-coordinate order, without assembling
    /// output-by-coordinate partials. Returned jacobian contains one action per output.
    pub fn evaluate_directional(
        &mut self,
        inputs: &[f64],
        direction: &[f64],
        providers: &mut ProviderBindings,
        cancelled: &Arc<AtomicBool>,
    ) -> Result<Evaluation, MathError> {
        self.evaluate_directional_trial(inputs, direction, providers, cancelled)
            .map(TrialEvaluation::into_evaluation)
    }
    pub(crate) fn evaluate_directional_trial(
        &mut self,
        inputs: &[f64],
        direction: &[f64],
        providers: &mut ProviderBindings,
        cancelled: &Arc<AtomicBool>,
    ) -> Result<TrialEvaluation, MathError> {
        let mut result = TrialEvaluation::default();
        self.evaluate_directional_trial_into(inputs, direction, providers, cancelled, &mut result)?;
        Ok(result)
    }
    pub(crate) fn evaluate_directional_trial_into(
        &mut self,
        inputs: &[f64],
        direction: &[f64],
        providers: &mut ProviderBindings,
        cancelled: &Arc<AtomicBool>,
        result: &mut TrialEvaluation,
    ) -> Result<(), MathError> {
        result.clear();
        if !self.body.directional
            || direction.len() != self.body.coordinate_offsets.len()
            || direction.iter().any(|v| !v.is_finite())
        {
            return Err(MathError::Contract(
                "finite requested-coordinate direction and directional artifact required".into(),
            ));
        }
        let evaluated = self.evaluate_seeded(
            inputs,
            DerivativeOrder::First,
            Some(direction),
            providers,
            cancelled,
            result,
        );
        if evaluated.is_err() {
            result.clear();
        }
        evaluated
    }
    fn evaluate_seeded(
        &mut self,
        inputs: &[f64],
        order: DerivativeOrder,
        direction: Option<&[f64]>,
        providers: &mut ProviderBindings,
        cancelled: &Arc<AtomicBool>,
        result: &mut TrialEvaluation,
    ) -> Result<(), MathError> {
        result.clear();
        self.check_scope()?;
        if inputs.len() != self.body.inputs || inputs.iter().any(|v| !v.is_finite()) {
            return Err(MathError::Contract(
                "finite ordered formal inputs required".into(),
            ));
        }
        if cancelled.load(Ordering::Acquire) {
            return Err(MathError::Cancelled);
        }
        self.prepare_order(order)?;
        let (Some(layout), Some(program)) = (
            self.body.layouts[order].as_ref(),
            self.programs[order].as_mut(),
        ) else {
            return Err(MathError::Contract("uncompiled derivative order".into()));
        };
        let frame = &mut self.frame[..layout.frame_entries(self.body.slots)?];
        let n = layout.coordinates.len();
        for (slot, &value) in inputs.iter().enumerate() {
            layout.clear_frame_slot(frame, slot);
            layout.set_frame_value(frame, slot, 0, value);
        }
        if order >= DerivativeOrder::First {
            if let Some(direction) = direction {
                for (axis, &slot) in self.body.coordinate_offsets.iter().enumerate() {
                    if self.body.product == ProductKind::WeightedHessianAction {
                        layout.set_frame_value(frame, slot, 1 + axis, 1.0);
                        layout.set_frame_value(frame, slot, n, direction[axis]);
                    } else {
                        layout.set_frame_value(frame, slot, 1, direction[axis]);
                    }
                }
            } else {
                for (i, &slot) in layout.coordinates.iter().enumerate() {
                    layout.set_frame_value(frame, slot, 1 + i, 1.0);
                }
            }
        }
        let context = EvaluationContext {
            cancelled,
            max_result_bytes: self.body.limits.scratch_bytes,
        };
        evaluate_stages(
            &mut program.stages,
            frame,
            layout,
            providers,
            &context,
            &mut result.applicability,
            &mut result.spare_witnesses,
        )?;
        context.check().map_err(|cause| MathError::Provider {
            source_id: SemanticId::NIL,
            provider: SemanticId::NIL,
            cause,
        })?;
        result.coordinates = n;
        result.second_pairs = layout.pairs.clone();
        result.second_requested = order >= DerivativeOrder::Second;
        result.first_requested = order >= DerivativeOrder::First;
        result.first_layout = if order >= DerivativeOrder::First {
            self.body.first_layout.clone()
        } else {
            Arc::default()
        };
        // Complete result extent belongs to the admitted operation cache; grow only
        // the actually demanded order, and retain it for the next trial.
        result
            .values
            .try_reserve_exact(self.body.outputs.len())
            .map_err(|_| MathError::Limit("trial value storage"))?;
        if order >= DerivativeOrder::First {
            result
                .first
                .try_reserve_exact(self.body.first_layout.axes.len())
                .map_err(|_| MathError::Limit("trial first storage"))?;
        }
        if order >= DerivativeOrder::Second {
            result
                .second
                .try_reserve_exact(self.body.outputs.len() * layout.pairs.len())
                .map_err(|_| MathError::Limit("trial second storage"))?;
        }
        for (row, &slot) in self.body.outputs.iter().enumerate() {
            let jet = &frame[layout.frame_slot_range(slot)];
            if jet.iter().any(|v| !v.is_finite()) {
                return Err(MathError::Contract(
                    "nonfinite or unassigned result jet".into(),
                ));
            }
            result.values.push(layout.frame_value(frame, slot, 0));
            if order >= DerivativeOrder::First {
                for (axis, _) in self.body.first_layout.entries(row) {
                    result.first.push(layout.frame_value(frame, slot, 1 + axis));
                }
            }
            if order >= DerivativeOrder::Second {
                for k in 0..layout.pairs.len() {
                    let value = layout.frame_value(frame, slot, 1 + n + k) * layout.raw_factor(1 + n + k);
                    result.second.push(value);
                }
            }
        }
        self.check_scope()?;
        Ok(())
    }

    fn prepare_order(&mut self, order: DerivativeOrder) -> Result<(), MathError> {
        self.check_scope()?;
        if self.programs[order]
            .as_ref()
            .is_some_and(|program| program.instantiated)
        {
            return Ok(());
        }
        let layout = self.body.layouts[order]
            .as_ref()
            .ok_or_else(|| MathError::Contract("uncompiled derivative order".into()))?;
        let template = self.body.programs[order]
            .as_ref()
            .ok_or_else(|| MathError::Contract("uncompiled derivative order".into()))?;
        let entries = layout.frame_entries(self.body.slots)?;
        let frame_growth = entries
            .saturating_sub(self.frame.len())
            .checked_mul(size_of::<f64>())
            .ok_or(MathError::Limit("order worker frame"))?;
        let clone_bytes = if self.programs[order].is_none() {
            self.body.order_worker_bytes[order]
        } else {
            0
        };
        let used = self
            .used_bytes
            .checked_add(frame_growth)
            .and_then(|n| n.checked_add(clone_bytes))
            .filter(|n| *n <= self.prepaid_bytes)
            .ok_or(MathError::Limit("prepaid order worker storage"))?;
        // The enclosing worker's complete allowance was prepaid. Check its exact
        // order populations before any frame growth or public library clone; no
        // optimizer, symbolic differentiation or vectorization runs in callbacks.
        if entries > self.frame.len() {
            self.frame
                .try_reserve_exact(entries - self.frame.len())
                .map_err(|_| MathError::Limit("order worker frame allocation"))?;
            self.frame.resize(entries, f64::NAN);
            // A failed instantiation retains the grown frame for its retry.
            self.used_bytes += frame_growth;
        }
        if self.programs[order].is_none() {
            let _phase = pse_diagnostics::observation::current().map(|context| {
                context.count("math.evaluator_template_instantiations", "count", 1);
                context.phase("math.evaluator_template_instantiation")
            });
            let mut stages = template.clone();
            #[cfg(test)]
            INSTANTIATION_PROBE.with_borrow_mut(|probe| probe.template_clones += 1);
            instantiate_stages(
                &mut stages,
                self.scope.as_ref().ok_or_else(|| {
                    MathError::Contract("detached numeric scratch".into())
                })?,
            )?;
            self.check_scope()?;
            self.programs[order] = Some(WorkerProgram {
                stages,
                instantiated: true,
            });
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count(
                    "math.evaluator_template_instantiations_completed",
                    "count",
                    1,
                );
            }
        } else if let Some(program) = &mut self.programs[order] {
            let scope = self.scope.as_ref().ok_or_else(|| {
                MathError::Contract("detached numeric scratch".into())
            })?;
            instantiate_stages(&mut program.stages, scope)?;
            scope.check().map_err(crate::error::scope_error)?;
            program.instantiated = true;
        }
        self.used_bytes = used;
        Ok(())
    }
}

// Consuming map_coeff moves the instruction/result-index vectors. In Symbolica
// 3.0.1 it allocates a replacement full numeric stack before dropping the old one.
// Preserve only the indexed constant interval; values equal to constants elsewhere
// are still poisoned. Registered/external function state is not transferable.
fn reset_stages(stages: Vec<CompiledStage>) -> Result<Vec<CompiledStage>, MathError> {
    stages
        .into_iter()
        .map(|stage| {
            Ok(match stage {
                CompiledStage::Block {
                    evaluator,
                    resettable,
                    components,
                    inputs,
                    outputs,
                    gather,
                    scatter,
                    mut arguments,
                    mut values,
                    source,
                    checks,
                    checked_ends,
                } => {
                    if !resettable {
                        return Err(MathError::Contract(
                            "unsupported evaluator scratch reset".into(),
                        ));
                    }
                    let evaluator = evaluator.reset()?;
                    // Absent structural arguments are immutable zeros. Only
                    // caller-written argument slots lose their numeric contents.
                    arguments.fill(0.0);
                    for &(argument, _) in &gather { arguments[argument] = f64::NAN; }
                    values.fill(f64::NAN);
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
                    }
                }
                CompiledStage::Applicability {
                    stages,
                    mut frame,
                    reads,
                    layout,
                    predicates,
                    inputs,
                    token,
                    plan,
                } => {
                    frame.fill(f64::NAN);
                    CompiledStage::Applicability {
                        stages: reset_stages(stages)?,
                        frame,
                        reads,
                        layout,
                        predicates,
                        inputs,
                        token,
                        plan,
                    }
                }
                CompiledStage::Domain {
                    stages,
                    mut frame,
                    reads,
                    layout,
                    argument,
                    token,
                    lineage,
                } => {
                    frame.fill(f64::NAN);
                    CompiledStage::Domain {
                        stages: reset_stages(stages)?,
                        frame,
                        reads,
                        layout,
                        argument,
                        token,
                        lineage,
                    }
                }
                CompiledStage::Branch {
                    require_separation,
                    comparison,
                    left,
                    right,
                    then,
                    otherwise,
                } => CompiledStage::Branch {
                    require_separation,
                    comparison,
                    left,
                    right,
                    then: reset_stages(then)?,
                    otherwise: reset_stages(otherwise)?,
                },
                CompiledStage::Provider {
                    partial,
                    inputs,
                    outputs,
                    request,
                    buffer,
                    binding: _,
                    mut arguments,
                    lift,
                    source,
                } => {
                    // ProviderLift owns another mutable library evaluator; retain no such
                    // product until its reset contract is qualified independently.
                    if lift.is_some() {
                        return Err(MathError::Contract(
                            "unsupported provider lift scratch reset".into(),
                        ));
                    }
                    arguments.fill(f64::NAN);
                    CompiledStage::Provider {
                        partial,
                        inputs,
                        outputs,
                        request,
                        buffer,
                        binding: None,
                        arguments,
                        lift,
                        source,
                    }
                }
                stage @ CompiledStage::Require { .. } => stage,
            })
        })
        .collect()
}

// Stages are in dependency order and scalar destinations are single assignment.
// Union, rather than replacement, covers shared destinations of mutually exclusive
// arms and domain-local slots. Extra reachability is safe; omitted derivatives are not.
fn coordinate_reachability(
    stages: &[Stage],
    symbols: &HashMap<Symbol, usize>,
    support: &mut [Vec<bool>],
) -> Result<(), MathError> {
    for stage in stages {
        match stage {
            Stage::Block {
                expressions,
                outputs,
                ..
            } => {
                for (expression, &output) in expressions.iter().zip(outputs) {
                    let inputs = reads(std::slice::from_ref(expression), symbols)?;
                    for coordinate in 0..support[output].len() {
                        support[output][coordinate] |=
                            inputs.iter().any(|&i| support[i][coordinate]);
                    }
                }
            }
            Stage::Provider {
                inputs, outputs, ..
            } => {
                for &output in outputs.iter().filter(|&&i| i != usize::MAX) {
                    for coordinate in 0..support[output].len() {
                        support[output][coordinate] |=
                            inputs.iter().any(|&i| support[i][coordinate]);
                    }
                }
            }
            Stage::Branch {
                then, otherwise, ..
            } => {
                coordinate_reachability(then, symbols, support)?;
                coordinate_reachability(otherwise, symbols, support)?;
            }
            // Predicate bodies only request values; they cannot contribute derivatives.
            Stage::Domain { .. } | Stage::Applicability { .. } | Stage::Require { .. } => {}
        }
    }
    Ok(())
}

fn construction_layout_shape_bytes(n: usize, width: usize) -> Result<usize, MathError> {
    let shape = n
        .checked_mul(size_of::<usize>())
        .and_then(|bytes| bytes.checked_add(size_of::<Vec<usize>>() + size_of::<(usize, usize)>()));
    shape
        .and_then(|bytes| width.checked_mul(bytes))
        .and_then(|bytes| bytes.checked_add(n.checked_mul(size_of::<usize>())?))
        .and_then(|bytes| bytes.checked_mul(4))
        .and_then(|bytes| bytes.checked_add(size_of::<JetLayout>()))
        .ok_or(MathError::Limit(
            "selected Taylor layout construction extent",
        ))
}

fn active_block_axes(
    inputs: &[usize],
    outputs: &[usize],
    numeric: &BTreeSet<usize>,
    support: &[Vec<bool>],
    axes: usize,
    order: DerivativeOrder,
) -> Vec<usize> {
    (0..axes)
        .filter(|&axis| {
            order > DerivativeOrder::Value
                && outputs.iter().any(|output| numeric.contains(output))
                && inputs.iter().any(|&input| support[input][axis])
        })
        .collect()
}

fn numeric_slots(
    stages: &[Stage],
    outputs: &[usize],
    symbols: &HashMap<Symbol, usize>,
) -> Result<BTreeSet<usize>, MathError> {
    fn collect(
        stages: &[Stage],
        needed: &mut BTreeSet<usize>,
        seen: &mut BTreeSet<usize>,
        symbols: &HashMap<Symbol, usize>,
    ) -> Result<(), MathError> {
        for stage in stages.iter().rev() {
            match stage {
                Stage::Block {
                    expressions,
                    outputs,
                    ..
                } => {
                    for (expression, &output) in expressions.iter().zip(outputs) {
                        if needed.remove(&output) {
                            seen.insert(output);
                            let inputs = reads(std::slice::from_ref(expression), symbols)?;
                            needed.extend(&inputs);
                            seen.extend(inputs);
                        }
                    }
                }
                Stage::Provider {
                    inputs, outputs, ..
                } => {
                    if outputs.iter().any(|i| needed.contains(i)) {
                        for output in outputs {
                            if needed.remove(output) {
                                seen.insert(*output);
                            }
                        }
                        needed.extend(inputs);
                        seen.extend(inputs);
                    }
                }
                Stage::Branch {
                    then, otherwise, ..
                } => {
                    let mut a = needed.clone();
                    let mut b = needed.clone();
                    collect(then, &mut a, seen, symbols)?;
                    collect(otherwise, &mut b, seen, symbols)?;
                    *needed = a.union(&b).copied().collect();
                }
                Stage::Require { .. } | Stage::Domain { .. } | Stage::Applicability { .. } => {}
            }
        }
        Ok(())
    }
    let mut needed = outputs.iter().copied().collect();
    let mut seen = BTreeSet::new();
    collect(stages, &mut needed, &mut seen, symbols)?;
    Ok(seen)
}

fn selected_capability(
    stages: &[Stage],
    symbols: &HashMap<Symbol, usize>,
    depends: &mut [bool],
    local_branches: bool,
    numeric: &BTreeSet<usize>,
) -> Result<DerivativeOrder, MathError> {
    let mut available = DerivativeOrder::Second;
    for stage in stages {
        match stage {
            Stage::Block {
                expressions,
                outputs,
                ..
            } => {
                for (expression, &output) in expressions.iter().zip(outputs) {
                    depends[output] |= reads(std::slice::from_ref(expression), symbols)?
                        .iter()
                        .any(|&i| depends[i]);
                }
            }
            Stage::Provider {
                spec,
                partial,
                inputs,
                outputs,
                ..
            } => {
                let remaining = (spec.derivatives.min(spec.smoothness) as usize)
                    .checked_sub(partial.len())
                    .ok_or_else(|| MathError::Contract("provider partial capability".into()))?;
                let dependent = inputs.iter().any(|&i| depends[i]);
                if dependent && outputs.iter().any(|i| numeric.contains(i)) {
                    available = available.min(match remaining {
                        0 => DerivativeOrder::Value,
                        1 => DerivativeOrder::First,
                        _ => DerivativeOrder::Second,
                    });
                }
                for &output in outputs.iter().filter(|&&s| s != usize::MAX) {
                    depends[output] |= dependent;
                }
            }
            Stage::Branch {
                continuity,
                left,
                right,
                then,
                otherwise,
                ..
            } => {
                if depends[*left] || depends[*right] {
                    available =
                        available.min(if local_branches && *continuity == DerivativeOrder::Value {
                            DerivativeOrder::Second
                        } else {
                            *continuity
                        });
                }
                available = available.min(selected_capability(
                    then,
                    symbols,
                    depends,
                    local_branches,
                    numeric,
                )?);
                available = available.min(selected_capability(
                    otherwise,
                    symbols,
                    depends,
                    local_branches,
                    numeric,
                )?);
            }
            Stage::Domain { .. } | Stage::Applicability { .. } | Stage::Require { .. } => {}
        }
    }
    Ok(available)
}

// Numeric vectors are already covered by BuildAllowance.entries. These are the
// separately owned descriptors and indices copied by Vec/JetLayout/ProviderSpec clone.
fn cloned_stage_descriptors(stages: &Vec<CompiledStage>) -> Option<usize> {
    fn indices<T>(values: &Vec<T>) -> Option<usize> {
        values.capacity().checked_mul(size_of::<T>())
    }
    fn layout(layout: &JetLayout) -> Option<usize> {
        layout
            .shape
            .iter()
            .try_fold(indices(&layout.shape)?, |n, shape| {
                n.checked_add(indices(shape)?)
            })?
            .checked_add(indices(&layout.coordinates)?)
    }
    stages.iter().try_fold(indices(stages)?, |n, stage| {
        let bytes = match stage {
            CompiledStage::Applicability {
                stages,
                layout: l,
                predicates,
                inputs,
                reads,
                ..
            } => cloned_stage_descriptors(stages)?
                .checked_add(layout(l)?)?
                .checked_add(indices(predicates)?)?
                .checked_add(indices(inputs)?)?
                .checked_add(indices(reads)?)?,
            CompiledStage::Domain {
                stages,
                layout: l,
                reads,
                ..
            } => cloned_stage_descriptors(stages)?
                .checked_add(layout(l)?)?
                .checked_add(indices(reads)?)?,
            CompiledStage::Block {
                components,
                inputs,
                outputs,
                gather,
                scatter,
                checks,
                checked_ends,
                ..
            } => indices(components)?
                .checked_add(indices(inputs)?)?
                .checked_add(indices(outputs)?)?
                .checked_add(indices(gather)?)?
                .checked_add(indices(scatter)?)?
                .checked_add(indices(checks)?)?
                .checked_add(indices(checked_ends)?)?,
            CompiledStage::Require { .. } => 0,
            CompiledStage::Branch {
                then, otherwise, ..
            } => {
                cloned_stage_descriptors(then)?.checked_add(cloned_stage_descriptors(otherwise)?)?
            }
            CompiledStage::Provider {
                partial,
                inputs,
                outputs,
                request,
                lift,
                ..
            } => {
                let spec = request.spec();
                let shapes = spec
                    .shapes
                    .inputs
                    .iter()
                    .chain(&spec.shapes.outputs)
                    .try_fold(
                        indices(&spec.shapes.inputs)?
                            .checked_add(indices(&spec.shapes.outputs)?)?,
                        |n, shape| {
                            let coordinates = shape
                                .coordinates
                                .iter()
                                .try_fold(indices(&shape.coordinates)?, |n, c| {
                                    n.checked_add(indices(c)?)
                                })?;
                            n.checked_add(indices(&shape.axes)?)?
                                .checked_add(indices(&shape.cells)?)?
                                .checked_add(coordinates)
                        },
                    )?;
                indices(&spec.inputs)?
                    .checked_add(indices(&spec.outputs)?)?
                    .checked_add(shapes)?
                    .checked_add(indices(partial)?)?
                    .checked_add(indices(inputs)?)?
                    .checked_add(indices(outputs)?)?
                    .checked_add(indices(&request.outputs)?)?
                    .checked_add(lift.as_ref().map_or(Some(0), |l| indices(&l.inputs))?)?
            }
        };
        n.checked_add(bytes)
    })
}

struct ValueScope {
    stages: Vec<Stage>,
    parameters: Vec<Atom>,
    symbols: HashMap<Symbol, usize>,
    reads: Vec<usize>,
    offsets: BTreeMap<usize, usize>,
}
fn value_scope(
    stages: &[Stage],
    captures: &[usize],
    parameters: &[Atom],
    symbols: &HashMap<Symbol, usize>,
) -> Result<ValueScope, MathError> {
    let mut selected = BTreeSet::new();
    stage_slots(stages, symbols, &mut selected)?;
    selected.extend(captures);
    let reads: Vec<_> = selected.into_iter().collect();
    let offsets = reads
        .iter()
        .copied()
        .enumerate()
        .map(|(local, parent)| (parent, local))
        .collect();
    let mut stages = stages.to_vec();
    lower_stages(&mut stages, &offsets)?;
    let parameters = reads
        .iter()
        .map(|index| parameters[*index].clone())
        .collect::<Vec<_>>();
    let symbols = symbol_map(&parameters);
    Ok(ValueScope {
        stages,
        parameters,
        symbols,
        reads,
        offsets,
    })
}

struct BuildAllowance {
    local_order: DerivativeOrder,
    operations: usize,
    providers: usize,
    entries: usize,
    /// Instruction storage retained by the template and copied by each attempt worker.
    instructions: usize,
    /// Actual exported instruction population, including provider lifts.
    instruction_count: usize,
    /// Shared native code/template allowance, never copied into worker charges.
    modules: usize,
    selected_first: bool,
    first_variables: Vec<Atom>,
    scalar_inputs: usize,
    selected_partial_calls: usize,
    /// Flat arithmetic modules were included by the selected-source preflight.
    /// Opaque control/provider construction consumes the installed owner's prepaid capacity.
    #[cfg(feature = "compiled-evaluator")]
    native_prepaid: bool,
}

struct SelectedFirstRegion {
    expressions: Vec<Atom>,
    scatter: Vec<(usize, Option<usize>)>,
    output_ends: Vec<usize>,
}
impl SelectedFirstRegion {
    fn new(expressions: &[Atom], outputs: &[usize], layout: &JetLayout,
        scope: &pse_kernels::ExecutionScope, allowance: &mut BuildAllowance) -> Result<Self, MathError> {
        let mut selected = Self { expressions: Vec::new(), scatter: Vec::new(), output_ends: Vec::new() };
        for (expression, &slot) in expressions.iter().zip(outputs) {
            for destination in layout.frame_slot_range(slot) {
                scope.check().map_err(crate::error::scope_error)?;
                let component = layout.frame_component(destination);
                let emitted = if component == 0 {
                    expression.clone()
                } else {
                    allowance.selected_partial_calls = allowance.selected_partial_calls.checked_sub(1)
                        .ok_or(MathError::Limit("selected first construction calls"))?;
                    crate::construction::additional(expression.as_view().get_byte_size().checked_mul(8)
                        .ok_or(MathError::Limit("selected first construction"))?)?;
                    if let Some(context) = pse_diagnostics::observation::current() {
                        context.count("math.selected_first_constructions", "count", 1);
                    }
                    layout.differentiate(expression, &allowance.first_variables, component)?
                };
                selected.scatter.push((destination, Some(selected.expressions.len())));
                selected.expressions.push(emitted);
            }
            selected.output_ends.push(selected.expressions.len());
        }
        Ok(selected)
    }
}
fn provider_request_order(
    order: DerivativeOrder,
    partials: usize,
) -> Result<DerivativeOrder, MathError> {
    match order as usize + partials {
        0 => Ok(DerivativeOrder::Value),
        1 => Ok(DerivativeOrder::First),
        2 => Ok(DerivativeOrder::Second),
        _ => Err(MathError::Contract(
            "external derivative order exhausted by explicit partial".into(),
        )),
    }
}
fn stage_retained_bytes(items: &[Stage], seen: &mut BTreeSet<usize>) -> usize {
    fn lineage_bytes(
        lineage: &Arc<pse_model::diagnostic::ValidityLineage>,
        seen: &mut BTreeSet<usize>,
    ) -> usize {
        if seen.insert(Arc::as_ptr(lineage) as usize) {
            size_of_val(lineage.as_ref()) + lineage.heap_bytes() + 2 * size_of::<usize>()
        } else {
            0
        }
    }
    size_of_val(items)
        + items
            .iter()
            .map(|s| match s {
                Stage::Block {
                    expressions,
                    outputs,
                    ..
                } => {
                    size_of_val(expressions.as_slice())
                        + expressions
                            .iter()
                            .map(|a| a.as_view().get_byte_size())
                            .sum::<usize>()
                        + size_of_val(outputs.as_slice())
                }
                Stage::Branch {
                    then, otherwise, ..
                } => stage_retained_bytes(then, seen) + stage_retained_bytes(otherwise, seen),
                Stage::Domain {
                    stages: local,
                    lineage,
                    ..
                } => stage_retained_bytes(local, seen) + lineage_bytes(lineage, seen),
                Stage::Applicability {
                    stages: local,
                    predicates,
                    inputs,
                    plan,
                    ..
                } => {
                    stage_retained_bytes(local, seen)
                        + (predicates.capacity() + inputs.capacity()) * size_of::<usize>()
                        + plan.retained_bytes()
                }
                Stage::Provider {
                    inputs,
                    outputs,
                    spec,
                    partial,
                    ..
                } => {
                    size_of_val(inputs.as_slice())
                        + size_of_val(partial.as_slice())
                        + size_of_val(outputs.as_slice())
                        + size_of_val(spec.inputs.as_slice())
                        + size_of_val(spec.outputs.as_slice())
                        + spec.shapes.retained_bytes()
                }
                Stage::Require { lineage, .. } => lineage
                    .as_ref()
                    .map_or(0, |lineage| lineage_bytes(lineage, seen)),
            })
            .sum::<usize>()
}

/// A demanded arithmetic description, owned with immutable selected support. Control
/// stages retain their barriers; their local value layouts are prepared once as well.
#[derive(Clone, Debug)]
struct PreparedStages {
    stages: Vec<PreparedStage>,
}
#[derive(Clone, Debug)]
struct PreparedStage {
    stage: Stage,
    region: Option<PreparedArithmeticRegion>,
    scope: Option<PreparedValueScope>,
    children: Vec<PreparedStages>,
}
#[derive(Clone, Debug)]
struct PreparedArithmeticRegion {
    inputs: Vec<usize>,
    parameters: Vec<Atom>,
    checks: Vec<SourceCheck>,
}
#[derive(Clone, Debug)]
struct PreparedValueScope {
    reads: Vec<usize>,
    offsets: BTreeMap<usize, usize>,
}
impl PreparedStages {
    fn new(
        stages: &[Stage],
        parameters: &[Atom],
        symbols: &HashMap<Symbol, usize>,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self, MathError> {
        if cancel.load(Ordering::Acquire) {
            return Err(MathError::Cancelled);
        }
        let source = stage_retained_bytes(stages, &mut BTreeSet::new());
        let parameter_bytes = parameters
            .iter()
            .try_fold(0usize, |bytes, atom| {
                bytes.checked_add(size_of::<Atom>() + atom.as_view().get_byte_size())
            })
            .ok_or(MathError::Limit("prepared arithmetic parameters"))?;
        // Prepay clones, region maps and compact nested-scope maps before forming
        // descriptions; each recursive child has its own source-qualified allowance.
        let allowance = source
            .checked_mul(4)
            .and_then(|bytes| {
                parameter_bytes
                    .checked_add(parameters.len().checked_mul(128)?)?
                    .checked_mul(stages.len())?
                    .checked_add(bytes)
            })
            .and_then(|bytes| {
                stages
                    .len()
                    .checked_mul(size_of::<PreparedStage>())?
                    .checked_add(bytes)
            })
            .ok_or(MathError::Limit("prepared arithmetic description"))?;
        crate::construction::additional(allowance)?;
        let mut prepared = Vec::new();
        let mut cursor = 0;
        while cursor < stages.len() {
            if cancel.load(Ordering::Acquire) {
                return Err(MathError::Cancelled);
            }
            let initial = cursor;
            let (next, combined, checks) = arithmetic_region(stages, initial, symbols, cancel)?;
            cursor = next;
            let mut stage = combined.unwrap_or_else(|| stages[initial].clone());
            let mut region = None;
            let mut scope = None;
            let mut children = Vec::new();
            match &mut stage {
                Stage::Block { expressions, .. } => {
                    let inputs = reads(expressions, symbols)?;
                    let parameters = inputs.iter().map(|&i| parameters[i].clone()).collect();
                    region = Some(PreparedArithmeticRegion {
                        inputs,
                        parameters,
                        checks,
                    });
                    if let Some(context) = pse_diagnostics::observation::current() {
                        context.count("math.arithmetic_region_preparations", "count", 1);
                    }
                }
                Stage::Applicability {
                    stages,
                    predicates,
                    inputs,
                    ..
                } => {
                    let captures = predicates
                        .iter()
                        .chain(inputs.iter())
                        .copied()
                        .collect::<Vec<_>>();
                    let local = value_scope(stages, &captures, parameters, symbols)?;
                    children.push(Self::new(
                        &local.stages,
                        &local.parameters,
                        &local.symbols,
                        cancel,
                    )?);
                    scope = Some(PreparedValueScope {
                        reads: local.reads,
                        offsets: local.offsets,
                    });
                    *stages = Vec::new();
                }
                Stage::Domain {
                    stages, argument, ..
                } => {
                    let local = value_scope(stages, &[*argument], parameters, symbols)?;
                    children.push(Self::new(
                        &local.stages,
                        &local.parameters,
                        &local.symbols,
                        cancel,
                    )?);
                    scope = Some(PreparedValueScope {
                        reads: local.reads,
                        offsets: local.offsets,
                    });
                    *stages = Vec::new();
                }
                Stage::Branch {
                    then, otherwise, ..
                } => {
                    children.push(Self::new(then, parameters, symbols, cancel)?);
                    children.push(Self::new(otherwise, parameters, symbols, cancel)?);
                    *then = Vec::new();
                    *otherwise = Vec::new();
                }
                Stage::Provider { .. } | Stage::Require { .. } => {}
            }
            prepared.push(PreparedStage {
                stage,
                region,
                scope,
                children,
            });
        }
        let product = Self { stages: prepared };
        if cancel.load(Ordering::Acquire) {
            return Err(MathError::Cancelled);
        }
        Ok(product)
    }
    fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.stages.capacity() * size_of::<PreparedStage>()
            + self
                .stages
                .iter()
                .map(|prepared| {
                    stage_retained_bytes(
                        std::slice::from_ref(&prepared.stage),
                        &mut BTreeSet::new(),
                    )
                    .saturating_sub(size_of::<Stage>())
                        + prepared.region.as_ref().map_or(0, |region| {
                            region.inputs.capacity() * size_of::<usize>()
                                + region.parameters.capacity() * size_of::<Atom>()
                                + region
                                    .parameters
                                    .iter()
                                    .map(|atom| atom.as_view().get_byte_size())
                                    .sum::<usize>()
                                + region.checks.capacity() * size_of::<SourceCheck>()
                        })
                        + prepared.scope.as_ref().map_or(0, |scope| {
                            scope.reads.capacity() * size_of::<usize>()
                                + scope.offsets.len() * (2 * size_of::<usize>() + 96)
                        })
                        + prepared.children.capacity() * size_of::<PreparedStages>()
                        + prepared
                            .children
                            .iter()
                            .map(Self::retained_bytes)
                            .sum::<usize>()
                })
                .sum::<usize>()
    }
}

fn arithmetic_region(
    stages: &[Stage],
    initial: usize,
    symbols: &HashMap<Symbol, usize>,
    cancel: &Arc<AtomicBool>,
) -> Result<(usize, Option<Stage>, Vec<SourceCheck>), MathError> {
    let mut cursor = initial + 1;
    let mut checks = Vec::new();
    let combined = if let Stage::Block {
        expressions,
        outputs,
        source,
    } = &stages[initial]
    {
        let mut produced = outputs.iter().copied().collect::<BTreeSet<_>>();
        let mut end = outputs.len();
        checks.push(SourceCheck {
            end_output: end,
            source: *source,
        });
        while let Some(Stage::Block {
            expressions: next,
            outputs: slots,
            source,
        }) = stages.get(cursor)
        {
            if cancel.load(Ordering::Acquire) {
                return Err(MathError::Cancelled);
            }
            if reads(next, symbols)?
                .iter()
                .any(|slot| produced.contains(slot))
            {
                break;
            }
            end = end
                .checked_add(slots.len())
                .ok_or(MathError::Limit("joint arithmetic outputs"))?;
            produced.extend(slots);
            checks.push(SourceCheck {
                end_output: end,
                source: *source,
            });
            cursor += 1;
        }
        if cursor > initial + 1 {
            let bytes = stages[initial..cursor]
                .iter()
                .try_fold(0usize, |bytes, stage| {
                    let Stage::Block {
                        expressions,
                        outputs,
                        ..
                    } = stage
                    else {
                        return None;
                    };
                    expressions
                        .iter()
                        .try_fold(bytes, |n, expr| {
                            n.checked_add(expr.as_view().get_byte_size())
                        })?
                        .checked_add(outputs.len().checked_mul(size_of::<usize>())?)
                })
                .ok_or(MathError::Limit("joint arithmetic construction"))?;
            crate::construction::additional(
                bytes
                    .checked_mul(2)
                    .ok_or(MathError::Limit("joint arithmetic construction"))?,
            )?;
            let mut joined = expressions.clone();
            let mut slots = outputs.clone();
            for sibling in &stages[initial + 1..cursor] {
                let Stage::Block {
                    expressions,
                    outputs,
                    ..
                } = sibling
                else {
                    break;
                };
                joined.extend(expressions.iter().cloned());
                slots.extend(outputs);
            }
            Some(Stage::Block {
                expressions: joined,
                outputs: slots,
                source: *source,
            })
        } else {
            None
        }
    } else {
        None
    };
    Ok((cursor, combined, checks))
}

#[allow(
    clippy::too_many_arguments,
    reason = "Compilation receives distinct symbol, budget, and cancellation contexts"
)]
fn compile_stages(
    program: &PreparedStages,
    coordinate_support: &[Vec<bool>],
    numeric: &BTreeSet<usize>,
    layout: &JetLayout,
    options: Optimization,
    limits: EvaluationLimits,
    execution_scope: &pse_kernels::ExecutionScope,
    allowance: &mut BuildAllowance,
    scalars: &mut library::ScalarCache,
) -> Result<Vec<CompiledStage>, MathError> {
    execution_scope.check().map_err(crate::error::scope_error)?;
    let cancelled = execution_scope.cancellation();
    let mut result = vec![];
    for prepared in &program.stages {
        let stage = &prepared.stage;
        execution_scope.check().map_err(crate::error::scope_error)?;
        result.push(match stage {
            Stage::Applicability {
                stages: _,
                predicates,
                inputs,
                token,
                plan,
            } => {
                let scope = prepared.scope.as_ref().ok_or_else(|| {
                    MathError::Contract("missing prepared applicability scope".into())
                })?;
                let value_layout = JetLayout::new(vec![], DerivativeOrder::Value, limits)?;
                allowance.entries = allowance
                    .entries
                    .checked_add(scope.reads.len())
                    .ok_or(MathError::Limit("applicability scratch"))?;
                limits.allocation(allowance.entries)?;
                CompiledStage::Applicability {
                    stages: compile_stages(
                        &prepared.children[0],
                        &[],
                        &BTreeSet::new(),
                        &value_layout,
                        options,
                        limits,
                        execution_scope,
                        allowance,
                        scalars,
                    )?,
                    frame: vec![f64::NAN; scope.reads.len()],
                    reads: scope.reads.clone(),
                    layout: value_layout,
                    predicates: predicates.iter().map(|slot| scope.offsets[slot]).collect(),
                    inputs: inputs.iter().map(|slot| scope.offsets[slot]).collect(),
                    token: *token,
                    plan: plan.clone(),
                }
            }
            Stage::Domain {
                stages: _,
                argument,
                token,
                lineage,
            } => {
                let scope = prepared
                    .scope
                    .as_ref()
                    .ok_or_else(|| MathError::Contract("missing prepared domain scope".into()))?;
                let value_layout = JetLayout::new(vec![], DerivativeOrder::Value, limits)?;
                allowance.entries = allowance
                    .entries
                    .checked_add(scope.reads.len())
                    .ok_or(MathError::Limit("domain predicate scratch"))?;
                limits.allocation(allowance.entries)?;
                CompiledStage::Domain {
                    stages: compile_stages(
                        &prepared.children[0],
                        &[],
                        &BTreeSet::new(),
                        &value_layout,
                        options,
                        limits,
                        execution_scope,
                        allowance,
                        scalars,
                    )?,
                    frame: vec![f64::NAN; scope.reads.len()],
                    reads: scope.reads.clone(),
                    layout: value_layout,
                    argument: scope.offsets[argument],
                    token: *token,
                    lineage: lineage.clone(),
                }
            }
            Stage::Block {
                expressions,
                outputs,
                source,
            } => {
                let region = prepared.region.as_ref().ok_or_else(|| {
                    MathError::Contract("missing prepared arithmetic region".into())
                })?;
                let inputs = &region.inputs;
                let params = &region.parameters;
                let active = active_block_axes(
                    inputs,
                    outputs,
                    numeric,
                    coordinate_support,
                    layout.coordinates.len(),
                    layout.order,
                )
                .into_iter()
                .map(|axis| (axis, layout.coordinates[axis]))
                .collect::<Vec<_>>();
                let pairs = layout
                    .pairs
                    .iter()
                    .filter_map(|&(i, j)| {
                        Some((
                            active.iter().position(|&(axis, _)| axis == i)?,
                            active.iter().position(|&(axis, _)| axis == j)?,
                        ))
                    })
                    .collect();
                let local = JetLayout::with_pairs(
                    active.iter().map(|&(_, slot)| slot).collect(),
                    layout.order,
                    pairs,
                    limits,
                )?;
                let mut components = vec![0];
                if layout.order >= DerivativeOrder::First {
                    components.extend(active.iter().map(|&(i, _)| 1 + i));
                }
                if layout.order >= DerivativeOrder::Second {
                    for &(i, j) in local.pairs.iter() {
                        let pair = (active[i].0, active[j].0);
                        let position = layout
                            .pairs
                            .iter()
                            .position(|&p| p == pair)
                            .ok_or_else(|| MathError::Contract("Taylor subset pair".into()))?;
                        components.push(1 + layout.coordinates.len() + position);
                    }
                }
                // A mixed Taylor component is absent when any differentiated axis
                // is outside this input's conservative support. Branch alternatives
                // and composed provider reachability are already included above.
                let zero_components = inputs
                    .iter()
                    .enumerate()
                    .flat_map(|(parameter, &input)| {
                        local
                            .shape
                            .iter()
                            .enumerate()
                            .skip(1)
                            .filter_map(|(component, powers)| {
                                powers
                                    .iter()
                                    .enumerate()
                                    .any(|(axis, power)| {
                                        *power > 0 && !coordinate_support[input][active[axis].0]
                                    })
                                    .then_some((parameter, component))
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();
                let selected_count = outputs.iter().map(|&slot| layout.frame_slot_range(slot).len()).sum::<usize>();
                let selected_first = allowance.selected_first;
                #[cfg(test)]
                let selected_first = selected_first && !FORCE_TAYLOR_FIRST.get();
                let selected = if selected_first
                    && inputs.iter().all(|&slot| slot < allowance.scalar_inputs)
                    && selected_count < outputs.len() * local.width()
                {
                    Some(SelectedFirstRegion::new(expressions, outputs, layout, execution_scope, allowance)?)
                } else { None };
                let inspected = if let Some(selected) = &selected {
                    library::bounded_scalar_evaluator(
                        &selected.expressions, params, options, cancelled,
                        EvaluationLimits { operations: allowance.operations, ..limits }, scalars,
                    )?
                } else { library::bounded_evaluator(
                    *source,
                    expressions,
                    params,
                    &local,
                    &zero_components,
                    options,
                    cancelled,
                    limits,
                    allowance.operations,
                    allowance.entries,
                    scalars,
                )? };
                let operations = operation_count(inspected.facts.operations);
                allowance.operations = allowance
                    .operations
                    .checked_sub(operations)
                    .ok_or(MathError::Limit("compiled operations"))?;
                let evaluator = inspected.evaluator;
                let input_len = evaluator.get_input_len();
                let output_len = evaluator.get_output_len();
                if input_len != if selected.is_some() { inputs.len() } else { inputs.len() * local.width() }
                    || output_len != selected.as_ref().map_or(outputs.len() * local.width(), |selected| selected.expressions.len())
                {
                    return Err(MathError::Contract("library vectorization layout".into()));
                }
                let storage = inspected.facts.storage;
                allowance.entries = allowance
                    .entries
                    .checked_add(
                        input_len + output_len + components.len() + storage.numeric_entries,
                    )
                    .ok_or(MathError::Limit("evaluator scratch"))?;
                limits.allocation(allowance.entries)?;
                allowance.instructions = allowance
                    .instructions
                    .checked_add(storage.instruction_bytes)
                    .ok_or(MathError::Limit("evaluator instruction storage"))?;
                allowance.instruction_count = allowance
                    .instruction_count
                    .checked_add(storage.instruction_count)
                    .ok_or(MathError::Limit("evaluator instruction population"))?;
                let _export = inspected.export;
                let resettable = inspected.facts.resettable;
                let evaluator = {
                    match options.evaluator {
                        library::Evaluator::Interpreted => BlockEvaluator::Interpreted(evaluator),
                        library::Evaluator::CompiledScalar { module_bytes } => {
                            #[cfg(feature = "compiled-evaluator")]
                            {
                                if module_bytes == 0 {
                                    return Err(MathError::Limit("zero compiled module allowance"));
                                }
                                allowance.modules = allowance
                                    .modules
                                    .checked_add(module_bytes)
                                    .ok_or(MathError::Limit("compiled module storage"))?;
                                if !allowance.native_prepaid {
                                    crate::construction::additional(module_bytes)?;
                                }
                                BlockEvaluator::Native(crate::native_compiled::compile(
                                    &evaluator,
                                    &storage,
                                    &_export,
                                    module_bytes,
                                    execution_scope,
                                )?)
                            }
                            #[cfg(not(feature = "compiled-evaluator"))]
                            {
                                let _ = module_bytes;
                                return Err(MathError::Contract(
                                    "compiled scalar evaluator requires compiled-evaluator feature"
                                        .into(),
                                ));
                            }
                        }
                    }
                };
                drop(_export);
                let checked_ends = region.checks.iter().map(|check| {
                    selected.as_ref().map_or(check.end_output * local.width(), |selected| selected.output_ends[check.end_output - 1])
                }).collect();
                let (gather, scatter) = if let Some(selected) = selected {
                    components = vec![0];
                    (inputs.iter().enumerate().filter_map(|(argument, &slot)| layout.frame_offset(slot, 0).map(|source| (argument, source))).collect(), selected.scatter)
                } else {
                    let component_width = components.len();
                    let gather = inputs.iter().enumerate().flat_map(|(row, &slot)| {
                        components.iter().enumerate().filter_map(move |(local, &component)| layout.frame_offset(slot, component).map(|source| (row * component_width + local, source)))
                    }).collect();
                    let scatter = outputs.iter().enumerate().flat_map(|(row, &slot)| {
                    layout.frame_slot_range(slot).map(|offset| {
                        let component = layout.frame_component(offset);
                        (offset, components.binary_search(&component).ok().map(|component_offset| row * local.width() + component_offset))
                    }).collect::<Vec<_>>()
                    }).collect();
                    (gather, scatter)
                };
                CompiledStage::Block {
                    evaluator,
                    resettable,
                    components,
                    inputs: inputs.clone(),
                    outputs: outputs.clone(),
                    gather,
                    scatter,
                    arguments: vec![0.0; input_len],
                    values: vec![0.0; output_len],
                    source: *source,
                    checks: region.checks.clone(),
                    checked_ends,
                }
            }
            Stage::Require {
                argument,
                condition,
                order,
                source,
                lineage,
            } => CompiledStage::Require {
                argument: *argument,
                condition: *condition,
                order: *order,
                source: *source,
                lineage: lineage.clone(),
            },
            Stage::Branch {
                continuity,
                comparison,
                left,
                right,
                then: _,
                otherwise: _,
            } => CompiledStage::Branch {
                require_separation: allowance.local_order > *continuity && left != right,
                comparison: *comparison,
                left: *left,
                right: *right,
                then: compile_stages(
                    &prepared.children[0],
                    coordinate_support,
                    numeric,
                    layout,
                    options,
                    limits,
                    execution_scope,
                    allowance,
                    scalars,
                )?,
                otherwise: compile_stages(
                    &prepared.children[1],
                    coordinate_support,
                    numeric,
                    layout,
                    options,
                    limits,
                    execution_scope,
                    allowance,
                    scalars,
                )?,
            },
            Stage::Provider {
                spec,
                partial,
                inputs,
                outputs,
                source,
            } => {
                allowance.providers = allowance
                    .providers
                    .checked_sub(1)
                    .ok_or(MathError::Limit("provider calls"))?;
                let differentiated = layout.order > DerivativeOrder::Value
                    && outputs.iter().any(|i| numeric.contains(i))
                    && inputs
                        .iter()
                        .any(|&i| coordinate_support[i].iter().any(|&dependent| dependent));
                let request = ProviderRequest {
                    demand: pse_kernels::ProviderDemand::CompleteJets,
                    outputs: outputs
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &slot)| (slot != usize::MAX).then_some(i))
                        .collect(),
                    order: provider_request_order(
                        if differentiated {
                            layout.order
                        } else {
                            DerivativeOrder::Value
                        },
                        partial.len(),
                    )?,
                };
                let request = PreparedProviderRequest::new(Arc::new(spec.clone()), request)
                    .map_err(|e| MathError::Contract(e.to_string()))?;
                let destinations = outputs
                    .iter()
                    .copied()
                    .filter(|&s| s != usize::MAX)
                    .collect();
                let lift = if differentiated && layout.width() > 1 {
                    Some(ProviderLift::compile(
                        inputs.len(),
                        layout,
                        options,
                        EvaluationLimits {
                            operations: allowance.operations,
                            ..limits
                        },
                        cancelled,
                        scalars,
                    )?)
                } else {
                    None
                };
                let lift_storage = lift.as_ref().map(|lift| lift.facts.storage);
                if let Some(storage) = &lift_storage {
                    allowance.instruction_count = allowance
                        .instruction_count
                        .checked_add(storage.instruction_count)
                        .ok_or(MathError::Limit("provider lift instruction population"))?;
                    allowance.instructions = allowance
                        .instructions
                        .checked_add(storage.instruction_bytes)
                        .ok_or(MathError::Limit("provider lift instruction storage"))?;
                }
                if let Some(lift) = &lift {
                    allowance.operations = allowance
                        .operations
                        .checked_sub(operation_count(lift.facts.operations))
                        .ok_or(MathError::Limit("provider lift operations"))?;
                }
                let extents = request.extents();
                let entries = inputs.len() + extents.values + extents.jacobian + extents.hessians;
                allowance.entries = allowance
                    .entries
                    .checked_add(
                        entries
                            + lift.as_ref().map_or(0, |l| {
                                l.scratch.len()
                                    + l.output.len()
                                    + lift_storage.as_ref().map_or(0, |s| s.numeric_entries)
                            }),
                    )
                    .ok_or(MathError::Limit("provider scratch"))?;
                limits.allocation(allowance.entries)?;
                CompiledStage::Provider {
                    partial: partial.clone(),
                    inputs: inputs.clone(),
                    outputs: destinations,
                    buffer: ProviderBuffer::new(&request),
                    request,
                    binding: None,
                    arguments: vec![0.0; inputs.len()],
                    lift,
                    source: *source,
                }
            }
        });
    }
    Ok(result)
}
fn operation_count(c: symbolica::evaluate::OperationCount) -> usize {
    c.additions
        .saturating_add(c.multiplications)
        .saturating_add(c.inversions)
        .saturating_add(c.function_calls)
}
fn evaluate_stages(
    stages: &mut [CompiledStage],
    frame: &mut [f64],
    layout: &JetLayout,
    providers: &mut ProviderBindings,
    context: &EvaluationContext<'_>,
    observations: &mut Vec<ApplicabilityWitness>,
    spare_witnesses: &mut Vec<ApplicabilityWitness>,
) -> Result<(), MathError> {
    for stage in stages {
        if context.cancelled.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        match stage {
            CompiledStage::Applicability {
                stages,
                frame: local,
                reads,
                layout: value_layout,
                predicates,
                inputs,
                token,
                plan,
            } => {
                for (value, slot) in local.iter_mut().zip(reads) {
                    *value = layout.frame_value(frame, *slot, 0);
                }
                evaluate_stages(
                    stages,
                    local,
                    value_layout,
                    providers,
                    context,
                    observations,
                    spare_witnesses,
                )?;
                if predicates
                    .iter()
                    .chain(inputs.iter())
                    .any(|slot| !local[*slot].is_finite())
                {
                    return Err(MathError::Contract(
                        "nonfinite applicability capture".into(),
                    ));
                }
                let mut witness = spare_witnesses
                    .iter()
                    .position(|w| Arc::ptr_eq(&w.plan, plan))
                    .map(|index| spare_witnesses.swap_remove(index))
                    .unwrap_or_else(|| ApplicabilityWitness {
                        plan: Arc::clone(plan),
                        predicates: Vec::new(),
                        inputs: Vec::new(),
                    });
                witness.predicates.clear();
                witness.inputs.clear();
                witness
                    .predicates
                    .try_reserve_exact(predicates.len())
                    .map_err(|_| MathError::Limit("applicability predicate storage"))?;
                witness
                    .inputs
                    .try_reserve_exact(inputs.len())
                    .map_err(|_| MathError::Limit("applicability input storage"))?;
                witness
                    .predicates
                    .extend(predicates.iter().map(|slot| local[*slot] > 0.));
                witness
                    .inputs
                    .extend(inputs.iter().map(|slot| local[*slot]));
                if !plan.admits(&witness.predicates) {
                    let mut assessment = plan.assess(&witness.predicates, &witness.inputs);
                    let mut prior = observations
                        .iter()
                        .flat_map(ApplicabilityWitness::hydrate)
                        .collect::<Vec<_>>();
                    let offset = prior.len();
                    prior.append(&mut assessment.observations);
                    assessment.observations = prior;
                    for index in &mut assessment.refused {
                        *index += offset;
                    }
                    witness.inputs.fill(f64::NAN);
                    witness.inputs.clear();
                    witness.predicates.clear();
                    spare_witnesses.push(witness);
                    return Err(MathError::Applicability(Box::new(assessment)));
                }
                observations.push(witness);
                layout.clear_frame_slot(frame, *token);
            }
            CompiledStage::Domain {
                stages,
                frame: local,
                reads,
                layout: value_layout,
                argument,
                token,
                lineage,
            } => {
                for (value, slot) in local.iter_mut().zip(reads) {
                    *value = layout.frame_value(frame, *slot, 0);
                }
                evaluate_stages(
                    stages,
                    local,
                    value_layout,
                    providers,
                    context,
                    observations,
                    spare_witnesses,
                )?;
                if !Condition::Positive.permits(local[*argument]) {
                    return Err(MathError::Validity(Box::new(lineage.as_ref().clone())));
                }
                layout.clear_frame_slot(frame, *token);
            }
            CompiledStage::Block {
                evaluator,
                gather,
                scatter,
                arguments,
                values,
                source,
                checks,
                checked_ends,
                ..
            } => {
                for &(argument, source) in gather.iter() {
                    arguments[argument] = frame[source];
                }
                evaluator
                    .evaluate(arguments, values)
                    .map_err(|e| MathError::Evaluation {
                        source_id: *source,
                        order: layout.order,
                        detail: e.to_string(),
                    })?;
                let mut first = 0;
                for (check, &end) in checks.iter().zip(checked_ends.iter()) {
                    if values[first..end].iter().any(|v| !v.is_finite()) {
                        return Err(MathError::Domain {
                            source_id: check.source,
                            requirement: "finite arithmetic jet",
                        });
                    }
                    first = end;
                }
                for &(destination, source) in scatter.iter() {
                    frame[destination] = source.map_or(0.0, |offset| values[offset]);
                }
            }
            CompiledStage::Require {
                argument,
                condition,
                order,
                source,
                lineage,
            } => {
                if layout.order >= *order && !condition.permits(layout.frame_value(frame, *argument, 0)) {
                    // A closure range names its lineage (Plan 23 H5).
                    return Err(match lineage {
                        Some(lineage) => MathError::Validity(Box::new(lineage.as_ref().clone())),
                        None => MathError::Domain {
                            source_id: *source,
                            requirement: condition.description(),
                        },
                    });
                }
            }
            CompiledStage::Branch {
                require_separation,
                comparison,
                left,
                right,
                then,
                otherwise,
            } => {
                let a = layout.frame_value(frame, *left, 0);
                let b = layout.frame_value(frame, *right, 0);
                if !a.is_finite() || !b.is_finite() {
                    return Err(MathError::Contract("unassigned branch input".into()));
                }
                if *require_separation && a == b {
                    return Err(MathError::Domain {
                        source_id: SemanticId::NIL,
                        requirement: "branch-local derivative trial is on an unproved control boundary",
                    });
                }
                evaluate_stages(
                    if comparison.select(a, b) {
                        then
                    } else {
                        otherwise
                    },
                    frame,
                    layout,
                    providers,
                    context,
                    observations,
                    spare_witnesses,
                )?;
            }
            CompiledStage::Provider {
                partial,
                inputs,
                outputs,
                request,
                binding,
                buffer,
                arguments,
                lift,
                source,
            } => {
                let bank = providers.identity();
                let slot = match *binding {
                    Some((identity, slot)) if identity == bank => slot,
                    _ => {
                        let slot = providers.resolve(request.key())
                            .ok_or_else(|| MathError::Contract("missing provider worker".into()))?;
                        providers.bind_request(slot, request).map_err(|e| MathError::Contract(e.to_string()))?;
                        *binding = Some((bank, slot));
                        slot
                    }
                };
                for (value, &slot) in arguments.iter_mut().zip(inputs.iter()) {
                    *value = layout.frame_value(frame, slot, 0);
                }
                let error = |cause| MathError::Provider {
                    source_id: *source,
                    provider: request.spec().id,
                    cause,
                };
                let values = providers
                    .fill(slot, buffer, arguments, request, ProviderRuntime::default(), context)
                    .map_err(error)?;
                for (row, &slot) in outputs.iter().enumerate() {
                    let n = inputs.len();
                    let selected = match partial.as_slice() {
                        [] => values.values[row],
                        [i] => values.jacobian[row * n + i],
                        [i, j] => values.hessians[(row * n + i) * n + j],
                        _ => return Err(MathError::Contract("external partial order".into())),
                    };
                    if let Some(lift) = lift.as_mut() {
                        let n = inputs.len();
                        for (parameter, value) in lift.inputs.iter().zip(&mut lift.scratch) {
                            *value = match *parameter {
                                LiftInput::Value => selected,
                                LiftInput::First(i) => {
                                    if let Some(j) = partial.first() {
                                        values.hessians[(row * n + j) * n + i]
                                    } else {
                                        values.jacobian[row * n + i]
                                    }
                                }
                                LiftInput::Second(i, j) => values.hessians[row * n * n + i * n + j],
                                LiftInput::Argument(i, k) => {
                                    layout.frame_value(frame, inputs[i], k) * layout.raw_factor(k)
                                }
                            };
                        }
                        lift.evaluator
                            .try_evaluate(&lift.scratch, &mut lift.output)
                            .map_err(|e| MathError::Evaluation {
                                source_id: *source,
                                order: layout.order,
                                detail: e.to_string(),
                            })?;
                        if lift.output.iter().any(|v| !v.is_finite()) {
                            return Err(MathError::Domain {
                                source_id: *source,
                                requirement: "finite composed provider jet",
                            });
                        }
                        layout.clear_frame_slot(frame, slot);
                        for (component, &value) in lift.output.iter().enumerate() {
                            layout.set_frame_value(frame, slot, component, value);
                        }
                    } else {
                        layout.clear_frame_slot(frame, slot);
                        layout.set_frame_value(frame, slot, 0, selected);
                    }
                }
            }
        }
    }
    Ok(())
}

fn symbol_map(parameters: &[Atom]) -> HashMap<Symbol, usize> {
    parameters
        .iter()
        .enumerate()
        .flat_map(|(i, a)| a.get_all_symbols(false).into_iter().map(move |s| (s, i)))
        .collect()
}
fn stage_slots(
    stages: &[Stage],
    symbols: &HashMap<Symbol, usize>,
    slots: &mut BTreeSet<usize>,
) -> Result<(), MathError> {
    for stage in stages {
        match stage {
            Stage::Block {
                expressions,
                outputs,
                ..
            } => {
                slots.extend(reads(expressions, symbols)?);
                slots.extend(outputs);
            }
            Stage::Require { argument, .. } => {
                slots.insert(*argument);
            }
            Stage::Domain {
                stages,
                argument,
                token,
                ..
            } => {
                slots.extend([*argument, *token]);
                stage_slots(stages, symbols, slots)?;
            }
            Stage::Applicability {
                stages,
                predicates,
                inputs,
                token,
                ..
            } => {
                slots.extend(predicates.iter().chain(inputs).copied());
                slots.insert(*token);
                stage_slots(stages, symbols, slots)?;
            }
            Stage::Branch {
                left,
                right,
                then,
                otherwise,
                ..
            } => {
                slots.extend([*left, *right]);
                stage_slots(then, symbols, slots)?;
                stage_slots(otherwise, symbols, slots)?;
            }
            Stage::Provider {
                inputs, outputs, ..
            } => {
                slots.extend(inputs);
                slots.extend(outputs.iter().copied().filter(|&slot| slot != usize::MAX));
            }
        }
    }
    Ok(())
}
fn lower_stages(stages: &mut [Stage], offsets: &BTreeMap<usize, usize>) -> Result<(), MathError> {
    let lower = |slot: &mut usize| -> Result<(), MathError> {
        if *slot != usize::MAX {
            *slot = offsets
                .get(slot)
                .copied()
                .ok_or_else(|| MathError::Contract("selected formal is missing".into()))?;
        }
        Ok(())
    };
    for stage in stages {
        match stage {
            Stage::Block { outputs, .. } => {
                for output in outputs {
                    lower(output)?;
                }
            }
            Stage::Require { argument, .. } => lower(argument)?,
            Stage::Domain {
                stages,
                argument,
                token,
                ..
            } => {
                lower(argument)?;
                lower(token)?;
                lower_stages(stages, offsets)?;
            }
            Stage::Applicability {
                stages,
                predicates,
                inputs,
                token,
                ..
            } => {
                for slot in predicates.iter_mut().chain(inputs) {
                    lower(slot)?;
                }
                lower(token)?;
                lower_stages(stages, offsets)?;
            }
            Stage::Branch {
                left,
                right,
                then,
                otherwise,
                ..
            } => {
                lower(left)?;
                lower(right)?;
                lower_stages(then, offsets)?;
                lower_stages(otherwise, offsets)?;
            }
            Stage::Provider {
                inputs, outputs, ..
            } => {
                for slot in inputs.iter_mut().chain(outputs) {
                    lower(slot)?;
                }
            }
        }
    }
    Ok(())
}
fn stage_providers(stages: &[Stage]) -> BTreeMap<ProviderKey, ProviderSpec> {
    let mut providers = BTreeMap::new();
    for stage in stages {
        match stage {
            Stage::Provider { spec, .. } => {
                providers.insert(spec.key(), spec.clone());
            }
            Stage::Domain { stages, .. } | Stage::Applicability { stages, .. } => {
                providers.extend(stage_providers(stages));
            }
            Stage::Branch {
                then, otherwise, ..
            } => {
                providers.extend(stage_providers(then));
                providers.extend(stage_providers(otherwise));
            }
            _ => {}
        }
    }
    providers
}
fn reads(expressions: &[Atom], symbols: &HashMap<Symbol, usize>) -> Result<Vec<usize>, MathError> {
    expressions
        .iter()
        .flat_map(|e| e.get_all_symbols(false))
        .filter(|symbol| !Atom::var(*symbol).is_constant())
        .map(|s| {
            symbols
                .get(&s)
                .copied()
                .ok_or_else(|| MathError::Contract("unknown block parameter".into()))
        })
        .collect::<Result<BTreeSet<_>, _>>()
        .map(|s| s.into_iter().collect())
}
fn has_obligations(stages: &[Stage]) -> bool {
    stages.iter().any(|s| {
        matches!(
            s,
            Stage::Require { .. }
                | Stage::Branch { .. }
                | Stage::Provider { .. }
                | Stage::Domain { .. }
                | Stage::Applicability { .. }
        )
    })
}
// Backward demand retains every authored obligation. Dead properties are excluded without hoisting.
fn prune(
    stages: &[Stage],
    needed: &mut BTreeSet<usize>,
    symbols: &HashMap<Symbol, usize>,
    all_effects: bool,
) -> Vec<Stage> {
    let mut result = vec![];
    for stage in stages.iter().rev() {
        match stage {
            Stage::Applicability {
                stages,
                predicates,
                inputs,
                token,
                plan,
            } => {
                if !needed.remove(token) && !all_effects {
                    continue;
                }
                let mut local = predicates
                    .iter()
                    .chain(inputs)
                    .copied()
                    .collect::<BTreeSet<_>>();
                let stages = prune(stages, &mut local, symbols, true);
                needed.extend(local);
                result.push(Stage::Applicability {
                    stages,
                    predicates: predicates.clone(),
                    inputs: inputs.clone(),
                    token: *token,
                    plan: plan.clone(),
                });
            }
            Stage::Domain {
                stages,
                argument,
                token,
                lineage,
            } => {
                if !needed.remove(token) && !all_effects {
                    continue;
                }
                let mut local = BTreeSet::from([*argument]);
                let stages = prune(stages, &mut local, symbols, true);
                needed.extend(local);
                result.push(Stage::Domain {
                    stages,
                    argument: *argument,
                    token: *token,
                    lineage: lineage.clone(),
                });
            }
            Stage::Block {
                expressions,
                outputs,
                source,
            } => {
                let chosen: Vec<_> = outputs
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| needed.contains(s))
                    .map(|(i, &s)| (expressions[i].clone(), s))
                    .collect();
                if !chosen.is_empty() {
                    for (_, s) in &chosen {
                        needed.remove(s);
                    }
                    for (e, _) in &chosen {
                        for symbol in e.get_all_symbols(false) {
                            if let Some(&slot) = symbols.get(&symbol) {
                                needed.insert(slot);
                            }
                        }
                    }
                    result.push(Stage::Block {
                        expressions: chosen.iter().map(|(e, _)| e.clone()).collect(),
                        outputs: chosen.iter().map(|(_, s)| *s).collect(),
                        source: *source,
                    });
                }
            }
            Stage::Require { argument, .. } => {
                if all_effects || needed.contains(argument) {
                    needed.insert(*argument);
                    result.push(stage.clone());
                }
            }
            Stage::Provider {
                spec,
                partial,
                inputs,
                outputs,
                source,
            } => {
                // The call itself is a fallible domain obligation, even after algebraic cancellation.
                let used = outputs.iter().any(|s| needed.contains(s));
                if !used && !all_effects {
                    continue;
                }
                let selected = outputs
                    .iter()
                    .map(|s| {
                        if needed.remove(s) || !used {
                            *s
                        } else {
                            usize::MAX
                        }
                    })
                    .collect();
                needed.extend(inputs);
                result.push(Stage::Provider {
                    spec: spec.clone(),
                    partial: partial.clone(),
                    inputs: inputs.clone(),
                    outputs: selected,
                    source: *source,
                });
            }
            Stage::Branch {
                continuity,
                comparison,
                left,
                right,
                then,
                otherwise,
            } => {
                let mut a = needed.clone();
                let mut b = needed.clone();
                let then = prune(then, &mut a, symbols, all_effects);
                let otherwise = prune(otherwise, &mut b, symbols, all_effects);
                if then.is_empty() && otherwise.is_empty() {
                    continue;
                }
                *needed = a.union(&b).copied().collect();
                needed.insert(*left);
                needed.insert(*right);
                result.push(Stage::Branch {
                    continuity: *continuity,
                    comparison: *comparison,
                    left: *left,
                    right: *right,
                    then,
                    otherwise,
                });
            }
        }
    }
    result.reverse();
    result
}

#[derive(Debug, Default)]
struct Fact {
    expression: Option<Atom>,
    first: BTreeSet<usize>,
    second: BTreeSet<(usize, usize)>,
    /// Actual producer occurrence, without inventing a source for formal inputs.
    source: Option<SemanticId>,
}
/// Private branch/obligation writes over immutable inherited facts. A local scope
/// copies only facts it assigns, regardless of the enclosing body's slot count.
enum Facts<'a> {
    Dense(Vec<Arc<Fact>>),
    Scoped {
        parent: &'a Facts<'a>,
        assigned: BTreeMap<usize, Arc<Fact>>,
    },
}
impl Facts<'_> {
    fn scope(&self) -> Facts<'_> {
        Facts::Scoped {
            parent: self,
            assigned: BTreeMap::new(),
        }
    }
    fn set(&mut self, slot: usize, fact: Arc<Fact>) {
        match self {
            Self::Dense(facts) => facts[slot] = fact,
            Self::Scoped { assigned, .. } => {
                assigned.insert(slot, fact);
            }
        }
    }
    fn written_slots(&self) -> impl Iterator<Item = usize> + '_ {
        match self {
            Self::Dense(_) => None,
            Self::Scoped { assigned, .. } => Some(assigned),
        }
        .into_iter()
        .flat_map(|assigned| assigned.keys().copied())
    }
}
impl std::ops::Index<usize> for Facts<'_> {
    type Output = Arc<Fact>;
    fn index(&self, slot: usize) -> &Self::Output {
        match self {
            Self::Dense(facts) => &facts[slot],
            Self::Scoped { parent, assigned } => {
                assigned.get(&slot).unwrap_or_else(|| &parent[slot])
            }
        }
    }
}
struct SupportAllowance<'a> {
    remaining: &'a mut usize,
    derivatives: usize,
}
impl SupportAllowance<'_> {
    fn consume(&mut self, required: usize, source_id: SemanticId) -> Result<(), MathError> {
        if required > *self.remaining {
            return Err(MathError::WorkLimit {
                source_id,
                resource: "derivative support construction",
                required,
                available: *self.remaining,
                // Support preparation precedes selection of a Taylor layout.
                components: 0,
            });
        }
        *self.remaining -= required;
        Ok(())
    }
    fn derivative(&mut self, source: SemanticId) -> Result<(), MathError> {
        self.consume(1, source)?;
        self.derivatives = self
            .derivatives
            .checked_add(1)
            .ok_or(MathError::Limit("support derivative calls"))?;
        Ok(())
    }
    fn insert(
        &mut self,
        target: &mut BTreeSet<(usize, usize)>,
        pair: (usize, usize),
        source: SemanticId,
    ) -> Result<(), MathError> {
        self.consume(1, source)?;
        target.insert(pair);
        Ok(())
    }
    fn extend(
        &mut self,
        target: &mut BTreeSet<(usize, usize)>,
        values: &BTreeSet<(usize, usize)>,
        source: SemanticId,
    ) -> Result<(), MathError> {
        self.consume(values.len(), source)?;
        target.extend(values);
        Ok(())
    }
    fn scope_facts<'a>(
        &mut self,
        facts: &'a Facts<'_>,
        source: SemanticId,
    ) -> Result<Facts<'a>, MathError> {
        // The scope is constant-size; each assigned fact is charged at its producer.
        self.consume(1, source)?;
        Ok(facts.scope())
    }
    fn equal_facts(&mut self, a: &Fact, b: &Fact, source: SemanticId) -> Result<bool, MathError> {
        // A distinct allocation can still denote exactly the same symbolic fact. Charge
        // a conservative byte-comparison bound before the library equality operation,
        // then charge each support entry comparison that is actually performed.
        let bytes = match (&a.expression, &b.expression) {
            (Some(a), Some(b)) => a
                .as_view()
                .get_byte_size()
                .checked_add(b.as_view().get_byte_size())
                .ok_or(MathError::Limit("support comparison extent"))?,
            _ => 1,
        };
        self.consume(bytes, source)?;
        if a.expression != b.expression
            || a.first.len() != b.first.len()
            || a.second.len() != b.second.len()
        {
            return Ok(false);
        }
        for (a, b) in a.first.iter().zip(&b.first) {
            self.consume(1, source)?;
            if a != b {
                return Ok(false);
            }
        }
        for (a, b) in a.second.iter().zip(&b.second) {
            self.consume(1, source)?;
            if a != b {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
fn dense_second(
    first: &BTreeSet<usize>,
    allowance: &mut SupportAllowance<'_>,
    source: SemanticId,
) -> Result<BTreeSet<(usize, usize)>, MathError> {
    let n = first.len();
    let count = n
        .checked_add(1)
        .and_then(|next| n.checked_mul(next))
        .map(|count| count / 2)
        .ok_or(MathError::Limit("derivative support cardinality"))?;
    allowance.consume(count, source)?;
    Ok(first
        .iter()
        .flat_map(|&i| first.range(i..).map(move |&j| (i, j)))
        .collect())
}
#[allow(
    clippy::too_many_arguments,
    reason = "Analysis propagates separate facts and obligation inventories through branches"
)]
fn analyze(
    stages: &[Stage],
    parameters: &[Atom],
    symbols: &HashMap<Symbol, usize>,
    facts: &mut Facts<'_>,
    controls: &mut BTreeSet<usize>,
    switches: &mut BTreeSet<usize>,
    providers: &mut BTreeMap<ProviderKey, ProviderSpec>,
    obligations: &mut Vec<(Option<Atom>, Condition)>,
    allowance: &mut SupportAllowance<'_>,
    requested: DerivativeOrder,
    coordinates: Option<&BTreeSet<usize>>,
    cancel: Option<&Arc<AtomicBool>>,
    numeric: Option<&BTreeSet<usize>>,
) -> Result<(), MathError> {
    for stage in stages {
        if cancel.is_some_and(|c| c.load(Ordering::Relaxed)) {
            return Err(MathError::Cancelled);
        }
        match stage {
            Stage::Applicability {
                stages,
                predicates,
                inputs,
                token,
                plan,
            } => {
                let source = plan.claim.form;
                let mut local = allowance.scope_facts(facts, source)?;
                analyze(
                    stages,
                    parameters,
                    symbols,
                    &mut local,
                    controls,
                    &mut BTreeSet::new(),
                    providers,
                    obligations,
                    allowance,
                    requested.min(DerivativeOrder::First),
                    coordinates,
                    cancel,
                    numeric,
                )?;
                for slot in predicates.iter().chain(inputs) {
                    allowance.consume(local[*slot].first.len(), source)?;
                    controls.extend(&local[*slot].first);
                }
                allowance.consume(1, source)?;
                facts.set(
                    *token,
                    Arc::new(Fact {
                        expression: Some(Atom::num(0)),
                        source: Some(source),
                        ..Fact::default()
                    }),
                );
            }
            Stage::Domain {
                stages,
                argument,
                token,
                lineage,
            } => {
                let source = lineage.source;
                let mut local = allowance.scope_facts(facts, source)?;
                // Domain boundaries do not constitute branch transitions of a numerical output.
                analyze(
                    stages,
                    parameters,
                    symbols,
                    &mut local,
                    controls,
                    &mut BTreeSet::new(),
                    providers,
                    obligations,
                    allowance,
                    requested.min(DerivativeOrder::First),
                    coordinates,
                    cancel,
                    numeric,
                )?;
                allowance.consume(local[*argument].first.len(), source)?;
                controls.extend(&local[*argument].first);
                obligations.push((local[*argument].expression.clone(), Condition::Positive));
                allowance.consume(1, source)?;
                facts.set(
                    *token,
                    Arc::new(Fact {
                        expression: Some(Atom::num(0)),
                        source: Some(source),
                        ..Fact::default()
                    }),
                );
            }
            Stage::Block {
                expressions,
                outputs,
                source,
            } => {
                for (expression, &slot) in expressions.iter().zip(outputs) {
                    let inputs = reads(std::slice::from_ref(expression), symbols)?;
                    for &i in &inputs {
                        allowance.consume(facts[i].first.len(), *source)?;
                    }
                    let first = if requested > DerivativeOrder::Value {
                        inputs
                            .iter()
                            .flat_map(|&i| facts[i].first.iter().copied())
                            .collect::<BTreeSet<_>>()
                    } else {
                        BTreeSet::new()
                    };
                    // Flattening is only an optional support/coefficient optimization.
                    // Bound substitution BEFORE allocating the expanded tree. Every symbol
                    // occurrence occupies at least one source byte; the byte product is a
                    // conservative replacement bound. Larger DAGs retain complete support.
                    let expanded = inputs.iter().try_fold(expression.clone(), |e, &i| {
                        let value = facts[i].expression.as_ref()?;
                        let bytes = e
                            .as_view()
                            .get_byte_size()
                            .checked_mul(value.as_view().get_byte_size().checked_add(1)?)?;
                        if bytes > 1024 * 1024 {
                            return None;
                        }
                        let result = e.replace(parameters[i].clone()).with(value.clone());
                        (operation_count(result.count_operations()) <= 16384).then_some(result)
                    });
                    let mut fact = Fact {
                        expression: expanded,
                        second: BTreeSet::new(),
                        first,
                        source: Some(*source),
                    };
                    if requested > DerivativeOrder::Value
                        && let Some(expr) = &fact.expression
                    {
                        if operation_count(expr.count_operations()) > 16384 {
                            return Err(MathError::Limit("local symbolic support expansion"));
                        }
                        let first = reads(std::slice::from_ref(expr), symbols)?;
                        allowance.consume(first.len(), *source)?;
                        fact.first = first
                            .into_iter()
                            .filter(|i| coordinates.is_none_or(|c| c.contains(i)))
                            .collect();
                        fact.second.clear();
                        if requested >= DerivativeOrder::Second
                            && numeric.is_none_or(|n| n.contains(&slot))
                        {
                            for &i in &fact.first {
                                allowance.derivative(*source)?;
                                let d = expr.derivative(
                                    Indeterminate::try_from(parameters[i].clone())
                                        .map_err(|e| MathError::Library(e.clone()))?,
                                );
                                for j in reads(std::slice::from_ref(&d), symbols)?
                                    .into_iter()
                                    .filter(|j| coordinates.is_none_or(|c| c.contains(j)))
                                {
                                    allowance.insert(
                                        &mut fact.second,
                                        (i.min(j), i.max(j)),
                                        *source,
                                    )?;
                                }
                            }
                        }
                    }
                    if requested >= DerivativeOrder::Second
                        && fact.expression.is_none()
                        && numeric.is_none_or(|n| n.contains(&slot))
                    {
                        // Compose support through the shared program. Losing the optional
                        // flattened expression does not make unrelated coordinates nonlinear.
                        // Symbolica owns each local derivative; only its dependency sets
                        // are propagated here (the two terms of the Hessian chain rule).
                        for &i in &inputs {
                            allowance.derivative(*source)?;
                            let derivative = expression.derivative(
                                Indeterminate::try_from(parameters[i].clone())
                                    .map_err(|e| MathError::Library(e.clone()))?,
                            );
                            if derivative == Atom::num(0) {
                                continue;
                            }
                            allowance.extend(&mut fact.second, &facts[i].second, *source)?;
                            for j in reads(std::slice::from_ref(&derivative), symbols)? {
                                for &a in &facts[i].first {
                                    for &b in &facts[j].first {
                                        allowance.insert(
                                            &mut fact.second,
                                            (a.min(b), a.max(b)),
                                            *source,
                                        )?;
                                    }
                                }
                            }
                        }
                    }
                    allowance.consume(1, *source)?;
                    facts.set(slot, Arc::new(fact));
                }
            }
            Stage::Require {
                argument,
                condition,
                order,
                source,
                ..
            } => {
                if requested >= *order {
                    allowance.consume(facts[*argument].first.len(), *source)?;
                    controls.extend(&facts[*argument].first);
                }
                if *order == DerivativeOrder::Value {
                    obligations.push((facts[*argument].expression.clone(), *condition));
                }
            }
            Stage::Provider {
                spec,
                inputs,
                outputs,
                source,
                ..
            } => {
                providers.insert(spec.key(), spec.clone());
                for &i in inputs {
                    allowance.consume(facts[i].first.len(), *source)?;
                }
                let first = inputs
                    .iter()
                    .flat_map(|&i| facts[i].first.iter().copied())
                    .collect::<BTreeSet<_>>();
                for &slot in outputs.iter().filter(|&&s| s != usize::MAX) {
                    allowance.consume(first.len(), *source)?;
                    allowance.consume(1, *source)?;
                    facts.set(
                        slot,
                        Arc::new(Fact {
                            expression: None,
                            first: first.clone(),
                            second: if requested >= DerivativeOrder::Second
                                && numeric.is_none_or(|n| n.contains(&slot))
                            {
                                dense_second(&first, allowance, *source)?
                            } else {
                                BTreeSet::new()
                            },
                            source: Some(*source),
                        }),
                    );
                }
            }
            Stage::Branch {
                continuity,
                left,
                right,
                then,
                otherwise,
                ..
            } => {
                let source = facts[*left]
                    .source
                    .or(facts[*right].source)
                    .unwrap_or(SemanticId::NIL);
                allowance.consume(facts[*left].first.len(), source)?;
                allowance.consume(facts[*right].first.len(), source)?;
                controls.extend(&facts[*left].first);
                controls.extend(&facts[*right].first);
                if *continuity == DerivativeOrder::Value {
                    switches.extend(&facts[*left].first);
                    switches.extend(&facts[*right].first);
                }
                let mut a = allowance.scope_facts(facts, source)?;
                let mut b = allowance.scope_facts(facts, source)?;
                analyze(
                    then,
                    parameters,
                    symbols,
                    &mut a,
                    controls,
                    switches,
                    providers,
                    obligations,
                    allowance,
                    requested,
                    coordinates,
                    cancel,
                    numeric,
                )?;
                analyze(
                    otherwise,
                    parameters,
                    symbols,
                    &mut b,
                    controls,
                    switches,
                    providers,
                    obligations,
                    allowance,
                    requested,
                    coordinates,
                    cancel,
                    numeric,
                )?;
                let mut merged = Vec::new();
                allowance.consume(
                    a.written_slots()
                        .count()
                        .checked_add(b.written_slots().count())
                        .ok_or(MathError::Limit("support branch writes"))?,
                    source,
                )?;
                let slots = a
                    .written_slots()
                    .chain(b.written_slots())
                    .collect::<BTreeSet<_>>();
                for i in slots {
                    if Arc::ptr_eq(&a[i], &b[i]) {
                        if !Arc::ptr_eq(&facts[i], &a[i]) {
                            allowance.consume(1, source)?;
                            merged.push((i, a[i].clone()));
                        }
                        continue;
                    }
                    if !allowance.equal_facts(&a[i], &b[i], source)? {
                        let mut second = BTreeSet::new();
                        allowance.extend(&mut second, &a[i].second, source)?;
                        allowance.extend(&mut second, &b[i].second, source)?;
                        allowance.consume(a[i].first.len(), source)?;
                        allowance.consume(b[i].first.len(), source)?;
                        allowance.consume(1, source)?;
                        merged.push((
                            i,
                            Arc::new(Fact {
                                expression: None,
                                first: a[i].first.union(&b[i].first).copied().collect(),
                                second,
                                source: a[i].source.or(b[i].source),
                            }),
                        ));
                    } else {
                        allowance.consume(1, source)?;
                        merged.push((i, a[i].clone()));
                    }
                }
                drop(a);
                drop(b);
                for (slot, fact) in merged {
                    facts.set(slot, fact);
                }
            }
        }
    }
    Ok(())
}

fn applicability_capture_extent(stages: &[Stage]) -> Option<usize> {
    stages.iter().try_fold(0usize, |sum, stage| {
        sum.checked_add(match stage {
            Stage::Applicability {
                stages,
                predicates,
                inputs,
                ..
            } => (2 * size_of::<ApplicabilityWitness>())
                .checked_add(predicates.len().checked_mul(2 * size_of::<bool>())?)?
                .checked_add(inputs.len().checked_mul(2 * size_of::<f64>())?)?
                .checked_add(applicability_capture_extent(stages)?)?,
            Stage::Domain { stages, .. } => applicability_capture_extent(stages)?,
            Stage::Branch {
                then, otherwise, ..
            } => applicability_capture_extent(then)?
                .checked_add(applicability_capture_extent(otherwise)?)?,
            _ => 0,
        })
    })
}
fn applicability_extent(stages: &[Stage]) -> usize {
    stages.iter().fold(0usize, |sum, stage| {
        sum.saturating_add(match stage {
            Stage::Applicability { stages, plan, .. } => plan
                .observation_bytes()
                .saturating_add(applicability_extent(stages)),
            Stage::Domain { stages, .. } => applicability_extent(stages),
            Stage::Branch {
                then, otherwise, ..
            } => applicability_extent(then).saturating_add(applicability_extent(otherwise)),
            _ => 0,
        })
    })
}

#[cfg(test)]
mod compact_tests {
    use super::*;

    const X: usize = 31;
    const Y: usize = 127;
    const GUARD: usize = 255;
    const RESULT: usize = 2048;

    fn analytic_body(slots: usize) -> PreparedBody {
        crate::initialize().unwrap();
        let x = library::formal(X).unwrap();
        let y = library::formal(Y).unwrap();
        let mut remaining = 100_000;
        PreparedBody::new_with_allowance(
            GUARD + 1,
            slots,
            vec![RESULT],
            vec![
                Stage::Block {
                    expressions: vec![&x * &x + Atom::num(3) * &x * &y],
                    outputs: vec![RESULT],
                    source: SemanticId::NIL,
                },
                Stage::Require {
                    argument: GUARD,
                    condition: Condition::Positive,
                    order: DerivativeOrder::Value,
                    source: SemanticId::NIL,
                    lineage: None,
                },
                // This unrelated output and its input must disappear from every selected extent.
                Stage::Block {
                    expressions: vec![library::formal(0).unwrap()],
                    outputs: vec![slots - 1],
                    source: SemanticId::NIL,
                },
            ],
            &mut remaining,
        )
        .unwrap()
    }
    fn compiled(
        body: &PreparedBody,
        coordinates: &[usize],
        order: DerivativeOrder,
    ) -> CompiledBody {
        body.compile(
            &[0],
            coordinates,
            order,
            Optimization::default(),
            EvaluationLimits::default(),
            &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::new(false), None),
        )
        .unwrap()
    }
    fn eval(
        worker: &mut Worker,
        values: &[f64],
        order: DerivativeOrder,
    ) -> Result<Evaluation, MathError> {
        worker.evaluate(
            values,
            order,
            &mut ProviderBindings::new(),
            &Arc::new(AtomicBool::new(false)),
        )
    }

    #[test]
    fn arithmetic_constructor_demand_uses_source_not_generous_scratch_capacity() {
        crate::initialize().unwrap();
        let x = library::formal(0).unwrap();
        let y = library::formal(1).unwrap();
        let body = PreparedBody::new(
            2,
            3,
            vec![2],
            vec![Stage::Block {
                expressions: vec![&y - &x - Atom::num(1)],
                outputs: vec![2],
                source: SemanticId::NIL,
            }],
            DerivativeOrder::Second,
        )
        .unwrap();
        let reference = EvaluationLimits {
            derivative_components: 1_000_000,
            operations: 100_000_000,
            scratch_bytes: 4usize << 30,
            provider_calls: 1_000_000,
        };
        let bound = body
            .arithmetic_compilation_allocation_bound(DerivativeOrder::Second, reference)
            .unwrap()
            .unwrap();
        assert!(bound < 1 << 20);
        assert_eq!(
            bound,
            body.arithmetic_compilation_allocation_bound(
                DerivativeOrder::Second,
                EvaluationLimits {
                    scratch_bytes: 16usize << 30,
                    ..reference
                }
            )
            .unwrap()
            .unwrap()
        );
        let unguarded_program = body
            .compile(
                &[0],
                &[0, 1],
                DerivativeOrder::Second,
                Optimization::default(),
                reference,
                &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::new(false), None),
            )
            .unwrap();
        assert!(unguarded_program.retained_bytes() + unguarded_program.worker_bytes() < bound);
        // A scalar Require checks an existing slot and adds bounded descriptors;
        // it does not turn an ordinary arithmetic body into an opaque provider.
        let guarded = analytic_body(4096);
        let guarded_bound = guarded
            .arithmetic_compilation_allocation_bound(DerivativeOrder::Second, reference)
            .unwrap()
            .unwrap();
        let guarded_program = compiled(&guarded, &[X, Y], DerivativeOrder::First);
        assert!(guarded_program.retained_bytes() + guarded_program.worker_bytes() < guarded_bound);
    }

    #[test]
    fn arithmetic_constructor_demand_preserves_requested_order_for_wide_rates() {
        crate::initialize().unwrap();
        let parameter = library::formal(256).unwrap();
        let expressions = (0..128)
            .map(|i| library::formal(i).unwrap() - &parameter)
            .collect();
        let body = PreparedBody::new(
            257,
            385,
            (257..385).collect(),
            vec![Stage::Block {
                expressions,
                outputs: (257..385).collect(),
                source: SemanticId::NIL,
            }],
            DerivativeOrder::Second,
        )
        .unwrap();
        let limits = EvaluationLimits {
            scratch_bytes: 4usize << 30,
            derivative_components: 1_000_000,
            operations: 100_000_000,
            provider_calls: 1_000_000,
        };
        let bound = |order| {
            body.arithmetic_compilation_allocation_bound(order, limits)
                .unwrap()
                .unwrap()
        };
        let value = bound(DerivativeOrder::Value);
        let first = bound(DerivativeOrder::First);
        let second = bound(DerivativeOrder::Second);
        let analysis = body
            .arithmetic_analysis_allocation_bound(None)
            .unwrap()
            .unwrap();
        let affine = body
            .arithmetic_analysis_allocation_bound(Some(128))
            .unwrap()
            .unwrap();
        assert!(analysis < 8 << 20, "analysis {analysis}");
        assert!(analysis < first && analysis < affine && affine < second);
        assert!(
            value < first && first < second,
            "value {value}, first {first}, second {second}"
        );
        assert!(first < 8usize << 30, "first {first}");
        assert!(second > 8usize << 30, "second {second}");
        // First-order preparation remains first order and evaluates every residual
        // and requested derivative; the smaller reservation removes no science.
        let program = body
            .compile(
                &(0..128).collect::<Vec<_>>(),
                &(0..257).collect::<Vec<_>>(),
                DerivativeOrder::First,
                Optimization::default(),
                limits,
                &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::new(false), None),
            )
            .unwrap();
        assert_eq!(program.compiled_order(), DerivativeOrder::First);
        assert!(program.retained_bytes() + program.worker_bytes() < first);
        let mut inputs = vec![3.; 257];
        inputs[256] = 2.;
        let result = eval(&mut program.worker(), &inputs, DerivativeOrder::First).unwrap();
        assert_eq!(result.values, vec![1.; 128]);
        let packed = program.worker().evaluate_trial(
            &inputs, DerivativeOrder::First, &mut ProviderBindings::default(),
            &Arc::new(AtomicBool::new(false)),
        ).unwrap();
        assert_eq!(packed.first.len(), 256, "two supported firsts per residual, rather than 128 times 257");
        assert_eq!(program.census().find(|c| c.order == DerivativeOrder::First).unwrap().result_entries, 384);
        for row in 0..128 {
            for coordinate in 0..257 {
                let expected = if coordinate == row {
                    1.
                } else if coordinate == 256 {
                    -1.
                } else {
                    0.
                };
                assert_eq!(result.jacobian[row * 257 + coordinate], expected);
            }
        }
    }

    #[test]
    fn trial_result_keeps_second_partials_packed_and_poisons_only_requested_frame() {
        let body = analytic_body(4096);
        let control = pse_diagnostics::Cancellation::new(false);
        let cancel = control.flag();
        let program = body
            .prepare_support(&[0], &[Y, X], DerivativeOrder::Second, &cancel)
            .unwrap()
            .compile(
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        assert_eq!(program.scalar_builds, 1);
        let mut worker = program.worker();
        assert!(worker.programs.values().all(Option::is_none));
        let result = worker
            .evaluate_trial(
                &[2.0, 3.0, 1.0],
                DerivativeOrder::Second,
                &mut ProviderBindings::new(),
                &cancel,
            )
            .unwrap();
        // x² + 3xy in requested [Y, X] coordinates has no Y² component.
        assert_eq!(result.second_pairs.as_ref(), &[(0, 1), (1, 1)]);
        assert_eq!(result.second, [3., 2.]);
        assert!(worker.programs[DerivativeOrder::Second].is_some());
        assert!(worker.programs[DerivativeOrder::First].is_none());
        assert!(worker.programs[DerivativeOrder::Value].is_none());
        let public = result.clone().into_evaluation();
        assert_eq!(public.hessians, [0., 3., 3., 2.]);
        for i in 0..2 {
            for j in 0..2 {
                assert_eq!(result.second(0, i, j), public.hessians[i * 2 + j]);
            }
        }
        worker.frame.fill(12345.0);
        let extent = worker.body.layouts[DerivativeOrder::Value]
                .as_ref()
                .unwrap()
                .frame_entries(worker.body.slots).unwrap();
        let value = worker
            .evaluate_trial(
                &[2.0, 3.0, 1.0],
                DerivativeOrder::Value,
                &mut ProviderBindings::new(),
                &cancel,
            )
            .unwrap();
        assert_eq!(value.values, public.values);
        assert!(value.second.is_empty());
        assert!(worker.frame[extent..].iter().all(|v| *v == 12345.0));
    }

    #[test]
    fn trial_buffers_reuse_requested_extents_and_clear_refused_results() {
        let control = pse_diagnostics::Cancellation::new(false);
        let cancel = control.flag();
        let program = analytic_body(4096)
            .prepare_support(&[0], &[Y, X], DerivativeOrder::Second, &cancel)
            .unwrap()
            .compile(
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        let mut worker = program.worker();
        let mut result = TrialEvaluation::default();
        let mut providers = ProviderBindings::new();
        worker
            .evaluate_trial_into(
                &[2., 3., 1.],
                DerivativeOrder::Value,
                &mut providers,
                &cancel,
                &mut result,
            )
            .unwrap();
        assert_eq!(
            (result.first.capacity(), result.second.capacity()),
            (0, 0)
        );
        worker
            .evaluate_trial_into(
                &[2., 3., 1.],
                DerivativeOrder::Second,
                &mut providers,
                &cancel,
                &mut result,
            )
            .unwrap();
        let storage = (
            result.values.as_ptr(),
            result.first.as_ptr(),
            result.second.as_ptr(),
        );
        let capacity = (
            result.values.capacity(),
            result.first.capacity(),
            result.second.capacity(),
        );
        let escaped = result.clone().into_evaluation();
        worker
            .evaluate_trial_into(
                &[4., 5., 1.],
                DerivativeOrder::Second,
                &mut providers,
                &cancel,
                &mut result,
            )
            .unwrap();
        assert_eq!(
            storage,
            (
                result.values.as_ptr(),
                result.first.as_ptr(),
                result.second.as_ptr()
            )
        );
        assert_ne!(result.values, escaped.values);
        assert!(
            worker
                .evaluate_trial_into(
                    &[f64::NAN, 5., 1.],
                    DerivativeOrder::Second,
                    &mut providers,
                    &cancel,
                    &mut result
                )
                .is_err()
        );
        assert!(
            result.values.is_empty()
                && result.first.is_empty()
                && result.second.is_empty()
                && result.applicability.is_empty()
        );
        assert_eq!(
            capacity,
            (
                result.values.capacity(),
                result.first.capacity(),
                result.second.capacity()
            )
        );
        worker
            .evaluate_trial_into(
                &[2., 3., 1.],
                DerivativeOrder::Value,
                &mut providers,
                &cancel,
                &mut result,
            )
            .unwrap();
        assert_eq!(result.values, escaped.values);
        assert!(result.first.is_empty() && result.second.is_empty());
        assert_eq!(storage.0, result.values.as_ptr());
    }

    #[test]
    fn compact_first_results_keep_structural_support_at_zero_and_dense_public_boundary() {
        crate::initialize().unwrap();
        let x = library::formal(0).unwrap();
        let body = PreparedBody::new(3, 5, vec![3, 4], vec![
            Stage::Block { expressions: vec![&x * &x, Atom::num(7)], outputs: vec![3, 4], source: SemanticId::NIL },
        ], DerivativeOrder::Second).unwrap();
        let program = body.compile(&[0, 1], &[2, 0, 1], DerivativeOrder::Second,
            Optimization::default(), EvaluationLimits::default(),
            &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
        ).unwrap();
        let mut worker = program.worker();
        let zero = worker.evaluate_trial(&[0., 3., 4.], DerivativeOrder::Second, &mut ProviderBindings::default(),
            &Arc::new(AtomicBool::new(false))).unwrap();
        assert_eq!(zero.first, [0.]);
        assert_eq!(zero.first_entries(0).collect::<Vec<_>>(), [(1, 0.)]);
        assert!(zero.first_entries(1).next().is_none());
        assert_eq!(zero.clone().into_evaluation().jacobian, [0.; 6]);
        let positive = worker.evaluate_trial(&[2., 3., 4.], DerivativeOrder::Second, &mut ProviderBindings::default(),
            &Arc::new(AtomicBool::new(false))).unwrap();
        assert_eq!(positive.first, [4.]);
        assert_eq!(positive.into_evaluation().jacobian, [0., 4., 0., 0., 0., 0.]);
        assert_eq!(program.census().find(|c| c.order == DerivativeOrder::Second).unwrap().result_entries, 5);
    }

    #[test]
    fn compact_frame_and_selected_flat_firsts_compare_actual_region_and_result_work() {
        use pse_diagnostics::observation::{ObservationHub, Policy, Value};
        crate::initialize().unwrap();
        let variables = (0..8).map(|slot| library::formal(slot).unwrap()).collect::<Vec<_>>();
        let expressions = variables.iter().map(|x| x * x).collect::<Vec<_>>();
        let body = PreparedBody::new(16, 24, (16..24).collect(), vec![
            Stage::Block { expressions, outputs: (16..24).collect(), source: SemanticId::NIL },
        ], DerivativeOrder::First).unwrap();
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("selected sparse first comparison");
        let context = operation.context();
        let _entered = context.enter();
        let selected = body.compile(&(0..8).collect::<Vec<_>>(), &(0..16).collect::<Vec<_>>(), DerivativeOrder::First,
            Optimization::default(), EvaluationLimits::default(),
            &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None)).unwrap();
        let counts = context.reader().snapshot(std::time::Duration::from_secs(60));
        assert_eq!(counts.metrics.iter().find(|m| m.name == "math.selected_first_constructions").unwrap().cumulative, Value::Count(8));
        FORCE_TAYLOR_FIRST.set(true);
        let taylor = body.compile(&(0..8).collect::<Vec<_>>(), &(0..16).collect::<Vec<_>>(), DerivativeOrder::First,
            Optimization::default(), EvaluationLimits::default(),
            &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None)).unwrap();
        FORCE_TAYLOR_FIRST.set(false);
        let actual = selected.census().find(|c| c.order == DerivativeOrder::First).unwrap();
        let reference = taylor.census().find(|c| c.order == DerivativeOrder::First).unwrap();
        assert_eq!(actual.frame_entries, 40);
        assert!(actual.frame_entries < selected.slots * actual.derivative_components);
        assert_eq!(actual.result_entries, 16);
        assert_eq!((actual.arithmetic_input_entries, actual.arithmetic_output_entries), (8, 16));
        assert_eq!((reference.arithmetic_input_entries, reference.arithmetic_output_entries), (72, 72));
        assert!(actual.library_operations < reference.library_operations);
        assert_eq!((selected.scalar_builds, taylor.scalar_builds), (2, 1));
        // Selected construction is paid during immutable preparation; callbacks
        // retain original values/checks and never construct another derivative.
        let mut worker = selected.worker();
        let mut input = vec![2.; 16];
        for x in [0., 2., -3., 4.] {
            input[0] = x;
            let first = eval(&mut worker, &input, DerivativeOrder::First).unwrap();
            assert_eq!(first, eval(&mut taylor.worker(), &input, DerivativeOrder::First).unwrap());
        }
        input[0] = 1e200;
        assert!(matches!(eval(&mut worker, &input, DerivativeOrder::First), Err(MathError::Domain { .. })));
        input[0] = 3.;
        assert_eq!(eval(&mut worker, &input, DerivativeOrder::First).unwrap(),
            eval(&mut taylor.worker(), &input, DerivativeOrder::First).unwrap());
        assert_eq!(context.reader().snapshot(std::time::Duration::from_secs(60)).metrics.iter()
            .find(|m| m.name == "math.selected_first_constructions").unwrap().cumulative, Value::Count(8));
        eprintln!("I01 selected: frame={} region-in={} region-out={} returned={} operations={} scalar-builds={}; Taylor: region-in={} region-out={} operations={} scalar-builds={}",
            actual.frame_entries, actual.arithmetic_input_entries, actual.arithmetic_output_entries, actual.result_entries,
            actual.library_operations, selected.scalar_builds, reference.arithmetic_input_entries,
            reference.arithmetic_output_entries, reference.library_operations, taylor.scalar_builds);
    }

    #[test]
    fn compact_branch_frames_write_union_zeros_after_failure_and_order_changes() {
        let program = instantiation_program(Optimization::default());
        let mut worker = program.worker();
        for order in [DerivativeOrder::Second, DerivativeOrder::First, DerivativeOrder::Value, DerivativeOrder::Second] {
            for point in [[2., 3., 1.], [-2., 3., 1.]] {
                assert_eq!(eval(&mut worker, &point, order).unwrap(), eval(&mut program.worker(), &point, order).unwrap());
            }
            assert!(eval(&mut worker, &[2., 3., -1.], order).is_err());
            assert_eq!(eval(&mut worker, &[-2., 4., 1.], order).unwrap(),
                eval(&mut program.worker(), &[-2., 4., 1.], order).unwrap());
        }
        worker.detach_controls();
        worker.reset_scratch().unwrap();
        worker.set_scope(pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None));
        assert_eq!(eval(&mut worker, &[2., 3., 1.], DerivativeOrder::Second).unwrap(),
            eval(&mut program.worker(), &[2., 3., 1.], DerivativeOrder::Second).unwrap());
    }

    #[test]
    fn dense_and_dependent_first_regions_keep_their_qualified_taylor_route() {
        crate::initialize().unwrap();
        let variables = (0..3).map(|slot| library::formal(slot).unwrap()).collect::<Vec<_>>();
        let sum = variables.iter().fold(Atom::num(0), |sum, x| sum + x);
        let dense = PreparedBody::new(3, 5, vec![3, 4], vec![Stage::Block {
            expressions: vec![&sum * &sum, &sum * &sum * &sum], outputs: vec![3, 4], source: SemanticId::NIL,
        }], DerivativeOrder::First).unwrap().compile(&[0, 1], &[0, 1, 2], DerivativeOrder::First,
            Optimization::default(), EvaluationLimits::default(),
            &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None)).unwrap();
        assert_eq!(dense.scalar_builds, 1, "dense firsts do not introduce an alternative scalar construction");
        let counts = dense.census().find(|c| c.order == DerivativeOrder::First).unwrap();
        assert_eq!((counts.arithmetic_input_entries, counts.arithmetic_output_entries), (12, 8));
        assert_eq!(eval(&mut dense.worker(), &[1., 2., 3.], DerivativeOrder::First).unwrap().jacobian,
            [12., 12., 12., 108., 108., 108.]);
        let intermediate = library::formal(2).unwrap();
        let dependent = PreparedBody::new(2, 4, vec![3], vec![
            Stage::Block { expressions: vec![&variables[0] * &variables[0]], outputs: vec![2], source: SemanticId::NIL },
            Stage::Block { expressions: vec![intermediate * &variables[1]], outputs: vec![3], source: SemanticId::NIL },
        ], DerivativeOrder::First).unwrap();
        let dependent = compiled(&dependent, &[0, 1], DerivativeOrder::First);
        let result = eval(&mut dependent.worker(), &[2., 3.], DerivativeOrder::First).unwrap();
        assert_eq!(result.values, [12.]);
        assert_eq!(result.jacobian, [12., 4.]);
    }

    #[cfg(feature = "compiled-evaluator")]
    #[test]
    fn native_compact_selected_firsts_retain_original_value_checks_and_recover() {
        crate::initialize().unwrap();
        let expressions = (0..4).map(|slot| { let x = library::formal(slot).unwrap(); &x * &x }).collect();
        let body = PreparedBody::new(8, 12, (8..12).collect(), vec![Stage::Block {
            expressions, outputs: (8..12).collect(), source: SemanticId::NIL,
        }], DerivativeOrder::First).unwrap();
        let program = body.compile(&(0..4).collect::<Vec<_>>(), &(0..8).collect::<Vec<_>>(), DerivativeOrder::First,
            Optimization { evaluator: library::Evaluator::CompiledScalar { module_bytes: 1 << 20 }, ..Optimization::default() },
            EvaluationLimits::default(), &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None)).unwrap();
        let counts = program.census().find(|c| c.order == DerivativeOrder::First).unwrap();
        assert_eq!((counts.arithmetic_input_entries, counts.arithmetic_output_entries), (4, 8));
        let mut worker = program.worker();
        let mut point = vec![2.; 8];
        let expected = eval(&mut worker, &point, DerivativeOrder::First).unwrap();
        point[0] = 1e200;
        assert!(matches!(eval(&mut worker, &point, DerivativeOrder::First), Err(MathError::Domain { .. })));
        point[0] = 2.;
        assert_eq!(eval(&mut worker, &point, DerivativeOrder::First).unwrap(), expected);
        worker.detach_controls();
        worker.reset_scratch().unwrap();
        worker.set_scope(pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None));
        assert_eq!(eval(&mut worker, &point, DerivativeOrder::First).unwrap(), expected);
    }

    #[test]
    fn selected_compilation_keeps_original_deadline_distinct_from_cancellation() {
        let control = pse_diagnostics::Cancellation::default();
        let body = analytic_body(4096);
        let support = body
            .prepare_support(&[0], &[Y, X], DerivativeOrder::First, &control)
            .unwrap();
        let scope =
            pse_kernels::ExecutionScope::new(control.clone(), Some(std::time::Instant::now()));
        assert!(matches!(
            support.prepare_scalars(Optimization::default(), EvaluationLimits::default(), &scope),
            Err(MathError::Scope(pse_kernels::ProviderError::Deadline))
        ));
        assert!(matches!(
            support.compile(Optimization::default(), EvaluationLimits::default(), &scope),
            Err(MathError::Scope(pse_kernels::ProviderError::Deadline))
        ));
        assert!(matches!(
            support.compile_product(
                ProductKind::SelectedPartials,
                Optimization::default(),
                EvaluationLimits::default(),
                &scope
            ),
            Err(MathError::Scope(pse_kernels::ProviderError::Deadline))
        ));
        assert!(!control.is_cancelled());
    }

    #[test]
    fn retained_scalar_products_survive_order_upgrades_and_keep_their_first_owner() {
        #[derive(Debug)]
        struct Owner;
        let control = pse_diagnostics::Cancellation::new(false);
        let cancel = control.flag();
        let body = analytic_body(4096);
        let value = body
            .prepare_support(&[0], &[Y, X], DerivativeOrder::Value, &cancel)
            .unwrap();
        let owner = Arc::new(Owner);
        let weak = Arc::downgrade(&owner);
        let scalars = value
            .prepare_scalars(
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap()
            .with_owner(owner.clone());
        assert_eq!(scalars.len(), 1);
        drop(owner);
        let mut programs = Vec::new();
        for order in [
            DerivativeOrder::Value,
            DerivativeOrder::First,
            DerivativeOrder::Second,
        ] {
            let support = value.upgrade(order, &cancel).unwrap();
            assert!(
                Arc::ptr_eq(&value.program, &support.program),
                "unchanged compact regions survive a derivative-order upgrade"
            );
            let program = support
                .compile_reusing(
                    Optimization::default(),
                    EvaluationLimits::default(),
                    &pse_kernels::ExecutionScope::new(control.clone(), None),
                    &scalars,
                )
                .unwrap();
            assert_eq!(
                program.scalar_builds, 0,
                "upgrade must not repeat scalar optimization"
            );
            assert_eq!(
                program.layouts.values().flatten().count(),
                order as usize + 1
            );
            let expected = compiled(&body, &[Y, X], order);
            assert_eq!(
                eval(&mut program.worker(), &[2., 3., 1.], order).unwrap(),
                eval(&mut expected.worker(), &[2., 3., 1.], order).unwrap()
            );
            programs.push(program);
        }
        let changed = value
            .compile_reusing(
                Optimization {
                    horner_iterations: 0,
                    ..Optimization::default()
                },
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(control.clone(), None),
                &scalars,
            )
            .unwrap();
        assert_eq!(
            changed.scalar_builds, 1,
            "optimizer settings are exact scalar identity"
        );
        drop(changed);
        let mut worker = programs[2].worker();
        drop(programs);
        drop(scalars);
        assert!(
            weak.upgrade().is_some(),
            "borrowed scalar lease survives cache/artifact removal"
        );
        let mut result = TrialEvaluation::default();
        worker
            .evaluate_trial_into(
                &[2., 3., 1.],
                DerivativeOrder::Second,
                &mut ProviderBindings::new(),
                &cancel,
                &mut result,
            )
            .unwrap();
        assert!(!result.values.is_empty());
        assert!(
            worker
                .evaluate_trial_into(
                    &[2., 3., 0.],
                    DerivativeOrder::Second,
                    &mut ProviderBindings::new(),
                    &cancel,
                    &mut result
                )
                .is_err()
        );
        assert!(result.values.is_empty() && result.first.is_empty() && result.second.is_empty());
        drop(worker);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn retained_scalar_upgrades_detach_the_completed_producers_abort_control() {
        let producer = pse_diagnostics::Cancellation::new(false);
        let body = analytic_body(4096);
        let value = body
            .prepare_support(&[0], &[Y, X], DerivativeOrder::Value, &producer)
            .unwrap();
        let scalars = value
            .prepare_scalars(
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(producer.clone(), None),
            )
            .unwrap();
        assert_eq!(
            Arc::strong_count(&producer),
            1,
            "exact evaluator must not retain the producer token"
        );
        // Successful Flight waiters cancel their token when the last waiter drains.
        producer.cancel();
        let old = Arc::downgrade(&producer);
        drop(producer);
        assert!(old.upgrade().is_none());
        for order in [DerivativeOrder::First, DerivativeOrder::Second] {
            let fresh = pse_diagnostics::Cancellation::new(false);
            let support = value.upgrade(order, &fresh).unwrap();
            let retained = support
                .compile_reusing(
                    Optimization::default(),
                    EvaluationLimits::default(),
                    &pse_kernels::ExecutionScope::new(fresh.clone(), None),
                    &scalars,
                )
                .unwrap();
            let original = support
                .compile(
                    Optimization::default(),
                    EvaluationLimits::default(),
                    &pse_kernels::ExecutionScope::new(fresh.clone(), None),
                )
                .unwrap();
            assert_eq!(retained.scalar_builds, 0);
            assert_eq!(
                Arc::strong_count(&fresh),
                1,
                "derived templates must not retain current constructor controls"
            );
            for requested in [
                DerivativeOrder::Value,
                DerivativeOrder::First,
                DerivativeOrder::Second,
            ]
            .into_iter()
            .filter(|requested| *requested <= order)
            {
                let operations = |program: &CompiledBody| {
                    program.programs[requested]
                        .as_ref()
                        .unwrap()
                        .iter()
                        .filter_map(|stage| match stage {
                            CompiledStage::Block {
                                evaluator: BlockEvaluator::Interpreted(evaluator),
                                ..
                            } => Some(operation_count(evaluator.count_operations())),
                            _ => None,
                        })
                        .sum::<usize>()
                };
                assert_eq!(
                    operations(&retained),
                    operations(&original),
                    "the drained producer must not truncate common-pair optimization"
                );
                assert_eq!(
                    eval(&mut retained.worker(), &[2., 3., 1.], requested).unwrap(),
                    eval(&mut original.worker(), &[2., 3., 1.], requested).unwrap()
                );
            }
            fresh.cancel();
            assert!(matches!(
                support.compile_reusing(
                    Optimization::default(),
                    EvaluationLimits::default(),
                    &pse_kernels::ExecutionScope::new(fresh.clone(), None),
                    &scalars
                ),
                Err(MathError::Cancelled)
            ));
        }
    }

    #[test]
    fn directional_compilation_constructs_only_the_consumed_first_template() {
        let control = pse_diagnostics::Cancellation::new(false);
        let cancel = control.flag();
        let support = analytic_body(4096)
            .prepare_support(&[0], &[Y, X], DerivativeOrder::First, &cancel)
            .unwrap();
        let scalars = support
            .prepare_scalars(
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        let program = support
            .compile_directional_reusing(
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(control.clone(), None),
                &scalars,
            )
            .unwrap();
        assert_eq!(program.scalar_builds, 0);
        assert!(program.layouts[DerivativeOrder::Value].is_none());
        assert!(program.layouts[DerivativeOrder::Second].is_none());
        assert!(program.programs[DerivativeOrder::Value].is_none());
        assert_eq!(program.layouts.values().flatten().count(), 1);
        let mut worker = program.worker();
        let mut result = TrialEvaluation::default();
        worker
            .evaluate_directional_trial_into(
                &[2., 3., 1.],
                &[1., 0.],
                &mut ProviderBindings::new(),
                &cancel,
                &mut result,
            )
            .unwrap();
        assert_eq!(result.first.len(), 1);
        assert!(worker.programs[DerivativeOrder::Value].is_none());
        assert!(worker.programs[DerivativeOrder::First].is_some());
        assert!(worker.clone().programs.values().all(Option::is_none));
    }

    #[test]
    fn lazy_order_workers_refuse_before_growth_and_upgrade_only_requested_products() {
        let control = pse_diagnostics::Cancellation::new(false);
        let cancel = control.flag();
        let program = analytic_body(4096)
            .prepare_support(&[0], &[Y, X], DerivativeOrder::Second, &cancel)
            .unwrap()
            .compile(
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        let mut worker = program.worker();
        assert!(worker.frame.is_empty());
        assert_eq!(worker.used_bytes, 0);
        let first = program.order_worker_bytes[DerivativeOrder::First]
            + program.layouts[DerivativeOrder::First]
                    .as_ref()
                    .unwrap()
                    .frame_entries(program.slots).unwrap()
                * size_of::<f64>();
        worker.prepaid_bytes = first - 1;
        assert!(matches!(
            eval(&mut worker, &[2., 3., 1.], DerivativeOrder::First),
            Err(MathError::Limit("prepaid order worker storage"))
        ));
        assert!(worker.frame.is_empty());
        assert!(worker.programs.values().all(Option::is_none));
        assert_eq!(worker.used_bytes, 0);
        worker.prepaid_bytes = program.worker_bytes();
        let result = eval(&mut worker, &[2., 3., 1.], DerivativeOrder::First).unwrap();
        assert_eq!(worker.used_bytes, first);
        assert_eq!(
            worker.frame.len(),
            program.layouts[DerivativeOrder::First]
                    .as_ref()
                    .unwrap()
                    .frame_entries(program.slots).unwrap()
        );
        assert!(worker.programs[DerivativeOrder::Value].is_none());
        assert!(worker.programs[DerivativeOrder::Second].is_none());
        let first_program = worker.programs[DerivativeOrder::First]
            .as_ref()
            .unwrap()
            .stages
            .as_ptr();
        let used = worker.used_bytes;
        assert_eq!(
            eval(&mut worker, &[2., 3., 1.], DerivativeOrder::First)
                .unwrap()
                .jacobian,
            result.jacobian
        );
        assert_eq!(worker.used_bytes, used);
        assert_eq!(
            worker.programs[DerivativeOrder::First]
                .as_ref()
                .unwrap()
                .stages
                .as_ptr(),
            first_program
        );
        for order in [DerivativeOrder::Value, DerivativeOrder::Second] {
            eval(&mut worker, &[2., 3., 1.], order).unwrap();
        }
        assert!(worker.programs.values().all(Option::is_some));
        assert!(worker.used_bytes <= worker.prepaid_bytes);
        assert_eq!(
            program.scalar_builds, 1,
            "callbacks never run the scalar optimizer"
        );
    }

    #[test]
    fn cloned_warm_worker_is_cold_and_original_verification_has_independent_scope() {
        let control = pse_diagnostics::Cancellation::new(false);
        let cancel = control.flag();
        let program = analytic_body(4096)
            .prepare_support(&[0], &[Y, X], DerivativeOrder::Second, &cancel)
            .unwrap()
            .compile(
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        let first_stop = pse_diagnostics::Cancellation::default();
        let mut original =
            program.worker_scoped(pse_kernels::ExecutionScope::new(first_stop.clone(), None));
        let expected = eval(&mut original, &[2., 3., 1.], DerivativeOrder::Second).unwrap();
        let mut clone = original.clone();
        assert!(clone.frame.is_empty());
        assert!(clone.programs.values().all(Option::is_none));
        assert_eq!(clone.used_bytes, 0);
        let second_stop = pse_diagnostics::Cancellation::default();
        clone.set_scope(pse_kernels::ExecutionScope::new(second_stop.clone(), None));
        first_stop.cancel();
        assert!(eval(&mut original, &[2., 3., 1.], DerivativeOrder::Second).is_err());
        assert!(eval(&mut clone, &[f64::NAN, 3., 1.], DerivativeOrder::Value).is_err());
        assert!(clone.frame.is_empty());
        let actual = eval(&mut clone, &[2., 3., 1.], DerivativeOrder::First).unwrap();
        assert_eq!(actual.values, expected.values);
        assert_eq!(actual.jacobian, expected.jacobian);
        assert!(clone.programs[DerivativeOrder::Second].is_none());
        assert!(!second_stop.is_cancelled());
        let mut cancelled_worker = program.worker();
        control.cancel();
        assert!(matches!(
            cancelled_worker.evaluate(
                &[2., 3., 1.],
                DerivativeOrder::Second,
                &mut ProviderBindings::new(),
                &cancel
            ),
            Err(MathError::Cancelled)
        ));
        assert!(cancelled_worker.frame.is_empty());
        assert!(cancelled_worker.programs.values().all(Option::is_none));
    }

    #[test]
    fn exclusive_scratch_reset_preserves_constants_and_discards_old_scope_values_and_failure() {
        let control = pse_diagnostics::Cancellation::new(false);
        let cancel = control.flag();
        let program = analytic_body(4096)
            .prepare_support(&[0], &[Y, X], DerivativeOrder::Second, &cancel)
            .unwrap()
            .compile(
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        let stop_control = pse_diagnostics::Cancellation::default();
        let stop = stop_control.flag();
        let weak = Arc::downgrade(&stop);
        let mut worker =
            program.worker_scoped(pse_kernels::ExecutionScope::new(stop_control.clone(), None));
        let prior = eval(&mut worker, &[2., 3., 1.], DerivativeOrder::Second).unwrap();
        assert!(eval(&mut worker, &[2., 3., -1.], DerivativeOrder::Value).is_err());
        let frame = worker.frame.as_ptr();
        let used = worker.used_bytes;
        worker.detach_controls();
        stop_control.cancel();
        drop(stop);
        drop(stop_control);
        assert!(
            weak.upgrade().is_none(),
            "detached storage cannot retain cancellation"
        );
        assert!(worker.reset_scratch().is_ok());
        assert_eq!(worker.frame.as_ptr(), frame);
        assert!(worker.frame.iter().all(|v| v.is_nan()));
        assert_eq!(worker.used_bytes, used);
        assert!(
            eval(&mut worker, &[4., 5., 1.], DerivativeOrder::Second).is_err(),
            "detached scratch is not an evaluator permission"
        );
        worker.set_scope(pse_kernels::ExecutionScope::new(
            pse_diagnostics::Cancellation::default(),
            None,
        ));
        let actual = eval(&mut worker, &[4., 5., 1.], DerivativeOrder::Second).unwrap();
        let mut cold = program.worker();
        assert_eq!(
            actual,
            eval(&mut cold, &[4., 5., 1.], DerivativeOrder::Second).unwrap()
        );
        assert_ne!(actual.values, prior.values);
        assert!(
            cold.programs[DerivativeOrder::First].is_none(),
            "independent worker stays lazy and cold"
        );
        // A reset qualifier cannot be inferred for a future external evaluator.
        let stages = worker.programs[DerivativeOrder::Second].as_mut().unwrap();
        fn disqualify(stages: &mut [CompiledStage]) -> bool {
            for stage in stages {
                match stage {
                    CompiledStage::Block { resettable, .. } => {
                        *resettable = false;
                        return true;
                    }
                    CompiledStage::Applicability { stages, .. }
                    | CompiledStage::Domain { stages, .. } => {
                        if disqualify(stages) {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            false
        }
        assert!(disqualify(&mut stages.stages));
        worker.detach_controls();
        assert!(matches!(
            worker.reset_scratch(),
            Err(MathError::Contract(_))
        ));
    }

    #[test]
    fn selected_artifact_demand_uses_compact_source_with_original_four_gib_scratch() {
        let body = analytic_body(4096);
        let control = pse_diagnostics::Cancellation::new(false);
        let cancel = control.flag();
        let support = body
            .prepare_support(&[0], &[X, Y], DerivativeOrder::First, &cancel)
            .unwrap();
        let limits = EvaluationLimits {
            derivative_components: 1_000_000,
            operations: 100_000_000,
            scratch_bytes: 4usize << 30,
            provider_calls: 1_000_000,
        };
        let demand = support
            .compilation_allocation_bound(
                limits,
                ProductKind::SelectedPartials,
                Optimization::default(),
            )
            .unwrap()
            .unwrap();
        assert!(demand < 1 << 20);
        let program = support
            .compile(
                Optimization::default(),
                limits,
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        assert!(program.retained_bytes() + program.worker_bytes() < demand);
        let mut worker = program.worker();
        let result = eval(&mut worker, &[2., 3., 1.], DerivativeOrder::First).unwrap();
        assert_eq!(result.values, [22.]);
        assert_eq!(result.jacobian, [13., 6.]);
    }
    #[test]
    fn selected_artifact_fixed_parameters_do_not_become_taylor_axes() {
        crate::initialize().unwrap();
        let x = library::formal(0).unwrap();
        let sum = (1..=128)
            .map(|slot| library::formal(slot).unwrap())
            .fold(Atom::num(0), |sum, parameter| sum + parameter);
        let body = PreparedBody::new(
            129,
            130,
            vec![129],
            vec![Stage::Block {
                expressions: vec![x - sum],
                outputs: vec![129],
                source: SemanticId::NIL,
            }],
            DerivativeOrder::Second,
        )
        .unwrap();
        let limits = EvaluationLimits {
            scratch_bytes: 1 << 20,
            ..Default::default()
        };
        let control = pse_diagnostics::Cancellation::new(false);
        let cancel = control.flag();
        // All 128 parameters are required value inputs, but x is the only
        // derivative coordinate. The unselected owner must remain distinct.
        assert!(
            body.arithmetic_compilation_allocation_bound(DerivativeOrder::Second, limits)
                .unwrap()
                .unwrap()
                > 8 << 20
        );
        let value = body
            .prepare_support(&[0], &[0], DerivativeOrder::Value, &cancel)
            .unwrap();
        let first = value.upgrade(DerivativeOrder::First, &cancel).unwrap();
        let second = first.upgrade(DerivativeOrder::Second, &cancel).unwrap();
        let value_bound = value
            .compilation_allocation_bound(
                limits,
                ProductKind::SelectedPartials,
                Optimization::default(),
            )
            .unwrap()
            .unwrap();
        let first_bound = first
            .compilation_allocation_bound(
                limits,
                ProductKind::SelectedPartials,
                Optimization::default(),
            )
            .unwrap()
            .unwrap();
        let second_bound = second
            .compilation_allocation_bound(
                limits,
                ProductKind::SelectedPartials,
                Optimization::default(),
            )
            .unwrap()
            .unwrap();
        assert!(value_bound < first_bound);
        assert!(first_bound < second_bound);
        assert!(second_bound < 8 << 20, "selected bound {second_bound}");
        let upgrade_bound = value
            .support_upgrade_allocation_bound(limits, DerivativeOrder::Second)
            .unwrap()
            .unwrap();
        assert!(second.retained_bytes() < upgrade_bound);
        let program = second
            .compile(
                Optimization::default(),
                limits,
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        assert_eq!(program.input_formals().len(), 129);
        assert_eq!(program.coordinates(), &[0]);
        assert!(program.retained_bytes() + program.worker_bytes() < second_bound);
        let mut inputs = vec![1.0; 129];
        inputs[0] = 128.0;
        let result = eval(&mut program.worker(), &inputs, DerivativeOrder::Second).unwrap();
        assert_eq!(result.values, [0.0]);
        assert_eq!(result.jacobian, [1.0]);
        assert_eq!(result.hessians, [0.0]);
        inputs[0] = 129.0;
        assert_eq!(
            eval(&mut program.worker(), &inputs, DerivativeOrder::Second)
                .unwrap()
                .values,
            [1.0]
        );
    }
    #[test]
    fn selected_artifact_sparse_horizon_population_preserves_orders_and_axes() {
        crate::initialize().unwrap();
        // Eight backward-Euler intervals: nine states, nine controls (the first
        // fixed), and initial-state/reference/time-step parameters. Each rate
        // reads only its two states, one control and the time-step parameter.
        let inputs = (0..21)
            .map(|slot| library::formal(slot).unwrap())
            .collect::<Vec<_>>();
        let mut stages = Vec::new();
        for interval in 1..=8 {
            stages.push(Stage::Block {
                expressions: vec![
                    (&inputs[interval] - &inputs[interval - 1]) / &inputs[20]
                        - (Atom::num(2) * &inputs[9 + interval] - &inputs[interval]),
                ],
                outputs: vec![20 + interval],
                source: SemanticId::NIL,
            });
        }
        stages.push(Stage::Block {
            expressions: vec![&inputs[0] - &inputs[18]],
            outputs: vec![29],
            source: SemanticId::NIL,
        });
        let cost = (1..=8)
            .map(|interval| {
                let error = &inputs[interval] - &inputs[19];
                &error * &error * &inputs[20]
            })
            .fold(Atom::num(0), |sum, cost| sum + cost);
        stages.push(Stage::Block {
            expressions: vec![cost],
            outputs: vec![30],
            source: SemanticId::NIL,
        });
        let body =
            PreparedBody::new(21, 31, (21..31).collect(), stages, DerivativeOrder::Second).unwrap();
        let coordinates = (0..9).chain(10..18).collect::<Vec<_>>();
        let outputs = (0..10).collect::<Vec<_>>();
        let control = pse_diagnostics::Cancellation::new(false);
        let cancel = control.flag();
        let limits = EvaluationLimits {
            scratch_bytes: 1 << 20,
            ..Default::default()
        };
        let first = body
            .prepare_support(&outputs, &coordinates, DerivativeOrder::First, &cancel)
            .unwrap();
        let second = first.upgrade(DerivativeOrder::Second, &cancel).unwrap();
        let first_bound = first
            .compilation_allocation_bound(
                limits,
                ProductKind::SelectedPartials,
                Optimization::default(),
            )
            .unwrap()
            .unwrap();
        let second_bound = second
            .compilation_allocation_bound(
                limits,
                ProductKind::SelectedPartials,
                Optimization::default(),
            )
            .unwrap()
            .unwrap();
        let directional_bound = first
            .compilation_allocation_bound(
                limits,
                ProductKind::JacobianAction,
                Optimization::default(),
            )
            .unwrap()
            .unwrap();
        assert!(directional_bound < first_bound);
        assert!(first_bound < second_bound);
        let body_bound = body
            .arithmetic_compilation_allocation_bound(DerivativeOrder::Second, limits)
            .unwrap()
            .unwrap();
        assert!(
            second_bound < body_bound,
            "selected {second_bound}, body-wide {body_bound}"
        );
        let mut reordered = coordinates.clone();
        reordered.reverse();
        let reordered = body
            .prepare_support(&outputs, &reordered, DerivativeOrder::Second, &cancel)
            .unwrap();
        assert_eq!(
            reordered
                .compilation_allocation_bound(
                    limits,
                    ProductKind::SelectedPartials,
                    Optimization::default()
                )
                .unwrap(),
            Some(second_bound)
        );
        let program = second
            .compile(
                Optimization::default(),
                limits,
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        let direction = first
            .compile_directional(
                Optimization::default(),
                limits,
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        assert!(program.retained_bytes() + program.worker_bytes() < second_bound);
        assert!(direction.retained_bytes() + direction.worker_bytes() < directional_bound);
        let values = program
            .input_formals()
            .iter()
            .map(|&formal| match formal {
                0..=8 => formal as f64 / 10.0,
                9..=17 => 0.5,
                18 => 0.0,
                19 => 1.0,
                20 => 0.25,
                _ => panic!("unexpected compact horizon formal {formal}"),
            })
            .collect::<Vec<_>>();
        let result = eval(&mut program.worker(), &values, DerivativeOrder::Second).unwrap();
        let axes = coordinates.len();
        // Independently known rate coefficients and quadratic tracking Hessian.
        assert_eq!(result.jacobian[0], -4.0);
        assert_eq!(result.jacobian[1], 5.0);
        assert_eq!(result.jacobian[9], -2.0);
        for axis in 0..axes {
            for other in 0..axes {
                assert_eq!(
                    result.hessians[9 * axes * axes + axis * axes + other],
                    if axis == other && (1..=8).contains(&axis) {
                        0.5
                    } else {
                        0.0
                    }
                );
            }
        }
        let seeds = (0..axes).map(|axis| (axis + 1) as f64).collect::<Vec<_>>();
        let action = direction
            .worker()
            .evaluate_directional(&values, &seeds, &mut ProviderBindings::new(), &cancel)
            .unwrap();
        for (row, &actual) in action.jacobian.iter().enumerate() {
            let expected = result.jacobian[row * axes..(row + 1) * axes]
                .iter()
                .zip(&seeds)
                .map(|(partial, seed)| partial * seed)
                .sum::<f64>();
            assert!((actual - expected).abs() < 1e-12);
        }
        assert!(matches!(
            second.compilation_allocation_bound(
                limits,
                ProductKind::JacobianAction,
                Optimization::default()
            ),
            Err(MathError::Contract(_))
        ));
    }
    #[test]
    fn compact_sparse_analytic_derivatives_reordering_and_unused_axis() {
        let body = analytic_body(4096);
        let product = compiled(&body, &[X, Y], DerivativeOrder::Second);
        assert_eq!(product.input_formals(), &[X, Y, GUARD]);
        assert_eq!(product.coordinates(), &[X, Y]);
        let mut worker = product.worker();
        let result = eval(&mut worker, &[2., 3., 1.], DerivativeOrder::Second).unwrap();
        assert_eq!(result.values, [22.]);
        assert_eq!(result.jacobian, [13., 6.]);
        assert_eq!(result.hessians, [2., 3., 3., 0.]);
        assert!(matches!(
            eval(&mut worker, &[2., 3., -1.], DerivativeOrder::Second),
            Err(MathError::Domain { .. })
        ));
        assert!(matches!(
            eval(&mut worker, &[2., 3.], DerivativeOrder::Value),
            Err(MathError::Contract(_))
        ));
        assert!(matches!(
            eval(&mut worker, &[2., 3., 1., 4.], DerivativeOrder::Value),
            Err(MathError::Contract(_))
        ));

        let reordered = compiled(&body, &[Y, X, 17], DerivativeOrder::Second);
        assert_eq!(reordered.input_formals(), &[17, X, Y, GUARD]);
        let result = eval(
            &mut reordered.worker(),
            &[99., 2., 3., 1.],
            DerivativeOrder::Second,
        )
        .unwrap();
        assert_eq!(result.jacobian, [6., 13., 0.]);
        assert_eq!(result.hessians, [0., 3., 0., 3., 2., 0., 0., 0., 0.]);
        assert_eq!(reordered.support().first, [BTreeSet::from([X, Y])]);
        assert_eq!(
            reordered.support().second,
            [BTreeSet::from([(X, X), (X, Y)])]
        );
        assert_eq!(reordered.support().controls, BTreeSet::new());
        let guarded = compiled(&body, &[GUARD], DerivativeOrder::Second);
        assert_eq!(guarded.input_formals(), [X, Y, GUARD]);
        assert_eq!(guarded.support().controls, BTreeSet::from([GUARD]));
        let result = eval(
            &mut guarded.worker(),
            &[2., 3., 1.],
            DerivativeOrder::Second,
        )
        .unwrap();
        assert_eq!(result.jacobian, [0.]);
        assert_eq!(result.hessians, [0.]);
        assert!(matches!(
            eval(
                &mut guarded.worker(),
                &[2., 3., -1.],
                DerivativeOrder::Second
            ),
            Err(MathError::Domain { .. })
        ));
    }

    #[test]
    fn compact_unrelated_growth_changes_no_selected_extent_or_result() {
        let small = compiled(&analytic_body(4096), &[Y, X], DerivativeOrder::Second);
        let large = compiled(&analytic_body(16384), &[Y, X], DerivativeOrder::Second);
        assert_eq!(small.input_formals(), [X, Y, GUARD]);
        assert_eq!(large.input_formals(), [X, Y, GUARD]);
        assert_eq!(small.scratch_bytes(), large.scratch_bytes());
        assert_eq!(small.worker_bytes(), large.worker_bytes());
        assert_eq!(small.retained_bytes(), large.retained_bytes());
        assert_eq!(
            small.prepared_support().retained_bytes(),
            large.prepared_support().retained_bytes()
        );
        assert_eq!(
            eval(&mut small.worker(), &[2., 3., 1.], DerivativeOrder::Second).unwrap(),
            eval(&mut large.worker(), &[2., 3., 1.], DerivativeOrder::Second).unwrap()
        );
    }

    #[test]
    fn compact_directional_and_support_upgrade_keep_authored_coordinates() {
        let body = analytic_body(4096);
        let control = pse_diagnostics::Cancellation::new(false);
        let cancel = control.flag();
        let value = body
            .prepare_support(&[0], &[Y, X], DerivativeOrder::Value, &cancel)
            .unwrap();
        let first = value.upgrade(DerivativeOrder::First, &cancel).unwrap();
        let directional = first
            .compile_directional(
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        assert_eq!(directional.input_formals(), [X, Y, GUARD]);
        let result = directional
            .worker()
            .evaluate_directional(&[2., 3., 1.], &[5., 4.], &mut ProviderBindings::new(), &cancel)
            .unwrap();
        assert_eq!(result.values, [22.]);
        assert_eq!(result.jacobian, [82.]);
        let second = first.upgrade(DerivativeOrder::Second, &cancel).unwrap();
        assert_eq!(second.coordinates(), [Y, X]);
        assert_eq!(second.support().first, [BTreeSet::from([X, Y])]);
        assert_eq!(second.support().second, [BTreeSet::from([(X, X), (X, Y)])]);
        let widened = first.incidence(&[17, X], &cancel).unwrap();
        assert_eq!(widened.input_formals(), [17, X, Y, GUARD]);
        assert_eq!(widened.support().first, [BTreeSet::from([X])]);
    }

    #[test]
    fn compact_source_identity_has_no_strong_body_or_allocation_lease() {
        let owner = Arc::new(AtomicBool::new(false));
        let body = analytic_body(4096).with_owner(owner.clone());
        let source = Arc::downgrade(&body.data);
        let separately_equal = analytic_body(4096);
        let support = body
            .prepare_support(
                &[0],
                &[X],
                DerivativeOrder::First,
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        assert!(support.matches_body(&body));
        assert!(support.matches_body(&separately_equal));
        assert_eq!(Arc::strong_count(&owner), 2);
        drop(body);
        assert!(source.upgrade().is_none());
        assert_eq!(Arc::strong_count(&owner), 1);
        assert!(!support.matches_body(&separately_equal));
        let product = support
            .compile(
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::new(false), None),
            )
            .unwrap();
        assert_eq!(
            eval(&mut product.worker(), &[2., 3., 1.], DerivativeOrder::First)
                .unwrap()
                .jacobian,
            [13.]
        );
    }

    #[test]
    fn compact_selection_diagnostics_cancellation_and_budget_remain_enforced() {
        let body = analytic_body(4096);
        let cancel = Arc::new(AtomicBool::new(false));
        for coordinates in [vec![X, X], vec![256]] {
            assert!(matches!(
                body.prepare_support(&[0], &coordinates, DerivativeOrder::First, &cancel),
                Err(MathError::Contract(_))
            ));
        }
        for outputs in [vec![], vec![0, 0], vec![1]] {
            assert!(matches!(
                body.prepare_support(&outputs, &[X], DerivativeOrder::First, &cancel),
                Err(MathError::Contract(_))
            ));
        }
        cancel.store(true, Ordering::Relaxed);
        assert!(matches!(
            body.prepare_support(&[0], &[X], DerivativeOrder::First, &cancel),
            Err(MathError::Cancelled)
        ));
        cancel.store(false, Ordering::Relaxed);
        let support = body
            .prepare_support(&[0], &[X], DerivativeOrder::First, &cancel)
            .unwrap();
        let mut exhausted = support.clone();
        Arc::make_mut(&mut exhausted.data).remaining_occurrences = 0;
        assert!(exhausted.upgrade(DerivativeOrder::Second, &cancel).is_err());
        assert_eq!(support.order(), DerivativeOrder::First);
        assert!(support.remaining_occurrences() > 0);
    }

    fn nested_branch_body() -> PreparedBody {
        use pse_model::{
            applicability::{Claim, Node, Region},
            generated::enums::ModelingValidityLayer,
        };
        crate::initialize().unwrap();
        let lineage = Arc::new(pse_model::diagnostic::ValidityLineage {
            layer: ModelingValidityLayer::Form,
            source: SemanticId::NIL,
            form: Some(SemanticId::NIL),
            sets: vec![],
            variables: vec![0],
            members: vec![],
        });
        let plan = Arc::new(Node {
            claim: Claim {
                id: Some(SemanticId::NIL),
                coverage: None,
                owner: SemanticId::NIL,
                owner_lineage: vec![SemanticId::NIL],
                evidence: Some(SemanticId::NIL),
                form: SemanticId::NIL,
                call: SemanticId::NIL,
                records: vec![],
                dependencies: vec![],
                layer: ModelingValidityLayer::Form,
                basis: None,
                reason: None,
            },
            region: Region::Predicate(0),
            dependencies: vec![],
            inputs: vec![("guard".into(), 0, SemanticId::NIL)],
            permissions: vec![],
        });
        let block = |expression, output| Stage::Block {
            expressions: vec![expression],
            outputs: vec![output],
            source: SemanticId::NIL,
        };
        let x = library::formal(X).unwrap();
        let y = library::formal(Y).unwrap();
        let g = library::formal(GUARD).unwrap();
        PreparedBody::new(
            GUARD + 1,
            4096,
            vec![RESULT],
            vec![
                Stage::Domain {
                    stages: vec![block(g.clone(), 3000)],
                    argument: 3000,
                    token: 3001,
                    lineage,
                },
                Stage::Applicability {
                    stages: vec![block(g, 3002)],
                    predicates: vec![3002],
                    inputs: vec![GUARD],
                    token: 3003,
                    plan,
                },
                block(Atom::num(0), 3004),
                Stage::Branch {
                    continuity: DerivativeOrder::Value,
                    comparison: Comparison::Lt,
                    left: 3004,
                    right: X,
                    then: vec![block(&x * &x, RESULT)],
                    otherwise: vec![block(&y * &y, RESULT)],
                },
            ],
            DerivativeOrder::Value,
        )
        .unwrap()
    }

    #[test]
    fn compact_nested_domain_applicability_and_both_branch_arms_capture_sparse_inputs() {
        let body = nested_branch_body();
        for order in [
            DerivativeOrder::Value,
            DerivativeOrder::First,
            DerivativeOrder::Second,
        ] {
            assert!(
                body.arithmetic_compilation_allocation_bound(order, EvaluationLimits::default())
                    .unwrap()
                    .is_none()
            );
        }
        assert!(
            body.arithmetic_analysis_allocation_bound(None)
                .unwrap()
                .is_none()
        );
        let product = body
            .prepare_support(
                &[0],
                &[X, Y],
                DerivativeOrder::First,
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        assert_eq!(product.input_formals(), [X, Y, GUARD]);
        assert_eq!(product.support().first, [BTreeSet::from([X, Y])]);
        let value = body
            .compile(
                &[0],
                &[],
                DerivativeOrder::Value,
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::new(false), None),
            )
            .unwrap();
        assert_eq!(value.input_formals(), [X, Y, GUARD]);
        // Domain and applicability perform the same scalar g read once, despite
        // separate typed barriers; branch products remain independent.
        assert_eq!(value.scalar_builds, 4);
        let mut worker = value.worker();
        assert_eq!(
            eval(&mut worker, &[2., 3., 1.], DerivativeOrder::Value)
                .unwrap()
                .values,
            [4.]
        );
        assert_eq!(
            eval(&mut worker, &[-2., 3., 1.], DerivativeOrder::Value)
                .unwrap()
                .values,
            [9.]
        );
        let mut trial = worker
            .evaluate_trial(
                &[2., 3., 1.],
                DerivativeOrder::Value,
                &mut ProviderBindings::new(),
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        assert_eq!(trial.applicability.len(), 1);
        let hydrated = trial.applicability[0].hydrate();
        assert_eq!(hydrated.len(), 1);
        assert_eq!(hydrated[0].inputs[0].value, 1.0);
        let captured = (
            trial.applicability[0].predicates.as_ptr(),
            trial.applicability[0].inputs.as_ptr(),
        );
        worker
            .evaluate_trial_into(
                &[-2., 3., 2.],
                DerivativeOrder::Value,
                &mut ProviderBindings::new(),
                &Arc::new(AtomicBool::new(false)),
                &mut trial,
            )
            .unwrap();
        assert_eq!(
            captured,
            (
                trial.applicability[0].predicates.as_ptr(),
                trial.applicability[0].inputs.as_ptr()
            )
        );
        assert_eq!(trial.applicability[0].hydrate()[0].inputs[0].value, 2.0);
        assert_eq!(hydrated[0].inputs[0].value, 1.0);
        assert!(
            worker
                .evaluate_trial_into(
                    &[2., 3., -1.],
                    DerivativeOrder::Value,
                    &mut ProviderBindings::new(),
                    &Arc::new(AtomicBool::new(false)),
                    &mut trial
                )
                .is_err()
        );
        assert!(trial.applicability.is_empty());
        assert!(
            trial
                .spare_witnesses
                .iter()
                .all(|w| w.inputs.is_empty() && w.predicates.is_empty())
        );
        worker
            .evaluate_trial_into(
                &[2., 3., 1.],
                DerivativeOrder::Value,
                &mut ProviderBindings::new(),
                &Arc::new(AtomicBool::new(false)),
                &mut trial,
            )
            .unwrap();
        assert_eq!(
            captured,
            (
                trial.applicability[0].predicates.as_ptr(),
                trial.applicability[0].inputs.as_ptr()
            )
        );
        let mut projected = TrialEvaluation::default();
        let plans = worker.body.projection_witnesses(DerivativeOrder::Value);
        let pairs = worker.body.projection_second_pairs();
        trial
            .project_into(
                &[0],
                &[],
                DerivativeOrder::Value,
                &plans,
                &pairs,
                &Arc::default(),
                &[],
                &mut projected,
            )
            .unwrap();
        let projected_capture = (
            projected.applicability[0].predicates.as_ptr(),
            projected.applicability[0].inputs.as_ptr(),
        );
        for _ in 0..8 {
            trial
                .project_into(
                    &[0],
                    &[],
                    DerivativeOrder::Value,
                    &plans,
                    &pairs,
                    &Arc::default(),
                    &[],
                    &mut projected,
                )
                .unwrap();
            assert_eq!(projected.applicability, trial.applicability);
            assert!(
                projected.spare_witnesses.is_empty(),
                "projection reuses witnesses instead of accumulating spare copies"
            );
            assert_eq!(
                projected_capture,
                (
                    projected.applicability[0].predicates.as_ptr(),
                    projected.applicability[0].inputs.as_ptr()
                )
            );
        }
        let slots = worker.body.slots;
        for stage in &worker.programs[DerivativeOrder::Value].as_ref().unwrap().stages {
            if let CompiledStage::Domain { frame, .. }
            | CompiledStage::Applicability { frame, .. } = stage
            {
                assert!(frame.len() < slots);
            }
        }
        assert!(eval(&mut worker, &[2., 3., -1.], DerivativeOrder::Value).is_err());
        worker.detach_controls();
        worker.reset_scratch().unwrap();
        fn assert_poisoned(stages: &[CompiledStage]) {
            for stage in stages {
                match stage {
                    CompiledStage::Domain { stages, frame, .. }
                    | CompiledStage::Applicability { stages, frame, .. } => {
                        assert!(frame.iter().all(|v| v.is_nan()));
                        assert_poisoned(stages);
                    }
                    CompiledStage::Block {
                        arguments, values, gather, ..
                    } => {
                        for (argument, value) in arguments.iter().enumerate() {
                            if gather.iter().any(|&(written, _)| written == argument) { assert!(value.is_nan()); }
                            else { assert_eq!(*value, 0.0); }
                        }
                        assert!(values.iter().all(|v| v.is_nan()));
                    }
                    CompiledStage::Branch {
                        then, otherwise, ..
                    } => {
                        assert_poisoned(then);
                        assert_poisoned(otherwise);
                    }
                    _ => {}
                }
            }
        }
        assert_poisoned(&worker.programs[DerivativeOrder::Value].as_ref().unwrap().stages);
        worker.set_scope(pse_kernels::ExecutionScope::new(
            pse_diagnostics::Cancellation::default(),
            None,
        ));
        assert_eq!(
            eval(&mut worker, &[4., 5., 1.], DerivativeOrder::Value)
                .unwrap()
                .values,
            [16.]
        );
    }

    fn instantiation_counts() -> (usize, usize, usize) {
        INSTANTIATION_PROBE.with_borrow(|probe| {
            (probe.visits, probe.evaluator_calls, probe.template_clones)
        })
    }

    fn instantiation_program(options: Optimization) -> CompiledBody {
        nested_branch_body()
            .compile_branch_local(
                &[0],
                &[X, Y],
                DerivativeOrder::Second,
                options,
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
            )
            .unwrap()
    }

    fn check_successful_order_instantiation(options: Optimization) {
        let program = instantiation_program(options);
        INSTANTIATION_PROBE.set(InstantiationProbe::default());
        let mut worker = program.worker();
        for (index, order) in [
            DerivativeOrder::Value,
            DerivativeOrder::First,
            DerivativeOrder::Second,
        ]
        .into_iter()
        .enumerate()
        {
            worker.prepare_order(order).unwrap();
            let expected = (8 * (index + 1), 5 * (index + 1), index + 1);
            assert_eq!(instantiation_counts(), expected);
            worker.prepare_order(order).unwrap();
            assert_eq!(instantiation_counts(), expected, "warm preparation never walks stages");
            let positive = eval(&mut worker, &[2., 3., 1.], order).unwrap();
            let negative = eval(&mut worker, &[-2., 3., 1.], order).unwrap();
            assert_eq!(positive.values, [4.]);
            assert_eq!(negative.values, [9.]);
            if order >= DerivativeOrder::First {
                assert_eq!(positive.jacobian, [4., 0.]);
                assert_eq!(negative.jacobian, [0., 6.]);
            }
            if order >= DerivativeOrder::Second {
                assert_eq!(positive.hessians, [2., 0., 0., 0.]);
                assert_eq!(negative.hessians, [0., 0., 0., 2.]);
            }
            assert_eq!(instantiation_counts(), expected, "both branch arms were already instantiated");
        }
        let ready = instantiation_counts();
        let cancelled = pse_diagnostics::Cancellation::new(true);
        worker.set_scope(pse_kernels::ExecutionScope::new(cancelled, None));
        assert!(matches!(worker.prepare_order(DerivativeOrder::Value), Err(MathError::Cancelled)));
        worker.set_scope(pse_kernels::ExecutionScope::new(
            pse_diagnostics::Cancellation::default(),
            Some(std::time::Instant::now()),
        ));
        assert!(matches!(
            worker.prepare_order(DerivativeOrder::Value),
            Err(MathError::Scope(pse_kernels::ProviderError::Deadline))
        ));
        assert_eq!(instantiation_counts(), ready, "scope rejection still precedes a warm hit");
        worker.detach_controls();
        worker.reset_scratch().unwrap();
        assert!(worker.programs.values().flatten().all(|program| !program.instantiated));
        worker.set_scope(pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None));
        worker.prepare_order(DerivativeOrder::Value).unwrap();
        assert_eq!(instantiation_counts(), (32, 20, 3), "reset retries without cloning templates");
        worker.prepare_order(DerivativeOrder::Value).unwrap();
        assert_eq!(instantiation_counts(), (32, 20, 3));
        assert_eq!(eval(&mut worker, &[4., 5., 1.], DerivativeOrder::Value).unwrap().values, [16.]);
        let mut replacement = worker.clone();
        replacement.prepare_order(DerivativeOrder::Value).unwrap();
        assert_eq!(instantiation_counts(), (40, 25, 4), "replacement worker inherits no successful state");
    }

    #[test]
    fn successful_order_instantiation_walks_nested_branches_once_and_resets() {
        check_successful_order_instantiation(Optimization::default());
    }

    #[cfg(feature = "compiled-evaluator")]
    #[test]
    fn native_successful_order_instantiation_walks_nested_branches_once_and_resets() {
        check_successful_order_instantiation(Optimization {
            evaluator: library::Evaluator::CompiledScalar { module_bytes: 1 << 20 },
            ..Optimization::default()
        });
    }

    #[test]
    fn incomplete_order_instantiation_retries_and_accounts_retained_frame() {
        let program = instantiation_program(Optimization::default());
        INSTANTIATION_PROBE.set(InstantiationProbe { fail_at: Some(4), ..InstantiationProbe::default() });
        let mut worker = program.worker();
        assert!(matches!(worker.prepare_order(DerivativeOrder::First), Err(MathError::Library(_))));
        assert_eq!(instantiation_counts(), (4, 1, 1));
        assert!(worker.programs[DerivativeOrder::First].is_none());
        assert_eq!(worker.used_bytes, worker.frame.len() * size_of::<f64>());
        INSTANTIATION_PROBE.with_borrow_mut(|probe| probe.fail_at = None);
        worker.prepare_order(DerivativeOrder::First).unwrap();
        assert_eq!(instantiation_counts(), (12, 6, 2));
        assert_eq!(worker.used_bytes, worker.frame.len() * size_of::<f64>() + program.order_worker_bytes[DerivativeOrder::First]);
        assert!(worker.programs[DerivativeOrder::First].as_ref().unwrap().instantiated);
        worker.prepare_order(DerivativeOrder::First).unwrap();
        assert_eq!(instantiation_counts(), (12, 6, 2));
    }

    #[test]
    fn cancelled_order_instantiation_publishes_only_after_final_scope_check() {
        let program = instantiation_program(Optimization::default());
        INSTANTIATION_PROBE.set(InstantiationProbe { cancel_at: Some(8), ..InstantiationProbe::default() });
        let mut worker = program.worker();
        // Cancellation in the last leaf is detected before publishing even though
        // every evaluator has been instantiated and there is no next stage check.
        assert!(matches!(worker.prepare_order(DerivativeOrder::Value), Err(MathError::Cancelled)));
        assert_eq!(instantiation_counts(), (8, 5, 1));
        assert!(worker.programs[DerivativeOrder::Value].is_none());
        assert_eq!(worker.used_bytes, worker.frame.len() * size_of::<f64>());
        INSTANTIATION_PROBE.with_borrow_mut(|probe| probe.cancel_at = None);
        worker.set_scope(pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None));
        worker.prepare_order(DerivativeOrder::Value).unwrap();
        assert_eq!(instantiation_counts(), (16, 10, 2));
        worker.detach_controls();
        worker.reset_scratch().unwrap();
        worker.set_scope(pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None));
        INSTANTIATION_PROBE.with_borrow_mut(|probe| probe.cancel_at = Some(18));
        assert!(matches!(worker.prepare_order(DerivativeOrder::Value), Err(MathError::Cancelled)));
        assert_eq!(instantiation_counts(), (18, 11, 2));
        assert!(!worker.programs[DerivativeOrder::Value].as_ref().unwrap().instantiated);
        INSTANTIATION_PROBE.with_borrow_mut(|probe| probe.cancel_at = None);
        worker.set_scope(pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None));
        worker.prepare_order(DerivativeOrder::Value).unwrap();
        assert_eq!(instantiation_counts(), (26, 16, 2));
        worker.prepare_order(DerivativeOrder::Value).unwrap();
        assert_eq!(instantiation_counts(), (26, 16, 2));
        assert_eq!(eval(&mut worker, &[2., 3., 1.], DerivativeOrder::Value).unwrap().values, [4.]);
    }

    #[test]
    fn compact_provider_retains_required_inputs_and_original_output_ordinal() {
        use pse_ids::ContentHash;
        use pse_kernels::{DerivativeSource, Port, ProviderError, ProviderShapes, ProviderValues};
        #[derive(Debug)]
        struct ProductProvider(ProviderSpec);
        impl Provider for ProductProvider {
                    fn fill(
                &mut self,
                inputs: &[f64],
                request: &PreparedProviderRequest,
                _runtime: ProviderRuntime<'_>,
                mut output: pse_kernels::ProviderOutput<'_>,
                context: &EvaluationContext<'_>,
            ) -> Result<(), ProviderError> {
                request.check_context(context)?;
                assert_eq!(request.outputs, [1]);
                assert_eq!(inputs.len(), 3);
                if inputs[2] <= 0. {
                    return Err(ProviderError::Trial("guard-only provider argument".into()));
                }
                output.copy_from(&ProviderValues {
                    values: vec![inputs[0] * inputs[1]],
                    jacobian: if request.order >= DerivativeOrder::First {
                        vec![inputs[1], inputs[0], 0.]
                    } else {
                        vec![]
                    },
                    hessians: if request.order >= DerivativeOrder::Second {
                        vec![0., 1., 0., 1., 0., 0., 0., 0., 0.]
                    } else {
                        vec![]
                    },
                })
            }
        }
        crate::initialize().unwrap();
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let quantity = pse_quantity::standard::ids::quantity("neutral");
        let unit = registry.quantity_type(quantity).unwrap().canonical_unit;
        let port = |n| Port {
            id: SemanticId::from_bytes([n; 16]),
            quantity,
            unit,
        };
        let spec = ProviderSpec {
            id: SemanticId::from_bytes([1; 16]),
            revision: ContentHash::from_bytes([2; 32]),
            data: ContentHash::from_bytes([3; 32]),
            shapes: ProviderShapes::default(),
            derivative_source: DerivativeSource::Analytic,
            inputs: vec![port(4), port(5), port(6)],
            outputs: vec![port(7), port(8)],
            derivatives: DerivativeOrder::Second,
            smoothness: DerivativeOrder::Second,
        };
        let body = PreparedBody::new(
            GUARD + 1,
            4096,
            vec![3000, RESULT],
            vec![Stage::Provider {
                spec: spec.clone(),
                partial: vec![],
                inputs: vec![X, Y, GUARD],
                outputs: vec![3000, RESULT],
                source: SemanticId::NIL,
            }],
            DerivativeOrder::Second,
        )
        .unwrap();
        let control = pse_diagnostics::Cancellation::new(false);
        let cancel = control.flag();
        let product = body
            .compile(
                &[1],
                &[Y, X],
                DerivativeOrder::Second,
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        assert_eq!(product.input_formals(), [X, Y, GUARD]);
        assert_eq!(product.prepared_support().outputs(), [1]);
        let key = spec.key();
        #[derive(Debug)]
        struct LatchedProvider {
            inner: ProductProvider,
            failed: bool,
        }
        impl Provider for LatchedProvider {
                    fn fill(
                &mut self,
                inputs: &[f64],
                request: &PreparedProviderRequest,
                _runtime: ProviderRuntime<'_>,
                output: pse_kernels::ProviderOutput<'_>,
                context: &EvaluationContext<'_>,
            ) -> Result<(), ProviderError> {
                if self.failed {
                    return Err(ProviderError::Trial("old provider failure latch".into()));
                }
                let result = self.inner.fill(inputs, request, _runtime, output, context);
                if result.is_err() {
                    self.failed = true;
                }
                result
            }
        }
        let fresh_provider = || {
            pse_kernels::ProviderBinding::new(spec.clone(), Box::new(LatchedProvider {
                inner: ProductProvider(spec.clone()),
                failed: false,
            }))
        };
        let mut scalar = product.worker();
        let mut prior_providers = ProviderBindings::from([(key, fresh_provider())]);
        assert_eq!(
            scalar
                .evaluate(
                    &[2., 3., 1.],
                    DerivativeOrder::Value,
                    &mut prior_providers,
                    &cancel
                )
                .unwrap()
                .values,
            [6.]
        );
        assert!(
            scalar
                .evaluate(
                    &[2., 3., -1.],
                    DerivativeOrder::Value,
                    &mut prior_providers,
                    &cancel
                )
                .is_err()
        );
        assert!(
            scalar
                .evaluate(
                    &[4., 5., 1.],
                    DerivativeOrder::Value,
                    &mut prior_providers,
                    &cancel
                )
                .is_err()
        );
        scalar.detach_controls();
        drop(prior_providers);
        scalar.reset_scratch().unwrap();
        scalar.set_scope(pse_kernels::ExecutionScope::new(
            pse_diagnostics::Cancellation::default(),
            None,
        ));
        let mut fresh_providers = ProviderBindings::from([(key, fresh_provider())]);
        assert_eq!(
            scalar
                .evaluate(
                    &[4., 5., 1.],
                    DerivativeOrder::Value,
                    &mut fresh_providers,
                    &cancel
                )
                .unwrap()
                .values,
            [20.]
        );
        let provider = pse_kernels::ProviderBinding::new(spec.clone(), Box::new(ProductProvider(spec.clone())));
        let mut providers = ProviderBindings::from([(key, provider)]);
        let mut worker = product.worker();
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let operation = hub.start("provider.prepared_dispatch");
        let _entered = operation.context().enter();
        let count = |name: &str| {
            operation.reader().snapshot(std::time::Duration::from_secs(60)).metrics.iter()
                .find_map(|metric| match metric.cumulative {
                    pse_diagnostics::observation::Value::Count(value) if metric.name == name => Some(value),
                    _ => None,
                }).unwrap_or(0)
        };
        let result = worker
            .evaluate(
                &[2., 3., 1.],
                DerivativeOrder::Second,
                &mut providers,
                &cancel,
            )
            .unwrap();
        assert_eq!(result.values, [6.]);
        assert_eq!(result.jacobian, [2., 3.]);
        assert_eq!(result.hessians, [0., 1., 1., 0.]);
        let immutable = ["kernel.provider.descriptor_hashes", "kernel.provider.request_preparations",
            "kernel.provider.slot_resolutions", "kernel.provider.request_bindings"];
        let before = immutable.map(count);
        assert_eq!(count("kernel.provider.slot_resolutions"), 1);
        for _ in 0..4 {
            assert_eq!(worker.evaluate(&[2., 3., 1.], DerivativeOrder::Second, &mut providers, &cancel).unwrap().values, [6.]);
        }
        assert_eq!(immutable.map(count), before, "warm dispatch borrows the prepared descriptor/request/slot");
        let mut replacement = ProviderBindings::from([(key, fresh_provider())]);
        assert_eq!(worker.evaluate(&[2., 3., 1.], DerivativeOrder::Second, &mut replacement, &cancel).unwrap().values, [6.]);
        assert_eq!(count("kernel.provider.slot_resolutions"), 2, "fresh bank rebinds once");
        assert!(
            worker
                .evaluate(
                    &[2., 3., -1.],
                    DerivativeOrder::Value,
                    &mut providers,
                    &cancel
                )
                .is_err()
        );
    }

    #[test]
    fn compact_output_projection_and_conditional_support_preserve_original_ordinals() {
        crate::initialize().unwrap();
        let x = library::formal(X).unwrap();
        let y = library::formal(Y).unwrap();
        let body = PreparedBody::new(
            GUARD + 1,
            4096,
            vec![RESULT, 3000],
            vec![Stage::Block {
                expressions: vec![&x * &x, &y * &y * &y],
                outputs: vec![RESULT, 3000],
                source: SemanticId::NIL,
            }],
            DerivativeOrder::Second,
        )
        .unwrap();
        let control = pse_diagnostics::Cancellation::new(false);
        let cancel = control.flag();
        let support = body
            .prepare_support(&[0, 1], &[Y, X], DerivativeOrder::Second, &cancel)
            .unwrap();
        let selected = support.select_outputs(&[1], &cancel).unwrap();
        let direct = body
            .prepare_support(&[1], &[Y, X], DerivativeOrder::Second, &cancel)
            .unwrap();
        assert_eq!(selected, direct);
        assert_eq!(selected.retained_bytes(), direct.retained_bytes());
        let product = selected
            .compile(
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        let result = eval(&mut product.worker(), &[2., 3.], DerivativeOrder::Second).unwrap();
        assert_eq!(result.values, [27.]);
        assert_eq!(result.jacobian, [27., 0.]);
        assert_eq!(result.hessians, [18., 0., 0., 0.]);
        let conditional = support
            .conditional_support(&[1], &[Y], DerivativeOrder::Second, &cancel)
            .unwrap();
        assert_eq!(conditional.input_formals(), [Y]);
        assert_eq!(conditional.support().first, [BTreeSet::from([Y])]);
        let product = conditional
            .compile(
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        assert_eq!(
            eval(&mut product.worker(), &[3.], DerivativeOrder::Second)
                .unwrap()
                .hessians,
            [18.]
        );
    }

    #[test]
    fn sparse_library_shape_omits_unconsumed_convolutions_and_retains_lower_orders() {
        crate::initialize().unwrap();
        let x = library::formal(0).unwrap();
        let y = library::formal(1).unwrap();
        let z = library::formal(2).unwrap();
        let body = PreparedBody::new(
            3,
            4,
            vec![3],
            vec![Stage::Block {
                expressions: vec![&x * &y + z.pow(2)],
                outputs: vec![3],
                source: SemanticId::NIL,
            }],
            DerivativeOrder::Second,
        )
        .unwrap();
        let cancel = pse_diagnostics::Cancellation::default();
        let limits = EvaluationLimits {
            derivative_components: 6,
            ..Default::default()
        };
        let compiled = body
            .compile(
                &[0],
                &[0, 1, 2],
                DerivativeOrder::Second,
                Optimization::default(),
                limits,
                &pse_kernels::ExecutionScope::new(cancel.clone(), None),
            )
            .unwrap();
        let second = compiled.layouts[DerivativeOrder::Second].as_ref().unwrap();
        assert_eq!(second.pairs.as_slice(), &[(0, 1), (2, 2)]);
        assert_eq!(second.width(), 6, "dense shape would need ten components");
        assert_eq!(
            compiled.census[DerivativeOrder::Second]
                .unwrap()
                .second_pairs,
            2
        );
        let mut worker = compiled.worker();
        let first = eval(&mut worker, &[2., 3., 4.], DerivativeOrder::First).unwrap();
        assert_eq!(first.values, [22.]);
        assert_eq!(first.jacobian, [3., 2., 8.]);
        assert!(!worker.census().instantiated_orders[2]);
        let result = eval(&mut worker, &[2., 3., 4.], DerivativeOrder::Second).unwrap();
        assert_eq!(result.hessians, [0., 1., 0., 1., 0., 0., 0., 0., 2.]);
        let linear = PreparedBody::new(
            1,
            2,
            vec![1],
            vec![Stage::Block {
                expressions: vec![x],
                outputs: vec![1],
                source: SemanticId::NIL,
            }],
            DerivativeOrder::Second,
        )
        .unwrap();
        let compiled = linear
            .compile(
                &[0],
                &[0],
                DerivativeOrder::Second,
                Optimization::default(),
                limits,
                &pse_kernels::ExecutionScope::new(cancel.clone(), None),
            )
            .unwrap();
        assert_eq!(
            eval(&mut compiled.worker(), &[2.], DerivativeOrder::Second)
                .unwrap()
                .hessians,
            [0.]
        );
    }

    #[test]
    fn joint_sibling_regions_preserve_sources_and_dependent_materialization() {
        crate::initialize().unwrap();
        let x = library::formal(0).unwrap();
        let y = library::formal(1).unwrap();
        let a = library::formal(2).unwrap();
        let first_source = pse_ids::named_id(SemanticId::NIL, "first sibling");
        let second_source = pse_ids::named_id(SemanticId::NIL, "second sibling");
        let body = PreparedBody::new(
            2,
            5,
            vec![3, 4],
            vec![
                Stage::Block {
                    expressions: vec![(&x + &y).pow(2)],
                    outputs: vec![2],
                    source: first_source,
                },
                Stage::Block {
                    expressions: vec![(&x + &y).pow(3)],
                    outputs: vec![3],
                    source: second_source,
                },
                Stage::Block {
                    expressions: vec![a.pow(2)],
                    outputs: vec![4],
                    source: SemanticId::NIL,
                },
            ],
            DerivativeOrder::Second,
        )
        .unwrap();
        use pse_diagnostics::observation::{ObservationHub, Policy, Value};
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("prepared sibling and dependent regions");
        let context = operation.context();
        let _entered = context.enter();
        let cancel = pse_diagnostics::Cancellation::default();
        let scope = pse_kernels::ExecutionScope::new(cancel.clone(), None);
        #[derive(Debug)]
        struct Owner;
        let owner = Arc::new(Owner);
        let weak_owner = Arc::downgrade(&owner);
        let support = body
            .prepare_support(&[0, 1], &[0, 1], DerivativeOrder::Second, &cancel.flag())
            .unwrap()
            .with_owner(owner.clone());
        drop(owner);
        let scalars = support
            .prepare_scalars(Optimization::default(), EvaluationLimits::default(), &scope)
            .unwrap();
        let compiled = support
            .compile_reusing(
                Optimization::default(),
                EvaluationLimits::default(),
                &scope,
                &scalars,
            )
            .unwrap();
        let count = |name: &str| {
            context
                .reader()
                .snapshot(std::time::Duration::from_secs(60))
                .metrics
                .iter()
                .find(|metric| metric.name == name)
                .map(|metric| metric.cumulative.clone())
        };
        assert_eq!(
            count("math.arithmetic_region_preparations"),
            Some(Value::Count(2))
        );
        assert_eq!(
            count("math.rational_representation_inspections"),
            Some(Value::Count(2))
        );
        assert_eq!(
            count("math.taylor_representation_inspections"),
            Some(Value::Count(4))
        );
        assert_eq!(
            count("math.floating_representation_inspections"),
            Some(Value::Count(6))
        );
        assert_eq!(
            compiled.census[DerivativeOrder::Second]
                .unwrap()
                .arithmetic_blocks,
            2
        );
        let mut origins = Vec::new();
        compiled.visit_arithmetic_origins(DerivativeOrder::Second, |origin| origins.push(origin));
        assert_eq!(
            origins.len(),
            compiled.arithmetic_origin_count(DerivativeOrder::Second)
        );
        assert_eq!(
            origins
                .iter()
                .map(|origin| (
                    origin.region,
                    origin.source,
                    origin.start_output,
                    origin.end_output
                ))
                .collect::<Vec<_>>(),
            [
                (0, first_source, 0, 1),
                (0, second_source, 1, 2),
                (1, SemanticId::NIL, 0, 1)
            ]
        );
        let mut first_worker = compiled.worker();
        let mut second_worker = compiled.worker();
        let result = eval(&mut first_worker, &[1., 2.], DerivativeOrder::Second).unwrap();
        assert_eq!(
            eval(&mut second_worker, &[2., 3.], DerivativeOrder::Second)
                .unwrap()
                .values,
            [125., 625.]
        );
        assert_eq!(
            eval(&mut first_worker, &[1., 2.], DerivativeOrder::Second).unwrap(),
            result
        );
        assert_eq!(result.values, [27., 81.]);
        assert_eq!(result.jacobian, [27., 27., 108., 108.]);
        assert_eq!(
            result.hessians,
            [18., 18., 18., 18., 108., 108., 108., 108.]
        );
        assert_eq!(
            eval(&mut compiled.worker(), &[1.5, 1.5], DerivativeOrder::Second).unwrap(),
            result,
            "equal-valued formals remain independent derivative coordinates"
        );
        assert!(
            matches!(eval(&mut compiled.worker(), &[1e200,1e200],DerivativeOrder::Value),
            Err(MathError::Domain { source_id, .. }) if source_id == first_source)
        );
        assert!(
            matches!(eval(&mut compiled.worker(), &[1e110, 0.], DerivativeOrder::Value),
            Err(MathError::Domain { source_id, .. }) if source_id == second_source),
            "the first finite sibling must not mask the next original failure"
        );
        assert!(matches!(
            support.compile_reusing(
                Optimization::default(),
                EvaluationLimits {
                    operations: 1,
                    ..Default::default()
                },
                &scope,
                &scalars
            ),
            Err(MathError::Limit(_))
        ));
        assert_eq!(
            count("math.arithmetic_region_preparations"),
            Some(Value::Count(2))
        );
        assert_eq!(
            count("math.rational_representation_inspections"),
            Some(Value::Count(2)),
            "current policy checks retained facts without a new rational export"
        );
        assert_eq!(
            count("math.floating_representation_inspections"),
            Some(Value::Count(6)),
            "evaluation and refusal do not inspect new representations"
        );
        drop(support);
        drop(scalars);
        drop(compiled);
        drop(first_worker);
        assert!(
            weak_owner.upgrade().is_some(),
            "a live worker retains prepared-region ownership after optional retention is cleared"
        );
        drop(second_worker);
        assert!(
            weak_owner.upgrade().is_none(),
            "the last worker releases the prepared description owner"
        );
    }

    #[cfg(feature = "compiled-evaluator")]
    #[test]
    fn native_scalar_production_blocks_preserve_coordinates_fp_and_worker_ownership() {
        use pse_diagnostics::observation::{ObservationHub, Policy, Value};
        crate::initialize().unwrap();
        let x = library::formal(0).unwrap();
        let y = library::formal(1).unwrap();
        let z = library::formal(2).unwrap();
        let body = PreparedBody::new(
            3,
            5,
            vec![3, 4],
            vec![Stage::Block {
                expressions: vec![&x * &y + &z, x.pow(2) + &y * Atom::num(10) + &z],
                outputs: vec![3, 4],
                source: SemanticId::NIL,
            }],
            DerivativeOrder::Second,
        )
        .unwrap();
        let control = pse_diagnostics::Cancellation::new(false);
        let cancel = control.flag();
        let options = Optimization {
            evaluator: library::Evaluator::CompiledScalar {
                module_bytes: 1 << 20,
            },
            ..Default::default()
        };
        let limits = EvaluationLimits::default();
        let support = body
            .prepare_support(&[0, 1], &[2, 0], DerivativeOrder::Second, &cancel)
            .unwrap();
        let bound = support
            .compilation_allocation_bound(limits, ProductKind::SelectedPartials, options)
            .unwrap()
            .unwrap();
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("native scalar production consumer");
        let context = operation.context();
        let _entered = context.enter();
        let scalars = support
            .prepare_scalars(
                Optimization::default(),
                limits,
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        let program = support
            .compile_reusing(
                options,
                limits,
                &pse_kernels::ExecutionScope::new(control.clone(), None),
                &scalars,
            )
            .unwrap();
        assert_eq!(
            program.scalar_builds, 0,
            "native execution must reuse exact scalar preparation"
        );
        assert!(program.retained_bytes() + program.worker_bytes() <= bound);
        assert!(matches!(
            &program.programs[DerivativeOrder::Value].as_ref().unwrap()[0],
            CompiledStage::Block {
                evaluator: BlockEvaluator::Native(_),
                ..
            }
        ));
        assert!(matches!(
            &program.programs[DerivativeOrder::First].as_ref().unwrap()[0],
            CompiledStage::Block {
                evaluator: BlockEvaluator::Native(_),
                ..
            }
        ));
        let reference = support
            .compile_reusing(
                Optimization::default(),
                limits,
                &pse_kernels::ExecutionScope::new(control.clone(), None),
                &scalars,
            )
            .unwrap();
        let mut expected = reference.worker();
        let mut first = program.worker();
        let mut second = program.worker();
        let witness = [1. + 2f64.powi(-27), 1. - 2f64.powi(-27), -1.];
        for input in [[2., 3., 5.], witness, [4., 2., 1.], [2., 3., 5.]] {
            let actual = eval(&mut first, &input, DerivativeOrder::Value).unwrap();
            let original = eval(&mut expected, &input, DerivativeOrder::Value).unwrap();
            assert_eq!(actual, original);
        }
        assert_eq!(
            eval(&mut first, &witness, DerivativeOrder::Value)
                .unwrap()
                .values[0]
                .to_bits(),
            0f64.to_bits()
        );
        assert_ne!(
            witness[0].mul_add(witness[1], witness[2]).to_bits(),
            0f64.to_bits()
        );
        assert_eq!(
            eval(&mut second, &[4., 2., 1.], DerivativeOrder::Value)
                .unwrap()
                .values,
            [9., 37.]
        );
        assert_eq!(
            eval(&mut first, &[2., 3., 5.], DerivativeOrder::First).unwrap(),
            eval(&mut expected, &[2., 3., 5.], DerivativeOrder::First).unwrap()
        );
        assert!(!second.census().instantiated_orders[2]);
        assert_eq!(
            eval(&mut first, &[2., 3., 5.], DerivativeOrder::Second).unwrap(),
            eval(&mut expected, &[2., 3., 5.], DerivativeOrder::Second).unwrap()
        );
        assert!(matches!(
            &program.programs[DerivativeOrder::Second].as_ref().unwrap()[0],
            CompiledStage::Block {
                evaluator: BlockEvaluator::Native(_),
                ..
            }
        ));
        // Both live worker and a late worker retain the executable after the
        // original immutable artifact and another worker have been released.
        let mut late = second.clone();
        drop(first);
        drop(program);
        assert_eq!(
            eval(&mut late, &[2., 3., 5.], DerivativeOrder::Value)
                .unwrap()
                .values,
            [11., 39.]
        );
        assert_eq!(
            eval(&mut second, &[4., 2., 1.], DerivativeOrder::Value)
                .unwrap()
                .values,
            [9., 37.]
        );
        let snapshot = context
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        let count = |name: &str| {
            snapshot
                .metrics
                .iter()
                .find(|metric| metric.name == name)
                .map(|metric| metric.cumulative.clone())
        };
        assert_eq!(
            count("math.native_scalar_compilations"),
            Some(Value::Count(3))
        );
        assert_eq!(
            count("math.native_scalar_evaluations"),
            Some(Value::Count(10))
        );
    }

    #[cfg(feature = "compiled-evaluator")]
    #[test]
    fn native_scalar_production_guards_and_nonfinite_results_keep_typed_errors() {
        crate::initialize().unwrap();
        let x = library::formal(0).unwrap();
        let body = PreparedBody::new(
            1,
            2,
            vec![1],
            vec![
                Stage::Require {
                    argument: 0,
                    condition: Condition::Positive,
                    order: DerivativeOrder::Value,
                    source: SemanticId::NIL,
                    lineage: None,
                },
                Stage::Block {
                    expressions: vec![x.log()],
                    outputs: vec![1],
                    source: SemanticId::NIL,
                },
            ],
            DerivativeOrder::Value,
        )
        .unwrap();
        let control = pse_diagnostics::Cancellation::new(false);
        let cancel = control.flag();
        let program = body
            .compile(
                &[0],
                &[0],
                DerivativeOrder::Value,
                Optimization {
                    evaluator: library::Evaluator::CompiledScalar {
                        module_bytes: 1 << 20,
                    },
                    ..Default::default()
                },
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        let mut worker = program.worker();
        assert_eq!(
            eval(&mut worker, &[1.], DerivativeOrder::Value)
                .unwrap()
                .values,
            [0.]
        );
        assert!(matches!(
            eval(&mut worker, &[-1.], DerivativeOrder::Value),
            Err(MathError::Domain { .. })
        ));
        assert!(eval(&mut worker, &[f64::INFINITY], DerivativeOrder::Value).is_err());
        control.cancel();
        assert!(matches!(
            worker.evaluate(&[1.], DerivativeOrder::Value, &mut ProviderBindings::new(), &cancel),
            Err(MathError::Cancelled)
        ));
    }

    #[test]
    fn observation_counts_actual_preparation_and_lazy_evaluator_instantiation() {
        use pse_diagnostics::observation::{ObservationHub, Policy, Value};
        let body = analytic_body(4096);
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("math preparation");
        let context = operation.context();
        let _entered = context.enter();
        let control = pse_diagnostics::Cancellation::new(false);
        let cancel = control.flag();
        let support = body
            .prepare_support(&[0], &[Y, X], DerivativeOrder::First, &cancel)
            .unwrap();
        let scalars = support
            .prepare_scalars(
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        let program = support
            .compile_reusing(
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(control.clone(), None),
                &scalars,
            )
            .unwrap();
        let snapshot = || {
            context
                .reader()
                .snapshot(std::time::Duration::from_secs(60))
        };
        let count = |name: &str| {
            snapshot()
                .metrics
                .iter()
                .find(|metric| metric.name == name)
                .map(|metric| metric.cumulative.clone())
        };
        assert_eq!(count("math.support_constructions"), Some(Value::Count(1)));
        assert_eq!(
            count("math.scalar_preparation_calls"),
            Some(Value::Count(1))
        );
        assert_eq!(
            count("math.compiled_body_constructions_completed"),
            Some(Value::Count(1))
        );
        assert_eq!(
            count("math.compiled_order_constructions"),
            Some(Value::Count(2))
        );
        assert_eq!(
            count("math.scalar_constructions_completed"),
            Some(Value::Count(1))
        );
        assert!(matches!(count("math.scalar_cache_reuses"), Some(Value::Count(n)) if n > 0));
        assert_eq!(
            count("math.arithmetic_region_preparations"),
            Some(Value::Count(1))
        );
        assert_eq!(
            count("math.rational_representation_inspections"),
            Some(Value::Count(1))
        );
        assert_eq!(
            count("math.taylor_representation_inspections"),
            Some(Value::Count(1))
        );
        assert_eq!(
            count("math.floating_representation_inspections"),
            Some(Value::Count(2))
        );
        assert_eq!(count("math.evaluator_template_instantiations"), None);
        let mut worker = program.worker();
        worker.prepare_order(DerivativeOrder::Value).unwrap();
        worker.prepare_order(DerivativeOrder::Value).unwrap();
        assert_eq!(
            count("math.evaluator_template_instantiations"),
            Some(Value::Count(1))
        );
        worker.prepare_order(DerivativeOrder::First).unwrap();
        worker.prepare_order(DerivativeOrder::First).unwrap();
        assert_eq!(
            count("math.evaluator_template_instantiations_completed"),
            Some(Value::Count(2))
        );
        // Rejection is a preparation attempt, and never starts support construction.
        assert!(
            body.prepare_support(&[usize::MAX], &[Y, X], DerivativeOrder::First, &cancel)
                .is_err()
        );
        assert_eq!(
            count("math.support_preparation_calls"),
            Some(Value::Count(2))
        );
        assert_eq!(count("math.support_constructions"), Some(Value::Count(1)));
        assert_eq!(
            count("math.support_constructions_completed"),
            Some(Value::Count(1))
        );
    }
}
