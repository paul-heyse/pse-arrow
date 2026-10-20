// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pure projections of explicitly scoped original source maps.
use super::*;
use pse_ids::SemanticId;

/// One actual local source-token mapping in an original semantic occurrence.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
pub struct ReviewSourceTerm {
    /// Actual owning semantic occurrence.
    pub instance: SemanticId,
    /// Original instance binding ordinal.
    pub ordinal: usize,
    /// Exact local check token, never guessed from arithmetic text.
    pub local: SemanticId,
    /// Exact original member supplied by the checked-member map.
    pub member: SemanticId,
    /// JSON pointer inside the decoded map, not inside its encoded byte array.
    pub payload_path: String,
}
/// Actual ordered slot or row addend, preserving aliases and repeated contributions.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
pub struct ReviewSourceOccurrence {
    /// Original occurrence identity.
    pub instance: SemanticId,
    /// Original instance binding ordinal.
    pub ordinal: usize,
    /// Original formal input, absent for an output contribution.
    pub formal: Option<usize>,
    /// Original body output, absent for an input slot.
    pub output: Option<usize>,
    /// Actual affine gather coefficient or signed output contribution coefficient.
    pub scale: f64,
    /// Actual affine gather offset, absent for an output contribution.
    pub offset: Option<f64>,
    /// Actual mapped source terms belonging to this occurrence.
    pub terms: Vec<ReviewSourceTerm>,
    /// JSON pointer within the decoded map to this original slot/addend.
    pub payload_path: String,
}
/// Supported physical attribution; no transformed dual/stationarity claim is made.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
pub struct ReviewSourceAttribution {
    /// Retained fact index. The authoritative raw source is `/facts/{fact}/payload`.
    pub fact: usize,
    /// Original affected member, preserving semantic identity.
    pub member: SemanticId,
    /// Original row, variable, fixed_input or source_member role.
    pub role: String,
    /// Recorded positive physical magnitude, absent when no nominal was recorded.
    pub nominal: Option<f64>,
    /// Actual authored declaration, when its lineage is retained.
    pub declaration: Option<SemanticId>,
    /// Actual instantiated member owner, when supplied by its lineage.
    pub instance: Option<SemanticId>,
    /// Descriptive display path, never an identity or resolver.
    pub display_path: Option<String>,
    /// Pointers inside the decoded map, kept separate from the real payload bytes.
    pub payload_paths: Vec<String>,
    /// Direct original check-token correspondences, including internal source
    /// members that have no free column or output contribution.
    pub source_terms: Vec<ReviewSourceTerm>,
    /// Original mapped slots/addends and their source terms.
    pub occurrences: Vec<ReviewSourceOccurrence>,
    /// Exact retained authored field spans for the member's declaration.
    pub fields: Vec<serde_json::Value>,
    /// Original map's explicit native pullback limitation.
    pub native_pullback_unavailable: String,
}
pub(super) fn valid_scope(scope: &str) -> bool {
    let parts = scope.split('/').collect::<Vec<_>>();
    matches!(parts.as_slice(), ["", "steps", step] if step.parse::<usize>().is_ok())
        || matches!(parts.as_slice(), ["", "steps", step, "native", mechanism, component]
            if step.parse::<usize>().is_ok() && mechanism.parse::<usize>().is_ok()
            && (*component == "None" || component.strip_prefix("Some(").and_then(|n| n.strip_suffix(')')).is_some_and(|n| n.parse::<usize>().is_ok())))
}
impl ReviewContextDocument {
    /// Attach an already retained map using an explicit original step/component
    /// association. A structure hash is never treated as a source-closure hash.
    pub fn attach_source_map(
        &mut self,
        step: usize,
        native: Option<(usize, Option<usize>)>,
        map: &super::super::ModelingSourceMapDocument,
    ) -> Result<(), WorkflowError> {
        map.validate()?;
        if !self.selection.selects(step) {
            return Err(contract("source map is outside selected original steps"));
        }
        let scope = native.map_or_else(
            || format!("/steps/{step}"),
            |(mechanism, component)| format!("/steps/{step}/native/{mechanism}/{component:?}"),
        );
        if native.is_some()
            && !self
                .facts
                .iter()
                .any(|fact| fact.path == scope && fact.codec == "native-attempt-record.v1")
        {
            return Err(contract(
                "source map native scope has no exact retained attempt record",
            ));
        }
        let path = format!("{scope}/source_map");
        if self.facts.iter().any(|fact| fact.path == path) {
            return Err(contract("duplicate selected source map scope"));
        }
        // Count borrowed inputs before allocating the encoded payload. The outer
        // document encodes payload as JSON byte integers: each original encoded
        // byte takes at most three digits and a comma, in addition to the empty
        // fact header and the facts-array delimiter.
        #[derive(serde::Serialize)]
        struct BorrowedFactHeader<'a> {
            path: &'a str,
            codec: &'a str,
            payload: &'a [u8],
        }
        let limit = self.selection.maximum_bytes;
        let used = super::super::replay::encoded_extent(self, limit)?;
        let map_bytes = super::super::replay::encoded_extent(map, limit.saturating_sub(used))?;
        let header_bytes = super::super::replay::encoded_extent(
            &BorrowedFactHeader {
                path: &path,
                codec: "modeling-source-map.v1",
                payload: &[],
            },
            limit.saturating_sub(used),
        )?;
        used.checked_add(header_bytes)
            .and_then(|total| {
                map_bytes
                    .checked_mul(4)
                    .and_then(|payload| total.checked_add(payload))
            })
            .and_then(|total| total.checked_add(1))
            .filter(|total| *total <= limit)
            .ok_or_else(|| {
                contract("selected review source map exceeds finite allocation before encoding")
            })?;
        let payload = serde_json::to_vec(map).map_err(|error| contract(error.to_string()))?;
        self.facts.push(ReviewContextFact {
            path,
            codec: "modeling-source-map.v1".into(),
            payload,
        });
        if let Err(error) = self.validate() {
            self.facts.pop();
            return Err(error);
        }
        Ok(())
    }
    /// Copy the already retained factual engine members for the existing Frozen
    /// engine slot. No runtime/session factory is consulted.
    pub fn engine_observations(
        &self,
    ) -> Result<Vec<pse_engine::session::CopiedExecutionObservations>, WorkflowError> {
        self.facts
            .iter()
            .filter(|fact| fact.codec == "engine-execution-observations.v1")
            .map(|fact| {
                serde_json::from_slice(&fact.payload).map_err(|error| contract(error.to_string()))
            })
            .collect()
    }
    /// Follow only exact original scope and member correspondences. An empty
    /// projection means unavailable mapping; it never establishes elimination or a
    /// universal native pullback. This operation performs no numerical work.
    pub fn source_map_attribution(
        &self,
        scope: &str,
        members: &[SemanticId],
    ) -> Result<Vec<ReviewSourceAttribution>, WorkflowError> {
        if !valid_scope(scope) {
            return Err(contract(
                "source attribution requires exact step/component scope",
            ));
        }
        let path = format!("{scope}/source_map");
        let Some((fact, encoded)) = self
            .facts
            .iter()
            .enumerate()
            .find(|(_, fact)| fact.path == path && fact.codec == "modeling-source-map.v1")
        else {
            return Ok(Vec::new());
        };
        let map: super::super::ModelingSourceMapDocument = serde_json::from_slice(&encoded.payload)
            .map_err(|error| contract(error.to_string()))?;
        map.validate()?;
        let native = if scope.contains("/native/") {
            let Some(fact) = self
                .facts
                .iter()
                .find(|fact| fact.path == scope && fact.codec == "native-attempt-record.v1")
            else {
                return Ok(Vec::new());
            };
            Some(
                serde_json::from_slice::<pse_backend_native::solve::NativeAttemptRecordDocument>(
                    &fact.payload,
                )
                .map_err(|error| contract(error.to_string()))?,
            )
        } else {
            None
        };
        let mut result = Vec::new();
        for member in members
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
        {
            if native.as_ref().is_some_and(|native| {
                !native.variables.contains(&member) && !native.rows.contains(&member)
            }) {
                continue;
            }
            let (role, nominal, coordinate) =
                if let Some(i) = map.rows.iter().position(|id| *id == member) {
                    (
                        "row",
                        Some(map.row_nominals[i].1),
                        Some(format!("/rows/{i}")),
                    )
                } else if let Some(i) = map.columns.iter().position(|id| *id == member) {
                    (
                        "variable",
                        Some(map.variable_nominals[i].1),
                        Some(format!("/columns/{i}")),
                    )
                } else if let Some(i) = map.fixed_inputs.iter().position(|id| *id == member) {
                    ("fixed_input", None, Some(format!("/fixed_inputs/{i}")))
                } else {
                    ("source_member", None, None)
                };
            let lineage = map
                .members
                .iter()
                .enumerate()
                .find(|(_, lineage)| lineage.member == member);
            let mut payload_paths = coordinate.into_iter().collect::<Vec<_>>();
            if let Some((i, _)) = lineage {
                payload_paths.push(format!("/members/{i}"));
            }
            let mut fields = Vec::new();
            if let Some((_, lineage)) = lineage {
                for (i, field) in map
                    .fields
                    .iter()
                    .enumerate()
                    .filter(|(_, field)| field.declaration == lineage.declaration)
                {
                    payload_paths.push(format!("/fields/{i}"));
                    fields.push(
                        serde_json::to_value(field).map_err(|error| contract(error.to_string()))?,
                    );
                }
            }
            let mut occurrences = Vec::new();
            let mut source_terms = Vec::new();
            for (i, occurrence) in map.instances.iter().enumerate() {
                let terms = || {
                    occurrence
                        .checked_members
                        .iter()
                        .enumerate()
                        .map(|(j, (local, member))| ReviewSourceTerm {
                            instance: occurrence.instance,
                            ordinal: occurrence.ordinal,
                            local: *local,
                            member: *member,
                            payload_path: format!("/instances/{i}/checked_members/{j}"),
                        })
                        .collect::<Vec<_>>()
                };
                for (j, (local, original)) in occurrence
                    .checked_members
                    .iter()
                    .enumerate()
                    .filter(|(_, (_, original))| *original == member)
                {
                    let path = format!("/instances/{i}/checked_members/{j}");
                    payload_paths.push(path.clone());
                    source_terms.push(ReviewSourceTerm {
                        instance: occurrence.instance,
                        ordinal: occurrence.ordinal,
                        local: *local,
                        member: *original,
                        payload_path: path,
                    });
                }
                for (j, slot) in occurrence
                    .slots
                    .iter()
                    .enumerate()
                    .filter(|(_, slot)| slot.member == member)
                {
                    occurrences.push(ReviewSourceOccurrence {
                        instance: occurrence.instance,
                        ordinal: occurrence.ordinal,
                        formal: Some(slot.formal),
                        output: None,
                        scale: slot.scale,
                        offset: Some(slot.offset),
                        terms: terms(),
                        payload_path: format!("/instances/{i}/slots/{j}"),
                    });
                }
                for (j, contribution) in occurrence.contributions.iter().enumerate() {
                    // The target's owner enum is private; its data-only tagged codec
                    // is the same authoritative original-row correspondence.
                    let target = serde_json::to_value(&contribution.target)
                        .map_err(|error| contract(error.to_string()))?;
                    if target.get("kind").and_then(serde_json::Value::as_str) == Some("row")
                        && target.get("row").is_some_and(|row| {
                            serde_json::from_value::<SemanticId>(row.clone())
                                .is_ok_and(|row| row == member)
                        })
                    {
                        occurrences.push(ReviewSourceOccurrence {
                            instance: occurrence.instance,
                            ordinal: occurrence.ordinal,
                            formal: None,
                            output: Some(contribution.output),
                            scale: contribution.scale,
                            offset: None,
                            terms: terms(),
                            payload_path: format!("/instances/{i}/contributions/{j}"),
                        });
                    }
                }
            }
            if payload_paths.is_empty() && occurrences.is_empty() {
                continue;
            }
            result.push(ReviewSourceAttribution {
                fact,
                member,
                role: role.into(),
                nominal,
                declaration: lineage.map(|(_, l)| l.declaration),
                instance: lineage.map(|(_, l)| l.instance),
                display_path: lineage.map(|(_, l)| l.path.clone()),
                payload_paths,
                source_terms,
                occurrences,
                fields,
                native_pullback_unavailable: map.native_pullback_unavailable.clone(),
            });
        }
        Ok(result)
    }
}
impl ReviewContext {
    /// Attach an explicitly selected existing map under this context's retained allowance.
    pub fn attach_source_map(
        &mut self,
        step: usize,
        native: Option<(usize, Option<usize>)>,
        map: &super::super::ModelingSourceMapDocument,
    ) -> Result<(), WorkflowError> {
        self.document.attach_source_map(step, native, map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_map_attachment_preflights_complete_byte_array_extent_before_payload_encoding() {
        let identity = pse_ids::ContentHash::from_bytes([71; 32]);
        let member = SemanticId::from_bytes([72; 16]);
        let map: super::super::super::ModelingSourceMapDocument = serde_json::from_value(serde_json::json!({
            "version": 1, "source_identity": identity, "numerical_identity": identity,
            "scope": "static_eligible_alternatives", "rows": [], "columns": [], "fixed_inputs": [],
            "row_nominals": [], "variable_nominals": [], "objective_signs": [], "fields": [],
            "members": [{ "member": member, "declaration": member, "instance": member, "path": "x".repeat(65536) }],
            "templates": [], "instances": [], "native_pullback_unavailable": "no transformed native pullback"
        })).unwrap();
        let map_bytes = super::super::super::replay::encoded_extent(&map, 1 << 20).unwrap();
        let request = ReviewContextRequest {
            maximum_bytes: map_bytes + 2048,
            ..Default::default()
        };
        let mut context = builder(&request, Some(RunId::from_bytes([73; 16])), None).document;
        context.validate().unwrap();
        // The map JSON itself fits, but the retained fact's integer byte-array
        // representation does not. Refusal precedes to_vec and leaves no fact.
        let error = context.attach_source_map(0, None, &map).unwrap_err();
        assert!(error.to_string().contains("before encoding"));
        assert!(context.facts.is_empty());
        context.selection.maximum_bytes = 1 << 20;
        context.attach_source_map(0, None, &map).unwrap();
        context.validate().unwrap();
    }
    #[test]
    fn source_attribution_preserves_affine_aliases_terms_spans_and_real_payload_references() {
        let row = SemanticId::from_bytes([1; 16]);
        let variable = SemanticId::from_bytes([2; 16]);
        let instance = SemanticId::from_bytes([3; 16]);
        let declaration = SemanticId::from_bytes([4; 16]);
        let local = SemanticId::from_bytes([5; 16]);
        let body = pse_ids::ContentHash::from_bytes([6; 32]);
        let document_id = SemanticId::from_bytes([7; 16]);
        let value = serde_json::json!({"version":1,"source_identity":body,"numerical_identity":body,
            "scope":"static_eligible_alternatives","rows":[row],"columns":[variable],"fixed_inputs":[],
            "row_nominals":[[row,10.]],"variable_nominals":[[variable,20.]],"objective_signs":[],
            "fields":[{"declaration":declaration,"role":"equation","position":0,"document":document_id,"start":3,"end":12}],
            "members":[{"member":row,"declaration":declaration,"instance":instance,"path":"cell.e"}],"templates":[],
            "instances":[{"ordinal":0,"instance":instance,"body":body,"checked_members":[[local,row]],
                "slots":[{"formal":0,"member":variable,"scale":2.,"offset":273.15,"fixed":false},
                         {"formal":1,"member":variable,"scale":-1.,"offset":0.,"fixed":false}],
                "contributions":[{"output":0,"target":{"kind":"row","row":row},"scale":-3.}],"demands":[]}],
            "native_pullback_unavailable":"no transformed dual pullback"});
        let map: super::super::super::ModelingSourceMapDocument =
            serde_json::from_value(value.clone()).unwrap();
        map.validate().unwrap();
        let mut context = builder(
            &ReviewContextRequest::default(),
            Some(RunId::from_bytes([8; 16])),
            None,
        )
        .document;
        context.attach_source_map(0, None, &map).unwrap();
        let attributions = context
            .source_map_attribution("/steps/0", &[row, variable])
            .unwrap();
        assert_eq!(attributions.len(), 2);
        let row = attributions.iter().find(|a| a.role == "row").unwrap();
        assert_eq!(row.nominal, Some(10.));
        assert_eq!(row.occurrences[0].scale, -3.);
        assert_eq!(row.occurrences[0].terms[0].local, local);
        assert_eq!(row.fields[0]["start"], 3);
        let variable = attributions.iter().find(|a| a.role == "variable").unwrap();
        assert_eq!(variable.occurrences.len(), 2);
        assert_eq!(variable.occurrences[0].offset, Some(273.15));
        // Raw references point at real retained bytes. Decoded coordinates are
        // resolved only after decoding that payload, never appended to bytes.
        let encoded = &context.facts[row.fact].payload;
        let decoded: serde_json::Value = serde_json::from_slice(encoded).unwrap();
        for path in &row.payload_paths {
            assert!(decoded.pointer(path).is_some());
        }
        assert!(decoded.pointer(&row.occurrences[0].payload_path).is_some());
        assert!(
            context
                .source_map_attribution("/steps/1", &[variable.member])
                .unwrap()
                .is_empty()
        );
        assert!(
            context
                .source_map_attribution("/steps/0/native/1/None", &[variable.member])
                .unwrap()
                .is_empty()
        );
        assert!(context.attach_source_map(0, None, &map).is_err());
        let mut invalid = value;
        invalid["instances"][0]["slots"][0]["member"] =
            serde_json::to_value(SemanticId::from_bytes([99; 16])).unwrap();
        let invalid: super::super::super::ModelingSourceMapDocument =
            serde_json::from_value(invalid).unwrap();
        assert!(invalid.validate().is_err());
    }
}
