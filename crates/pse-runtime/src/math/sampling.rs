// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One finite library call under existing native admission, original clock and joined ownership.
use super::{MathRuntimeError, MathService};
use crate::workflow::{
    BindingAssignment, BindingQuantity, PointOverlay,
    sampling::{
        Coordinate, DesignAttribution, DesignKind, DesignRequest, GeneratedDesign, interpolate,
    },
};
use egobox_doe::{Lhs, LhsKind, Random, SamplingMethod};
use ndarray::Array2;
use ndarray_rand::rand::SeedableRng;
use pse_model::scalars::FiniteBound;
use rand_xoshiro::Xoshiro256Plus;
use std::sync::Arc;

fn extent_error() -> MathRuntimeError {
    MathRuntimeError::Limit("design extent or work overflow")
}
fn add(a: usize, b: usize) -> Result<usize, MathRuntimeError> {
    a.checked_add(b).ok_or_else(extent_error)
}
fn mul(a: usize, b: usize) -> Result<usize, MathRuntimeError> {
    a.checked_mul(b).ok_or_else(extent_error)
}
/// Allocation liveness for egobox-doe 0.35.1 lhs.rs/random.rs/utils.rs:
/// classic retains cut/a/c, rnd/rdpoints/lhs and column temporaries; map conversion may
/// overlap another complete array. Maximin adds retained candidates and pdist's tuple
/// vector (geometric growth including relocation overlap), distance output and Rayon
/// collection storage. These are conservative capacity bounds, not RSS measurements.
fn extents(
    request: &DesignRequest,
    coordinates: &[Coordinate],
    threads: usize,
    stack: usize,
    foreign: usize,
) -> Result<(usize, usize), MathRuntimeError> {
    let n = request.count;
    let d = coordinates.len();
    let cells = mul(n, d)?;
    let mut metadata = 4096usize;
    let mut repeated_paths = 0usize;
    for c in coordinates {
        let path = match &c.description.target {
            crate::workflow::BindingTarget::Path(p) => p.len(),
            _ => 0,
        };
        metadata = add(
            metadata,
            add(
                size_of::<Coordinate>(),
                add(
                    path,
                    mul(
                        c.description.levels.len(),
                        size_of::<crate::workflow::sampling::DesignLevel>(),
                    )?,
                )?,
            )?,
        )?;
        repeated_paths = add(repeated_paths, path)?;
    }
    let retained = add(
        metadata,
        add(
            mul(n, size_of::<PointOverlay>())?,
            add(
                mul(cells, size_of::<BindingAssignment>())?,
                mul(n, repeated_paths)?,
            )?,
        )?,
    )?;
    let arrays = mul(
        add(mul(cells, 12)?, add(mul(n, 12)?, mul(d, 8)?)?)?,
        size_of::<f64>(),
    )?;
    let grid_workspace = if request.kind == DesignKind::Grid {
        mul(d, size_of::<usize>())?
    } else {
        0
    };
    let pairs = if request.kind.quadratic() {
        mul(n, n - 1)? / 2
    } else {
        0
    };
    let work = add(mul(d, 128)?, add(mul(cells, 16)?, mul(mul(pairs, d)?, 5)?)?)?;
    if work > request.maximum_work {
        return Err(MathRuntimeError::Limit("design generation work allowance"));
    }
    let team = if request.kind.quadratic() {
        add(mul(threads, add(stack, 4096)?)?, foreign)?
    } else {
        0
    };
    let peak = add(
        retained,
        add(grid_workspace, add(arrays, add(mul(pairs, 96)?, team)?)?)?,
    )?;
    if peak > isize::MAX as usize {
        return Err(extent_error());
    }
    Ok((peak, retained))
}
impl MathService {
    pub(crate) async fn generate_design_job(
        self: &Arc<Self>,
        request: &DesignRequest,
        coordinates: Vec<Coordinate>,
        cancel: &crate::CancelSource,
    ) -> Result<GeneratedDesign, MathRuntimeError> {
        let threads = if request.kind.quadratic() {
            self.cores()
        } else {
            1
        };
        let stack = self.stack_bytes();
        let (peak, retained) = extents(
            request,
            &coordinates,
            threads,
            stack,
            self.policy.foreign_bytes,
        )?;
        if peak > self.policy.worker_bytes {
            return Err(MathRuntimeError::Limit("design worker byte allowance"));
        }
        let request = request.clone();
        let native = pse_diagnostics::Cancellation::new(cancel.token().is_cancelled());
        let job=self.job_retained_scoped(threads,peak,native.clone(),cancel.deadline(),move |stop| {
            let generated=|| construct(&request,&coordinates,&stop);
            let points=if request.kind.quadratic() {
                let pool=super::joined_pool::JoinedPool::new(threads,stack,super::joined_pool::spawn_pool_worker)?;
                // install completes before Drop joins the team, inside the admitted job.
                pool.install(generated)??
            } else { generated()? };
            if stop.is_cancelled() { return Err(MathRuntimeError::Cancelled); }
            let algorithm=if request.kind==DesignKind::Grid {
                "local-cartesian-v1;bounded-map-v1;unit-preimage-v1;grid-last-axis-fast-v1"
            } else {
                "egobox-doe=0.35.1;Xoshiro256Plus=0.6.0;normalized;bounded-map-v1;unit-preimage-v1"
            };
            let attribution=DesignAttribution {kind:request.kind,seed:request.seed,count:request.count,algorithm:algorithm.into(),coordinates:request.coordinates};
            Ok(((points,attribution),retained))
        });
        let ((points, attribution), owner) = cancel.join_native(&native, job, || {}).await?;
        Ok(GeneratedDesign::from_parts(points, attribution, owner))
    }
}
fn construct(
    request: &DesignRequest,
    coordinates: &[Coordinate],
    stop: &pse_diagnostics::Cancellation,
) -> Result<Vec<PointOverlay>, MathRuntimeError> {
    if stop.is_cancelled() {
        return Err(MathRuntimeError::Cancelled);
    }
    let n = request.count;
    let d = coordinates.len();
    let grid_strides = if request.kind == DesignKind::Grid {
        if request.grid_levels.len() != d {
            return Err(extent_error());
        }
        let mut strides = vec![1usize; d];
        let mut product = 1usize;
        for j in (0..d).rev() {
            let count = request.grid_levels[j];
            if count == 0 {
                return Err(extent_error());
            }
            strides[j] = product;
            product = mul(product, count)?;
        }
        if product != n {
            return Err(extent_error());
        }
        Some(strides)
    } else {
        None
    };
    // This internal matrix is structurally valid. Supplier sample() never performs mapping.
    let limits = Array2::from_shape_fn((d, 2), |(_, column)| column as f64);
    let normalized = match request.kind {
        DesignKind::Grid => None,
        DesignKind::Random => Some(
            Random::<f64, _>::new_with_rng(&limits, Xoshiro256Plus::seed_from_u64(request.seed))
                .normalized_sample(n),
        ),
        kind => {
            let kind = match kind {
                DesignKind::Classic => LhsKind::Classic,
                DesignKind::Centered => LhsKind::Centered,
                DesignKind::Maximin => LhsKind::Maximin,
                DesignKind::CenteredMaximin => LhsKind::CenteredMaximin,
                _ => return Err(extent_error()),
            };
            Some(
                Lhs::<f64, _>::new_with_rng(&limits, Xoshiro256Plus::seed_from_u64(request.seed))
                    .kind(kind)
                    .normalized_sample(n),
            )
        }
    };
    // Supplier calls have no polling hook. Their preflight work/byte extent bounds the
    // noninterruptible region; stop is observed before mapping/publication, never dropped work.
    if stop.is_cancelled() {
        return Err(MathRuntimeError::Cancelled);
    }
    let mut points = Vec::with_capacity(n);
    for i in 0..n {
        if stop.is_cancelled() {
            return Err(MathRuntimeError::Cancelled);
        }
        let mut assignments = Vec::with_capacity(d);
        for (j, c) in coordinates.iter().enumerate() {
            let (u, index) = if let Some(normalized) = &normalized {
                (normalized[[i, j]], None)
            } else {
                let stride = grid_strides.as_ref().ok_or_else(extent_error)?[j];
                let index = (i / stride) % request.grid_levels[j];
                (
                    if request.grid_levels[j] == 1 {
                        0.0
                    } else {
                        index as f64 / (request.grid_levels[j] - 1) as f64
                    },
                    Some(index),
                )
            };
            let value = if c.description.levels.is_empty() {
                interpolate(c.lower, c.upper, u)
            } else {
                let index = index.unwrap_or_else(|| level_index(u, c.description.levels.len()));
                c.description.levels[index].value().ok_or_else(|| {
                    pse_math::MathError::Contract("admitted integer design level invariant".into())
                })?
            };
            if !value.is_finite() || value < c.lower || value > c.upper {
                return Err(
                    pse_math::MathError::Contract("design mapping invariant".into()).into(),
                );
            }
            let magnitude = FiniteBound::try_new(value).map_err(|_| {
                pse_math::MathError::Contract("finite design mapping invariant".into())
            })?;
            assignments.push(BindingAssignment {
                target: c.description.target.clone(),
                value: BindingQuantity {
                    magnitude,
                    quantity: c.description.quantity,
                    unit: c.description.unit,
                },
            });
        }
        points.push(PointOverlay { assignments });
    }
    Ok(points)
}
/// Exact floor(u*levels) from binary64's dyadic representation, without integer-to-float loss.
fn level_index(u: f64, levels: usize) -> usize {
    if u == 1.0 {
        return levels - 1;
    }
    let bits = u.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as u32;
    if exponent == 0 {
        return 0;
    }
    let significand = (bits & ((1_u64 << 52) - 1)) | (1_u64 << 52);
    let shift = 1075 - exponent;
    if shift >= 128 {
        0
    } else {
        ((u128::from(significand) * (levels as u128)) >> shift) as usize
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interpolation_preserves_extreme_bounds_and_endpoint_bits_without_clamp() {
        for (a, b) in [
            (-f64::MAX, f64::MAX),
            (f64::MAX.next_down(), f64::MAX),
            (-f64::MAX, -f64::MAX.next_down()),
            (-f64::MIN_POSITIVE, f64::MIN_POSITIVE),
            (-0.0, 0.0),
            (f64::from_bits(1), f64::from_bits(5)),
        ] {
            assert_eq!(interpolate(a, b, 0.0).to_bits(), a.to_bits());
            assert_eq!(interpolate(a, b, 1.0).to_bits(), b.to_bits());
            for i in 0..=1024 {
                let v = interpolate(a, b, i as f64 / 1024.0);
                assert!(v.is_finite() && v >= a && v <= b, "{a} {b} {v}");
            }
        }
    }
}

#[cfg(test)]
mod admission_tests {
    use super::*;
    #[tokio::test]
    async fn sampler_byte_and_work_refusal_precedes_library_construction() {
        let (service, _cache) = super::super::tests::service_with_policy(
            128 << 20,
            super::super::MathPolicy {
                worker_bytes: 1024,
                foreign_bytes: 0,
                ..Default::default()
            },
        );
        let quantity = pse_quantity::QuantityTypeId::from_id(pse_ids::SemanticId::NIL);
        let description = crate::workflow::sampling::DesignCoordinate {
            target: crate::workflow::BindingTarget::Member(pse_ids::SemanticId::NIL),
            quantity,
            unit: pse_quantity::UnitId::from_id(pse_ids::SemanticId::NIL),
            range: Some([
                FiniteBound::try_new(0.).unwrap(),
                FiniteBound::try_new(1.).unwrap(),
            ]),
            levels: vec![],
        };
        let mut request = DesignRequest {
            root: pse_modeling::DeclarationId::from_id(pse_ids::SemanticId::NIL),
            bindings: Default::default(),
            coordinates: vec![description.clone()],
            kind: DesignKind::Classic,
            count: 100,
            seed: 0,
            grid_levels: vec![],
            maximum_work: 100000,
        };
        let coordinates = vec![Coordinate {
            description,
            lower: 0.,
            upper: 1.,
        }];
        let baseline = service.pool.reserved();
        let error = service
            .generate_design_job(&request, coordinates.clone(), &crate::CancelSource::new())
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            MathRuntimeError::Limit("design worker byte allowance")
        ));
        assert_eq!(service.pool.reserved(), baseline);
        request.maximum_work = 1;
        let error = service
            .generate_design_job(&request, coordinates, &crate::CancelSource::new())
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            MathRuntimeError::Limit("design generation work allowance")
        ));
        assert_eq!(service.pool.reserved(), baseline);
    }
    #[tokio::test]
    async fn high_dimensional_singleton_grid_completes_under_linear_admitted_work() {
        let dimensions = 32_768;
        let (service, _cache) = super::super::tests::service_with_policy(
            128 << 20,
            super::super::MathPolicy {
                worker_bytes: 64 << 20,
                ..Default::default()
            },
        );
        let coordinates = (0..dimensions)
            .map(|index| {
                let value = index as f64;
                let description = crate::workflow::sampling::DesignCoordinate {
                    target: crate::workflow::BindingTarget::Path(format!("x{index}")),
                    quantity: pse_quantity::QuantityTypeId::from_id(pse_ids::SemanticId::NIL),
                    unit: pse_quantity::UnitId::from_id(pse_ids::SemanticId::NIL),
                    range: Some([FiniteBound::try_new(value).unwrap(); 2]),
                    levels: vec![],
                };
                Coordinate {
                    description,
                    lower: value,
                    upper: value,
                }
            })
            .collect::<Vec<_>>();
        let request = DesignRequest {
            root: pse_modeling::DeclarationId::from_id(pse_ids::SemanticId::NIL),
            bindings: Default::default(),
            coordinates: coordinates
                .iter()
                .map(|coordinate| coordinate.description.clone())
                .collect(),
            kind: DesignKind::Grid,
            count: 1,
            seed: 0,
            grid_levels: vec![1; dimensions],
            maximum_work: 150 * dimensions,
        };
        let design = service
            .generate_design_job(&request, coordinates, &crate::CancelSource::new())
            .await
            .unwrap();
        assert_eq!(design.points().len(), 1);
        assert_eq!(design.points()[0].assignments.len(), dimensions);
        for (index, assignment) in design.points()[0].assignments.iter().enumerate() {
            assert_eq!(
                assignment.target,
                crate::workflow::BindingTarget::Path(format!("x{index}"))
            );
            assert_eq!(assignment.value.magnitude.into_inner(), index as f64);
        }
    }
    #[test]
    fn exact_equal_slice_mapping_handles_endpoints_and_large_level_counts() {
        assert_eq!(level_index(0., 3), 0);
        assert_eq!(level_index(1., 3), 2);
        assert_eq!(level_index(0.5, 3), 1);
        assert_eq!(level_index(0.5, usize::MAX), usize::MAX / 2);
        assert_eq!(level_index(f64::from_bits(1), usize::MAX), 0);
        assert!(level_index(1.0_f64.next_down(), usize::MAX) < usize::MAX);
    }
}
