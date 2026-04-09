/// An implementation exists iff A + B = C
pub trait IsSum<const A: usize, const B: usize, const C: usize>: sealed::Sealed {}

#[derive(Debug)]
pub struct SixEqualsFourPlusTwo;
impl sealed::Sealed for SixEqualsFourPlusTwo {}
impl IsSum<4, 2, 6> for SixEqualsFourPlusTwo {}

// Seal `IsSum` so that improper instances cannot be defined.
mod sealed {
    pub trait Sealed {}
}
