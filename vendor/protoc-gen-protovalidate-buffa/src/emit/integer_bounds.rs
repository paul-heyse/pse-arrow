//! Discharge only comparisons established by the actual Rust integer domain.

pub(super) trait Integer: Copy + PartialEq {
    const MIN: Self;
    const MAX: Self;
}
macro_rules! integer { ($($ty:ty),*) => { $(
    impl Integer for $ty { const MIN: Self = <$ty>::MIN; const MAX: Self = <$ty>::MAX; }
)* }; }
integer!(i32, i64, u32, u64);

pub(super) fn lower<T: Integer>(value: &T) -> bool { *value != T::MIN }
pub(super) fn upper<T: Integer>(value: &T) -> bool { *value != T::MAX }
pub(super) fn discharged<T: Integer>(low: T, high: T) -> (bool, bool) {
    (!lower(&low), !upper(&high))
}
