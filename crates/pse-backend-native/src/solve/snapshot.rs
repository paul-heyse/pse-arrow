// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Portable finite seed reconstruction; no native solver continuation is reconstructed.
use super::*;
use pse_model::diagnostic::Observation;

fn number(observation: &Observation) -> Result<f64, ProblemError> {
    match observation {
        Observation::Real(value) if value.is_finite() => Ok(*value),
        Observation::Nonfinite(_) => Err(ProblemError::Contract(
            "nonfinite observation cannot restore a native seed".into(),
        )),
        _ => Err(ProblemError::Contract(
            "native seed requires a finite real observation".into(),
        )),
    }
}
fn vector(observations: &[Observation]) -> Result<Vec<f64>, ProblemError> {
    observations.iter().map(number).collect()
}
fn inconsistent() -> ProblemError {
    ProblemError::Contract("inconsistent portable native seed dimensions".into())
}
fn agree(dimension: &mut Option<usize>, next: usize) -> Result<(), ProblemError> {
    if dimension.is_some_and(|held| held != next) {
        return Err(inconsistent());
    }
    *dimension = Some(next);
    Ok(())
}
struct RestoreExtent(usize);
impl RestoreExtent {
    fn vector<T>(&mut self, length: usize) -> Result<(), ProblemError> {
        self.0 = length
            .checked_mul(size_of::<T>())
            .and_then(|bytes| self.0.checked_add(bytes))
            .ok_or_else(|| ProblemError::memory("portable native seed extent overflow"))?;
        Ok(())
    }
    fn numbers(&mut self, observations: &[Observation]) -> Result<(), ProblemError> {
        for observation in observations {
            number(observation)?;
        }
        self.vector::<f64>(observations.len())
    }
    fn working(&mut self, working: &WorkingSetSnapshot) -> Result<(), ProblemError> {
        let active = working.active.as_ref().ok_or_else(|| {
            ProblemError::Unsupported(
                "portable working-set snapshot has unavailable activity".into(),
            )
        })?;
        #[cfg(feature = "pounce")]
        {
            self.vector::<pounce_rs::sqp::BoundStatus>(active.bounds.len())?;
            self.vector::<pounce_rs::sqp::ConsStatus>(active.constraints.len())?;
            Ok(())
        }
        #[cfg(not(feature = "pounce"))]
        {
            let _ = active;
            Err(ProblemError::Unsupported(
                "portable working-set restoration requires POUNCE support".into(),
            ))
        }
    }
}
impl WarmStartSnapshot {
    /// Validate portable values and bound their reconstructed native-owned allocation
    /// without allocating. The caller reserves this extent before restore(). Full
    /// target layout/shape compatibility is still checked by the consuming adapter.
    pub fn restore_extent(&self) -> Result<usize, ProblemError> {
        let mut extent = RestoreExtent(size_of::<WarmStart>());
        let supported = match &self.payload {
            WarmPayloadSnapshot::Root { primal } => {
                extent.numbers(primal)?;
                matches!(self.backend, Backend::Kinsol | Backend::Petsc)
            }
            WarmPayloadSnapshot::Nlp {
                primal,
                bound_duals,
                row_duals,
                barrier,
                working_set,
            } => {
                extent.numbers(primal)?;
                if let Some((lower, upper)) = bound_duals {
                    if lower.len() != primal.len() || upper.len() != primal.len() {
                        return Err(inconsistent());
                    }
                    extent.numbers(lower)?;
                    extent.numbers(upper)?;
                }
                if let Some(rows) = row_duals {
                    extent.numbers(rows)?;
                }
                if let Some(barrier) = barrier
                    && number(barrier)? <= 0.
                {
                    return Err(ProblemError::Contract(
                        "portable native barrier must be positive".into(),
                    ));
                }
                if let Some(working) = working_set {
                    if self.backend != Backend::Pounce {
                        return Err(ProblemError::Contract(
                            "portable working set requires its POUNCE backend".into(),
                        ));
                    }
                    extent.working(working)?;
                }
                matches!(
                    self.backend,
                    Backend::Ipopt | Backend::Pounce | Backend::Uno
                )
            }
            WarmPayloadSnapshot::Highs {
                primal,
                dual,
                basis,
            } => {
                if primal.is_none() && dual.is_none() && basis.is_none() {
                    return Err(ProblemError::Contract("empty portable HiGHS seed".into()));
                }
                let (mut columns, mut rows) = (None, None);
                if let Some(primal) = primal {
                    extent.numbers(primal)?;
                    agree(&mut columns, primal.len())?;
                }
                if let Some((column_dual, row_dual)) = dual {
                    extent.numbers(column_dual)?;
                    extent.numbers(row_dual)?;
                    agree(&mut columns, column_dual.len())?;
                    agree(&mut rows, row_dual.len())?;
                }
                if let Some(basis) = basis {
                    if !basis
                        .columns
                        .iter()
                        .chain(&basis.rows)
                        .all(|code| (0..=4).contains(code))
                    {
                        return Err(ProblemError::Contract(
                            "invalid portable HiGHS basis code".into(),
                        ));
                    }
                    agree(&mut columns, basis.columns.len())?;
                    agree(&mut rows, basis.rows.len())?;
                    extent.vector::<i32>(basis.columns.capacity())?;
                    extent.vector::<i32>(basis.rows.capacity())?;
                }
                self.backend == Backend::Highs
            }
        };
        if !supported {
            return Err(ProblemError::Unsupported(
                "portable seed payload does not match its native backend".into(),
            ));
        }
        Ok(extent.0)
    }
    /// Reconstruct the actual portable seed under a caller-owned precharged extent.
    /// Finite IEEE values, signed zero and layout/profile/data/origin are preserved.
    /// Nonfinite export observations and unavailable working state refuse explicitly;
    /// a reconstructed point/basis is not resumable native history.
    pub fn restore(&self) -> Result<WarmStart, ProblemError> {
        self.restore_extent()?;
        let payload = match &self.payload {
            WarmPayloadSnapshot::Root { primal } => WarmPayload::Root(vector(primal)?),
            WarmPayloadSnapshot::Nlp {
                primal,
                bound_duals,
                row_duals,
                barrier,
                working_set,
            } => WarmPayload::Nlp {
                primal: vector(primal)?,
                bounds: bound_duals
                    .as_ref()
                    .map(|(lower, upper)| -> Result<_, ProblemError> {
                        Ok((vector(lower)?, vector(upper)?))
                    })
                    .transpose()?,
                rows: row_duals.as_ref().map(|rows| vector(rows)).transpose()?,
                barrier: barrier.as_ref().map(number).transpose()?,
                working: working_set.as_ref().map(restore_working).transpose()?,
            },
            WarmPayloadSnapshot::Highs {
                primal,
                dual,
                basis,
            } => WarmPayload::Highs {
                primal: primal.as_ref().map(|primal| vector(primal)).transpose()?,
                dual: dual
                    .as_ref()
                    .map(|(columns, rows)| -> Result<_, ProblemError> {
                        Ok((vector(columns)?, vector(rows)?))
                    })
                    .transpose()?,
                basis: basis.clone(),
            },
        };
        Ok(WarmStart {
            origin: self.origin.clone(),
            compatibility: Compatibility {
                layout: self.layout,
                profile: self.profile,
                data: self.data,
                backend: self.backend,
            },
            payload,
        })
    }
}
fn restore_working(working: &WorkingSetSnapshot) -> Result<WorkingSet, ProblemError> {
    #[cfg(feature = "pounce")]
    {
        use pounce_rs::sqp::{BoundStatus as B, ConsStatus as C};
        let active = working.active.as_ref().ok_or_else(|| {
            ProblemError::Unsupported(
                "portable working-set snapshot has unavailable activity".into(),
            )
        })?;
        Ok(WorkingSet {
            transformation: working.transformation,
            active: pounce_rs::sqp::WorkingSet {
                bounds: active
                    .bounds
                    .iter()
                    .map(|status| match status {
                        BoundActivity::Inactive => B::Inactive,
                        BoundActivity::AtLower => B::AtLower,
                        BoundActivity::AtUpper => B::AtUpper,
                        BoundActivity::Fixed => B::Fixed,
                    })
                    .collect(),
                constraints: active
                    .constraints
                    .iter()
                    .map(|status| match status {
                        RowActivity::Inactive => C::Inactive,
                        RowActivity::AtLower => C::AtLower,
                        RowActivity::AtUpper => C::AtUpper,
                        RowActivity::Equality => C::Equality,
                    })
                    .collect(),
            },
        })
    }
    #[cfg(not(feature = "pounce"))]
    {
        let _ = working;
        Err(ProblemError::Unsupported(
            "portable working-set restoration requires POUNCE support".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn snapshot(backend: Backend, payload: WarmPayloadSnapshot) -> WarmStartSnapshot {
        WarmStartSnapshot {
            origin: None,
            layout: ContentHash::from_bytes([1; 32]),
            profile: ContentHash::from_bytes([2; 32]),
            data: ContentHash::from_bytes([3; 32]),
            backend,
            payload,
        }
    }
    #[test]
    fn portable_seed_restore_preserves_finite_bits_metadata_and_complete_nlp_state() {
        let seed = WarmStart {
            origin: None,
            compatibility: crate::solver_tests::stamp(Backend::Ipopt),
            payload: WarmPayload::Nlp {
                primal: vec![-0., 0.125],
                bounds: Some((vec![0., 1.], vec![2., 3.])),
                rows: Some(vec![4., -5.]),
                barrier: Some(0.25),
                working: None,
            },
        };
        let encoded = serde_json::to_vec(&seed.snapshot()).unwrap();
        let portable: WarmStartSnapshot = serde_json::from_slice(&encoded).unwrap();
        assert!(
            portable.restore_extent().unwrap() >= size_of::<WarmStart>() + 8 * size_of::<f64>()
        );
        let restored = portable.restore().unwrap();
        assert_eq!(restored.compatibility, seed.compatibility);
        assert_eq!(restored.content_key(), seed.content_key());
        let WarmPayload::Nlp {
            primal, barrier, ..
        } = restored.payload
        else {
            panic!("NLP seed");
        };
        assert_eq!(primal[0].to_bits(), (-0_f64).to_bits());
        assert_eq!(barrier, Some(0.25));
    }
    #[test]
    fn portable_seed_restore_supports_roots_and_basis_only_highs_and_refuses_bad_shapes() {
        let root = snapshot(
            Backend::Kinsol,
            WarmPayloadSnapshot::Root {
                primal: vec![Observation::number(-0.)],
            },
        );
        let WarmPayload::Root(values) = root.restore().unwrap().payload else {
            panic!("root seed");
        };
        assert_eq!(values[0].to_bits(), (-0_f64).to_bits());
        let mut highs = snapshot(
            Backend::Highs,
            WarmPayloadSnapshot::Highs {
                primal: None,
                dual: None,
                basis: Some(Basis {
                    columns: vec![0, 1, 2],
                    rows: vec![3, 4],
                }),
            },
        );
        assert!(highs.restore_extent().unwrap() >= size_of::<WarmStart>() + 5 * size_of::<i32>());
        let WarmPayload::Highs {
            basis: Some(basis), ..
        } = highs.restore().unwrap().payload
        else {
            panic!("basis seed");
        };
        assert_eq!(basis.columns, [0, 1, 2]);
        if let WarmPayloadSnapshot::Highs { primal, .. } = &mut highs.payload {
            *primal = Some(vec![Observation::number(1.)]);
        }
        assert!(highs.restore_extent().is_err());
        let wrong = snapshot(Backend::Highs, root.payload);
        assert!(wrong.restore().is_err());
    }
    #[test]
    fn portable_seed_restore_refuses_nonfinite_or_nonnumeric_values_and_unknown_working_state() {
        for value in [
            Observation::number(f64::NAN),
            Observation::number(f64::INFINITY),
            Observation::number(f64::NEG_INFINITY),
            Observation::Missing,
            Observation::Integer(1),
        ] {
            let portable = snapshot(
                Backend::Kinsol,
                WarmPayloadSnapshot::Root {
                    primal: vec![value],
                },
            );
            assert!(matches!(
                portable.restore_extent(),
                Err(ProblemError::Contract(_))
            ));
            assert!(portable.restore().is_err());
        }
        let unknown = snapshot(
            Backend::Pounce,
            WarmPayloadSnapshot::Nlp {
                primal: vec![Observation::number(1.)],
                bound_duals: None,
                row_duals: None,
                barrier: None,
                working_set: Some(WorkingSetSnapshot {
                    transformation: ContentHash::from_bytes([8; 32]),
                    active: None,
                }),
            },
        );
        assert!(matches!(
            unknown.restore(),
            Err(ProblemError::Unsupported(_))
        ));
    }
    #[cfg(feature = "pounce")]
    #[test]
    fn portable_seed_restore_rebuilds_actual_pounce_working_set_without_continuation_claim() {
        let portable = snapshot(
            Backend::Pounce,
            WarmPayloadSnapshot::Nlp {
                primal: vec![Observation::number(1.)],
                bound_duals: None,
                row_duals: None,
                barrier: None,
                working_set: Some(WorkingSetSnapshot {
                    transformation: ContentHash::from_bytes([8; 32]),
                    active: Some(ActiveSetSnapshot {
                        bounds: vec![
                            BoundActivity::Inactive,
                            BoundActivity::AtLower,
                            BoundActivity::AtUpper,
                            BoundActivity::Fixed,
                        ],
                        constraints: vec![
                            RowActivity::Inactive,
                            RowActivity::AtLower,
                            RowActivity::AtUpper,
                            RowActivity::Equality,
                        ],
                    }),
                }),
            },
        );
        let restored = portable.restore().unwrap();
        assert_eq!(
            serde_json::to_value(restored.snapshot()).unwrap(),
            serde_json::to_value(portable).unwrap()
        );
    }
}
