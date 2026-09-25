// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selection and checked lowering of the existing authored template vocabulary.
use super::{WorkflowError, relation};
use pse_ids::{FramedHasher, SemanticId};
use pse_model::HeapUsage;
use pse_model::diagnostic::{BoundaryClass, BoundaryDiagnostic};
use pse_relations::generated::reference::{connection_bindings, law_bindings};
use pse_relations::{
    columnar::{FieldCheckedBatch, RelationRow},
    generated::authored::*,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) mod lower;
pub(super) use lower::lower;
#[cfg(test)]
mod tests;

/// An explicit accounting of selected execution versus stored declarations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmissionEntry {
    /// Registry relation name.
    pub relation: String,
    /// Rows that affect this model's checked projection.
    pub selected: usize,
    /// Rows outside the selected model; retained input is not execution evidence.
    pub nonexecuting: usize,
}

pub(super) fn invalid(
    ids: impl IntoIterator<Item = SemanticId>,
    rule: impl Into<String>,
) -> WorkflowError {
    BoundaryDiagnostic::new(BoundaryClass::InvalidModel, "selected_admission", ids, rule).into()
}
pub(super) fn unsupported(
    ids: impl IntoIterator<Item = SemanticId>,
    rule: impl Into<String>,
) -> WorkflowError {
    BoundaryDiagnostic::new(BoundaryClass::Unsupported, "selected_admission", ids, rule).into()
}

// The rows themselves and their wire codecs remain registry-generated. This macro
// only removes mechanical collection plumbing at the workflow boundary.
macro_rules! declarations {
    ($($field:ident : $module:ident => $key:expr),+ $(,)?) => {
        /// Exact authored relations used to select and specialize reusable units.
        #[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct CompositionDeclarations {
            $(#[doc = concat!("Rows from authored.", stringify!($module), ".")]
              #[serde(default)] pub $field: Vec<$module::Row>,)+
        }
        impl CompositionDeclarations {
            pub(crate) fn load(batches: &BTreeMap<SemanticId, FieldCheckedBatch>) -> Result<Self, WorkflowError> {
                Ok(Self { $($field: batches.get(&$module::RELATION_ID).map($module::Row::rows).transpose().map_err(relation)?.unwrap_or_default(),)+ })
            }
            fn relation_name(field: &str) -> String { match field { $(stringify!($field) => {
                let namespace=if matches!(stringify!($module), "connection_bindings"|"law_bindings") { "reference" } else { "authored" };
                format!("{namespace}.{}",stringify!($module))
            },)+ _ => field.into() } }
            pub(crate) fn known_relations() -> BTreeSet<SemanticId> {
                BTreeSet::from([$($module::RELATION_ID,)+])
            }
            pub(crate) fn canonicalize(&mut self) -> Result<(), WorkflowError> {
                $(self.$field.sort_by_key($key);
                if self.$field.windows(2).any(|w| $key(&w[0]) == $key(&w[1])) {
                    return Err(invalid([], concat!("duplicate authored.", stringify!($module), " key")));
                })+
                Ok(())
            }
            pub(crate) fn merge(&mut self, other: &Self) -> Result<(), WorkflowError> {
                $(for row in &other.$field {
                    if let Some(old) = self.$field.iter().find(|old| $key(old) == $key(row)) {
                        if old != row { return Err(invalid([], concat!("conflicting authored.", stringify!($module), " key"))); }
                    } else { self.$field.push(row.clone()); }
                })+
                self.canonicalize()
            }
            pub(crate) fn bytes(&self) -> usize { 0usize $(.saturating_add(self.$field.owned_bytes()))+ }
            pub(crate) fn tables(&self, registry: &pse_schema::Registry) -> Result<BTreeMap<SemanticId, FieldCheckedBatch>, WorkflowError> {
                let mut out = BTreeMap::new();
                $(if !self.$field.is_empty() {
                    let mut b = $module::Builder::with_registry(registry, self.$field.len()).map_err(relation)?;
                    for row in &self.$field { b.push(row.clone()).map_err(relation)?; }
                    out.insert($module::RELATION_ID, b.finish().map_err(relation)?);
                })+
                Ok(out)
            }
        }
    }
}
declarations! {
    reaction_applications: reaction_applications => |r: &reaction_applications::Row| r.application_id,
    material_systems: material_systems => |r: &material_systems::Row| r.material_system_id,
    species: species => |r: &species::Row| r.species_id,
    species_elements: species_elements => |r: &species_elements::Row| (r.species_id, r.element_id),
    phases: phases => |r: &phases::Row| r.phase_id,
    phase_species: phase_species => |r: &phase_species::Row| (r.phase_id, r.species_id),
    reactions: reactions => |r: &reactions::Row| r.reaction_id,
    stoichiometry: stoichiometry => |r: &stoichiometry::Row| (r.reaction_id, r.phase_id, r.species_id),
    laws: template_law_instances => |r: &template_law_instances::Row| r.law_instance_decl_id,
    law_contracts: template_law_contracts => |r: &template_law_contracts::Row| r.law_instance_decl_id,
    law_bindings: law_bindings => |r: &law_bindings::Row| r.law_template_id,
    roots: model_compositions => |r: &model_compositions::Row| r.model_id,
    templates: templates => |r: &templates::Row| r.template_id,
    parameters: template_params => |r: &template_params::Row| (r.template_id, r.name.clone()),
    features: template_features => |r: &template_features::Row| (r.template_id, r.name.clone()),
    feature_rules: template_feature_rules => |r: &template_feature_rules::Row| (r.template_id, r.antecedent.clone(), r.consequent.clone()),
    guards: template_guards => |r: &template_guards::Row| r.guard_id,
    template_domains: template_domains => |r: &template_domains::Row| (r.template_id, r.name.clone()),
    template_domain_bindings: template_domain_bindings => |r: &template_domain_bindings::Row| (r.template_id, r.name.clone()),
    symbols: template_symbols => |r: &template_symbols::Row| r.symbol_decl_id,
    equations: template_equations => |r: &template_equations::Row| r.equation_decl_id,
    ports: template_ports => |r: &template_ports::Row| (r.template_id, r.name.clone()),
    port_members: template_port_members => |r: &template_port_members::Row| (r.template_id, r.ordinal),
    contributions: template_contributions => |r: &template_contributions::Row| r.contribution_decl_id,
    submodels: template_submodels => |r: &template_submodels::Row| (r.template_id, r.name.clone()),
    instances: instances => |r: &instances::Row| r.instance_id,
    instance_domains: instance_domain_bindings => |r: &instance_domain_bindings::Row| (r.instance_id, r.domain_name.clone()),
    domains: domains => |r: &domains::Row| r.domain_id,
    members: domain_members => |r: &domain_members::Row| (r.domain_id, r.ordinal),
    connections: connections => |r: &connections::Row| r.connection_id,
    connection_rules: connection_bindings => |r: &connection_bindings::Row| r.rule_template_id,
    case_specs: case_specs => |r: &case_specs::Row| r.spec_id,
}

impl CompositionDeclarations {
    /// Physics and admission policies participate in revision identity even when
    /// they do not change an equation body. Presentation names/docs do not.
    pub(super) fn frame_material_contract(&self, hash: &mut FramedHasher) {
        use pse_model::SemanticFrame;
        hash.str("selected-material-contract.v1");
        self.reaction_applications.frame(hash);
        self.species_elements.frame(hash);
        self.phase_species.frame(hash);
        self.stoichiometry.frame(hash);
        hash.str("material_systems")
            .u64(self.material_systems.len() as u64);
        for row in &self.material_systems {
            row.material_system_id.frame(hash);
            row.species_ids.frame(hash);
            row.phase_ids.frame(hash);
        }
        hash.str("species").u64(self.species.len() as u64);
        for row in &self.species {
            row.species_id.frame(hash);
            row.formula.frame(hash);
            row.mw.frame(hash);
            row.component_type.frame(hash);
            row.charge.frame(hash);
            row.dissociation_species.frame(hash);
            row.valid_phase_types.frame(hash);
        }
        hash.str("phases").u64(self.phases.len() as u64);
        for row in &self.phases {
            row.phase_id.frame(hash);
            row.phase_type.frame(hash);
            row.is_solvent_phase.frame(hash);
        }
        hash.str("reactions").u64(self.reactions.len() as u64);
        for row in &self.reactions {
            row.reaction_id.frame(hash);
            row.kind.frame(hash);
            row.basis.frame(hash);
            row.concentration_form.frame(hash);
            row.reaction_phase_id.frame(hash);
        }
    }

    pub(super) fn select(
        &mut self,
        model: SemanticId,
        cases: &BTreeSet<SemanticId>,
        provider_materials: &BTreeSet<SemanticId>,
    ) -> Result<(BTreeSet<SemanticId>, Vec<AdmissionEntry>), WorkflowError> {
        self.canonicalize()?;
        let mut report = vec![];
        macro_rules! retain {
            ($field:ident, $keep:expr) => {{
                let before = self.$field.len();
                self.$field.retain($keep);
                report.push(AdmissionEntry {
                    relation: Self::relation_name(stringify!($field)),
                    selected: self.$field.len(),
                    nonexecuting: before - self.$field.len(),
                });
            }};
        }
        retain!(roots, |r| r.model_id == model);
        let mut selected = BTreeSet::from([model]);
        selected.extend(cases);
        let mut instances = BTreeSet::new();
        if let Some(root) = self.roots.first() {
            if !self
                .instances
                .iter()
                .any(|r| r.instance_id == root.root_instance_id)
            {
                return Err(invalid(
                    [model, root.root_instance_id],
                    "composition root is missing",
                ));
            }
            if self
                .instances
                .iter()
                .find(|r| r.instance_id == root.root_instance_id)
                .is_some_and(|r| r.parent_instance_id.is_some())
            {
                return Err(invalid(
                    [root.root_instance_id],
                    "selected composition root has a parent",
                ));
            }
            instances.insert(root.root_instance_id);
            loop {
                let old = instances.len();
                for row in &self.instances {
                    if row
                        .parent_instance_id
                        .is_some_and(|p| instances.contains(&p))
                    {
                        instances.insert(row.instance_id);
                    }
                }
                if old == instances.len() {
                    break;
                }
            }
        }
        retain!(instances, |r| instances.contains(&r.instance_id));
        let templates: BTreeSet<_> = self.instances.iter().map(|r| r.template_id).collect();
        selected.extend(&instances);
        selected.extend(&templates);
        retain!(templates, |r| templates.contains(&r.template_id));
        retain!(parameters, |r| templates.contains(&r.template_id));
        retain!(features, |r| templates.contains(&r.template_id));
        retain!(feature_rules, |r| templates.contains(&r.template_id));
        retain!(guards, |r| templates.contains(&r.template_id));
        retain!(template_domains, |r| templates.contains(&r.template_id));
        retain!(template_domain_bindings, |r| templates
            .contains(&r.template_id));
        retain!(symbols, |r| templates.contains(&r.template_id));
        retain!(equations, |r| templates.contains(&r.template_id));
        retain!(ports, |r| templates.contains(&r.template_id));
        retain!(port_members, |r| templates.contains(&r.template_id));
        retain!(contributions, |r| templates.contains(&r.template_id));
        retain!(submodels, |r| templates.contains(&r.template_id));
        retain!(laws, |r| templates.contains(&r.template_id));
        let law_ids: BTreeSet<_> = self.laws.iter().map(|r| r.law_instance_decl_id).collect();
        let law_templates: BTreeSet<_> = self.laws.iter().map(|r| r.law_template_id).collect();
        retain!(law_contracts, |r| law_ids.contains(&r.law_instance_decl_id));
        retain!(law_bindings, |r| law_templates.contains(&r.law_template_id));
        selected.extend(law_ids);
        selected.extend(law_templates);
        retain!(reaction_applications, |r| r.model_id == model
            && cases.contains(&r.case_id));
        let reaction_ids: BTreeSet<_> = self
            .reaction_applications
            .iter()
            .map(|r| r.reaction_id)
            .collect();
        let material_ids: BTreeSet<_> = self
            .reaction_applications
            .iter()
            .map(|r| r.material_system_id)
            .chain(provider_materials.iter().copied())
            .collect();
        retain!(material_systems, |r| material_ids
            .contains(&r.material_system_id));
        let species_ids: BTreeSet<_> = self
            .material_systems
            .iter()
            .flat_map(|r| r.species_ids.iter().copied())
            .collect();
        let phase_ids: BTreeSet<_> = self
            .material_systems
            .iter()
            .flat_map(|r| r.phase_ids.iter().copied())
            .collect();
        retain!(species, |r| species_ids.contains(&r.species_id));
        retain!(species_elements, |r| species_ids.contains(&r.species_id));
        retain!(phases, |r| phase_ids.contains(&r.phase_id));
        retain!(phase_species, |r| phase_ids.contains(&r.phase_id)
            && species_ids.contains(&r.species_id));
        retain!(reactions, |r| reaction_ids.contains(&r.reaction_id));
        retain!(stoichiometry, |r| reaction_ids.contains(&r.reaction_id));
        selected.extend(reaction_ids);
        selected.extend(material_ids);
        selected.extend(species_ids);
        selected.extend(phase_ids);
        selected.extend(self.reaction_applications.iter().map(|r| r.application_id));
        retain!(instance_domains, |r| instances.contains(&r.instance_id));
        let mut domains: BTreeSet<_> = self
            .instance_domains
            .iter()
            .map(|r| r.domain_id)
            .chain(
                self.template_domain_bindings
                    .iter()
                    .filter_map(|r| r.source.domain.as_ref().map(|d| d.domain_id)),
            )
            .collect();
        for binding in &self.template_domain_bindings {
            if let Some(parameter) = &binding.source.parameter {
                for instance in self
                    .instances
                    .iter()
                    .filter(|i| i.template_id == binding.template_id)
                {
                    let value = instance
                        .param_values
                        .iter()
                        .find(|p| p.name == parameter.name)
                        .map(|p| p.value.as_str())
                        .or_else(|| {
                            self.parameters
                                .iter()
                                .find(|p| {
                                    p.template_id == binding.template_id && p.name == parameter.name
                                })
                                .and_then(|p| p.default.as_deref())
                        });
                    if let Some(value) = value {
                        domains.insert(SemanticId::parse_hex(value).map_err(|_| {
                            invalid(
                                [instance.instance_id],
                                "domain parameter is not an identity",
                            )
                        })?);
                    }
                }
            }
        }
        retain!(domains, |r| domains.contains(&r.domain_id));
        retain!(members, |r| domains.contains(&r.domain_id));
        retain!(case_specs, |r| cases.contains(&r.case_id));
        let specs: BTreeSet<_> = self.case_specs.iter().map(|r| r.spec_id).collect();
        selected.extend(domains);
        selected.extend(specs);
        selected.extend(self.symbols.iter().map(|r| r.symbol_decl_id));
        selected.extend(self.equations.iter().map(|r| r.equation_decl_id));
        selected.extend(self.contributions.iter().map(|r| r.contribution_decl_id));
        // Port IDs are declared by the shared instance/name identity operation.
        let ports: BTreeSet<_> = self
            .instances
            .iter()
            .flat_map(|i| {
                self.ports
                    .iter()
                    .filter(move |p| p.template_id == i.template_id)
                    .map(move |p| port_id(i.instance_id, &p.name))
            })
            .collect();
        retain!(connections, |r| ports.contains(&r.from_port_id)
            || ports.contains(&r.to_port_id));
        selected.extend(ports);
        selected.extend(self.connections.iter().map(|r| r.connection_id));
        let rules: BTreeSet<_> = self
            .connections
            .iter()
            .map(|r| r.rule_template_id)
            .collect();
        retain!(connection_rules, |r| rules.contains(&r.rule_template_id));
        selected.extend(rules);
        Ok((selected, report))
    }
}
/// Stable port identity shared by authored connection producers and admission.
pub fn port_id(instance: SemanticId, name: &str) -> SemanticId {
    pse_ids::named_id(instance, &format!("port:{name}"))
}
/// Stable scalar occurrence identity; an empty index denotes a scalar declaration.
pub fn symbol_id(
    instance: SemanticId,
    declaration: SemanticId,
    members: &[SemanticId],
) -> SemanticId {
    let mut h = FramedHasher::new("pse.composition.symbol.v1");
    h.id(&instance).id(&declaration).u64(members.len() as u64);
    for id in members {
        h.id(id);
    }
    h.finish_id()
}

pub(super) fn classify_documents(
    batches: &BTreeMap<SemanticId, FieldCheckedBatch>,
    selected: &BTreeSet<SemanticId>,
    physical: &super::PhysicalContext,
    registry: &pse_schema::Registry,
    report: &mut Vec<AdmissionEntry>,
) -> Result<(), WorkflowError> {
    let known = CompositionDeclarations::known_relations();
    for (id, batch) in batches {
        if known.contains(id) {
            continue;
        }
        let spec = registry
            .relation_by_id(*id)
            .ok_or_else(|| invalid([*id], "unknown declaration family"))?;
        let name = spec.key.qualified_name();
        let executed = matches!(
            name.as_str(),
            "authored.computation_models"
                | "authored.physical_balances"
                | "authored.numerical_requirements"
                | "authored.provider_scaling_bindings"
                | "authored.default_scaling"
                | "authored.dynamic_cases"
                | "authored.fit_cases"
                | "authored.native_providers"
                | "authored.datasets"
                | "authored.observations"
        ) || physical.sources.contains_key(&spec.key);
        let descriptive = matches!(
            name.as_str(),
            "authored.packages"
                | "authored.package_unit_sets"
                | "authored.entities"
                | "authored.entity_names"
                | "authored.documents"
                | "authored.aliases"
                | "provenance.assertions"
                | "authored.template_display"
                | "authored.template_display_indices"
        );
        let mut consumed = 0;
        for row in 0..batch.batch().num_rows() {
            let mut witnesses = BTreeSet::new();
            for (column, field) in spec.columns.iter().enumerate() {
                selected_ids(
                    field,
                    batch.batch().column(column).as_ref(),
                    row,
                    selected,
                    &mut witnesses,
                )?;
            }
            if !witnesses.is_empty() && !executed && !descriptive {
                return Err(unsupported(
                    witnesses,
                    format!("selected {name} declaration has no execution interpretation"),
                ));
            }
            if executed && (!witnesses.is_empty() || physical.sources.contains_key(&spec.key)) {
                consumed += 1;
            }
        }
        report.push(AdmissionEntry {
            relation: name,
            selected: consumed,
            nonexecuting: batch.batch().num_rows() - consumed,
        });
    }
    Ok(())
}

fn selected_ids(
    field: &pse_schema::model::FieldContract,
    array: &dyn datafusion::arrow::array::Array,
    row: usize,
    selected: &BTreeSet<SemanticId>,
    found: &mut BTreeSet<SemanticId>,
) -> Result<(), WorkflowError> {
    use datafusion::arrow::{
        array::{AsArray, LargeListViewArray, ListViewArray},
        datatypes::DataType,
    };
    use pse_relations::columnar::ArrowValue;
    use pse_schema::model::{ExtensionUse, FieldContract};
    if array.is_null(row) {
        return Ok(());
    }
    if field.extension() == Some(ExtensionUse::SemanticId) {
        let id = SemanticId::read(array, row).map_err(relation)?;
        if selected.contains(&id) {
            found.insert(id);
        }
        return Ok(());
    }
    let nested = match field.data_type() {
        DataType::Struct(fields) => {
            let structure = array.as_struct();
            for (i, f) in fields.iter().enumerate() {
                selected_ids(
                    &FieldContract::from_field(f.as_ref().clone()),
                    structure.column(i).as_ref(),
                    row,
                    selected,
                    found,
                )?;
            }
            None
        }
        DataType::List(f) => Some((f, array.as_list::<i32>().value(row))),
        DataType::LargeList(f) => Some((f, array.as_list::<i64>().value(row))),
        DataType::FixedSizeList(f, _) => Some((f, array.as_fixed_size_list().value(row))),
        DataType::ListView(f) => Some((
            f,
            array
                .as_any()
                .downcast_ref::<ListViewArray>()
                .ok_or_else(|| invalid([], "list-view contract"))?
                .value(row),
        )),
        DataType::LargeListView(f) => Some((
            f,
            array
                .as_any()
                .downcast_ref::<LargeListViewArray>()
                .ok_or_else(|| invalid([], "large-list-view contract"))?
                .value(row),
        )),
        DataType::Map(f, _) => {
            let values: datafusion::arrow::array::ArrayRef =
                std::sync::Arc::new(array.as_map().value(row));
            Some((f, values))
        }
        _ => None,
    };
    if let Some((field, values)) = nested {
        let field = FieldContract::from_field(field.as_ref().clone());
        for i in 0..values.len() {
            selected_ids(&field, values.as_ref(), i, selected, found)?;
        }
    }
    Ok(())
}
