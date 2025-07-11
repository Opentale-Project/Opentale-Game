use itertools::Itertools;

pub fn cube_cartesian_product<T: Clone>(
    iter: impl Iterator<Item = T> + Clone,
) -> impl Iterator<Item = (T, T, T)> {
    iter.clone()
        .cartesian_product(iter.clone())
        .cartesian_product(iter.clone())
        .map(|((x, y), z)| (x, y, z))
}
