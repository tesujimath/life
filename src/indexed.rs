/// something which has an index
pub trait Indexed<Idx> {
    fn index(&self) -> Idx;
}
