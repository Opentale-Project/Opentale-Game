use std::ops::Range;

use rand::{rngs::StdRng, Rng};

#[derive(Clone, Copy)]
pub struct EntryRange {
    pub start: f32,
    pub end: f32,
}

impl EntryRange {
    pub fn new(a: f32, b: f32) -> Self {
        Self { start: a, end: b }
    }

    pub fn get_value(&self, percent: f32) -> f32 {
        self.start + (self.end - self.start) * percent
    }

    pub fn get_sub_range(&self, percent_a: f32, percent_b: f32) -> Self {
        Self::new(self.get_value(percent_a), self.get_value(percent_b))
    }

    pub fn get_value_with_steps(&self, i: i32, max: i32) -> f32 {
        self.get_value(i as f32 / max as f32)
    }

    pub fn get_sub_range_with_steps(&self, i_a: i32, i_b: i32, max: i32) -> Self {
        Self::new(
            self.get_value_with_steps(i_a, max),
            self.get_value_with_steps(i_b, max),
        )
    }

    pub fn rng(&self, rng: &mut StdRng) -> f32 {
        rng.random_range(self.start..self.end)
    }
}

impl From<Range<f32>> for EntryRange {
    fn from(range: Range<f32>) -> Self {
        Self::new(range.start, range.end)
    }
}

impl Into<Range<f32>> for EntryRange {
    fn into(self) -> Range<f32> {
        self.start..self.end
    }
}
