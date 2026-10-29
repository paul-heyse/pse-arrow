// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public store-free design generation, physical geometry and retained realized outputs.
//! Requires force-validate; no solver, canonical service or Python fixture.
#![recursion_limit = "512"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "public native fixture and independent design assertions fail directly"
)]

#[path = "reference_support/mod.rs"]
mod reference_support;

use futures_util::FutureExt;
use pse_model::scalars::FiniteBound;
use pse_runtime::{
    CancelSource,
    workflow::{
        BindingTarget, ModelingPackage, PointOverlay,
        sampling::{
            DesignCoordinate, DesignExport, DesignKind, DesignLevel, DesignRequest, GeneratedDesign,
        },
    },
};
use reference_support::{NativeContext, Result};
use std::panic::AssertUnwindSafe;

const SOURCE: &str = "package designs {def Root {
    var x:Scalar; annotation bounds x(-2,6); annotation start x(2); eq scalar:x==2;
    var length:Length; annotation bounds length(10{m},26{m});
    annotation start length(18{m}); eq distance:length==18{m};
}}";

fn finite(value: f64) -> FiniteBound {
    FiniteBound::try_new(value).unwrap()
}

async fn request(context: &NativeContext, package: &ModelingPackage) -> Result<DesignRequest> {
    let services = context.runtime.preparation_services();
    let root = package
        .declarations(&services)
        .await?
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let physical = package.physical_context();
    let mut coordinates = Vec::new();
    for (path, name, bounds) in [("x", "Scalar", [-2., 6.]), ("length", "Length", [10., 26.])] {
        let quantity = physical.resolve_quantity(name)?;
        let unit = physical
            .quantities()
            .quantity_type(quantity)?
            .canonical_unit;
        coordinates.push(DesignCoordinate {
            target: BindingTarget::Path(path.into()),
            quantity,
            unit,
            range: Some(bounds.map(finite)),
            levels: vec![],
        });
    }
    Ok(DesignRequest {
        root,
        bindings: Default::default(),
        coordinates,
        kind: DesignKind::Classic,
        count: 8,
        seed: 91,
        grid_levels: vec![],
        maximum_work: 1_000_000,
    })
}

fn assert_physical_points(points: &[PointOverlay], request: &DesignRequest) {
    assert_eq!(points.len(), request.count);
    for point in points {
        assert_eq!(point.assignments.len(), request.coordinates.len());
        for (assignment, coordinate) in point.assignments.iter().zip(&request.coordinates) {
            assert_eq!(assignment.target, coordinate.target);
            assert_eq!(assignment.value.quantity, coordinate.quantity);
            assert_eq!(assignment.value.unit, coordinate.unit);
            let value = assignment.value.magnitude.into_inner();
            let [lower, upper] = coordinate.range.unwrap().map(FiniteBound::into_inner);
            assert!(value.is_finite() && value >= lower && value <= upper);
        }
    }
}

fn assert_geometry(points: &[PointOverlay], request: &DesignRequest) {
    assert_physical_points(points, request);
    if request.kind == DesignKind::Grid {
        // Independent Cartesian expectation: the last coordinate varies fastest.
        let expected = [
            [-2., 10.],
            [-2., 18.],
            [-2., 26.],
            [6., 10.],
            [6., 18.],
            [6., 26.],
        ];
        assert_eq!(points.len(), expected.len());
        for (point, expected) in points.iter().zip(expected) {
            for (assignment, expected) in point.assignments.iter().zip(expected) {
                assert_eq!(assignment.value.magnitude.into_inner(), expected);
            }
        }
    } else if !request.coordinates[0].levels.is_empty() {
        // Twenty realized occurrences from two integral levels remain twenty
        // inputs. Duplicate coordinates are not deduplicated into a set.
        assert_eq!(points.len(), 20);
        for level in [0., 1.] {
            assert_eq!(
                points
                    .iter()
                    .filter(|point| point.assignments[0].value.magnitude.into_inner() == level)
                    .count(),
                10
            );
        }
    } else {
        assert_eq!(points.len(), 8);
        for (axis, [lower, width]) in [[-2., 8.], [10., 16.]].into_iter().enumerate() {
            let normalized = points
                .iter()
                .map(|point| (point.assignments[axis].value.magnitude.into_inner() - lower) / width)
                .collect::<Vec<_>>();
            assert!(normalized.iter().all(|value| (0.0..1.0).contains(value)));
            if request.kind != DesignKind::Random {
                let mut strata = normalized
                    .iter()
                    .map(|value| (value * 8.).floor() as usize)
                    .collect::<Vec<_>>();
                strata.sort_unstable();
                assert_eq!(strata, vec![0, 1, 2, 3, 4, 5, 6, 7]);
            }
            if matches!(
                request.kind,
                DesignKind::Centered | DesignKind::CenteredMaximin
            ) {
                for value in normalized {
                    assert!((value * 8. - (value * 8.).floor() - 0.5).abs() <= 1e-12);
                }
            }
        }
    }
}

#[tokio::test]
async fn public_native_designs_preserve_physical_geometry_and_outlive_producers() -> Result<()> {
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        assert!(
            context.runtime.canonical_store().is_err(),
            "store-free construction"
        );
        let package = context.package(SOURCE).await?;
        let mut request = request(&context, &package).await?;
        let producer = context.runtime.clone();
        let services = producer.preparation_services();
        let mut retained: Vec<(DesignRequest, GeneratedDesign, DesignExport)> = Vec::new();
        for kind in [
            DesignKind::Classic,
            DesignKind::Centered,
            DesignKind::Random,
            DesignKind::Maximin,
            DesignKind::CenteredMaximin,
            DesignKind::Grid,
        ] {
            request.kind = kind;
            if kind == DesignKind::Grid {
                request.count = 6;
                request.grid_levels = vec![2, 3];
            }
            let design = Box::pin(package.generate_design(
                &services,
                &request,
                Default::default(),
                &CancelSource::new(),
            ))
            .await?;
            assert_geometry(design.points(), &request);
            let alias = design.clone();
            let export = design.export(&services)?;
            drop(design);
            retained.push((request.clone(), alias, export));
        }
        request.kind = DesignKind::Classic;
        request.count = 20;
        request.grid_levels.clear();
        request.coordinates.truncate(1);
        request.coordinates[0].range = Some([finite(0.), finite(1.)]);
        request.coordinates[0].levels = vec![DesignLevel::Integer(0), DesignLevel::Integer(1)];
        let design = Box::pin(package.generate_design(
            &services,
            &request,
            Default::default(),
            &CancelSource::new(),
        ))
        .await?;
        let export = design.export(&services)?;
        retained.push((request, design, export));
        drop(package);
        drop(services);
        producer.clear_program_cache();
        drop(producer);
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(retained)
    })
    .catch_unwind()
    .await;
    // Drain the native preparation owner even on an assertion panic. Returned
    // designs/exports retain their own realized point storage after root disposal.
    let closed = context.close().await;
    drop(context);
    let retained = match outcome {
        Err(panic) => {
            let _ = closed;
            std::panic::resume_unwind(panic);
        }
        Ok(result) => {
            closed?;
            result?
        }
    };
    for (request, design, export) in retained {
        assert_geometry(design.points(), &request);
        assert_eq!(export.points, design.points());
        assert_eq!(design.attribution().kind, request.kind);
        assert_eq!(design.attribution().count, request.count);
        assert_eq!(export.attribution.kind, request.kind);
        assert_eq!(export.attribution.count, request.count);
        drop(design);
        // Serialize the retained actual coordinates, never regenerate from the
        // attribution seed or assert portability of a supplier's random stream.
        let document = serde_json::to_value(&export)?;
        assert_eq!(document["points"].as_array().unwrap().len(), request.count);
        assert_geometry(&export.points, &request);
    }
    Ok(())
}

#[tokio::test]
async fn public_native_designs_refuse_invalid_physics_shapes_work_and_cancellation() -> Result<()> {
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        let package = context.package(SOURCE).await?;
        let request = request(&context, &package).await?;
        let services = context.runtime.preparation_services();
        let valid = Box::pin(package.generate_design(
            &services,
            &request,
            Default::default(),
            &CancelSource::new(),
        ))
        .await?;
        assert_geometry(valid.points(), &request);
        let mut refusals = Vec::new();
        let mut invalid = request.clone();
        invalid.coordinates[0].target = BindingTarget::Path("missing".into());
        refusals.push(("unknown target", invalid));
        let mut invalid = request.clone();
        invalid.coordinates[0].unit = request.coordinates[1].unit;
        refusals.push(("registered length unit for scalar", invalid));
        let mut invalid = request.clone();
        invalid.coordinates[0].quantity = request.coordinates[1].quantity;
        invalid.coordinates[0].unit = request.coordinates[1].unit;
        refusals.push(("length meaning for scalar target", invalid));
        let mut invalid = request.clone();
        invalid.coordinates[0].range = Some([finite(6.), finite(-2.)]);
        refusals.push(("reversed interval", invalid));
        let mut invalid = request.clone();
        invalid.coordinates[0].range = Some([finite(-3.), finite(6.)]);
        refusals.push(("outside authored bounds", invalid));
        let mut invalid = request.clone();
        invalid.count = 0;
        refusals.push(("empty requested population", invalid));
        let mut invalid = request.clone();
        invalid.maximum_work = 1;
        refusals.push(("insufficient complete-design work", invalid));
        let mut invalid = request.clone();
        invalid.kind = DesignKind::Grid;
        invalid.grid_levels = vec![2, 3];
        refusals.push(("grid product differs from count", invalid));
        let mut invalid = request.clone();
        invalid.kind = DesignKind::Maximin;
        invalid.count = 1;
        refusals.push(("maximin singleton", invalid));
        for (label, invalid) in refusals {
            assert!(
                Box::pin(package.generate_design(
                    &services,
                    &invalid,
                    Default::default(),
                    &CancelSource::new(),
                ))
                .await
                .is_err(),
                "{label} must refuse"
            );
        }
        let cancelled = CancelSource::new();
        cancelled.cancel();
        assert!(
            Box::pin(package.generate_design(&services, &request, Default::default(), &cancelled,))
                .await
                .is_err()
        );
        // Refusals and a cancelled request do not close the healthy producer.
        let after = Box::pin(package.generate_design(
            &services,
            &request,
            Default::default(),
            &CancelSource::new(),
        ))
        .await?;
        assert_geometry(after.points(), &request);
        assert_geometry(valid.points(), &request);
        Ok(())
    })
    .catch_unwind()
    .await;
    let closed = context.close().await;
    match outcome {
        Err(panic) => {
            let _ = closed;
            std::panic::resume_unwind(panic);
        }
        Ok(result) => {
            closed?;
            result
        }
    }
}
