// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    reason = "test fixture construction and exact independent value assertions"
)]
//! Ordinary package sources with indexed material contributions and exact stock law contracts.
#[path = "conservation_source/declarations.rs"]
mod declarations;

use pse_authoring::{
    ParseBudget,
    document::{DocumentBundle, load_package, load_package_texts},
};
use pse_ids::SemanticId;
use pse_schema::Registry;
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf};

pub(crate) fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
#[derive(Clone, Copy)]
pub(crate) enum Balance {
    ComponentTotal,
    ComponentPhase,
    ElementMolar,
    ElementMass,
    Energy,
    EnergyBroadcast,
    Pressure,
}
impl Balance {
    pub(crate) fn member(self) -> &'static str {
        match self {
            Self::ComponentTotal => "componentTotal",
            Self::ComponentPhase => "componentPhase",
            Self::ElementMolar | Self::ElementMass => "elementTotal",
            Self::Energy | Self::EnergyBroadcast => "enthalpyTotal",
            Self::Pressure => "pressureTotal",
        }
    }
    pub(crate) fn source_type(self) -> &'static str {
        match self {
            Self::ComponentTotal | Self::ComponentPhase | Self::ElementMolar => {
                "d5da9b09f9de4279b55641b93ddb196d"
            }
            Self::ElementMass => "0a61743827224c0388e3d42f1da2eb5d",
            Self::Energy => "b5065a6e90254578b286947f2b1423c6",
            Self::EnergyBroadcast => "e1f2106da9eb4fe0aa2749fa5469fa1a",
            Self::Pressure => "a0fa145fd8b2ecec448c39666960debc",
        }
    }
    pub(crate) fn result_type(self) -> &'static str {
        match self {
            Self::ComponentTotal => "08f85f7bd5164fc79f815eaf7e78e748",
            Self::ComponentPhase | Self::Pressure => self.source_type(),
            Self::ElementMolar | Self::ElementMass => "c2a3ac7b619c45239bae6808fa7b9fa2",
            Self::Energy => "e1f2106da9eb4fe0aa2749fa5469fa1a",
            Self::EnergyBroadcast => "b5065a6e90254578b286947f2b1423c6",
        }
    }
    pub(crate) fn source_axes(self) -> Vec<&'static str> {
        match self {
            Self::Energy => vec!["p"],
            Self::Pressure | Self::EnergyBroadcast => vec![],
            _ => vec!["p", "j"],
        }
    }
    pub(crate) fn result_axes(self) -> Vec<&'static str> {
        match self {
            Self::ComponentTotal => vec!["j"],
            Self::ComponentPhase => vec!["p", "j"],
            Self::ElementMolar | Self::ElementMass => vec!["e"],
            Self::Energy | Self::Pressure => vec![],
            Self::EnergyBroadcast => vec!["p"],
        }
    }
    fn family(self) -> &'static str {
        match self {
            Self::Energy | Self::EnergyBroadcast => "energy",
            Self::Pressure => "momentum",
            _ => "material",
        }
    }
    fn subject(self) -> &'static str {
        match self {
            Self::Energy | Self::EnergyBroadcast => "energy",
            Self::Pressure => "momentum",
            _ => "phase_species",
        }
    }
}
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent orthogonal fixture mutations for conservation requirements"
)]
pub(crate) struct Options {
    pub(crate) balance: Balance,
    pub(crate) hydrogen_count: f64,
    pub(crate) molecular_weight: bool,
    pub(crate) unsupported: bool,
    pub(crate) fixed_phase: bool,
    pub(crate) fixed_subject: bool,
}
impl Options {
    pub(crate) fn new(balance: Balance) -> Self {
        Self {
            balance,
            hydrogen_count: 2.0,
            molecular_weight: true,
            unsupported: false,
            fixed_phase: false,
            fixed_subject: false,
        }
    }
    fn result_axes(&self) -> Vec<&'static str> {
        self.balance
            .result_axes()
            .into_iter()
            .filter(|axis| {
                !((self.fixed_phase && *axis == "p") || (self.fixed_subject && *axis == "j"))
            })
            .collect()
    }
    fn result_type(&self) -> &'static str {
        match (self.balance, self.fixed_subject, self.fixed_phase) {
            (Balance::ComponentTotal, true, _) | (Balance::ComponentPhase, true, true) => {
                "8ce2f0977877ae56c3712f710201be89"
            }
            (Balance::ComponentPhase, true, false) => "9899b78472c84b92b127e787b7fbd361",
            (Balance::ComponentPhase, false, true) => "08f85f7bd5164fc79f815eaf7e78e748",
            _ => self.balance.result_type(),
        }
    }
}
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn stock(path: &str) -> Value {
    let text = std::fs::read_to_string(root().join("packages/reference").join(path)).unwrap();
    serde_json::from_str(
        &text
            .lines()
            .filter(|line| !line.starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .unwrap()
}
pub(crate) fn documents(registry: &Registry, options: &Options) -> Vec<DocumentBundle> {
    let mut bundles = ["elements", "physical"]
        .into_iter()
        .map(|package| {
            load_package(
                &root().join("packages/reference").join(package),
                registry,
                ParseBudget::default(),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let laws = stock("units/laws/balances.yaml");
    let binding = laws["law_bindings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["balance_member"] == options.balance.member())
        .unwrap()
        .clone();
    let template = laws["templates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["template_id"] == binding["law_template_id"])
        .unwrap();
    let mut template = template.clone();
    template["package_id"] = json!(id(90));
    let mut texts = BTreeMap::from([(
        "package.toml".to_owned(),
        format!(
            "[package]\npackage_id=\"{}\"\nname=\"conservation.fixture\"\nversion=\"1.0.0\"\nkind=\"model\"\ndoc=\"Independent conservation source oracle.\"\nid_policy=\"explicit\"\ndependencies=[{{package_id=\"b27409be5572b8712e47db271fae28cd\",version_req=\"=1.0.0\"}}]\n",
            id(90)
        ),
    )]);
    texts.insert(
        "templates/control.yaml".into(),
        declarations::template(options.balance).to_string(),
    );
    texts.insert(
        "instances/control.yaml".into(),
        declarations::instance().to_string(),
    );
    texts.insert(
        "materials/system.yaml".into(),
        declarations::materials(options).to_string(),
    );
    texts.insert("methods/state.yaml".into(),json!({"method_specs":[{"method_id":id(82),"family":"state_definition","name":"FixtureState","version":"1","provides":[],"requires":[],"parameter_kinds":[],"realization":{"kind":"equation_template","equation_template":{"template_id":id(60)}},"validity":[],"doc":"No inferred property reads; this method identifies the actual state template."}]}).to_string());
    texts.insert("properties/system.yaml".into(),json!({"property_packages":[{"id":id(95),"name":"system","material_system_id":id(92),"unit_set_id":"e95dc4e82a284f3877f924cb656b1de2","state_definition_method_id":id(82),"temperature_ref":298.15,"pressure_ref":100_000.0,"include_enthalpy_of_formation":false,"bubble_dew_method_id":null,"doc":"Actual selected material inventory."}]}).to_string());
    let contracts = laws["element_projection_contracts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["law_template_id"] == binding["law_template_id"])
        .cloned()
        .collect::<Vec<_>>();
    let axes = options.result_axes();
    let subject_axis = axes.iter().position(|axis| matches!(*axis, "j" | "e"));
    let phase_axis = axes.iter().position(|axis| *axis == "p");
    texts.insert("laws/balance.yaml".into(),json!({
        "templates":[template],"law_bindings":if options.unsupported {vec![]} else {vec![binding.clone()]},"element_projection_contracts":contracts,
        "template_law_instances":[{"template_id":id(60),"law_instance_decl_id":id(80),"law_template_id":binding["law_template_id"],"scope":"self","options":[{"key":"balance_type","value":options.balance.member()}],"guard_id":null}],
        "template_law_contracts":[{"law_instance_decl_id":id(80),"indexed_by":axes,"quantity_type_id":options.result_type(),"balance_enum_id":binding["balance_enum_id"],"coordinates":{"member":if options.fixed_subject {Some(json!({"kind":"fixed","fixed":{"entity_id":id(100)}}))}else{subject_axis.map(|position|json!({"kind":"axis","axis":{"position":position}}))},"phase":if options.fixed_phase {Some(json!({"kind":"fixed","fixed":{"entity_id":id(110)}}))}else{phase_axis.map(|position|json!({"kind":"axis","axis":{"position":position}}))}},"default_balance":null}]
    }).to_string());
    bundles.push(load_package_texts(texts, registry, ParseBudget::default()).unwrap());
    bundles
}
