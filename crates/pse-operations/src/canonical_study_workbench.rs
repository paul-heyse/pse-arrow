// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Short native selections over compact study metadata, separate from worker readiness.
use crate::{
    canonical::transport::{original_deadline, within_clock},
    canonical::{CanonicalError, CanonicalStore, REQUEST_TIMEOUT, protected_query},
    canonical_codec as codec,
};
use surrealdb::types::{Bytes, Object, Value};
use std::sync::Arc;

/// Indexed predicates over actual original scientific and engineering owners.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct StudyWorkbenchFilter {
    /// Exact study key, including terminal studies.
    pub study: Option<String>,
    /// Exact owning project; branch membership is resolved by its native owner.
    pub project: Option<String>,
    /// Exact branch key; not its mutable display name.
    pub branch: Option<String>,
    /// Exact canonical modeling source revision.
    pub source: Option<String>,
    /// Exact admitted physical source revision.
    pub physical_revision: Option<String>,
    /// Exact immutable historical engineering context key.
    pub context: Option<String>,
    /// Original canonical terminal class, independent from scientific usability.
    pub outcome: Option<pse_model::generated::enums::CanonicalTerminalOutcome>,
    /// Dataset/selection keys select consumed ancestry; publication operation keys
    /// select the exact resulting parameter source. Existing ResearchReference keys
    /// select their explicitly named source/run neighborhood, without proving consumption.
    pub research_reference: Option<String>,
}
/// Compact immutable attribution and current header facts. No definitions or trajectories.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyWorkbenchHeader {
    /// Exact study identity.
    pub study: String,
    /// Exact parent execution.
    pub run: String,
    /// Original source owner.
    pub problem: String,
    /// Exact original modeling revision.
    pub source: String,
    /// Exact admitted physical revision, unavailable for historical untyped metadata.
    pub physical_revision: Option<String>,
    /// Exact physical source interpretation.
    pub physical_identity: Option<String>,
    /// Exact engineering context, if selected.
    pub context: Option<String>,
    /// Complete activated population size, before outcome filtering or analysis.
    pub point_count: u64,
    /// Actual cancellation authority generation; not a metadata revision.
    pub generation: u64,
    /// Original complete membership is activated.
    pub active: bool,
    /// Actual persistent cancellation flag.
    pub cancelled: bool,
    /// Parent summary concluded.
    pub terminal: bool,
    /// Exact terminal parent attempt, when admitted.
    pub result_attempt: Option<String>,
    /// Original operational outcome, not scientific qualification.
    pub outcome: Option<pse_model::generated::enums::CanonicalTerminalOutcome>,
}
/// One frozen point's compact original facts; no operation, report or payload is decoded.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyCohortMember {
    /// Exact occurrence key.
    pub key: String,
    /// Exact owning study.
    pub study: String,
    /// Actual immutable member position.
    pub ordinal: u64,
    /// Exact original run.
    pub run: String,
    /// Exact currently assigned attempt; retries remain separate sources.
    pub attempt: Option<String>,
    /// Exact captured closed manifest; later completion cannot reclassify absence.
    pub manifest: Option<String>,
    /// Actual native point metadata revision captured in this selection.
    pub metadata_revision: u64,
    /// Whether the policy has settled this occurrence.
    pub settled: bool,
    /// Original lifecycle, scientific availability, attempt count and effect facts.
    pub facts: pse_model::study::PointFacts,
}
/// One bounded live browsing page; it never grants frozen completeness.
#[derive(Clone, Debug)]
pub struct NativeStudySelection {
    /// Compact matched headers, in stable key order.
    pub headers: Vec<StudyWorkbenchHeader>,
    /// More live headers exist after this page.
    pub has_more: bool,
}
/// Compact acknowledgement of an exact immutable native capture.
#[derive(Clone, Debug)]
pub struct NativeStudyCapture {
    /// Exact captured header count.
    pub header_count: u64,
    /// Exact captured occurrence count, including unavailable members.
    pub member_count: u64,
    /// One shared actual owner through page and scan drain.
    pub capture: StudyCapture,
}
/// A stable capture position distinct from an occurrence's within-study ordinal.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "CapturedStudyRowFor{T}")]
pub struct CapturedStudyRow<T> {
    /// Global immutable position in this exact capture.
    pub position: u64,
    /// Original captured facts.
    pub value: T,
}
/// One bounded decoded prefix. The caller owns its response/retained allocation.
#[derive(Clone, Debug)]
pub struct NativeStudyPage<T> {
    /// Contiguous original positions after the requested cursor.
    pub rows: Vec<CapturedStudyRow<T>>,
}

/// Nonserializable exact capture capability. Clones and scans share one actual
/// process owner; dropping the last one enqueues authenticated bounded cleanup.
#[derive(Clone,Debug)]
pub struct StudyCapture(Arc<CaptureOwner>);
#[derive(Debug)]
struct CaptureOwner {
    store: CanonicalStore,
    key: String,
    owner: String,
    request: Vec<u8>,
    executor: tokio::runtime::Handle,
    release_needed:std::sync::atomic::AtomicBool,
    metadata: std::sync::Mutex<Option<Arc<dyn std::fmt::Debug+Send+Sync>>>,
}
struct CaptureCustody {request:Vec<u8>,owner:std::sync::Weak<CaptureOwner>}
type CaptureOwners=std::collections::BTreeMap<(String,String,String),CaptureCustody>;
static CAPTURE_OWNERS: std::sync::LazyLock<std::sync::Mutex<CaptureOwners>>=std::sync::LazyLock::new(||std::sync::Mutex::new(std::collections::BTreeMap::new()));
fn capture_owner(store:&CanonicalStore,key:&str,owner:&str,request:&[u8])->Result<StudyCapture,CanonicalError> {
    let mut owners=CAPTURE_OWNERS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let identity=(store.issuing_store()?.to_owned(),key.to_owned(),owner.to_owned());
    if let Some(existing)=owners.get(&identity) {
        if existing.request!=request {return Err(CanonicalError::OperationReused);}
        let existing=existing.owner.upgrade().ok_or_else(||CanonicalError::CaptureDraining{capture:key.into()})?;
        return Ok(StudyCapture(existing));
    }
    store.result_read_drain.retain()?;
    let capture=Arc::new(CaptureOwner{store:store.clone(),key:key.into(),owner:owner.into(),request:request.to_vec(),executor:tokio::runtime::Handle::current(),release_needed:std::sync::atomic::AtomicBool::new(true),metadata:std::sync::Mutex::new(None)});
    owners.insert(identity,CaptureCustody{request:request.to_vec(),owner:Arc::downgrade(&capture)});
    Ok(StudyCapture(capture))
}
impl StudyCapture {
    /// Immutable exact capture identity; this identifier alone grants no read permission.
    pub fn key(&self)->&str {&self.0.key}
    /// Fixed exact request/key custody, independent of member/page row allocations.
    pub fn custody_bytes(&self)->Result<usize,CanonicalError> {
        self.0.store.handle_heap_bytes().and_then(|bytes|bytes.checked_add(size_of::<CaptureOwner>()+size_of::<((String,String,String),CaptureCustody)>()+8*size_of::<usize>()+self.0.key.capacity()*2+self.0.owner.capacity()*2+self.0.request.capacity()*2+self.0.store.issuing_store().ok()?.len())).ok_or(CanonicalError::PayloadLimit)
    }
    /// Attach one fixed custody/release grant. Page/document grants remain with
    /// their actual consumers; replay cannot accumulate them in this owner.
    pub fn retain_metadata_owner(&self, owner: Arc<dyn std::fmt::Debug+Send+Sync>) {
        self.0.metadata.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get_or_insert(owner);
    }
    /// Native read builders in this crate consume this unforgeable exact witness.
    pub(crate) fn authority(&self)->(&CanonicalStore,&str,&str,&[u8]) {
        (&self.0.store,&self.0.key,&self.0.owner,&self.0.request)
    }
}
impl StudyCapture {
    async fn page<T>(&self, kind: &str, after: Option<u64>, maximum: usize, decode: fn(Object)->Result<T,CanonicalError>) -> Result<NativeStudyPage<T>,CanonicalError> {
        let limit=if kind=="headers" {64} else {pse_schema::catalog::study_workbench::STUDY_CAPTURE_PAGE_ROWS};
        if maximum==0 || maximum>limit {return Err(CanonicalError::PayloadLimit);}
        let (store,key,owner,request)=self.authority();
        store.files.require_current()?;
        let activity=store.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async {
            let mut response=protected_query("canonical_study_workbench::capture_page",|| Ok(store.db.metadata_query(crate::canonical::MetadataResponse::StudyPage,"RETURN fn::pse_workbench_v1::capture_page($pse_rpc_expires_at,$capture,$owner,$request,$kind,$after);")
                .bind(("capture",key.to_owned())).bind(("owner",owner.to_owned())).bind(("request",Bytes::from(request.to_vec())))
                .bind(("kind",kind.to_owned())).bind(("after",after.map(codec::encode_uint).transpose()?.unwrap_or(codec::encode_int(-1)?))))).await?;
            let page=response.take::<Vec<Object>>(0)?;
            if page.len()>limit {return Err(CanonicalError::PayloadLimit);}
            let mut previous=after;
            let mut rows=Vec::new();
            for mut row in page.into_iter().take(maximum) {
                let position=codec::decode_uint(codec::required(&mut row,"ordinal")?)?;
                if position!=previous.map_or(Some(0),|value|value.checked_add(1)).ok_or(CanonicalError::PayloadLimit)? {return Err(CanonicalError::IncompleteResponse);}
                let Value::Object(body)=codec::required(&mut row,"body")? else {return Err(CanonicalError::IncompleteResponse);};
                rows.push(CapturedStudyRow{position,value:decode(body)?});
                previous=Some(position);
            }
            Ok(NativeStudyPage{rows})
        })).await
    }
    /// Read one bounded immutable header page under this actual capture owner.
    pub async fn header_page(&self, after: Option<u64>, maximum: usize)->Result<NativeStudyPage<StudyWorkbenchHeader>,CanonicalError> {
        self.page("headers",after,maximum,header).await
    }
    /// Read one bounded immutable member page, preserving captured absence forever.
    pub async fn member_page(&self, after: Option<u64>, maximum: usize)->Result<NativeStudyPage<StudyCohortMember>,CanonicalError> {
        self.page("members",after,maximum,member).await
    }
}
impl Drop for CaptureOwner {
    fn drop(&mut self) {
        if !self.release_needed.load(std::sync::atomic::Ordering::Acquire) {
            if let Ok(issuing)=self.store.issuing_store() {CAPTURE_OWNERS.lock().unwrap_or_else(std::sync::PoisonError::into_inner).remove(&(issuing.to_owned(),self.key.clone(),self.owner.clone()));}
            self.store.result_read_drain.release().complete(Ok(()));
            return;
        }
        let store=self.store.clone();
        #[cfg(any(test,feature="test-support"))]
        let store={let mut store=store;store.fixture_lifetime=None;store};
        let key=self.key.clone(); let owner=self.owner.clone(); let request=self.request.clone();
        let metadata=self.metadata.get_mut().unwrap_or_else(std::sync::PoisonError::into_inner).take();
        let release=store.result_read_drain.release_capture(store.clone(),key.clone(),owner.clone(),request.clone(),metadata.clone());
        self.executor.spawn(async move {
            let result=crate::canonical::transport::optional_response_ownership_scope(metadata,release_capture_custody(&store,&key,&owner,&request)).await;
            if let Err(error)=&result { crate::canonical::operation_failed("canonical_study_workbench::capture_release",error); }
            drop(store);
            release.complete(result);
        });
    }
}

/// Retry only this capture's bounded native cleanup; immutable custody is never
/// reacquired. The local tombstone remains until native release is acknowledged.
pub(crate) async fn release_capture_custody(store:&CanonicalStore,key:&str,owner:&str,request:&[u8])->Result<(),CanonicalError> {
    release_capture_as(store,key,owner,request,owner).await
}
async fn release_capture_as(store:&CanonicalStore,key:&str,owner:&str,request:&[u8],custodian:&str)->Result<(),CanonicalError> {
    #[cfg(all(test,feature="canonical-tests"))]
    let pause={CAPTURE_RELEASE_PAUSES.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get(key).cloned()};
    #[cfg(all(test,feature="canonical-tests"))]
    if let Some(pause)=pause {
        pause.entered.notify_one();
        pause.resume.notified().await;
    }
    loop {
        let mut response=protected_query("canonical_study_workbench::capture_release",|| Ok(store.db.query("RETURN fn::pse_workbench_v1::release_capture($pse_rpc_expires_at,$key,$owner,$request,$custodian);")
            .bind(("key",key.to_owned())).bind(("owner",owner.to_owned())).bind(("request",Bytes::from(request.to_vec()))).bind(("custodian",custodian.to_owned())))).await?;
        #[cfg(all(test,feature="canonical-tests"))]
        if CAPTURE_RELEASE_REPLY_LOSS.lock().unwrap_or_else(std::sync::PoisonError::into_inner).remove(key) {
            return Err(CanonicalError::IncompleteResponse);
        }
        if codec::decode_boolean(response.take::<Value>(0)?)? {
            CAPTURE_OWNERS.lock().unwrap_or_else(std::sync::PoisonError::into_inner).remove(&(store.issuing_store()?.to_owned(),key.to_owned(),owner.to_owned()));
            return Ok(());
        }
    }
}
#[cfg(all(test,feature="canonical-tests"))]
#[derive(Debug,Default)]
pub(crate) struct CaptureReleasePause {pub(crate) entered:tokio::sync::Notify,pub(crate) resume:tokio::sync::Notify}
#[cfg(all(test,feature="canonical-tests"))]
static CAPTURE_RELEASE_PAUSES:std::sync::LazyLock<std::sync::Mutex<std::collections::BTreeMap<String,Arc<CaptureReleasePause>>>>=std::sync::LazyLock::new(||std::sync::Mutex::new(std::collections::BTreeMap::new()));
#[cfg(all(test,feature="canonical-tests"))]
pub(crate) fn pause_capture_release(key:&str)->Arc<CaptureReleasePause>{
    let pause=Arc::new(CaptureReleasePause::default());
    CAPTURE_RELEASE_PAUSES.lock().unwrap_or_else(std::sync::PoisonError::into_inner).insert(key.into(),pause.clone());pause
}
#[cfg(all(test,feature="canonical-tests"))]
static CAPTURE_RELEASE_REPLY_LOSS:std::sync::LazyLock<std::sync::Mutex<std::collections::BTreeSet<String>>>=std::sync::LazyLock::new(||std::sync::Mutex::new(std::collections::BTreeSet::new()));
#[cfg(all(test,feature="canonical-tests"))]
pub(crate) fn lose_capture_release_reply(key:&str) {
    CAPTURE_RELEASE_REPLY_LOSS.lock().unwrap_or_else(std::sync::PoisonError::into_inner).insert(key.into());
}

type CompactStudyAttribution = (Option<String>, Option<String>, Option<String>);

/// Project immutable scientific metadata into the same original create transaction.
/// Legacy opaque metadata has no invented physical/context attribution.
pub(crate) fn index_metadata(bytes: &[u8]) -> Result<CompactStudyAttribution, CanonicalError> {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) else {
        return Ok((None, None, None));
    };
    if value.get("version").and_then(serde_json::Value::as_u64) != Some(5) {
        return Ok((None, None, None));
    }
    let string = |v: Option<&serde_json::Value>| -> Result<Option<String>, CanonicalError> {
        v.filter(|v| !v.is_null())
            .map(|v| {
                v.as_str()
                    .filter(|s| !s.is_empty() && s.len() <= 4096)
                    .map(str::to_owned)
                    .ok_or_else(|| {
                        CanonicalError::Configuration("invalid compact study attribution".into())
                    })
            })
            .transpose()
    };
    // PhysicalSource transports an algorithm-qualified ContentHash. Canonical
    // context/source records compare its digest form, not its JSON encoding.
    let physical_identity = string(value.pointer("/physical/identity"))?
        .map(|encoded| {
            pse_ids::ContentHash::parse_prefixed(&encoded)
                .map(|identity| identity.to_hex())
                .map_err(|error| {
                    CanonicalError::Configuration(format!(
                        "invalid study physical source identity: {error}"
                    ))
                })
        })
        .transpose()?;
    Ok((
        string(value.pointer("/physical/revision"))?,
        physical_identity,
        string(value.pointer("/context/context/key"))?,
    ))
}
fn optional(row: &mut Object, name: &str) -> Result<Option<String>, CanonicalError> {
    match row.remove(name) {
        None | Some(Value::None) | Some(Value::Null) => Ok(None),
        Some(v) => Ok(Some(codec::decode_string(v)?)),
    }
}
fn header(mut row: Object) -> Result<StudyWorkbenchHeader, CanonicalError> {
    use codec::{decode_boolean as flag, decode_string as text, decode_uint as uint, required};
    let outcome = optional(&mut row, "outcome")?
        .map(|s| serde_json::from_value(serde_json::Value::String(s)))
        .transpose()
        .map_err(|e| CanonicalError::Configuration(e.to_string()))?;
    Ok(StudyWorkbenchHeader {
        study: text(required(&mut row, "key")?)?,
        run: text(required(&mut row, "run")?)?,
        problem: text(required(&mut row, "problem")?)?,
        source: text(required(&mut row, "revision")?)?,
        physical_revision: optional(&mut row, "physical_revision")?,
        physical_identity: optional(&mut row, "physical_identity")?,
        context: optional(&mut row, "context")?,
        point_count: uint(required(&mut row, "point_count")?)?,
        generation: uint(required(&mut row, "generation")?)?,
        active: flag(required(&mut row, "active")?)?,
        cancelled: flag(required(&mut row, "cancelled")?)?,
        terminal: flag(required(&mut row, "terminal")?)?,
        result_attempt: optional(&mut row, "result_attempt")?,
        outcome,
    })
}
fn member(mut row: Object) -> Result<StudyCohortMember, CanonicalError> {
    use codec::{decode_boolean as flag, decode_string as text, decode_uint as uint, required};
    let facts = codec::decode_bytes(required(&mut row, "facts")?)?;
    if facts.len() > 16384 {
        return Err(CanonicalError::PayloadLimit);
    }
    let metadata_revision = uint(required(&mut row, "revision")?)?;
    let occurrence = uint(required(&mut row, "occurrence")?)?;
    let mut facts: pse_model::study::PointFacts = serde_json::from_slice(facts.as_slice())
        .map_err(|e| CanonicalError::Configuration(e.to_string()))?;
    if u64::from(facts.key.0) != occurrence {
        return Err(CanonicalError::Configuration(
            "cohort occurrence differs".into(),
        ));
    }
    facts.revision = metadata_revision;
    Ok(StudyCohortMember {
        key: text(required(&mut row, "key")?)?,
        study: text(required(&mut row, "study")?)?,
        ordinal: uint(required(&mut row, "ordinal")?)?,
        run: text(required(&mut row, "run")?)?,
        attempt: optional(&mut row, "attempt")?,
        manifest: optional(&mut row, "manifest")?,
        metadata_revision,
        settled: flag(required(&mut row, "settled")?)?,
        facts,
    })
}
/// Bound original values separately from generated SQL. Every supplied predicate is native.
pub fn workbench_sql(filter: &StudyWorkbenchFilter, frozen: bool) -> String {
    let mut prelude = String::new();
    if filter.project.is_some() {
        prelude.push_str("LET $problems=SELECT VALUE problem FROM canonical_project_branches WHERE project=$project LIMIT 65; IF array::len($problems)>64 { THROW 'PSE_REFUSAL:study_workbench_narrow_selection'; }; ");
    }
    let mut predicates = vec!["key>$after".to_owned()];
    for (present, predicate) in [
        (filter.study.is_some(), "key=$study"),
        (filter.source.is_some(), "revision=$source"),
        (
            filter.physical_revision.is_some(),
            "physical_revision=$physical_revision",
        ),
        (filter.context.is_some(), "context=$context"),
        (
            filter.branch.is_some(),
            "problem=(SELECT problem FROM ONLY type::record('canonical_project_branches',$branch)).problem",
        ),
        (filter.project.is_some(), "problem IN $problems"),
        (
            filter.outcome.is_some(),
            "type::record('canonical_runs',run).terminal_class=$outcome",
        ),
        (
            filter.research_reference.is_some(),
            "(type::record('canonical_runs',run).research_selection=$research_reference OR type::record('canonical_runs',run).research_dataset=$research_reference OR run=type::record('canonical_research_references',$research_reference).run OR revision=type::record('canonical_research_references',$research_reference).revision OR (type::record('canonical_research_publications',$research_reference).selection!=NONE AND revision=type::record('canonical_research_publications',$research_reference).revision AND type::record('canonical_runs',run).research_selection=type::record('canonical_research_publications',$research_reference).selection))",
        ),
    ] {
        if present {
            predicates.push(predicate.into());
        }
    }
    let finish=if frozen {"RETURN fn::pse_workbench_v1::commit_capture($pse_rpc_expires_at,$capture,$owner,$request,$rows,$maximum,$members);"} else {"RETURN fn::pse_workbench_v1::browse($pse_rpc_expires_at,$rows,$maximum);"};
    let sql = format!(
        "LET $rows=SELECT key,run,problem,revision,physical_revision,physical_identity,context,point_count,generation,active,cancelled,terminal FROM canonical_studies WHERE {} ORDER BY key LIMIT $extent; {finish}",predicates.join(" AND ")
    );
    format!("BEGIN; {prelude}{sql} COMMIT;")
}
impl CanonicalStore {
    /// Compact selected-run retry head; never fetch requests, source payloads or completion bytes.
    pub async fn workbench_run_generation(&self, run: &str) -> Result<Option<u64>, CanonicalError> {
        if run.is_empty() || run.len() > 4096 {
            return Err(CanonicalError::PayloadLimit);
        }
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async {
            let mut response=protected_query("canonical_study_workbench::run_generation",||Ok(self.db.query(crate::canonical_history::run_query("SELECT current_generation FROM ONLY type::record('canonical_runs',$run);","run")).bind(("run",run.to_owned())))).await?;
            response.take::<Option<Object>>(0)?.map(|mut row|codec::decode_uint(codec::required(&mut row,"current_generation")?).map_err(Into::into)).transpose()
        })).await
    }
    /// Select one bounded live browsing page; it never certifies frozen completeness.
    pub async fn select_study_workbench(
        &self,
        filter: &StudyWorkbenchFilter,
        after: Option<&str>,
        maximum: usize,
    ) -> Result<NativeStudySelection, CanonicalError> {
        if !(1..=64).contains(&maximum) {
            return Err(CanonicalError::PayloadLimit);
        }
        for value in [
            &filter.study,
            &filter.project,
            &filter.branch,
            &filter.source,
            &filter.physical_revision,
            &filter.context,
            &filter.research_reference,
        ]
        .into_iter()
        .flatten()
        .map(String::as_str)
        .chain(after)
        {
            if value.is_empty() || value.len() > 4096 {
                return Err(CanonicalError::PayloadLimit);
            }
        }
        let sql = workbench_sql(filter, false);
        let activity = self.admit_activity()?;
        activity
            .scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
                let mut response = protected_query("canonical_study_workbench::select", || {
                    Ok(self
                        .db
                        .query(sql.clone())
                        .bind(("study", filter.study.clone()))
                        .bind(("project", filter.project.clone()))
                        .bind(("branch", filter.branch.clone()))
                        .bind(("source", filter.source.clone()))
                        .bind(("physical_revision", filter.physical_revision.clone()))
                        .bind(("context", filter.context.clone()))
                        .bind(("research_reference", filter.research_reference.clone()))
                        .bind((
                            "outcome",
                            filter
                                .outcome
                                .map(serde_json::to_value)
                                .transpose()
                                .map_err(|e| CanonicalError::Configuration(e.to_string()))?
                                .and_then(|v| v.as_str().map(str::to_owned)),
                        ))
                        .bind(("after", after.unwrap_or("").to_owned()))
                        .bind(("maximum", maximum))
                        .bind(("extent", maximum + 1))
                        .bind(("members", 0)))
                })
                .await?;
                let index = response
                    .num_statements()
                    .checked_sub(2)
                    .ok_or(CanonicalError::IncompleteResponse)?;
                let mut row = match response.take::<Value>(index)? {
                    Value::Object(row) => row,
                    _ => return Err(CanonicalError::IncompleteResponse),
                };
                let has_more = codec::decode_boolean(codec::required(&mut row, "has_more")?)?;
                let rows = match codec::required(&mut row, "headers")? {
                    Value::Array(rows) => rows,
                    _ => return Err(CanonicalError::IncompleteResponse),
                };
                let headers = rows
                    .into_iter()
                    .map(|v| match v {
                        Value::Object(v) => header(v),
                        _ => Err(CanonicalError::IncompleteResponse),
                    })
                    .collect::<Result<_, _>>()?;
                Ok(NativeStudySelection { headers, has_more })
            }))
            .await
    }
    /// Explicitly resume abandoned/failed cleanup. A live local scan refuses;
    /// another actual process must first prove and register the original owner's
    /// drain with the existing file-owner recovery operation. Disconnection alone
    /// never supplies that proof or drops a captured protection.
    pub async fn recover_study_capture(&self,key:&str)->Result<(),CanonicalError> {
        if key.is_empty() || key.len()>4096 {return Err(CanonicalError::PayloadLimit);}
        let activity=self.admit_activity()?;
        activity.scope(async {
            let custodian=self.registered_file_owner().await?;
            let mut response=protected_query("canonical_study_workbench::capture_recovery_identity",|| Ok(self.db.query("SELECT owner,request FROM ONLY type::record('canonical_study_captures',$capture);").bind(("capture",key.to_owned())))).await?;
            let mut row=response.take::<Option<Object>>(0)?.ok_or(CanonicalError::IncompleteResponse)?;
            let owner=codec::decode_string(codec::required(&mut row,"owner")?)?;
            let request=codec::decode_bytes(codec::required(&mut row,"request")?)?;
            let identity=(self.issuing_store()?.to_owned(),key.to_owned(),owner.clone());
            {
                let mut owners=CAPTURE_OWNERS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                if owners.get(&identity).is_some_and(|entry|entry.owner.strong_count()!=0) {return Err(CanonicalError::CaptureInUse{capture:key.into()});}
                owners.entry(identity).or_insert_with(||CaptureCustody{request:request.as_slice().to_vec(),owner:std::sync::Weak::new()});
            }
            release_capture_as(self,key,&owner,request.as_slice(),&custodian).await
        }).await
    }

    /// Capture an exact immutable request once, retaining the original gate grant.
    /// Complete population admission shares the public workbench's declared bound.
    pub async fn capture_study_workbench(&self,key:&str,filter:&StudyWorkbenchFilter,maximum:usize,members:usize)->Result<NativeStudyCapture,CanonicalError> {
        if key.is_empty() || key.len()>4096 || !(1..=64).contains(&maximum) || !(1..=pse_schema::catalog::study_workbench::STUDY_COHORT_MAX_MEMBERS).contains(&members) {return Err(CanonicalError::PayloadLimit);}
        for value in [&filter.study,&filter.project,&filter.branch,&filter.source,&filter.physical_revision,&filter.context,&filter.research_reference].into_iter().flatten() {
            if value.is_empty() || value.len()>4096 {return Err(CanonicalError::PayloadLimit);}
        }
        let activity=self.admit_activity()?;
        activity.scope(async {
            self.ensure_writes()?;
            let owner=self.registered_file_owner().await?;
            let request=serde_json::to_vec(&(1_u8,filter,maximum,members)).map_err(|e|CanonicalError::Configuration(e.to_string()))?;
            let capture=capture_owner(self,key,&owner,&request)?;
            let receipt=within_clock(original_deadline(REQUEST_TIMEOUT),async {
                let opened=protected_query("canonical_study_workbench::capture_open",|| Ok(self.db.query("RETURN fn::pse_workbench_v1::open_capture($pse_rpc_expires_at,$capture,$owner,$request);").bind(("capture",key.to_owned())).bind(("owner",owner.clone())).bind(("request",Bytes::from(request.clone()))))).await.and_then(|mut response|response.take::<Option<Object>>(0)?.ok_or(CanonicalError::IncompleteResponse));
                let opened=crate::canonical::settlement::settle_exact("study-cohort-gate",key,&request,opened,async {
                    let mut response=protected_query("canonical_study_workbench::gate_readback",|| Ok(self.db.query("SELECT * FROM ONLY type::record('canonical_study_captures',$capture);").bind(("capture",key.to_owned())))).await?;
                    Ok(response.take::<Option<Object>>(0)?)
                },|row| row.get("request")==Some(&Value::Bytes(Bytes::from(request.clone()))) && row.get("owner")==Some(&Value::String(owner.clone())),CanonicalError::source_transaction_refused).await.map_err(|error| {
                    if matches!(error,CanonicalError::OperationReused) {capture.0.release_needed.store(false,std::sync::atomic::Ordering::Release);}
                    error
                })?;
                if opened.get("state")==Some(&Value::String("frozen".into())) {return Ok(opened);}
                if opened.get("state")!=Some(&Value::String("pending".into())) {return Err(CanonicalError::CaptureClosed{capture:key.into()});}
                let sql=workbench_sql(filter,true);
                let write=protected_query("canonical_study_workbench::capture_commit",|| Ok(self.db.query(sql.clone())
                    .bind(("capture",key.to_owned())).bind(("owner",owner.clone())).bind(("request",Bytes::from(request.clone())))
                    .bind(("study",filter.study.clone())).bind(("project",filter.project.clone())).bind(("branch",filter.branch.clone())).bind(("source",filter.source.clone())).bind(("physical_revision",filter.physical_revision.clone())).bind(("context",filter.context.clone())).bind(("research_reference",filter.research_reference.clone()))
                    .bind(("outcome",filter.outcome.map(serde_json::to_value).transpose().map_err(|e| CanonicalError::Configuration(e.to_string()))?.and_then(|v|v.as_str().map(str::to_owned))))
                    .bind(("after",String::new())).bind(("maximum",maximum)).bind(("extent",maximum+1)).bind(("members",members)))).await.and_then(|mut response| {let index=response.num_statements().checked_sub(2).ok_or(CanonicalError::IncompleteResponse)?;response.take::<Option<Object>>(index)?.ok_or(CanonicalError::IncompleteResponse)});
                crate::canonical::settlement::settle_exact("study-cohort-capture",key,&request,write,async {
                    let mut response=protected_query("canonical_study_workbench::capture_readback",|| Ok(self.db.query("SELECT * FROM ONLY type::record('canonical_study_captures',$capture);").bind(("capture",key.to_owned())))).await?;
                    let row=response.take::<Option<Object>>(0)?;
                    Ok(row.filter(|row|row.get("state")==Some(&Value::String("frozen".into()))))
                },|row| row.get("request")==Some(&Value::Bytes(Bytes::from(request.clone()))) && row.get("owner")==Some(&Value::String(owner.clone())) && row.get("state")==Some(&Value::String("frozen".into())),CanonicalError::source_transaction_refused).await
            }).await?;
            let mut receipt=receipt;
            let header_count=codec::decode_uint(codec::required(&mut receipt,"header_count")?)?;
            let member_count=codec::decode_uint(codec::required(&mut receipt,"member_count")?)?;
            if header_count>maximum as u64 || member_count>members as u64 {return Err(CanonicalError::IncompleteResponse);}
            Ok(NativeStudyCapture{header_count,member_count,capture})
        }).await
    }

}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn workbench_query_has_real_predicates_and_never_interpolates_values() {
        let filter = StudyWorkbenchFilter {
            study: Some("';DELETE canonical_studies;".into()),
            source: Some("source".into()),
            physical_revision: Some("physical".into()),
            outcome: Some(pse_model::generated::enums::CanonicalTerminalOutcome::Succeeded),
            ..Default::default()
        };
        let sql = workbench_sql(&filter, true);
        assert!(!sql.contains("DELETE"));
        assert!(sql.contains("revision=$source"));
        assert!(sql.contains("physical_revision=$physical_revision"));
        assert!(sql.contains(".terminal_class=$outcome"));
        assert!(!sql.contains("eligible"));
    }
    #[test]
    fn index_attribution_is_exact_optional_and_not_invented_for_opaque_metadata() {
        assert_eq!(index_metadata(&[9]).unwrap(), (None, None, None));
        let identity = pse_ids::ContentHash::from_bytes([7; 32]);
        let metadata = serde_json::json!({
            "version": 5,
            "physical": {"revision": "p", "identity": identity},
            "context": {"context": {"key": "cut"}},
        });
        assert_eq!(metadata["physical"]["identity"], identity.to_prefixed());
        assert_eq!(
            index_metadata(&serde_json::to_vec(&metadata).unwrap()).unwrap(),
            (
                Some("p".into()),
                Some(identity.to_hex()),
                Some("cut".into())
            )
        );
        assert_eq!(
            index_metadata(br#"{"version":5,"physical":null,"context":null}"#).unwrap(),
            (None, None, None)
        );
        assert_eq!(
            index_metadata(br#"{"version":4,"physical":{"identity":"opaque historical value"}}"#)
                .unwrap(),
            (None, None, None)
        );
    }
    #[test]
    fn index_attribution_rejects_untyped_or_malformed_physical_checksums() {
        let identity = pse_ids::ContentHash::from_bytes([7; 32]);
        for invalid in [
            serde_json::json!(identity.to_hex()),
            serde_json::json!(format!("sha256:{}", identity.to_hex())),
            serde_json::json!("blake3:short"),
            serde_json::json!(17),
        ] {
            let metadata = serde_json::json!({"version": 5, "physical": {"revision": "p", "identity": invalid}});
            assert!(index_metadata(&serde_json::to_vec(&metadata).unwrap()).is_err());
        }
    }
}
