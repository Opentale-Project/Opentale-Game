#[derive(Debug, PartialEq, Eq)]
pub struct AmbiantOcclusion {
    pub corner_1: u8,
    pub corner_2: u8,
    pub corner_3: u8,
    pub corner_4: u8,
}

impl AmbiantOcclusion {
    pub fn new_full() -> Self {
        AmbiantOcclusion {
            corner_1: 3,
            corner_2: 3,
            corner_3: 3,
            corner_4: 3,
        }
    }

    pub fn get_colors(&self) -> [[f32; 4]; 4] {
        let corner_1 = (self.corner_1 as f32 / 4.) + 0.25;
        let corner_2 = (self.corner_2 as f32 / 4.) + 0.25;
        let corner_3 = (self.corner_3 as f32 / 4.) + 0.25;
        let corner_4 = (self.corner_4 as f32 / 4.) + 0.25;

        [
            [corner_1, corner_1, corner_1, 1.],
            [corner_2, corner_2, corner_2, 1.],
            [corner_3, corner_3, corner_3, 1.],
            [corner_4, corner_4, corner_4, 1.],
        ]
    }

    pub fn turn_quad(&self) -> bool {
        self.corner_1 + self.corner_3 > self.corner_2 + self.corner_4
    }
}
