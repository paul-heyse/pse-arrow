// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The closed enumerations of the physical-typing registry (blueprint §6.2, §6.3, §7.2).
//!
//! Every enumeration here is a *closed dictionary*: the member list in this file is the
//! authority the registry relation (`schema_enums`) and the generated Python and Arrow
//! dictionaries agree with, and [`closed_enum`] makes the spelling of a member exist
//! exactly once. `as_str` is that spelling, `parse` is its inverse, `Display` and the
//! `serde` implementations both route through `as_str`, so a member cannot acquire a
//! second textual form by being serialized a different way (prime directive 3).
//!
//! Spelling conventions follow the section the member list comes from: [`Opcode`] uses the
//! `PascalCase` opcode names of §7.2, and everything else uses the `snake_case` spelling
//! the relation declarations of §6.2 and §6.3 write inside `enum(...)`.
//!
//! [`closed_enum`]: crate::closed_enum

/// Declares a closed enumeration with one authoritative spelling per member.
///
/// The macro generates the enumeration, `ALL` in declaration order, `as_str`, `parse`,
/// `Display` and `serde` implementations that both route through `as_str`. Declaration
/// order is registry order.
///
/// Two forms exist. The plain form declares members without discriminants:
///
/// ```
/// pse_quantity::closed_enum! {
///     /// Which end of a pipe a stream enters.
///     pub enum Side {
///         /// The upstream end.
///         Inlet => "inlet",
///         /// The downstream end.
///         Outlet => "outlet",
///     }
/// }
///
/// assert_eq!(Side::ALL.len(), 2);
/// assert_eq!(Side::Inlet.as_str(), "inlet");
/// assert_eq!(Side::parse("outlet"), Some(Side::Outlet));
/// assert_eq!(Side::parse("nowhere"), None);
/// ```
///
/// The second form pins the discriminants an upstream enumeration assigns, and adds
/// `code()` returning them:
///
/// ```
/// pse_quantity::closed_enum! {
///     /// An enumeration whose numeric values are part of its contract.
///     pub enum Coded: u8 {
///         /// The first member.
///         First = 1 => "first",
///         /// The fifty-first member.
///         FiftyFirst = 51 => "fifty_first",
///     }
/// }
///
/// assert_eq!(Coded::FiftyFirst.code(), 51);
/// ```
#[macro_export]
macro_rules! closed_enum {
    (@impls $name:ident { $($variant:ident => $text:literal),+ }) => {
        impl $name {
            #[doc = concat!("Every member of [`", stringify!($name), "`], in registry order.")]
            ///
            /// Declaration order is the order the registry relation, the generated Arrow
            /// dictionary and the generated Python enumeration all use; it is part of the
            /// contract, not an implementation detail.
            pub const ALL: &'static [Self] = <Self as ::strum::VariantArray>::VARIANTS;

            #[doc = concat!("The registry spelling of this [`", stringify!($name), "`] member.")]
            ///
            /// This is the only textual form of the member: `Display` and `serde` both
            /// route through it.
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $text,)+
                }
            }

            #[doc = concat!("The [`", stringify!($name), "`] member with this spelling, if any.")]
            ///
            /// The inverse of [`Self::as_str`]. An unknown spelling is `None` rather than a
            /// fallback member, because a closed dictionary has no "other".
            pub fn parse(text: &str) -> ::core::option::Option<Self> {
                <Self as ::core::str::FromStr>::from_str(text).ok()
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                let text =
                    <::std::string::String as ::serde::Deserialize>::deserialize(deserializer)?;
                Self::parse(&text).ok_or_else(|| {
                    <D::Error as ::serde::de::Error>::invalid_value(
                        ::serde::de::Unexpected::Str(&text),
                        &concat!("a `", stringify!($name), "` member spelling"),
                    )
                })
            }
        }
    };

    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident : $repr:ident {
            $(
                $(#[$vmeta:meta])*
                $variant:ident = $value:literal => $text:literal
            ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, ::strum::EnumString, ::strum::Display, ::strum::VariantArray)]
        #[repr($repr)]
        $vis enum $name {
            $(
                $(#[$vmeta])*
                #[strum(serialize = $text)]
                $variant = $value,
            )+
        }

        impl $name {
            #[doc = concat!("The upstream numeric value of this [`", stringify!($name), "`] member.")]
            ///
            /// The value is part of the contract with the system the enumeration was
            /// preserved from; it is never a row position or an artifact-local ordinal.
            pub const fn code(self) -> $repr {
                self as $repr
            }
        }

        $crate::closed_enum!(@impls $name { $($variant => $text),+ });
    };

    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$vmeta:meta])*
                $variant:ident => $text:literal
            ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, ::strum::EnumString, ::strum::Display, ::strum::VariantArray)]
        $vis enum $name {
            $(
                $(#[$vmeta])*
                #[strum(serialize = $text)]
                $variant,
            )+
        }

        $crate::closed_enum!(@impls $name { $($variant => $text),+ });
    };
}

crate::closed_enum! {
    /// Physical operation vocabulary for reference quantity rules.
    ///
    /// Membership describes physical signatures, not executable math capability.
    /// The compiler admits only operations implemented by `pse-math`.
    pub enum Opcode {
        /// A literal value with an explicit unit, or an integer literal.
        Const => "Const",
        /// A reference to a symbol; the symbol supplies the complete quantity type.
        SymbolRef => "SymbolRef",
        /// Ordered binary addition under the complete §8.3 addition rule.
        Add => "Add",
        /// Ordered binary subtraction under the complete §8.3 addition rule.
        Sub => "Sub",
        /// An ordered constant plus coefficient/child terms; children may be nonlinear.
        Affine => "Affine",
        /// An ordered weight/value mean with an explicit normalization policy.
        WeightedMean => "WeightedMean",
        /// Ordered binary multiplication.
        Mul => "Mul",
        /// Ordered binary division; the divisor must be nonzero.
        Div => "Div",
        /// Exponentiation; the exponent is dimensionless unless the base is.
        Pow => "Pow",
        /// Arithmetic negation.
        Neg => "Neg",
        /// Absolute value; nonsmooth at zero.
        Abs => "Abs",
        /// The exponential function; dimensionless in, dimensionless out.
        Exp => "Exp",
        /// The natural logarithm; the argument must be positive.
        Log => "Log",
        /// The base-10 logarithm; the argument must be positive.
        Log10 => "Log10",
        /// The square root; the argument must be nonnegative, and it is nonsmooth at zero.
        Sqrt => "Sqrt",
        /// The sine of a dimensionless argument.
        Sin => "Sin",
        /// The cosine of a dimensionless argument.
        Cos => "Cos",
        /// The tangent of a dimensionless argument.
        Tan => "Tan",
        /// The arcsine; the argument lies in `[-1, 1]`.
        Asin => "Asin",
        /// The arccosine; the argument lies in `[-1, 1]`.
        Acos => "Acos",
        /// The arctangent of a dimensionless argument.
        Atan => "Atan",
        /// The hyperbolic sine of a dimensionless argument.
        Sinh => "Sinh",
        /// The hyperbolic cosine of a dimensionless argument.
        Cosh => "Cosh",
        /// The hyperbolic tangent of a dimensionless argument.
        Tanh => "Tanh",
        /// The error function of a dimensionless argument.
        Erf => "Erf",
        /// A smooth maximum of two arguments with a positive smoothing parameter.
        SmoothMax => "SmoothMax",
        /// A smooth minimum of two arguments with a positive smoothing parameter.
        SmoothMin => "SmoothMin",
        /// A smooth absolute value with a positive smoothing parameter.
        SmoothAbs => "SmoothAbs",
        /// A square root regularized by a positive smoothing parameter.
        SafeSqrt => "SafeSqrt",
        /// A logarithm regularized by a positive smoothing parameter.
        SafeLog => "SafeLog",
        /// A branch on an explicit guard; the branches share a complete quantity type.
        Conditional => "Conditional",
        /// A sum of its body over a bound index of a domain.
        SumOver => "SumOver",
        /// A product of its body over a bound index of a domain.
        ProdOver => "ProdOver",
        /// A minimum of its body over a bound index of a domain.
        MinOver => "MinOver",
        /// A maximum of its body over a bound index of a domain.
        MaxOver => "MaxOver",
        /// A read of a group member at an explicit coordinate map.
        Gather => "Gather",
        /// An explicit broadcast of its body over a domain.
        Broadcast => "Broadcast",
        /// A derivative of its body with respect to a continuous domain.
        Derivative => "Derivative",
        /// An integral of its body over a continuous domain.
        Integral => "Integral",
        /// A call into a kernel binding, selecting one declared output.
        KernelCall => "KernelCall",
        /// A reference to one unknown of an implicit system.
        ImplicitRef => "ImplicitRef",
        /// A declared unit-conversion edge, inserted by P3 or P9 and checked by P10.
        UnitConvert => "UnitConvert",
        /// A piecewise-linear interpolation over declared breakpoints.
        PiecewiseLinear => "PiecewiseLinear",
    }
}

crate::closed_enum! {
    /// What a domain indexes (blueprint §6.3, `authored.domains.kind`).
    pub enum DomainKind {
        /// A time domain; continuous in dynamic models.
        Time => "time",
        /// A spatial length domain.
        Length => "length",
        /// The species of a material system.
        Species => "species",
        /// The phases of a material system.
        Phase => "phase",
        /// Valid phase/species pairs.
        PhaseSpecies => "phase_species",
        /// The chemical elements a species is composed of.
        Element => "element",
        /// The reactions of a reaction package.
        Reaction => "reaction",
        /// A set of ports on an instance.
        PortSet => "port_set",
        /// The stages of a staged unit.
        Stage => "stage",
        /// Discretization cells.
        Cell => "cell",
        /// Discretization faces.
        Face => "face",
        /// Mesh nodes.
        Node => "node",
        /// A domain a package declares that no other member names.
        Custom => "custom",
    }
}

crate::closed_enum! {
    /// What a quantity is *about* (blueprint §6.2, `reference.quantity_types.subject_kind`).
    ///
    /// The subject is not the shape: a per-phase composition of one species has shape
    /// `[phase]` and subject `species`.
    pub enum SubjectKind {
        /// The quantity is about one species.
        Species => "species",
        /// The quantity is about one phase.
        Phase => "phase",
        /// The quantity is about one element.
        Element => "element",
        /// The quantity is about one reaction.
        Reaction => "reaction",
        /// The quantity is about nothing in particular.
        None => "none",
    }
}

crate::closed_enum! {
    /// Whether a quantity is a point on an affine scale or a difference of two points
    /// (blueprint §8.1).
    pub enum ScaleKind {
        /// A point: summing two of them is meaningless, and an affine unit offset applies.
        Point => "point",
        /// A difference of two points on one scale: additive, and unit offsets do not apply.
        Difference => "difference",
    }
}

crate::closed_enum! {
    /// Whether a kind's values may be summed freely (blueprint §6.2, §8.3).
    ///
    /// The distinction is what keeps the sum of two material flows from being rejected as
    /// an invalid affine-point sum.
    pub enum QuantityAdditionKind {
        /// Flows, transported energy, amounts: a signed sum keeps the ordinary point form.
        Additive => "additive",
        /// Temperature, pressure, specific enthalpy: a sum of two points is rejected.
        OriginSensitive => "origin_sensitive",
    }
}

crate::closed_enum! {
    /// The amount a specific quantity is specific *to* (blueprint §6.2, `reference.bases.kind`).
    pub enum BasisKind {
        /// Per mole.
        Molar => "molar",
        /// Per unit mass.
        Mass => "mass",
        /// Per unit volume.
        Volume => "volume",
        /// Per unit energy.
        Energy => "energy",
        /// Per unit volume at a declared standard state.
        StandardVolume => "standard_volume",
        /// No basis obligation at all.
        Dimensionless => "dimensionless",
    }
}

crate::closed_enum! {
    /// How a composition is expressed (blueprint §6.2, `reference.bases.composition_basis`).
    pub enum CompositionBasis {
        /// Moles of the species over total moles.
        MoleFraction => "mole_fraction",
        /// Mass of the species over total mass.
        MassFraction => "mass_fraction",
        /// Volume of the species over total volume.
        VolumeFraction => "volume_fraction",
        /// Moles of solute per unit mass of solvent.
        Molality => "molality",
        /// Moles of solute per unit volume of solution.
        Molarity => "molarity",
    }
}

crate::closed_enum! {
    /// What a rate is per (blueprint §6.2, `reference.bases.rate_basis`).
    ///
    /// §6.2 declares the column without listing its members. These three are the working
    /// set for wave 1; registry packet A-1 declares the same members in `schema_enums`, and
    /// a conformance test compares the two lists. Adding a member is a registry change,
    /// not a local edit here.
    #[allow(
        clippy::enum_variant_names,
        reason = "the shared `Per` prefix is the registry spelling of `rate_basis`, not a naming habit"
    )]
    pub enum RateBasis {
        /// Per unit volume, as in a homogeneous reaction rate.
        PerVolume => "per_volume",
        /// Per unit mass, as in a catalyst-mass-based rate.
        PerMass => "per_mass",
        /// Per unit area, as in a surface reaction or a flux.
        PerArea => "per_area",
    }
}

crate::closed_enum! {
    /// Which convention a reference state fixes (blueprint §6.2, §8.4).
    pub enum ReferenceStateKind {
        /// Elements in their reference form at the declared conditions.
        ElementalAtConditions => "elemental_at_conditions",
        /// The compound itself at the declared conditions.
        CompoundAtConditions => "compound_at_conditions",
        /// The ideal gas at the declared conditions.
        IdealGasAtConditions => "ideal_gas_at_conditions",
        /// A convention a package declares explicitly.
        Custom => "custom",
    }
}

crate::closed_enum! {
    /// How a registered conversion between two quantity types is carried out
    /// (blueprint §6.2, `reference.conversion_rules.kind`).
    pub enum ConversionKind {
        /// A multiplication by a constant factor.
        Scale => "scale",
        /// A multiplication followed by an offset, as between an absolute and a gauge datum.
        Affine => "affine",
        /// A kernel call with declared parameters, as molar to mass through the molecular
        /// weight.
        Kernel => "kernel",
    }
}

crate::closed_enum! {
    /// How a registered operation resolves the basis of its result (blueprint §6.2, §8.3).
    pub enum BasisRule {
        /// Take the basis of the operand named by `basis_source`.
        Preserve => "preserve",
        /// Require every relevant operand to agree, and return that common basis.
        RequireEqual => "require_equal",
        /// Insert the registered conversion before matching.
        RegisteredConversion => "registered_conversion",
        /// Require matching non-absent tags under the operation's invariant, and return none.
        Cancel => "cancel",
        /// Use the operation's explicit result field under a required invariant.
        DeclaredResult => "declared_result",
    }
}

crate::closed_enum! {
    /// How a registered operation resolves the reference state of its result
    /// (blueprint §6.2, §8.3).
    pub enum ReferenceRule {
        /// Take the reference state of the operand named by `reference_source`.
        Preserve => "preserve",
        /// Require every relevant operand to agree, and return that common reference state.
        RequireEqual => "require_equal",
        /// Insert the registered conversion before matching.
        RegisteredConversion => "registered_conversion",
        /// Require matching non-absent tags under the operation's invariant, and return none.
        Cancel => "cancel",
        /// Use the operation's explicit result field under a required invariant.
        DeclaredResult => "declared_result",
    }
}

crate::closed_enum! {
    /// How a registered operation resolves the point/difference scale of its result
    /// (blueprint §6.2, §8.3).
    pub enum QuantityScaleRule {
        /// Take the scale kind of the operand named by `scale_source`.
        Preserve => "preserve",
        /// The result is a point.
        Point => "point",
        /// The result is a difference.
        Difference => "difference",
        /// Apply the origin-sensitive addition table (point ± difference, point − point, …).
        Addition => "addition",
        /// Apply the weighted-mean rule, the one rule that may average points.
        WeightedMean => "weighted_mean",
    }
}

crate::closed_enum! {
    /// How a registered operation resolves the index shape of its result
    /// (blueprint §6.2, §8.3).
    pub enum QuantityShapeRule {
        /// The result is a scalar.
        Scalar => "scalar",
        /// Take the shape of the operand named by `shape_source`.
        Preserve => "preserve",
        /// Require every operand to carry the same index identities, and keep them.
        SameIndices => "same_indices",
        /// The operation adds the index its payload names.
        ExplicitBroadcast => "explicit_broadcast",
        /// The operation binds and removes the index its payload names.
        ReduceBoundIndex => "reduce_bound_index",
    }
}

crate::closed_enum! {
    /// How a registered operation resolves the subject of its result
    /// (blueprint §6.2, §8.3).
    pub enum SubjectRule {
        /// Take the subject of the operand named by `subject_source`.
        Preserve => "preserve",
        /// Require every relevant operand to agree, and return that common subject.
        RequireEqual => "require_equal",
        /// Use the operation's explicit `result_subject_kind`.
        DeclaredResult => "declared_result",
    }
}

crate::closed_enum! {
    /// How a weighted mean normalizes its weights (blueprint §6.9, §7.2).
    pub enum WeightNormalization {
        /// Evaluate an ordered numerator over an ordered denominator; a zero sum is a typed
        /// domain error.
        DivideBySum => "divide_by_sum",
        /// Trust a named invariant that the weights sum to one; only generated templates
        /// may select it.
        CertifiedUnitSum => "certified_unit_sum",
    }
}

crate::closed_enum! {
    /// Which reduction a `SumOver`/`ProdOver`/`MinOver`/`MaxOver` node performs
    /// used by physically typed finite reductions.
    pub enum ReductionKind {
        /// An ordered sum over the bound index.
        Sum => "sum",
        /// An ordered product over the bound index.
        Prod => "prod",
        /// A minimum over the bound index; nonsmooth.
        Min => "min",
        /// A maximum over the bound index; nonsmooth.
        Max => "max",
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize as _;
    use serde::de::IntoDeserializer as _;
    use serde::de::value::{Error as ValueError, StrDeserializer};

    use super::{
        BasisKind, BasisRule, CompositionBasis, ConversionKind, DomainKind, Opcode,
        QuantityAdditionKind, QuantityScaleRule, QuantityShapeRule, RateBasis, ReductionKind,
        ReferenceRule, ReferenceStateKind, ScaleKind, SubjectKind, SubjectRule,
        WeightNormalization,
    };

    /// The two methods the shared round-trip body needs.
    trait HasText: Sized + Copy + PartialEq + core::fmt::Debug {
        fn text(&self) -> &'static str;
        fn from_text(text: &str) -> Option<Self>;
    }

    macro_rules! has_text {
        ($($name:ident),+ $(,)?) => {
            $(
                impl HasText for $name {
                    fn text(&self) -> &'static str {
                        self.as_str()
                    }
                    fn from_text(text: &str) -> Option<Self> {
                        Self::parse(text)
                    }
                }
            )+
        };
    }

    has_text!(
        Opcode,
        DomainKind,
        SubjectKind,
        ScaleKind,
        QuantityAdditionKind,
        BasisKind,
        CompositionBasis,
        RateBasis,
        ReferenceStateKind,
        ConversionKind,
        BasisRule,
        ReferenceRule,
        QuantityScaleRule,
        QuantityShapeRule,
        SubjectRule,
        WeightNormalization,
        ReductionKind,
    );

    /// `parse` inverts `as_str` on every member, and no two members share a spelling.
    fn round_trip<T: HasText>(all: &'static [T]) {
        let mut seen: Vec<&'static str> = Vec::new();
        for member in all {
            let text = member.text();
            assert_eq!(
                T::from_text(text),
                Some(*member),
                "`{text}` does not parse back to its own member"
            );
            assert!(!seen.contains(&text), "`{text}` is spelled twice");
            seen.push(text);
        }
        assert_eq!(T::from_text("definitely not a member"), None);
    }

    #[test]
    fn every_enum_round_trips_through_its_registry_spelling() {
        round_trip(Opcode::ALL);
        round_trip(DomainKind::ALL);
        round_trip(SubjectKind::ALL);
        round_trip(ScaleKind::ALL);
        round_trip(QuantityAdditionKind::ALL);
        round_trip(BasisKind::ALL);
        round_trip(CompositionBasis::ALL);
        round_trip(RateBasis::ALL);
        round_trip(ReferenceStateKind::ALL);
        round_trip(ConversionKind::ALL);
        round_trip(BasisRule::ALL);
        round_trip(ReferenceRule::ALL);
        round_trip(QuantityScaleRule::ALL);
        round_trip(QuantityShapeRule::ALL);
        round_trip(SubjectRule::ALL);
        round_trip(WeightNormalization::ALL);
        round_trip(ReductionKind::ALL);
    }

    /// §6.3 spells domain kinds in `snake_case`; §7.2 spells opcodes in `PascalCase`.
    #[test]
    fn spellings_follow_the_section_they_come_from() {
        assert_eq!(DomainKind::PhaseSpecies.as_str(), "phase_species");
        assert_eq!(DomainKind::PortSet.as_str(), "port_set");
        assert_eq!(Opcode::WeightedMean.as_str(), "WeightedMean");
        assert_eq!(
            QuantityAdditionKind::OriginSensitive.as_str(),
            "origin_sensitive"
        );
        assert_eq!(WeightNormalization::DivideBySum.as_str(), "divide_by_sum");
    }

    /// `Display` and `Deserialize` read the same spelling `as_str` writes: one authority.
    #[test]
    fn display_and_serde_use_the_registry_spelling() {
        assert_eq!(DomainKind::Cell.to_string(), "cell");

        let de: StrDeserializer<'_, ValueError> = "SumOver".into_deserializer();
        assert_eq!(Opcode::deserialize(de).ok(), Some(Opcode::SumOver));
    }

    /// A spelling outside the dictionary is a deserialization failure, never a fallback.
    #[test]
    fn an_unknown_spelling_fails_deserialization() {
        let de: StrDeserializer<'_, ValueError> = "sum_over".into_deserializer();
        assert!(Opcode::deserialize(de).is_err());
    }
}
