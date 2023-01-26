pub trait SeekableIterator<Idx, T>: Iterator<Item = T> {
    /// seek to first item at or past `i`, consuming it only if it is exactly at `i`
    fn seek(&mut self, i: Idx) -> Option<T>;

    // this isn't quite like standard library peek(), since usage here is with Neighbourhood, which is created as the iterator progresses
    // so we return by value not reference
    // TODO think if there's a better way, or consider renaming
    fn peek(&self) -> Option<T>;
}
