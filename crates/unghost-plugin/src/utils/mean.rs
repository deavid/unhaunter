#[derive(Default)]
pub(crate) struct MeanValue {
    pub mean: f32,
    pub len: f32,
}

impl MeanValue {
    pub(crate) fn push_len(&mut self, val: f32, len: f32) {
        if len > 0.0 {
            self.mean = (self.mean * self.len + val * len) / (self.len + len);
            self.len += len;
        }
    }

    pub(crate) fn avg(&mut self) -> f32 {
        self.len = 0.0000001;
        self.mean
    }
}
