#[inline]
pub fn div_floor<
    T: num_traits::Num + num_traits::identities::One + num_traits::sign::Signed + Copy,
>(
    x: T,
    y: T,
) -> T {
    let new = x / y;
    if x.is_negative() ^ y.is_negative() {
        new - T::one()
    } else {
        new
    }
}
