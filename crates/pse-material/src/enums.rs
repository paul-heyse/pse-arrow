// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The closed enumerations of the material system (blueprint §6.4, §6.14).
//!
//! §6.14 lists the enumerations preserved from IDAES by name. Where IDAES has a spelling,
//! `as_str` *is* that spelling — `liquidPhase`, not `liquid`; `moleFraction`, not
//! `mole_fraction` — because the member text is the parity contract a governance test
//! compares against `idaes-pse==2.12.0`, and a prettier spelling here would be a second
//! authority for one fact (prime directive 3). [`ReactionKind`] has no IDAES enumeration
//! behind it, so it follows the §6.4 `snake_case` convention instead.
//!
//! This is a clean-room re-implementation: the member *names* are preserved deliberately
//! for parity, and nothing else is taken from the IDAES sources
//! (`docs/relationship-to-idaes.md`).

pse_quantity::closed_enum! {
    /// What kind of component a species is (blueprint §6.4, §6.14 `ComponentType`).
    ///
    /// A platform enumeration mirroring the IDAES component class hierarchy, so an
    /// `Apparent` species can carry `dissociation_species` rows and an `Ion` can carry a
    /// charge without either being a separate relation.
    pub enum ComponentType {
        /// An ordinary component.
        Component => "Component",
        /// A solute.
        Solute => "Solute",
        /// A solvent.
        Solvent => "Solvent",
        /// An ion of unspecified sign.
        Ion => "Ion",
        /// A negatively charged ion.
        Anion => "Anion",
        /// A positively charged ion.
        Cation => "Cation",
        /// An apparent species that dissociates into true species.
        Apparent => "Apparent",
    }
}

pse_quantity::closed_enum! {
    /// What kind of phase a phase is (blueprint §6.4, §6.14 `PhaseType`).
    ///
    /// The spellings are the IDAES ones: `undefined`, `liquidPhase`, `vaporPhase`,
    /// `solidPhase`, `aqueousPhase`.
    pub enum PhaseType {
        /// No phase type was declared; the phase is valid for any species.
        Undefined => "undefined",
        /// A liquid phase.
        Liquid => "liquidPhase",
        /// A vapor phase.
        Vapor => "vaporPhase",
        /// A solid phase.
        Solid => "solidPhase",
        /// An aqueous phase.
        Aqueous => "aqueousPhase",
    }
}

pse_quantity::closed_enum! {
    /// What kind of reaction a reaction is (blueprint §6.4, `authored.reactions.kind`).
    ///
    /// No IDAES enumeration corresponds, so these follow the §6.4 `snake_case` convention.
    pub enum ReactionKind {
        /// A kinetically limited reaction with a rate expression.
        Rate => "rate",
        /// A reaction at equilibrium.
        Equilibrium => "equilibrium",
        /// A reaction inherent to the property package, always at equilibrium.
        Inherent => "inherent",
    }
}

pse_quantity::closed_enum! {
    /// How a reaction's concentrations are expressed (blueprint §6.4, §6.14
    /// `ConcentrationForm`).
    pub enum ConcentrationForm {
        /// Moles per unit volume of solution.
        Molarity => "molarity",
        /// Thermodynamic activity.
        Activity => "activity",
        /// Moles of solute per unit mass of solvent.
        Molality => "molality",
        /// Moles of the species over total moles.
        MoleFraction => "moleFraction",
        /// Mass of the species over total mass.
        MassFraction => "massFraction",
        /// The species' partial pressure.
        PartialPressure => "partialPressure",
    }
}

pse_quantity::closed_enum! {
    /// Which form of Henry's law a declaration uses (blueprint §6.4, §6.14 `HenryType`).
    ///
    /// The numeric values are part of the contract, not row positions: §6.14 records that
    /// values below 51 are constants of the form `c/P` and values above are volatilities
    /// of the form `P/c`, so the code decides which way the correlation is applied.
    pub enum HenryType: u8 {
        /// Henry constant, concentration over pressure.
        Hcp = 1 => "Hcp",
        /// Henry constant, mole fraction over pressure.
        Hxp = 2 => "Hxp",
        /// Henry volatility, pressure over concentration.
        Kpc = 51 => "Kpc",
        /// Henry volatility, pressure over mole fraction.
        Kpx = 52 => "Kpx",
    }
}

impl HenryType {
    /// The threshold §6.14 states: below it a member is a constant `c/P`, at or above it a
    /// volatility `P/c`.
    const VOLATILITY_THRESHOLD: u8 = 51;

    /// Is this a volatility (`P/c`) rather than a constant (`c/P`)?
    pub const fn is_volatility(self) -> bool {
        self.code() >= Self::VOLATILITY_THRESHOLD
    }
}

#[cfg(test)]
mod tests {
    use super::{ComponentType, ConcentrationForm, HenryType, PhaseType, ReactionKind};

    /// The §6.14 spellings are the parity contract; `liquid` would break it.
    #[test]
    fn phase_types_carry_the_idaes_spelling() {
        assert_eq!(PhaseType::Undefined.as_str(), "undefined");
        assert_eq!(PhaseType::Liquid.as_str(), "liquidPhase");
        assert_eq!(PhaseType::Vapor.as_str(), "vaporPhase");
        assert_eq!(PhaseType::Solid.as_str(), "solidPhase");
        assert_eq!(PhaseType::Aqueous.as_str(), "aqueousPhase");
        assert_eq!(PhaseType::ALL.len(), 5);
    }

    #[test]
    fn concentration_forms_carry_the_idaes_spelling() {
        assert_eq!(ConcentrationForm::MoleFraction.as_str(), "moleFraction");
        assert_eq!(ConcentrationForm::MassFraction.as_str(), "massFraction");
        assert_eq!(
            ConcentrationForm::PartialPressure.as_str(),
            "partialPressure"
        );
        assert_eq!(ConcentrationForm::ALL.len(), 6);
    }

    #[test]
    fn component_types_carry_the_idaes_class_names() {
        assert_eq!(ComponentType::Component.as_str(), "Component");
        assert_eq!(ComponentType::Apparent.as_str(), "Apparent");
        assert_eq!(ComponentType::ALL.len(), 7);
    }

    /// A reaction kind has no IDAES enumeration behind it, so it uses the §6.4 convention.
    #[test]
    fn reaction_kinds_use_the_platform_convention() {
        assert_eq!(ReactionKind::Rate.as_str(), "rate");
        assert_eq!(ReactionKind::ALL.len(), 3);
    }

    /// The numeric values are the §6.14 contract with IDAES, not row positions.
    #[test]
    fn henry_type_discriminants_are_the_idaes_values() {
        assert_eq!(HenryType::Hcp.code(), 1);
        assert_eq!(HenryType::Hxp.code(), 2);
        assert_eq!(HenryType::Kpc.code(), 51);
        assert_eq!(HenryType::Kpx.code(), 52);
        assert_eq!(HenryType::Hcp as u8, 1);
        assert_eq!(HenryType::Kpx as u8, 52);
    }

    /// §6.14: below 51 a member is a constant `c/P`, at or above it a volatility `P/c`.
    #[test]
    fn the_henry_threshold_separates_constants_from_volatilities() {
        assert!(!HenryType::Hcp.is_volatility());
        assert!(!HenryType::Hxp.is_volatility());
        assert!(HenryType::Kpc.is_volatility());
        assert!(HenryType::Kpx.is_volatility());
    }

    /// Every member parses back from its own spelling, and an unknown spelling is `None`.
    #[test]
    fn every_enum_round_trips_through_its_spelling() {
        for member in ComponentType::ALL {
            assert_eq!(ComponentType::parse(member.as_str()), Some(*member));
        }
        for member in PhaseType::ALL {
            assert_eq!(PhaseType::parse(member.as_str()), Some(*member));
        }
        for member in ReactionKind::ALL {
            assert_eq!(ReactionKind::parse(member.as_str()), Some(*member));
        }
        for member in ConcentrationForm::ALL {
            assert_eq!(ConcentrationForm::parse(member.as_str()), Some(*member));
        }
        for member in HenryType::ALL {
            assert_eq!(HenryType::parse(member.as_str()), Some(*member));
        }
        assert_eq!(PhaseType::parse("liquid"), None);
        assert_eq!(ConcentrationForm::parse("mole_fraction"), None);
    }
}
