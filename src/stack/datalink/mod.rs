// C:\...\sonar\src\stack\datalink\mod.rs

use crate::modem::ModemTrait;
use dev_utils::{debug, info, trace, warn};
use std::error::Error;

const BITS_PER_CHARACTER: usize = 10;
const LEADER_TONE_CHARS: usize = 5;

pub trait CodecTrait {
    fn encode(&self, payload: &[u8]) -> Result<Vec<f32>, Box<dyn Error>>;
    fn decode(&mut self, samples: &[f32]) -> Result<Option<Vec<u8>>, Box<dyn Error>>;
    fn reset_state(&mut self);
}

pub struct SonarCodec {
    modem: Box<dyn ModemTrait>,
    config: SonarCodecConfig,
    audio_buffer: Vec<f32>,
    is_receiving: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct SonarCodecConfig {
    pub sample_rate: u32,
    pub baud_rate: u32,
    pub confidence_threshold: f32,
}

impl SonarCodec {
    pub fn new(modem: Box<dyn ModemTrait>, config: SonarCodecConfig) -> Self {
        Self {
            modem,
            config,
            audio_buffer: Vec::with_capacity((config.sample_rate * 2) as usize),
            is_receiving: false,
        }
    }

    fn samples_per_bit(&self) -> f32 {
        self.config.sample_rate as f32 / self.config.baud_rate as f32
    }

    fn samples_per_character(&self) -> usize {
        (self.samples_per_bit() * BITS_PER_CHARACTER as f32).round() as usize
    }

    fn analyze_character_frame(&self, frame_samples: &[f32]) -> (f32, u8) {
        let samples_per_bit = self.samples_per_bit();
        let mut bits = [false; BITS_PER_CHARACTER];
        let mut signals = [0.0; BITS_PER_CHARACTER];
        let mut noises = [0.0; BITS_PER_CHARACTER];

        let mut current_pos_f32: f32 = 0.0;
        for i in 0..BITS_PER_CHARACTER {
            let start = current_pos_f32.round() as usize;
            let end = (current_pos_f32 + samples_per_bit).round() as usize;
            current_pos_f32 += samples_per_bit;

            if end > frame_samples.len() { return (0.0, 0); }

            if let Ok((mark_energy, space_energy)) = self.modem.analyze_bit(&frame_samples[start..end]) {
                if mark_energy > space_energy {
                    bits[i] = true;
                    signals[i] = mark_energy;
                    noises[i] = space_energy;
                } else {
                    bits[i] = false;
                    signals[i] = space_energy;
                    noises[i] = mark_energy;
                }
            } else {
                return (0.0, 0);
            }
        }

        if bits[0] || !bits[BITS_PER_CHARACTER - 1] { return (0.0, 0); }

        let mut avg_mark_signal = 0.0;
        let mut mark_count = 0;
        let mut avg_space_signal = 0.0;
        let mut space_count = 0;

        for i in 0..BITS_PER_CHARACTER {
            if bits[i] { avg_mark_signal += signals[i]; mark_count += 1; } 
            else { avg_space_signal += signals[i]; space_count += 1; }
        }

        if mark_count > 0 { avg_mark_signal /= mark_count as f32; }
        if space_count > 0 { avg_space_signal /= space_count as f32; }

        let mut total_divergence = 0.0;
        for i in 0..BITS_PER_CHARACTER {
            let avg_for_bit = if bits[i] { avg_mark_signal } else { avg_space_signal };
            total_divergence += (signals[i] - avg_for_bit).abs() / (avg_for_bit + f32::EPSILON);
        }
        let normalized_divergence = total_divergence / BITS_PER_CHARACTER as f32;

        let total_signal: f32 = signals.iter().sum();
        let total_noise: f32 = noises.iter().sum();
        let snr = total_signal / (total_noise + f32::EPSILON);
        let confidence = snr * (1.0 - normalized_divergence).max(0.0);
        
        let mut byte = 0u8;
        for i in 0..8 { if bits[i + 1] { byte |= 1 << i; } }
        
        (confidence, byte)
    }
}

impl CodecTrait for SonarCodec {
    fn encode(&self, payload: &[u8]) -> Result<Vec<f32>, Box<dyn Error>> {
        let mut bitstream = Vec::new();
        for _ in 0..(LEADER_TONE_CHARS * BITS_PER_CHARACTER) { bitstream.push(true); }
        for &byte in payload {
            bitstream.push(false);
            for i in 0..8 { bitstream.push((byte >> i) & 1 == 1); }
            bitstream.push(true);
        }
        self.modem.modulate(&bitstream)
    }

    fn decode(&mut self, samples: &[f32]) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
        self.audio_buffer.extend_from_slice(samples);
        let samples_per_char = self.samples_per_character();
        let mut found_bytes = Vec::new();
        
        let mut current_search_offset = 0;

        loop {
            let search_window_size = if self.is_receiving {
                // TRACKING MODE: Narrow search around the expected position
                (self.samples_per_bit() * 0.5).round() as usize
            } else {
                // SEARCHING MODE: Wide search to find the first signal
                (self.samples_per_bit() * 1.5).round() as usize
            };

            let search_area_end = current_search_offset + search_window_size + samples_per_char;
            if search_area_end > self.audio_buffer.len() {
                break; // Not enough data to conduct a full search from our current position
            }

            let mut best_confidence = 0.0;
            let mut best_byte = 0;
            let mut best_frame_start_pos = 0;

            for offset in 0..search_window_size {
                let pos = current_search_offset + offset;
                let frame_window = &self.audio_buffer[pos..(pos + samples_per_char)];
                let (confidence, byte) = self.analyze_character_frame(frame_window);
                if confidence > best_confidence {
                    best_confidence = confidence;
                    best_byte = byte;
                    best_frame_start_pos = pos;
                }
            }

            if best_confidence > self.config.confidence_threshold {
                if !self.is_receiving {
                    warn!("--- SIGNAL DETECTED (Confidence: {:.2}) ---", best_confidence);
                    self.is_receiving = true;
                }
                info!("CHARACTER FOUND! Byte: 0x{:02X} ('{}'), Confidence: {:.2}", best_byte, if (best_byte as char).is_ascii_graphic() { best_byte as char } else { '.' }, best_confidence);
                found_bytes.push(best_byte);

                // The next search should start exactly one character's length after this one started.
                current_search_offset = best_frame_start_pos + samples_per_char;

            } else {
                // No character found in this search window.
                // We're done for this call to decode().
                // First, drain the audio we just fruitlessly searched.
                let drain_end = (current_search_offset + search_window_size).min(self.audio_buffer.len());
                self.audio_buffer.drain(..drain_end);
                break;
            }
        }

        // If we found at least one character, we must drain the buffer up to where the next character should start.
        if current_search_offset > 0 {
             self.audio_buffer.drain(..current_search_offset);
        }

        Ok(if found_bytes.is_empty() { None } else { Some(found_bytes) })
    }

    fn reset_state(&mut self) {
        if self.is_receiving {
            warn!("--- SIGNAL LOST (Timeout) ---");
            self.is_receiving = false;
        }
    }
}