pub(crate) trait NonEmptyArray<T, const N: usize> {
    const COMPTIME_SIZE_CHECK: () = assert!(N > 0);

    fn into_array(self) -> [T; N];
}

impl<T, const N: usize> NonEmptyArray<T, N> for [T; N] {
    fn into_array(self) -> [T; N] {
        #[allow(path_statements)] // used to evaluate array size at compile time
        Self::COMPTIME_SIZE_CHECK;
        self
    }
}
