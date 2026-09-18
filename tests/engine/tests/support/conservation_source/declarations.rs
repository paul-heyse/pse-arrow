// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Authored source declarations; no constructed or intermediate stage rows.
use super::{Balance, Options, Value, id, json};
pub(super) fn template(balance: Balance) -> Value {
    let axes = balance.source_axes();
    let suffix = if axes.is_empty() {
        String::new()
    } else {
        format!("[{}]", axes.join(","))
    };
    let domain_rows=[("p","phase"),("j","species"),("e","element")].map(|(name,kind)|json!({"template_id":id(60),"name":name,"kind":kind,"continuous":false,"members_from":null,"bounds":null,"unit_id":null}));
    let bindings=[("p","phase"),("j","species"),("e","element")].map(|(name,source)|json!({"template_id":id(60),"name":name,"source":{"kind":source,"domain":null,"parameter":null}}));
    let symbols=[(40,"inflow"),(41,"outflow")].map(|(symbol,name)|json!({"id":id(symbol),"template_id":id(60),"name":name,"role":"variable","quantity_type_id":balance.source_type(),"indexed_by":axes,"doc":"Independent actual stream coordinate."}));
    let contributions=[(50,"inflow","into_scope"),(51,"outflow","out_of_scope")].map(|(contribution,name,orientation)|json!({"template_id":id(60),"contribution_decl_id":id(contribution),"name":format!("{name}_contribution"),"law_family":balance.family(),"expression":format!("{name}{suffix}"),"orientation":orientation,"scope":"self","guard_id":null,"doc":"Complete indexed source contribution."}));
    let material = matches!(
        balance,
        Balance::ComponentTotal
            | Balance::ComponentPhase
            | Balance::ElementMolar
            | Balance::ElementMass
    );
    let contracts=[50,51].map(|contribution|json!({"contribution_decl_id":id(contribution),"indexed_by":axes,"quantity_type_id":balance.source_type(),"subject":{"kind":balance.subject(),(balance.subject()):if material {json!({"member":{"kind":"axis","axis":{"position":1}},"phase":{"kind":"axis","axis":{"position":0}}})}else{json!({"phase":if axes.is_empty(){None}else{Some(json!({"kind":"axis","axis":{"position":0}}))}})}}}));
    json!({"templates":[{"id":id(60),"name":"ExplicitControl","version":"1","kind":"control_volume","doc":"Two phases and two species remain indexed until later compilation."}],"template_symbols":symbols,"template_symbol_contracts":[{"symbol_decl_id":id(40),"solver_type":"continuous","semantic_role":"state"},{"symbol_decl_id":id(41),"solver_type":"continuous","semantic_role":"state"}],"template_domains":domain_rows,"template_domain_bindings":bindings,"template_scopes":[{"template_id":id(60),"name":"self","scope_id":id(65)}],"template_contributions":contributions,"template_contribution_contracts":contracts})
}
pub(super) fn instance() -> Value {
    json!({"scopes":[{"scope_id":id(65),"root_term_id":id(66)}],"selector_terms":[{"term_id":id(66),"scope_id":id(65),"parent_term_id":null,"ordinal":0,"op":"self","entity_id":null,"entity_kind":null,"tag":null,"parameter_name":null}],"instances":[{"id":id(70),"template_id":id(60),"name":"control","property_package_id":id(95),"param_values":[],"feature_values":[],"doc":"Actual fixture control."}]})
}
pub(super) fn materials(options: &Options) -> Value {
    let species=[(100,"hydrogen","H2",0.002_016),(101,"oxygen","O2",0.031_998)].map(|(species,name,formula,mw)|json!({"id":id(species),"name":name,"formula":formula,"mw":options.molecular_weight.then_some(mw),"component_type":"Component","charge":0,"dissociation_species":null,"valid_phase_types":["vaporPhase","liquidPhase"],"doc":"Known diatomic stoichiometry with optional explicit kg/mol source value."}));
    let phases=[(110,"vapor","vaporPhase"),(111,"liquid","liquidPhase")].map(|(phase,name,kind)|json!({"id":id(phase),"name":name,"phase_type":kind,"is_solvent_phase":false,"doc":"Declared phase; no equation of state is assumed."}));
    let pairs = [110, 111]
        .into_iter()
        .flat_map(|phase| {
            [100, 101].map(move |species| json!({"phase_id":id(phase),"species_id":id(species)}))
        })
        .collect::<Vec<_>>();
    json!({"species":species,"species_elements":[{"species_id":id(100),"element_id":"2fb5ca65384a2466d41fc32d22a0c40d","count":options.hydrogen_count},{"species_id":id(101),"element_id":"0b238b1504393021f5c19d87528e9c7f","count":2.0}],"phases":phases,"phase_species":pairs,"material_systems":[{"id":id(92),"name":"two_species_two_phases","species_ids":[id(100),id(101)],"phase_ids":[id(110),id(111)],"doc":"Complete finite material subjects."}]})
}
