// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Current source fixtures shared by native engineering tests and cold inspection.
#![allow(
    clippy::unwrap_used,
    reason = "test fixture construction and exact independent value assertions"
)]

use pse_authoring::{
    ParseBudget,
    document::{DocumentBundle, load_package, load_package_texts},
};
use pse_schema::Registry;
use std::{collections::BTreeMap, path::Path};

pub(crate) const FTPX: &str = "bb3ea14061b24335a0c78a25c67e67b6";
pub(crate) const FCTP: &str = "c207b61b96124774b194cd76024f772f";
const NITROGEN: &str = "6aa4a49e750d46ea8da0da40f164dd78";
pub(crate) const HEATER: &str = "242af871dd464fcb82edcad1991b0f13";
pub(crate) const MIXER: &str = "26a0c885415b4cda8f3628f5fc75030b";

pub(crate) fn sources(
    root: &Path,
    registry: &Registry,
    unit: &str,
    state: &str,
) -> Vec<DocumentBundle> {
    let mut sources = Vec::new();
    for name in [
        "elements",
        "physical",
        "methods",
        "states",
        "units",
        "thermo-examples",
    ] {
        let original = load_package(
            &root.join("packages/reference").join(name),
            registry,
            ParseBudget::default(),
        )
        .unwrap();
        if name == "thermo-examples" && state == FCTP {
            // Change the authored state-method selection before parsing/admission.
            let mut texts = original
                .documents
                .iter()
                .map(|document| (document.path.clone(), document.text.clone()))
                .collect::<BTreeMap<_, _>>();
            let packages = texts.get_mut("properties/packages.yaml").unwrap();
            assert!(packages.contains(FTPX));
            *packages = packages.replace(FTPX, FCTP);
            sources.push(load_package_texts(texts, registry, ParseBudget::default()).unwrap());
        } else {
            sources.push(original);
        }
    }
    let inlet_assignment = if unit == MIXER {
        "\n  - name: inlet_domain\n    value: 'eaeaeaeaeaeaeaeaeaeaeaeaeaeaeaea'"
    } else {
        ""
    };
    let mut texts = BTreeMap::from([
        (
            "package.toml".to_owned(),
            "[package]\npackage_id = 'ecececececececececececececececec'\nname = 'native_engineering'\nversion = '1.0.0'\nkind = 'model'\nid_policy = 'explicit'\ndependencies = [\n{package_id = 'df905102f26b4e32a8978ee25184962f', version_req = '=1.0.0'},\n{package_id = '7f518f9ba5af4b8cb54bfdfd4e2c805a', version_req = '=1.0.0'}\n]\ndoc = 'Current source engineering workflow.'\n".to_owned(),
        ),
        (
            "instances/unit.yaml".to_owned(),
            format!(
                "instances:\n- id: 'efefefefefefefefefefefefefefefef'\n  template_id: '{unit}'\n  name: equipment\n  property_package_id: '{NITROGEN}'\n  param_values:\n  - name: property_package\n    value: '{NITROGEN}'{inlet_assignment}\n  feature_values: []\n  doc: 'One ideal single-phase nitrogen unit.'\n"
            ),
        ),
    ]);
    if unit == MIXER {
        texts.insert("materials/inlets.yaml".to_owned(), serde_json::json!({
            "domains": [{
                "id": "eaeaeaeaeaeaeaeaeaeaeaeaeaeaeaea",
                "owner_entity_id": "efefefefefefefefefefefefefefefef",
                "kind": "port_set", "continuous": false,
                "unit_id": null, "parent_domain_id": null,
                "doc": "Actual finite mixer inlet domain."
            }],
            "domain_members": [
                {"domain_id": "eaeaeaeaeaeaeaeaeaeaeaeaeaeaeaea", "id": "edededededededededededededededed", "ordinal": 0, "label": "feed_a", "coordinate": null, "ref_entity_id": null},
                {"domain_id": "eaeaeaeaeaeaeaeaeaeaeaeaeaeaeaea", "id": "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee", "ordinal": 1, "label": "feed_b", "coordinate": null, "ref_entity_id": null}
            ]
        }).to_string());
    }
    sources.push(load_package_texts(texts, registry, ParseBudget::default()).unwrap());
    sources
}
