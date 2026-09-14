// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Declared 6.12 derived structure contracts.
use super::declarations::{column, relation};
use crate::RegistryBuilder;
use crate::model::{LogicalType as T, Namespace as N, SnapshotClass as S};
/// Declare the structural contracts; no later pass is implemented by these declarations.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_problems(builder);
    declare_variable_order(builder);
    declare_equation_order(builder);
    declare_bound_values(builder);
    declare_case_bound_substitutions(builder);
    declare_incidence(builder);
    declare_dm_partition(builder);
    declare_blocks(builder);
    declare_block_members(builder);
    declare_sparsity_patterns(builder);
    declare_evaluation_programs(builder);
    declare_backend_bindings(builder);
    super::declarations::enumeration(builder, "RowKind", ["equality", "inequality", "objective"]);
    super::declarations::enumeration(
        builder,
        "ValueSource",
        ["case_spec", "package_default", "template_default"],
    );
    super::declarations::enumeration(builder, "DMKind", ["equation", "variable"]);
    super::declarations::enumeration(
        builder,
        "DMBlock",
        ["underconstrained", "square", "overconstrained"],
    );
    super::declarations::enumeration(
        builder,
        "BlockKind",
        ["scc", "connected_component", "bordered_diagonal"],
    );
    super::declarations::enumeration(builder, "SparsityKind", ["jacobian", "hessian"]);
    super::declarations::enumeration(builder, "BindingStatus", ["supported", "unsupported"]);
}
fn declare_problems(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "problems",
        S::Derived,
        &["problem_id"],
        vec![
            column("problem_id", T::id()),
            column("model_revision_id", T::id()),
            column("case_id", T::id()),
            column("discretization_policy_ids", T::list(T::id())),
            column("variable_count", T::U64),
            column("equation_count", T::U64),
            column("inequality_count", T::U64),
            column("objective_count", T::U64),
            column("degrees_of_freedom", T::I64),
            column("input_bundle_hash", T::hash()),
        ],
        "blueprint §6.12 derived structure: problems.",
    );
}

fn declare_variable_order(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "variable_order",
        S::Derived,
        &["problem_id", "symbol_id"],
        vec![
            column("problem_id", T::id()),
            column("symbol_id", T::id()),
            column("position", T::U64).optional(),
            column("treatment", T::enumeration("Treatment")),
            column("lower", T::Ext(crate::model::ExtensionUse::Bound)),
            column("upper", T::Ext(crate::model::ExtensionUse::Bound)),
            column("initial", T::F64).optional(),
            column("scale", T::F64),
            column("offset", T::F64),
        ],
        "blueprint §6.12 derived structure: variable_order.",
    );
}

fn declare_equation_order(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "equation_order",
        S::Derived,
        &["problem_id", "equation_id"],
        vec![
            column("problem_id", T::id()),
            column("equation_id", T::id()),
            column("position", T::U64).optional(),
            column("active", T::Bool),
            column("scale", T::F64),
            column("kind", T::enumeration("RowKind")),
        ],
        "blueprint §6.12 derived structure: equation_order.",
    );
}

fn declare_bound_values(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "bound_values",
        S::Derived,
        &["problem_id", "symbol_id"],
        vec![
            column("problem_id", T::id()),
            column("symbol_id", T::id()),
            column("treatment", T::enumeration("Treatment")),
            column("value", T::F64),
            column("unit_id", T::id()),
            column("value_hash", T::hash()),
            column("source_spec_id", T::id()).optional(),
        ],
        "blueprint §6.12 derived structure: bound_values.",
    );
}

fn declare_case_bound_substitutions(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "case_bound_substitutions",
        S::Derived,
        &["problem_id", "symbol_id"],
        vec![
            column("problem_id", T::id()),
            column("symbol_id", T::id()),
            column("value", T::F64),
            column("unit_id", T::id()),
            column("value_hash", T::hash()),
            column("source", T::enumeration("ValueSource")),
        ],
        "blueprint §6.12 derived structure: case_bound_substitutions.",
    );
}

fn declare_incidence(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "incidence",
        S::Derived,
        &["problem_id", "equation_id", "symbol_id"],
        vec![
            column("problem_id", T::id()),
            column("equation_id", T::id()),
            column("symbol_id", T::id()),
            column("linear", T::Bool),
            column("coefficient", T::F64).optional(),
        ],
        "blueprint §6.12 derived structure: incidence.",
    );
}

fn declare_dm_partition(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "dm_partition",
        S::Derived,
        &["problem_id", "kind", "id"],
        vec![
            column("problem_id", T::id()),
            column("kind", T::enumeration("DMKind")),
            column("id", T::id()),
            column("block", T::enumeration("DMBlock")),
            column("matched_id", T::id()).optional(),
        ],
        "blueprint §6.12 derived structure: dm_partition.",
    );
}

fn declare_blocks(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "blocks",
        S::Derived,
        &["block_id"],
        vec![
            column("problem_id", T::id()),
            column("block_id", T::id()),
            column("kind", T::enumeration("BlockKind")),
            column("order", T::U32),
            column("size", T::U32),
        ],
        "blueprint §6.12 derived structure: blocks.",
    );
}

fn declare_block_members(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "block_members",
        S::Derived,
        &["block_id", "kind", "id"],
        vec![
            column("block_id", T::id()),
            column("kind", T::enumeration("DMKind")),
            column("id", T::id()),
        ],
        "blueprint §6.12 derived structure: block_members.",
    );
}

fn declare_sparsity_patterns(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "sparsity_patterns",
        S::Derived,
        &["problem_id", "kind"],
        vec![
            column("problem_id", T::id()),
            column("kind", T::enumeration("SparsityKind")),
            column("row_ptr", T::list(T::U32)),
            column("col_idx", T::list(T::U32)),
            column("content_hash", T::hash()),
        ],
        "blueprint §6.12 derived structure: sparsity_patterns.",
    );
}

fn declare_evaluation_programs(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "evaluation_programs",
        S::Derived,
        &["program_id"],
        vec![
            column("problem_id", T::id()),
            column("program_id", T::id()),
            column("artifact_hash", T::hash()),
            column("instruction_count", T::U64),
            column("workspace_size", T::U64),
        ],
        "blueprint §6.12 derived structure: evaluation_programs.",
    );
}

fn declare_backend_bindings(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "backend_bindings",
        S::Derived,
        &["problem_id", "backend"],
        vec![
            column("problem_id", T::id()),
            column("backend", T::enumeration("Backend")),
            column("status", T::enumeration("BindingStatus")),
            column("unsupported_opcodes", T::list(T::Text)),
            column("artifact_hash", T::hash()).optional(),
        ],
        "blueprint §6.12 derived structure: backend_bindings.",
    );
}
