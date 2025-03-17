//! AudioMix is a struct that holds the audio data in a format that can be used by the Bevy audio system.
//! This is useful when using the rodio effects over AudioSource which return a source. AudioMix is an
//! asset that also implements Decoder, so it should be possible to use as AudioPlayer::<AudioMix>.

use bevy::audio::Source;
use bevy::prelude::*;
use std::{sync::Arc, time::Duration};

#[derive(Asset, Reflect, Clone, Debug)]
pub struct AudioMix {
    pub source_data: Arc<[i16]>,
    pub channels: u16,
    pub sample_rate: u32,
    pub duration: Option<Duration>,
    pub current_frame_len: Option<usize>,
}

impl AudioMix {
    pub fn from_source(mut src: impl Source<Item = i16>) -> Self {
        let channels = src.channels();
        let sample_rate = src.sample_rate();
        let duration = src.total_duration();
        let current_frame_len = src.current_frame_len();
        let mut vec_data: Vec<i16> = vec![];
        // Collecting the vector causes rodion to crash on iter size hint. So instead we do it one at a time.
        for sample in src.by_ref() {
            vec_data.push(sample);
        }
        let source_data: Arc<[i16]> = vec_data.into();
        AudioMix {
            source_data,
            channels,
            sample_rate,
            duration,
            current_frame_len,
        }
    }
}

pub struct AudioMixDecoder {
    pub audio_mix: AudioMix,
    pub iter_pos: usize,
}

impl AudioMixDecoder {
    pub fn new(audio_mix: &AudioMix) -> Self {
        AudioMixDecoder {
            audio_mix: audio_mix.clone(),
            iter_pos: 0,
        }
    }
}

impl Iterator for AudioMixDecoder {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sample) = self.audio_mix.source_data.get(self.iter_pos) {
            self.iter_pos += 1;
            Some(*sample)
        } else {
            None
        }
    }
}

impl Source for AudioMixDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        Some(
            self.audio_mix
                .source_data
                .len()
                .saturating_sub(self.iter_pos),
        )
    }

    fn channels(&self) -> u16 {
        self.audio_mix.channels
    }

    fn sample_rate(&self) -> u32 {
        self.audio_mix.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        self.audio_mix.duration
    }
}

impl Decodable for AudioMix {
    type DecoderItem = i16;

    type Decoder = AudioMixDecoder;

    fn decoder(&self) -> Self::Decoder {
        AudioMixDecoder::new(self)
    }
}
