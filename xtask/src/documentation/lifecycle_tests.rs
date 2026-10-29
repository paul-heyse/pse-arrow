// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;

struct Fixture{directory:tempfile::TempDir}
impl Fixture{
    fn new()->Self{let directory=tempfile::tempdir().unwrap();fs::create_dir_all(directory.path().join("docs/plans")).unwrap();Self{directory}}
    fn root(&self)->&Path{self.directory.path()}
    fn write(&self,path:&str,text:&str)->std::path::PathBuf{let path=self.root().join(path);fs::create_dir_all(path.parent().unwrap()).unwrap();fs::write(&path,text).unwrap();path}
    fn plan(&self,number:u32,status:&str)->String{let path=format!("docs/plans/{number:02}-work.md");self.write(&path,&format!("---\nstatus: in-progress\n---\n\n## Work\n\n| Packet | Status |\n|---|---|\n| P1 | {status} |\n"));path}
}
fn binding(path:&str)->Binding{Binding{path:path.into(),name:"packets".into(),heading:"Work".into(),headers:vec!["Packet".into(),"Status".into()],kind:"packet".into(),id_column:0,id_pattern:"^(.+)$".into(),occurrence:None,status_column:Some(1),owner_column:None,status_profile:"native".into()}}
fn policy()->Policy{Policy{status_profiles:BTreeMap::from([("native".into(),StatusProfile{order:vec!["open".into(),"complete".into()],patterns:BTreeMap::from([("open".into(),vec!["(?i)pending|open".into()]),("complete".into(),vec!["(?i)^(done|disproved)".into()])])})]),..Policy::default()}}
fn config()->site::Configuration{site::Configuration{tools:BTreeMap::new(),publication:site::Publication::default(),collections:vec![site::Collection{title:"Plans".into(),root:"plans".into(),recursive:true,scope:"History".into()}],overrides:Vec::new(),asset_bundles:Vec::new()}}

#[test]
fn native_tables_preserve_cells_rows_and_ignore_fenced_examples(){
    let body="# Work\n\n```md\n| Fake | Status |\n|---|---|\n| X | done |\n```\n\n## Packets\n\n| Packet | Meaning | Status |\n|---|---|---|\n| P1 | `left\\|right` and [link](a.md) | Implemented; qualification pending |\n";let result=tables(body,true).unwrap();assert_eq!(result.len(),1);assert_eq!(result[0].heading,"Packets");assert_eq!(result[0].headers,["Packet","Meaning","Status"]);assert!(result[0].rows[0][1].contains("left|right"));assert!(result[0].rows[0][1].contains("[link](a.md)"));assert_eq!(body.lines().nth(result[0].row_lines[0]-1).unwrap(),"| P1 | `left\\|right` and [link](a.md) | Implemented; qualification pending |");
}
#[test]
fn table_arity_is_parser_owned_and_only_bound_tables_are_schema(){
    assert!(tables("## Work\n\n| Packet | Status |\n|---|---|\n| P1 | `a|b` | done |\n",true).unwrap_err().to_string().contains("arity"));
    let fixture=Fixture::new();let path="docs/plans/28-work.md";fixture.write(path,"## Explanation\n\n| A | B |\n|---|---|\n| valid abbreviated GFM row |\n\n## Work\n\n| Packet | Status |\n|---|---|\n| P1 | done |\n");let mut policy=policy();policy.scope_tables.push(binding(path));assert_eq!(aggregate(fixture.root(),&policy,None,None).unwrap().len(),1);policy.scope_tables[0].heading="Missing".into();assert!(aggregate(fixture.root(),&policy,None,None).unwrap_err().to_string().contains("exactly one"));
}
#[test]
fn mixed_disproof_and_remaining_scope_preserves_raw_status_and_physical_line(){
    let fixture=Fixture::new();let path=fixture.plan(28,"disproved as thrash; open as cost");let mut policy=policy();policy.scope_tables.push(binding(&path));let items=aggregate(fixture.root(),&policy,None,None).unwrap();assert_eq!(items[0].state,"open");assert_eq!(items[0].raw_status,"disproved as thrash; open as cost");let line=items[0].source_line.unwrap();assert!(fs::read_to_string(fixture.root().join(path)).unwrap().lines().nth(line-1).unwrap().contains("P1"));
}
#[test]
fn dependencies_do_not_transfer_status_but_explicit_ownership_does(){
    let fixture=Fixture::new();let first=fixture.plan(28,"Implemented; qualification pending");let second=fixture.plan(32,"done");let mut policy=policy();policy.scope_tables=vec![binding(&first),binding(&second)];policy.relationships.push(Relationship{source:format!("{second}::packets::P1"),kind:"depends-on".into(),target:format!("{first}::packets::P1")});let items=aggregate(fixture.root(),&policy,None,None).unwrap();assert_eq!(items.iter().map(|item|item.state.as_str()).collect::<Vec<_>>(),["open","complete"]);policy.relationships[0].kind="status-owned-by".into();let items=aggregate(fixture.root(),&policy,None,None).unwrap();assert_eq!(items[1].native_state.as_deref(),Some("complete"));assert_eq!(items[1].state,"open");assert_eq!(items[1].status_owner.as_deref(),Some(items[0].id.as_str()));policy.relationships.push(Relationship{source:format!("{first}::packets::P1"),kind:"status-owned-by".into(),target:format!("{second}::packets::P1")});assert!(aggregate(fixture.root(),&policy,None,None).unwrap_err().to_string().contains("cyclic"));
}
#[test]
fn selected_document_queries_expand_connected_native_owners_and_keep_unbound_unknown(){
    let fixture=Fixture::new();let first=fixture.plan(28,"pending");let second=fixture.plan(32,"done");fixture.write("docs/plans/33-unbound.md","# New owner\n");let mut policy=policy();policy.scope_tables=vec![binding(&first),binding(&second)];policy.relationships.push(Relationship{source:format!("{second}::packets::P1"),kind:"status-owned-by".into(),target:format!("{first}::packets::P1")});let selected=BTreeSet::from([second]);assert_eq!(aggregate(fixture.root(),&policy,Some(&selected),None).unwrap().len(),2);let all=aggregate(fixture.root(),&policy,None,None).unwrap();assert_eq!(all[2].kind,"unbound-document");assert_eq!(all[2].state,"unknown");assert!(all[2].status_owner.is_none());
}
#[test]
fn ambiguous_selectors_duplicate_ids_bad_columns_and_captured_sources_are_checked(){
    let fixture=Fixture::new();let path=fixture.plan(28,"done");let original=fs::read_to_string(fixture.root().join(&path)).unwrap();fixture.write(&path,&format!("{original}\n{original}"));let mut policy=policy();policy.scope_tables.push(binding(&path));assert!(aggregate(fixture.root(),&policy,None,None).unwrap_err().to_string().contains("exactly one"));policy.scope_tables[0].occurrence=Some(1);assert_eq!(aggregate(fixture.root(),&policy,None,None).unwrap().len(),1);fixture.write(&path,&original.replace("| P1 | done |","| P1 | done |\n| P1 | done |"));assert!(aggregate(fixture.root(),&policy,None,None).unwrap_err().to_string().contains("duplicate native"));policy.scope_tables[0].id_column=99;assert!(aggregate(fixture.root(),&policy,None,None).unwrap_err().to_string().contains("outside table"));policy.scope_tables[0].id_column=0;let reader=|selected:&str|->Result<Vec<u8>>{ensure!(selected==path,"wrong captured source");Ok(original.as_bytes().to_vec())};assert_eq!(aggregate(fixture.root(),&policy,None,Some(&reader)).unwrap().len(),1);
}
#[test]
fn metadata_defaults_aliases_and_exact_exceptions_preserve_precedence(){
    let fixture=Fixture::new();let path=fixture.write("docs/plans/28-work.md","# Owner\n");let mut policy=Policy{topics:vec!["work".into()],defaults:BTreeMap::from([("doc_retention".into(),Value::String("protected-unknown".into()))]),collections:BTreeMap::from([("plans".into(),BTreeMap::from([("doc_role".into(),Value::String("plan".into()))]))]),bundles:vec![Bundle{root:"docs/plans".into(),defaults:BTreeMap::from([("doc_topics".into(),Value::Sequence(vec![Value::String("work".into())]))])}],..Policy::default()};
    let raw=b"---\ndisposition_owner: 28-work.md\ndoc_retention: while-dependent\n---\n";let document=metadata::read(raw,"fixture",false,None,fixture.root()).unwrap();let result=interpreted(fixture.root(),&path,&document,&policy,&config()).unwrap();assert_eq!(result["doc_owner"].as_str(),Some("docs/plans/28-work.md"));assert_eq!(result["doc_retention"].as_str(),Some("while-dependent"));assert_eq!(result["metadata_origins"].as_mapping().unwrap()["doc_owner"].as_str(),Some("authored metadata"));policy.exceptions.push(Exception{path:"docs/plans/28-work.md".into(),values:BTreeMap::from([("doc_retention".into(),Value::String("while-current".into()))])});assert!(interpreted(fixture.root(),&path,&document,&policy,&config()).unwrap_err().to_string().contains("conflicting explicit"));policy.exceptions.clear();policy.bundles.push(policy.bundles[0].clone());assert!(interpreted(fixture.root(),&path,&document,&policy,&config()).unwrap_err().to_string().contains("ambiguous bundle"));
}
#[test]
fn historical_owner_is_not_current_authority_and_reference_anchors_are_checked(){
    let fixture=Fixture::new();let path=fixture.write("review.md","# Review\n");let source=format!("---\ndisposition_owner: git:{}:docs/plans/27-old.md\n---\n","a".repeat(40));let document=metadata::read(source.as_bytes(),"review",false,None,fixture.root()).unwrap();let result=interpreted(fixture.root(),&path,&document,&Policy::default(),&site::Configuration{collections:Vec::new(),..config()}).unwrap();assert!(!result.contains_key("doc_owner"));assert_eq!(result["doc_retention"].as_str(),Some("protected-unknown"));assert!(result.contains_key("historical_owner"));fixture.write("docs/owner.md","# Owner\n\n## 1. Contract\n\n<a id=\"explicit\"></a>\n");assert_eq!(reference(fixture.root(),"docs/owner.md#section-1",None).unwrap(),"docs/owner.md#section-1");assert!(reference(fixture.root(),"docs/owner.md#explicit",None).is_ok());assert!(reference(fixture.root(),"docs/owner.md#missing",None).is_err());assert!(reference(fixture.root(),"../outside.md",None).is_err());
}
#[test]
fn policy_validation_and_inventory_retirement_keep_authorization_separate(){
    let fixture=Fixture::new();let path=fixture.plan(28,"done");let raw=fs::read_to_string(fixture.root().join(&path)).unwrap();fixture.write(&path,&raw.replacen("## Work","# Work",1));fixture.write("docs/site.toml",r#"[publication]
entrypoints=["plans/28-work.md"]
[[collections]]
title="Plans"
root="plans"
scope="Current"
"#);fixture.write("docs/lifecycle.toml",r#"version=1
topics=["work"]
[defaults]
doc_role="plan"
doc_topics=["work"]
"#);let policy=load_policy(fixture.root(),None).unwrap();let result=snapshot(fixture.root(),Operation::RetirePlan,&policy).unwrap();assert_eq!(result["deletion_authorized"],false);assert_eq!(result["documents"][0]["path"],path);assert_eq!(result["documents"][0]["doc_retention"],"protected-unknown");assert!(result["candidates"][0]["blockers"].as_array().unwrap().iter().any(|value|value=="owner release not established"));fixture.write("docs/lifecycle.toml","version=1\ntopics=['work','work']\n");assert!(load_policy(fixture.root(),None).unwrap_err().to_string().contains("unique declared"));
}
