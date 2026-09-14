// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Blueprint §6.14 dictionaries; only sanctioned member names are preserved from IDAES.
//! Runtime enum parity against the pinned package is a separate qualification gate.

use crate::builder::RegistryBuilder;
use crate::model::{EnumDecl, EnumMember};

/// Declares the named compatibility dictionaries.
#[allow(
    clippy::too_many_lines,
    reason = "A flat sanctioned dictionary inventory is reviewed against blueprint section 6.14 as one declaration."
)]
pub fn declare(builder: &mut RegistryBuilder) {
    compatibility(
        builder,
        "DistributedVars",
        "idaes.core.base.control_volume1d",
        &["variant", "uniform"],
    );
    compatibility(
        builder,
        "MaterialBalanceType",
        "idaes.core.base.control_volume_base",
        &[
            "useDefault",
            "none",
            "componentPhase",
            "componentTotal",
            "elementTotal",
            "total",
        ],
    );
    compatibility(
        builder,
        "EnergyBalanceType",
        "idaes.core.base.control_volume_base",
        &[
            "useDefault",
            "none",
            "enthalpyPhase",
            "enthalpyTotal",
            "energyPhase",
            "energyTotal",
            "isothermal",
        ],
    );
    compatibility(
        builder,
        "MomentumBalanceType",
        "idaes.core.base.control_volume_base",
        &[
            "none",
            "pressureTotal",
            "pressurePhase",
            "momentumTotal",
            "momentumPhase",
        ],
    );
    compatibility(
        builder,
        "FlowDirection",
        "idaes.core.base.control_volume_base",
        &["notSet", "forward", "backward"],
    );
    compatibility(
        builder,
        "MaterialFlowBasis",
        "idaes.core.base.process_base",
        &["molar", "mass", "other"],
    );
    compatibility(
        builder,
        "InitializationStatus",
        "idaes.core.initialization.initializer_base",
        &["Ok", "none", "Failed", "DoF", "PrecheckFailed", "Error"],
    );
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
    compatibility(
        builder,
        "DefaultScalingRecommendation",
        "idaes.core.scaling.custom_scaler_base",
        &[
            "userInputRecommended",
            "userInputRequired",
            "userSetManually",
        ],
    );
    compatibility(
        builder,
        "DaeVarTypes",
        "idaes.core.solvers.petsc",
        &["ALGEBRAIC", "DIFFERENTIAL", "DERIVATIVE", "TIME"],
    );
    compatibility(
        builder,
        "ControllerType",
        "idaes.models.control.controller",
        &["P", "PI", "PD", "PID"],
    );
    compatibility(
        builder,
        "ControllerMVBoundType",
        "idaes.models.control.controller",
        &["NONE", "SMOOTH_BOUND", "LOGISTIC"],
    );
    compatibility(
        builder,
        "ControllerAntiwindupType",
        "idaes.models.control.controller",
        &["NONE", "CONDITIONAL_INTEGRATION", "BACK_CALCULATION"],
    );
    compatibility(
        builder,
        "HXType",
        "idaes.models.costing.SSLW",
        &["floating_head", "fixed_head", "Utube", "kettle_vap"],
    );
    compatibility(
        builder,
        "HXMaterial",
        "idaes.models.costing.SSLW",
        &[
            "CarbonSteelCarbonSteel",
            "CarbonSteelBrass",
            "CarbonSteelStainlessSteel",
            "CarbonSteelMonel",
            "CarbonSteelTitanium",
            "CarbonSteelCrMoSteel",
            "CrMoSteelCrMoSteel",
            "StainlessSteelStainlessSteel",
            "MonelMonel",
            "TitaniumTitanium",
        ],
    );
    compatibility(
        builder,
        "HXTubeLength",
        "idaes.models.costing.SSLW",
        &["EightFoot", "TwelveFoot", "SixteenFoot", "TwentyFoot"],
    );
    compatibility(
        builder,
        "VesselMaterial",
        "idaes.models.costing.SSLW",
        &[
            "CarbonSteel",
            "LowAlloySteel",
            "StainlessSteel304",
            "StainlessSteel316",
            "Carpenter20CB3",
            "Nickel200",
            "Monel400",
            "Inconel600",
            "Incoloy825",
            "Titanium",
        ],
    );
    compatibility(
        builder,
        "TrayType",
        "idaes.models.costing.SSLW",
        &["Sieve", "Valve", "BubbleCap"],
    );
    compatibility(
        builder,
        "TrayMaterial",
        "idaes.models.costing.SSLW",
        &[
            "CarbonSteel",
            "StainlessSteel303",
            "StainlessSteel316",
            "Carpenter20CB3",
            "Monel",
        ],
    );
    compatibility(
        builder,
        "HeaterMaterial",
        "idaes.models.costing.SSLW",
        &["CarbonSteel", "CrMoSteel", "StainlessSteel"],
    );
    compatibility(
        builder,
        "HeaterSource",
        "idaes.models.costing.SSLW",
        &[
            "Fuel",
            "Reformer",
            "Pyrolysis",
            "HotWater",
            "Salts",
            "DowthermA",
            "steamBoiler",
        ],
    );
    compatibility(
        builder,
        "CompressorType",
        "idaes.models.costing.SSLW",
        &["Centrifugal", "Reciprocating", "Screw"],
    );
    compatibility(
        builder,
        "CompressorDriveType",
        "idaes.models.costing.SSLW",
        &["ElectricMotor", "SteamTurbine", "gasTurbine"],
    );
    compatibility(
        builder,
        "CompressorMaterial",
        "idaes.models.costing.SSLW",
        &["CarbonSteel", "StainlessSteel", "NickelAlloy"],
    );
    compatibility(
        builder,
        "PumpMaterial",
        "idaes.models.costing.SSLW",
        &[
            "CastIron",
            "DuctileIron",
            "CastSteel",
            "Bronze",
            "StainlessSteel",
            "HastelloyC",
            "Monel",
            "Nickel",
            "Titanium",
            "NiAlBronze",
            "CarbonSteel",
        ],
    );
    compatibility(
        builder,
        "PumpType",
        "idaes.models.costing.SSLW",
        &["Centrifugal", "ExternalGear", "Reciprocating"],
    );
    compatibility(
        builder,
        "PumpMotorType",
        "idaes.models.costing.SSLW",
        &["Open", "Enclosed", "ExplosionProof"],
    );
    compatibility(
        builder,
        "FanType",
        "idaes.models.costing.SSLW",
        &[
            "CentrifugalBackward",
            "CentrifugalStraight",
            "VaneAxial",
            "TubeAxial",
        ],
    );
    compatibility(
        builder,
        "FanMaterial",
        "idaes.models.costing.SSLW",
        &["CarbonSteel", "Fiberglass", "StainlessSteel", "NickelAlloy"],
    );
    compatibility(
        builder,
        "BlowerType",
        "idaes.models.costing.SSLW",
        &["Centrifugal", "Rotary"],
    );
    compatibility(
        builder,
        "BlowerMaterial",
        "idaes.models.costing.SSLW",
        &[
            "CarbonSteel",
            "Aluminum",
            "Fiberglass",
            "StainlessSteel",
            "NickelAlloy",
        ],
    );
    compatibility(
        builder,
        "StateIndex",
        "idaes.models.properties.modular_properties.base.utility",
        &["true", "apparent"],
    );
    compatibility(
        builder,
        "CubicType",
        "idaes.models.properties.modular_properties.eos.ceos_common",
        &["PR", "SRK"],
    );
    compatibility(
        builder,
        "FlashType",
        "idaes.models.unit_models.feed_flash",
        &["isothermal", "isenthalpic"],
    );
    compatibility(
        builder,
        "HeatExchangerFlowPattern",
        "idaes.models.unit_models.heat_exchanger",
        &["countercurrent", "cocurrent", "crossflow"],
    );
    compatibility(
        builder,
        "MixingType",
        "idaes.models.unit_models.mixer",
        &["none", "extensive"],
    );
    compatibility(
        builder,
        "MomentumMixingType",
        "idaes.models.unit_models.mixer",
        &["none", "minimize", "equality", "minimize_and_equality"],
    );
    compatibility(
        builder,
        "ThermodynamicAssumption",
        "idaes.models.unit_models.pressure_changer",
        &["isothermal", "isentropic", "pump", "adiabatic"],
    );
    compatibility(
        builder,
        "SplittingType",
        "idaes.models.unit_models.separator",
        &[
            "totalFlow",
            "phaseFlow",
            "componentFlow",
            "phaseComponentFlow",
        ],
    );
    compatibility(
        builder,
        "EnergySplittingType",
        "idaes.models.unit_models.separator",
        &[
            "none",
            "equal_temperature",
            "equal_molar_enthalpy",
            "enthalpy_split",
        ],
    );
    compatibility(
        builder,
        "ValveFunctionType",
        "idaes.models.unit_models.valve",
        &["linear", "quick_opening", "equal_percentage"],
    );
    compatibility(
        builder,
        "ComponentType",
        "idaes.core.base.components",
        &pse_material::enums::ComponentType::ALL
            .iter()
            .map(|value| value.as_str())
            .collect::<Vec<_>>(),
    );
    compatibility(
        builder,
        "PhaseType",
        "idaes.core.base.phases",
        &pse_material::enums::PhaseType::ALL
            .iter()
            .map(|value| value.as_str())
            .collect::<Vec<_>>(),
    );
    compatibility(
        builder,
        "HenryType",
        "idaes.models.properties.modular_properties.phase_equil.henry",
        &pse_material::enums::HenryType::ALL
            .iter()
            .map(|value| value.as_str())
            .collect::<Vec<_>>(),
    );
    compatibility(
        builder,
        "ConcentrationForm",
        "idaes.models.properties.modular_properties.base.utility",
        &pse_material::enums::ConcentrationForm::ALL
            .iter()
            .map(|value| value.as_str())
            .collect::<Vec<_>>(),
    );
    builder.declare_enum(EnumDecl::idaes(
        "DiscretizationScheme",
        "pyomo.dae",
        vec![
            EnumMember::idaes("BACKWARD", "BACKWARD", "Backward finite difference"),
            EnumMember::idaes("FORWARD", "FORWARD", "Forward finite difference"),
            EnumMember::idaes("CENTRAL", "CENTRAL", "Central finite difference"),
            EnumMember::idaes("LAGRANGE_RADAU", "LAGRANGE-RADAU", "Radau collocation"),
            EnumMember::idaes(
                "LAGRANGE_LEGENDRE",
                "LAGRANGE-LEGENDRE",
                "Legendre collocation",
            ),
        ],
    ));
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
