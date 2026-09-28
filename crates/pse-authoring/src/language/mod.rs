// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Version-one modeling language over registry-generated semantic declarations.
mod parser;
mod render;
mod static_value;
pub use static_value::{StaticValue, parse_static, split};
#[cfg(test)]
mod kernel_language;

pub use parser::{IdentityPolicy, assign_ids, assign_ids_with, parse};
pub use pse_model::generated::authored::modeling_declarations::*;
pub use pse_model::generated::authored::modeling_declarations::{
    AuthoredModelingDeclarationsFieldValue as Value,
    AuthoredModelingDeclarationsFieldValueSelected as Selected, Row as Declaration,
};
pub use render::render;
