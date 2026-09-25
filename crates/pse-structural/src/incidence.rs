// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Checked semantic boundary around library-owned matching, DM and BTF.
use crate::projection::{GraphLimits, ProjectionError, Scope};
use pounce_presolve::{
    BlockTriangularForm, DMPart, DulmageMendelsohnPartition, EqualityIncidence,
    InequalityIncidence, ProbeView, SquareComponents, coupling::classify_block,
    matching::hopcroft_karp,
};
use pse_ids::SemanticId;
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    sync::atomic::{AtomicBool, Ordering},
};

/// Qualified bound for pounce's recursive augmenting path search.
pub const MATCHING_ROWS: usize = 100_000;
/// Stack reserved through matching thread exit.
pub const MATCHING_STACK: usize = 32 * 1024 * 1024;
/// Algorithm and support interpretation, distinct from numerical rank.
pub const PROVENANCE: &str = "pounce-presolve/0.12.0; HK/DM/BTF; conservative-all-branch-support; exact-finite-equalities; v1";
/// One selected constraint, including rows with no incidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Constraint {
    /// Semantic row.
    pub id: SemanticId,
    /// Missing lower bound is negative infinity.
    pub lower: Option<f64>,
    /// Missing upper bound is positive infinity.
    pub upper: Option<f64>,
}
/// Original contribution provenance, preserved when matching deduplicates pairs.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Incidence {
    /// Semantic row.
    pub row: SemanticId,
    /// Semantic free variable.
    pub column: SemanticId,
    /// Contributing instance.
    pub instance: SemanticId,
    /// Local output ordinal.
    pub output: usize,
}
/// Admitted complete selected-case incidence, not a claim about every model case.
#[derive(Clone, Debug, PartialEq)]
pub struct CaseIncidence {
    scope: Scope,
    rows: Vec<Constraint>,
    columns: Vec<SemanticId>,
    edges: Vec<Incidence>,
    objective: BTreeSet<SemanticId>,
}
/// Semantic partition members.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Part {
    /// Equality rows.
    pub rows: Vec<SemanticId>,
    /// Free variables.
    pub columns: Vec<SemanticId>,
}
/// One equality BTF block; coupling does not establish independent eliminability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    /// Semantic members.
    pub members: Part,
    /// Conservative external objective/inequality coupling.
    pub coupling: pounce_presolve::coupling::AuxiliaryCouplingClass,
}
/// Complete conservative structural result. No numerical rank claim is made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructuralAnalysis {
    /// Complete admitted selected scope.
    pub scope: Scope,
    /// Library interpretation.
    pub provenance: &'static str,
    /// One maximum matching witness, not a canonical unique mathematical matching.
    pub matching: Vec<(SemanticId, SemanticId)>,
    /// Overdetermined region.
    pub over: Part,
    /// Underdetermined region.
    pub under: Part,
    /// Square region.
    pub square: Part,
    /// Prerequisite-first blocks with semantic tie breaking.
    pub blocks: Vec<Block>,
    /// Original contribution incidence, including duplicate semantic pairs.
    pub contributions: Vec<Incidence>,
}
impl CaseIncidence {
    /// Validate every dimension/reference before the library can silently omit an entry.
    pub fn new(
        scope: Scope,
        mut rows: Vec<Constraint>,
        mut columns: Vec<SemanticId>,
        mut edges: Vec<Incidence>,
        objective: BTreeSet<SemanticId>,
        limits: GraphLimits,
    ) -> Result<Self, ProjectionError> {
        if matches!(scope, Scope::Independent { .. }) {
            return Err(ProjectionError::Partial);
        }
        limits.check(
            rows.len()
                .checked_add(columns.len())
                .ok_or(ProjectionError::Limit)?,
            edges.len(),
        )?;
        if rows.len() > MATCHING_ROWS || columns.len() > i32::MAX as usize {
            return Err(ProjectionError::Limit);
        }
        rows.sort_by_key(|r| r.id);
        columns.sort();
        edges.sort();
        if rows.windows(2).any(|r| r[0].id == r[1].id) || columns.windows(2).any(|c| c[0] == c[1]) {
            return Err(ProjectionError::Invalid(
                "duplicate incidence inventory".into(),
            ));
        }
        if rows.iter().any(|r| {
            r.lower.is_some_and(|x| !x.is_finite())
                || r.upper.is_some_and(|x| !x.is_finite())
                || r.lower.zip(r.upper).is_some_and(|(l, u)| l > u)
        }) {
            return Err(ProjectionError::Invalid("invalid row bounds".into()));
        }
        let row_ids: BTreeSet<_> = rows.iter().map(|r| r.id).collect();
        for e in &edges {
            if !row_ids.contains(&e.row) {
                return Err(ProjectionError::Missing(e.row));
            }
            if columns.binary_search(&e.column).is_err() {
                return Err(ProjectionError::Missing(e.column));
            }
        }
        for &c in &objective {
            if columns.binary_search(&c).is_err() {
                return Err(ProjectionError::Missing(c));
            }
        }
        Ok(Self {
            scope,
            rows,
            columns,
            edges,
            objective,
        })
    }
    /// Extract an independent region only from a complete parent after all crossing checks.
    pub fn independent(
        &self,
        rows: BTreeSet<SemanticId>,
        columns: BTreeSet<SemanticId>,
    ) -> Result<Self, ProjectionError> {
        let model = match self.scope {
            Scope::Whole(id) | Scope::Independent { model: id, .. } => id,
            Scope::Partial(_) => return Err(ProjectionError::Partial),
        };
        if !rows.iter().all(|r| self.rows.iter().any(|x| x.id == *r))
            || !columns.iter().all(|c| self.columns.contains(c))
        {
            return Err(ProjectionError::Invalid(
                "unknown independent members".into(),
            ));
        }
        for e in &self.edges {
            if rows.contains(&e.row) != columns.contains(&e.column) {
                return Err(ProjectionError::Crossing(e.instance));
            }
        }
        // A shared scalar objective couples all of its support, even across equality components.
        if !self.objective.is_disjoint(&columns) && !self.objective.is_subset(&columns) {
            return Err(ProjectionError::Invalid(
                "crossing objective support".into(),
            ));
        }
        Ok(Self {
            scope: Scope::Independent {
                model,
                members: rows.union(&columns).copied().collect(),
            },
            rows: self
                .rows
                .iter()
                .filter(|r| rows.contains(&r.id))
                .cloned()
                .collect(),
            columns: self
                .columns
                .iter()
                .filter(|c| columns.contains(c))
                .copied()
                .collect(),
            edges: self
                .edges
                .iter()
                .filter(|e| rows.contains(&e.row))
                .cloned()
                .collect(),
            objective: self.objective.intersection(&columns).copied().collect(),
        })
    }
    /// Execute on the qualified stack; cancellation is checked between library stages.
    pub fn analyze(&self, cancel: &AtomicBool) -> Result<StructuralAnalysis, ProjectionError> {
        if matches!(self.scope, Scope::Partial(_)) {
            return Err(ProjectionError::Partial);
        }
        std::thread::scope(|scope| {
            std::thread::Builder::new()
                .name("pse-matching".into())
                .stack_size(MATCHING_STACK)
                .spawn_scoped(scope, || self.analyze_inner(cancel))
                .map_err(|e| ProjectionError::Invalid(e.to_string()))?
                .join()
                .map_err(|_| ProjectionError::Invalid("structural worker panic".into()))?
        })
    }
    fn analyze_inner(&self, cancel: &AtomicBool) -> Result<StructuralAnalysis, ProjectionError> {
        let checkpoint = || {
            if cancel.load(Ordering::Relaxed) {
                Err(ProjectionError::Cancelled)
            } else {
                Ok(())
            }
        };
        checkpoint()?;
        let rows: BTreeMap<_, _> = self
            .rows
            .iter()
            .enumerate()
            .map(|(i, r)| (r.id, i))
            .collect();
        let cols: BTreeMap<_, _> = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, &c)| (c, i))
            .collect();
        let pairs: BTreeSet<_> = self
            .edges
            .iter()
            .map(|e| (rows[&e.row], cols[&e.column]))
            .collect();
        let ri: Vec<_> = pairs.iter().map(|&(r, _)| r as i32).collect();
        let ci: Vec<_> = pairs.iter().map(|&(_, c)| c as i32).collect();
        let lower: Vec<_> = self
            .rows
            .iter()
            .map(|r| r.lower.unwrap_or(f64::NEG_INFINITY))
            .collect();
        let upper: Vec<_> = self
            .rows
            .iter()
            .map(|r| r.upper.unwrap_or(f64::INFINITY))
            .collect();
        let probe = ProbeView {
            n_vars: cols.len(),
            m_rows: rows.len(),
            jac_irow: &ri,
            jac_jcol: &ci,
            jac_values: None,
            g_l: &lower,
            g_u: &upper,
            linearity: None,
            one_based: false,
            eq_tol: 0.0,
            excluded_vars: None,
            excluded_rows: None,
        };
        let inc = EqualityIncidence::from_probe(&probe);
        let expected: Vec<_> = self
            .rows
            .iter()
            .enumerate()
            .filter_map(|(i, r)| {
                r.lower
                    .zip(r.upper)
                    .is_some_and(|(l, u)| l == u)
                    .then_some(i)
            })
            .collect();
        if inc.eq_row_inner_idx != expected {
            return Err(ProjectionError::Invalid("library equality coverage".into()));
        }
        let m = hopcroft_karp(&inc);
        checkpoint()?;
        let invalid = || ProjectionError::Invalid("library structural result invariant".into());
        if m.row_to_var.len() != inc.n_eq_rows()
            || m.var_to_row.len() != inc.n_vars
            || m.row_to_var.iter().flatten().count() != m.size
            || m.var_to_row.iter().flatten().count() != m.size
        {
            return Err(invalid());
        }
        for (r, c) in m.row_to_var.iter().enumerate() {
            if let Some(c) = c
                && (*c >= inc.n_vars
                    || m.var_to_row[*c] != Some(r)
                    || !inc.neighbors(r).contains(c))
            {
                return Err(invalid());
            }
        }
        let dm = DulmageMendelsohnPartition::from_matching(&inc, &m);
        checkpoint()?;
        if dm.row_part.len() != inc.n_eq_rows() || dm.col_part.len() != inc.n_vars {
            return Err(invalid());
        }
        let validate_partition = |parts: &[DMPart], lists: [&[usize]; 3]| -> bool {
            let mut seen = vec![false; parts.len()];
            for (indices, part) in
                lists
                    .into_iter()
                    .zip([DMPart::Over, DMPart::Square, DMPart::Under])
            {
                for &i in indices {
                    if i >= seen.len() || seen[i] || parts[i] != part {
                        return false;
                    }
                    seen[i] = true;
                }
            }
            seen.into_iter().all(|x| x)
        };
        if !validate_partition(
            &dm.row_part,
            [&dm.over_rows, &dm.square_rows, &dm.under_rows],
        ) || !validate_partition(
            &dm.col_part,
            [&dm.over_cols, &dm.square_cols, &dm.under_cols],
        ) {
            return Err(invalid());
        }
        let components = SquareComponents::of_square_part(&inc, &m, &dm);
        let mut blocks = vec![];
        let mut seen_r = BTreeSet::new();
        let mut seen_c = BTreeSet::new();
        for comp in components.components {
            let component_rows: BTreeSet<_> = comp.eq_rows.iter().copied().collect();
            let component_cols: BTreeSet<_> = comp.cols.iter().copied().collect();
            if comp.eq_rows.len() != comp.cols.len()
                || comp.eq_rows.iter().any(|&r| {
                    r >= inc.n_eq_rows() || dm.row_part[r] != DMPart::Square || !seen_r.insert(r)
                })
                || comp.cols.iter().any(|&c| {
                    c >= inc.n_vars || dm.col_part[c] != DMPart::Square || !seen_c.insert(c)
                })
            {
                return Err(invalid());
            }
            if comp
                .eq_rows
                .iter()
                .any(|&r| m.row_to_var[r].is_none_or(|c| !component_cols.contains(&c)))
            {
                return Err(invalid());
            }
            let btf = BlockTriangularForm::of_component(&inc, &m, &comp);
            let mut br = BTreeSet::new();
            let mut bc = BTreeSet::new();
            let mut owners = BTreeMap::new();
            for (i, b) in btf.blocks.iter().enumerate() {
                if b.eq_rows.len() != b.cols.len()
                    || b.eq_rows
                        .iter()
                        .any(|r| !component_rows.contains(r) || !br.insert(*r))
                    || b.cols
                        .iter()
                        .any(|c| !component_cols.contains(c) || !bc.insert(*c))
                {
                    return Err(invalid());
                }
                for &c in &b.cols {
                    owners.insert(c, i);
                }
            }
            if br.len() != comp.eq_rows.len() || bc.len() != comp.cols.len() {
                return Err(invalid());
            }
            for (i, b) in btf.blocks.iter().enumerate() {
                for &r in &b.eq_rows {
                    if inc
                        .neighbors(r)
                        .iter()
                        .any(|c| owners.get(c).is_some_and(|&j| j > i))
                    {
                        return Err(invalid());
                    }
                }
            }
            blocks.extend(btf.blocks);
            checkpoint()?;
        }
        if seen_r != dm.square_rows.iter().copied().collect()
            || seen_c != dm.square_cols.iter().copied().collect()
        {
            return Err(invalid());
        }
        // Library topological ordering with semantic tie breaks across components/blocks.
        let mut owner = BTreeMap::new();
        let mut graph = petgraph::graph::DiGraph::<usize, ()>::new();
        let nodes: Vec<_> = (0..blocks.len()).map(|i| graph.add_node(i)).collect();
        for (i, b) in blocks.iter().enumerate() {
            for &c in &b.cols {
                owner.insert(c, i);
            }
        }
        for (i, b) in blocks.iter().enumerate() {
            for &r in &b.eq_rows {
                for c in inc.neighbors(r) {
                    if let Some(&j) = owner.get(c)
                        && i != j
                    {
                        graph.add_edge(nodes[j], nodes[i], ());
                    }
                }
            }
        }
        let order = rustworkx_core::dag_algo::lexicographical_topological_sort(
            &graph,
            |i| {
                Ok::<_, std::convert::Infallible>(
                    blocks[graph[i]]
                        .eq_rows
                        .iter()
                        .map(|&r| self.rows[inc.eq_row_inner_idx[r]].id)
                        .min(),
                )
            },
            false,
            None,
        )
        .map_err(|_| invalid())?;
        if order.len() != blocks.len() {
            return Err(invalid());
        }
        let part = |rs: &[usize], cs: &[usize]| {
            let mut rows: Vec<_> = rs
                .iter()
                .map(|&r| self.rows[inc.eq_row_inner_idx[r]].id)
                .collect();
            let mut columns: Vec<_> = cs.iter().map(|&c| self.columns[c]).collect();
            rows.sort();
            columns.sort();
            Part { rows, columns }
        };
        let ineq = InequalityIncidence::from_probe(&probe);
        let objective: HashSet<_> = self.objective.iter().map(|c| cols[c]).collect();
        Ok(StructuralAnalysis {
            scope: self.scope.clone(),
            provenance: PROVENANCE,
            matching: m
                .row_to_var
                .iter()
                .enumerate()
                .filter_map(|(r, c)| {
                    c.map(|c| (self.rows[inc.eq_row_inner_idx[r]].id, self.columns[c]))
                })
                .collect(),
            over: part(&dm.over_rows, &dm.over_cols),
            under: part(&dm.under_rows, &dm.under_cols),
            square: part(&dm.square_rows, &dm.square_cols),
            blocks: order
                .iter()
                .map(|node| {
                    let b = &blocks[graph[*node]];
                    Block {
                        members: part(&b.eq_rows, &b.cols),
                        coupling: classify_block(b, &ineq, &objective),
                    }
                })
                .collect(),
            contributions: self.edges.clone(),
        })
    }
}

#[cfg(test)]
#[path = "incidence_tests.rs"]
mod tests;
