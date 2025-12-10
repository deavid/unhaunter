#[derive(Default)]
pub struct MeanValue {
    pub mean: f32,
    pub len: f32,
}

impl MeanValue {
    pub fn _push(&mut self, val: f32) {
        self.push_len(val, 1.0)
    }

    pub fn push_len(&mut self, val: f32, len: f32) {
        if len > 0.0 {
            self.mean = (self.mean * self.len + val * len) / (self.len + len);
            self.len += len;
        }
    }

    pub fn avg(&mut self) -> f32 {
        self.len = 0.0000001;
        self.mean
    }
}
