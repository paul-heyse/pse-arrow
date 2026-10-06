// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! POUNCE's bounded natural-parameter walk, with explicit original-coordinate events.
//! All endpoints remain predictions. The pinned routine does not relinearize an NLP
//! and extrapolates its remainder when the segment cap binds; that endpoint must pass
//! the common semantic screening and a native original-model corrector.
/// Scalar arclength families corrected by admitted native libraries.
pub mod arclength;
/// Scoped convex QP library homotopy and correction.
pub mod qp;
use super::{
    Advance, Prediction, Side,
    activity::{Factor, Limits, Work},
};
use crate::{ProblemError, solve::Execution};
use pounce_sens_core::{
    SensBacksolver,
    boundcheck::{BoundMultiplier, step_along_path},
    rowlimit::{RowLimitView, WatchedRow},
};
use pse_ids::SemanticId;
use std::sync::Arc;

/// Original quantity behind a library primal coordinate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coordinate {
    /// An original solve variable.
    Variable(SemanticId),
    /// One finite limit on an original inequality row.
    Row {
        /// Original semantic row identity.
        id: SemanticId,
        /// The watched original limit.
        side: Side,
    },
}
/// One actually tracked activation/release breakpoint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Event {
    /// Fraction of the requested parameter change at the breakpoint.
    pub at: f64,
    /// Original row or variable whose limit changes.
    pub coordinate: Coordinate,
    /// Lower or upper original limit.
    pub side: Side,
    /// True for activation, false for release.
    pub activated: bool,
}
/// Whether the library tracked the whole perturbation or extrapolated a remainder.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Coverage {
    /// No untracked remainder was reported by the segment-cap contract.
    Tracked,
    /// Segment allowance bound; the final endpoint includes an untracked remainder.
    Partial {
        /// Last actually tracked breakpoint fraction, never the extrapolated endpoint.
        through: f64,
    },
}
/// A start-only proposal and its actual library path evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct PathPrediction {
    /// Original-variable endpoint and canonical row multipliers.
    pub prediction: Prediction,
    /// Actual named breakpoints, including inequality-row releases.
    pub events: Vec<Event>,
    /// Actual segment tracking coverage.
    pub coverage: Coverage,
    /// Actual release/refactor/backsolve work.
    pub work: Work,
}
/// Complete physical-coordinate inputs supplied by the retained source owner.
#[derive(Clone, Debug)]
pub(super) struct Point {
    pub factor: super::KktFactor,
    pub original_variables: usize,
    pub variables: Vec<SemanticId>,
    pub rows: Vec<SemanticId>,
    pub parameters: Vec<(SemanticId, f64)>,
    pub pins: Vec<i32>,
    pub primal: Vec<f64>,
    pub row_dual: Vec<f64>,
    pub lower: Vec<f64>,
    pub upper: Vec<f64>,
    pub values: Vec<f64>,
    pub row_bounds: Vec<(f64, f64)>,
    pub bounds: Vec<(f64, f64)>,
    pub jacobian: Vec<(usize, usize, f64)>,
}
/// Track a finite natural-parameter change through the retained active-set operator.
///
/// # Errors
/// Invalid coordinates or allowance, a typed original execution stop, or a failed
/// library walk. Library diagnostic text is used only as evidence, never classification.
pub fn predict(
    advance: &Advance,
    parameters: &[(SemanticId, f64)],
    segments: usize,
    limits: Limits,
    execution: Execution,
) -> Result<PathPrediction, Arc<ProblemError>> {
    track(
        advance.activity_point(),
        parameters,
        segments,
        limits,
        execution,
    )
}
fn track(
    point: Point,
    parameters: &[(SemanticId, f64)],
    segments: usize,
    limits: Limits,
    execution: Execution,
) -> Result<PathPrediction, Arc<ProblemError>> {
    if segments == 0
        || parameters.len() != point.parameters.len()
        || parameters
            .iter()
            .zip(&point.parameters)
            .any(|((id, value), (source, _))| id != source || !value.is_finite())
    {
        return Err(Arc::new(ProblemError::Contract(
            "activity path parameter coordinates or segment allowance".into(),
        )));
    }
    let delta = parameters
        .iter()
        .zip(&point.parameters)
        .map(|((_, target), (_, source))| target - source)
        .collect::<Vec<_>>();
    if delta.iter().any(|v| !v.is_finite()) {
        return Err(Arc::new(ProblemError::Contract(
            "nonfinite activity parameter step".into(),
        )));
    }
    execution.check()?;
    let factor = Factor::new(
        point.factor.clone(),
        point.original_variables,
        execution.clone(),
        limits,
    )?;
    let n = point.factor.layout().variables;
    if n != point.original_variables + point.parameters.len()
        || point.bounds.len() != n
        || point.rows.len() != point.values.len()
        || point.row_bounds.len() != point.rows.len()
        || point.row_dual.len() != point.rows.len()
        || point.variables.len() != point.original_variables
    {
        return Err(Arc::new(ProblemError::Contract(
            "retained activity source coordinate extent".into(),
        )));
    }
    let mut rhs = vec![0.; factor.dim()];
    for (&pin, value) in point.pins.iter().zip(&delta) {
        let row = usize::try_from(pin)
            .map_err(|_| ProblemError::Contract("negative activity parameter pin".into()))?;
        let destination = rhs
            .get_mut(row)
            .ok_or_else(|| ProblemError::Contract("activity parameter pin outside KKT".into()))?;
        *destination = -value; // Parameter pins use -e in the original active-set layout.
    }
    let mut watched = Vec::new();
    let mut coordinates = Vec::new();
    for (row, &(lower, upper)) in point.row_bounds.iter().enumerate() {
        if lower == upper {
            continue;
        }
        for (limit, side, sign) in [(lower, Side::Lower, -1.), (upper, Side::Upper, 1.)] {
            if !limit.is_finite() {
                continue;
            }
            let coefficients = point
                .jacobian
                .iter()
                .filter_map(|&(r, c, v)| (r == row).then_some((c, sign * v)))
                .collect::<Vec<_>>();
            let active = point
                .factor
                .layout()
                .rows
                .iter()
                .enumerate()
                .find_map(|(k, (r, s))| {
                    (r.get() == row && *s == side).then_some((n + k, sign * point.row_dual[row]))
                });
            // The coordinate is the local linearization's row VALUE. Its affine
            // offset belongs in the base value, not in the derivative coefficients.
            watched.push(WatchedRow {
                coefficients,
                limit: sign * limit,
                base_value: sign * point.values[row],
                active,
            });
            coordinates.push(Coordinate::Row {
                id: point.rows[row],
                side,
            });
        }
    }
    let view = RowLimitView::new(factor.clone(), n, watched)
        .ok_or_else(|| ProblemError::Contract("activity row observer coordinate map".into()))?;
    let rhs = view
        .lift_rhs(&rhs)
        .ok_or_else(|| ProblemError::Contract("activity observer RHS map".into()))?;
    let mut primal = point.primal.clone();
    primal.extend(point.parameters.iter().map(|(_, v)| *v));
    let (mut lower, mut upper): (Vec<_>, Vec<_>) = point.bounds.iter().copied().unzip();
    // Parameter equations move by their prescribed RHS; no library bound event
    // may treat that authored perturbation as a replacement start.
    lower[point.original_variables..].fill(f64::NEG_INFINITY);
    upper[point.original_variables..].fill(f64::INFINITY);
    let (primal, lower, upper) = view
        .primal_box(&primal, &lower, &upper)
        .ok_or_else(|| ProblemError::Contract("activity observer primal map".into()))?;
    let mut multipliers = Vec::new();
    let mut forced = Vec::new();
    for bound in factor.bound_rows().into_iter().flatten() {
        // Parameter pins are prescribed equations. Their multipliers cannot release
        // an authored parameter step as if it were a variable inequality.
        if bound.var_row >= point.original_variables {
            continue;
        }
        let base = if bound.lower {
            point.lower[bound.var_row]
        } else {
            point.upper[bound.var_row]
        };
        let row = view
            .from_base(bound.row)
            .ok_or_else(|| ProblemError::Contract("activity bound multiplier map".into()))?;
        multipliers.push(BoundMultiplier { row, base });
        forced.push(row);
    }
    for (k, (row, side)) in point.factor.layout().rows.iter().enumerate() {
        if *side == Side::Equal {
            continue;
        }
        let base_row = n + k;
        let mapped = view
            .from_base(base_row)
            .ok_or_else(|| ProblemError::Contract("activity row multiplier map".into()))?;
        let sign = factor
            .sign(base_row)
            .ok_or_else(|| ProblemError::Contract("activity multiplier orientation".into()))?;
        multipliers.push(BoundMultiplier {
            row: mapped,
            base: sign * point.row_dual[row.get()],
        });
        forced.push(mapped);
    }
    // Exact active-set enforcement has no barrier weak-sigma rows. A zero
    // multiplier is still releasable through its named KKT row at t=0.
    let result = crate::quality::contained(|| {
        step_along_path(
            &view,
            &rhs,
            &primal,
            &lower,
            &upper,
            &multipliers,
            segments,
            &forced,
            &[],
            &[],
            f64::EPSILON,
        )
        .map_err(ProblemError::numerical)
    });
    if let Some(error) = factor.failure() {
        return Err(error);
    }
    execution.check()?;
    let (step, breakpoints) = match result {
        Ok(result) => result,
        Err(error) => {
            return Err(factor
                .numerical_failure()
                .unwrap_or_else(|| Arc::new(error)));
        }
    };
    if step.iter().any(|v| !v.is_finite()) {
        return Err(Arc::new(ProblemError::numerical(
            "nonfinite activity path endpoint",
        )));
    }
    let mut row_dual = point.row_dual.clone();
    for (k, (row, _)) in point.factor.layout().rows.iter().enumerate() {
        let base_row = n + k;
        let mapped = view
            .from_base(base_row)
            .ok_or_else(|| ProblemError::internal("activity endpoint row map"))?;
        row_dual[row.get()] += step[mapped]
            * factor
                .sign(base_row)
                .ok_or_else(|| ProblemError::internal("activity endpoint orientation"))?;
    }
    // Observer reactions also carry the multipliers of newly activated rows.
    // They are separate from the original base-active multiplier coordinates.
    for (j, coordinate) in coordinates.iter().enumerate() {
        let Coordinate::Row { id, side } = *coordinate else {
            return Err(Arc::new(ProblemError::internal(
                "activity observer identity",
            )));
        };
        let row = point
            .rows
            .iter()
            .position(|r| *r == id)
            .ok_or_else(|| ProblemError::internal("activity observer original row"))?;
        let observer_multiplier = view.dim() - view.n_observers() + j;
        // The observer equation is t-Gx=0, so its stationarity force is
        // -G^T mu. Original row lambda therefore receives -sign(G)*mu.
        row_dual[row] -= step[observer_multiplier] * if side == Side::Lower { -1. } else { 1. };
    }
    let mut events = Vec::with_capacity(breakpoints.len());
    for event in &breakpoints {
        let (coordinate, side) = if event.var_row < point.original_variables {
            (
                Coordinate::Variable(point.variables[event.var_row]),
                if event.lower {
                    Side::Lower
                } else {
                    Side::Upper
                },
            )
        } else if event.var_row >= n {
            let coordinate = *coordinates
                .get(event.var_row - n)
                .ok_or_else(|| ProblemError::internal("activity breakpoint observer map"))?;
            let Coordinate::Row { side, .. } = coordinate else {
                return Err(Arc::new(ProblemError::internal("activity row coordinate")));
            };
            (coordinate, side)
        } else {
            return Err(Arc::new(ProblemError::internal(
                "activity parameter reported as a bound event",
            )));
        };
        events.push(Event {
            at: event.at,
            coordinate,
            side,
            activated: event.pinned,
        });
    }
    let coverage = if breakpoints.len() == segments {
        Coverage::Partial {
            through: breakpoints.last().map_or(0., |v| v.at),
        }
    } else {
        Coverage::Tracked
    };
    let prediction = Prediction {
        variables: point.variables,
        primal: point.primal.iter().zip(&step).map(|(x, d)| x + d).collect(),
        row_dual,
        step: delta,
    };
    Ok(PathPrediction {
        prediction,
        events,
        coverage,
        work: factor.work(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{kkt::Layout, solve::Controls};
    use pse_math::index::{OriginalCol, OriginalRow};
    fn id(value: u8) -> SemanticId {
        crate::solver_tests::id(value)
    }
    fn limits() -> Limits {
        Limits {
            backsolves: 1000,
            refactorizations: 100,
            bytes: 1 << 20,
        }
    }
    fn execution() -> Execution {
        Execution::new(Arc::default(), &Controls::default())
    }
    // f=1/2(x-p)^2, with either x>=0 as a variable or as row g=x>=0.
    // The parameter is an explicit -e_p equation, distinct from releasable bounds.
    fn point(p: f64, row: bool) -> Point {
        let active = p < 0.;
        let x = p.max(0.);
        let rows = if row && active {
            vec![(OriginalRow::new(0), Side::Lower)]
        } else {
            vec![]
        };
        let mut bounds = Vec::new();
        if !row && active {
            bounds.push((OriginalCol::new(0), Side::Lower));
        }
        bounds.push((OriginalCol::new(1), Side::Equal));
        let layout = Layout {
            variables: 2,
            rows,
            bounds,
        };
        let dimension = layout.dim();
        let mut rr = vec![0, 1, 1];
        let mut cc = vec![0, 0, 1];
        let mut vv = vec![1., -1., 1.];
        for coordinate in 2..dimension {
            rr.push(coordinate);
            cc.push(coordinate);
            vv.push(0.);
        }
        if active {
            rr.push(2);
            cc.push(0);
            vv.push(if row { 1. } else { -1. });
        }
        rr.push(dimension - 1);
        cc.push(1);
        vv.push(-1.);
        let scales = vec![2., 0.25, 5., 3.][..dimension].to_vec();
        let objective = 7.;
        for ((value, row), column) in vv.iter_mut().zip(&rr).zip(&cc) {
            *value *= scales[*row] * scales[*column] / objective;
        }
        let matrix = feral::CscMatrix::from_triplets(dimension, &rr, &cc, &vv).unwrap();
        let (solver, inertia) = crate::conditioning::factor(&matrix).unwrap();
        assert_eq!(inertia.2, 0);
        let factor = super::super::KktFactor {
            solver: Arc::new(solver),
            matrix: Arc::new(matrix),
            layout: Arc::new(layout),
            scales: scales.into(),
            objective,
            bound_rows: vec![].into(),
            point: crate::square_response::point_key(&[x, p]),
            variables: vec![id(10), id(12)].into(),
            rows: if row {
                vec![id(11)].into()
            } else {
                vec![].into()
            },
            variable_bounds: vec![(f64::NEG_INFINITY, f64::INFINITY), (p, p)].into(),
            normalization: pse_math::normalization::Normalization::identity(2, usize::from(row))
                .key(),
            row_dual: if row {
                vec![p.min(0.)].into()
            } else {
                vec![].into()
            },
            lower_dual: vec![(-p).max(0.), 0.].into(),
            upper_dual: vec![0., 0.].into(),
        };
        Point {
            factor,
            original_variables: 1,
            variables: vec![id(10)],
            rows: if row { vec![id(11)] } else { vec![] },
            parameters: vec![(id(12), p)],
            pins: vec![i32::try_from(dimension - 1).unwrap()],
            primal: vec![x],
            row_dual: if row { vec![p.min(0.)] } else { vec![] },
            lower: vec![(-p).max(0.)],
            upper: vec![0.],
            values: if row { vec![x] } else { vec![] },
            row_bounds: if row {
                vec![(0., f64::INFINITY)]
            } else {
                vec![]
            },
            bounds: vec![
                if row {
                    (f64::NEG_INFINITY, f64::INFINITY)
                } else {
                    (0., f64::INFINITY)
                },
                (f64::NEG_INFINITY, f64::INFINITY),
            ],
            jacobian: if row { vec![(0, 0, 1.)] } else { vec![] },
        }
    }
    #[test]
    fn original_bound_release_uses_actual_refactor_and_natural_units() {
        let path = track(
            point(-1., false),
            &[(id(12), 1.)],
            10,
            limits(),
            execution(),
        )
        .unwrap();
        assert!((path.prediction.primal[0] - 1.).abs() < 1e-12);
        assert_eq!(path.coverage, Coverage::Tracked);
        assert_eq!(path.events.len(), 1);
        assert_eq!(path.events[0].coordinate, Coordinate::Variable(id(10)));
        assert_eq!(path.events[0].side, Side::Lower);
        assert!(!path.events[0].activated);
        assert!((path.events[0].at - 0.5).abs() < 1e-12);
        assert!(path.work.refactorizations > 0);
        assert!(path.work.backsolves > 0);
        assert!(path.work.factor_bytes > 0);
    }
    #[test]
    fn inequality_row_activation_and_release_preserve_original_lower_orientation() {
        let released = track(point(-1., true), &[(id(12), 1.)], 10, limits(), execution()).unwrap();
        assert!((released.prediction.primal[0] - 1.).abs() < 1e-12);
        assert!(released.prediction.row_dual[0].abs() < 1e-12);
        assert_eq!(
            released.events[0].coordinate,
            Coordinate::Row {
                id: id(11),
                side: Side::Lower
            }
        );
        assert_eq!(released.events[0].side, Side::Lower);
        assert!(!released.events[0].activated);
        assert!((released.events[0].at - 0.5).abs() < 1e-12);
        let activated =
            track(point(1., true), &[(id(12), -2.)], 10, limits(), execution()).unwrap();
        assert!(activated.prediction.primal[0].abs() < 1e-12);
        assert!(
            (activated.prediction.row_dual[0] + 2.).abs() < 1e-12,
            "{activated:?}"
        );
        assert_eq!(
            activated.events[0].coordinate,
            Coordinate::Row {
                id: id(11),
                side: Side::Lower
            }
        );
        assert!(activated.events[0].activated);
        assert!((activated.events[0].at - 1. / 3.).abs() < 1e-12);
    }
    #[test]
    fn segment_limit_reports_last_tracked_fraction_not_extrapolated_endpoint() {
        let path = track(point(-1., false), &[(id(12), 1.)], 1, limits(), execution()).unwrap();
        assert!(matches!(path.coverage,Coverage::Partial {through} if (through-0.5).abs()<1e-12));
        assert!((path.prediction.primal[0] - 1.).abs() < 1e-12);
    }
    #[test]
    fn terminal_work_and_cancel_stops_preserve_original_typed_causes() {
        let limit = Limits {
            backsolves: 1,
            ..limits()
        };
        let failure = track(point(-1., true), &[(id(12), 1.)], 10, limit, execution()).unwrap_err();
        assert!(matches!(
            failure.as_ref(),
            ProblemError::Limit {
                kind: crate::LimitKind::Work,
                ..
            }
        ));
        let execution = execution();
        execution
            .cancel
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let failure =
            track(point(-1., true), &[(id(12), 1.)], 10, limits(), execution).unwrap_err();
        assert!(matches!(failure.as_ref(), ProblemError::Cancelled));
    }
}
