// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Kernel descriptors add actual typed symbols and calls without claiming execution.
use super::{Realizer, invalid};
use crate::CompilerError;
use pse_ids::SemanticId;
use pse_mathir::{
    Opcode, Payload,
    relations::{ParameterBinding, vec_sink::KernelBinding},
};
use pse_quantity::{QuantityTypeId, UnitId};
use pse_relations::generated::{
    compiled,
    enums::{
        BoundKind, ExpressionRootRole, SolverVariableType, SymbolRole, VariableLifecycle,
        VariableSemanticRole,
    },
    extension_values::Bound,
    reference,
};

#[derive(Clone, Debug)]
pub(crate) struct KernelMethod {
    pub descriptor: reference::kernel_specs::Row,
    pub method: SemanticId,
    pub requirement: SemanticId,
    pub state_scope: SemanticId,
    pub state: SemanticId,
    pub index: Vec<SemanticId>,
    pub output_ordinal: i64,
    pub inputs: Vec<(String, SemanticId)>,
    pub parameters: Vec<ParameterBinding>,
    pub support: std::collections::BTreeSet<crate::passes::native_outputs::SourceKey>,
}
impl Realizer<'_> {
    #[expect(
        clippy::too_many_lines,
        reason = "kernel_methods keeps the native relation inputs and dependency ordered assembly visible in one place"
    )]
    pub(super) fn kernel_methods(&mut self, methods: &[KernelMethod]) -> Result<(), CompilerError> {
        for method in methods {
            self.cancel.checkpoint()?;
            self.context(method.state, None)?;
            for source in &method.support {
                let (role, _) = self
                    .sources
                    .get(&source.relation)
                    .ok_or_else(|| invalid("kernel source has no actual bound role"))?;
                if role != &source.port {
                    return Err(invalid("kernel source role differs from realization input"));
                }
                let position = self.source_position(source.relation, &source.key)?;
                self.active_support.insert(position);
            }
            let output = method
                .descriptor
                .outputs
                .get(
                    usize::try_from(method.output_ordinal)
                        .map_err(|_| invalid("negative kernel output ordinal"))?,
                )
                .ok_or_else(|| invalid("kernel output ordinal absent"))?;
            let quantity = self
                .physical
                .quantity_type(QuantityTypeId::from_id(output.quantity_type_id))?
                .clone();
            if !quantity.key.shape.is_empty() || !method.index.is_empty() {
                return Err(invalid(
                    "kernel output requires an indexed expression carrier, not a scalar symbol",
                ));
            }
            let mut inputs = Vec::new();
            if method.inputs.len() != method.descriptor.inputs.len() {
                return Err(invalid(
                    "kernel input inventory differs from exact descriptor",
                ));
            }
            for ((name, symbol), port) in method.inputs.iter().zip(&method.descriptor.inputs) {
                if name != &port.name {
                    return Err(invalid(
                        "kernel input order/name differs from exact descriptor",
                    ));
                }
                let actual = self
                    .symbols
                    .values()
                    .find(|row| row.symbol_id == *symbol)
                    .ok_or_else(|| invalid("kernel input provider did not realize"))?;
                pse_quantity::admission::require_same_contract(
                    QuantityTypeId::from_id(port.quantity_type_id),
                    QuantityTypeId::from_id(actual.quantity_type_id),
                    self.physical,
                )?;
                let conversion = pse_quantity::convert_spec_for_type(
                    self.physical.unit(UnitId::from_id(actual.unit_id))?,
                    self.physical.unit(UnitId::from_id(port.natural_unit_id))?,
                    &self
                        .physical
                        .quantity_type(QuantityTypeId::from_id(actual.quantity_type_id))?
                        .key,
                )?;
                let mut node = self.graph.symbol(*symbol)?;
                self.node_support
                    .entry(node)
                    .or_default()
                    .extend(self.active_support.iter().copied());
                if actual.unit_id != port.natural_unit_id {
                    node = self.graph.insert(
                        Opcode::UnitConvert,
                        Payload::UnitConvert(conversion),
                        &[node],
                        Some(method.state),
                    )?;
                }
                self.node_support
                    .entry(node)
                    .or_default()
                    .extend(self.active_support.iter().copied());
                inputs.push((name.clone(), node));
            }
            let binding = crate::passes::p9::kernel_binding_id(
                method.state_scope,
                method.method,
                &method.index,
            );
            let actual = KernelBinding {
                binding,
                kernel: method.descriptor.kernel_id,
                scope: method.state,
                parameters: method.parameters.clone(),
                inputs,
            };
            if self
                .kernels
                .get(&binding)
                .is_some_and(|previous| previous != &actual)
            {
                return Err(invalid(
                    "kernel binding identity has conflicting actual ordered values",
                ));
            }
            self.kernels.insert(binding, actual);
            self.kernel_support
                .entry(binding)
                .or_default()
                .extend(self.active_support.iter().copied());
            let mut root = self.graph.insert(
                Opcode::KernelCall,
                Payload::KernelCall {
                    kernel_binding: binding,
                    output_ordinal: u16::try_from(method.output_ordinal)
                        .map_err(|_| invalid("kernel output exceeds declared width"))?,
                },
                &[],
                Some(method.state),
            )?;
            self.node_support
                .entry(root)
                .or_default()
                .extend(self.active_support.iter().copied());
            if output.natural_unit_id != quantity.canonical_unit.as_id() {
                let conversion = pse_quantity::convert_spec_for_type(
                    self.physical
                        .unit(UnitId::from_id(output.natural_unit_id))?,
                    self.physical.unit(quantity.canonical_unit)?,
                    &quantity.key,
                )?;
                root = self.graph.insert(
                    Opcode::UnitConvert,
                    Payload::UnitConvert(conversion),
                    &[root],
                    Some(method.state),
                )?;
            }
            self.node_support
                .entry(root)
                .or_default()
                .extend(self.active_support.iter().copied());
            let symbol = crate::passes::p9::kernel_output_id(
                method.state_scope,
                method.method,
                method.output_ordinal,
                &method.index,
            );
            let row = compiled::symbols::Row {
                symbol_id: symbol,
                ordinal: 0,
                owner_instance_id: method.state,
                symbol_decl_id: symbol,
                qualified_name: format!(
                    "kernel.{}.{}.{}",
                    method.state_scope.to_hex(),
                    method.method.to_hex(),
                    output.name
                ),
                index: method.index.clone(),
                quantity_type_id: quantity.id.as_id(),
                unit_id: quantity.canonical_unit.as_id(),
                role: SymbolRole::Expression,
                solver_type: SolverVariableType::Continuous,
                semantic_role: VariableSemanticRole::ReportingOnly,
                lifecycle: VariableLifecycle::GeneratedSemantic,
                default_lower: Bound {
                    kind: BoundKind::Unbounded,
                    value: None,
                },
                default_upper: Bound {
                    kind: BoundKind::Unbounded,
                    value: None,
                },
                default_initial: None,
                derivation_id: Self::derivation(method.state, method.requirement, "kernel-symbol"),
            };
            self.remember_symbol(row.symbol_id);
            if let Some(previous) = self
                .symbols
                .insert((method.state, symbol, method.index.clone()), row.clone())
            {
                let mut previous = previous;
                previous.ordinal = row.ordinal;
                previous.derivation_id = row.derivation_id;
                if previous != row {
                    return Err(invalid(
                        "shared kernel output symbol has conflicting meanings",
                    ));
                }
            }
            self.append(
                "compiled.symbol_expressions",
                compiled::symbol_expressions::Row {
                    symbol_id: symbol,
                    node_id: root.0,
                    derivation_id: Self::derivation(method.state, symbol, "kernel-body"),
                },
            )?;
            for role in [
                ExpressionRootRole::SymbolExpression,
                ExpressionRootRole::MethodOutput,
            ] {
                self.append(
                    "compiled.expression_roots",
                    compiled::expression_roots::Row {
                        owner_id: symbol,
                        role,
                        ordinal: 0,
                        node_id: root.0,
                        derivation_id: Self::derivation(method.state, symbol, "kernel-root"),
                    },
                )?;
            }
            self.append(
                "compiled.method_realizations",
                compiled::method_realizations::Row {
                    requirement_id: method.requirement,
                    method_id: method.method,
                    output_symbol_id: symbol,
                    realization: compiled::method_realizations::CompiledMethodRealizationsFieldRealization::from_kernel_output(
                        compiled::method_realizations::CompiledMethodRealizationsFieldRealizationKernelOutput {
                            kernel_binding_id: binding,
                            output_ordinal: method.output_ordinal,
                        }
                    ),
                    derivation_id: Self::derivation(
                        method.state,
                        method.requirement,
                        "kernel-realization",
                    ),
                },
            )?;
            self.append(
                "compiled.kernel_output_symbols",
                compiled::kernel_output_symbols::Row {
                    symbol_id: symbol,
                    requirement_id: method.requirement,
                    state_scope_id: method.state_scope,
                    method_id: method.method,
                    kernel_binding_id: binding,
                    output_ordinal: method.output_ordinal,
                    index: method.index.clone(),
                    quantity_type_id: quantity.id.as_id(),
                    derivation_id: Self::derivation(
                        method.state,
                        method.requirement,
                        "kernel-correspondence",
                    ),
                },
            )?;
        }
        Ok(())
    }
}
