/// an iterator that allows for seeking to a position by index
pub trait SeekableIterator<Idx, T>: Iterator<Item = T> {
    fn seek(&mut self, i: Idx);
}
