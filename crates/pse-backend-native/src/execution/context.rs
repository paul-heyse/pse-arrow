// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit observations used by contextual admission. Only composition and final
//! preflight discover process state; policy assessment consumes an immutable snapshot.
use super::{BackendExecution, Table};
use crate::{ProblemError, settings::ipopt::Runtime, solve::Backend};
use pse_ids::{ContentHash, FramedHasher};
use std::collections::BTreeMap;

/// The build/linkage fact for one registered adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildObservation {
    /// This binary links the native implementation.
    pub linked: bool,
    /// The adapter's additional native build contract, when one exists.
    pub identity: Option<ContentHash>,
}
/// One immutable build/runtime observation supplied by the composition boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    /// Registered adapter observations, including absent native implementations.
    pub adapters: BTreeMap<Backend, BuildObservation>,
    /// Actual process/thread state consumed by Ipopt linear-solver admission.
    pub ipopt: Option<Runtime>,
    /// The declared OpenMP cancellation setting consumed by SCIP's nested Ipopt.
    pub scip_omp_cancellation: bool,
}
impl Snapshot {
    /// Observe once at composition, never from a pure candidate assessment.
    pub fn observe(table: &Table) -> Self {
        let adapters = table
            .adapters()
            .map(|adapter| {
                (
                    adapter.backend(),
                    BuildObservation {
                        linked: adapter.linked(),
                        identity: adapter.build(),
                    },
                )
            })
            .collect();
        #[cfg(feature = "ipopt")]
        let ipopt = Some(Runtime::observe());
        #[cfg(not(feature = "ipopt"))]
        let ipopt = None;
        Self {
            adapters,
            ipopt,
            scip_omp_cancellation: std::env::var("OMP_CANCELLATION")
                .is_ok_and(|value| value.eq_ignore_ascii_case("true")),
        }
    }
    /// Whether this observation links the requested adapter.
    pub fn linked(&self, backend: Backend) -> bool {
        self.adapters
            .get(&backend)
            .is_some_and(|build| build.linked)
    }
    /// Identity of the exact observations consumed by assessment.
    pub fn identity(&self) -> ContentHash {
        let mut h = FramedHasher::new(pse_ids::Frame::NativeContextSnapshotV1);
        for (backend, build) in &self.adapters {
            h.str(backend.as_str()).bool(build.linked);
            match build.identity {
                Some(identity) => {
                    h.hash(&identity);
                }
                None => {
                    h.u64(0);
                }
            }
        }
        match self.ipopt {
            Some(runtime) => {
                h.bool(true)
                    .u64(u64::from(runtime.linked))
                    .bool(runtime.cancellation)
                    .part(&runtime.proc_bind.to_le_bytes())
                    .part(&runtime.cbwr.to_le_bytes())
                    .bool(runtime.mkl_dynamic);
            }
            None => {
                h.bool(false);
            }
        }
        h.bool(self.scip_omp_cancellation);
        h.finish_hash()
    }
    /// Revalidate the selected observation immediately before entering its native scope.
    /// A changed observation refuses this request; it never requests another backend.
    pub fn validate_for<A: BackendExecution + ?Sized>(
        &self,
        adapter: &A,
    ) -> Result<(), ProblemError> {
        let current = BuildObservation {
            linked: adapter.linked(),
            identity: adapter.build(),
        };
        if self.adapters.get(&adapter.backend()) != Some(&current) || !current.linked {
            return Err(ProblemError::Unsupported(
                "selected native build observation changed".into(),
            ));
        }
        Ok(())
    }
}
