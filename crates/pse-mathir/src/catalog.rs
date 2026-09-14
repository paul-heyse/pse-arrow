// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Projection of the operator authority into registry rows (blueprint §7.3).
use crate::opspec::{OPERATOR_TABLE, OperatorSpec};
use pse_quantity::{OperationId, registry::QuantityRegistry};

/// A registry adapter consumes the table's declaration rather than reconstructing it.
pub trait OperatorSpecSink {
    /// Emit one operator with the applicable registered physical operation identities.
    fn operator_spec(&mut self, spec: &OperatorSpec, quantity_operation_ids: &[OperationId]);
}
/// Emit every operator in authoritative order, including those with no registered rules.
pub fn emit_operator_specs(registry: &QuantityRegistry, sink: &mut dyn OperatorSpecSink) {
    for spec in &OPERATOR_TABLE {
        let operations: Vec<_> = registry
            .operations_for(spec.opcode)
            .map(|rule| rule.id)
            .collect();
        sink.operator_spec(spec, &operations);
    }
}
