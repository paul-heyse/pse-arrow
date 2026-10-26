// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded original result locators under one exact immutable study capture.
use crate::{
    canonical::{CanonicalError, MetadataResponse, protected_query},
    canonical_codec as codec,
    canonical_execution::{ResultSetDescriptor, decode_result_descriptors},
    canonical_results::{JoinedResultBlockPage, ResultOutputPage},
    canonical_study_workbench::StudyCapture,
    generated::surreal as wire,
};
use pse_model::generated::runtime::{
    canonical_result_batches::Row as Batch, canonical_result_manifests::Row as Manifest,
};
use surrealdb::types::{Bytes, Object, Value};

/// One exact scientific output predicate, independent of mutable membership.
#[derive(Clone, Debug)]
pub struct CaptureResultRequest {
    /// Semantic relation identity, as recorded by the original producer.
    pub relation: String,
    /// Original selected scientific output identity.
    pub output: String,
    /// Original output partition.
    pub partition: String,
    /// Dense output-index selection rather than scalar block selection.
    pub dense: bool,
    /// Inclusive original row coordinate.
    pub start: u64,
    /// Exclusive original row coordinate.
    pub end: u64,
}
/// Stable continuation within one immutable member subset and exact predicate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CaptureResultCursor {
    /// First selected member still requiring work.
    pub member: Option<u64>,
    /// Last consumed original block or dense-index coordinate for that member.
    pub after: Option<u64>,
}
/// Existing checked native page vocabulary; no scientific decoding is duplicated.
#[derive(Debug)]
pub enum CaptureResultLocators {
    /// Original scalar block page.
    Scalar(JoinedResultBlockPage),
    /// Original dense owner-index page.
    Dense(ResultOutputPage),
}
/// One source's bounded locator prefix, with closed descriptors verified first.
#[derive(Debug)]
pub struct CaptureResultSource {
    /// Stable exact capture member coordinate.
    pub position: u64,
    /// Original closed manifest, including exact absent relation membership.
    pub manifest: Manifest,
    /// The selected declared relation, when present in that manifest.
    pub set: Option<ResultSetDescriptor>,
    /// Original joined locators; absent only when the relation is absent.
    pub locators: Option<CaptureResultLocators>,
    /// Checked exact set permission sharing this capture's one actual owner.
    pub read: Option<CapturedResultRead>,
}
/// One useful page can cover several captured attempts; it grants no new ticket.
#[derive(Debug)]
pub struct CaptureResultPage {
    /// Ordered source prefixes, bounded collectively by native bytes and rows.
    pub sources: Vec<CaptureResultSource>,
    /// Continuation bound to the same request and selected member subset.
    pub next: CaptureResultCursor,
    /// The supplied subset has no remaining locators.
    pub exhausted: bool,
}
fn invalid(message: &str) -> CanonicalError {
    CanonicalError::Configuration(message.into())
}
fn object(value: Value) -> Result<Object, CanonicalError> {
    if let Value::Object(value) = value {
        Ok(value)
    } else {
        Err(CanonicalError::IncompleteResponse)
    }
}
fn optional_uint(value: Value) -> Result<Option<u64>, CanonicalError> {
    match value {
        Value::None | Value::Null => Ok(None),
        value => codec::decode_uint(value).map(Some).map_err(Into::into),
    }
}

fn decode_capture_page(
    value: Value,
    selected: &[u64],
    request: &CaptureResultRequest,
    cursor: CaptureResultCursor,
) -> Result<CaptureResultPage, CanonicalError> {
    let mut page = object(value)?;
    let next = CaptureResultCursor {
        member: optional_uint(codec::required(&mut page, "member")?)?,
        after: optional_uint(codec::required(&mut page, "after")?)?,
    };
    let exhausted = codec::decode_boolean(codec::required(&mut page, "exhausted")?)?;
    let Value::Array(items) = codec::required(&mut page, "items")? else {
        return Err(CanonicalError::IncompleteResponse);
    };
    if !page.is_empty() || items.len() > 64 || (!exhausted && items.is_empty()) {
        return Err(CanonicalError::IncompleteResponse);
    }
    let mut sources = Vec::new();
    let mut previous = None;
    let mut source_index =
        selected.partition_point(|position| cursor.member.is_some_and(|first| *position < first));
    let mut expected_next = cursor;
    let mut partial = false;
    for item in items {
        let mut item = object(item)?;
        let position = codec::decode_uint(codec::required(&mut item, "position")?)?;
        if partial
            || selected.get(source_index) != Some(&position)
            || previous.is_some_and(|old| position <= old)
            || cursor.member.is_some_and(|first| position < first)
        {
            return Err(CanonicalError::IncompleteResponse);
        }
        source_index += 1;
        previous = Some(position);
        let attempt = codec::decode_string(codec::required(&mut item, "attempt")?)?;
        let generation = codec::decode_uint(codec::required(&mut item, "generation")?)?;
        let interpretation = codec::decode_string(codec::required(&mut item, "interpretation")?)?;
        let manifest = wire::decode_canonical_result_manifests(object(codec::required(
            &mut item, "manifest",
        )?)?)?;
        if manifest.attempt != attempt
            || manifest.generation != generation
            || interpretation != wire::INTERPRETATION
        {
            return Err(invalid("captured manifest association differs"));
        }
        let set = decode_result_descriptors(&manifest)?
            .into_iter()
            .find(|set| set.name == request.relation);
        let native_set = codec::required(&mut item, "set")?;
        let native_page = codec::required(&mut item, "page")?;
        let locators = if let Some(set) = &set {
            let mut native_set = object(native_set)?;
            if codec::decode_string(codec::required(&mut native_set, "key")?)? != set.key
                || codec::decode_uint(codec::required(&mut native_set, "next_ordinal")?)?
                    != set.batch_count
            {
                return Err(invalid("captured set differs from closed descriptor"));
            }
            let after = if cursor.member == Some(position) {
                cursor.after
            } else {
                None
            };
            let page = object(native_page)?;
            Some(if request.dense {
                CaptureResultLocators::Dense(crate::canonical_results::decode_output_result_page(
                    page,
                    &attempt,
                    &interpretation,
                    set,
                    &request.output,
                    &request.partition,
                    request.start,
                    request.end,
                    after,
                )?)
            } else {
                CaptureResultLocators::Scalar(crate::canonical_results::decode_joined_result_page(
                    page,
                    &attempt,
                    &interpretation,
                    set,
                    &request.relation,
                    "0",
                    request.start,
                    request.end,
                    after,
                )?)
            })
        } else {
            if !matches!(native_set, Value::None | Value::Null)
                || !matches!(native_page, Value::None | Value::Null)
            {
                return Err(invalid("captured relation absent from closed manifest"));
            }
            None
        };
        let (finished, after) = match &locators {
            None => (true, None),
            Some(CaptureResultLocators::Scalar(page)) => (page.exhausted, page.next),
            Some(CaptureResultLocators::Dense(page)) => (page.exhausted, page.next),
        };
        partial = !finished;
        expected_next = CaptureResultCursor {
            member: Some(if finished {
                position
                    .checked_add(1)
                    .ok_or(CanonicalError::PayloadLimit)?
            } else {
                position
            }),
            after: if finished { None } else { after },
        };
        sources.push(CaptureResultSource {
            position,
            manifest,
            set,
            locators,
            read: None,
        });
    }
    if next != expected_next
        || exhausted && (partial || source_index != selected.len())
        || !exhausted && !partial && source_index == selected.len()
    {
        return Err(CanonicalError::IncompleteResponse);
    }
    if !exhausted
        && (next.member < cursor.member
            || next.member == cursor.member && next.after <= cursor.after)
    {
        return Err(CanonicalError::IncompleteResponse);
    }
    Ok(CaptureResultPage {
        sources,
        next,
        exhausted,
    })
}

impl StudyCapture {
    /// Fetch one bounded useful original locator page under this capture's sole protection.
    pub async fn result_page(
        &self,
        selected: &[u64],
        request: &CaptureResultRequest,
        cursor: CaptureResultCursor,
    ) -> Result<CaptureResultPage, CanonicalError> {
        if selected.len() > pse_schema::catalog::study_workbench::STUDY_CAPTURE_PAGE_ROWS
            || selected.windows(2).any(|pair| pair[0] >= pair[1])
            || request.start > request.end
            || [&request.relation, &request.output, &request.partition]
                .iter()
                .any(|value| {
                    value.is_empty()
                        || value.len() > pse_schema::catalog::substrate::METADATA_ID_BYTES
                })
        {
            return Err(CanonicalError::PayloadLimit);
        }
        let (store, key, owner, identity) = self.authority();
        store.files.require_current()?;
        let activity = store.admit_activity()?;
        activity.scope(crate::canonical::transport::metadata_scope(MetadataResponse::CaptureLocators,async {
            let selected_values=selected.iter().copied().map(codec::encode_uint).collect::<Result<Vec<_>,_>>()?;
            let mut response=protected_query("canonical_capture_results::result_page",||Ok(store.db.metadata_query(MetadataResponse::CaptureLocators,
                "RETURN fn::pse_workbench_v1::result_page($pse_rpc_expires_at,$capture,$owner,$request,$selected,$relation,$output,$partition,$dense,$start,$end,$member,$after);")
                .bind(("capture",key.to_owned())).bind(("owner",owner.to_owned())).bind(("request",Bytes::from(identity.to_vec())))
                .bind(("selected",selected_values.clone())).bind(("relation",request.relation.clone())).bind(("output",request.output.clone()))
                .bind(("partition",request.partition.clone())).bind(("dense",request.dense)).bind(("start",codec::encode_uint(request.start)?)).bind(("end",codec::encode_uint(request.end)?))
                .bind(("member",cursor.member.map(codec::encode_uint).transpose()?.unwrap_or(Value::None))).bind(("after",cursor.after.map(codec::encode_uint).transpose()?.unwrap_or(Value::None))))).await?;
            let mut page=decode_capture_page(response.take::<Value>(0)?,selected,request,cursor)?;
            for source in &mut page.sources {
                if let Some(set)=&source.set {
                    source.read=Some(CapturedResultRead(std::sync::Arc::new(CapturedReadPermission{capture:self.clone(),position:source.position,attempt:source.manifest.attempt.clone(),set:set.clone(),metadata:std::sync::OnceLock::new()})));
                }
            }
            Ok(page)
        })).await
    }
}
/// Checked original closed-set permission. Clones share the existing capture;
/// constructing one never acquires a result ticket or an independent protection.
#[derive(Clone, Debug)]
pub struct CapturedResultRead(std::sync::Arc<CapturedReadPermission>);
#[derive(Debug)]
struct CapturedReadPermission {
    capture: StudyCapture,
    position: u64,
    attempt: String,
    set: ResultSetDescriptor,
    metadata: std::sync::OnceLock<std::sync::Arc<dyn std::fmt::Debug + Send + Sync>>,
}
impl CapturedResultRead {
    /// Retain only this compact permission allocation through physical borrowers.
    /// The grant cannot be replaced or accumulate temporary locator-page buffers.
    pub fn retain_metadata_owner(
        &self,
        owner: std::sync::Arc<dyn std::fmt::Debug + Send + Sync>,
    ) -> Result<(), CanonicalError> {
        if let Some(old) = self.0.metadata.get() {
            return if std::sync::Arc::ptr_eq(old, &owner) {
                Ok(())
            } else {
                Err(invalid("captured permission allocation owner differs"))
            };
        }
        if self.0.metadata.set(owner.clone()).is_err()
            && self
                .0
                .metadata
                .get()
                .is_none_or(|old| !std::sync::Arc::ptr_eq(old, &owner))
        {
            return Err(invalid("captured permission allocation owner differs"));
        }
        Ok(())
    }
    /// Compact permission metadata, excluding its already charged shared capture.
    pub fn retained_bytes(&self) -> Result<usize, CanonicalError> {
        [
            size_of::<CapturedReadPermission>(),
            2 * size_of::<usize>(),
            self.0.attempt.capacity(),
            self.0.set.key.capacity(),
            self.0.set.name.capacity(),
            self.0.set.batches_digest.capacity(),
        ]
        .into_iter()
        .try_fold(0usize, usize::checked_add)
        .ok_or(CanonicalError::PayloadLimit)
    }
    fn verify(&self, batch: &Batch) -> Result<(), CanonicalError> {
        crate::canonical_execution::validate_result_batch(batch)?;
        if batch.attempt != self.0.attempt
            || batch.result_set != self.0.set.key
            || batch.ordinal >= self.0.set.batch_count
        {
            return Err(invalid(
                "captured payload outside exact admitted manifest descriptor",
            ));
        }
        Ok(())
    }
    /// Authenticate one exact captured file descriptor before the existing physical verifier.
    pub async fn open_result_file(
        &self,
        batch: &Batch,
    ) -> Result<crate::canonical_files::ProtectedFile, CanonicalError> {
        self.verify(batch)?;
        let (store, key, owner, request) = self.0.capture.authority();
        store.files.require_current()?;
        let activity = store.admit_activity()?;
        activity.scope(crate::canonical::transport::metadata_scope(MetadataResponse::Header,async {
            let expected=wire::encode_canonical_result_batches(batch)?;
            let mut response=protected_query("canonical_capture_results::file_descriptor",||Ok(store.db.metadata_query(MetadataResponse::Header,"RETURN fn::pse_workbench_v1::result_descriptor($pse_rpc_expires_at,$capture,$owner,$request,$position,$expected);")
                .bind(("capture",key.to_owned())).bind(("owner",owner.to_owned())).bind(("request",Bytes::from(request.to_vec()))).bind(("position",codec::encode_uint(self.0.position)?)).bind(("expected",expected.clone())))).await?;
            if wire::decode_canonical_result_batches(object(response.take::<Value>(0)?)?)?!=*batch {return Err(invalid("captured file descriptor differs"));}
            let descriptor=store.file_descriptor(batch).await?;
            store.open_physical_file(descriptor,std::sync::Arc::new(self.clone()))
        })).await
    }
    /// Hydrate one exact captured inline body through the existing encoding verifier.
    pub async fn read_inline_payload(
        &self,
        batch: &Batch,
    ) -> Result<pse_model::generated::runtime::canonical_inline_payloads::Row, CanonicalError> {
        self.verify(batch)?;
        let crate::canonical_execution::PayloadLocator::Inline { key: inline, .. } =
            crate::canonical_execution::result_payload_locator(batch)?
        else {
            return Err(invalid("captured inline locator required"));
        };
        let (store, key, owner, request) = self.0.capture.authority();
        store.files.require_current()?;
        let activity = store.admit_activity()?;
        activity.scope(crate::canonical::transport::metadata_scope(MetadataResponse::InlinePayload,async {
            let expected=wire::encode_canonical_result_batches(batch)?;
            let mut response=protected_query("canonical_capture_results::inline_payload",||Ok(store.db.metadata_query(MetadataResponse::InlinePayload,"RETURN fn::pse_workbench_v1::inline_payload($pse_rpc_expires_at,$capture,$owner,$request,$position,$expected,$inline);")
                .bind(("capture",key.to_owned())).bind(("owner",owner.to_owned())).bind(("request",Bytes::from(request.to_vec()))).bind(("position",codec::encode_uint(self.0.position)?)).bind(("expected",expected.clone())).bind(("inline",inline.clone())))).await?;
            crate::canonical_results::validate_inline_body(batch,wire::decode_canonical_inline_payloads(object(response.take::<Value>(0)?)?)?)
        })).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source(position: u64) -> Value {
        let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalPayloadV1);
        hash.str("pse.execution.result-manifest.v1");
        hash.part(b"[]");
        let manifest = Manifest {
            key: "attempt".into(),
            attempt: "attempt".into(),
            generation: 7,
            digest: hash.finish_hash().to_hex(),
            descriptors: b"[]".to_vec().into(),
        };
        let mut row = Object::default();
        row.insert("position", codec::encode_uint(position).unwrap());
        row.insert("attempt", Value::String("attempt".into()));
        row.insert("generation", codec::encode_uint(7).unwrap());
        row.insert("interpretation", Value::String(wire::INTERPRETATION.into()));
        row.insert(
            "manifest",
            Value::Object(wire::encode_canonical_result_manifests(&manifest).unwrap()),
        );
        row.insert("set", Value::None);
        row.insert("page", Value::None);
        Value::Object(row)
    }
    fn page(positions: &[u64], next: u64, exhausted: bool) -> Value {
        let mut row = Object::default();
        row.insert(
            "items",
            Value::Array(positions.iter().copied().map(source).collect()),
        );
        row.insert("member", codec::encode_uint(next).unwrap());
        row.insert("after", Value::None);
        row.insert("exhausted", Value::Bool(exhausted));
        Value::Object(row)
    }
    fn request() -> CaptureResultRequest {
        CaptureResultRequest {
            relation: "relation".into(),
            output: "relation".into(),
            partition: "0".into(),
            dense: false,
            start: 0,
            end: 1,
        }
    }
    #[test]
    fn capture_locator_pages_require_exact_selected_prefix_and_continuation() {
        let request = request();
        let cursor = CaptureResultCursor::default();
        assert_eq!(
            decode_capture_page(page(&[0, 3], 4, true), &[0, 3], &request, cursor)
                .unwrap()
                .sources
                .len(),
            2
        );
        assert!(
            decode_capture_page(page(&[3], 4, true), &[0, 3], &request, cursor).is_err(),
            "omitted source is not an empty result"
        );
        assert!(
            decode_capture_page(page(&[0, 3], 3, true), &[0, 3], &request, cursor).is_err(),
            "cursor cannot replay a completed source"
        );
        assert!(
            decode_capture_page(page(&[0], 1, true), &[0, 3], &request, cursor).is_err(),
            "a short prefix cannot claim completeness"
        );
        assert!(decode_capture_page(page(&[], 0, false), &[0, 3], &request, cursor).is_err());
        let first = decode_capture_page(page(&[0], 1, false), &[0, 3], &request, cursor).unwrap();
        assert_eq!(first.next.member, Some(1));
        assert!(
            decode_capture_page(page(&[3], 4, true), &[0, 3], &request, first.next)
                .unwrap()
                .exhausted
        );
    }
}
