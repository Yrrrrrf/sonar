// C:\...\sonar\src\stack\datalink\mod.rs

use crate::modem::ModemTrait;
use dev_utils::{debug, info, trace, warn};
use std::error::Error;

// --- Constants ---
const BITS_PER_CHARACTER: usize = 10;
const LEADER_TONE_CHARS: usize = 5;

// --- Public API ---

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
    // When in tracking mode, this stores the expected start of the next frame.
    // It's an Option because we start in searching mode (None).
    next_frame_start_pos: Option<usize>,
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
            next_frame_start_pos: None,
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

        let mut current_pos_f32: f32 = 0.0;
        for i in 0..BITS_PER_CHARACTER {
            let start = current_pos_f32.round() as usize;
            let end = (current_pos_f32 + samples_per_bit).round() as usize;
            current_pos_f32 += samples_per_bit;

            if end > frame_samples.len() { return (0.0, 0); }

            if let Ok((mark_energy, space_energy)) = self.modem.analyze_bit(&frame_samples[start..end]) {
                bits[i] = mark_energy > space_energy;
                signals[i] = if bits[i] { mark_energy } else { space_energy };
            } else {
                return (0.0, 0);
            }
        }

        if bits[0] /*start*/ || !bits[BITS_PER_CHARACTER - 1] /*stop*/ {
            return (0.0, 0);
        }
        
        // --- Enhanced Confidence Calculation ---
        let mut avg_mark_signal = 0.0;
        let mut mark_count = 0;
        let mut avg_space_signal = 0.0;
        let mut space_count = 0;

        for i in 0..BITS_PER_CHARACTER {
            if bits[i] {
                avg_mark_signal += signals[i];
                mark_count += 1;
            } else {
                avg_space_signal += signals[i];
                space_count += 1;
            }
        }

        if mark_count > 0 { avg_mark_signal /= mark_count as f32; }
        if space_count > 0 { avg_space_signal /= space_count as f32; }

        let mut total_divergence = 0.0;
        for i in 0..BITS_PER_CHARACTER {
            let avg_for_bit = if bits[i] { avg_mark_signal } else { avg_space_signal };
            let divergence = (signals[i] - avg_for_bit).abs() / (avg_for_bit + f32::EPSILON);
            total_divergence += divergence;
        }
        let normalized_divergence = total_divergence / BITS_PER_CHARACTER as f32;
        
        let total_signal: f32 = signals.iter().sum();
        // Here, "noise" is the energy of the opposite frequency, which we don't have directly.
        // So we use a simplified SNR where consistency is the primary factor.
        let confidence = (1.0 - normalized_divergence).max(0.0) * total_signal.sqrt();
        
        let mut byte = 0u8;
        let data_bits = &bits[1..9];
        for i in 0..8 {
            if data_bits[i] {
                byte |= 1 << i;
            }
        }
        (confidence, byte)
    }
}

impl CodecTrait for SonarCodec {
    fn encode(&self, payload: &[u8]) -> Result<Vec<f32>, Box<dyn Error>> {
        let mut bitstream = Vec::new();
        for _ in 0..(LEADER_TONE_CHARS * BITS_PER_CHARACTER) { bitstream.push(true); }
        for &byte in payload {
            bitstream.push(false); // Start bit
            for i in 0..8 { bitstream.push((byte >> i) & 1 == 1); }
            bitstream.push(true); // Stop bit
        }
        self.modem.modulate(&bitstream)
    }

    fn decode(&mut self, samples: &[f32]) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
        self.audio_buffer.extend_from_slice(samples);
        let samples_per_char = self.samples_per_character();
        let mut found_bytes = Vec::new();

        loop {
            let (search_window, search_offset) = match self.next_frame_start_pos {
                Some(pos) => ((self.samples_per_bit() * 0.5).round() as usize, pos),
                None => ((self.samples_per_bit() * 1.5).round() as usize, 0),
            };
            
            if search_offset + search_window + samples_per_char > self.audio_buffer.len() {
                break; // Not enough data to conduct a search
            }

            let mut best_confidence = 0.0;
            let mut best_byte = 0;
            let mut best_frame_start_pos = 0;

            for offset in 0..search_window {
                let current_pos = search_offset + offset;
                let frame_window = &self.audio_buffer[current_pos..(current_pos + samples_per_char)];
                let (confidence, byte) = self.analyze_character_frame(frame_window);
                if confidence > best_confidence {
                    best_confidence = confidence;
                    best_byte = byte;
                    best_frame_start_pos = current_pos;
                }
            }

            if best_confidence > self.config.confidence_threshold {
                if !self.is_receiving {
                    warn!("--- SIGNAL DETECTED (Confidence: {:.2}) ---", best_confidence);
                    self.is_receiving = true;
                }
                info!("CHARACTER FOUND! Byte: 0x{:02X} ('{}'), Confidence: {:.2}", best_byte, if (best_byte as char).is_ascii_graphic() { best_byte as char } else { '.' }, best_confidence);
                found_bytes.push(best_byte);

                let samples_to_drain = best_frame_start_pos + samples_per_char;
                self.audio_buffer.drain(..samples_to_drain);
                self.next_frame_start_pos = Some(0);
            } else {
                if self.next_frame_start_pos.is_some() {
                    // We were tracking but lost the signal.
                    self.reset_state();
                }
                // Discard some data to prevent getting stuck on noise
                let samples_to_drain = (samples_per_char / 2).max(1);
                self.audio_buffer.drain(..samples_to_drain);
                break;
            }
        }

        Ok(if found_bytes.is_empty() { None } else { Some(found_bytes) })
    }

    fn reset_state(&mut self) {
        if self.is_receiving {
            warn!("--- SIGNAL LOST (Timeout/Error) ---");
            self.is_receiving = false;
            self.next_frame_start_pos = None;
        }
    }
}