// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Version-one modeling language over registry-generated semantic declarations.
mod cells;
mod parser;
mod render;
mod static_value;
mod types;
mod version;
pub use static_value::{StaticValue, parse_static, split};
#[cfg(test)]
mod kernel_language;

pub use parser::{IdentityPolicy, assign_ids, assign_ids_with, parse};
pub use pse_model::generated::authored::modeling_declarations::*;
pub use pse_model::generated::authored::modeling_declarations::{
    AuthoredModelingDeclarationsFieldValue as Value,
    AuthoredModelingDeclarationsFieldValueSelected as Selected, Row as Declaration,
};
pub use pse_model::generated::structures::{
    ModelingCompleteness, ModelingIntegerRange, ModelingLineageEntry, ModelingProvenance,
    VersionRequirement,
};
pub use cells::{
    Cell, CellBoolean, CellIdentifier, CellInteger, CellKind, CellPath, CellQuantity,
    CellReference, CellReferences, CellRow, CellSelected, CellText, CellUncertainty,
    CellUnitFactor, CellValue, KeyCell, KeyCellKind, KeyCellSelected, ModelingUncertaintyKind,
    cell, key_cell, key_cell_value, parse_cell, render_cell, unit_factors, unit_product,
};
pub use render::render;
pub use version::{exact_requirement, requirement_admits, requirement_version};
pub use types::{
    TypeArenaViolation, TypeExponent, TypeNode, TypeNodeKind, TypeRef, TypeTree, node as type_node,
    parse_type, render_type, type_paths,
};
