use crate::random_seed;
use crate::types::evidence::Evidence;
use bevy::prelude::*;
use rand::Rng;
use std::fmt::Debug;

/// Per-ghost randomized noise offsets for unique behavior patterns
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoiseOffsets {
    // Evidence-specific offsets
    pub freezing_temp_x: f32,
    pub freezing_temp_y: f32,
    pub floating_orbs_x: f32,
    pub floating_orbs_y: f32,
    pub uv_ectoplasm_x: f32,
    pub uv_ectoplasm_y: f32,
    pub emf_level5_x: f32,
    pub emf_level5_y: f32,
    pub evp_recording_x: f32,
    pub evp_recording_y: f32,
    pub spirit_box_x: f32,
    pub spirit_box_y: f32,
    pub rl_presence_x: f32,
    pub rl_presence_y: f32,
    pub cpm500_x: f32,
    pub cpm500_y: f32,

    // Behavioral multiplier offsets
    pub visual_alpha_multiplier_x: f32,
    pub visual_alpha_multiplier_y: f32,
    pub rage_tendency_multiplier_x: f32,
    pub rage_tendency_multiplier_y: f32,
}

impl NoiseOffsets {
    /// Generate random noise offsets for a ghost
    pub fn new_random() -> Self {
        let mut rng = random_seed::rng();
        Self {
            freezing_temp_x: rng.random_range(0.0..100.0),
            freezing_temp_y: rng.random_range(0.0..100.0),
            floating_orbs_x: rng.random_range(0.0..100.0),
            floating_orbs_y: rng.random_range(0.0..100.0),
            uv_ectoplasm_x: rng.random_range(0.0..100.0),
            uv_ectoplasm_y: rng.random_range(0.0..100.0),
            emf_level5_x: rng.random_range(0.0..100.0),
            emf_level5_y: rng.random_range(0.0..100.0),
            evp_recording_x: rng.random_range(0.0..100.0),
            evp_recording_y: rng.random_range(0.0..100.0),
            spirit_box_x: rng.random_range(0.0..100.0),
            spirit_box_y: rng.random_range(0.0..100.0),
            rl_presence_x: rng.random_range(0.0..100.0),
            rl_presence_y: rng.random_range(0.0..100.0),
            cpm500_x: rng.random_range(0.0..100.0),
            cpm500_y: rng.random_range(0.0..100.0),
            visual_alpha_multiplier_x: rng.random_range(0.0..100.0),
            visual_alpha_multiplier_y: rng.random_range(0.0..100.0),
            rage_tendency_multiplier_x: rng.random_range(0.0..100.0),
            rage_tendency_multiplier_y: rng.random_range(0.0..100.0),
        }
    }

    /// Get noise offsets for a specific evidence type
    pub fn get_evidence_offsets(&self, evidence: Evidence) -> (f32, f32) {
        match evidence {
            Evidence::FreezingTemp => (self.freezing_temp_x, self.freezing_temp_y),
            Evidence::FloatingOrbs => (self.floating_orbs_x, self.floating_orbs_y),
            Evidence::UVEctoplasm => (self.uv_ectoplasm_x, self.uv_ectoplasm_y),
            Evidence::EMFLevel5 => (self.emf_level5_x, self.emf_level5_y),
            Evidence::EVPRecording => (self.evp_recording_x, self.evp_recording_y),
            Evidence::SpiritBox => (self.spirit_box_x, self.spirit_box_y),
            Evidence::RLPresence => (self.rl_presence_x, self.rl_presence_y),
            Evidence::CPM500 => (self.cpm500_x, self.cpm500_y),
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct GhostBehaviorDynamics {
    pub freezing_temp_clarity: f32,
    pub floating_orbs_clarity: f32,
    pub uv_ectoplasm_clarity: f32,
    pub emf_level5_clarity: f32,
    pub evp_recording_clarity: f32,
    pub spirit_box_clarity: f32,
    pub rl_presence_clarity: f32,
    pub cpm500_clarity: f32,
    pub visual_alpha_multiplier: f32,
    pub rage_tendency_multiplier: f32,
    pub noise_offsets: NoiseOffsets,
}

impl Default for GhostBehaviorDynamics {
    fn default() -> Self {
        Self {
            freezing_temp_clarity: 1.0,
            floating_orbs_clarity: 1.0,
            uv_ectoplasm_clarity: 1.0,
            emf_level5_clarity: 1.0,
            evp_recording_clarity: 1.0,
            spirit_box_clarity: 1.0,
            rl_presence_clarity: 1.0,
            cpm500_clarity: 1.0,
            visual_alpha_multiplier: 1.0,
            rage_tendency_multiplier: 1.0,
            noise_offsets: NoiseOffsets::new_random(),
        }
    }
}

impl GhostBehaviorDynamics {
    pub fn get_clarity(&self, evidence: Evidence) -> f32 {
        match evidence {
            Evidence::FreezingTemp => self.freezing_temp_clarity,
            Evidence::FloatingOrbs => self.floating_orbs_clarity,
            Evidence::UVEctoplasm => self.uv_ectoplasm_clarity,
            Evidence::EMFLevel5 => self.emf_level5_clarity,
            Evidence::EVPRecording => self.evp_recording_clarity,
            Evidence::SpiritBox => self.spirit_box_clarity,
            Evidence::RLPresence => self.rl_presence_clarity,
            Evidence::CPM500 => self.cpm500_clarity,
        }
    }

    pub fn set_clarity(&mut self, evidence: Evidence, value: f32) {
        match evidence {
            Evidence::FreezingTemp => self.freezing_temp_clarity = value,
            Evidence::FloatingOrbs => self.floating_orbs_clarity = value,
            Evidence::UVEctoplasm => self.uv_ectoplasm_clarity = value,
            Evidence::EMFLevel5 => self.emf_level5_clarity = value,
            Evidence::EVPRecording => self.evp_recording_clarity = value,
            Evidence::SpiritBox => self.spirit_box_clarity = value,
            Evidence::RLPresence => self.rl_presence_clarity = value,
            Evidence::CPM500 => self.cpm500_clarity = value,
        }
    }
}
