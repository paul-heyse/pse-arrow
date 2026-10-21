// PROBE A (throwaway): body granularity measurements over admitted PreparedBody values.
#![allow(missing_docs, clippy::unwrap_used, reason = "throwaway design-review probe")]
use super::*;
use crate::guarded::PreparedBody;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::time::Instant;
use symbolica::atom::AtomCore;
use symbolica::evaluate::OperationCount;
use symbolica::id::Replacement;

/// One per-output occurrence: an admitted body and the global coordinate index of each
/// body input, in body input order.
#[derive(Debug)]
pub struct Entry<'a> {
    pub body: &'a PreparedBody,
    pub globals: Vec<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct Measure {
    /// value, first, second: total ops (add+mul+inv+fun)
    pub ops: [usize; 3],
    pub temps: [usize; 3],
    pub instructions: [usize; 3],
    pub build_ms: [f64; 3],
    pub width: [usize; 3],
    pub skipped: [bool; 3],
}
impl Measure {
    fn add(&mut self, other: &Measure) {
        for i in 0..3 {
            self.ops[i] += other.ops[i];
            self.temps[i] += other.temps[i];
            self.instructions[i] += other.instructions[i];
            self.build_ms[i] += other.build_ms[i];
            self.width[i] = self.width[i].max(other.width[i]);
            self.skipped[i] |= other.skipped[i];
        }
    }
}
fn total(c: OperationCount) -> usize {
    c.additions + c.multiplications + c.inversions + c.function_calls
}
/// Value/first/second Taylor programs of one exact evaluator whose parameters are the
/// independent coordinates `params` (global ids). Each parameter carries only its own axis.
fn taylor(
    evaluator: &ExpressionEvaluator<Complex<Rational>>,
    params: &[usize],
    max_width: usize,
    measure: &mut Measure,
    start: Instant,
) -> Result<(), String> {
    let export = evaluator.export_instructions();
    measure.ops[0] = total(evaluator.count_operations());
    measure.temps[0] = export.temporary_count;
    measure.instructions[0] = export.instructions.len();
    measure.build_ms[0] = start.elapsed().as_secs_f64() * 1e3;
    measure.width[0] = 1;
    let n = params.len();
    for order in 1..=2usize {
        let mut shape = vec![vec![0usize; n]];
        for i in 0..n {
            let mut t = vec![0; n];
            t[i] = 1;
            shape.push(t);
        }
        if order == 2 {
            for i in 0..n {
                for j in i..n {
                    let mut t = vec![0; n];
                    t[i] += 1;
                    t[j] += 1;
                    shape.push(t);
                }
            }
        }
        measure.width[order] = shape.len();
        if shape.len() > max_width {
            measure.skipped[order] = true;
            continue;
        }
        let mut zeros = Vec::new();
        for p in 0..n {
            for (c, powers) in shape.iter().enumerate().skip(1) {
                if powers
                    .iter()
                    .enumerate()
                    .any(|(axis, &power)| power > 0 && axis != p)
                {
                    zeros.push((p, c));
                }
            }
        }
        let t0 = Instant::now();
        let dual = symbolica::prelude::HyperDual::<Complex<Rational>>::new(shape);
        let vectorized = evaluator
            .clone()
            .vectorize(&symbolica::prelude::Dualizer::new(dual, zeros))?;
        let export = vectorized.export_instructions();
        measure.ops[order] = total(vectorized.count_operations());
        measure.temps[order] = export.temporary_count;
        measure.instructions[order] = export.instructions.len();
        measure.build_ms[order] = t0.elapsed().as_secs_f64() * 1e3;
    }
    Ok(())
}
fn exact(
    expressions: &[Atom],
    parameters: &[Atom],
) -> Result<ExpressionEvaluator<Complex<Rational>>, String> {
    exact_evaluator(
        expressions,
        parameters,
        Optimization::default(),
        &Arc::new(std::sync::atomic::AtomicBool::new(false)),
    )
    .map_err(|e| e.to_string())
}
/// Rename body formals `formal(i)` to global positional formals `formal(globals[i])`.
fn relabel(atom: &Atom, globals: &[usize]) -> Result<Atom, String> {
    let replacements = globals
        .iter()
        .enumerate()
        .map(|(i, &g)| {
            Ok(Replacement::new(
                formal(i).map_err(|e| e.to_string())?.to_pattern(),
                formal(g + 1_000_000).map_err(|e| e.to_string())?.to_pattern(),
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(atom.replace_multiple(&replacements))
}
fn used_globals(atom: &Atom, universe: &BTreeMap<symbolica::atom::Symbol, usize>) -> BTreeSet<usize> {
    let mut out = BTreeSet::new();
    atom.as_view().visitor(&mut |node| {
        if let symbolica::atom::AtomView::Var(v) = node
            && let Some(&g) = universe.get(&v.get_symbol())
        {
            out.insert(g);
        }
        true
    });
    out
}

/// Measure one group of per-output occurrences (one instance). Returns a report line set.
pub fn measure_group(label: &str, entries: &[Entry<'_>], max_width: usize) -> (String, [Measure; 6]) {
    let mut log = String::new();
    // Global coordinate universe (offset to avoid clashing with body-local formals).
    let globals = entries
        .iter()
        .flat_map(|e| e.globals.iter().copied())
        .collect::<BTreeSet<_>>();
    let universe = globals
        .iter()
        .map(|&g| {
            let atom = formal(g + 1_000_000).unwrap();
            let symbolica::atom::AtomView::Var(v) = atom.as_view() else {
                unreachable!()
            };
            (v.get_symbol(), g)
        })
        .collect::<BTreeMap<_, _>>();
    // (a) today's compiled body census, per occurrence.
    let mut today = Measure::default();
    let mut today_excluded = Measure::default();
    let mut today_failed = 0usize;
    let scope = pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None);
    for entry in entries {
        let substitutable = entry.body.probe_expressions().iter().all(Option::is_some);
        let today = if substitutable { &mut today } else { &mut today_excluded };
        let outputs = (0..entry.body.outputs.len()).collect::<Vec<_>>();
        let coordinates = (0..entry.body.probe_inputs()).collect::<Vec<_>>();
        for (k, order) in [
            pse_kernels::DerivativeOrder::Value,
            pse_kernels::DerivativeOrder::First,
            pse_kernels::DerivativeOrder::Second,
        ]
        .into_iter()
        .enumerate()
        {
            let t0 = Instant::now();
            match entry.body.compile(
                &outputs,
                &coordinates,
                order,
                Optimization::default(),
                crate::jets::EvaluationLimits::default(),
                &scope,
            ) {
                Ok(compiled) => {
                    today.build_ms[k] += t0.elapsed().as_secs_f64() * 1e3;
                    for census in compiled.census().filter(|c| c.order == order) {
                        today.ops[k] += census.library_operations;
                        today.instructions[k] += census.library_instructions;
                        today.temps[k] += census.frame_entries;
                        today.width[k] = today.width[k].max(census.derivative_components);
                    }
                }
                Err(_) => {
                    today_failed += 1;
                    today.skipped[k] = true;
                }
            }
        }
    }
    // Substituted per-output atoms over global formals.
    let mut atoms = Vec::new();
    let mut excluded = 0usize;
    for entry in entries {
        assert_eq!(entry.body.probe_inputs(), entry.globals.len(), "body input arity");
        for expression in entry.body.probe_expressions() {
            match expression {
                Some(atom) => atoms.push(relabel(atom, &entry.globals).unwrap()),
                None => excluded += 1,
            }
        }
    }
    // (a') per-output inlined Symbolica evaluators (own coordinates only).
    let mut inlined = Measure::default();
    let mut inlined_failed = 0usize;
    for atom in &atoms {
        let used = used_globals(atom, &universe).into_iter().collect::<Vec<_>>();
        let params = used
            .iter()
            .map(|&g| formal(g + 1_000_000).unwrap())
            .collect::<Vec<_>>();
        let t0 = Instant::now();
        let mut m = Measure::default();
        match exact(std::slice::from_ref(atom), &params)
            .and_then(|ev| taylor(&ev, &used, max_width, &mut m, t0))
        {
            Ok(()) => inlined.add(&m),
            Err(_) => inlined_failed += 1,
        }
    }
    // (b) one evaluator over all outputs of the group, union coordinates.
    let union = globals.iter().copied().collect::<Vec<_>>();
    let union_params = union
        .iter()
        .map(|&g| formal(g + 1_000_000).unwrap())
        .collect::<Vec<_>>();
    let mut group = Measure::default();
    let t0 = Instant::now();
    let group_result = if atoms.is_empty() {
        Err("no substitutable outputs".to_owned())
    } else {
        exact(&atoms, &union_params).and_then(|ev| taylor(&ev, &union, max_width, &mut group, t0))
    };
    // (c) per-output evaluators over the union parameters, merged.
    let mut merged = Measure::default();
    let t0 = Instant::now();
    let merged_result = (|| -> Result<(), String> {
        let mut acc: Option<ExpressionEvaluator<Complex<Rational>>> = None;
        for atom in &atoms {
            let ev = exact(std::slice::from_ref(atom), &union_params)?;
            match &mut acc {
                None => acc = Some(ev),
                Some(a) => a.merge(ev, Some(1))?,
            }
        }
        let acc = acc.ok_or("no substitutable outputs")?;
        taylor(&acc, &union, max_width, &mut merged, t0)
    })();
    // (d) symbolic derivatives per output over its own coordinates, then one joint evaluator.
    let mut symbolic = Measure::default();
    let symbolic_result = (|| -> Result<(), String> {
        if atoms.is_empty() {
            return Err("no substitutable outputs".into());
        }
        let mut values = Vec::new();
        let mut firsts = Vec::new();
        let mut seconds = Vec::new();
        for atom in &atoms {
            values.push(atom.clone());
            let used = used_globals(atom, &universe).into_iter().collect::<Vec<_>>();
            let symbols = used
                .iter()
                .map(|&g| {
                    let a = formal(g + 1_000_000).unwrap();
                    let symbolica::atom::AtomView::Var(v) = a.as_view() else { unreachable!() };
                    v.get_symbol()
                })
                .collect::<Vec<_>>();
            let mut d1 = Vec::new();
            for &x in &symbols {
                d1.push(atom.derivative(x));
            }
            for (i, di) in d1.iter().enumerate() {
                for &y in &symbols[i..] {
                    seconds.push(di.derivative(y));
                }
            }
            firsts.extend(d1);
        }
        let groups = [values.clone(), [values.clone(), firsts.clone()].concat(), [values, firsts, seconds].concat()];
        for (order, expressions) in groups.iter().enumerate() {
            let t0 = Instant::now();
            let ev = exact(expressions, &union_params)?;
            let export = ev.export_instructions();
            symbolic.ops[order] = total(ev.count_operations());
            symbolic.temps[order] = export.temporary_count;
            symbolic.instructions[order] = export.instructions.len();
            symbolic.build_ms[order] = t0.elapsed().as_secs_f64() * 1e3;
            symbolic.width[order] = expressions.len();
        }
        Ok(())
    })();
    let _ = writeln!(
        log,
        "GROUP {label}: occurrences={} substituted_outputs={} excluded_outputs={excluded} union_coords={} today_failed_compiles={today_failed} inlined_failed={inlined_failed} group={:?} merged={:?} symbolic={:?}",
        entries.len(),
        atoms.len(),
        union.len(),
        group_result.as_ref().err(),
        merged_result.as_ref().err(),
        symbolic_result.as_ref().err(),
    );
    for (name, m) in [("a_today_sub", &today), ("a_today_exc", &today_excluded), ("a_inlined", &inlined), ("b_group", &group), ("c_merge", &merged), ("d_symbolic", &symbolic)] {
        let _ = writeln!(
            log,
            "  {name:<10} ops V/1/2={:?} temps={:?} instr={:?} width={:?} skipped={:?} build_ms={:.1?}",
            m.ops, m.temps, m.instructions, m.width, m.skipped, m.build_ms
        );
    }
    (log, [today, today_excluded, inlined, group, merged, symbolic])
}
