// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Checked dense matching and BTF on the caller's already qualified stack.
use crate::incidence::MATCHING_ROWS;
use crate::projection::ProjectionError;
use pounce_presolve::{
    BlockTriangularBlock, BlockTriangularForm, DMPart, DulmageMendelsohnPartition,
    EqualityIncidence, SquareComponents,
    matching::{BipartiteMatching, hopcroft_karp},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::{AtomicBool, Ordering},
};

/// Library result whose matching, partitions, coverage and prerequisite order are checked.
#[derive(Debug)]
pub struct DenseDecomposition {
    /// Maximum matching in dense equality/variable coordinates.
    pub matching: BipartiteMatching,
    /// Checked complete DM partition.
    pub partition: DulmageMendelsohnPartition,
    /// Square blocks in prerequisite-first order, independent of semantic ordering.
    pub blocks: Vec<BlockTriangularBlock>,
    /// Distinct (prerequisite block, consuming block) edges.
    pub dependencies: Vec<(usize, usize)>,
}

/// Source-derived peak for pounce-presolve 0.12.0 HK/DM/components/BTF and checks.
/// Excludes caller-owned input and the caller's separately admitted matching stack.
/// Vertex terms cover matching/layers, DM marks/queues, component/SCC inventories,
/// block maps/sets and vector-capacity overlap. Edge terms cover CSR transpose,
/// connected/SCC adjacency and returned dependency sorting, including old/new growth.
/// Nested semantic payload remains with the consumer. Overflow refuses before work.
pub fn allocation_bound(
    rows: usize,
    columns: usize,
    edges: usize,
) -> Result<usize, ProjectionError> {
    if rows > MATCHING_ROWS || columns > i32::MAX as usize {
        return Err(ProjectionError::Limit);
    }
    let nodes = rows.checked_add(columns).ok_or(ProjectionError::Limit)?;
    let trees = pse_model::allocation::btree_allocation_bound::<usize, usize>(nodes)
        .and_then(|n| n.checked_mul(8))
        .ok_or(ProjectionError::Limit)?;
    nodes
        .checked_mul(128 * size_of::<usize>())
        .and_then(|n| {
            edges
                .checked_mul(64 * size_of::<usize>())
                .and_then(|e| n.checked_add(e))
        })
        .and_then(|n| n.checked_add(trees))
        .and_then(|n| n.checked_add(size_of::<DenseDecomposition>()))
        .ok_or(ProjectionError::Limit)
}

/// Validate and decompose on the current lane; this function never spawns a thread.
/// The caller supplies prepayment and a stack qualified for MATCHING_ROWS.
/// # Errors
/// Cancellation, cardinality refusal or malformed input/library output.
pub fn decompose(
    inc: &EqualityIncidence,
    cancel: &AtomicBool,
) -> Result<DenseDecomposition, ProjectionError> {
    let checkpoint = || {
        if cancel.load(Ordering::Acquire) {
            Err(ProjectionError::Cancelled)
        } else {
            Ok(())
        }
    };
    checkpoint()?;
    validate_input(inc)?;
    let m = hopcroft_karp(inc);
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
            && (*c >= inc.n_vars || m.var_to_row[*c] != Some(r) || !inc.neighbors(r).contains(c))
        {
            return Err(invalid());
        }
    }
    let dm = DulmageMendelsohnPartition::from_matching(inc, &m);
    checkpoint()?;
    if dm.row_part.len() != inc.n_eq_rows() || dm.col_part.len() != inc.n_vars {
        return Err(invalid());
    }
    let validate_partition = |parts: &[DMPart], lists: [&[usize]; 3]| -> bool {
        let mut seen = vec![false; parts.len()];
        for (indices, part) in lists
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
    let components = SquareComponents::of_square_part(inc, &m, &dm);
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
            || comp
                .cols
                .iter()
                .any(|&c| c >= inc.n_vars || dm.col_part[c] != DMPart::Square || !seen_c.insert(c))
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
        let btf = BlockTriangularForm::of_component(inc, &m, &comp);
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
    let dependencies = block_dependencies(inc, &m, &blocks)?;
    checkpoint()?;
    Ok(DenseDecomposition {
        matching: m,
        partition: dm,
        blocks,
        dependencies,
    })
}

fn invalid() -> ProjectionError {
    ProjectionError::Invalid("library structural result invariant".into())
}
fn validate_input(inc: &EqualityIncidence) -> Result<(), ProjectionError> {
    allocation_bound(inc.n_eq_rows(), inc.n_vars, inc.vars.len())?;
    if inc.adj_ptr.len()
        != inc
            .n_eq_rows()
            .checked_add(1)
            .ok_or(ProjectionError::Limit)?
        || inc.adj_ptr.first() != Some(&0)
        || inc.adj_ptr.last() != Some(&inc.vars.len())
        || inc
            .adj_ptr
            .windows(2)
            .any(|p| p[0] > p[1] || p[1] > inc.vars.len())
        || inc.eq_row_inner_idx.windows(2).any(|p| p[0] >= p[1])
        || inc.vars.iter().any(|&c| c >= inc.n_vars)
    {
        return Err(ProjectionError::Invalid(
            "dense equality incidence extent or index".into(),
        ));
    }
    for row in 0..inc.n_eq_rows() {
        if inc.neighbors(row).windows(2).any(|p| p[0] >= p[1]) {
            return Err(ProjectionError::Invalid(
                "dense equality incidence is not sorted and unique".into(),
            ));
        }
    }
    Ok(())
}
fn block_dependencies(
    inc: &EqualityIncidence,
    matching: &BipartiteMatching,
    blocks: &[BlockTriangularBlock],
) -> Result<Vec<(usize, usize)>, ProjectionError> {
    let mut columns = vec![None; inc.n_vars];
    let mut rows = vec![false; inc.n_eq_rows()];
    for (owner, block) in blocks.iter().enumerate() {
        if block.cols.is_empty() || block.cols.len() != block.eq_rows.len() {
            return Err(invalid());
        }
        for &column in &block.cols {
            let target = columns.get_mut(column).ok_or_else(invalid)?;
            if target.replace(owner).is_some() {
                return Err(invalid());
            }
        }
        for &row in &block.eq_rows {
            let target = rows.get_mut(row).ok_or_else(invalid)?;
            if std::mem::replace(target, true)
                || matching
                    .row_to_var
                    .get(row)
                    .copied()
                    .flatten()
                    .is_none_or(|c| !block.cols.contains(&c))
            {
                return Err(invalid());
            }
        }
    }
    let mut dependencies = Vec::new();
    for (owner, block) in blocks.iter().enumerate() {
        for &row in &block.eq_rows {
            for &column in inc.neighbors(row) {
                if let Some(prerequisite) = columns[column] {
                    if prerequisite > owner {
                        return Err(invalid());
                    }
                    if prerequisite != owner {
                        dependencies.push((prerequisite, owner));
                    }
                }
            }
        }
    }
    dependencies.sort_unstable();
    dependencies.dedup();
    Ok(dependencies)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn chain() -> EqualityIncidence {
        EqualityIncidence {
            n_vars: 3,
            eq_row_inner_idx: vec![0, 1, 2],
            adj_ptr: vec![0, 1, 3, 5],
            vars: vec![0, 0, 1, 1, 2],
        }
    }
    #[test]
    fn checked_dense_chain_orders_prerequisites_and_refuses_bad_library_blocks() {
        let input = chain();
        let mut result = decompose(&input, &AtomicBool::new(false)).unwrap();
        assert_eq!(result.matching.size, 3);
        assert_eq!(
            result
                .blocks
                .iter()
                .map(|b| b.cols.clone())
                .collect::<Vec<_>>(),
            vec![vec![0], vec![1], vec![2]]
        );
        assert_eq!(result.dependencies, vec![(0, 1), (1, 2)]);
        result.blocks.swap(0, 2);
        assert!(block_dependencies(&input, &result.matching, &result.blocks).is_err());
        result.blocks.swap(0, 2);
        result.blocks[1].cols[0] = 0;
        assert!(block_dependencies(&input, &result.matching, &result.blocks).is_err());
    }
    #[test]
    fn dense_input_is_checked_before_library_indexing_and_cancel_before_work() {
        let input = chain();
        assert_eq!(
            decompose(&input, &AtomicBool::new(true)).unwrap_err(),
            ProjectionError::Cancelled
        );
        for bad in [
            EqualityIncidence {
                adj_ptr: vec![0, 99, 99, 99],
                ..chain()
            },
            EqualityIncidence {
                vars: vec![0, 0, 9, 1, 2],
                ..chain()
            },
            EqualityIncidence {
                eq_row_inner_idx: vec![0, 0, 2],
                ..chain()
            },
        ] {
            assert!(matches!(
                decompose(&bad, &AtomicBool::new(false)),
                Err(ProjectionError::Invalid(_))
            ));
        }
        assert!(allocation_bound(MATCHING_ROWS + 1, 1, 0).is_err());
        assert!(allocation_bound(1, 1, usize::MAX).is_err());
    }
}
