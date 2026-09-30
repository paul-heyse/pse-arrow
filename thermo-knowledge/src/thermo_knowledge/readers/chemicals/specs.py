# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The tab-separated handbook tables of the chemicals payload, one table per file.

Each table keeps its file's own headings as column names (see `tabular` for the declaration
language). The file name is the variant's only statement of origin, so the table name follows the
file name and `_artifact` says which file a row comes from. The files state no units except where
a heading does (the Global Warming Potential tables, the Marcus conductivities, the heat
capacities of solids); every other column is `not stated`, because the units live in the
library's docstrings, not in the data. Columns named `CAS` or `CASRN` are text with hyphens unless
the file writes the number without them (then kind `c`, an integer).
"""

from __future__ import annotations

from thermo_knowledge.readers.chemicals.tabular import Delimited, delimited

CHEMICALS_TABLES: tuple[Delimited, ...] = (
    delimited(
        "critical_psrk_appendix",
        "Critical Properties/Appendix to PSRK Revision 4.tsv",
        "CAS;Chemical;Tc:f;Pc:f;Vc:f;omega:f",
    ),
    delimited(
        "critical_crc_organics",
        "Critical Properties/CRCCriticalOrganics.tsv",
        "CAS;Chemical;Tc:f;Tc_error:f;Pc:f;Pc_error:f;Vc:f;Vc_error:f",
    ),
    delimited(
        "critical_dippr_pina_martines",
        "Critical Properties/DIPPRPinaMartines.tsv",
        "CAS;Tc:f;Pc:f;Vc:f",
    ),
    delimited(
        "critical_iupac_organic",
        "Critical Properties/IUPACOrganicCriticalProps.tsv",
        "CAS;Chemical;MW:f;Tc:f;Pc:f;Vc:f;Zc:f;Reference:i",
    ),
    delimited(
        "critical_mathews_1972_inorganic",
        "Critical Properties/Mathews1972InorganicCriticalProps.tsv",
        "CAS;Chemical;MW:f;Tc:f;Pc:f;Vc:f;Zc:f",
    ),
    delimited(
        "critical_passut_danner_1973",
        "Critical Properties/PassutDanner1973.tsv",
        "CAS;Chemical;Tc:f;Pc:f;omega:f",
    ),
    delimited(
        "critical_yaws_collection",
        "Critical Properties/Yaws Collection.tsv",
        "CAS;Chemical;Tc:f;Pc:f;Vc:f;omega:f",
    ),
    delimited(
        "critical_fedors_vc_predictions",
        "Critical Properties/fedors_Vc_predictions.tsv",
        "CAS:c;Vc:f",
    ),
    delimited(
        "critical_omega_psat_tc_predictions",
        "Critical Properties/omega_Psat_Tc_predictions.tsv",
        "CAS:c;omega:f",
    ),
    delimited(
        "critical_wilson_jasperson_predictions",
        "Critical Properties/wilson_jasperson_Tc_Pc_predictions.tsv",
        "CAS:c;Tc:f;Pc:f",
    ),
    delimited(
        "density_costald",
        "Density/COSTALD Parameters.tsv",
        "CAS;Chemical;omega_SRK:f;Vchar:f;Z_RA:f",
    ),
    delimited(
        "density_crc_molten_inorganics",
        "Density/CRC Inorganics densties of molten compounds and salts.tsv",
        "CAS;Chemical;MW:f;rho:f;k:f;Tm:f;Tmax:f",
    ),
    delimited(
        "density_crc_liquid_inorganic_constant",
        "Density/CRC Liquid Inorganic Constant Densities.tsv",
        "CAS;Chemical;Vm:f",
    ),
    delimited(
        "density_crc_solid_inorganic_constant",
        "Density/CRC Solid Inorganic Constant Densities.tsv",
        "CAS;Chemical;Vm:f",
    ),
    delimited(
        "density_crc_virial_polynomials",
        "Density/CRC Virial polynomials.tsv",
        "CAS;Chemical;a1:f;a2:f;a3:f;a4:f;a5:f",
    ),
    delimited(
        "density_mchaweh_sn0_deltas",
        "Density/Mchaweh SN0 deltas.tsv",
        "CAS;Chemical;delta_SRK:f",
    ),
    delimited(
        "density_perry_105",
        "Density/Perry Parameters 105.tsv",
        "CAS;Chemical;C1:f;C2:f;C3:f;C4:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "density_vdi_ppds_saturated_liquids",
        "Density/VDI PPDS Density of Saturated Liquids.tsv",
        "CAS;Chemical;MW:f;Tc:f;rhoc:f;A:f;B:f;C:f;D:f",
    ),
    delimited(
        "electrolytes_crc_aqueous_ions",
        "Electrolytes/CRC Thermodynamic Properties of Aqueous Ions.tsv",
        "CAS;Formula;Name;MW:f;Hf(aq):f;Gf(aq):f;S(aq):f;Cp(aq):f",
    ),
    delimited(
        "electrolytes_crc_conductivity_infinite_dilution",
        "Electrolytes/CRC conductivity infinite dilution.tsv",
        "CAS;Formula;lambda:f",
    ),
    delimited(
        "electrolytes_dissociations",
        "Electrolytes/Electrolyte dissociations.tsv",
        "Electrolyte name;Electrolyte CAS;Electrolyte Formula;Anion formula;Anion CAS;Anion charge:i;Anion count:i;Cation formula;Cation CAS;Cation charge:i;Cation count:i",
    ),
    delimited(
        "electrolytes_laliberte_2009",
        "Electrolytes/Laliberte2009.tsv",
        "Name;CASRN;Formula;MW:f;c0:f;c1:f;c2:f;c3:f;c4:f;Min T:f;Max T:f;Max w:f;No of points in corr:i;v1:f;v2:f;v3:f;v4:f;v5:f;v6:f;Min T:f;Max T:f;Max w:f;No of points in corr:i;a1:f;a2:f;a3:f;a4:f;a5:f;a6:f;Min T:f;Max T:f;Max w:f;No of points in corr:i",
    ),
    delimited(
        "electrolytes_lange_pure_species_conductivity",
        "Electrolytes/Lange Pure Species Conductivity.tsv",
        "CAS;Chemical;T:f;Conductivity:f",
    ),
    delimited(
        "electrolytes_magomedov_thermal_conductivity",
        "Electrolytes/Magomedov Thermal Conductivity.tsv",
        "CASRN;Formula;Chemical;Ai:f",
    ),
    delimited(
        "electrolytes_marcus_ion_conductivities",
        "Electrolytes/Marcus Ion Conductivities.tsv",
        "CASRN;Formula;Charge:i;Conductivity, cm^2 S^-1 mol^-1|Conductivity:f:cm^2 S^-1 mol^-1",
    ),
    delimited(
        "electrolytes_mccleskey_electrical_conductivity",
        "Electrolytes/McCleskey Electrical Conductivity.tsv",
        "formula;CASRN;c1:f;c2:f;c3:f;d1:f;d2:f;d3:f;B:f;multiplier:i",
    ),
    delimited(
        "electrolytes_permittivity_of_liquids",
        "Electrolytes/Permittivity (Dielectric Constant) of Liquids.tsv",
        "CAS;Chemical;T:f;Permittivity:f;A:f;B:f;C:f;D:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "environment_crc_logp",
        "Environment/CRC logP table.tsv",
        "CAS;Name ;logP:f",
    ),
    delimited(
        "environment_gwp_2007",
        "Environment/Official Global Warming Potentials 2007.tsv",
        "CAS;Name;Formula;Lifetime, years|Lifetime:f:years;Radiative efficiency, W/m^2/ppb|Radiative_efficiency:f:W/m^2/ppb;SAR 100yr:f;20yr GWP:f;100yr GWP:f;500yr GWP:f",
    ),
    delimited(
        "environment_gwp_2014",
        "Environment/Official Global Warming Potentials 2014.tsv",
        "CAS;Name;Formula;Lifetime, years|Lifetime:f:years;Radiative efficiency, W/m^2/ppb|Radiative_efficiency:f:W/m^2/ppb;20yr GWP:f;100yr GWP:f;20yr GTP:f;50yr GTP:f;100yr GTP:f;20yr AGWP:f;100yr AGWP:f;20yr AGTP:f;50yr AGTP:f;100yr AGTP:f",
        quoted=True,
    ),
    delimited(
        "environment_gwp_2021",
        "Environment/Official Global Warming Potentials 2021.tsv",
        "Name;CAS;Acronym;Formula;Lifetime, years|Lifetime:f:years;Radiative efficiency, W/m^2/ppb|Radiative_efficiency:f:W/m^2/ppb;20yr AGWP:f;20yr GWP:f;100yr AGWP:f;100yr GWP:f;500yr AGWP:f;500yr GWP:f;50yr AGTP:f;50yr GTP:f;100yr AGTP:f;100yr GTP:f;50yr CGTP:f;100yr CGTP:f",
    ),
    delimited(
        "environment_ozone_depletion_potentials",
        "Environment/Ozone Depletion Potentials.tsv",
        "CAS;ODP2 Max:f;ODP1 Max:f;ODP1;Name;ODP2;Lifetime:f;ODP2 Min:f;ODP1 Min:f;ODP2 Design:f;ODP1 Design:f",
    ),
    delimited(
        "environment_syrres_logp",
        "Environment/Syrres logP data.csv.gz",
        "CAS;Chemical;logP:f",
    ),
    delimited(
        "heat_capacity_crc_solids",
        "Heat Capacity/CRCHeatCapacitySolids.tsv",
        "CASRN;NAME/T(K) J/mol/K|Name;200|Cp_200:f:J/mol/K;250|Cp_250:f:J/mol/K;300|Cp_300:f:J/mol/K;"
        "350|Cp_350:f:J/mol/K;400|Cp_400:f:J/mol/K;500|Cp_500:f:J/mol/K;600|Cp_600:f:J/mol/K",
    ),
    delimited(
        "heat_capacity_crc_standard_properties",
        "Heat Capacity/CRC Standard Thermodynamic Properties of Chemical Substances.tsv",
        "CAS;Chemical;Hfs:f;Gfs:f;S0s:f;Cps:f;Hfl:f;Gfl:f;S0l:f;Cpl:f;Hfg:f;Gfg:f;S0g:f;Cpg:f",
    ),
    delimited(
        "heat_capacity_perry_2_153_dippr_100",
        "Heat Capacity/Perry_Table_2-153_DIPPR_100.tsv",
        "CAS;Chemical;A:f;B:f;C:f;D:f;E:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "heat_capacity_perry_2_153_dippr_114",
        "Heat Capacity/Perry_Table_2-153_DIPPR_114.tsv",
        "CAS;Name;A:f;B:f;C:f;D:f;E:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "heat_capacity_perry_2_151_2",
        "Heat Capacity/Perrys Table 2-151-2.tsv",
        "Ful CAS;Formula;main phase;subphase;Constant Term:f;+*T term|plus_T_term:f;/T^2 term|over_T2_term:f;*T^2 term|times_T2_term:f;Tmin:f;Tmax:f;%error|percent_error",
    ),
    delimited(
        "heat_capacity_perry_2_151",
        "Heat Capacity/Perrys Table 2-151.tsv",
        "Ful CAS;Formula;main phase;subphase;Constant Term:f;+*T term|plus_T_term:f;/T^2 term|over_T2_term:f;*T^2 term|times_T2_term:f;Tmin:f;Tmax:f;%error|percent_error",
    ),
    delimited(
        "heat_capacity_poling_databank",
        "Heat Capacity/PolingDatabank.tsv",
        "CAS;Chemical;Tmin:f;Tmax:f;a0:f;a1:f;a2:f;a3:f;a4:f;Cpg:f;Cpl:f",
    ),
    delimited(
        "heat_capacity_trc_organic_gas",
        "Heat Capacity/TRC Thermodynamics of Organic Compounds in the Gas State.tsv",
        "CAS;Chemical;Tmin:f;Tmax:f;a0:f;a1:f;a2:f;a3:f;a4:f;a5:f;a6:f;a7:f;I:f;J:f;Hfg:f",
    ),
    delimited(
        "heat_capacity_zabransky",
        "Heat Capacity/Zabransky.tsv",
        "CASRN;Name;Data Type;Uncertainty;Tmin:f;Tmax:f;A1-spline:f;A2-spline:f;A3-spline:f;A4-spline:f;A1-quasi:f;A2-quasi:f;A3-quasi:f;A4-quasi:f;A5-quasi:f;A6-quasi:f;Tc:f",
    ),
    delimited(
        "interface_jasper_lange",
        "Interface/Jasper-Lange.tsv",
        "CAS;Name;a:f;b:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "interface_mulero_cachadina",
        "Interface/MuleroCachadinaParameters.tsv",
        "CAS;Fluid;sigma0:f;n0:f;sigma1:f;n1:f;sigma2:f;n2:f;Tc:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "interface_somayajulu",
        "Interface/Somayajulu.tsv",
        "CAS;Chemical;Tt:f;Tc:f;A:f;B:f;C:f",
    ),
    delimited(
        "interface_somayajulu_revised",
        "Interface/SomayajuluRevised.tsv",
        "CAS;Chemical;Tt:f;Tc:f;A:f;B:f;C:f",
    ),
    delimited(
        "interface_vdi_ppds_surface_tensions",
        "Interface/VDI PPDS surface tensions.tsv",
        "CAS;Chemical;Tm:f;Tc:f;A:f;B:f;C:f;D:f;E:f",
    ),
    delimited(
        "misc_crc_organic_refractive_index",
        "Misc/CRC Handbook Organic RI.csv",
        "CAS;RI:f;RIT:f",
    ),
    delimited(
        "misc_dahmen_marquardt_ignition_delay",
        "Misc/Dahmen_Marquardt_ignition_delay_IQT.tsv",
        "CAS;IGNITION_DELAY:f",
    ),
    delimited(
        "misc_florian_liming_cetane_experimental",
        "Misc/Florian_Liming_CETANE_experimental.tsv",
        "CAS;CETANE:f",
    ),
    delimited(
        "misc_florian_liming_mon_experimental",
        "Misc/Florian_Liming_MON_experimental.tsv",
        "CAS;MON:f",
    ),
    delimited(
        "misc_florian_liming_ron_mon_ann",
        "Misc/Florian_Liming_RON_MON_ANN.tsv",
        "CAS;RON:f;MON:f",
    ),
    delimited(
        "misc_florian_liming_ron_experimental",
        "Misc/Florian_Liming_RON_experimental.tsv",
        "CAS;RON:f",
    ),
    delimited(
        "misc_muller_dipoles",
        "Misc/Muller Supporting Info Dipoles.csv",
        "CAS;Chemical;dipole_moment:f",
    ),
    delimited(
        "misc_physical_constants_inorganic",
        "Misc/Physical Constants of Inorganic Compounds.csv",
        "CAS;Chemical;Tm:f;Tb:f;rho:f",
    ),
    delimited(
        "misc_physical_constants_organic",
        "Misc/Physical Constants of Organic Compounds.csv",
        "CAS;Name;Tm:f;Tb:f;rho:f;RI:f",
    ),
    delimited(
        "misc_poling_dipole",
        "Misc/Poling Dipole.csv",
        "CAS;Chemical;dipole_moment:f",
    ),
    delimited(
        "misc_combustdb_cetane",
        "Misc/Travis_Kessler_Combustdb_CETANE.tsv",
        "CAS;CETANE:f",
    ),
    delimited(
        "misc_combustdb_mon",
        "Misc/Travis_Kessler_Combustdb_MON.tsv",
        "CAS;MON:f",
    ),
    delimited(
        "misc_combustdb_ron",
        "Misc/Travis_Kessler_Combustdb_RON.tsv",
        "CAS;RON:f",
    ),
    delimited(
        "misc_combustdb_predictions",
        "Misc/Travis_Kessler_Combustdb_predictions.tsv",
        "CAS;CETANE:f;MON:f;RON:f;POUR_POINT:f;CLOUD_POINT:f",
    ),
    delimited(
        "misc_hansen_alshehri",
        "Misc/alshehri_hansen_solubility_parameters.tsv",
        "CAS;HANSEN_DELTA_D:f;HANSEN_DELTA_P:f;HANSEN_DELTA_H:f",
    ),
    delimited(
        "misc_cccbdb_dipoles",
        "Misc/cccbdb.nist.gov Dipoles.csv",
        "CAS;Chemical;dipole_moment:f",
    ),
    delimited(
        "misc_chemsep_radius_of_gyrations",
        "Misc/chemsep_radius_of_gyrations.tsv",
        "CAS;RG:f",
    ),
    delimited(
        "misc_common_chemistry",
        "Misc/common_chemistry_data.tsv",
        "CAS:c;Tm:f;Tb:f;Vms:f;Vml:f",
    ),
    delimited(
        "misc_heos_constants",
        "Misc/heos_constants.tsv",
        "CAS;ID;name;Tc:f;Pc:f;Vc:f;Tt:f;Pt:f;Tb:f;omega:f",
    ),
    delimited(
        "misc_hansen_hspipy",
        "Misc/hspipy_hansen_solubility_parameters.tsv",
        "CAS;HANSEN_DELTA_D:f;HANSEN_DELTA_P:f;HANSEN_DELTA_H:f",
    ),
    delimited(
        "misc_joback_predictions",
        "Misc/joback_predictions.tsv",
        "CAS:c;Tm:f;Hfus:f;Hvap:f;Tb:f;Tc:f;Pc:f;Vc:f;Hfg:f;Cpg0:f;Cpg1:f;Cpg2:f;Cpg3:f;mul0:f;mul1:f",
    ),
    delimited(
        "misc_psi4_dipoles",
        "Misc/psi4_dipoles.tsv",
        "CAS;dipole_moment:f",
    ),
    delimited(
        "misc_psi4_linear",
        "Misc/psi4_linear.tsv",
        "CAS;linear:i",
    ),
    delimited(
        "misc_psi4_radius_of_gyrations",
        "Misc/psi4_radius_of_gyrations.tsv",
        "CAS;RG:f",
    ),
    delimited(
        "misc_hansen_ruben_manuel",
        "Misc/ruben_manuel_hansen_solubility_parameters.tsv",
        "CAS;HANSEN_DELTA_D:f;HANSEN_DELTA_P:f;HANSEN_DELTA_H:f",
    ),
    delimited(
        "misc_hansen_schrier",
        "Misc/schrier_hansen_solubility_parameters.tsv",
        "CAS;HANSEN_DELTA_D:f;HANSEN_DELTA_P:f;HANSEN_DELTA_H:f",
    ),
    delimited(
        "misc_webbook_constants",
        "Misc/webbook_constants.tsv",
        "CAS:c;Tt:f;Tm:f;Tb:f;Tc:f;Pt:f;Pc:f;Vc:f;Hfs:f;Hfl:f;Hfg:f;S0s:f;S0l:f;S0g:f;Hsub:f;Hfus:f;Hvap:f;AntoineTmin:f;AntoineTmax:f;AntoineA:f;AntoineB:f;AntoineC:f",
    ),
    delimited(
        "misc_wikidata_properties",
        "Misc/wikidata_properties.tsv",
        "CAS:c;Tm:f;Tb:f;Hfus:f;Hvap:f;Hf:f;S0:f;LFL:f;UFL:f;T_flash:f;T_autoignition:f;RI:f;RIT;logP:f",
    ),
    delimited(
        "phase_change_alibakhshi_hvap",
        "Phase Change/Alibakhshi one-coefficient enthalpy of vaporization.tsv",
        "CAS;Chemical;C:f",
    ),
    delimited(
        "phase_change_crc_heat_of_fusion",
        "Phase Change/CRC Handbook Heat of Fusion.tsv",
        "CAS;Chemical;Formula;Hfus:f",
    ),
    delimited(
        "phase_change_crc_heat_of_vaporization",
        "Phase Change/CRC Handbook Heat of Vaporization.tsv",
        "CAS;Chemical;Formula;Tb:f;HvapTb:f;Hvap298:f",
    ),
    delimited(
        "phase_change_ghazerati_sublimation",
        "Phase Change/Ghazerati Appendix Sublimation Enthalpy.tsv",
        "CAS;Chemical;Hsub:f;error:f",
    ),
    delimited(
        "phase_change_ghazerati_vaporization",
        "Phase Change/Ghazerati Appendix Vaporization Enthalpy.tsv",
        "CAS;Chemical;Hvap298:f",
    ),
    delimited(
        "phase_change_opennotebook_melting_points",
        "Phase Change/OpenNotebook Melting Points.tsv",
        "CAS;Tm:f",
    ),
    delimited(
        "phase_change_perry_2_150",
        "Phase Change/Table 2-150 Heats of Vaporization of Inorganic and Organic Liquids.tsv",
        "CAS;Chemical;Tc:f;C1:f;C2:f;C3:f;C4:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "phase_change_vdi_ppds_hvap",
        "Phase Change/VDI PPDS Enthalpies of vaporization.tsv",
        "CAS;Chemical;MW:f;Tc:f;A:f;B:f;C:f;D:f;E:f",
    ),
    delimited(
        "phase_change_yaws_boiling_points",
        "Phase Change/Yaws Boiling Points.tsv",
        "CAS;Tb:f",
    ),
    delimited(
        "reactions_api_tdb_albahri_hf_gas",
        "Reactions/API TDB Albahri Hf (g).tsv",
        "CAS;Compound;Hfg:f",
    ),
    delimited(
        "reactions_atct_1_112_gas",
        "Reactions/ATcT 1.112 (g).tsv",
        "CAS;Chemical;Formula;Hfg_0K:f;Hfg:f;uncertainty:f",
    ),
    delimited(
        "reactions_atct_1_112_liquid",
        "Reactions/ATcT 1.112 (l).tsv",
        "CAS;Chemical;Formula;Hfl_0K:f;Hfl:f;uncertainty:f",
    ),
    delimited(
        "reactions_janaf_1998",
        "Reactions/JANAF_1998.tsv",
        "CAS;Chemical;formula;Hfl:f;Gfl:f;S0l:f;Cpl:f;Hfg:f;Gfg:f;S0g:f;Cpg:f",
    ),
    delimited(
        "reactions_yaws_hf_s0_gas",
        "Reactions/Yaws Hf S0 (g).tsv",
        "CAS;name;Hfg:f;S0g:f",
    ),
    delimited(
        "safety_dippr_t_flash_serat",
        "Safety/DIPPR T_flash Serat.csv",
        "CAS;Name;T_flash:f",
    ),
    delimited(
        "safety_iarc_carcinogens",
        "Safety/IARC Carcinogen Database.tsv",
        "CAS;description;group:i;volumes;year",
    ),
    delimited(
        "safety_iec_60079_20_1_2010",
        "Safety/IS IEC 60079-20-1 2010.tsv",
        "CAS;Names;T_flash:f;T_autoignition:f;LFL:f;UFL:f",
    ),
    delimited(
        "safety_nfpa_497_2008",
        "Safety/NFPA 497 2008.tsv",
        "CAS;Name;T_flash:f;T_autoignition:f;LFL:f;UFL:f",
    ),
    delimited(
        "safety_ntp_carcinogens",
        "Safety/National Toxicology Program Carcinogens.tsv",
        "CAS;Chemical;Listing:i",
    ),
    delimited(
        "safety_ontario_exposure_limits_tsv",
        "Safety/Ontario Exposure Limits.tsv",
        "CASRN;Agent [CAS No.];Time-Weighted Average Limit (TWA);Short-Term Exposure Limit (STEL);Ceiling Limit (C);Notations;MW",
    ),
    delimited(
        "thermal_conductivity_perry_2_314_vapor",
        "Thermal Conductivity/Table 2-314 Vapor Thermal Conductivity of Inorganic and Organic Substances.tsv",
        "CAS;Chemical;C1:f;C2:f;C3:f;C4:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "thermal_conductivity_perry_2_315_liquid",
        "Thermal Conductivity/Table 2-315 Thermal Conductivity of Inorganic and Organic Liquids.tsv",
        "CAS;Chemical;C1:f;C2:f;C3:f;C4:f;C5:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "thermal_conductivity_vdi_ppds_gases",
        "Thermal Conductivity/VDI PPDS Thermal conductivity of gases.tsv",
        "CAS;Chemical;A:f;B:f;C:f;D:f;E:f",
    ),
    delimited(
        "thermal_conductivity_vdi_ppds_saturated_liquids",
        "Thermal Conductivity/VDI PPDS Thermal conductivity of saturated liquids.tsv",
        "CAS;Chemical;A:f;B:f;C:f;D:f;E:f",
    ),
    delimited(
        "triple_staveley_1981",
        "Triple Properties/Staveley 1981.tsv",
        "CAS;Chemical;Formula ;Tt:f;Pt:f;Pt_uncertainty:f",
    ),
    delimited(
        "vapor_pressure_alcock_itkin_horrigan_metals",
        "Vapor Pressure/Alcock_Itkin_Horrigan_metalic_elements.tsv",
        "CAS;name;A:f;B:f;C:f;D:f;E:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "vapor_pressure_alcock_itkin_horrigan_metals_sublimation",
        "Vapor Pressure/Alcock_Itkin_Horrigan_metalic_elements_sublimation.tsv",
        "Name;CAS;A:f;B:f;C:f;D:f;E:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "vapor_pressure_antoine_poling",
        "Vapor Pressure/Antoine Collection Poling.tsv",
        "CAS;Chemical;A:f;B:f;C:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "vapor_pressure_antoine_extended_poling",
        "Vapor Pressure/Antoine Extended Collection Poling.tsv",
        "CAS;Chemical;A:f;B:f;C:f;Tc:f;to:f;n:f;E:f;F:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "vapor_pressure_landolt_antoine_v20",
        "Vapor Pressure/Landolt_antoine_V20.tsv",
        "CAS;Name;A:f;B:f;C:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "vapor_pressure_landolt_antoine_sublimation_v20",
        "Vapor Pressure/Landolt_antoine_sublimation_V20.tsv",
        "CAS;Name;A:f;B:f;C:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "vapor_pressure_perry_2_8",
        "Vapor Pressure/Table 2-8 Vapor Pressure of Inorganic and Organic Liquids.tsv",
        "CAS;Chemical;C1:f;C2:f;C3:f;C4:f;C5:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "vapor_pressure_vdi_ppds_boiling_temperatures",
        "Vapor Pressure/VDI PPDS Boiling temperatures at different pressures.tsv",
        "CAS;Chemical;Tm:f;Tc:f;Pc:f;A:f;B:f;C:f;D:f",
    ),
    delimited(
        "vapor_pressure_wagner_poling",
        "Vapor Pressure/Wagner Collection Poling.tsv",
        "CAS;Name;A:f;B:f;C:f;D:f;Tc:f;Pc:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "vapor_pressure_wagner_mcgarry",
        "Vapor Pressure/Wagner Original McGarry.tsv",
        "CAS;Name;A:f;B:f;C:f;D:f;Pc:f;Tc:f;Tmin:f",
    ),
    delimited(
        "viscosity_dutt_prasad_3_term",
        "Viscosity/Dutt Prasad 3 term.tsv",
        "CAS;Chemical;A:f;B:f;C:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "viscosity_magalhaes_lennard_jones",
        "Viscosity/MagalhaesLJ.tsv",
        "CAS;Chemical;molecular_diameter:f;Stockmayer:f",
    ),
    delimited(
        "viscosity_poling_lennard_jones",
        "Viscosity/PolingLJ.tsv",
        "CAS;Formula;Name;molecular_diameter:f;Stockmayer:f",
    ),
    delimited(
        "viscosity_perry_2_312_vapor",
        "Viscosity/Table 2-312 Vapor Viscosity of Inorganic and Organic Substances.tsv",
        "CAS;Chemical;C1:f;C2:f;C3:f;C4:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "viscosity_perry_2_313_liquid",
        "Viscosity/Table 2-313 Viscosity of Inorganic and Organic Liquids.tsv",
        "CAS;Chemical;C1:f;C2:f;C3:f;C4:f;C5:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "viscosity_vdi_ppds_gases",
        "Viscosity/VDI PPDS Dynamic viscosity of gases polynomials.tsv",
        "CAS;Chemical;A:f;B:f;C:f;D:f;E:f",
    ),
    delimited(
        "viscosity_vdi_ppds_saturated_liquids",
        "Viscosity/VDI PPDS Dynamic viscosity of saturated liquids polynomials.tsv",
        "CAS;Chemical;Formula;A:f;B:f;C:f;D:f;E:f",
    ),
    delimited(
        "viscosity_viswanath_natarajan_2_term_exponential",
        "Viscosity/Viswanath Natarajan Dynamic 2 term Exponential.tsv",
        "CAS;Substance;Formula;C:f;D:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "viscosity_viswanath_natarajan_2_term",
        "Viscosity/Viswanath Natarajan Dynamic 2 term.tsv",
        "CAS;Name;Formula;A:f;B:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "viscosity_viswanath_natarajan_3_term",
        "Viscosity/Viswanath Natarajan Dynamic 3 term.tsv",
        "CAS;Name;Formula;A:f;B:f;C:f;Tmin:f;Tmax:f",
    ),
    delimited(
        "misc_element_data",
        "Misc/Element data.csv",
        "name;CAS;atomic number:i;element symbol;covalent radius:f;rB0:f;Van derWaal radius:f;"
        "atomic mass:f;number of outshell electrons:i;most common isotope:i;"
        "most common isotopic mass:f;valences:i;:i;:i",
    ),
)
"""Every delimited table that only the chemicals payload holds."""
