use std::f64::consts::E;

use noise::{math::vectors::Vector2, MultiFractal, NoiseFn, Seedable};

pub struct GFT<T> {
    pub octaves: usize,
    pub frequency: f64,
    pub lacunarity: f64,
    pub persistence: f64,
    pub gradient: f64,
    pub amplitude: f64,

    source: T,
    scale_factor: f64,
}

impl<T> GFT<T>
where
    T: NoiseFn<f64, 2>,
{
    pub const DEFAULT_OCTAVE_COUNT: usize = 6;
    pub const DEFAULT_FREQUENCY: f64 = 1.0;
    pub const DEFAULT_LACUNARITY: f64 = 2.0; 
    //pub const DEFAULT_LACUNARITY: f64 = core::f64::consts::PI * 2.0 / 3.0;
    pub const DEFAULT_PERSISTENCE: f64 = 0.5;
    pub const DEFAULT_GRADIENT: f64 = 1.;
    pub const DEFAULT_AMPLITUDE: f64 = 1.;
    pub const DEFAULT_SEED: u32 = 0;

    pub fn new_with_source(source: T) -> Self {
        Self {
            octaves: Self::DEFAULT_OCTAVE_COUNT,
            frequency: Self::DEFAULT_FREQUENCY,
            lacunarity: Self::DEFAULT_LACUNARITY,
            persistence: Self::DEFAULT_PERSISTENCE,
            gradient: Self::DEFAULT_GRADIENT,
            amplitude: Self::DEFAULT_AMPLITUDE,
            source: source,
            scale_factor: Self::calc_scale_factor(
                Self::DEFAULT_PERSISTENCE,
                Self::DEFAULT_OCTAVE_COUNT,
            ),
        }
    }

    fn calc_scale_factor(persistence: f64, octaves: usize) -> f64 {
        let denom = (1..=octaves).fold(0.0, |acc, x| 
            acc + persistence.powi(x as i32)
        );

        1.0 / denom
    }
}

impl<T> GFT<T>
where
    T: NoiseFn<f64, 2> + Default + Seedable,
{
    pub fn new(seed: u32) -> Self {
        Self::new_with_source(T::default().set_seed(seed))
    }
}

impl<T> Default for GFT<T>
where
    T: Default + NoiseFn<f64, 2> + Seedable,
{
    fn default() -> Self {
        Self::new(Self::DEFAULT_SEED)
    }
}

impl<T> MultiFractal for GFT<T>
where
    T: NoiseFn<f64, 2>,
{
    fn set_octaves(self, octaves: usize) -> Self {
        if self.octaves == octaves {
            return self;
        }

        Self {
            octaves,
            scale_factor: Self::calc_scale_factor(self.persistence, octaves),
            ..self
        }
    }

    fn set_frequency(self, frequency: f64) -> Self {
        Self { frequency, ..self }
    }

    fn set_lacunarity(self, lacunarity: f64) -> Self {
        Self { lacunarity, ..self }
    }

    fn set_persistence(self, persistence: f64) -> Self {
        Self {
            persistence,
            scale_factor: Self::calc_scale_factor(persistence, self.octaves),
            ..self
        }
    }
}

impl<T> GFT<T> {
    pub fn set_amplitude(self, amplitude: f64) -> Self {
        Self { amplitude, ..self }
    }

    pub fn set_gradient(self, gradient: f64) -> Self {
        Self { gradient, ..self }
    }

    fn get_gradient_influence(&self, flatness: f64) -> f64 {
        //1. / (1. + (flatness * self.gradient))
        (E * 0.375).powf(-(flatness * self.gradient).powi(2))
    }
}

/// 2-dimensional Fbm noise
impl<T> NoiseFn<f64, 2> for GFT<T>
where
    T: NoiseFn<f64, 2>,
{
    fn get(&self, point: [f64; 2]) -> f64 {
        let point = Vector2::from(point);
        let derivative_offset = 0.001;
        let offset_x_point = Vector2::new(derivative_offset, 0.);
        let offset_y_point = Vector2::new(0., derivative_offset);

        let mut result = 0.0;

        let mut total_flatness = 0.;

        for x in 0..self.octaves as i32 {
            let frequency = self.frequency * self.lacunarity.powi(x);
            let amplitude = self.amplitude * self.persistence.powi(x);

            // Get the signal.
            let noise_value = self.source.get(
                (point * frequency).into_array()
            );
            let noise_value_offset_x = self
                .source
                .get((point * frequency + offset_x_point).into_array());
            let noise_value_offset_y = self
                .source
                .get((point * frequency + offset_y_point).into_array());

            let derivative = Vector2::new(
                (noise_value_offset_x - noise_value) / derivative_offset,
                (noise_value_offset_y - noise_value) / derivative_offset,
            );
            total_flatness += derivative.magnitude() * (1. / (x + 1) as f64);

            let gradience = self.get_gradient_influence(total_flatness);

            // Add the signal to the result.
            result += noise_value * gradience * amplitude;
        }

        // Scale the result into the [-1,1] range
        result * self.scale_factor
    }
}
