use noise::{NoiseFn, Seedable};

pub struct Steepness<T> {
    source: T,
    sample_offset: f64,
}

impl<T> Steepness<T> {
    const DEFAULT_OFFSET: f64 = 1f64;

    pub fn new(source: T) -> Self {
        Self {
            source,
            sample_offset: Self::DEFAULT_OFFSET,
        }
    }
}

impl<T> Default for Steepness<T>
where
    T: Default + Seedable,
{
    fn default() -> Self {
        Self {
            source: Default::default(),
            sample_offset: Self::DEFAULT_OFFSET,
        }
    }
}

impl<T> Seedable for Steepness<T>
where
    T: Default + Seedable,
{
    fn set_seed(mut self, seed: u32) -> Self {
        self.source = T::default().set_seed(seed);
        self
    }

    fn seed(&self) -> u32 {
        self.source.seed()
    }
}

impl<T> NoiseFn<f64, 2usize> for Steepness<T>
where
    T: NoiseFn<f64, 2usize>,
{
    fn get(&self, point: [f64; 2usize]) -> f64 {
        let value_main = self.source.get(point);
        let value_offset_x = self.source.get([
            point[0] + self.sample_offset, 
            point[1]
        ]);
        let value_offset_y = self.source.get([
            point[0], 
            point[1] + self.sample_offset
        ]);

        let steepness_x = (value_main - value_offset_x).abs();
        let steepness_y = (value_main - value_offset_y).abs();

        (steepness_x + steepness_y) / 2.
    }
}
