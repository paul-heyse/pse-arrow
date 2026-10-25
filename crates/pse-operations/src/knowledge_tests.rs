// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
fn parsed<T: std::str::FromStr>(value: &str) -> T
where
    T::Err: std::fmt::Debug,
{
    value.parse().unwrap()
}
fn subject(id: &str, kind: &str, source: &str, role: &str) -> KnowledgeSubject {
    KnowledgeSubject {
        id: id.into(),
        kind: parsed(kind),
        owner: "fixture-owner".into(),
        native_id: id.into(),
        label: id.into(),
        basis: "declared".into(),
        native_state: None,
        state_basis: None,
        publication_scope: None,
        document_role: None,
        native_version: Some(digest(id)),
        evidence: None,
        sources: vec![SubjectSource {
            source_id: source.into(),
            role: parsed(role),
        }],
        topics: Vec::new(),
    }
}
pub(crate) fn corpus() -> KnowledgeCorpus {
    let sources = ["docs", "code", "evidence"]
        .into_iter()
        .map(|id| KnowledgeSource {
            id: id.into(),
            kind: "fixture-owner-source".into(),
            locator: format!("owner:{id}"),
            version_digest: digest(id),
            basis: "declared".into(),
            title: id.into(),
            status: "current".into(),
        })
        .collect();
    let mut document = subject("contract-page", "document", "docs", "metadata");
    document.document_role = Some(parsed("contract"));
    document.publication_scope = Some("Current".into());
    let mut contract = subject("blueprint §23.2", "contract", "docs", "metadata");
    contract.document_role = Some(parsed("contract"));
    contract.publication_scope = Some("Current".into());
    contract.topics = vec![
        SubjectTopic {
            topic: "boundary".into(),
            source_id: "docs".into(),
        },
        SubjectTopic {
            topic: "validation".into(),
            source_id: "docs".into(),
        },
    ];
    let mut decision = subject("ADR-fixture", "decision", "docs", "metadata");
    decision.document_role = Some(parsed("decision"));
    decision.publication_scope = Some("Current".into());
    decision.native_state = Some("proposed".into());
    decision.state_basis = Some("original ADR front matter".into());
    let mut package = subject(
        "plan::packages::A01",
        "plan_package",
        "docs",
        "native_scope",
    );
    package.document_role = Some(parsed("plan"));
    package.native_state = Some("planned".into());
    package.state_basis = Some("bound native status column".into());
    package.sources.push(SubjectSource {
        source_id: "docs".into(),
        role: parsed("metadata"),
    });
    let mut dependency = package.clone();
    dependency.id = "plan::packages::A02".into();
    dependency.native_id = dependency.id.clone();
    dependency.label = "dependency".into();
    let mut question = subject(
        "question-original",
        "research_question",
        "evidence",
        "question_input",
    );
    question.basis = "user-question".into();
    let mut experiment = subject(
        "experiment-original",
        "experiment",
        "evidence",
        "owner_evidence",
    );
    experiment.basis = "native".into();
    let mut diagnostic = subject(
        "diagnostic-original",
        "diagnostic",
        "evidence",
        "owner_evidence",
    );
    diagnostic.basis = "native".into();
    let mut reference = subject(
        "evidence-original",
        "evidence_reference",
        "evidence",
        "owner_evidence",
    );
    reference.basis = "native".into();
    reference.evidence = Some(EvidenceIdentity {
        authority: "original-fixture-owner".into(),
        codec: "fixture-v1".into(),
        source_id: "original-source".into(),
        version: digest("original-source"),
        member: "original-member".into(),
    });
    let mut subjects = vec![
        document, contract, decision, package, dependency, question, experiment, diagnostic,
        reference,
    ];
    for (id, kind) in [
        ("decl-a", "declaration"),
        ("decl-b", "declaration"),
        ("generator-shared", "generator"),
        ("output-a", "generated_artifact"),
        ("output-b", "generated_artifact"),
    ] {
        subjects.push(subject(id, kind, "code", "producer_declaration"));
    }
    let edge = |id: &str, from: &str, to: &str, relation: &str| KnowledgeEdge {
        id: id.into(),
        from: from.into(),
        to: to.into(),
        relation: parsed(relation),
        basis: "declared".into(),
    };
    KnowledgeCorpus {
        version: 2,
        expected_active: None,
        sources,
        subjects,
        excerpts: vec![
            KnowledgeExcerpt {
                id: "contract-excerpt".into(),
                source_id: "docs".into(),
                subject_id: Some("blueprint §23.2".into()),
                text: "Boundary validation rejects invalid native values".into(),
                locator: "owner:docs#23.2".into(),
                basis: "text".into(),
            },
            KnowledgeExcerpt {
                id: "decision-excerpt".into(),
                source_id: "docs".into(),
                subject_id: Some("ADR-fixture".into()),
                text: "Recycle iteration proposal remains proposed".into(),
                locator: "owner:docs#decision".into(),
                basis: "text".into(),
            },
            KnowledgeExcerpt {
                id: "unattributed-excerpt".into(),
                source_id: "docs".into(),
                subject_id: None,
                text:
                    "Recycle in a shared file does not attribute this text to its contract section"
                        .into(),
                locator: "owner:docs#other".into(),
                basis: "text".into(),
            },
        ],
        edges: vec![
            edge(
                "contains-section",
                "contract-page",
                "blueprint §23.2",
                "contains",
            ),
            edge(
                "governs-package",
                "blueprint §23.2",
                "plan::packages::A01",
                "governs",
            ),
            edge(
                "depends",
                "plan::packages::A01",
                "plan::packages::A02",
                "depends_on",
            ),
            edge(
                "supports-contract",
                "evidence-original",
                "blueprint §23.2",
                "supports",
            ),
            edge(
                "answers-question",
                "experiment-original",
                "question-original",
                "answers",
            ),
            edge(
                "diagnoses-experiment",
                "diagnostic-original",
                "experiment-original",
                "diagnoses",
            ),
            edge(
                "question-reference",
                "question-original",
                "evidence-original",
                "references",
            ),
        ],
        productions: vec![
            KnowledgeProduction {
                id: "production-a".into(),
                declaration_id: "decl-a".into(),
                generator_id: "generator-shared".into(),
                artifact_id: "output-a".into(),
                direction: None,
                basis: "declared".into(),
            },
            KnowledgeProduction {
                id: "production-b".into(),
                declaration_id: "decl-b".into(),
                generator_id: "generator-shared".into(),
                artifact_id: "output-b".into(),
                direction: None,
                basis: "declared".into(),
            },
        ],
    }
}
fn query(mode: KnowledgeMode, generation: Option<String>) -> KnowledgeQuery {
    KnowledgeQuery {
        version: pse_model::document::Version,
        generation,
        mode,
        kind: None,
        owner: None,
        native_id: None,
        document_role: None,
        native_state: None,
        publication_scope: None,
        topics: Vec::new(),
        subject_id: None,
        relation: None,
        declaration_id: None,
        generator_id: None,
        artifact_id: None,
        text: None,
        limit: 64,
        cursor: None,
    }
}
#[test]
fn knowledge_query_checks_version_before_body_and_preserves_predicate() {
    let example =
        KnowledgeQuery::decode_json(&serde_json::to_vec(&knowledge_query_example()).unwrap())
            .unwrap();
    example.validate().unwrap();
    let request = query(KnowledgeMode::Productions, None);
    let mut value = serde_json::to_value(&request).unwrap();
    assert_eq!(value["version"], 1);
    let decoded = KnowledgeQuery::decode_json(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(decoded.mode, KnowledgeMode::Productions);
    value["version"] = serde_json::json!(2);
    value["mode"] = serde_json::json!({"malformed":"current body"});
    let error = KnowledgeQuery::decode_json(&serde_json::to_vec(&value).unwrap()).unwrap_err();
    assert_eq!(error.location().stage, pse_model::document::DecodeStage::Header);
    assert!(KnowledgeQuery::decode_json(br#"{"mode":"subjects"}"#).is_err());
    assert!(KnowledgeQuery::decode_json(br#"{"version":1,"version":1}"#).is_err());
}
#[cfg(feature = "canonical-tests")]
async fn rejects_native_row(
    importer: &EvidenceStore,
    table: &str,
    mut row: Object,
    label: &str,
) -> Result<(), EvidenceError> {
    row.insert("key", Value::String(digest(label)));
    // Use fresh logical identities so a uniqueness refusal cannot masquerade as
    // the native applicability/reference assertion under examination.
    for field in match table {
        "knowledge_subjects" => &["subject_id", "native_id"][..],
        "knowledge_subject_topics" => &["topic"][..],
        "knowledge_excerpts" => &["excerpt_id"][..],
        "knowledge_subject_edges" => &["edge_id"][..],
        "knowledge_productions" => &["production_id"][..],
        _ => &[][..],
    } {
        row.insert(*field, Value::String(label.into()));
    }
    let statement = if declaration::ENFORCED_RELATIONS.contains(&table) {
        format!(
            "INSERT RELATION INTO {table} object::extend($row,{{id:type::record('{table}',$row.key)}});"
        )
    } else {
        format!("CREATE type::record('{table}',$row.key) CONTENT $row;")
    };
    assert!(
        canonical::bounded_query(importer.query(statement)?.bind(("row", row)))
            .await
            .is_err(),
        "native schema must reject {label}"
    );
    let mut response = canonical::bounded_query(
        importer
            .query(format!("SELECT * FROM ONLY type::record('{table}',$key);"))?
            .bind(("key", digest(label))),
    )
    .await?;
    assert!(
        response.take::<Option<Object>>(0)?.is_none(),
        "rejected {label} must not create a row"
    );
    Ok(())
}
#[cfg(feature = "canonical-tests")]
fn prepared_object(prepared: &Prepared, table: &str) -> Object {
    let Value::Object(row) = &prepared.rows[table][0] else {
        panic!("prepared typed object")
    };
    row.clone()
}
#[test]
fn knowledge_corpus_exact_membership_origin_and_identity() {
    let original = corpus();
    original.validate().unwrap();
    let identity = original.generation().unwrap();
    let mut reversed = original.clone();
    reversed.sources.reverse();
    reversed.subjects.reverse();
    reversed.edges.reverse();
    reversed.productions.reverse();
    reversed.excerpts.reverse();
    for subject in &mut reversed.subjects {
        subject.sources.reverse();
        subject.topics.reverse();
    }
    reversed.expected_active = Some("a".repeat(64));
    assert_eq!(identity, reversed.generation().unwrap());
    let mut wrong = original.clone();
    wrong
        .subjects
        .iter_mut()
        .find(|subject| subject.id == "question-original")
        .unwrap()
        .native_state = Some("accepted".into());
    assert!(wrong.validate().is_err());
    let mut wrong = original.clone();
    wrong.edges[0].from = "generator-shared".into();
    assert!(
        wrong.validate().is_err(),
        "closed relation applicability must be enforced"
    );
    let mut wrong = original.clone();
    wrong.productions[0].generator_id = "output-b".into();
    assert!(wrong.validate().is_err());
    let mut wrong = original.clone();
    wrong
        .subjects
        .iter_mut()
        .find(|subject| subject.id == "blueprint §23.2")
        .unwrap()
        .topics[0]
        .source_id = "code".into();
    assert!(
        wrong.validate().is_err(),
        "topics must retain native metadata attribution"
    );
    let mut wrong = original.clone();
    wrong.excerpts[0].source_id = "code".into();
    assert!(
        wrong.validate().is_err(),
        "a shared carrier cannot invent subject correspondence"
    );
    let mut value = serde_json::to_value(original).unwrap();
    value["unknown"] = true.into();
    assert!(serde_json::from_value::<KnowledgeCorpus>(value).is_err());
}
#[test]
fn knowledge_query_refuses_unbounded_or_unstructured_text_and_mixed_modes() {
    let mut request = query(KnowledgeMode::Subjects, None);
    request.validate().unwrap();
    request.text = Some("boundary".into());
    assert!(
        request.validate().is_err(),
        "text requires an explicit structured population"
    );
    request.kind = Some(parsed("contract"));
    request.validate().unwrap();
    request.limit = 65;
    assert!(request.validate().is_err());
    let mut request = query(KnowledgeMode::Productions, None);
    assert!(request.validate().is_err());
    request.artifact_id = Some("output-a".into());
    request.validate().unwrap();
    request.kind = Some(parsed("generator"));
    assert!(request.validate().is_err());
}
#[cfg(feature = "canonical-tests")]
pub(crate) async fn exercise_native(
    importer: &EvidenceStore,
    reader: &EvidenceStore,
) -> Result<(), EvidenceError> {
    let before = reader.status().await?;
    let original = tempfile::NamedTempFile::new().map_err(|cause| invalid(cause.to_string()))?;
    std::fs::write(original.path(), b"original owner bytes")
        .map_err(|cause| invalid(cause.to_string()))?;
    let mut first_corpus = corpus();
    first_corpus.sources[0].locator = format!("explicit-owner:{}", original.path().display());
    first_corpus.sources[0].version_digest = digest(b"original owner bytes");
    assert!(reader.knowledge_import(&first_corpus).await.is_err());
    assert!(reader.knowledge_reset().await.is_err());
    let staged = prepare(&first_corpus)?;
    let candidate = staged.rows["knowledge_sources"][0].clone();
    let _refusal = canonical::bounded_query(
        reader
            .query("CREATE type::record('knowledge_sources',$row.key) CONTENT $row;")?
            .bind(("row", candidate.clone())),
    )
    .await;
    let mut absence = canonical::bounded_query(
        importer
            .query("SELECT * FROM knowledge_sources WHERE key=$row.key;")?
            .bind(("row", candidate)),
    )
    .await?;
    assert!(absence.take::<Vec<Object>>(0)?.is_empty());
    importer.knowledge_stage(&staged).await?;
    assert!(reader.knowledge_status().await?["active_generation"].is_null());
    assert!(
        reader
            .knowledge_select(&query(
                KnowledgeMode::Subjects,
                Some(staged.generation.clone())
            ))
            .await
            .is_err()
    );
    let mut rival = first_corpus.clone();
    rival.subjects[0].label = "concurrent alternative".into();
    let rival_prepared = prepare(&rival)?;
    importer.knowledge_stage(&rival_prepared).await?;
    let mut wrong_kind = prepared_object(&staged, "knowledge_productions");
    let wrong_generator = member(&staged.generation, "output-b")?;
    wrong_kind.insert("generator_key", Value::String(wrong_generator.clone()));
    wrong_kind.insert(
        "generator_key_record",
        crate::canonical_codec::record_link("knowledge_subjects", &wrong_generator),
    );
    rejects_native_row(
        importer,
        "knowledge_productions",
        wrong_kind,
        "production wrong native subject kind",
    )
    .await?;
    let mut cross_cut = prepared_object(&staged, "knowledge_productions");
    let other_generator = member(&rival_prepared.generation, "generator-shared")?;
    cross_cut.insert("generator_key", Value::String(other_generator.clone()));
    cross_cut.insert(
        "generator_key_record",
        crate::canonical_codec::record_link("knowledge_subjects", &other_generator),
    );
    rejects_native_row(
        importer,
        "knowledge_productions",
        cross_cut,
        "production cross-cut native reference",
    )
    .await?;
    let mut dangling = prepared_object(&staged, "knowledge_subject_sources");
    let absent = member(&staged.generation, "absent-source")?;
    dangling.insert("source_key", Value::String(absent.clone()));
    dangling.insert(
        "source_key_record",
        crate::canonical_codec::record_link("knowledge_sources", &absent),
    );
    rejects_native_row(
        importer,
        "knowledge_subject_sources",
        dangling,
        "dangling native source reference",
    )
    .await?;
    let attributed = staged.rows["knowledge_excerpts"]
        .iter()
        .find_map(|value| match value {
            Value::Object(row) if matches!(row.get("subject_key"), Some(Value::String(_))) => {
                Some(row.clone())
            }
            _ => None,
        })
        .expect("explicit attributed excerpt");
    let mut missing_alias = attributed.clone();
    missing_alias.remove("subject_key_record");
    rejects_native_row(
        importer,
        "knowledge_excerpts",
        missing_alias,
        "omitted optional subject record alias",
    )
    .await?;
    let mut wrong_alias = attributed.clone();
    wrong_alias.insert(
        "subject_key_record",
        crate::canonical_codec::record_link(
            "knowledge_subjects",
            &member(&staged.generation, "contract-page")?,
        ),
    );
    rejects_native_row(
        importer,
        "knowledge_excerpts",
        wrong_alias,
        "mismatched optional subject record alias",
    )
    .await?;
    let wrong_source = member(&staged.generation, "code")?;
    let mut misattributed = attributed;
    misattributed.insert("source_key", Value::String(wrong_source.clone()));
    misattributed.insert(
        "source_key_record",
        crate::canonical_codec::record_link("knowledge_sources", &wrong_source),
    );
    rejects_native_row(
        importer,
        "knowledge_excerpts",
        misattributed,
        "existing source without subject attribution",
    )
    .await?;
    let mut wrong_topic = prepared_object(&staged, "knowledge_subject_topics");
    wrong_topic.insert("source_key", Value::String(wrong_source.clone()));
    wrong_topic.insert(
        "source_key_record",
        crate::canonical_codec::record_link("knowledge_sources", &wrong_source),
    );
    rejects_native_row(
        importer,
        "knowledge_subject_topics",
        wrong_topic,
        "topic source without subject metadata contribution",
    )
    .await?;
    let mut wrong_edge = prepared_object(&staged, "knowledge_subject_edges");
    let generator = member(&staged.generation, "generator-shared")?;
    let output = member(&staged.generation, "output-a")?;
    wrong_edge.insert("source_key", Value::String(generator.clone()));
    wrong_edge.insert(
        "in",
        crate::canonical_codec::record_link("knowledge_subjects", &generator),
    );
    wrong_edge.insert("target_key", Value::String(output.clone()));
    wrong_edge.insert(
        "out",
        crate::canonical_codec::record_link("knowledge_subjects", &output),
    );
    wrong_edge.insert("relation", Value::String("contains".into()));
    rejects_native_row(
        importer,
        "knowledge_subject_edges",
        wrong_edge,
        "relation kind and endpoint combination",
    )
    .await?;
    let mut wrong_subject = staged.rows["knowledge_subjects"]
        .iter()
        .find_map(|value| match value {
            Value::Object(row)
                if row.get("subject_id") == Some(&Value::String("ADR-fixture".into())) =>
            {
                Some(row.clone())
            }
            _ => None,
        })
        .expect("native decision fixture");
    wrong_subject.insert("kind", Value::String("generator".into()));
    rejects_native_row(
        importer,
        "knowledge_subjects",
        wrong_subject,
        "subject kind with inapplicable lifecycle and role",
    )
    .await?;
    let (left, right) = tokio::join!(
        importer.knowledge_activate(&first_corpus, &staged),
        importer.knowledge_activate(&rival, &rival_prepared)
    );
    let active = match (left, right) {
        (Ok(receipt), Err(_)) | (Err(_), Ok(receipt)) => receipt.generation,
        other => panic!("one native CAS activation must win: {other:?}"),
    };
    first_corpus.expected_active = Some(active);
    let first = importer.knowledge_import(&first_corpus).await?;
    importer
        .ack_fault
        .store(3, std::sync::atomic::Ordering::SeqCst);
    assert_eq!(
        first.generation,
        importer.knowledge_import(&first_corpus).await?.generation
    );
    let mut interrupted = first_corpus.clone();
    interrupted.expected_active = Some(first.generation.clone());
    interrupted.subjects[0].label = "interrupted staged generation".into();
    let incomplete = prepare(&interrupted)?;
    importer.knowledge_stage(&incomplete).await?;
    let omitted = prepared_object(&incomplete, "knowledge_excerpts");
    canonical::bounded_query(
        importer
            .query("DELETE type::record('knowledge_excerpts',$key);")?
            .bind(("key", omitted.get("key").cloned().expect("prepared key"))),
    )
    .await?;
    assert!(
        importer
            .knowledge_activate(&interrupted, &incomplete)
            .await
            .is_err(),
        "interrupted staging cannot publish an incomplete generation"
    );
    assert_eq!(
        reader.knowledge_status().await?["active_generation"],
        first.generation
    );
    let mut count=canonical::bounded_query(importer.query("SELECT count() AS n FROM knowledge_sources WHERE generation=$generation GROUP ALL;")?.bind(("generation",first.generation.clone()))).await?;
    let counts: Vec<Object> = count.take(0)?;
    assert_eq!(
        counts[0].get("n"),
        Some(&Value::Number(surrealdb::types::Number::Int(3)))
    );
    let base = query(KnowledgeMode::Subjects, Some(first.generation.clone()));
    let selected = KnowledgeQuery {
        kind: Some(parsed("contract")),
        owner: Some("fixture-owner".into()),
        native_id: Some("blueprint §23.2".into()),
        document_role: Some(parsed("contract")),
        publication_scope: Some("Current".into()),
        topics: vec!["boundary".into(), "validation".into()],
        ..base.clone()
    };
    let page = reader.knowledge_select(&selected).await?;
    assert_eq!(page.subjects.len(), 1);
    assert_eq!(page.subjects[0].id, "blueprint §23.2");
    assert_eq!(page.sources.len(), 3);
    let no_hits = reader
        .knowledge_select(&KnowledgeQuery {
            text: Some("recycle".into()),
            ..selected.clone()
        })
        .await?;
    assert!(
        no_hits.subjects.is_empty(),
        "other sections in the same file never create subject hits"
    );
    assert_eq!(no_hits.sources.len(), 3);
    let matched = reader
        .knowledge_select(&KnowledgeQuery {
            text: Some("boundary".into()),
            ..selected.clone()
        })
        .await?;
    assert_eq!(matched.subjects.len(), 1);
    assert_eq!(
        matched.excerpts[0].subject_id.as_deref(),
        Some("blueprint §23.2")
    );
    let proposal = reader
        .knowledge_select(&KnowledgeQuery {
            kind: Some(parsed("decision")),
            native_state: Some("proposed".into()),
            publication_scope: Some("Current".into()),
            ..base.clone()
        })
        .await?;
    assert_eq!(
        proposal.subjects.len(),
        1,
        "current publication does not mean accepted decision"
    );
    let graph_request = KnowledgeQuery {
        subject_id: Some("plan::packages::A01".into()),
        relation: Some(parsed("depends_on")),
        ..query(KnowledgeMode::Graph, Some(first.generation.clone()))
    };
    assert_eq!(
        reader.knowledge_select(&graph_request).await?.edges[0].to,
        "plan::packages::A02"
    );
    let production_request = KnowledgeQuery {
        generator_id: Some("generator-shared".into()),
        ..query(KnowledgeMode::Productions, Some(first.generation.clone()))
    };
    assert_eq!(
        reader
            .knowledge_select(&production_request)
            .await?
            .productions
            .len(),
        2
    );
    let false_pair = reader
        .knowledge_select(&KnowledgeQuery {
            declaration_id: Some("decl-a".into()),
            artifact_id: Some("output-b".into()),
            ..production_request.clone()
        })
        .await?;
    assert!(
        false_pair.productions.is_empty(),
        "shared generator never creates a declaration/output cross-product"
    );
    assert_eq!(false_pair.sources.len(), 3);
    for request in [&selected, &graph_request, &production_request] {
        let plan = reader.knowledge_explain(request).await?;
        assert!(
            !serde_json::from_str::<serde_json::Value>(&plan.raw_plan_json)
                .unwrap()
                .is_null()
        );
        eprintln!("knowledge structured native plan {}", plan.raw_plan_json);
    }
    let all = KnowledgeQuery {
        generation: None,
        limit: 1,
        ..base.clone()
    };
    let cursor = reader
        .knowledge_select(&all)
        .await?
        .next
        .expect("multiple exact subjects");
    std::fs::write(original.path(), b"modified source")
        .map_err(|cause| invalid(cause.to_string()))?;
    let mut later = first_corpus.clone();
    later.expected_active = Some(first.generation.clone());
    later.sources[0].version_digest = digest(b"modified source");
    later.sources[0].status = "stale".into();
    let second = importer.knowledge_import(&later).await?;
    assert!(
        reader
            .knowledge_select(&KnowledgeQuery {
                cursor: Some(cursor),
                ..all
            })
            .await
            .is_err()
    );
    importer.knowledge_import(&first_corpus).await?;
    assert_eq!(
        reader.knowledge_status().await?["active_generation"],
        second.generation
    );
    assert_eq!(
        reader
            .knowledge_select(&base)
            .await?
            .sources
            .iter()
            .find(|source| source.id == "docs")
            .unwrap()
            .status,
        "current"
    );
    for table in [
        "knowledge_subject_topics",
        "knowledge_excerpts",
        "knowledge_subject_edges",
        "knowledge_productions",
    ] {
        let mut current = later.clone();
        current.expected_active = reader.knowledge_status().await?["active_generation"]
            .as_str()
            .map(str::to_owned);
        current.subjects[0].label = format!("active corruption {table}");
        let receipt = importer.knowledge_import(&current).await?;
        let prepared = prepare(&current)?;
        importer.knowledge_reconcile(&prepared, true).await?;
        let Value::Object(row) = &prepared.rows[table][0] else {
            panic!("prepared typed object")
        };
        let Some(Value::String(key)) = row.get("key") else {
            panic!("prepared key")
        };
        canonical::bounded_query(
            importer
                .query(format!("DELETE type::record('{table}',$key);"))?
                .bind(("key", key.clone())),
        )
        .await?;
        assert!(
            importer.knowledge_import(&current).await.is_err(),
            "active {table} membership must be reconciled even after lost acknowledgement"
        );
        assert_eq!(
            reader.knowledge_status().await?["active_generation"],
            receipt.generation
        );
    }
    importer.knowledge_reset().await?;
    assert_eq!(reader.status().await?, before);
    assert_eq!(
        std::fs::read(original.path()).map_err(|cause| invalid(cause.to_string()))?,
        b"modified source"
    );
    let mut rebuilt = first_corpus.clone();
    rebuilt.expected_active = None;
    importer.knowledge_import(&rebuilt).await?;
    assert_eq!(reader.knowledge_select(&selected).await?.subjects.len(), 1);
    assert_eq!(reader.status().await?, before);
    Ok(())
}
