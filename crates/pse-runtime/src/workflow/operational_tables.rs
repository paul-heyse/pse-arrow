// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The query surface over the operational store (Plan 22 O9).
//!
//! Each operational relation appears to DataFusion as a read-only [`TableProvider`] under
//! the schema `pse_ops` of a durable runtime's query sessions. A scan runs the relation's
//! generated statement (`pse_operations::tables`) with the typed filters recognized in the
//! query pushed down to PostgreSQL, and builds each page with the relation's generated
//! `pse-relations` builder, so every value is checked against its registry field contract
//! on the way in. Pages are bounded and read one at a time as DataFusion polls the stream;
//! nothing is materialized beyond the page being built.
//!
//! Pushdown is [`TableProviderFilterPushDown::Inexact`] for every recognized predicate
//! (an identity column equal to or in a list of literals, a vocabulary column equal to or
//! in a list of member spellings, a time column compared with or between instants) and
//! unsupported otherwise: the statement returns a superset of the matching rows and
//! DataFusion applies every filter again. A scan declares no constraints, so an engine
//! session binds it as an observed native source, and a query that reads it is never
//! served from a cache.
//!
//! The ADBC PostgreSQL driver is not used: it would add a C driver manager for tables the
//! generated builders already serve (`docs/dev/operational-store.md`).
use super::{Durability, RunResult, Runtime, WorkflowError};
use chrono::{DateTime, Utc};
use datafusion::{
    arrow::{
        datatypes::{DataType, SchemaRef, TimeUnit},
        record_batch::RecordBatch,
    },
    catalog::{Session, TableProvider},
    common::{DataFusionError, Result, ScalarValue, TableReference, tree_node::TreeNodeRecursion},
    execution::TaskContext,
    logical_expr::{
        Between, BinaryExpr, Expr, Operator, TableProviderFilterPushDown, TableType, expr::InList,
    },
    physical_expr::{EquivalenceProperties, PhysicalExpr},
    physical_plan::{
        DisplayAs, DisplayFormatType, ExecutionPlan, Partitioning, PlanProperties,
        SendableRecordBatchStream,
        execution_plan::{Boundedness, EmissionType},
        metrics::{ExecutionPlanMetricsSet, MetricBuilder, MetricsSet},
        stream::RecordBatchStreamAdapter,
    },
};
use pse_engine::session::EngineSession;
use pse_ids::SemanticId;
use pse_operations::{
    Store,
    tables::{
        AttemptScan, IncumbentScan, JobScan, ProgressEventScan, ProgressValueScan,
        PublicationMemberScan, PublicationScan, Scan, SettlementScan, SolutionScan, StudyPointScan,
        StudyScan, TimeRange, TransitionScan, WorkspaceScan,
    },
};
use pse_relations::columnar::RelationRow;
use std::{
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
    str::FromStr,
    sync::Arc,
};

/// The schema the operational relations are bound under in a query session.
pub const OPERATIONAL_SCHEMA: &str = "pse_ops";

/// How the predicates on a pushable column are recognized.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    /// A `pse.semantic_id` column (`FixedSizeBinary(16)`): equality and membership.
    Id,
    /// A registry vocabulary column (member spellings): equality and membership.
    Member,
    /// A `ts_us` column (`Timestamp(µs, UTC)`): comparisons and ranges.
    Time,
}

/// A recognized predicate on one column.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Predicate {
    /// The column equals one of these identities.
    Ids(BTreeSet<SemanticId>),
    /// The column is one of these spellings.
    Members(BTreeSet<String>),
    /// The column lies in this inclusive interval of microseconds since the Unix epoch.
    Range { from: Option<i64>, to: Option<i64> },
}

/// The recognized predicates of one scan, by column: each column's are intersected.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Constraints {
    ids: BTreeMap<String, BTreeSet<SemanticId>>,
    members: BTreeMap<String, BTreeSet<String>>,
    ranges: BTreeMap<String, (Option<i64>, Option<i64>)>,
}

impl Constraints {
    fn add(&mut self, column: &str, predicate: Predicate) {
        match predicate {
            Predicate::Ids(ids) => {
                self.ids
                    .entry(column.to_owned())
                    .and_modify(|held| held.retain(|id| ids.contains(id)))
                    .or_insert(ids);
            }
            Predicate::Members(members) => {
                self.members
                    .entry(column.to_owned())
                    .and_modify(|held| held.retain(|m| members.contains(m)))
                    .or_insert(members);
            }
            Predicate::Range { from, to } => {
                let held = self.ranges.entry(column.to_owned()).or_default();
                held.0 = held.0.max(from);
                held.1 = match (held.1, to) {
                    (Some(a), Some(b)) => Some(a.min(b)),
                    (a, b) => a.or(b),
                };
            }
        }
    }

    /// The identities a column is constrained to; empty when unconstrained, `None` when
    /// no identity can match.
    fn ids<T: From<SemanticId>>(&self, column: &str) -> Option<Vec<T>> {
        match self.ids.get(column) {
            None => Some(Vec::new()),
            Some(ids) if ids.is_empty() => None,
            Some(ids) => Some(ids.iter().copied().map(T::from).collect()),
        }
    }

    /// The vocabulary members a column is constrained to; empty when unconstrained,
    /// `None` when no member can match (a spelling that is not a member never does).
    fn members<T: FromStr + Ord>(&self, column: &str) -> Option<Vec<T>> {
        match self.members.get(column) {
            None => Some(Vec::new()),
            Some(spellings) => {
                let members: BTreeSet<T> =
                    spellings.iter().filter_map(|s| s.parse().ok()).collect();
                (!members.is_empty()).then(|| members.into_iter().collect())
            }
        }
    }

    /// The interval a time column is constrained to; `None` when it is empty. A bound
    /// beyond the representable instants is left open.
    fn range(&self, column: &str) -> Option<TimeRange> {
        let Some(&(from, to)) = self.ranges.get(column) else {
            return Some(TimeRange::default());
        };
        if let (Some(from), Some(to)) = (from, to)
            && from > to
        {
            return None;
        }
        Some(TimeRange {
            from: from.and_then(DateTime::<Utc>::from_timestamp_micros),
            to: to.and_then(DateTime::<Utc>::from_timestamp_micros),
        })
    }
}

/// A relation's typed filter, built from the predicates its query recognized.
pub(crate) trait Pushdown: Scan<Row: RelationRow> {
    /// The columns whose predicates the relation's statement takes.
    const COLUMNS: &'static [(&'static str, Kind)];
    /// The most rows one page holds.
    const PAGE: usize = 1024;

    /// The filter; `None` when no row can match.
    fn build(constraints: &Constraints) -> Option<Self>;
}

impl Pushdown for AttemptScan {
    const COLUMNS: &'static [(&'static str, Kind)] = &[
        ("attempt_id", Kind::Id),
        ("run_id", Kind::Id),
        ("state", Kind::Member),
        ("created_at", Kind::Time),
    ];
    fn build(c: &Constraints) -> Option<Self> {
        Some(Self {
            attempts: c.ids("attempt_id")?,
            runs: c.ids("run_id")?,
            states: c.members("state")?,
            created: c.range("created_at")?,
        })
    }
}

impl Pushdown for TransitionScan {
    const COLUMNS: &'static [(&'static str, Kind)] =
        &[("attempt_id", Kind::Id), ("at", Kind::Time)];
    fn build(c: &Constraints) -> Option<Self> {
        Some(Self {
            attempts: c.ids("attempt_id")?,
            at: c.range("at")?,
        })
    }
}

impl Pushdown for JobScan {
    const COLUMNS: &'static [(&'static str, Kind)] = &[
        ("job_id", Kind::Id),
        ("attempt_id", Kind::Id),
        ("state", Kind::Member),
        ("enqueued_at", Kind::Time),
    ];
    fn build(c: &Constraints) -> Option<Self> {
        Some(Self {
            jobs: c.ids("job_id")?,
            attempts: c.ids("attempt_id")?,
            states: c.members("state")?,
            enqueued: c.range("enqueued_at")?,
        })
    }
}

impl Pushdown for ProgressEventScan {
    const COLUMNS: &'static [(&'static str, Kind)] =
        &[("attempt_id", Kind::Id), ("at", Kind::Time)];
    fn build(c: &Constraints) -> Option<Self> {
        Some(Self {
            attempts: c.ids("attempt_id")?,
            at: c.range("at")?,
        })
    }
}

impl Pushdown for ProgressValueScan {
    const COLUMNS: &'static [(&'static str, Kind)] = &[("attempt_id", Kind::Id)];
    fn build(c: &Constraints) -> Option<Self> {
        Some(Self {
            attempts: c.ids("attempt_id")?,
        })
    }
}

impl Pushdown for IncumbentScan {
    const COLUMNS: &'static [(&'static str, Kind)] =
        &[("attempt_id", Kind::Id), ("at", Kind::Time)];
    fn build(c: &Constraints) -> Option<Self> {
        Some(Self {
            attempts: c.ids("attempt_id")?,
            at: c.range("at")?,
        })
    }
}

impl Pushdown for SolutionScan {
    const COLUMNS: &'static [(&'static str, Kind)] = &[
        ("solution_id", Kind::Id),
        ("created_by", Kind::Id),
        ("created_at", Kind::Time),
    ];
    // A seed's vectors can be long: a page holds fewer solutions.
    const PAGE: usize = 64;
    fn build(c: &Constraints) -> Option<Self> {
        Some(Self {
            solutions: c.ids("solution_id")?,
            creators: c.ids("created_by")?,
            created: c.range("created_at")?,
        })
    }
}

impl Pushdown for StudyScan {
    const COLUMNS: &'static [(&'static str, Kind)] = &[
        ("study_id", Kind::Id),
        ("attempt_id", Kind::Id),
        ("state", Kind::Member),
        ("created_at", Kind::Time),
    ];
    fn build(c: &Constraints) -> Option<Self> {
        Some(Self {
            studies: c.ids("study_id")?,
            attempts: c.ids("attempt_id")?,
            states: c.members("state")?,
            created: c.range("created_at")?,
        })
    }
}

impl Pushdown for StudyPointScan {
    const COLUMNS: &'static [(&'static str, Kind)] = &[
        ("study_id", Kind::Id),
        ("job_id", Kind::Id),
        ("state", Kind::Member),
    ];
    fn build(c: &Constraints) -> Option<Self> {
        Some(Self {
            studies: c.ids("study_id")?,
            jobs: c.ids("job_id")?,
            states: c.members("state")?,
        })
    }
}

impl Pushdown for WorkspaceScan {
    const COLUMNS: &'static [(&'static str, Kind)] = &[("workspace_id", Kind::Id)];
    fn build(c: &Constraints) -> Option<Self> {
        Some(Self {
            workspaces: c.ids("workspace_id")?,
        })
    }
}

impl Pushdown for PublicationScan {
    const COLUMNS: &'static [(&'static str, Kind)] = &[
        ("publication_id", Kind::Id),
        ("workspace_id", Kind::Id),
        ("attempt_id", Kind::Id),
        ("committed_at", Kind::Time),
    ];
    fn build(c: &Constraints) -> Option<Self> {
        Some(Self {
            publications: c.ids("publication_id")?,
            workspaces: c.ids("workspace_id")?,
            attempts: c.ids("attempt_id")?,
            committed: c.range("committed_at")?,
        })
    }
}

impl Pushdown for PublicationMemberScan {
    const COLUMNS: &'static [(&'static str, Kind)] = &[("publication_id", Kind::Id)];
    fn build(c: &Constraints) -> Option<Self> {
        Some(Self {
            publications: c.ids("publication_id")?,
        })
    }
}

impl Pushdown for SettlementScan {
    const COLUMNS: &'static [(&'static str, Kind)] =
        &[("attempt_id", Kind::Id), ("settled_at", Kind::Time)];
    fn build(c: &Constraints) -> Option<Self> {
        Some(Self {
            attempts: c.ids("attempt_id")?,
            settled: c.range("settled_at")?,
        })
    }
}

// ------------------------------------------------------------ recognition --

/// The column a predicate side names, through casts that keep its values' order and
/// identity for its kind.
fn column(expr: &Expr, columns: &[(&'static str, Kind)]) -> Option<(&'static str, Kind)> {
    let (inner, cast) = match expr {
        Expr::Column(_) => (expr, None),
        Expr::Cast(cast) => (cast.expr.as_ref(), Some(cast.field.data_type())),
        Expr::TryCast(cast) => (cast.expr.as_ref(), Some(cast.field.data_type())),
        _ => return None,
    };
    let Expr::Column(column) = inner else {
        return None;
    };
    let &(name, kind) = columns.iter().find(|(name, _)| *name == column.name)?;
    let preserved = match (kind, cast) {
        (_, None) => true,
        (Kind::Id, Some(ty)) => matches!(
            ty,
            DataType::Binary
                | DataType::LargeBinary
                | DataType::BinaryView
                | DataType::FixedSizeBinary(16)
        ),
        (Kind::Member, Some(ty)) => {
            matches!(
                ty,
                DataType::Utf8 | DataType::LargeUtf8 | DataType::Utf8View
            )
        }
        // A finer or equal unit keeps every instant distinct and in order.
        (Kind::Time, Some(DataType::Timestamp(unit, zone))) => {
            matches!(unit, TimeUnit::Microsecond | TimeUnit::Nanosecond)
                && zone.as_deref().is_none_or(utc)
        }
        (Kind::Time, Some(_)) => false,
    };
    preserved.then_some((name, kind))
}

fn utc(zone: &str) -> bool {
    matches!(zone, "UTC" | "+00:00" | "Z" | "utc")
}

fn literal(expr: &Expr) -> Option<&ScalarValue> {
    match expr {
        Expr::Literal(value, _) if !value.is_null() => Some(value),
        _ => None,
    }
}

/// The identity an identity literal names; `None` for a value that is not a binary
/// literal, `Some(None)` for bytes no identity can equal.
fn identity(value: &ScalarValue) -> Option<Option<SemanticId>> {
    let bytes = match value {
        ScalarValue::FixedSizeBinary(_, Some(bytes))
        | ScalarValue::Binary(Some(bytes))
        | ScalarValue::LargeBinary(Some(bytes))
        | ScalarValue::BinaryView(Some(bytes)) => bytes.as_slice(),
        _ => return None,
    };
    Some(<[u8; 16]>::try_from(bytes).ok().map(SemanticId::from_bytes))
}

fn spelling(value: &ScalarValue) -> Option<String> {
    match value {
        ScalarValue::Utf8(Some(text))
        | ScalarValue::LargeUtf8(Some(text))
        | ScalarValue::Utf8View(Some(text)) => Some(text.clone()),
        _ => None,
    }
}

/// An instant literal in microseconds since the Unix epoch, rounded outward: down for a
/// lower bound, up for an upper bound, so the interval never shrinks. An Arrow timestamp
/// value is the epoch offset whatever its zone, which only changes how it displays.
fn micros(value: &ScalarValue, lower: bool) -> Option<i64> {
    let (units, per_second): (i64, i64) = match value {
        ScalarValue::TimestampSecond(Some(v), _) => (*v, 1),
        ScalarValue::TimestampMillisecond(Some(v), _) => (*v, 1_000),
        ScalarValue::TimestampMicrosecond(Some(v), _) => (*v, 1_000_000),
        ScalarValue::TimestampNanosecond(Some(v), _) => {
            return Some(if lower {
                v.div_euclid(1_000)
            } else {
                v.div_euclid(1_000) + i64::from(v.rem_euclid(1_000) != 0)
            });
        }
        _ => return None,
    };
    units.checked_mul(1_000_000 / per_second)
}

fn swap(op: Operator) -> Option<Operator> {
    Some(match op {
        Operator::Eq => Operator::Eq,
        Operator::Lt => Operator::Gt,
        Operator::LtEq => Operator::GtEq,
        Operator::Gt => Operator::Lt,
        Operator::GtEq => Operator::LtEq,
        _ => return None,
    })
}

/// The typed predicate a filter states on one pushable column, if it states exactly one.
fn recognize(expr: &Expr, columns: &[(&'static str, Kind)]) -> Option<(&'static str, Predicate)> {
    match expr {
        // DataFusion rewrites a short `IN` list as a disjunction of equalities: a union of
        // memberships on one column is recognized as one.
        Expr::BinaryExpr(BinaryExpr {
            left,
            op: Operator::Or,
            right,
        }) => {
            let (left, right) = (recognize(left, columns)?, recognize(right, columns)?);
            if left.0 != right.0 {
                return None;
            }
            let union = match (left.1, right.1) {
                (Predicate::Ids(mut a), Predicate::Ids(b)) => {
                    a.extend(b);
                    Predicate::Ids(a)
                }
                (Predicate::Members(mut a), Predicate::Members(b)) => {
                    a.extend(b);
                    Predicate::Members(a)
                }
                _ => return None,
            };
            Some((left.0, union))
        }
        Expr::BinaryExpr(BinaryExpr { left, op, right }) => {
            let (name, kind, value, op) = match (column(left, columns), column(right, columns)) {
                (Some((name, kind)), None) => (name, kind, literal(right)?, *op),
                (None, Some((name, kind))) => (name, kind, literal(left)?, swap(*op)?),
                _ => return None,
            };
            let predicate = match (kind, op) {
                (Kind::Id, Operator::Eq) => Predicate::Ids(identity(value)?.into_iter().collect()),
                (Kind::Member, Operator::Eq) => {
                    Predicate::Members(BTreeSet::from([spelling(value)?]))
                }
                (Kind::Time, Operator::Eq) => Predicate::Range {
                    from: Some(micros(value, true)?),
                    to: Some(micros(value, false)?),
                },
                (Kind::Time, Operator::Gt | Operator::GtEq) => Predicate::Range {
                    from: Some(micros(value, true)?),
                    to: None,
                },
                (Kind::Time, Operator::Lt | Operator::LtEq) => Predicate::Range {
                    from: None,
                    to: Some(micros(value, false)?),
                },
                _ => return None,
            };
            Some((name, predicate))
        }
        Expr::InList(InList {
            expr,
            list,
            negated: false,
        }) => {
            let (name, kind) = column(expr, columns)?;
            let values = list.iter().map(literal).collect::<Option<Vec<_>>>()?;
            let predicate = match kind {
                Kind::Id => Predicate::Ids(
                    values
                        .into_iter()
                        .map(identity)
                        .collect::<Option<Vec<_>>>()?
                        .into_iter()
                        .flatten()
                        .collect(),
                ),
                Kind::Member => Predicate::Members(
                    values
                        .into_iter()
                        .map(spelling)
                        .collect::<Option<BTreeSet<_>>>()?,
                ),
                Kind::Time => return None,
            };
            Some((name, predicate))
        }
        Expr::Between(Between {
            expr,
            negated: false,
            low,
            high,
        }) => {
            let (name, Kind::Time) = column(expr, columns)? else {
                return None;
            };
            Some((
                name,
                Predicate::Range {
                    from: Some(micros(literal(low)?, true)?),
                    to: Some(micros(literal(high)?, false)?),
                },
            ))
        }
        _ => None,
    }
}

/// The typed filter of a scan from the filters DataFusion pushed into it; `None` when no
/// row can match.
pub(crate) fn scan_of<S: Pushdown>(filters: &[Expr]) -> Option<S> {
    let mut constraints = Constraints::default();
    for (name, predicate) in filters.iter().filter_map(|f| recognize(f, S::COLUMNS)) {
        constraints.add(name, predicate);
    }
    S::build(&constraints)
}

// ---------------------------------------------------------------- provider --

/// One operational relation as a read-only DataFusion table.
#[derive(Debug)]
pub(crate) struct OperationalTable<S> {
    store: Store,
    registry: Arc<pse_schema::Registry>,
    schema: SchemaRef,
    scan: PhantomData<fn() -> S>,
}

fn external(error: impl std::error::Error + Send + Sync + 'static) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}

/// The query schema of `S`'s relation: what its generated builder produces, without the
/// relation-level identity a query source does not carry. The registry's native checks
/// are bound first: the builders validate every page with them.
fn schema_of<S: Pushdown>(
    registry: &pse_schema::Registry,
) -> Result<SchemaRef, pse_relations::RelationError> {
    pse_engine::validation::bind_defaults(registry)?;
    let empty = S::Row::finish(S::Row::builder(registry, 0)?)?;
    Ok(pse_engine::session::query_schema::schema(
        empty.batch().schema().as_ref(),
    ))
}

impl<S: Pushdown> OperationalTable<S> {
    /// The relation of `S` over `store`.
    ///
    /// # Errors
    /// The registry lacks the relation's generated declaration.
    pub(crate) fn new(
        store: Store,
        registry: Arc<pse_schema::Registry>,
    ) -> Result<Self, pse_relations::RelationError> {
        Ok(Self {
            store,
            schema: schema_of::<S>(&registry)?,
            registry,
            scan: PhantomData,
        })
    }
}

impl<S: Pushdown> TableProvider for OperationalTable<S> {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&self.schema)
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> Result<Vec<TableProviderFilterPushDown>> {
        Ok(filters
            .iter()
            .map(|filter| {
                if recognize(filter, S::COLUMNS).is_some() {
                    TableProviderFilterPushDown::Inexact
                } else {
                    TableProviderFilterPushDown::Unsupported
                }
            })
            .collect())
    }

    fn scan<'s, 't, 'p, 'f, 'future>(
        &'s self,
        state: &'t dyn Session,
        projection: Option<&'p Vec<usize>>,
        filters: &'f [Expr],
        limit: Option<usize>,
    ) -> pse_engine::BoxFut<'future, Result<Arc<dyn ExecutionPlan>>>
    where
        's: 'future,
        't: 'future,
        'p: 'future,
        'f: 'future,
        Self: 'future,
    {
        Box::pin(async move {
            let schema = match projection {
                Some(columns) => Arc::new(self.schema.project(columns)?),
                None => Arc::clone(&self.schema),
            };
            let page = state.config().batch_size().clamp(1, S::PAGE);
            let plan: Arc<dyn ExecutionPlan> = Arc::new(OperationalScanExec::<S> {
                store: self.store.clone(),
                registry: Arc::clone(&self.registry),
                scan: scan_of::<S>(filters),
                projection: projection.cloned(),
                limit,
                page,
                properties: Arc::new(PlanProperties::new(
                    EquivalenceProperties::new(Arc::clone(&schema)),
                    Partitioning::UnknownPartitioning(1),
                    EmissionType::Incremental,
                    Boundedness::Bounded,
                )),
                metrics: ExecutionPlanMetricsSet::new(),
            });
            Ok(plan)
        })
    }
}

/// Streams one operational relation from its generated statement in bounded pages.
#[derive(Debug)]
pub(crate) struct OperationalScanExec<S> {
    store: Store,
    registry: Arc<pse_schema::Registry>,
    /// The typed filter the statement runs with; `None` when no row can match.
    scan: Option<S>,
    projection: Option<Vec<usize>>,
    limit: Option<usize>,
    page: usize,
    properties: Arc<PlanProperties>,
    metrics: ExecutionPlanMetricsSet,
}

impl<S: Pushdown> DisplayAs for OperationalScanExec<S> {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OperationalScanExec: {OPERATIONAL_SCHEMA}.{}, filter={:?}, page={}, limit={:?}",
            S::TABLE,
            self.scan,
            self.page,
            self.limit
        )
    }
}

/// Where a scan's stream is between pages.
struct Cursor<S: Scan> {
    scan: S,
    store: Store,
    registry: Arc<pse_schema::Registry>,
    projection: Option<Vec<usize>>,
    after: Option<S::Key>,
    remaining: Option<usize>,
    page: usize,
    fetched: datafusion::physical_plan::metrics::Count,
    done: bool,
}

impl<S: Pushdown> Cursor<S> {
    /// Read and build the next page; `None` once the relation is exhausted.
    async fn next(mut self) -> Result<Option<(RecordBatch, Self)>> {
        let want = self.remaining.map_or(self.page, |r| r.min(self.page));
        if self.done || want == 0 {
            return Ok(None);
        }
        let limit = i64::try_from(want).map_err(external)?;
        let rows = self
            .scan
            .page(&self.store, self.after.as_ref(), limit)
            .await
            .map_err(external)?;
        self.fetched.add(rows.len());
        self.done = rows.len() < want;
        self.remaining = self.remaining.map(|r| r.saturating_sub(rows.len()));
        let Some(last) = rows.last() else {
            return Ok(None);
        };
        self.after = Some(S::key(last));
        let mut builder = S::Row::builder(&self.registry, rows.len()).map_err(external)?;
        for row in rows {
            S::Row::push(&mut builder, row).map_err(external)?;
        }
        let batch = pse_engine::session::query_schema::batch(
            S::Row::finish(builder).map_err(external)?.batch(),
        )?;
        let batch = match &self.projection {
            Some(columns) => batch.project(columns)?,
            None => batch,
        };
        Ok(Some((batch, self)))
    }
}

impl<S: Pushdown> ExecutionPlan for OperationalScanExec<S> {
    fn name(&self) -> &'static str {
        "OperationalScanExec"
    }

    fn properties(&self) -> &Arc<PlanProperties> {
        &self.properties
    }

    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        Vec::new()
    }

    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if children.is_empty() {
            Ok(self)
        } else {
            Err(DataFusionError::Plan(
                "an operational scan has no inputs".into(),
            ))
        }
    }

    fn apply_expressions(
        &self,
        _: &mut dyn FnMut(&Arc<dyn PhysicalExpr>) -> Result<TreeNodeRecursion>,
    ) -> Result<TreeNodeRecursion> {
        Ok(TreeNodeRecursion::Continue)
    }

    fn metrics(&self) -> Option<MetricsSet> {
        Some(self.metrics.clone_inner())
    }

    fn execute(
        &self,
        partition: usize,
        _context: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        if partition != 0 {
            return Err(DataFusionError::Execution(format!(
                "an operational scan has one partition, not {}",
                partition + 1
            )));
        }
        let schema = self.schema();
        let Some(scan) = self.scan.clone() else {
            return Ok(Box::pin(RecordBatchStreamAdapter::new(
                schema,
                futures_util::stream::empty(),
            )));
        };
        let cursor = Cursor {
            scan,
            store: self.store.clone(),
            registry: Arc::clone(&self.registry),
            projection: self.projection.clone(),
            after: None,
            remaining: self.limit,
            page: self.page,
            fetched: MetricBuilder::new(&self.metrics).counter("fetched_rows", partition),
            done: false,
        };
        Ok(Box::pin(RecordBatchStreamAdapter::new(
            schema,
            futures_util::stream::try_unfold(cursor, Cursor::next),
        )))
    }
}

// ------------------------------------------------------------ registration --

fn table<S: Pushdown>(
    store: &Store,
    registry: &Arc<pse_schema::Registry>,
) -> Result<(&'static str, Arc<dyn TableProvider>), WorkflowError> {
    let table =
        OperationalTable::<S>::new(store.clone(), Arc::clone(registry)).map_err(super::relation)?;
    Ok((S::TABLE, Arc::new(table)))
}

/// Every operational relation the query surface serves, by its table name, over `store`.
fn tables(
    store: &Store,
    registry: &Arc<pse_schema::Registry>,
) -> Result<Vec<(&'static str, Arc<dyn TableProvider>)>, WorkflowError> {
    Ok(vec![
        table::<AttemptScan>(store, registry)?,
        table::<TransitionScan>(store, registry)?,
        table::<JobScan>(store, registry)?,
        table::<ProgressEventScan>(store, registry)?,
        table::<ProgressValueScan>(store, registry)?,
        table::<IncumbentScan>(store, registry)?,
        table::<SolutionScan>(store, registry)?,
        table::<StudyScan>(store, registry)?,
        table::<StudyPointScan>(store, registry)?,
        table::<WorkspaceScan>(store, registry)?,
        table::<PublicationScan>(store, registry)?,
        table::<PublicationMemberScan>(store, registry)?,
        table::<SettlementScan>(store, registry)?,
    ])
}

impl Runtime {
    /// The operational relations of this runtime's store as read-only providers, by their
    /// table name under [`OPERATIONAL_SCHEMA`]; none for an ephemeral runtime.
    ///
    /// # Errors
    /// The registry lacks an operational relation's generated declaration.
    pub fn operational_tables(
        &self,
    ) -> Result<Vec<(&'static str, Arc<dyn TableProvider>)>, WorkflowError> {
        match &self.durability {
            Durability::Ephemeral => Ok(Vec::new()),
            Durability::Durable(operations) => tables(operations.store(), &self.registry),
        }
    }

    /// `session` with the operational relations bound under [`OPERATIONAL_SCHEMA`] as
    /// observed native sources, when this runtime is durable; an ephemeral runtime binds
    /// none and returns the session unchanged.
    ///
    /// # Errors
    /// A name the session already binds to another source; cancellation.
    pub fn with_operational_tables(
        &self,
        session: &EngineSession,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<EngineSession, WorkflowError> {
        let mut session = session.clone();
        for (name, provider) in self.operational_tables()? {
            session = session.with_provider(
                TableReference::partial(OPERATIONAL_SCHEMA, name),
                provider,
                cancel,
            )?;
        }
        Ok(session)
    }

    /// A query session over the operational relations (under [`OPERATIONAL_SCHEMA`], when
    /// durable), starting from `base` (a publication's session, say) or an empty session,
    /// and with the retained result relations of `result` under `workspace.<namespace>`.
    ///
    /// # Errors
    /// The result's relations could not be encoded or bound; a binding conflict;
    /// cancellation.
    pub fn query_session(
        &self,
        base: Option<&EngineSession>,
        result: Option<&RunResult>,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<EngineSession, WorkflowError> {
        let session = match base {
            Some(base) => base.clone(),
            None => self
                .sessions
                .candidate(BTreeMap::new(), Arc::clone(&self.registry), cancel)?,
        };
        let session = match result {
            None => session,
            Some(result) => {
                let tables = result.tables().map_err(WorkflowError::Shared)?;
                let mut rows = BTreeMap::new();
                for (id, batch) in tables {
                    let spec = self.registry.relation_by_id(*id).ok_or_else(|| {
                        super::contract("a result relation is absent from the registry")
                    })?;
                    rows.insert(spec.key, batch.clone());
                }
                session.with_checked_workspace(rows, cancel)?
            }
        };
        self.with_operational_tables(&session, cancel)
    }
}

#[cfg(test)]
#[path = "operational_tables_tests.rs"]
mod tests;
