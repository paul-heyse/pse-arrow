// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! §6.8 symbol.

use crate::builder::RegistryBuilder;

/// Declares the §6.8 symbol contracts.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_solver_variable_type_vocabulary(builder);
    declare_variable_semantic_role_vocabulary(builder);
}

fn declare_solver_variable_type_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "SolverVariableType",
        ["continuous", "binary", "integer"],
    );
}

fn declare_variable_semantic_role_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "VariableSemanticRole",
        [
            "state",
            "design_capacity",
            "allocation",
            "slack",
            "aux_reformulation",
            "reporting_only",
        ],
    );
}
