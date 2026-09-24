// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use pounce_presolve::matching::{BipartiteMatching, hopcroft_karp};
use pounce_presolve::{
    BlockTriangularForm, DMPart, DulmageMendelsohnPartition, EqualityIncidence, ProbeView,
    SquareComponents,
};
fn incidence(nr: usize, nc: usize, mask: u32) -> EqualityIncidence {
    let mut vars = vec![];
    let mut ptr = vec![0];
    for r in 0..nr {
        for c in 0..nc {
            if mask & (1 << (r * nc + c)) != 0 {
                vars.push(c)
            }
        }
        ptr.push(vars.len());
    }
    EqualityIncidence {
        n_vars: nc,
        eq_row_inner_idx: (0..nr).collect(),
        adj_ptr: ptr,
        vars,
    }
}
fn enumerate(
    i: &EqualityIncidence,
    r: usize,
    rows: &mut Vec<Option<usize>>,
    used: &mut Vec<bool>,
    best: &mut usize,
    all: &mut Vec<Vec<Option<usize>>>,
) {
    if r == i.n_eq_rows() {
        let count = rows.iter().filter(|c| c.is_some()).count();
        if count > *best {
            *best = count;
            all.clear()
        }
        if count == *best {
            all.push(rows.clone())
        }
        return;
    }
    rows.push(None);
    enumerate(i, r + 1, rows, used, best, all);
    rows.pop();
    for &c in i.neighbors(r) {
        if !used[c] {
            used[c] = true;
            rows.push(Some(c));
            enumerate(i, r + 1, rows, used, best, all);
            rows.pop();
            used[c] = false
        }
    }
}
fn validate(i: &EqualityIncidence, m: &BipartiteMatching) {
    let dm = DulmageMendelsohnPartition::from_matching(i, m);
    let comps = SquareComponents::of_square_part(i, m, &dm);
    let mut seen_r = vec![false; i.n_eq_rows()];
    let mut seen_c = vec![false; i.n_vars];
    for comp in comps.components {
        assert_eq!(comp.eq_rows.len(), comp.cols.len());
        let btf = BlockTriangularForm::of_component(i, m, &comp);
        let mut owner = vec![None; i.n_vars];
        for (k, b) in btf.blocks.iter().enumerate() {
            assert_eq!(b.eq_rows.len(), b.cols.len());
            for &c in &b.cols {
                assert!(!seen_c[c]);
                seen_c[c] = true;
                owner[c] = Some(k)
            }
            for &r in &b.eq_rows {
                assert!(!seen_r[r]);
                seen_r[r] = true
            }
        }
        for (k, b) in btf.blocks.iter().enumerate() {
            for &r in &b.eq_rows {
                for &c in i.neighbors(r) {
                    if let Some(j) = owner[c] {
                        assert!(j <= k, "wrong order")
                    }
                }
            }
        }
    }
    for (r, seen) in seen_r.iter().enumerate() {
        assert_eq!(*seen, dm.row_part[r] == DMPart::Square)
    }
    for (c, seen) in seen_c.iter().enumerate() {
        assert_eq!(*seen, dm.col_part[c] == DMPart::Square)
    }
}
#[test]
fn exhaustive_matching_dm_and_btf_against_all_maximum_witnesses() {
    let mut graphs = 0;
    let mut matchings = 0;
    for nr in 0..=4 {
        for nc in 0..=4 {
            for mask in 0..1u32 << (nr * nc) {
                let inc = incidence(nr, nc, mask);
                let m = hopcroft_karp(&inc);
                let mut best = 0;
                let mut all = vec![];
                enumerate(
                    &inc,
                    0,
                    &mut vec![],
                    &mut vec![false; nc],
                    &mut best,
                    &mut all,
                );
                assert_eq!(m.size, best);
                validate(&inc, &m);
                let d = DulmageMendelsohnPartition::from_matching(&inc, &m);
                let mut over_r = vec![false; nr];
                let mut under_c = vec![false; nc];
                for rows in all {
                    let mut cols = vec![None; nc];
                    for (r, c) in rows.iter().enumerate() {
                        if let Some(c) = c {
                            cols[*c] = Some(r)
                        } else {
                            over_r[r] = true
                        }
                    }
                    for (c, row) in cols.iter().enumerate() {
                        if row.is_none() {
                            under_c[c] = true
                        }
                    }
                    let alt = BipartiteMatching {
                        row_to_var: rows,
                        var_to_row: cols,
                        size: best,
                    };
                    let ad = DulmageMendelsohnPartition::from_matching(&inc, &alt);
                    assert_eq!(d.row_part, ad.row_part);
                    assert_eq!(d.col_part, ad.col_part);
                    validate(&inc, &alt);
                    matchings += 1;
                }
                let mut under_r = vec![false; nr];
                let mut over_c = vec![false; nc];
                for r in 0..nr {
                    for &c in inc.neighbors(r) {
                        if over_r[r] {
                            over_c[c] = true
                        }
                        if under_c[c] {
                            under_r[r] = true
                        }
                    }
                }
                for r in 0..nr {
                    assert_eq!(
                        d.row_part[r],
                        if over_r[r] {
                            DMPart::Over
                        } else if under_r[r] {
                            DMPart::Under
                        } else {
                            DMPart::Square
                        }
                    )
                }
                for c in 0..nc {
                    assert_eq!(
                        d.col_part[c],
                        if over_c[c] {
                            DMPart::Over
                        } else if under_c[c] {
                            DMPart::Under
                        } else {
                            DMPart::Square
                        }
                    )
                }
                graphs += 1;
            }
        }
    }
    let mut p = ProbeView {
        n_vars: 1,
        m_rows: 1,
        jac_irow: &[0, 3],
        jac_jcol: &[0, 0],
        jac_values: None,
        g_l: &[0.],
        g_u: &[0.],
        linearity: None,
        one_based: false,
        eq_tol: 0.,
        excluded_vars: None,
        excluded_rows: None,
    };
    assert_eq!(EqualityIncidence::from_probe(&p).neighbors(0), &[0]);
    p.jac_values = Some(&[0., 1.]);
    assert!(EqualityIncidence::from_probe(&p).neighbors(0).is_empty());
    assert_eq!(graphs, 74_963);
    assert_eq!(matchings, 259_537);
}

#[test]
fn qualified_long_augmenting_path_stack() {
    std::thread::Builder::new()
        .stack_size(super::MATCHING_STACK)
        .spawn(|| {
            let n = super::MATCHING_ROWS;
            let mut inc = EqualityIncidence {
                n_vars: n,
                eq_row_inner_idx: (0..n).collect(),
                adj_ptr: vec![0],
                vars: vec![],
            };
            for r in 0..n {
                if r == n - 1 {
                    inc.vars.push(0)
                } else {
                    inc.vars.extend([r, r + 1])
                }
                inc.adj_ptr.push(inc.vars.len());
            }
            assert_eq!(hopcroft_karp(&inc).size, n);
        })
        .unwrap()
        .join()
        .unwrap();
}
fn id(n: u8) -> pse_ids::SemanticId {
    pse_ids::SemanticId::from_bytes([n; 16])
}
fn semantic(scope: crate::projection::Scope) -> super::CaseIncidence {
    use super::{CaseIncidence, Constraint, Incidence};
    CaseIncidence::new(
        scope,
        vec![
            Constraint {
                id: id(1),
                lower: Some(0.),
                upper: Some(0.),
            },
            Constraint {
                id: id(2),
                lower: Some(0.),
                upper: Some(2.),
            },
            Constraint {
                id: id(3),
                lower: Some(0.),
                upper: Some(0.),
            },
        ],
        vec![id(11), id(12)],
        vec![
            Incidence {
                row: id(1),
                column: id(11),
                instance: id(20),
                output: 0,
            },
            Incidence {
                row: id(1),
                column: id(11),
                instance: id(21),
                output: 1,
            },
            Incidence {
                row: id(2),
                column: id(11),
                instance: id(22),
                output: 0,
            },
        ],
        [id(11)].into_iter().collect(),
        crate::projection::GraphLimits {
            nodes: 10,
            edges: 10,
        },
    )
    .unwrap()
}
#[test]
fn complete_inventory_isolates_coupling_partial_and_provenance() {
    use crate::projection::{ProjectionError, Scope};
    let c = semantic(Scope::Whole(id(99)));
    let a = c
        .analyze(&std::sync::atomic::AtomicBool::new(false))
        .unwrap();
    assert_eq!(a.matching, vec![(id(1), id(11))]);
    assert_eq!(a.over.rows, vec![id(3)]);
    assert_eq!(a.under.columns, vec![id(12)]);
    assert_eq!(a.contributions.len(), 3);
    assert_eq!(
        a.blocks[0].coupling,
        pounce_presolve::coupling::AuxiliaryCouplingClass::ObjectiveAndInequalityCoupled
    );
    assert!(matches!(
        semantic(Scope::Partial(id(99))).analyze(&std::sync::atomic::AtomicBool::new(false)),
        Err(ProjectionError::Partial)
    ));
    assert!(c.independent([id(1)].into(), [id(11)].into()).is_err());
    assert!(
        c.independent([id(1), id(2)].into(), [id(11)].into())
            .unwrap()
            .analyze(&std::sync::atomic::AtomicBool::new(false))
            .is_ok()
    );
    let mut bad = c.clone();
    bad.rows[0].upper = Some(f64::INFINITY);
    assert!(
        super::CaseIncidence::new(
            Scope::Whole(id(99)),
            bad.rows,
            bad.columns,
            bad.edges,
            bad.objective,
            crate::projection::GraphLimits {
                nodes: 10,
                edges: 10
            }
        )
        .is_err()
    );
    let mut bad = c.clone();
    bad.edges[0].column = id(55);
    assert!(
        super::CaseIncidence::new(
            Scope::Whole(id(99)),
            bad.rows,
            bad.columns,
            bad.edges,
            bad.objective,
            crate::projection::GraphLimits {
                nodes: 10,
                edges: 10
            }
        )
        .is_err()
    );
}
#[test]
fn empty_rectangular_and_structural_not_numerical_rank() {
    use super::{CaseIncidence, Constraint, Incidence};
    use crate::projection::{GraphLimits, Scope};
    let empty = CaseIncidence::new(
        Scope::Whole(id(99)),
        vec![],
        vec![id(10)],
        vec![],
        Default::default(),
        GraphLimits {
            nodes: 10,
            edges: 10,
        },
    )
    .unwrap()
    .analyze(&std::sync::atomic::AtomicBool::new(false))
    .unwrap();
    assert_eq!(empty.under.columns, vec![id(10)]);
    // The numerical matrix [[1,1],[1,1]] is singular but its complete support has a perfect matching.
    let rows = vec![
        Constraint {
            id: id(1),
            lower: Some(0.),
            upper: Some(0.),
        },
        Constraint {
            id: id(2),
            lower: Some(0.),
            upper: Some(0.),
        },
    ];
    let edges = [1, 2]
        .into_iter()
        .flat_map(|r| {
            [10, 11].into_iter().map(move |c| Incidence {
                row: id(r),
                column: id(c),
                instance: id(20),
                output: 0,
            })
        })
        .collect();
    let a = CaseIncidence::new(
        Scope::Whole(id(99)),
        rows,
        vec![id(10), id(11)],
        edges,
        Default::default(),
        GraphLimits {
            nodes: 10,
            edges: 10,
        },
    )
    .unwrap()
    .analyze(&std::sync::atomic::AtomicBool::new(false))
    .unwrap();
    assert_eq!(a.matching.len(), 2);
    assert_eq!(a.blocks.len(), 1);
}
