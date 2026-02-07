use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct ExposureModel {
    pub history: VecDeque<f32>,
    pub weights: Vec<f32>,
    pub lux: f32,
    pub current: f32,
}

impl ExposureModel {
    pub fn new() -> Self {
        let mut history = VecDeque::new();
        let mut weights = Vec::<f32>::new();
        const N: usize = 240;
        for i in 0..N {
            history.push_back(1.0);
            let weight =
                0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (N as f32 - 1.0)).cos());
            weights.push(weight);
        }
        Self {
            history,
            weights,
            lux: 1.0,
            current: 1.0,
        }
    }

    pub fn add_sample(&mut self, sample: f32) {
        self.history.push_back(sample);
        while self.history.len() > 240 {
            self.history.pop_front();
        }

        let mut sum_weights = 0.0;
        let mut sum_values = 0.0;
        for (i, &v) in self.history.iter().enumerate() {
            let weight = self.weights.get(i).copied().unwrap_or(1.0);
            sum_values += v * weight;
            sum_weights += weight;
        }
        self.lux = sum_values / (sum_weights + 0.001);
    }

    pub fn update(&mut self, dt: f32) {
        let mut target = self.lux;
        // Partial eye adaptation: sqrt-ish curve that allows more adaptation in dark
        target = target.clamp(0.0, 100.0).powf(0.5) * 1.5 - 0.05;
        target = target.clamp(0.05, 10.0);

        if !target.is_normal() {
            target = self.current;
        }

        // Additional IIR filter: 63.2% in 2 seconds (tau = 2s)
        let alpha = 1.0 - (-dt / 2.0).exp();
        self.current = self.current * (1.0 - alpha) + target * alpha;
    }
}

impl Default for ExposureModel {
    fn default() -> Self {
        Self::new()
    }
}
