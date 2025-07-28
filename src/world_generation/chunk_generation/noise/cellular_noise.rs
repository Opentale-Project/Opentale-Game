use fastnoise_lite::{CellularDistanceFunction, CellularReturnType, FastNoiseLite, NoiseType};
use noise::{NoiseFn, Seedable};

pub struct Cellular {
    fast_noise: FastNoiseLite,
}

impl Cellular {
    pub fn new(seed: i32) -> Self {
        let mut noise = FastNoiseLite::with_seed(seed);
        noise.set_noise_type(Some(NoiseType::Cellular));
        Self { fast_noise: noise }
    }

    pub fn set_frequency(mut self, frequency: f32) -> Self {
        self.fast_noise.set_frequency(Some(frequency));
        self
    }

    pub fn set_distance_function(
        mut self, 
        distance_function: CellularDistanceFunction
    ) -> Self {
        self.fast_noise
            .set_cellular_distance_function(Some(distance_function));
        self
    }

    pub fn set_return_type(mut self, return_type: CellularReturnType) -> Self {
        self.fast_noise
            .set_cellular_return_type(Some(return_type));
        self
    }

    pub fn set_jitter(mut self, jitter: f32) -> Self {
        self.fast_noise
            .set_cellular_jitter(Some(jitter));
        self
    }
}

impl Seedable for Cellular {
    fn set_seed(mut self, seed: u32) -> Self {
        self.fast_noise
            .set_seed(Some(seed as i32));
        self
    }

    fn seed(&self) -> u32 {
        0
    }
}

impl NoiseFn<f64, 2> for Cellular {
    fn get(&self, point: [f64; 2]) -> f64 {
        let noise = self.fast_noise.get_noise_2d(
            point[0] as f32, 
            point[1] as f32
        );
        
        ((noise + 1.) * 0.5).into()
    }
}
