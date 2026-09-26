// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Class-aware admission of conservative original-equation support.
use crate::{OracleContract, ProblemError};
use pse_structural::{
    incidence::{CaseIncidence, Constraint, Incidence, StructuralAnalysis},
    projection::{GraphLimits, Scope},
};

/// A square equation closure or an NLP that may retain optimization degrees of freedom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Every original row and free variable must participate in the matching.
    Roots,
    /// Every equality must match; inequalities and optimization freedom remain admissible.
    Nlp,
}

/// Consume the compiler's existing library-owned analysis without repeating matching.
pub fn admit(analysis: &StructuralAnalysis, mode: Mode) -> Result<(), ProblemError> {
    if matches!(analysis.scope, Scope::Partial(_)) {
        return Err(ProblemError::Contract(
            "structural admission requires complete scope".into(),
        ));
    }
    let rows = analysis.over.rows.clone();
    let columns = if mode == Mode::Roots {
        analysis.under.columns.clone()
    } else {
        vec![]
    };
    if !rows.is_empty() || !columns.is_empty() {
        return Err(ProblemError::Structural {
            mode,
            rows,
            columns,
        });
    }
    Ok(())
}

/// Reuse a checked matching witness only against the complete current equation
/// inventory and support. This is linear witness validation, not another matching
/// search. Opaque callback oracles retain the full low-level analysis below.
pub fn check(
    contract: &OracleContract,
    pattern: faer::sparse::SymbolicSparseColMatRef<'_, usize>,
    bounds: &[(f64, f64)],
    mode: Mode,
    analysis: Option<&StructuralAnalysis>,
) -> Result<(), ProblemError> {
    let Some(analysis) = analysis else {
        return oracle(contract, pattern, bounds, mode);
    };
    admit(analysis, mode)?;
    let invalid = || {
        ProblemError::Contract("compiler matching does not establish current oracle support".into())
    };
    if bounds.len() != contract.rows.len()
        || pattern.nrows() != contract.rows.len()
        || pattern.ncols() != contract.variables.len()
    {
        return Err(invalid());
    }
    let rows = contract
        .rows
        .iter()
        .enumerate()
        .map(|(i, id)| (*id, i))
        .collect::<std::collections::BTreeMap<_, _>>();
    let columns = contract
        .variables
        .iter()
        .enumerate()
        .map(|(i, v)| (v.id, i))
        .collect::<std::collections::BTreeMap<_, _>>();
    let required = bounds
        .iter()
        .enumerate()
        .filter_map(|(i, (l, u))| (l.is_finite() && l == u).then_some(i))
        .collect::<std::collections::BTreeSet<_>>();
    let mut matched = std::collections::BTreeSet::new();
    let mut used = std::collections::BTreeSet::new();
    for (row, column) in &analysis.matching {
        let r = *rows.get(row).ok_or_else(invalid)?;
        let c = *columns.get(column).ok_or_else(invalid)?;
        if !required.contains(&r)
            || !matched.insert(r)
            || !used.insert(c)
            || pattern.row_idx()[pattern.col_range(c)]
                .binary_search(&r)
                .is_err()
        {
            return Err(invalid());
        }
    }
    if matched != required
        || (mode == Mode::Roots
            && (required.len() != contract.rows.len() || used.len() != contract.variables.len()))
    {
        return Err(invalid());
    }
    Ok(())
}

/// Low-level callers undergo the same analysis using their original residual Jacobian.
pub fn oracle(
    contract: &OracleContract,
    pattern: faer::sparse::SymbolicSparseColMatRef<'_, usize>,
    bounds: &[(f64, f64)],
    mode: Mode,
) -> Result<(), ProblemError> {
    let n = contract.variables.len();
    let m = contract.rows.len();
    if pattern.nrows() != m
        || pattern.ncols() != n
        || bounds.len() != m
        || (mode == Mode::Roots && (m != n || bounds.iter().any(|(l, u)| l != u || !l.is_finite())))
    {
        return Err(ProblemError::Contract(
            "original structural dimensions/equality contract".into(),
        ));
    }
    let rows = contract
        .rows
        .iter()
        .zip(bounds)
        .map(|(&id, &(lower, upper))| Constraint {
            id,
            lower: (lower != f64::NEG_INFINITY).then_some(lower),
            upper: (upper != f64::INFINITY).then_some(upper),
        })
        .collect();
    let mut edges = Vec::new();
    for c in 0..n {
        for r in pattern.row_idx_of_col(c) {
            edges.push(Incidence {
                row: contract.rows[r],
                column: contract.variables[c].id,
                instance: contract.rows[r],
                output: 0,
            });
        }
    }
    let limits = GraphLimits {
        nodes: n.saturating_add(m),
        edges: edges.len(),
    };
    let incidence = CaseIncidence::new(
        Scope::Whole(pse_ids::derive_id(
            "pse.native.structural-scope.v1",
            &[contract.identity.as_bytes()],
        )),
        rows,
        contract.variables.iter().map(|v| v.id).collect(),
        edges,
        Default::default(),
        limits,
    )
    .map_err(|e| ProblemError::Contract(e.to_string()))?;
    let analysis = incidence
        .analyze(&std::sync::atomic::AtomicBool::new(false))
        .map_err(|e| ProblemError::Contract(e.to_string()))?;
    admit(&analysis, mode)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn id(n: u8) -> pse_ids::SemanticId {
        pse_ids::SemanticId::from_bytes([n; 16])
    }
    #[test]
    fn roots_and_nlp_admit_original_matching_by_class() {
        let c = OracleContract {
            identity: pse_ids::ContentHash::from_bytes([1; 32]),
            variables: vec![
                crate::Variable {
                    id: id(1),
                    lower: -f64::INFINITY,
                    upper: f64::INFINITY,
                },
                crate::Variable {
                    id: id(2),
                    lower: -f64::INFINITY,
                    upper: f64::INFINITY,
                },
            ],
            rows: vec![id(3), id(4)],
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness: pse_kernels::DerivativeOrder::First,
        };
        let pattern = |pairs: &[(usize, usize)]| {
            faer::sparse::SymbolicSparseColMat::try_new_from_indices(
                2,
                2,
                &pairs
                    .iter()
                    .map(|&(r, c)| faer::sparse::Pair::new(r, c))
                    .collect::<Vec<_>>(),
            )
            .unwrap()
            .0
        };
        let good = pattern(&[(0, 0), (1, 1)]);
        oracle(&c, good.as_ref(), &[(0., 0.); 2], Mode::Roots).unwrap();
        let bad = pattern(&[(0, 0), (1, 0)]);
        let Err(ProblemError::Structural { rows, columns, .. }) =
            oracle(&c, bad.as_ref(), &[(0., 0.); 2], Mode::Roots)
        else {
            panic!("missing named deficiency")
        };
        assert!(rows.contains(&id(3)) && rows.contains(&id(4)));
        assert_eq!(columns, vec![id(2)]);
        assert!(oracle(&c, bad.as_ref(), &[(0., 0.); 2], Mode::Nlp).is_err());
        // Inequality rows do not erase valid optimization freedom or inherit square admission.
        oracle(&c, bad.as_ref(), &[(0., 0.), (-1., 1.)], Mode::Nlp).unwrap();
        assert!(oracle(&c, bad.as_ref(), &[(0., 0.), (-1., 1.)], Mode::Roots).is_err());
    }
    #[test]
    fn matching_witness_reuse_checks_edges_membership_and_equality_class() {
        let contract = OracleContract {
            identity: pse_ids::ContentHash::from_bytes([1; 32]),
            variables: vec![crate::Variable {
                id: id(1),
                lower: -1.0,
                upper: 1.0,
            }],
            rows: vec![id(2)],
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness: pse_kernels::DerivativeOrder::First,
        };
        let graph = CaseIncidence::new(
            Scope::Whole(id(3)),
            vec![Constraint {
                id: id(2),
                lower: Some(0.0),
                upper: Some(0.0),
            }],
            vec![id(1)],
            vec![Incidence {
                row: id(2),
                column: id(1),
                instance: id(4),
                output: 0,
            }],
            Default::default(),
            GraphLimits { nodes: 2, edges: 1 },
        )
        .unwrap();
        let analysis = graph
            .analyze(&std::sync::atomic::AtomicBool::new(false))
            .unwrap();
        let pattern = pse_math::sparse::AssemblyMatrix::new(1, 1, &[(0, 0)], 10).unwrap();
        // Repeated validation consumes the same witness, without invoking analyze.
        for _ in 0..1000 {
            check(
                &contract,
                pattern.matrix().symbolic(),
                &[(0.0, 0.0)],
                Mode::Roots,
                Some(&analysis),
            )
            .unwrap();
        }
        let empty = pse_math::sparse::AssemblyMatrix::new(1, 1, &[], 10).unwrap();
        assert!(
            check(
                &contract,
                empty.matrix().symbolic(),
                &[(0.0, 0.0)],
                Mode::Roots,
                Some(&analysis)
            )
            .is_err()
        );
        assert!(
            check(
                &contract,
                pattern.matrix().symbolic(),
                &[(-1.0, 1.0)],
                Mode::Nlp,
                Some(&analysis)
            )
            .is_err()
        );
        let mut stale = contract.clone();
        stale.variables[0].id = id(9);
        assert!(
            check(
                &stale,
                pattern.matrix().symbolic(),
                &[(0.0, 0.0)],
                Mode::Roots,
                Some(&analysis)
            )
            .is_err()
        );
        let mut incomplete = analysis.clone();
        incomplete.matching.clear();
        assert!(
            check(
                &contract,
                pattern.matrix().symbolic(),
                &[(0.0, 0.0)],
                Mode::Roots,
                Some(&incomplete)
            )
            .is_err()
        );
    }
}
