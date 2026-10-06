// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Storage-independent scientific product completeness.
use super::declarations::{column,relation};
use crate::{RegistryBuilder,model::{FieldContract as T,Namespace as N,SnapshotClass as S}};
use std::collections::BTreeSet;

pub(super) fn declare(builder:&mut RegistryBuilder) {
    builder.declare_artifact_profile("relations",BTreeSet::new());
    relation(builder,N::Reference,"artifact_profiles",S::Model,&["kind"],vec![column("kind",T::native(arrow_schema::DataType::Utf8)),column("required_relations",T::list(T::id()))],"Declared scientific product completeness, including selected empty relations.");
}
pub(super) fn declare_profiles(builder:&mut RegistryBuilder) {
    let common=BTreeSet::from(["runtime.diagnostics_findings".to_owned(),"provenance.derivations".to_owned()]);
    builder.declare_artifact_profile("run",BTreeSet::from(["runtime.run_lineage".to_owned(),"runtime.candidate_assessments".to_owned(),"runtime.modeling_checks".to_owned()]));
    let mut source=common.clone();
    source.extend(builder.declared_relations().iter().filter(|relation|matches!(relation.authority,crate::model::Authority::Authored|crate::model::Authority::Reference)&&relation.snapshot_class!=S::Sidecar).map(|relation|relation.key.qualified_name()));
    builder.declare_artifact_profile("source",source.clone());
    builder.declare_artifact_profile("case",source);
    builder.declare_artifact_profile("diagnostics",common);
    builder.declare_artifact_profile("inspection",BTreeSet::new());
}
