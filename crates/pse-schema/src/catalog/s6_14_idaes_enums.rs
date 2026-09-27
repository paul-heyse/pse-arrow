// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Blueprint §6.14 dictionaries; only sanctioned member names are preserved from IDAES.
//! Runtime enum parity against the pinned package is a separate qualification gate.

use crate::builder::RegistryBuilder;
use crate::model::{EnumDecl, EnumMember};

/// Declares the scaling strategy interpreted by the numerical kernel.
pub fn declare(builder: &mut RegistryBuilder) {
    compatibility(
        builder,
        "ConstraintScalingScheme",
        "idaes.core.scaling.custom_scaler_base",
        &[
            "harmonicMean",
            "inverseSum",
            "inverseRSS",
            "inverseMaximum",
            "inverseMinimum",
        ],
    );
}

fn compatibility(
    builder: &mut RegistryBuilder,
    name: &'static str,
    source: &'static str,
    members: &[&'static str],
) {
    builder.declare_enum(EnumDecl::idaes(
        name,
        source,
        members
            .iter()
            .map(|member| EnumMember::idaes(member, member, member))
            .collect(),
    ));
}
