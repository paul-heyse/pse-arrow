// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Closed bootstrap admission until every exact native index is ready.

use super::{CanonicalError, CanonicalStore, MetadataResponse, bounded_query, study_module, wire};
use crate::canonical_script::{Envelope, GuardedScript};
use surrealdb_types::Value;

fn invalid(message: &str) -> CanonicalError {
    CanonicalError::Configuration(format!("canonical bootstrap: {message}"))
}

/// Missing/failed/changed definitions are errors, never evidence of readiness.
fn indexes_ready(rows: Value) -> Result<bool, CanonicalError> {
    let Value::Array(rows) = rows else {
        return Err(invalid("index inventory is absent"));
    };
    if rows.len() != wire::BOOTSTRAP_INDEX_DEFINITIONS.len() {
        return Err(invalid("index inventory is incomplete"));
    }
    let mut ready = true;
    for (row, expected) in rows.into_iter().zip(wire::BOOTSTRAP_INDEX_DEFINITIONS) {
        let Value::Object(row) = row else {
            return Err(invalid("index observation is malformed"));
        };
        if row.get("definition") != Some(&Value::String((*expected).into())) {
            return Err(invalid("required index definition differs"));
        }
        let Some(Value::Object(building)) = row.get("building") else {
            return Err(invalid("index build status is absent"));
        };
        let Some(Value::String(status)) = building.get("status") else {
            return Err(invalid("index build status is malformed"));
        };
        match status.as_str() {
            "ready" => {}
            "started" | "cleaning" | "indexing" => ready = false,
            _ => {
                return Err(invalid(
                    "required index build failed or has unsupported status",
                ));
            }
        }
    }
    Ok(ready)
}

impl CanonicalStore {
    pub(super) async fn complete_bootstrap(&self) -> Result<(), CanonicalError> {
        let receipt = study_module::StudyModuleReceipt::read(&self.state_path)?;
        let mut original_marker = None;
        let mut original_custody = None;
        loop {
            let script = GuardedScript::new(Envelope::BootstrapObservation)
                .returning::<Value>("", wire::BOOTSTRAP_OBSERVATION)?;
            let slot = script.result();
            let mut response = bounded_query(
                self.activation_db
                    .metadata_script(MetadataResponse::Bootstrap, script),
            )
            .await?;
            let Value::Object(mut observation) = slot.take(&mut response)? else {
                return Err(invalid("bootstrap observation is absent"));
            };
            let Some(Value::Object(marker)) = observation.remove("marker") else {
                return Err(invalid("installation marker is absent"));
            };
            let decoded = wire::decode_canonical_interpretations(marker.clone())?;
            if decoded.interpretation != wire::INTERPRETATION
                || decoded.schema_digest != wire::SCHEMA_DIGEST
                || decoded.study_module_sha256 != receipt.sha256
                || decoded.payload_incarnation != self.files.incarnation
                || decoded.schema_version != pse_schema::catalog::substrate::SCHEMA_VERSION
                || uuid::Uuid::parse_str(&decoded.issuing_store).is_err()
                || uuid::Uuid::parse_str(&decoded.schema_epoch).is_err()
            {
                return Err(invalid(
                    "installation identity differs from selected target",
                ));
            }
            let mut normalized_marker = marker.clone();
            normalized_marker.insert("admission_open", Value::Bool(false));
            if let Some(original) = &original_marker {
                if original != &normalized_marker {
                    return Err(invalid("original physical installation identity changed"));
                }
            } else {
                original_marker = Some(normalized_marker);
            }
            let bootstrap = observation
                .remove("bootstrap")
                .ok_or_else(|| invalid("bootstrap custody is absent"))?;
            if decoded.admission_open {
                if bootstrap != Value::None {
                    return Err(invalid("open catalog still holds bootstrap custody"));
                }
            } else {
                let Value::Object(guard) = &bootstrap else {
                    return Err(invalid("closed catalog has no bootstrap custody"));
                };
                if guard.get("incarnation") != marker.get("schema_epoch")
                    || observation.get("pending") != Some(&Value::None)
                {
                    return Err(invalid(
                        "closed catalog belongs to another physical transition",
                    ));
                }
                if let Some(original) = &original_custody {
                    if original != &bootstrap {
                        return Err(invalid("original bootstrap custody changed"));
                    }
                } else {
                    original_custody = Some(bootstrap.clone());
                }
            }
            if !indexes_ready(
                observation
                    .remove("indexes")
                    .ok_or_else(|| invalid("index inventory is absent"))?,
            )? {
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                continue;
            }
            if decoded.admission_open {
                return Ok(());
            }
            // Repeat the exact physical observations in the publication transaction,
            // and compare the original complete marker/custody, not a new identity.
            let body = format!(
                "LET $marker=SELECT * FROM ONLY canonical_interpretations:current FOR UPDATE; LET $bootstrap=SELECT * FROM ONLY canonical_guards:bootstrap FOR UPDATE; IF $marker!=$expected_marker OR $bootstrap!=$expected_bootstrap {{THROW 'PSE_REFUSAL:activation_predecessor_changed';}}; LET $checked=({}); IF $checked.pending!=NONE OR array::len($checked.indexes)!=array::len($definitions) OR ![$checked.indexes,$definitions].transpose().all(|$pair|$pair[0].definition=$pair[1] AND $pair[0].building.status='ready') {{THROW 'PSE_REFUSAL:activation_predecessor_changed';}}; {} IF !({}) {{THROW 'PSE_REFUSAL:study_readiness_module_identity_changed';}}; UPDATE ONLY canonical_interpretations:current SET admission_open=true; DELETE ONLY canonical_guards:bootstrap;",
                wire::BOOTSTRAP_OBSERVATION,
                study_module::capability_validation_body()?,
                study_module::capability_validation_predicate()
            );
            let publication = super::protected_query("canonical::publish_bootstrap", || {
                let script = GuardedScript::new(Envelope::BootstrapPublication)
                    .returning::<Value>(&body, "NONE")?;
                Ok(self
                    .activation_db
                    .metadata_script(MetadataResponse::Header, script)
                    .bind(("expected_marker", marker.clone()))
                    .bind(("expected_bootstrap", bootstrap.clone()))
                    .bind(("definitions", wire::BOOTSTRAP_INDEX_DEFINITIONS.to_vec()))
                    .bind(("bucket", receipt.bucket()))
                    .bind(("backend", receipt.backend()?))
                    .bind(receipt.catalog_bindings()))
            })
            .await;
            let mut response = match publication {
                Ok(response) => response,
                // Another initializer may have published this same installation.
                // Reconcile its exact original identity and physical readiness;
                // never replay a transport outcome whose effect is unknown.
                Err(CanonicalError::Refused {
                    code: crate::canonical::RefusalCode::ActivationPredecessorChanged,
                    ..
                }) => continue,
                Err(error) => return Err(error),
            };
            crate::canonical_script::ResultSlot::<Value>::returned().take(&mut response)?;
            return Ok(());
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unreachable,
    reason = "bootstrap controls require exact native observations"
)]
mod tests {
    use super::*;
    use surrealdb_types::Object;
    fn ready_rows() -> Value {
        Value::Array(
            wire::BOOTSTRAP_INDEX_DEFINITIONS
                .iter()
                .map(|definition| {
                    let mut row = Object::new();
                    row.insert("definition", Value::String((*definition).into()));
                    let mut building = Object::new();
                    building.insert("status", Value::String("ready".into()));
                    row.insert("building", Value::Object(building));
                    Value::Object(row)
                })
                .collect::<Vec<_>>()
                .into(),
        )
    }
    #[test]
    fn bootstrap_exact_observation_has_native_readonly_syntax() {
        let script = GuardedScript::new(Envelope::BootstrapObservation)
            .returning::<Object>("", wire::BOOTSTRAP_OBSERVATION)
            .unwrap();
        assert!(script.schema_policy().1);
        let sql = script.into_parts().0;
        assert!(sql.starts_with("BEGIN READ ONLY ISOLATION LEVEL SNAPSHOT;"));
        surrealdb_expr::parser::parse(&sql).unwrap();
    }
    #[test]
    fn bootstrap_requires_every_exact_index_and_explicit_ready_state() {
        assert!(indexes_ready(ready_rows()).unwrap());
        assert!(indexes_ready(Value::None).is_err());
        assert!(indexes_ready(Value::Array(Vec::<Value>::new().into())).is_err());
        for status in ["error", "aborted", "unknown"] {
            let Value::Array(mut rows) = ready_rows() else {
                unreachable!()
            };
            let Value::Object(row) = &mut rows[0] else {
                unreachable!()
            };
            let Value::Object(building) = row.get_mut("building").unwrap() else {
                unreachable!()
            };
            building.insert("status", Value::String(status.into()));
            assert!(indexes_ready(Value::Array(rows)).is_err());
        }
        let Value::Array(mut rows) = ready_rows() else {
            unreachable!()
        };
        let Value::Object(row) = &mut rows[0] else {
            unreachable!()
        };
        row.insert(
            "definition",
            Value::String(
                "DEFINE INDEX file_intent ON canonical_file_instances FIELDS owner".into(),
            ),
        );
        assert!(indexes_ready(Value::Array(rows)).is_err());
        let Value::Array(mut rows) = ready_rows() else {
            unreachable!()
        };
        let Value::Object(row) = &mut rows[0] else {
            unreachable!()
        };
        row.remove("building");
        row.insert(
            "definition",
            Value::String(wire::BOOTSTRAP_INDEX_DEFINITIONS[0].into()),
        );
        assert!(indexes_ready(Value::Array(rows)).is_err());
    }
    #[test]
    fn bootstrap_pending_state_cannot_open_admission() {
        for status in ["started", "cleaning", "indexing"] {
            let Value::Array(mut rows) = ready_rows() else {
                unreachable!()
            };
            let Value::Object(row) = &mut rows[0] else {
                unreachable!()
            };
            let Value::Object(building) = row.get_mut("building").unwrap() else {
                unreachable!()
            };
            building.insert("status", Value::String(status.into()));
            assert!(!indexes_ready(Value::Array(rows)).unwrap());
        }
    }
    async fn closed_fixture() -> CanonicalStore {
        let state = std::env::var("PSE_SURREAL_STATE").expect("explicit isolated server fixture");
        let mut options =
            super::super::CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap();
        options.database = format!("canonical_test_bootstrap_{}", uuid::Uuid::new_v4().simple());
        let store = crate::native_fixture::canonical_fixture_with_options(&options, false).unwrap();
        store.provision_context().await.unwrap();
        super::super::complete_response(store.submit_initialization().await.unwrap()).unwrap();
        store
    }
    #[tokio::test]
    async fn closed_bootstrap_resumes_same_physical_identity_before_admitting_clients() {
        let store = closed_fixture().await;
        let before: Option<Object> = super::super::request(store.db.schema_marker())
            .await
            .unwrap()
            .take(0)
            .unwrap();
        let before = before.unwrap();
        assert_eq!(before.get("admission_open"), Some(&Value::Bool(false)));
        assert!(store.open().await.is_err());
        let (first, second) = tokio::join!(store.create(), store.create());
        first.unwrap();
        second.unwrap();
        let after: Option<Object> = super::super::request(store.db.schema_marker())
            .await
            .unwrap()
            .take(0)
            .unwrap();
        let mut expected = before;
        expected.insert("admission_open", Value::Bool(true));
        assert_eq!(after, Some(expected));
        store.create().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn bootstrap_missing_index_refuses_without_opening_admission() {
        let store = closed_fixture().await;
        bounded_query(
            store.activation_db.script(
                GuardedScript::new(Envelope::BootstrapPublication)
                    .returning::<Value>(
                        "REMOVE INDEX file_intent ON canonical_file_instances;",
                        "NONE",
                    )
                    .unwrap(),
            ),
        )
        .await
        .unwrap();
        assert!(store.create().await.is_err());
        let marker: Option<Object> = super::super::request(store.db.schema_marker())
            .await
            .unwrap()
            .take(0)
            .unwrap();
        assert_eq!(
            marker.unwrap().get("admission_open"),
            Some(&Value::Bool(false))
        );
        assert!(store.open().await.is_err());
        store.remove_isolated_fixture().await.unwrap();
    }
}
