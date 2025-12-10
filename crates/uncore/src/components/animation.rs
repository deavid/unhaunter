use bevy::prelude::*;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub enum CharacterAnimationDirection {
    NN,
    NW,
    WW,
    SW,
    SS,
    SE,
    EE,
    NE,
}

impl CharacterAnimationDirection {
    fn from_dir(dx: f32, dy: f32) -> Self {
        let dst = (dx * dx + dy * dy).sqrt() + 0.0000000001;
        let dx = (dx / dst).round() as i32;
        let dy = (dy / dst).round() as i32;
        match dx {
            1 => match dy {
                1 => Self::NE,
                -1 => Self::SE,
                _ => Self::EE,
            },
            0 => match dy {
                1 => Self::NN,
                -1 => Self::SS,
                _ => Self::SS,
            },
            -1 => match dy {
                1 => Self::NW,
                -1 => Self::SW,
                _ => Self::WW,
            },
            _ => Self::EE,
        }
    }

    fn to_delta_idx(self) -> usize {
        match self {
            Self::NN => 0,
            Self::NW => 1,
            Self::WW => 2,
            Self::SW => 3,
            Self::SS => 16,
            Self::SE => 17,
            Self::EE => 18,
            Self::NE => 19,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CharacterAnimationState {
    Standing,
    Walking,
}

impl CharacterAnimationState {
    fn to_delta_idx(self) -> usize {
        match self {
            Self::Standing => 32,
            Self::Walking => 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CharacterAnimation {
    pub state: CharacterAnimationState,
    pub dir: CharacterAnimationDirection,
}

impl CharacterAnimation {
    pub fn from_dir(dx: f32, dy: f32) -> Self {
        let dst = (dx * dx + dy * dy).sqrt() + 0.001;
        let state = if dst > 1.0 {
            CharacterAnimationState::Walking
        } else {
            CharacterAnimationState::Standing
        };
        let dir = CharacterAnimationDirection::from_dir(dx, dy);
        Self { state, dir }
    }

    pub fn to_vec(self) -> Vec<usize> {
        let i = self.state.to_delta_idx() + self.dir.to_delta_idx();
        vec![i, i + 4, i + 8, i + 12]
    }
}

#[derive(Component)]
pub struct AnimationTimer {
    timer: Timer,
    frames: Vec<usize>,
    idx: usize,
}

impl AnimationTimer {
    pub fn from_range<I: IntoIterator<Item = usize>>(timer: Timer, range: I) -> Self {
        let frames: Vec<usize> = range.into_iter().collect();
        AnimationTimer {
            timer,
            frames,
            idx: 0,
        }
    }

    pub fn set_range<I: IntoIterator<Item = usize>>(&mut self, range: I) {
        self.frames = range.into_iter().collect();
    }

    pub fn tick(&mut self, delta: Duration) -> Option<usize> {
        self.timer.tick(delta);
        if !self.timer.just_finished() {
            return None;
        }
        self.idx = (self.idx + 1) % self.frames.len();
        Some(self.frames[self.idx])
    }
}
