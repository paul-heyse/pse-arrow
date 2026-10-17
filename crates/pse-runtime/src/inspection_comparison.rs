// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pure bounded interpretations of the existing joined invocation document.
use crate::invocation::{InvocationEvidenceDocument,InvocationOutcome,ProductAvailability};
use pse_diagnostics::{NextObservation,StatementKind,observation::{OperationId,Value}};
use pse_model::{document::Version,generated::identities::{AttemptId,RunId}};
use serde::{Deserialize,Serialize};

const TEXT_LIMIT:usize=512;
/// Response bounds, not a scientific or performance acceptance policy.
#[derive(Clone,Copy,Debug,Serialize,Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComparisonPolicy {
    /// Maximum returned statements; effective ceiling is 256.
    pub statements:usize,
    /// Source steps, products, decisions and intervals inspected per collection;
    /// effective ceiling is 64. Omitted source items remain explicit.
    pub source_items:usize,
}
impl Default for ComparisonPolicy {
    fn default()->Self {Self{statements:64,source_items:32}}
}
impl ComparisonPolicy {
    fn bounded(self)->Self {Self{statements:self.statements.min(256),source_items:self.source_items.min(64)}}
}
/// Actual source document identity. Absence of diagnostics does not invent an operation.
#[derive(Clone,Debug,Serialize,Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InvocationReference {
    /// Original execution identity.
    pub run_id:RunId,
    /// Durable attempt, only when registered.
    pub attempt_id:Option<AttemptId>,
    /// Actual admitted diagnostic operation.
    pub operation:Option<OperationId>,
}
impl From<&InvocationEvidenceDocument> for InvocationReference {
    fn from(doc:&InvocationEvidenceDocument)->Self {Self{run_id:doc.run_id,attempt_id:doc.attempt_id,operation:doc.operation}}
}
/// A source-native field reference rather than a second invocation or work ledger.
#[derive(Clone,Debug,Serialize,Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InvocationEvidenceReference {
    /// Exact source execution/attempt/operation.
    pub invocation:InvocationReference,
    /// JSON pointer into the original InvocationEvidenceDocument.
    pub field:String,
    /// Actual field value; missing values remain unavailable.
    pub value:Value,
    /// Performing clock/population meaning, never exclusive wall or inferred wait.
    pub clock_scope:String,
    /// Relevant retained coverage does not establish a complete population.
    pub partial:bool,
}
/// Shared statement strength and optional next observations retain their existing meanings.
#[derive(Clone,Debug,Serialize,Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InvocationComparisonStatement {
    /// Established fact, hypothesis, or interpretation limitation.
    pub kind:StatementKind,
    /// Stable interpretation code.
    pub code:String,
    /// Bounded explanation; no performance winner is selected.
    pub message:String,
    /// Actual source fields supporting the explanation.
    pub evidence:Vec<InvocationEvidenceReference>,
    /// Discriminating optional observations; never performed by this operation.
    pub next:Vec<NextObservation>,
}
/// Immutable explanation/comparison output over borrowed source documents.
#[derive(Clone,Debug,Serialize,Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InvocationComparisonDocument {
    /// Standalone response contract.
    pub version:Version<1>,
    /// Explicit policy name; descriptive comparisons do not change science.
    pub policy_name:String,
    /// Interpretation algorithm version.
    pub policy_version:u32,
    /// Effective bounded response policy.
    pub policy:ComparisonPolicy,
    /// Actual first/source invocation.
    pub baseline:InvocationReference,
    /// Actual second invocation, absent for a single-invocation explanation.
    pub candidate:Option<InvocationReference>,
    /// Evidence-linked interpretations.
    pub statements:Vec<InvocationComparisonStatement>,
    /// Statements omitted by the response bound.
    pub omitted_statements:usize,
    /// Source records not inspected because of this policy's bounds.
    pub omitted_source_items:usize,
    /// Source or response bounds/loss prevent complete interpretation.
    pub partial:bool,
}
struct Builder {document:InvocationComparisonDocument}
fn text(value:&str)->Value {
    if value.len()>TEXT_LIMIT {Value::Unavailable("source text exceeds comparison bound".into())}
    else {Value::Text(value.into())}
}
fn optional_count(value:Option<u64>)->Value {value.map_or_else(||Value::Unavailable("owner counter unavailable".into()),Value::Count)}
fn seconds(value:Option<f64>)->Value {
    value.map_or_else(||Value::Unavailable("owner time unavailable".into()),Value::number)
}
fn evidence(doc:&InvocationEvidenceDocument,field:impl Into<String>,value:Value,scope:&str,partial:bool)->InvocationEvidenceReference {
    let oversized=scope.len()>TEXT_LIMIT;
    InvocationEvidenceReference {invocation:doc.into(),field:field.into(),value,
        clock_scope:if oversized {"source clock scope exceeds comparison bound".into()}else{scope.into()},
        partial:partial||oversized}
}
fn inv(doc:&InvocationEvidenceDocument,field:&str,value:Value)->InvocationEvidenceReference {
    evidence(doc,field,value,&doc.clock_scope,doc.timing_unavailable.is_some())
}
fn counted(doc:&InvocationEvidenceDocument,field:impl Into<String>,value:Value,partial:bool)->InvocationEvidenceReference {
    evidence(doc,field,value,"inclusive synchronous owner counters; no elapsed clock",partial)
}
impl Builder {
    fn new(a:&InvocationEvidenceDocument,b:Option<&InvocationEvidenceDocument>,policy:ComparisonPolicy)->Self {
        Self{document:InvocationComparisonDocument{version:Version,policy_name:"source-linked-invocation-comparison".into(),policy_version:1,
            policy:policy.bounded(),baseline:a.into(),candidate:b.map(Into::into),statements:Vec::new(),
            omitted_statements:0,omitted_source_items:0,partial:false}}
    }
    fn add(&mut self,kind:StatementKind,code:&str,message:impl Into<String>,evidence:Vec<InvocationEvidenceReference>,next:Vec<NextObservation>) {
        let message=message.into();
        self.document.partial|=evidence.iter().any(|source|source.partial || matches!(source.value,Value::Unavailable(_)));
        if self.document.statements.len()>=self.document.policy.statements {
            self.document.omitted_statements=self.document.omitted_statements.saturating_add(1);
            self.document.partial=true;
            return;
        }
        let message=if message.len()>TEXT_LIMIT {
            self.document.partial=true;
            let end=message.char_indices().map(|(index,_)|index).take_while(|index|*index<=TEXT_LIMIT-3).last().unwrap_or(0);
            format!("{}...",&message[..end])
        }else{message};
        self.document.statements.push(InvocationComparisonStatement{kind,code:code.into(),message,evidence,next});
    }
    fn omitted(&mut self,total:usize) {
        let omitted=total.saturating_sub(self.document.policy.source_items);
        self.document.omitted_source_items=self.document.omitted_source_items.saturating_add(omitted);
        self.document.partial|=omitted>0;
    }
}
// Only the small source-owned enums below enter this helper. Their serde declaration,
// rather than another spelling table, owns their exported names.
fn enum_text(value:impl Serialize)->Value {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(value))=>text(&value),
        _=>Value::Unavailable("source enum spelling unavailable".into()),
    }
}
fn outcome(value:InvocationOutcome)->Value {enum_text(value)}
fn availability(value:ProductAvailability)->Value {enum_text(value)}
fn source_partial(doc:&InvocationEvidenceDocument)->bool {
    doc.omitted_intervals>0 || doc.omitted_product_steps>0 || doc.attempts.omitted_steps>0 || doc.attempts.omitted_native_attempts>0
}
fn source_valid(builder:&mut Builder,doc:&InvocationEvidenceDocument)->bool {
    if doc.version==1 && doc.run_id==doc.attempts.run_id {return true;}
    builder.add(StatementKind::Limitation,"invocation.source_identity_unavailable","The joined document version or its retained attempt run identity is incompatible. No joined work or elapsed interpretation is established from this source.",
        vec![counted(doc,"/version",Value::Count(u64::from(doc.version)),true),counted(doc,"/run_id",text(&doc.run_id.to_string()),true),
            counted(doc,"/attempts/run_id",text(&doc.attempts.run_id.to_string()),true)],vec![NextObservation::InspectRelated]);
    false
}
fn retained_failure(builder:&mut Builder,doc:&InvocationEvidenceDocument) {
    if let Some(failure)=&doc.attempts.failure {
        builder.add(StatementKind::Established,"invocation.retained_failure","The original attempt owner retained this typed failure. It describes supplied failure evidence, not a new assessment or proof that every strategy operation failed.",vec![
            counted(doc,"/attempts/failure/observation",enum_text(failure.observation),false),
            counted(doc,"/attempts/failure/code",enum_text(failure.code),false),
            counted(doc,"/attempts/failure/class",enum_text(failure.class),false),
            counted(doc,"/attempts/failure/retained_only",Value::Boolean(failure.retained_only),false)],vec![NextObservation::InspectRelated]);
    }
}
fn step_context(builder:&mut Builder,doc:&InvocationEvidenceDocument,index:usize,step:&crate::workflow::AttemptStepSummary) {
    let prefix=format!("/attempts/steps/{index}");
    let predicate=|value:Option<bool>|value.map_or_else(||Value::Unavailable("native owner predicate unavailable".into()),Value::Boolean);
    builder.add(if step.start_submitted.is_some() && step.reused_native_state.is_some(){StatementKind::Established}else{StatementKind::Limitation},"invocation.start_and_reuse","These are the actual native owner's supplied start/reuse predicates. Retained native state does not establish reused numeric factors; a submitted seed is distinct from retained state.",vec![
        counted(doc,format!("{prefix}/start_submitted"),predicate(step.start_submitted),false),
        counted(doc,format!("{prefix}/reused_native_state"),predicate(step.reused_native_state),false)],vec![]);
    if let Some(start)=&step.start {
        builder.add(StatementKind::Established,"invocation.start_metadata","The start owner retained these seed facts. The data identity describes the seed, not all scientific parameters, fixed values, coefficients or bounds.",vec![
            counted(doc,format!("{prefix}/start/previous_attempt"),optional_count(start.previous_attempt.map(|v|v as u64)),false),
            counted(doc,format!("{prefix}/start/layout"),start.layout.map_or_else(||Value::Unavailable("seed layout unavailable".into()),|v|text(&v.to_string())),false),
            counted(doc,format!("{prefix}/start/data"),start.data.map_or_else(||Value::Unavailable("seed data unavailable".into()),|v|text(&v.to_string())),false),
            counted(doc,format!("{prefix}/start/sparse_variables"),Value::Count(start.sparse_variables as u64),false),
            counted(doc,format!("{prefix}/start/submitted"),Value::Boolean(start.submitted),false)],vec![]);
        builder.omitted(start.transformations.len());
        if !start.transformations.is_empty() {
            // Retain the source reference without serializing potentially large nested
            // presolve/working-set paths into this bounded interpretation.
            builder.add(StatementKind::Limitation,"invocation.seed_transformations_uninterpreted","The original start document retains transformations. This bounded scalar interpretation does not reconstruct their coordinate mappings or infer compatible numeric state.",vec![
                counted(doc,format!("{prefix}/start/transformations"),Value::Unavailable("inspect the retained transformation path".into()),true)],vec![NextObservation::InspectRelated]);
        }
    } else {
        builder.add(StatementKind::Limitation,"invocation.start_metadata_unavailable","No detailed start metadata was retained. Absence is not evidence that no seed was submitted or that native state was cold.",vec![counted(doc,format!("{prefix}/start"),Value::Unavailable("detailed start metadata unavailable".into()),true)],vec![NextObservation::InspectRelated]);
    }
    for (name,options) in [("effective_options",&step.effective_options),("native_defaults",&step.native_defaults)] {
        builder.omitted(options.len());
        if name=="native_defaults" && options.is_empty() {
            builder.add(StatementKind::Limitation,"invocation.native_defaults_unavailable","No native defaults were read back at this boundary. Empty queried defaults do not establish that the native library has no defaults or that submitted options describe all effective behavior.",vec![
                counted(doc,format!("{prefix}/{name}"),Value::Unavailable("native default readback unavailable".into()),true)],vec![NextObservation::InspectRelated]);
        }
        for (key,value) in options.iter().take(builder.document.policy.source_items) {
            if key.len()>TEXT_LIMIT {
                builder.document.omitted_source_items=builder.document.omitted_source_items.saturating_add(1);
                builder.document.partial=true;
                continue;
            }
            let key=key.replace('~',"~0").replace('/',"~1");
            let (value,leaf)=match value {
                pse_backend_native::solve::OptionValue::Bool(v)=>(Value::Boolean(*v),"/value"),
                pse_backend_native::solve::OptionValue::Integer(v)=>(Value::Integer(i64::from(*v)),"/value"),
                pse_backend_native::solve::OptionValue::Real(v) if v.is_finite()=>(Value::number(*v),"/value"),
                pse_backend_native::solve::OptionValue::Real(_)=>(Value::Unavailable("nonfinite native option; inspect original tagged observation".into()),""),
                pse_backend_native::solve::OptionValue::Text(v)=>(text(v),"/value"),
            };
            builder.add(if matches!(value,Value::Unavailable(_)){StatementKind::Limitation}else{StatementKind::Established},if name=="effective_options" {"invocation.submitted_option"}else{"invocation.queried_default"},
                if name=="effective_options" {"This is an actual adapter-submitted option, distinct from queried defaults and unspecified native behavior."}else{"This is an actual queried native default, distinct from an adapter-submitted override."},
                vec![counted(doc,format!("{prefix}/{name}/{key}{leaf}"),value,false)],vec![]);
        }
    }
}
fn context_comparison(builder:&mut Builder,a:&InvocationEvidenceDocument,ai:usize,left:&crate::workflow::AttemptStepSummary,b:&InvocationEvidenceDocument,bi:usize,right:&crate::workflow::AttemptStepSummary) {
    let value=|v:Option<bool>|v.map_or_else(||Value::Unavailable("native owner predicate unavailable".into()),Value::Boolean);
    for (field,l,r) in [("start_submitted",left.start_submitted,right.start_submitted),("reused_native_state",left.reused_native_state,right.reused_native_state)] {
        if l!=r {
            builder.add(StatementKind::Limitation,"comparison.start_or_reuse_context_changed","Recorded seed submission or retained native state differs. Literal work/whole-loop averages remain descriptive facts; this context change limits causal cost attribution and does not establish numeric factor reuse.",vec![
                counted(a,format!("/attempts/steps/{ai}/{field}"),value(l),false),counted(b,format!("/attempts/steps/{bi}/{field}"),value(r),false)],vec![NextObservation::InspectRelated]);
        }
    }
    if let Some((l,r))=left.start.as_ref().zip(right.start.as_ref()) {
        if l.data!=r.data || l.layout!=r.layout || l.previous_attempt!=r.previous_attempt || l.sparse_variables!=r.sparse_variables || l.submitted!=r.submitted || l.transformations.len()!=r.transformations.len() {
            builder.add(StatementKind::Limitation,"comparison.seed_context_changed","Actual retained seed metadata differs. Different starts or coordinate transformations can change the trajectory; they do not establish an intrinsic solver or callback cost difference.",vec![
                counted(a,format!("/attempts/steps/{ai}/start/data"),l.data.map_or_else(||Value::Unavailable("seed data unavailable".into()),|v|text(&v.to_string())),false),
                counted(b,format!("/attempts/steps/{bi}/start/data"),r.data.map_or_else(||Value::Unavailable("seed data unavailable".into()),|v|text(&v.to_string())),false),
                counted(a,format!("/attempts/steps/{ai}/start/layout"),l.layout.map_or_else(||Value::Unavailable("seed layout unavailable".into()),|v|text(&v.to_string())),false),
                counted(b,format!("/attempts/steps/{bi}/start/layout"),r.layout.map_or_else(||Value::Unavailable("seed layout unavailable".into()),|v|text(&v.to_string())),false),
                counted(a,format!("/attempts/steps/{ai}/start/submitted"),Value::Boolean(l.submitted),false),
                counted(b,format!("/attempts/steps/{bi}/start/submitted"),Value::Boolean(r.submitted),false),
                counted(a,format!("/attempts/steps/{ai}/start/sparse_variables"),Value::Count(l.sparse_variables as u64),false),
                counted(b,format!("/attempts/steps/{bi}/start/sparse_variables"),Value::Count(r.sparse_variables as u64),false),
                counted(a,format!("/attempts/steps/{ai}/start/previous_attempt"),optional_count(l.previous_attempt.map(|v|v as u64)),false),
                counted(b,format!("/attempts/steps/{bi}/start/previous_attempt"),optional_count(r.previous_attempt.map(|v|v as u64)),false),
                counted(a,format!("/attempts/steps/{ai}/start/transformations"),Value::Unavailable("inspect source transformation path".into()),true),
                counted(b,format!("/attempts/steps/{bi}/start/transformations"),Value::Unavailable("inspect source transformation path".into()),true)],vec![NextObservation::InspectRelated]);
        }
    } else if left.start.is_some()!=right.start.is_some() {
        builder.add(StatementKind::Limitation,"comparison.seed_context_unavailable","Detailed start coverage differs. Missing seed metadata is not a matching cold-start context.",vec![
            counted(a,format!("/attempts/steps/{ai}/start"),Value::Unavailable("paired detailed start context not established".into()),true),
            counted(b,format!("/attempts/steps/{bi}/start"),Value::Unavailable("paired detailed start context not established".into()),true)],vec![NextObservation::InspectRelated]);
    }
    for (name,l,r) in [("effective_options",&left.effective_options,&right.effective_options),("native_defaults",&left.native_defaults,&right.native_defaults)] {
      if l!=r {
        builder.add(StatementKind::Limitation,"comparison.native_settings_changed","Submitted options or queried native defaults differ. The individual retained values are reported separately; absent default readback cannot establish matching unspecified behavior.",vec![
            counted(a,format!("/attempts/steps/{ai}/{name}"),Value::Unavailable("inspect bounded submitted option/default facts".into()),true),
            counted(b,format!("/attempts/steps/{bi}/{name}"),Value::Unavailable("inspect bounded submitted option/default facts".into()),true)],vec![NextObservation::InspectRelated]);
      }
    }
}
fn strategy_comparison(builder:&mut Builder,doc:&InvocationEvidenceDocument) {
    let limit=builder.document.policy.source_items;
    for (index,step) in doc.attempts.steps.iter().take(limit).enumerate() {
        builder.omitted(step.decisions.len());
        step_context(builder,doc,index,step);
        let started=step.decisions.iter().take(limit).filter(|event|event.kind==pse_model::generated::enums::NumericalEventKind::Started).count();
        let failure=step.decisions.iter().take(limit).enumerate().find(|(_,event)|matches!(event.kind,pse_model::generated::enums::NumericalEventKind::Refused|pse_model::generated::enums::NumericalEventKind::Abandoned));
        if started>1 || failure.is_some() || step.accepted==Some(false) {
            let mut fields=vec![counted(doc,format!("/attempts/steps/{index}/accepted"),step.accepted.map_or_else(||Value::Unavailable("original permission unavailable".into()),Value::Boolean),false)];
            fields.extend(step.decisions.iter().take(limit).enumerate().filter(|(_,event)|event.kind==pse_model::generated::enums::NumericalEventKind::Started).take(2)
                .map(|(event,decision)|counted(doc,format!("/attempts/steps/{index}/decisions/{event}/mechanism"),Value::Count(decision.mechanism as u64),step.omitted_decisions>0)));
            if let Some((event,decision))=failure {fields.push(counted(doc,format!("/attempts/steps/{index}/decisions/{event}/kind"),text(decision.kind.as_str()),step.omitted_decisions>0));}
            builder.add(StatementKind::Established,"comparison.strategy_history","Retained strategy evidence contains multiple starts, refusal/abandonment or original refusal. Declared stages are not automatically retries; native records and decision work may overlap and are never summed again.",fields,vec![NextObservation::InspectRelated]);
        }
        if step.dropped_progress.is_some_and(|count|count>0) || step.omitted_decisions>0 {
            builder.add(StatementKind::Limitation,"comparison.strategy_history_partial","Progress loss and decision-prefix omissions prevent reconstructing continuous behavior or complete retries. They do not by themselves invalidate independently retained synchronous terminal counters.",
                vec![counted(doc,format!("/attempts/steps/{index}/dropped_progress"),optional_count(step.dropped_progress),step.dropped_progress.is_some_and(|count|count>0)),
                    counted(doc,format!("/attempts/steps/{index}/omitted_decisions"),Value::Count(step.omitted_decisions as u64),step.omitted_decisions>0)],vec![NextObservation::TimelineCapture]);
        }
    }
}
fn explain(builder:&mut Builder,doc:&InvocationEvidenceDocument) {
    let limit=builder.document.policy.source_items;
    builder.document.partial|=source_partial(doc);
    retained_failure(builder,doc);
    builder.add(StatementKind::Established,"invocation.outcome","The invocation owner reported this operational outcome. Original scientific acceptance and requested product availability remain separate facts.",
        vec![inv(doc,"/outcome",outcome(doc.outcome)),counted(doc,"/attempts/usable",Value::Boolean(doc.attempts.usable),false)],vec![]);
    if let Some(preparation)=doc.preparations {
        builder.add(StatementKind::Established,"invocation.preparation_population","The task-local owner recorded these preparation and rebind populations. They are not native iteration counts, and their presence does not establish preparation time inside the supervisor clock.",
            [("views",preparation.views),("observations",preparation.observations),("rebuilt",preparation.rebuilt),("shared",preparation.shared)].into_iter()
                .map(|(name,value)|counted(doc,format!("/preparations/{name}"),Value::Count(value as u64),false)).collect(),vec![]);
    }
    if let Some(unavailable)=doc.timing_unavailable {
        builder.add(StatementKind::Limitation,"invocation.timing_unavailable","Performing clocks were not established. Missing intervals/endpoints are unavailable, not zero elapsed work.",
            vec![inv(doc,"/timing_unavailable",enum_text(unavailable))],vec![NextObservation::TimelineCapture]);
    }
    if source_partial(doc) {
        builder.add(StatementKind::Limitation,"invocation.coverage_partial","The retained document omits intervals, product steps or attempts. Prefixes do not establish complete history, total retries or continuous convergence.",vec![
            counted(doc,"/omitted_intervals",Value::Count(doc.omitted_intervals as u64),doc.omitted_intervals>0),
            counted(doc,"/omitted_product_steps",Value::Count(doc.omitted_product_steps as u64),doc.omitted_product_steps>0),
            counted(doc,"/attempts/omitted_steps",Value::Count(doc.attempts.omitted_steps as u64),doc.attempts.omitted_steps>0),
            counted(doc,"/attempts/omitted_native_attempts",Value::Count(doc.attempts.omitted_native_attempts as u64),doc.attempts.omitted_native_attempts>0)],vec![NextObservation::InspectRelated]);
    }
    builder.omitted(doc.products.len());
    for (index,product) in doc.products.iter().take(limit).enumerate() {
        if product.availability!=ProductAvailability::Available {
            builder.add(StatementKind::Established,"invocation.product_availability","This product's own governing availability is recorded independently of native convergence and response readiness.",
                vec![counted(doc,format!("/products/{index}/role"),text(&product.role),false),
                    counted(doc,format!("/products/{index}/availability"),availability(product.availability),false)],vec![]);
        }
    }
    builder.omitted(doc.intervals.len());
    for (index,interval) in doc.intervals.iter().take(limit).enumerate() {
        let elapsed=interval.ended_seconds.filter(|end|end.is_finite() && interval.started_seconds.is_finite() && interval.started_seconds>=0. && *end>=interval.started_seconds)
            .map(|end|end-interval.started_seconds);
        builder.add(if elapsed.is_some() {StatementKind::Established}else{StatementKind::Limitation},"invocation.inclusive_interval",
            elapsed.map_or_else(||"This owner interval has no complete valid endpoint. Its age does not establish computation, contention, a hang or wait time.".into(),
                |elapsed|format!("This directly observed owner interval lasted {elapsed:.6} seconds inclusively. Nested/native/postsolve intervals may overlap; do not sum them as exclusive wall time.")),
            vec![inv(doc,&format!("/intervals/{index}/stage"),text(&interval.stage)),inv(doc,&format!("/intervals/{index}/started_seconds"),seconds(Some(interval.started_seconds))),
                evidence(doc,format!("/intervals/{index}/ended_seconds"),seconds(interval.ended_seconds),&doc.clock_scope,doc.timing_unavailable.is_some()||elapsed.is_none())],
            if elapsed.is_some(){vec![]}else{vec![NextObservation::CpuProfile,NextObservation::OptionalWaitProbe]});
    }
    builder.omitted(doc.attempts.steps.len());
    for (index,step) in doc.attempts.steps.iter().take(limit).enumerate() {
        step_context(builder,doc,index,step);
        if let Some(work)=step.work {
            builder.add(StatementKind::Established,"invocation.recorded_work","The synchronous owner supplied these inclusive work populations. Unknown counters stay unavailable; step, native and decision charges are not added together as separate work.",
                [("attempts",Some(work.attempts)),("evaluations",work.evaluations),("iterations",work.iterations),("factorizations",work.factorizations),("proof_steps",work.proof_steps)].into_iter()
                    .map(|(name,value)|counted(doc,format!("/attempts/steps/{index}/work/{name}"),optional_count(value),false)).collect(),vec![]);
        }
        if step.accepted==Some(false) {
            builder.add(StatementKind::Established,"invocation.original_refused","The original owner refused this step. A native return or available timing does not convert the invocation into a scientifically successful comparison endpoint.",
                vec![counted(doc,format!("/attempts/steps/{index}/accepted"),Value::Boolean(false),false)],vec![]);
        }
        if step.omitted_decisions>0 || step.dropped_progress.is_some_and(|count|count>0) {
            builder.add(StatementKind::Limitation,"invocation.strategy_history_partial","Decision-prefix omissions and lossy progress are distinct from synchronous terminal work counters. Neither reconstructs missing retries or continuous stagnation.",
                vec![counted(doc,format!("/attempts/steps/{index}/omitted_decisions"),Value::Count(step.omitted_decisions as u64),step.omitted_decisions>0),
                    counted(doc,format!("/attempts/steps/{index}/dropped_progress"),optional_count(step.dropped_progress),step.dropped_progress.is_some_and(|count|count>0))],vec![NextObservation::TimelineCapture]);
        }
        builder.omitted(step.decisions.len());
        let started=step.decisions.iter().take(limit).filter(|event|event.kind==pse_model::generated::enums::NumericalEventKind::Started).count();
        if started>1 {
            builder.add(StatementKind::Established,"invocation.strategy_multiple_starts","More than one strategy operation started in this retained step. These may be declared stages or recovery/retry; their inclusive work must not be charged or summed again.",
                step.decisions.iter().take(limit).enumerate().filter(|(_,event)|event.kind==pse_model::generated::enums::NumericalEventKind::Started).take(4)
                    .map(|(event,decision)|counted(doc,format!("/attempts/steps/{index}/decisions/{event}/mechanism"),Value::Count(decision.mechanism as u64),step.omitted_decisions>0)).collect(),vec![NextObservation::InspectRelated]);
        }
        for (event,decision) in step.decisions.iter().take(limit).enumerate().filter(|(_,event)|matches!(event.kind,pse_model::generated::enums::NumericalEventKind::Refused|pse_model::generated::enums::NumericalEventKind::Abandoned)) {
            builder.add(StatementKind::Established,"invocation.strategy_refusal","A strategy operation was refused or abandoned at this recorded phase. The event is not evidence that all other stages failed or that later original permission was established.",
                vec![counted(doc,format!("/attempts/steps/{index}/decisions/{event}/kind"),text(decision.kind.as_str()),step.omitted_decisions>0),
                    counted(doc,format!("/attempts/steps/{index}/decisions/{event}/phase"),text(decision.phase.as_str()),step.omitted_decisions>0)],vec![]);
        }
    }
    builder.add(StatementKind::Limitation,"invocation.opaque_cost","This invocation document does not supply exclusive callback/inner/factor CPU or scheduler blocking intervals. Native inclusive wall, supervisor wall and process CPU cannot establish a causal unit-cost increase or wait by subtraction.",
        vec![inv(doc,"/endpoints/terminal_seconds",seconds(doc.endpoints.terminal_seconds))],vec![NextObservation::InspectRelated,NextObservation::CpuProfile,NextObservation::OptionalWaitProbe]);
}
/// Explain an actual retained invocation without model evaluation, store access or native calls.
pub fn explain_invocation(doc:&InvocationEvidenceDocument,policy:ComparisonPolicy)->InvocationComparisonDocument {
    let mut builder=Builder::new(doc,None,policy);
    if source_valid(&mut builder,doc) {explain(&mut builder,doc);}
    builder.document
}
fn same_products(a:&InvocationEvidenceDocument,b:&InvocationEvidenceDocument,limit:usize)->bool {
    let unique=|doc:&InvocationEvidenceDocument|doc.products.iter().take(limit).enumerate()
        .all(|(index,product)|!doc.products[..index].iter().any(|prior|prior.role==product.role&&prior.step==product.step));
    unique(a) && unique(b) && a.products.len()<=limit && b.products.len()<=limit && a.products.len()==b.products.len() && a.omitted_product_steps==0 && b.omitted_product_steps==0 && a.products.iter().all(|left| {
        b.products.iter().any(|right|left.role==right.role && left.step==right.step &&
            (left.availability==ProductAvailability::NotRequested)==(right.availability==ProductAvailability::NotRequested))
    })
}
fn comparable(a:&InvocationEvidenceDocument,b:&InvocationEvidenceDocument,limit:usize)->bool {
    a.family==b.family && same_products(a,b,limit) && a.attempts.requested_steps==b.attempts.requested_steps &&
        a.outcome==InvocationOutcome::Completed && b.outcome==InvocationOutcome::Completed && a.attempts.usable && b.attempts.usable &&
        a.products.iter().chain(&b.products).all(|product|product.availability!=ProductAvailability::Unavailable)
}
/// Compare recorded populations and owner clocks. No winner or causal numerical verdict is inferred.
pub fn compare_invocations(a:&InvocationEvidenceDocument,b:&InvocationEvidenceDocument,policy:ComparisonPolicy)->InvocationComparisonDocument {
    let mut builder=Builder::new(a,Some(b),policy);
    let valid_a=source_valid(&mut builder,a);
    let valid_b=source_valid(&mut builder,b);
    if !valid_a || !valid_b {return builder.document;}
    let limit=builder.document.policy.source_items;
    builder.document.partial=source_partial(a)||source_partial(b);
    retained_failure(&mut builder,a);retained_failure(&mut builder,b);
    builder.omitted(a.products.len());builder.omitted(b.products.len());
    for (ai,left) in a.products.iter().take(limit).enumerate() {
        let Some((bi,right))=b.products.iter().take(limit).enumerate().find(|(_,right)|right.role==left.role&&right.step==left.step) else{continue;};
        if left.availability!=right.availability {
            builder.add(StatementKind::Established,"comparison.product_availability_changed","The governing availability of this requested product changed. Response readiness or an unchanged native status cannot substitute for the product's original permission.",
                vec![counted(a,format!("/products/{ai}/role"),text(&left.role),false),counted(a,format!("/products/{ai}/availability"),availability(left.availability),false),
                    counted(b,format!("/products/{bi}/role"),text(&right.role),false),counted(b,format!("/products/{bi}/availability"),availability(right.availability),false)],vec![NextObservation::InspectRelated]);
        }
    }
    if !comparable(a,b,limit) {
        builder.add(StatementKind::Limitation,"comparison.functional_scope_changed","Family, requested products, original usability or outcome differs. Literal recorded work can be inspected, but these endpoints are not ranked as one performance winner.",
            vec![counted(a,"/family",text(&a.family),false),counted(b,"/family",text(&b.family),false),
                inv(a,"/outcome",outcome(a.outcome)),inv(b,"/outcome",outcome(b.outcome)),
                counted(a,"/attempts/usable",Value::Boolean(a.attempts.usable),false),counted(b,"/attempts/usable",Value::Boolean(b.attempts.usable),false)],vec![NextObservation::InspectRelated]);
    }
    if !same_products(a,b,limit) {
        let mut fields=vec![counted(a,"/omitted_product_steps",Value::Count(a.omitted_product_steps as u64),a.omitted_product_steps>0),
            counted(b,"/omitted_product_steps",Value::Count(b.omitted_product_steps as u64),b.omitted_product_steps>0)];
        for (doc,other) in [(a,b),(b,a)] {
            if let Some((index,product))=doc.products.iter().take(limit).enumerate().find(|(_,product)|!other.products.iter().take(limit).any(|candidate|candidate.role==product.role&&candidate.step==product.step&&
                (candidate.availability==ProductAvailability::NotRequested)==(product.availability==ProductAvailability::NotRequested))) {
                fields.push(counted(doc,format!("/products/{index}/role"),text(&product.role),false));
                fields.push(counted(doc,format!("/products/{index}/availability"),availability(product.availability),false));
            }
        }
        builder.add(StatementKind::Limitation,"comparison.requested_products_changed","The recorded product requests differ or their bounded prefix is incomplete. Additional analysis/publication can change work; matching native status alone does not establish equal requested outputs.",
            fields,vec![NextObservation::InspectRelated]);
    }
    if a.clock_scope==b.clock_scope && a.timing_unavailable.is_none() && b.timing_unavailable.is_none() {
        for (name,left,right) in [("qualification_seconds",a.endpoints.qualification_seconds,b.endpoints.qualification_seconds),("terminal_seconds",a.endpoints.terminal_seconds,b.endpoints.terminal_seconds),("response_ready_seconds",a.endpoints.response_ready_seconds,b.endpoints.response_ready_seconds)] {
            let observed=left.zip(right).filter(|(left,right)|left.is_finite()&&right.is_finite()&&*left>=0.&&*right>=0.);
            builder.add(if observed.is_some(){StatementKind::Established}else{StatementKind::Limitation},"comparison.supervisor_endpoint",
                observed.map_or_else(||"A comparable supervisor endpoint is unavailable; missing qualification is not zero qualification time.".into(),|(left,right)|format!("The same recorded supervisor endpoint changed from {left:.6} to {right:.6} seconds. It excludes work outside the declared clock scope and does not isolate a cause.")),
                vec![inv(a,&format!("/endpoints/{name}"),seconds(left)),inv(b,&format!("/endpoints/{name}"),seconds(right))],vec![]);
        }
    } else {
        builder.add(StatementKind::Limitation,"comparison.clock_scope_unavailable","The supervisor clocks have different boundaries or collection was unavailable. Their values cannot establish a shared elapsed-cost comparison.",vec![inv(a,"/clock_scope",text(&a.clock_scope)),inv(b,"/clock_scope",text(&b.clock_scope))],vec![NextObservation::TimelineCapture]);
    }
    builder.omitted(a.attempts.steps.len());builder.omitted(b.attempts.steps.len());
    if a.attempts.steps.is_empty() || b.attempts.steps.is_empty() {
        builder.add(StatementKind::Limitation,"comparison.unit_cost_unavailable","No paired native step population is retained. Integration-only or omitted records cannot establish native per-unit cost from a supervisor endpoint.",
            vec![counted(a,"/attempts/omitted_steps",Value::Count(a.attempts.omitted_steps as u64),true),counted(b,"/attempts/omitted_steps",Value::Count(b.attempts.omitted_steps as u64),true)],vec![NextObservation::InspectRelated,NextObservation::TimelineCapture]);
    }
    for (ai,left) in a.attempts.steps.iter().take(limit).enumerate() {
        let Some((bi,right))=b.attempts.steps.iter().take(limit).enumerate().find(|(_,right)|right.step==left.step) else{continue;};
        context_comparison(&mut builder,a,ai,left,b,bi,right);
        let route_equal=left.backend==right.backend;
        if !route_equal {
            builder.add(StatementKind::Established,"comparison.route_changed","The actual native backend changed for this original step. Its family-specific evaluation/iteration units and elapsed costs are not interchangeable.",
                vec![counted(a,format!("/attempts/steps/{ai}/backend"),left.backend.map_or_else(||Value::Unavailable("native route unavailable".into()),|backend|text(backend.as_str())),false),
                    counted(b,format!("/attempts/steps/{bi}/backend"),right.backend.map_or_else(||Value::Unavailable("native route unavailable".into()),|backend|text(backend.as_str())),false)],vec![NextObservation::InspectRelated]);
        }
        if let Some((lw,rw))=left.work.zip(right.work) {
            for (name,l,r) in [("attempts",Some(lw.attempts),Some(rw.attempts)),("evaluations",lw.evaluations,rw.evaluations),("iterations",lw.iterations,rw.iterations),("factorizations",lw.factorizations,rw.factorizations),("proof_steps",lw.proof_steps,rw.proof_steps)] {
                if l!=r {
                    builder.add(if l.is_some()&&r.is_some(){StatementKind::Established}else{StatementKind::Limitation},"comparison.recorded_work_changed",
                        "The synchronous owner recorded different work populations for this step. These inclusive counts are not added to native/decision counts; a route or work-mix change can alter their meaning and does not prove higher per-unit cost.",
                        vec![counted(a,format!("/attempts/steps/{ai}/work/{name}"),optional_count(l),false),counted(b,format!("/attempts/steps/{bi}/work/{name}"),optional_count(r),false)],vec![NextObservation::InspectRelated]);
                }
            }
        }
        let native=left.native_observations.as_deref().zip(right.native_observations.as_deref());
        let matching=native.filter(|(l,r)|route_equal&&left.backend==Some(l.backend)&&right.backend==Some(r.backend)&&l.backend==r.backend&&l.problem==r.problem&&l.profile==r.profile&&l.coordinates==r.coordinates&&left.effective_options==right.effective_options);
        let ratio=matching.and_then(|(l,r)|match (&l.counters,&r.counters) {
            (pse_backend_native::solve::NativeCounters::Ipopt{iterations:Some(li),wall_seconds:Some(lt),..},pse_backend_native::solve::NativeCounters::Ipopt{iterations:Some(ri),wall_seconds:Some(rt),..})
                if *li>0 && *ri>0 && lt.is_finite()&&rt.is_finite()&&*lt>=0.&&*rt>=0.=>Some((*li,*lt,*ri,*rt)),_=>None,
        });
        if let Some((li,lt,ri,rt))=ratio.filter(|_|comparable(a,b,limit)&&left.accepted==Some(true)&&right.accepted==Some(true)) {
            let mut fields=vec![evidence(a,format!("/attempts/steps/{ai}/native_observations/counters/wall_seconds/value"),seconds(Some(lt)),"Ipopt native algorithm wall including callbacks",false),
                counted(a,format!("/attempts/steps/{ai}/native_observations/counters/iterations"),Value::Count(li),false),
                evidence(b,format!("/attempts/steps/{bi}/native_observations/counters/wall_seconds/value"),seconds(Some(rt)),"Ipopt native algorithm wall including callbacks",false),
                counted(b,format!("/attempts/steps/{bi}/native_observations/counters/iterations"),Value::Count(ri),false)];
            if let Some((left,right))=matching {
                for (doc,index,native) in [(a,ai,left),(b,bi,right)] {
                    fields.push(counted(doc,format!("/attempts/steps/{index}/native_observations/problem"),text(&native.problem.to_string()),false));
                    fields.push(counted(doc,format!("/attempts/steps/{index}/native_observations/profile"),text(&native.profile.to_string()),false));
                }
            }
            builder.add(StatementKind::Established,"comparison.native_inclusive_seconds_per_iteration",format!("For each retained native report with matching recorded structural problem/profile/backend/options, inclusive wall per reported iteration was {:.6} versus {:.6} seconds. This describes whole-loop averages; changed callback/factor work mix prevents a causal unit-cost or exclusive CPU conclusion.",lt/li as f64,rt/ri as f64),
                fields,vec![NextObservation::InspectRelated,NextObservation::CpuProfile]);
        } else {
            builder.add(StatementKind::Limitation,"comparison.unit_cost_unavailable","Matching complete native clock/count populations and problem/profile/backend/options were not established, or scientific endpoints differ. Counter absence is not zero cost. Inspect same-product callback/inner/factor durations and completed counts to distinguish extra work from more expensive work.",
                vec![counted(a,format!("/attempts/steps/{ai}/native_observations"),Value::Unavailable("compatible native wall/count population not established".into()),true),
                    counted(b,format!("/attempts/steps/{bi}/native_observations"),Value::Unavailable("compatible native wall/count population not established".into()),true)],vec![NextObservation::InspectRelated,NextObservation::CpuProfile]);
        }
    }
    if let Some((left,right))=a.preparations.zip(b.preparations) {
        for (name,l,r) in [("views",left.views,right.views),("observations",left.observations,right.observations),("rebuilt",left.rebuilt,right.rebuilt),("shared",left.shared,right.shared)] {
            if l!=r {
                builder.add(StatementKind::Established,"comparison.preparation_population_changed","Task-local preparation/rebind work changed. This may move work outside or inside the supervisor boundary; counts alone cannot measure displaced time or a cold/warm speedup.",
                    vec![counted(a,format!("/preparations/{name}"),Value::Count(l as u64),false),counted(b,format!("/preparations/{name}"),Value::Count(r as u64),false)],vec![NextObservation::TimelineCapture]);
            }
        }
    } else {
        builder.add(StatementKind::Limitation,"comparison.preparation_unavailable","Task-local preparation coverage was not established for both invocations. Missing preparation counts are unavailable, not zero work or proof that setup was displaced.",
            vec![counted(a,"/preparations",Value::Unavailable("paired preparation population not established".into()),a.preparations.is_none()),
                counted(b,"/preparations",Value::Unavailable("paired preparation population not established".into()),b.preparations.is_none())],vec![NextObservation::TimelineCapture]);
    }
    builder.omitted(a.intervals.len());builder.omitted(b.intervals.len());
    for (ai,left) in a.intervals.iter().take(limit).enumerate() {
        let ordinal=a.intervals[..ai].iter().filter(|item|item.stage==left.stage).count();
        let Some((bi,right))=b.intervals.iter().take(limit).enumerate().filter(|(_,item)|item.stage==left.stage).nth(ordinal) else{continue;};
        let duration=|interval:&crate::invocation::InvocationInterval|interval.ended_seconds.filter(|end|end.is_finite()&&interval.started_seconds.is_finite()&&interval.started_seconds>=0.&&*end>=interval.started_seconds).map(|end|end-interval.started_seconds);
        if a.clock_scope==b.clock_scope && a.timing_unavailable.is_none() && b.timing_unavailable.is_none()
            && let Some((l,r))=duration(left).zip(duration(right)) && l!=r {
            builder.add(StatementKind::Established,"comparison.owner_interval_changed",format!("Matching named owner intervals changed from {l:.6} to {r:.6} inclusive seconds. This identifies observed preparation/inner/postsolve displacement only at the supplied owner boundary, not exclusive wall contributions or a causal wait."),
                vec![inv(a,&format!("/intervals/{ai}/stage"),text(&left.stage)),inv(a,&format!("/intervals/{ai}/started_seconds"),seconds(Some(left.started_seconds))),inv(a,&format!("/intervals/{ai}/ended_seconds"),seconds(left.ended_seconds)),
                    inv(b,&format!("/intervals/{bi}/stage"),text(&right.stage)),inv(b,&format!("/intervals/{bi}/started_seconds"),seconds(Some(right.started_seconds))),inv(b,&format!("/intervals/{bi}/ended_seconds"),seconds(right.ended_seconds))],vec![NextObservation::InspectRelated]);
        }
    }
    if source_partial(a)||source_partial(b) {
        builder.add(StatementKind::Limitation,"comparison.coverage_partial","One retained invocation is a partial prefix. Missing attempts, products and intervals are not zero work; aggregate native/strategy clocks are not summed to fill gaps.",
            vec![counted(a,"/attempts/omitted_native_attempts",Value::Count(a.attempts.omitted_native_attempts as u64),source_partial(a)),counted(b,"/attempts/omitted_native_attempts",Value::Count(b.attempts.omitted_native_attempts as u64),source_partial(b))],vec![NextObservation::TimelineCapture]);
    }
    strategy_comparison(&mut builder,a);strategy_comparison(&mut builder,b);
    builder.add(StatementKind::Limitation,"comparison.scientific_input_context_unknown","Matching recorded structural problem/profile/options does not establish equal parameters, fixed values, coefficients or bounds. Seed data identifies the start only. Literal counts and inclusive means remain descriptive; causal cost attribution requires the relevant supplied numerical context.",vec![
        counted(a,"/attempts/run_id",text(&a.attempts.run_id.to_string()),false),counted(b,"/attempts/run_id",text(&b.attempts.run_id.to_string()),false)],vec![NextObservation::InspectRelated]);
    builder.add(StatementKind::Limitation,"comparison.causal_cost_unavailable","A larger wall endpoint or unchanged checkpoint alone cannot distinguish computation, contention, opaque native work or continuous stagnation. Selected same-product owner counts, CPU stacks and scheduler evidence can discriminate the remaining cause.",
        vec![inv(a,"/endpoints/terminal_seconds",seconds(a.endpoints.terminal_seconds)),inv(b,"/endpoints/terminal_seconds",seconds(b.endpoints.terminal_seconds))],vec![NextObservation::InspectRelated,NextObservation::CpuProfile,NextObservation::OptionalWaitProbe]);
    builder.document
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{invocation::{InvocationEndpoints,InvocationInterval,InvocationProduct},workflow::{AttemptSummary,AttemptStepSummary,AttemptIntervals,AttemptDecision}};
    use pse_backend_native::solve::{Backend,NativeCounters,NativeObservations,ObservationCoordinates,StopFidelity};
    use pse_ids::ContentHash;
    use pse_model::{strategy::{Phase,WorkObservation},generated::enums::NumericalEventKind};
    use std::sync::Arc;
    fn fixture(id:u8,evaluations:u64,wall:f64)->InvocationEvidenceDocument {
        let run_id=RunId::from_bytes([id;16]);
        let mut attempts=AttemptSummary::empty_for_test(run_id);
        attempts.requested_steps=1;
        attempts.steps.push(AttemptStepSummary {
            step:0,backend:Some(Backend::Ipopt),observation:None,accepted:Some(true),
            work:Some(WorkObservation{attempts:1,evaluations:Some(evaluations),iterations:Some(4),factorizations:Some(2),proof_steps:None}),
            start_submitted:Some(false),reused_native_state:Some(false),normalized_physical_violation:None,
            effective_options:Default::default(),native_defaults:Default::default(),start:None,intervals:AttemptIntervals::default(),
            decisions:Vec::new(),omitted_decisions:0,findings:Vec::new(),omitted_findings:0,dropped_progress:Some(0),raw_map:None,
            native_observations:Some(Arc::new(NativeObservations {backend:Backend::Ipopt,problem:ContentHash::from_bytes([11;32]),profile:ContentHash::from_bytes([12;32]),
                coordinates:ObservationCoordinates::SubmittedOracle,stop_fidelity:StopFidelity::EvaluationAndIterationCheckpoints,current:None,
                counters:NativeCounters::Ipopt{iterations:Some(4),objective_evaluations:Some(evaluations),constraint_evaluations:Some(0),gradient_evaluations:Some(4),jacobian_evaluations:Some(4),hessian_evaluations:Some(4),cpu_seconds:None,system_seconds:None,wall_seconds:Some(wall)}})),
        });
        InvocationEvidenceDocument {version:1,run_id,attempt_id:None,operation:None,family:"modeling".into(),clock_scope:"fixture supervisor wall from entry; preparation before entry excluded".into(),timing_unavailable:None,
            canonical_run:None,canonical_attempt:None,outcome:InvocationOutcome::Completed,
            endpoints:InvocationEndpoints{qualification_seconds:Some(wall+1.),terminal_seconds:Some(wall+2.),response_ready_seconds:Some(wall+3.)},
            intervals:vec![InvocationInterval{stage:"native_join_and_workflow_postsolve".into(),started_seconds:1.,ended_seconds:Some(wall),outcome:Some(InvocationOutcome::Completed)}],omitted_intervals:0,
            products:vec![InvocationProduct{role:"solution".into(),step:Some(0),availability:ProductAvailability::Available}],omitted_product_steps:0,
            preparations:Some(crate::math::PreparationCounts{views:1,observations:1,rebuilt:0,shared:0}),attempts:Arc::new(attempts)}
    }
    fn step(doc:&mut InvocationEvidenceDocument)->&mut AttemptStepSummary {&mut Arc::get_mut(&mut doc.attempts).unwrap().steps[0]}
    fn has(doc:&InvocationComparisonDocument,code:&str)->bool {doc.statements.iter().any(|statement|statement.code==code)}
    fn statement<'a>(doc:&'a InvocationComparisonDocument,code:&str)->&'a InvocationComparisonStatement {
        doc.statements.iter().find(|statement|statement.code==code).unwrap()
    }
    #[test]
    fn invocation_comparison_distinguishes_recorded_more_work_from_descriptive_unit_ratio() {
        let a=fixture(1,10,8.);
        let b=fixture(2,20,8.);
        let result=compare_invocations(&a,&b,ComparisonPolicy::default());
        let work=statement(&result,"comparison.recorded_work_changed");
        assert_eq!(work.kind,StatementKind::Established);
        assert_eq!(work.evidence[0].value,Value::Count(10));
        assert_eq!(work.evidence[1].value,Value::Count(20));
        assert_eq!(work.evidence[0].field,"/attempts/steps/0/work/evaluations");
        let ratio=statement(&result,"comparison.native_inclusive_seconds_per_iteration");
        assert!(ratio.message.contains("2.000000 versus 2.000000"));
        assert!(ratio.message.contains("prevents a causal unit-cost"));
        assert_eq!(ratio.evidence[0].invocation.run_id,a.run_id);
        assert_eq!(ratio.evidence[2].invocation.run_id,b.run_id);
        assert!(ratio.evidence.iter().all(|field|field.invocation.operation.is_none()));
        assert!(has(&result,"comparison.causal_cost_unavailable"));
    }
    #[test]
    fn invocation_comparison_higher_inclusive_mean_is_not_exclusive_or_causal_cost() {
        let a=fixture(1,10,8.);
        let b=fixture(2,10,12.);
        let result=compare_invocations(&a,&b,ComparisonPolicy::default());
        assert!(!has(&result,"comparison.recorded_work_changed"));
        let ratio=statement(&result,"comparison.native_inclusive_seconds_per_iteration");
        assert!(ratio.message.contains("2.000000 versus 3.000000"));
        assert!(ratio.message.contains("whole-loop averages"));
        assert!(ratio.evidence[0].clock_scope.contains("including callbacks"));
        let interval=statement(&result,"comparison.owner_interval_changed");
        assert!(interval.message.contains("7.000000 to 11.000000"));
        assert!(interval.message.contains("not exclusive"));
    }
    #[test]
    fn invocation_comparison_changed_route_profile_problem_options_and_missing_counter_refuse_ratio() {
        let a=fixture(1,10,8.);
        for change in 0..7 {
            let mut b=fixture(2,10,12.);
            match change {
                0=>step(&mut b).backend=Some(Backend::Uno),
                1=>Arc::get_mut(step(&mut b).native_observations.as_mut().unwrap()).unwrap().profile=ContentHash::from_bytes([91;32]),
                2=>Arc::get_mut(step(&mut b).native_observations.as_mut().unwrap()).unwrap().problem=ContentHash::from_bytes([92;32]),
                3=>{step(&mut b).effective_options.insert("actual_changed_option".into(),pse_backend_native::solve::OptionValue::Integer(2));},
                change=>{
                    let native=Arc::get_mut(step(&mut b).native_observations.as_mut().unwrap()).unwrap();
                    if let NativeCounters::Ipopt{iterations,wall_seconds,..}=&mut native.counters {
                        if change==4 {*iterations=None;} else if change==5 {*iterations=Some(0);} else {*wall_seconds=Some(f64::NAN);}
                    }
                }
            }
            let result=compare_invocations(&a,&b,ComparisonPolicy::default());
            assert!(!has(&result,"comparison.native_inclusive_seconds_per_iteration"),"changed premise {change}");
            let withheld=statement(&result,"comparison.unit_cost_unavailable");
            assert_eq!(withheld.kind,StatementKind::Limitation);
            assert!(!withheld.next.is_empty());
            assert!(withheld.evidence.iter().all(|field|matches!(field.value,Value::Unavailable(_))));
            if change==0 {assert!(has(&result,"comparison.route_changed"));}
        }
    }
    #[test]
    fn invocation_comparison_changed_requested_outputs_and_original_outcomes_are_not_winners() {
        let a=fixture(1,10,8.);
        for change in 0..4 {
            let mut b=fixture(2,10,8.);
            match change {
                0=>b.products.push(InvocationProduct{role:"response".into(),step:Some(0),availability:ProductAvailability::Available}),
                1=>b.products[0].availability=ProductAvailability::Unavailable,
                2=>b.outcome=InvocationOutcome::Failed,
                _=>Arc::get_mut(&mut b.attempts).unwrap().usable=false,
            }
            let result=compare_invocations(&a,&b,ComparisonPolicy::default());
            assert!(has(&result,"comparison.functional_scope_changed"));
            assert!(!has(&result,"comparison.native_inclusive_seconds_per_iteration"));
            if change==0 {assert!(has(&result,"comparison.requested_products_changed"));}
            if change==1 {assert!(has(&result,"comparison.product_availability_changed"));}
        }
    }
    #[test]
    fn invocation_comparison_partial_progress_is_not_partial_terminal_getter_or_stagnation() {
        let a=fixture(1,10,8.);
        let mut b=fixture(2,10,8.);
        step(&mut b).dropped_progress=Some(7);
        step(&mut b).omitted_decisions=2;
        Arc::get_mut(&mut b.attempts).unwrap().omitted_native_attempts=3;
        let result=compare_invocations(&a,&b,ComparisonPolicy::default());
        assert!(result.partial);
        assert!(has(&result,"comparison.coverage_partial"));
        assert!(has(&result,"comparison.strategy_history_partial"));
        assert!(has(&result,"comparison.native_inclusive_seconds_per_iteration"),"complete native terminal getters remain a scoped descriptive average");
        assert!(statement(&result,"comparison.strategy_history_partial").message.contains("not by themselves invalidate"));
    }
    #[test]
    fn invocation_comparison_failed_strategy_preserves_original_permission_and_preparation_scope() {
        let a=fixture(1,10,8.);
        let mut b=fixture(2,20,12.);
        let event=|mechanism,kind|AttemptDecision{mechanism,kind,phase:Phase::Native,decision:None,observation:None,transition:None,permission:None,original:None,work:None};
        step(&mut b).decisions=vec![event(0,NumericalEventKind::Started),event(0,NumericalEventKind::Refused),event(1,NumericalEventKind::Started)];
        step(&mut b).accepted=Some(false);
        Arc::get_mut(&mut b.attempts).unwrap().usable=false;
        b.preparations.as_mut().unwrap().views=3;
        let result=compare_invocations(&a,&b,ComparisonPolicy::default());
        assert!(has(&result,"comparison.strategy_history"));
        assert!(has(&result,"comparison.preparation_population_changed"));
        assert!(has(&result,"comparison.functional_scope_changed"));
        assert!(!has(&result,"comparison.native_inclusive_seconds_per_iteration"));
        assert!(statement(&result,"comparison.strategy_history").message.contains("not automatically retries"));
        let single=explain_invocation(&b,ComparisonPolicy::default());
        assert!(has(&single,"invocation.original_refused"));
        assert!(has(&single,"invocation.strategy_refusal"));
        assert!(has(&single,"invocation.strategy_multiple_starts"));
    }
    #[test]
    fn invocation_comparison_is_passive_bounded_and_links_existing_source_fields() {
        let a=fixture(1,10,8.);
        let b=fixture(2,20,12.);
        let before=serde_json::to_vec(&a).unwrap();
        let result=compare_invocations(&a,&b,ComparisonPolicy::default());
        let source_a=serde_json::to_value(&a).unwrap();
        let source_b=serde_json::to_value(&b).unwrap();
        for statement in &result.statements {
            assert!(!statement.evidence.is_empty());
            assert!(statement.message.len()<=TEXT_LIMIT);
            for evidence in &statement.evidence {
                let source=if evidence.invocation.run_id==a.run_id {&source_a}else{&source_b};
                let actual=source.pointer(&evidence.field).unwrap_or_else(||panic!("{}",evidence.field));
                match &evidence.value {
                    Value::Number(value)=>assert_eq!(actual.as_f64(),Some(*value),"{}",evidence.field),
                    Value::Count(value)=>assert_eq!(actual.as_u64(),Some(*value),"{}",evidence.field),
                    Value::Integer(value)=>assert_eq!(actual.as_i64(),Some(*value),"{}",evidence.field),
                    Value::Boolean(value)=>assert_eq!(actual.as_bool(),Some(*value),"{}",evidence.field),
                    Value::Text(value)=>assert_eq!(actual.as_str(),Some(value.as_str()),"{}",evidence.field),
                    _=>{},
                }
                assert!(!evidence.clock_scope.is_empty());
            }
        }
        assert_eq!(before,serde_json::to_vec(&a).unwrap());
        let bounded=compare_invocations(&a,&b,ComparisonPolicy{statements:2,source_items:0});
        assert!(bounded.statements.len()<=2);
        assert!(bounded.omitted_statements>0);
        assert!(bounded.omitted_source_items>0);
        assert!(bounded.partial);
        let encoded=serde_json::to_vec(&result).unwrap();
        let decoded:InvocationComparisonDocument=serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded.baseline.run_id,a.run_id);
        assert_eq!(decoded.candidate.unwrap().run_id,b.run_id);
    }
    fn seed(data:u8)->crate::workflow::AttemptStartSummary {
        crate::workflow::AttemptStartSummary{previous_attempt:Some(0),layout:Some(ContentHash::from_bytes([17;32])),data:Some(ContentHash::from_bytes([data;32])),transformations:Vec::new(),sparse_variables:3,submitted:true}
    }
    #[test]
    fn invocation_comparison_seed_submission_and_reuse_are_actual_context_not_factor_reuse() {
        let mut a=fixture(1,10,8.);
        step(&mut a).start=Some(seed(21));
        let mut b=fixture(2,10,12.);
        step(&mut b).start=Some(seed(22));
        step(&mut b).start_submitted=Some(true);
        step(&mut b).reused_native_state=Some(true);
        let result=compare_invocations(&a,&b,ComparisonPolicy::default());
        assert!(has(&result,"comparison.seed_context_changed"));
        let context=statement(&result,"comparison.start_or_reuse_context_changed");
        assert!(context.message.contains("does not establish numeric factor reuse"));
        assert!(has(&result,"comparison.native_inclusive_seconds_per_iteration"),"changed context limits causal attribution, not literal descriptive means");
        assert!(has(&result,"comparison.scientific_input_context_unknown"));
        let source=serde_json::to_value(&b).unwrap();
        for field in &statement(&result,"comparison.seed_context_changed").evidence {
            if field.invocation.run_id==b.run_id && let Value::Text(value)=&field.value {
                assert_eq!(source.pointer(&field.field).and_then(serde_json::Value::as_str),Some(value.as_str()));
            }
        }
        let single=explain_invocation(&b,ComparisonPolicy::default());
        let predicates=statement(&single,"invocation.start_and_reuse");
        assert_eq!(predicates.evidence[0].value,Value::Boolean(true));
        assert_eq!(predicates.evidence[1].value,Value::Boolean(true));
        assert!(has(&single,"invocation.start_metadata"));
        // Each supplied scalar dimension is independently decision-relevant;
        // data identity alone must not erase layout/submission/prior context.
        for change in 0..4 {
            let mut candidate=fixture(3,10,8.);
            step(&mut candidate).start=Some(seed(21));
            let start=step(&mut candidate).start.as_mut().unwrap();
            match change {
                0=>start.layout=Some(ContentHash::from_bytes([19;32])),
                1=>start.previous_attempt=Some(2),
                2=>start.sparse_variables=4,
                _=>start.submitted=false,
            }
            assert!(has(&compare_invocations(&a,&candidate,ComparisonPolicy::default()),"comparison.seed_context_changed"),"metadata dimension {change}");
        }
        step(&mut b).start_submitted=None;
        step(&mut b).reused_native_state=None;
        step(&mut b).start=None;
        let missing=explain_invocation(&b,ComparisonPolicy::default());
        assert_eq!(statement(&missing,"invocation.start_and_reuse").kind,StatementKind::Limitation);
        assert!(has(&missing,"invocation.start_metadata_unavailable"));
    }
    #[test]
    fn invocation_comparison_queried_defaults_are_distinct_and_exact_bounded_leaves() {
        let a=fixture(1,10,8.);
        let mut b=fixture(2,10,8.);
        step(&mut b).native_defaults.insert("default/~mode".into(),pse_backend_native::solve::OptionValue::Integer(9));
        step(&mut b).effective_options.insert("submitted_only".into(),pse_backend_native::solve::OptionValue::Bool(true));
        let result=compare_invocations(&a,&b,ComparisonPolicy::default());
        assert!(has(&result,"comparison.native_settings_changed"));
        let default=statement(&result,"invocation.queried_default");
        assert_eq!(default.evidence[0].field,"/attempts/steps/0/native_defaults/default~1~0mode/value");
        assert_eq!(default.evidence[0].value,Value::Integer(9));
        let source=serde_json::to_value(&b).unwrap();
        assert_eq!(source.pointer(&default.evidence[0].field).and_then(serde_json::Value::as_i64),Some(9));
        assert!(has(&result,"invocation.native_defaults_unavailable"));
        let submitted=statement(&result,"invocation.submitted_option");
        assert_eq!(submitted.evidence[0].value,Value::Boolean(true));
        let limited=compare_invocations(&a,&b,ComparisonPolicy{statements:64,source_items:0});
        assert!(!has(&limited,"invocation.queried_default"));
        assert!(limited.partial);
    }
    #[test]
    fn invocation_comparison_accounts_for_skipped_retained_decisions() {
        let mut a=fixture(1,10,8.);
        let b=fixture(2,10,8.);
        let event=|mechanism|AttemptDecision{mechanism,kind:NumericalEventKind::Started,phase:Phase::Native,decision:None,observation:None,transition:None,permission:None,original:None,work:None};
        step(&mut a).decisions=vec![event(0),event(1),event(2)];
        let result=compare_invocations(&a,&b,ComparisonPolicy{statements:64,source_items:1});
        assert_eq!(result.omitted_source_items,2);
        assert!(result.partial,"the interpreter itself inspected only one retained decision");
        assert!(!has(&result,"comparison.strategy_history"),"one inspected start does not establish multiple starts");
    }
    #[test]
    fn invocation_comparison_retains_failure_before_any_step() {
        let mut a=fixture(1,10,8.);
        let b=fixture(2,10,8.);
        a.outcome=InvocationOutcome::Failed;
        let attempts=Arc::get_mut(&mut a.attempts).unwrap();
        attempts.steps.clear();
        attempts.usable=false;
        attempts.failure=Some(crate::workflow::AttemptFailureSummary{
            observation:pse_model::generated::enums::NumericalAttemptObservation::CapabilityRefusal,
            code:pse_model::diagnostic::DiagnosticCode::AuthoringReference,
            class:pse_model::diagnostic::BoundaryClass::Unsupported,retained_only:true});
        let result=compare_invocations(&a,&b,ComparisonPolicy::default());
        let failure=statement(&result,"invocation.retained_failure");
        assert_eq!(failure.kind,StatementKind::Established);
        let source=serde_json::to_value(&a).unwrap();
        for field in &failure.evidence {
            match &field.value {
                Value::Text(value)=>assert_eq!(source.pointer(&field.field).and_then(serde_json::Value::as_str),Some(value.as_str())),
                Value::Boolean(value)=>assert_eq!(source.pointer(&field.field).and_then(serde_json::Value::as_bool),Some(*value)),
                _=>panic!("failure fields must retain source-native primitive values"),
            }
        }
        assert!(has(&explain_invocation(&a,ComparisonPolicy::default()),"invocation.retained_failure"));
        assert!(!has(&result,"comparison.native_inclusive_seconds_per_iteration"));
    }
    #[test]
    fn invocation_comparison_unavailable_timing_open_interval_and_wrong_join_refuse_inference() {
        let a=fixture(1,10,8.);
        let mut b=fixture(2,10,12.);
        b.timing_unavailable=Some(pse_diagnostics::observation::RecordAvailability::Unavailable);
        b.intervals[0].ended_seconds=None;
        b.preparations=None;
        let single=explain_invocation(&b,ComparisonPolicy::default());
        assert!(has(&single,"invocation.timing_unavailable"));
        assert_eq!(statement(&single,"invocation.inclusive_interval").kind,StatementKind::Limitation);
        let result=compare_invocations(&a,&b,ComparisonPolicy::default());
        assert!(has(&result,"comparison.clock_scope_unavailable"));
        assert!(has(&result,"comparison.preparation_unavailable"));
        assert!(!has(&result,"comparison.owner_interval_changed"));
        Arc::get_mut(&mut b.attempts).unwrap().run_id=RunId::from_bytes([9;16]);
        let invalid=compare_invocations(&a,&b,ComparisonPolicy::default());
        assert_eq!(invalid.statements.len(),1);
        assert_eq!(invalid.statements[0].code,"invocation.source_identity_unavailable");
    }
}
