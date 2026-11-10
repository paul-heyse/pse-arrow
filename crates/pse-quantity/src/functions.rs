// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

pse_contract::projection_enum! {
/// Current authored function vocabulary. Source-only forms have no numerical handler.
#[derive(
    serde::Serialize,
    serde::Deserialize,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
#[strum(serialize_all = "snake_case")]
pub enum Function {
    projection: { name: "AuthoredBuiltinFunction", rust_path: crate::functions::Function,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Exponential.
    Exp => 1;
    /// Natural logarithm.
    Log => 2;
    /// Square root.
    Sqrt => 3;
    /// Value-only absolute value.
    Abs => 4;
    /// Sine.
    Sin => 5;
    /// Cosine.
    Cos => 6;
    /// Guarded minimum.
    Min => 7;
    /// Guarded maximum.
    Max => 8;
    /// Source inspection and physical unit rename; not executable arithmetic.
    Convert => 9;
    /// Source inspection and lexical index binding; not executable arithmetic.
    Broadcast => 10;
    /// Stable natural logarithm of one plus a dimensionless increment.
    Log1p => 11;
    /// Euclidean magnitude of two compatible linear or difference quantities.
    Magnitude => 12;
    /// Stable natural logarithm of a positive ratio, retaining both arguments.
    LogRatio => 13;
}
}
impl Function {
    /// Exact current spellings, generated from the enum declaration.
    pub fn parse(name: &str) -> Option<Self> {
        name.parse().ok()
    }
    /// Declared authored spelling.
    pub fn as_str(self) -> &'static str {
        self.into()
    }
    /// All declared variants in declaration order.
    pub fn all() -> &'static [Self] {
        <Self as strum::VariantArray>::VARIANTS
    }
}

/// Exhaustive implemented unary constructors. Every variant has physical admission and library arithmetic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryFunction {
    /// Natural exponential.
    Exp,
    /// Natural logarithm.
    Log,
    /// Stable natural logarithm of one plus the argument.
    Log1p,
    /// Real square root.
    Sqrt,
    /// Value-only absolute value.
    Abs,
    /// Sine.
    Sin,
    /// Cosine.
    Cos,
}
/// Actual exhaustive handler, also projected into generated capability metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Implementation {
    /// Library arithmetic with its physical and domain constructor.
    Unary(UnaryFunction),
    /// Lazy scalar comparison and selected value.
    Extremum {
        /// Select minimum when true, maximum otherwise.
        minimum: bool,
    },
    /// Authored vocabulary outside the selected execution profile.
    Unavailable,
    /// Stable two-component Euclidean magnitude.
    Magnitude,
    /// Stable positive ratio with original argument guards.
    LogRatio,
}
impl Implementation {
    /// Stable capability spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unary(_) => "guarded_library_scalar",
            Self::Extremum { .. } => "guarded_extremum",
            Self::Unavailable => "unavailable",
            Self::Magnitude | Self::LogRatio => "guarded_library_scalar",
        }
    }
}
impl Function {
    /// Bind each authored spelling to a concrete exhaustive handler, never a separate support flag.
    pub const fn implementation(self) -> Implementation {
        match self {
            Self::Exp => Implementation::Unary(UnaryFunction::Exp),
            Self::Log => Implementation::Unary(UnaryFunction::Log),
            Self::Log1p => Implementation::Unary(UnaryFunction::Log1p),
            Self::Magnitude => Implementation::Magnitude,
            Self::LogRatio => Implementation::LogRatio,
            Self::Sqrt => Implementation::Unary(UnaryFunction::Sqrt),
            Self::Abs => Implementation::Unary(UnaryFunction::Abs),
            Self::Sin => Implementation::Unary(UnaryFunction::Sin),
            Self::Cos => Implementation::Unary(UnaryFunction::Cos),
            Self::Min => Implementation::Extremum { minimum: true },
            Self::Max => Implementation::Extremum { minimum: false },
            Self::Convert | Self::Broadcast => Implementation::Unavailable,
        }
    }
}
