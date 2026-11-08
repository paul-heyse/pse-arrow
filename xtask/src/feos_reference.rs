// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent oracle generation only. No product provider imports FeOS.
use anyhow::{Context, Result, ensure};
use datafusion::arrow::{
    array::{ArrayRef, Float64Array, Int64Array, RecordBatch, StringArray},
    compute::concat_batches,
    datatypes::{DataType, Field, Schema},
};
use feos::pcsaft::{PcSaft, PcSaftParameters};
use feos_core::{Contributions, State, parameter::IdentifierOption};
use nalgebra::DVector;
use parquet::{
    arrow::{ArrowWriter, arrow_reader::ParquetRecordBatchReaderBuilder},
    basic::Compression,
    file::properties::WriterProperties,
};
use quantity::{JOULE, KELVIN, METER, MOL, PASCAL, RGAS};
use std::{path::Path, sync::Arc};

pub(super) fn run(root: &Path, output: &Path, regenerate: bool) -> Result<()> {
    let source = root.join("packages/reference/data/gross-sadowski-2001/data/gross2001.json");
    let content = std::fs::read(&source)?;
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(&content)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    ensure!(
        hash == "f4b4018c7f02341b937c086cbb38626b72c1f3f3d9677a26b463112e16dfd404",
        "frozen parameter artifact differs"
    );
    let cases: [(&[&str], f64, f64, &[f64]); 7] = [
        (
            &["74-82-8", "74-84-0", "74-98-6"],
            300.,
            10.,
            &[0.2, 0.3, 0.5],
        ),
        (
            &["74-82-8", "74-84-0", "74-98-6"],
            350.,
            34.565566349336066,
            &[0.2, 0.3, 0.5],
        ),
        (
            &["74-82-8", "74-84-0", "74-98-6"],
            400.,
            1000.,
            &[0.6, 0.25, 0.15],
        ),
        (
            &["74-82-8", "74-84-0", "74-98-6"],
            300.,
            14000.,
            &[0.1, 0.2, 0.7],
        ),
        (
            &["74-82-8", "74-84-0", "74-98-6"],
            298.15,
            10.,
            &[0.2, 0.3, 0.5],
        ),
        (&["106-97-8"], 350., 10., &[1.]),
        (&["110-54-3"], 350., 10., &[1.]),
    ];
    let mut ids = Vec::new();
    let mut state_values = [const { Vec::<f64>::new() }; 6];
    let mut component_cases = Vec::new();
    let mut components = Vec::new();
    let mut fractions = Vec::new();
    let mut fugacities = Vec::new();
    for (index, (cas, temperature, density, x)) in cases.into_iter().enumerate() {
        let parameters =
            PcSaftParameters::from_json(cas.to_vec(), &source, None, IdentifierOption::Cas)
                .context("FeOS parameter admission")?;
        let eos = PcSaft::new(parameters);
        let state = State::new_nvt(
            &&eos,
            temperature * KELVIN,
            (1. / density) * METER.powi::<3>(),
            (DVector::from_column_slice(x), MOL),
        )?;
        let pressure = (state.pressure(Contributions::Total) / PASCAL).into_value();
        let z = pressure / ((RGAS / (JOULE / MOL / KELVIN)).into_value() * temperature * density);
        let outputs = [
            temperature,
            density,
            pressure,
            (state.residual_molar_enthalpy() / (JOULE / MOL)).into_value(),
            // Our departure convention compares ideal gas at the same T and p.
            (state.residual_molar_entropy() / (JOULE / MOL / KELVIN)).into_value()
                + (RGAS / (JOULE / MOL / KELVIN)).into_value() * z.ln(),
            (state.residual_molar_isobaric_heat_capacity() / (JOULE / MOL / KELVIN)).into_value(),
        ];
        ensure!(
            outputs.iter().all(|value| value.is_finite()),
            "nonfinite FeOS state {index}"
        );
        ids.push(i64::try_from(index)?);
        for (column, value) in state_values.iter_mut().zip(outputs) {
            column.push(value);
        }
        for ((cas, fraction), ln_phi) in cas.iter().zip(x.iter()).zip(state.ln_phi().iter()) {
            ensure!(ln_phi.is_finite(), "nonfinite FeOS fugacity");
            component_cases.push(i64::try_from(index)?);
            components.push(*cas);
            fractions.push(*fraction);
            fugacities.push(*ln_phi);
        }
    }
    if regenerate {
        std::fs::create_dir_all(output)?;
    }
    let mut fields = vec![Field::new("case", DataType::Int64, false)];
    fields.extend(
        [
            "T",
            "rho",
            "pressure",
            "h_residual",
            "s_residual",
            "cp_residual",
        ]
        .map(|name| Field::new(name, DataType::Float64, false)),
    );
    let mut arrays: Vec<ArrayRef> = vec![Arc::new(Int64Array::from(ids))];
    arrays.extend(
        state_values
            .into_iter()
            .map(|values| -> ArrayRef { Arc::new(Float64Array::from(values)) }),
    );
    retain(
        &output.join("states.parquet"),
        RecordBatch::try_new(Arc::new(Schema::new(fields)), arrays)?,
        regenerate,
    )?;
    let fields = [
        Field::new("case", DataType::Int64, false),
        Field::new("subject", DataType::Utf8, false),
        Field::new("fraction", DataType::Float64, false),
        Field::new("ln_phi", DataType::Float64, false),
    ];
    retain(
        &output.join("components.parquet"),
        RecordBatch::try_new(
            Arc::new(Schema::new(fields.to_vec())),
            vec![
                Arc::new(Int64Array::from(component_cases)),
                Arc::new(StringArray::from(components)),
                Arc::new(Float64Array::from(fractions)),
                Arc::new(Float64Array::from(fugacities)),
            ],
        )?,
        regenerate,
    )?;
    let action = if regenerate { "Wrote" } else { "Compared" };
    println!(
        "{action} FeOS 0.10.1 references for seven states and all component fugacities at fixed T and p"
    );
    Ok(())
}

fn retain(path: &Path, actual: RecordBatch, regenerate: bool) -> Result<()> {
    if regenerate {
        return write(path, actual);
    }
    let reader = ParquetRecordBatchReaderBuilder::try_new(std::fs::File::open(path)?)?;
    let schema = reader.schema().clone();
    ensure!(
        actual.schema() == schema,
        "stale reference schema in {}",
        path.display()
    );
    let batches = reader
        .build()?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let frozen = concat_batches(&schema, &batches)?;
    ensure!(
        actual.num_rows() == frozen.num_rows(),
        "stale reference population in {}",
        path.display()
    );
    for (index, field) in schema.fields().iter().enumerate() {
        let (observed, expected) = (actual.column(index), frozen.column(index));
        let same = if *field.data_type() == DataType::Float64
            && !matches!(field.name().as_str(), "T" | "rho" | "fraction")
        {
            let observed = observed
                .as_any()
                .downcast_ref::<Float64Array>()
                .context("generated reference must have typed floating values")?;
            let expected = expected
                .as_any()
                .downcast_ref::<Float64Array>()
                .context("retained reference must have typed floating values")?;
            // Both runs use the same FeOS release and authored physical inputs.
            // Bound binary64 evaluation variation separately from native acceptance.
            observed.iter().zip(expected.iter()).all(|(left, right)| {
                matches!((left, right), (Some(left), Some(right))
                    if left.is_finite() && right.is_finite()
                        && (left - right).abs()
                            <= 64. * f64::EPSILON * left.abs().max(right.abs()).max(1.))
            })
        } else {
            observed.to_data() == expected.to_data()
        };
        ensure!(
            same,
            "stale reference column {} in {}",
            field.name(),
            path.display()
        );
    }
    Ok(())
}

fn write(path: &Path, batch: RecordBatch) -> Result<()> {
    let properties = WriterProperties::builder()
        .set_compression(Compression::ZSTD(Default::default()))
        .build();
    let mut writer = ArrowWriter::try_new(
        std::fs::File::create(path)?,
        batch.schema(),
        Some(properties),
    )?;
    writer.write(&batch)?;
    writer.close()?;
    Ok(())
}
