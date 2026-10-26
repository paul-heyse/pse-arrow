// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Python-owned workbench and passive observer projections; policy stays in native owners.
use super::*;
impl NativeRuntime {
    pub(super) fn study_workbench_page_native(
        &self,
        py: Python<'_>,
        request: &[u8],
    ) -> PyResult<documents::DocumentValue<native::StudyWorkbenchPage>> {
        let request = documents::decode_document(py, "study workbench", request, 1024 * 1024)?;
        let stop = CancelSource::new();
        blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime.study_workbench_page(request).await
            }),
            || stop.cancel(),
        )
        .map(documents::DocumentValue)
    }
    pub(super) fn freeze_study_cohort_native(
        &self,
        py: Python<'_>,
        request: &[u8],
    ) -> PyResult<NativeFrozenStudyCohort> {
        let request = documents::decode_document(py, "study cohort", request, 1024 * 1024)?;
        let stop = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime.freeze_study_cohort(request).await
            }),
            || stop.cancel(),
        )?;
        Ok(NativeFrozenStudyCohort {
            owner: self.owner.clone(),
            inner: Arc::new(inner),
        })
    }
    pub(super) fn observe_study_native(
        &self,
        py: Python<'_>,
        request: &[u8],
        cursor: Option<&[u8]>,
    ) -> PyResult<NativeStudyObserver> {
        let request = documents::decode_document(py, "study observation", request, 1024 * 1024)?;
        let cursor = cursor
            .map(|bytes| {
                documents::decode_document(py, "study observation cursor", bytes, 1024 * 1024)
            })
            .transpose()?;
        let stop = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |runtime| async move {
                runtime.observe_study(request, cursor).await
            }),
            || stop.cancel(),
        )?;
        let cancel = inner.observation_token();
        Ok(NativeStudyObserver {
            owner: self.owner.clone(),
            slot: Mutex::new(ObserverSlot::Idle(Box::new(inner))),
            cancel,
        })
    }
}
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeFrozenStudyCohort {
    owner: Arc<runtime::Runtime>,
    inner: Arc<native::FrozenStudyCohort>,
}
#[pymethods]
impl NativeFrozenStudyCohort {
    #[getter]
    fn receipt(&self) -> documents::DocumentValue<native::StudyCohortReceipt> {
        documents::DocumentValue(self.inner.receipt().clone())
    }
    fn header_page(
        &self,
        py: Python<'_>,
        request: &[u8],
    ) -> PyResult<documents::DocumentValue<native::StudyCohortHeaderPage>> {
        let request =
            documents::decode_document(py, "study cohort header page", request, 1024 * 1024)?;
        let cancel = pse_columnar::CancellationToken::new();
        blocking(
            py,
            &self.owner,
            async {
                cancel
                    .until_cancelled(self.inner.header_page(request))
                    .await
                    .map_err(pse_engine::EngineError::from)?
            },
            || cancel.cancel(),
        )
        .map(documents::DocumentValue)
    }
    fn member_page(
        &self,
        py: Python<'_>,
        request: &[u8],
    ) -> PyResult<documents::DocumentValue<native::StudyCohortMemberPage>> {
        let request =
            documents::decode_document(py, "study cohort member page", request, 1024 * 1024)?;
        let cancel = pse_columnar::CancellationToken::new();
        blocking(
            py,
            &self.owner,
            async {
                cancel
                    .until_cancelled(self.inner.member_page(request))
                    .await
                    .map_err(pse_engine::EngineError::from)?
            },
            || cancel.cancel(),
        )
        .map(documents::DocumentValue)
    }
    #[pyo3(signature=(member,relation,*,start,end))]
    fn result(
        &self,
        py: Python<'_>,
        member: usize,
        relation: &str,
        start: u64,
        end: u64,
    ) -> PyResult<inspection::TableStream> {
        let cancel = pse_columnar::CancellationToken::new();
        let reader = blocking(
            py,
            &self.owner,
            self.inner
                .result(member, relation, start, end, cancel.clone()),
            || cancel.cancel(),
        )?;
        Ok(inspection::TableStream::from_canonical(
            reader,
            self.owner.clone(),
        ))
    }
    fn aggregate(&self, py: Python<'_>, request: &[u8]) -> PyResult<NativeStudyKpiAnalysis> {
        let request = documents::decode_document(py, "study KPI", request, 1024 * 1024)?;
        let cancel = pse_columnar::CancellationToken::new();
        let inner = blocking(
            py,
            &self.owner,
            self.inner.aggregate(&request, cancel.clone()),
            || cancel.cancel(),
        )?;
        Ok(NativeStudyKpiAnalysis {
            inner,
            owner: self.owner.clone(),
        })
    }
}
#[pyclass(frozen, module = "pse._native")]
#[derive(Debug)]
pub(crate) struct NativeStudyKpiAnalysis {
    owner: Arc<runtime::Runtime>,
    inner: native::StudyKpiAnalysis,
}
#[pymethods]
impl NativeStudyKpiAnalysis {
    #[getter]
    fn cohort(&self) -> documents::DocumentValue<native::StudyCohortReceipt> {
        documents::DocumentValue(self.inner.cohort.clone())
    }
    #[getter]
    fn included(&self) -> u64 {
        self.inner.included
    }
    #[getter]
    fn unavailable(&self) -> u64 {
        self.inner.unavailable
    }
    #[getter]
    fn progress(&self) -> documents::DocumentValue<native::StudyCohortScanProgress> {
        documents::DocumentValue(self.inner.progress())
    }
    fn classification_page(
        &self,
        py: Python<'_>,
        request: &[u8],
    ) -> PyResult<documents::DocumentValue<native::StudyCohortClassificationPage>> {
        let request = documents::decode_document(
            py,
            "study cohort classification page",
            request,
            1024 * 1024,
        )?;
        let cancel = pse_columnar::CancellationToken::new();
        blocking(
            py,
            &self.owner,
            async {
                cancel
                    .until_cancelled(self.inner.classification_page(request))
                    .await
                    .map_err(pse_engine::EngineError::from)?
            },
            || cancel.cancel(),
        )
        .map(documents::DocumentValue)
    }
    fn batches(&self) -> Vec<physical_input::PhysicalOutput> {
        self.inner
            .batches
            .iter()
            .cloned()
            .map(pyo3_arrow::PyRecordBatch::new)
            .map(physical_input::PhysicalOutput)
            .collect()
    }
}
#[derive(Debug)]
enum ObserverSlot {
    Idle(Box<native::StudyObserver>),
    Polling,
    Closed,
}
#[pyclass(frozen, module = "pse._native")]
#[derive(Debug)]
pub(crate) struct NativeStudyObserver {
    owner: Arc<runtime::Runtime>,
    slot: Mutex<ObserverSlot>,
    cancel: pse_columnar::CancellationToken,
}
impl NativeStudyObserver {
    fn slot(&self, py: Python<'_>) -> PyResult<std::sync::MutexGuard<'_, ObserverSlot>> {
        self.slot
            .lock()
            .map_err(|_| invalid(py, "study observer lock poisoned"))
    }
}
#[pymethods]
impl NativeStudyObserver {
    fn cursor(
        &self,
        py: Python<'_>,
    ) -> PyResult<documents::DocumentValue<native::StudyObservationCursor>> {
        match &*self.slot(py)? {
            ObserverSlot::Idle(observer) => Ok(documents::DocumentValue(observer.cursor())),
            _ => Err(invalid(py, "study observer busy or closed")),
        }
    }
    fn attach_run(&self, py: Python<'_>, run: &NativeRunHandle) -> PyResult<()> {
        match &mut *self.slot(py)? {
            ObserverSlot::Idle(observer) => observer
                .attach_run(&run.inner)
                .map_err(|e| errors::diagnostic(py, &e)),
            _ => Err(invalid(py, "study observer busy or closed")),
        }
    }
    fn poll(
        &self,
        py: Python<'_>,
    ) -> PyResult<documents::DocumentValue<native::StudyObservationPage>> {
        let mut observer = {
            let mut slot = self.slot(py)?;
            match std::mem::replace(&mut *slot, ObserverSlot::Polling) {
                ObserverSlot::Idle(observer) => observer,
                other => {
                    *slot = other;
                    return Err(invalid(py, "study observer busy or closed"));
                }
            }
        };
        let page = blocking(py, &self.owner, observer.poll(), || self.cancel.cancel());
        let mut slot = self.slot(py)?;
        if matches!(*slot, ObserverSlot::Closed) || page.is_err() {
            *slot = ObserverSlot::Closed;
            drop(slot);
            let _enter = self.owner.executor.enter();
            observer.close();
        } else {
            *slot = ObserverSlot::Idle(observer);
        }
        page.map(documents::DocumentValue)
    }
    fn close(&self, py: Python<'_>) -> PyResult<()> {
        self.cancel.cancel();
        let old = std::mem::replace(&mut *self.slot(py)?, ObserverSlot::Closed);
        let _enter = self.owner.executor.enter();
        drop(old);
        Ok(())
    }
}
impl Drop for NativeStudyObserver {
    fn drop(&mut self) {
        self.cancel.cancel();
        let _enter = self.owner.executor.enter();
        if let Ok(slot) = self.slot.get_mut() {
            *slot = ObserverSlot::Closed;
        }
    }
}
