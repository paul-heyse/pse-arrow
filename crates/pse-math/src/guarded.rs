// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Control and domain barriers around library-owned arithmetic blocks.
//!
//! This layer never interprets arithmetic operations. A block is a Symbolica evaluator;
//! only branch selection, obligations and fallible provider calls are project-owned.
use crate::MathError;
pub use crate::execution::{CompiledBody, Evaluation, PreparedBody, Support, Worker};
use pse_ids::SemanticId;
use pse_kernels::{DerivativeOrder, ProviderSpec};
use symbolica::atom::{Atom, AtomCore, Symbol};

/// Index into evaluation-local scalar storage.
pub type Slot = usize;

/// Runtime condition retained from syntax before normalization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    /// Real logarithm or differentiable square root.
    Positive,
    /// Value-only real square root.
    Nonnegative,
    /// Denominator or negative integral power.
    Nonzero,
}
impl Condition {
    pub(crate) fn permits(self, value: f64) -> bool {
        value.is_finite()
            && match self {
                Self::Positive => value > 0.0,
                Self::Nonnegative => value >= 0.0,
                Self::Nonzero => value != 0.0,
            }
    }
    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Nonnegative => "nonnegative",
            Self::Nonzero => "nonzero",
        }
    }
}

/// Comparison of already evaluated finite scalars.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Comparison {
    /// Exact equality.
    Eq,
    /// Exact inequality.
    Ne,
    /// Strict ordering.
    Lt,
    /// Nonstrict ordering.
    Le,
}
impl Comparison {
    pub(crate) fn select(self, left: f64, right: f64) -> bool {
        match self {
            Self::Eq => left == right,
            Self::Ne => left != right,
            Self::Lt => left < right,
            Self::Le => left <= right,
        }
    }
}

/// One region in dependency order. Constructed only after physical admission.
#[derive(Clone, Debug, PartialEq)]
#[allow(
    clippy::large_enum_variant,
    reason = "Stages retain owned expressions inline for compilation without per-stage indirection"
)]
pub(crate) enum Stage {
    /// Safe multi-output arithmetic owned by Symbolica.
    Block {
        /// Expressions in output order.
        expressions: Vec<Atom>,
        /// Destination slots.
        outputs: Vec<Slot>,
        /// Authored occurrence for failures.
        source: SemanticId,
    },
    /// A pre-normalization obligation.
    Require {
        /// Previously computed argument.
        argument: Slot,
        /// Required real domain.
        condition: Condition,
        /// Minimum admission order at which this condition is necessary.
        order: DerivativeOrder,
        /// Authored occurrence.
        source: SemanticId,
    },
    /// Lazy regions. Both branches write the same declared result slots.
    Branch {
        /// Comparison.
        comparison: Comparison,
        /// Left comparison scalar.
        left: Slot,
        /// Right comparison scalar.
        right: Slot,
        /// True region.
        then: Vec<Stage>,
        /// False region.
        otherwise: Vec<Stage>,
    },
    /// Multi-output fallible call, outside Symbolica's infallible scalar callbacks.
    Provider {
        /// Exact compiled registration, including phase and data revision.
        spec: ProviderSpec,
        /// Ordered input slots.
        inputs: Vec<Slot>,
        /// Ordered value output slots.
        outputs: Vec<Slot>,
        /// Authored occurrence.
        source: SemanticId,
    },
}

pub(crate) fn validate_dependencies(
    stages: &[Stage],
    symbols: &std::collections::HashMap<Symbol, usize>,
    assigned: &mut std::collections::BTreeSet<usize>,
    depth: usize,
    remaining: &mut usize,
) -> Result<(), MathError> {
    if depth > 128 || stages.len() > *remaining {
        return Err(MathError::Limit("obligation schedule"));
    }
    *remaining -= stages.len();
    for stage in stages {
        let read = |slots: &[usize]| {
            if slots.iter().all(|v| assigned.contains(v)) {
                Ok(())
            } else {
                Err(MathError::Contract(
                    "obligation or block reads a value before its domain-safe producer".into(),
                ))
            }
        };
        let outputs = match stage {
            Stage::Block {
                expressions,
                outputs,
                ..
            } => {
                if expressions.is_empty() || expressions.len() != outputs.len() {
                    return Err(MathError::Contract("block output arity".into()));
                }
                for expr in expressions {
                    let inputs = expr
                        .get_all_symbols(false)
                        .into_iter()
                        .map(|symbol| {
                            symbols.get(&symbol).copied().ok_or_else(|| {
                                MathError::Contract("unregistered library parameter".into())
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    read(&inputs)?;
                }
                Some(outputs)
            }
            Stage::Require { argument, .. } => {
                read(&[*argument])?;
                None
            }
            Stage::Provider {
                spec,
                inputs,
                outputs,
                ..
            } => {
                if inputs.len() != spec.inputs.len() || outputs.len() != spec.outputs.len() {
                    return Err(MathError::Contract("provider slot arity".into()));
                }
                read(inputs)?;
                Some(outputs)
            }
            Stage::Branch {
                left,
                right,
                then,
                otherwise,
                ..
            } => {
                read(&[*left, *right])?;
                let mut a = assigned.clone();
                let mut b = assigned.clone();
                validate_dependencies(then, symbols, &mut a, depth + 1, remaining)?;
                validate_dependencies(otherwise, symbols, &mut b, depth + 1, remaining)?;
                *assigned = a.intersection(&b).copied().collect();
                None
            }
        };
        if let Some(outputs) = outputs {
            for slot in outputs {
                if *slot >= symbols.len() {
                    return Err(MathError::Contract("stage destination out of range".into()));
                }
                if !assigned.insert(*slot) {
                    return Err(MathError::Contract(
                        "stage overwrites an already assigned scalar".into(),
                    ));
                }
            }
        }
    }
    Ok(())
}
