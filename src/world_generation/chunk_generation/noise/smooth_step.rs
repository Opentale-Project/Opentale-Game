use noise::{NoiseFn, Seedable};
use std::f64::consts::PI;

pub struct SmoothStep<T> {
    noise: T,
    steps: f64,
    smoothness: f64,
}

impl<T> SmoothStep<T> {
    pub const DEFAULT_STEPS: f64 = 1.;
    pub const DEFAULT_SMOOTHNESS: f64 = 0.25;

    pub fn new(source: T) -> Self {
        Self {
            noise: source,
            steps: Self::DEFAULT_STEPS,
            smoothness: Self::DEFAULT_SMOOTHNESS,
        }
    }

    pub fn set_steps(self, steps: f64) -> Self {
        Self { steps, ..self }
    }

    pub fn set_smoothness(self, smoothness: f64) -> Self {
        Self { smoothness, ..self }
    }
}

impl<T> Default for SmoothStep<T>
where
    T: Default + Seedable,
{
    fn default() -> Self {
        Self {
            noise: Default::default(),
            steps: Self::DEFAULT_STEPS,
            smoothness: Self::DEFAULT_SMOOTHNESS,
        }
    }
}

impl<T> Seedable for SmoothStep<T>
where
    T: Default + Seedable,
{
    fn set_seed(mut self, seed: u32) -> Self {
        self.noise = T::default().set_seed(seed);
        self
    }

    fn seed(&self) -> u32 {
        self.noise.seed()
    }
}

impl<T, const D: usize> NoiseFn<f64, D> for SmoothStep<T>
where
    T: NoiseFn<f64, D>,
{
    fn get(&self, point: [f64; D]) -> f64 {
        smooth_floor(
            self.noise.get(point) * self.steps + 0.5, 
            self.smoothness
        ) / self.steps
    }
}

// https://www.desmos.com/calculator/zyrixan1eo
fn smooth_floor(x: f64, factor: f64) -> f64 {
    let sigmoid_value = sigmoid((PI * x).sin(), factor);
    x + (2.0 * sigmoid_value - 1.0) * ((PI * x).cos().asin() / PI) - 0.5
}

fn sigmoid(x: f64, factor: f64) -> f64 {
    x / (factor + x.abs()) * 0.5 + 0.5
}
