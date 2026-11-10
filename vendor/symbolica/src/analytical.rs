//! Borrowed current-operation controls for exact analytical products.
use crate::{atom::{Atom,AtomView,Indeterminate},domains::atom::AtomField,evaluate::{OperationControl,OperationExtent},poly::series::{Series,SeriesDepth,SeriesError},state::Workspace};
/// Whole-operation failure; no variant contains a partial analytical product.
#[derive(Debug)]
pub enum SeriesOperationError<E>{
    /// The original caller refused further work.
    Control(E),
    /// Native formal series semantics refused the expression.
    Series(SeriesError),
}
impl<E> From<SeriesError> for SeriesOperationError<E>{fn from(error:SeriesError)->Self{Self::Series(error)}}
/// Differentiate with safe points before each recursive source operation. A scalar
/// exact arithmetic/normalization call remains indivisible. Refusal discards the
/// complete product, leaving caller-owned source and control unchanged.
pub fn derivative_with<C:OperationControl>(source:AtomView<'_>,axis:&Indeterminate,control:&mut C)->Result<Atom,C::Error>{
    let mut output=Atom::Zero;
    Workspace::get_local().with(|workspace|source.derivative_with_ws_into_control(axis,workspace,&mut output,control))?;
    control.check(OperationExtent{input_bytes:source.get_byte_size(),output_bytes:output.as_view().get_byte_size(),..Default::default()})?;
    Ok(output)
}
/// Expand using the ordinary native series implementation, polling the borrowed
/// control at recursive entry, retry and repeated derivative generation. Native
/// scalar series arithmetic is indivisible; callers admit its bounded width first.
pub fn series_with<C:OperationControl>(source:AtomView<'_>,axis:&Indeterminate,point:AtomView<'_>,depth:SeriesDepth,control:&mut C)->Result<Series<AtomField>,SeriesOperationError<C::Error>>{
    source.series_control(axis,point,depth,control)
}
