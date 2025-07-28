use noise::{NoiseFn, Seedable};

pub struct ShiftNScale<T, const SCALE: i32, const SHIFT: i32> {
    noise: T,
}

impl<T, const SCALE: i32, const SHIFT: i32> ShiftNScale<T, SCALE, SHIFT> {
    pub fn new(source: T) -> Self {
        Self { noise: source }
    }
}

impl<
    T, 
    const SCALE: i32, 
    const SHIFT: i32
> Default for ShiftNScale<T, SCALE, SHIFT>
where
    T: Default + Seedable,
{
    fn default() -> Self {
        Self {
            noise: Default::default(),
        }
    }
}

impl<
    T, 
    const SCALE: i32, 
    const SHIFT: i32
> Seedable for ShiftNScale<T, SCALE, SHIFT>
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

impl<
    T, 
    const SCALE: i32, 
    const SHIFT: i32, 
    const D: usize
> NoiseFn<f64, D> for ShiftNScale<T, SCALE, SHIFT>
where
    T: Default + Seedable + NoiseFn<f64, D>,
{
    fn get(&self, point: [f64; D]) -> f64 {
        (self.noise.get(point) + SHIFT as f64) / SCALE as f64
    }
}
