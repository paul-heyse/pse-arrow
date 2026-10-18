// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Disposable review prototype of analyses::ResponseEvidence sample membership.
//! It tests the membership kernel only, not graph construction or scientific permission.
use std::collections::HashSet;

#[derive(Clone, Copy)]
struct Row {
    run: u64,
    sample: i64,
    time: f64,
    output: u64,
    unit: u64,
}

fn key(row: Row) -> Option<(u64, i64, u64, u64, u64)> {
    if row.time.is_nan() {
        return None;
    }
    // f64 equality equates signed zeros and rejects every NaN. Keep those semantics.
    let time = if row.time == 0.0 { 0 } else { row.time.to_bits() };
    Some((row.run, row.sample, time, row.output, row.unit))
}

fn original(samples: &[Row], response: Row, visits: &mut usize) -> bool {
    samples.iter().any(|sample| {
        *visits += 1;
        sample.run == response.run
            && sample.sample == response.sample
            && sample.time == response.time
            && sample.output == response.output
            && sample.unit == response.unit
    })
}

fn main() {
    for (states, sample_times, directions) in [(80, 5, 1), (80, 25, 1), (80, 25, 4)] {
        let samples: Vec<_> = (0..sample_times)
            .flat_map(|sample| {
                (0..states).map(move |output| Row {
                    run: 1,
                    sample,
                    time: 2.0 * sample as f64 / (sample_times - 1) as f64,
                    output,
                    unit: 7,
                })
            })
            .collect();
        let index: HashSet<_> = samples.iter().copied().filter_map(key).collect();
        let mut visits = 0;
        let mut probes = 0;
        for sample in &samples {
            for _ in 0..directions {
                assert!(original(&samples, *sample, &mut visits));
                assert!(key(*sample).is_some_and(|value| index.contains(&value)));
                probes += 1;
            }
        }
        // Changed membership dimensions and missing/NaN keys must not gain permission.
        let first = samples[0];
        let adverse = [
            Row { run: 2, ..first },
            Row { sample: -1, ..first },
            Row { time: f64::NAN, ..first },
            Row { time: f64::INFINITY, ..first },
            Row { time: 0.125, ..first },
            Row { output: 9999, ..first },
            Row { unit: 9999, ..first },
            Row { time: -0.0, ..first },
        ];
        for row in adverse {
            let mut ignored = 0;
            assert_eq!(original(&samples, row, &mut ignored), key(row).is_some_and(|k| index.contains(&k)));
        }
        // Duplicate keys and unusual float payloads retain the original membership behavior.
        let special = [first, first, Row { time: f64::NAN, ..first }, Row { time: f64::INFINITY, ..first }];
        let special_index: HashSet<_> = special.iter().copied().filter_map(key).collect();
        for row in special {
            let mut ignored = 0;
            assert_eq!(original(&special, row, &mut ignored), key(row).is_some_and(|k| special_index.contains(&k)));
        }
        println!("{{\"states\":{states},\"sample_times\":{sample_times},\"directions\":{directions},\"sample_rows\":{},\"response_rows\":{probes},\"original_sample_visits\":{visits},\"index_build_rows\":{},\"indexed_membership_probes\":{probes},\"adverse_checks\":12,\"correct\":true}}", samples.len(), samples.len());
    }
}
