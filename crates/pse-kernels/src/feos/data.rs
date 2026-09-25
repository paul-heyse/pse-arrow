// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Checked, explicit bindings to library-owned chemical parameter records.
use super::{ComponentBinding, Eos, ProviderError};
use ::feos::{
    ideal_gas::{Dippr, DipprParameters},
    pcsaft::{PcSaft, PcSaftAssociationRecord, PcSaftBinaryRecord, PcSaftParameters, PcSaftRecord},
};
use feos_core::{
    EquationOfState,
    parameter::{BinaryRecord, Identifier, IdentifierOption, PureRecord},
};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
pub use pse_model::generated::enums::MissingInteractionPolicy;
use std::collections::{BTreeMap, BTreeSet};

/// Authored parameter data, including its source and missing-pair policy.
#[derive(Clone, Debug)]
pub struct FeosData {
    /// FeOS pure PC-SAFT records as JSON.
    pub pcsaft: String,
    /// FeOS pure DIPPR records as JSON.
    pub ideal_gas: String,
    /// FeOS binary PC-SAFT records as JSON.
    pub binary: String,
    /// Source attribution. This does not certify empirical validity.
    pub provenance: String,
    /// Explicit handling of absent binary records.
    pub missing_interactions: MissingInteractionPolicy,
}
/// Resolved chemical identity, in the provider's composition order.
#[derive(Clone, Debug)]
pub struct ComponentRecord {
    /// Authored species.
    pub species: SemanticId,
    /// Matched CAS identifier in both datasets.
    pub cas: String,
    /// Formula supplied by the dataset, when present.
    pub formula: Option<String>,
    /// Kilograms per mole.
    pub molar_mass: f64,
}
impl FeosData {
    /// Bundled records as an ordinary explicit data declaration.
    pub fn light_hydrocarbons() -> Self {
        Self {
            pcsaft: include_str!("../../data/pcsaft-light-hydrocarbons.json").into(),
            ideal_gas: include_str!("../../data/ideal-gas-light-hydrocarbons.json").into(),
            binary: "[]".into(),
            provenance: "FeOS example PC-SAFT and DIPPR light-hydrocarbon records; see data/README.md; explicit zero binary interactions".into(),
            missing_interactions: MissingInteractionPolicy::Zero,
        }
    }
    pub(super) fn admit(
        &self,
        components: &[ComponentBinding],
    ) -> Result<(Eos, ContentHash, Vec<ComponentRecord>), ProviderError> {
        let bad = |message: &str| ProviderError::Contract(message.into());
        if self.provenance.trim().is_empty()
            || self.provenance.len() > 4096
            || self
                .pcsaft
                .len()
                .saturating_add(self.ideal_gas.len())
                .saturating_add(self.binary.len())
                > 16 * 1024 * 1024
        {
            return Err(bad("bounded data and explicit provenance required"));
        }
        let pc: Vec<PureRecord<PcSaftRecord, PcSaftAssociationRecord>> =
            serde_json::from_str(&self.pcsaft).map_err(|e| bad(&e.to_string()))?;
        let ig: Vec<PureRecord<Dippr, ()>> =
            serde_json::from_str(&self.ideal_gas).map_err(|e| bad(&e.to_string()))?;
        let binary: Vec<BinaryRecord<Identifier, PcSaftBinaryRecord, PcSaftAssociationRecord>> =
            serde_json::from_str(&self.binary).map_err(|e| bad(&e.to_string()))?;
        fn index<M, A>(
            rows: &[PureRecord<M, A>],
        ) -> Result<BTreeMap<String, usize>, ProviderError> {
            let mut result = BTreeMap::new();
            for (i, row) in rows.iter().enumerate() {
                let cas = row
                    .identifier
                    .as_str(IdentifierOption::Cas)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| ProviderError::Contract("missing record CAS".into()))?;
                if result.insert(cas.into(), i).is_some() {
                    return Err(ProviderError::Contract(format!("duplicate CAS {cas}")));
                }
            }
            Ok(result)
        }
        let pi = index(&pc)?;
        let ii = index(&ig)?;
        let mut pairs = BTreeSet::new();
        for row in &binary {
            let a = row
                .id1
                .as_str(IdentifierOption::Cas)
                .ok_or_else(|| bad("missing binary CAS"))?;
            let b = row
                .id2
                .as_str(IdentifierOption::Cas)
                .ok_or_else(|| bad("missing binary CAS"))?;
            let pair = if a <= b {
                (a.to_owned(), b.to_owned())
            } else {
                (b.to_owned(), a.to_owned())
            };
            if !pi.contains_key(a) || !pi.contains_key(b) || !pairs.insert(pair) {
                return Err(bad("unknown or duplicate binary pair"));
            }
        }
        let mut selected = BTreeSet::new();
        let mut selected_pc = vec![];
        let mut selected_ig = vec![];
        let mut records = vec![];
        for binding in components {
            if binding.pcsaft_cas != binding.ideal_gas_cas
                || !selected.insert(binding.pcsaft_cas.clone())
            {
                return Err(bad(
                    "duplicate species record or inconsistent chemical identity",
                ));
            }
            let p = &pc[*pi
                .get(&binding.pcsaft_cas)
                .ok_or_else(|| bad("unknown PC-SAFT species record"))?];
            let i = &ig[*ii
                .get(&binding.ideal_gas_cas)
                .ok_or_else(|| bad("unknown ideal-gas species record"))?];
            if [
                p.molarweight,
                p.model_record.m,
                p.model_record.sigma,
                p.model_record.epsilon_k,
            ]
            .iter()
            .any(|x| !x.is_finite() || *x <= 0.0)
            {
                return Err(bad("invalid PC-SAFT record"));
            }
            selected_pc.push(p.clone());
            selected_ig.push(i.clone());
            records.push(ComponentRecord {
                species: binding.species,
                cas: binding.pcsaft_cas.clone(),
                formula: p.identifier.formula.clone(),
                molar_mass: p.molarweight / 1000.0,
            });
        }
        if self.missing_interactions == MissingInteractionPolicy::RequireExplicit {
            for a in &selected {
                for b in selected
                    .range::<String, _>((std::ops::Bound::Excluded(a), std::ops::Bound::Unbounded))
                {
                    if !pairs.contains(&(a.clone(), b.clone())) {
                        return Err(bad("missing explicitly required binary interaction"));
                    }
                }
            }
        }
        let pc = PcSaftParameters::from_records(selected_pc, binary, IdentifierOption::Cas)
            .map_err(super::failure)?;
        let ig = DipprParameters::from_records(selected_ig, vec![], IdentifierOption::Cas)
            .map_err(super::failure)?;
        let mut hash = FramedHasher::new("pse.feos.data.v2");
        hash.str(&self.pcsaft)
            .str(&self.ideal_gas)
            .str(&self.binary)
            .str(&self.provenance)
            .str(self.missing_interactions.as_str());
        for binding in components {
            hash.id(&binding.species)
                .str(&binding.pcsaft_cas)
                .str(&binding.ideal_gas_cas);
        }
        Ok((
            EquationOfState::new(Dippr::new(ig), PcSaft::new(pc)),
            hash.finish_hash(),
            records,
        ))
    }
}
