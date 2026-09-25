// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

/// Current authored function vocabulary. Source-only forms have no numerical handler.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, strum::EnumString, strum::IntoStaticStr, strum::VariantArray,
)]
#[strum(serialize_all = "snake_case")]
pub enum Function {
    /// Exponential.
    Exp,
    /// Natural logarithm.
    Log,
    /// Base-ten logarithm.
    Log10,
    /// Square root.
    Sqrt,
    /// Value-only absolute value.
    Abs,
    /// Sine.
    Sin,
    /// Cosine.
    Cos,
    /// Tangent.
    Tan,
    /// Guarded minimum.
    Min,
    /// Guarded maximum.
    Max,
    /// Source inspection and physical unit rename; not executable arithmetic.
    Convert,
    /// Source inspection and lexical index binding; not executable arithmetic.
    Broadcast,
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
    /// Base-ten logarithm.
    Log10,
    /// Real square root.
    Sqrt,
    /// Value-only absolute value.
    Abs,
    /// Sine.
    Sin,
    /// Cosine.
    Cos,
    /// Guarded tangent.
    Tan,
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
}
impl Implementation {
    /// Stable capability spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unary(_) => "guarded_library_scalar",
            Self::Extremum { .. } => "guarded_extremum",
            Self::Unavailable => "unavailable",
        }
    }
}
impl Function {
    /// Bind each authored spelling to a concrete exhaustive handler, never a separate support flag.
    pub const fn implementation(self) -> Implementation {
        match self {
            Self::Exp => Implementation::Unary(UnaryFunction::Exp),
            Self::Log => Implementation::Unary(UnaryFunction::Log),
            Self::Log10 => Implementation::Unary(UnaryFunction::Log10),
            Self::Sqrt => Implementation::Unary(UnaryFunction::Sqrt),
            Self::Abs => Implementation::Unary(UnaryFunction::Abs),
            Self::Sin => Implementation::Unary(UnaryFunction::Sin),
            Self::Cos => Implementation::Unary(UnaryFunction::Cos),
            Self::Tan => Implementation::Unary(UnaryFunction::Tan),
            Self::Min => Implementation::Extremum { minimum: true },
            Self::Max => Implementation::Extremum { minimum: false },
            Self::Convert | Self::Broadcast => Implementation::Unavailable,
        }
    }
}
