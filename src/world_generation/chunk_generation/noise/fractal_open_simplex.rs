use noise::core::simplex::simplex_2d;
use noise::permutationtable::PermutationTable;
use noise::{NoiseFn, Seedable, Vector2};

#[derive(Clone, Copy, Debug)]
pub struct FractalOpenSimplex<R>
where
    R: NoiseFn<f64, 2usize>,
{
    seed: u32,
    frequency: f64,
    amplitude: f64,
    octaves: i32,
    lacunarity: f64,
    persistence: f64,
    permutation_table: PermutationTable,
    roughness: R,
}

impl<R> FractalOpenSimplex<R>
where
    R: NoiseFn<f64, 2usize>,
{
    pub fn new(
        seed: u32,
        frequency: f64,
        amplitude: f64,
        octaves: i32,
        lacunarity: f64,
        persistence: f64,
        roughness: R,
    ) -> Self {
        Self {
            seed,
            frequency,
            amplitude,
            octaves,
            lacunarity,
            persistence,
            permutation_table: PermutationTable::new(seed),
            roughness,
        }
    }
}

impl<R> Seedable for FractalOpenSimplex<R>
where
    R: NoiseFn<f64, 2usize>,
{
    fn set_seed(self, seed: u32) -> Self {
        if self.seed == seed {
            return self;
        }

        Self {
            seed,
            frequency: self.frequency,
            amplitude: self.amplitude,
            octaves: self.octaves,
            lacunarity: self.lacunarity,
            persistence: self.persistence,
            permutation_table: self.permutation_table,
            roughness: self.roughness,
        }
    }

    fn seed(&self) -> u32 {
        self.seed
    }
}

impl<R> NoiseFn<f64, 2usize> for FractalOpenSimplex<R>
where
    R: NoiseFn<f64, 2usize>,
{
    fn get(&self, point: [f64; 2]) -> f64 {
        let roughness = 0.; //self.roughness.get(point);
        fractal_noise(
            point[0],
            point[1],
            self.frequency,
            self.amplitude,
            self.octaves,
            self.lacunarity,
            self.persistence + roughness,
            &self.permutation_table,
        )
        .max(2.)
    }
}

pub fn fractal_noise(
    x: f64,
    z: f64,
    frequency: f64,
    amplitude: f64,
    octaves: i32,
    lacunarity: f64,
    persistence: f64,
    hasher: &PermutationTable,
) -> f64 {
    let mut noise_value: f64 = 0.;
    let mut _total_flatness: f64 = 0.;

    for octave in 0..octaves {
        let result = noise(
            x,
            z,
            frequency * lacunarity.powi(octave),
            amplitude * persistence.powi(octave),
            hasher,
        );
        let flatness = result.1[0].abs() + result.1[1].abs();
        _total_flatness += flatness;
        noise_value += result.0; // * (1. / (1. + total_flatness));
    }

    noise_value
}

pub fn noise(
    x: f64,
    z: f64,
    frequency: f64,
    amplitude: f64,
    hasher: &PermutationTable,
) -> (f64, [f64; 2]) {
    let result = simplex_2d(
        Vector2::new(
            x * frequency, 
            z * frequency
        ), 
        hasher
    );
    ((result.0 + 1.) * 0.5 * amplitude, result.1)
}
