/// A trait for compile-time validation that an array is non-empty.
pub(crate) trait NonEmptyArray<T, const N: usize> {
    const COMPTIME_SIZE_CHECK: () = assert!(N > 0);

    fn into_array(self) -> [T; N];
}

impl<T, const N: usize> NonEmptyArray<T, N> for [T; N] {
    fn into_array(self) -> [T; N] {
        #[expect(path_statements)] // used to evaluate array size at compile time
        Self::COMPTIME_SIZE_CHECK;
        self
    }
}
