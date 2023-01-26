/// an iterator that allows for seeking to a position by index, and also peeking
pub trait SeekablePeekableIterator<Idx, T>: Iterator<Item = T> {
    fn seek(&mut self, i: Idx);

    // this isn't quite like standard library peek(), since usage here is with Neighbourhood, which is created as the iterator progresses
    // so we return by value not reference
    // TODO think if there's a better way, or consider renaming
    fn peek(&self) -> Option<T>;
}
