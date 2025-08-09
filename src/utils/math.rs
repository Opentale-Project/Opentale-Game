use std::ops::{Mul, Sub};

/// Framerate independent lerp.
/// Useful range of decay: 1 - 25 (slow - fast)
pub fn lerp_decay<
    T: Copy
        + Sub
        + std::ops::Add<
            <<T as std::ops::Sub>::Output as std::ops::Mul<f32>>::Output,
            Output = T,
        >,
>(
    a: T,
    b: T,
    decay: f32,
    delta_time: f32,
) -> T
where
    <T as Sub>::Output: Mul<f32>,
{
    b + (a - b) * (-decay * delta_time).exp()
}
