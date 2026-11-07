pub fn find_window<T : PartialEq>(slice: &[T], window: &[T]) -> Option<usize> {
    for (offset, self_window) in slice.windows(window.len()).enumerate() {
        if self_window == window { return Some(offset) }
    }
    None
}

/// Split once at/excluding `window`
pub fn split_once<'a, T : PartialEq>(slice: &'a [T], window: &'_ [T]) -> Option<(&'a [T], &'a [T])> {
    let o = find_window(slice, window)?;
    let (a, b) = slice.split_at(o);
    let (_, b) = b.split_at(window.len());
    Some((a,b))
}
