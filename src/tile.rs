
pub trait Tile: Copy+Eq+From<usize>+From<Self::Numeric> {
    type Numeric: Ord+Clone+Copy+PartialEq+Eq;
    const MAX: usize;

    fn invalid() -> Self;
    fn is_valid(&self) -> bool;
    fn as_usize(&self) -> usize;
    fn as_numeric(&self) -> Self::Numeric;
}
