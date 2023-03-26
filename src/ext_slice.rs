pub trait ExtSlice {
    type Item;

    #[doc(hidden)] fn _ext_slice(&self) -> &[Self::Item];

    fn find_window(&self, window: &[Self::Item]) -> Option<usize> where Self::Item : PartialEq {
        for (offset, self_window) in self._ext_slice().windows(window.len()).enumerate() {
            if self_window == window { return Some(offset) }
        }
        None
    }

    /// Split once at/excluding `window`
    fn split_once<'a>(&'a self, window: &[Self::Item]) -> Option<(&'a [Self::Item], &'a [Self::Item])> where Self::Item : PartialEq {
        let o = self.find_window(window)?;
        let (a, b) = self._ext_slice().split_at(o);
        let (_, b) = b.split_at(window.len());
        Some((a,b))
    }
}

impl<T> ExtSlice for [T] {
    type Item = T;
    fn _ext_slice(&self) -> &[Self::Item] { self }
}
