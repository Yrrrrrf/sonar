// C:\...\sonar\src\stack\datalink\mod.rs

use crate::modem::ModemTrait;
use dev_utils::{debug, info, trace, warn};
use std::error::Error;

// --- Constants ---
// A character frame is 1 start bit + 8 data bits + 1 stop bit = 10 bits total.
const BITS_PER_CHARACTER: usize = 10;
// A leader tone of mark signals to help the receiver's AGC and level detection stabilize.
const LEADER_TONE_CHARS: usize = 5;

// --- Public API ---

/// The primary trait for encoding and decoding data using an underlying modem.
pub trait CodecTrait {
    /// Encodes a byte payload into a vector of audio samples.
    fn encode(&self, payload: &[u8]) -> Result<Vec<f32>, Box<dyn Error>>;

    /// Decodes a stream of audio samples, returning any complete bytes found.
    /// This method is stateful and should be called repeatedly with new samples.
    fn decode(&mut self, samples: &[f32]) -> Result<Option<Vec<u8>>, Box<dyn Error>>;
}

/// The SonarCodec implements a robust, confidence-based acoustic modem protocol.
pub struct SonarCodec {
    modem: Box<dyn ModemTrait>,
    config: SonarCodecConfig,
    audio_buffer: Vec<f32>,
}

/// Configuration for the SonarCodec.
#[derive(Debug, Clone, Copy)]
pub struct SonarCodecConfig {
    pub sample_rate: u32,
    pub baud_rate: u32,
    pub confidence_threshold: f32,
}

impl SonarCodec {
    /// Creates a new SonarCodec with a given modem and configuration.
    pub fn new(modem: Box<dyn ModemTrait>, config: SonarCodecConfig) -> Self {
        Self {
            modem,
            config,
            // Pre-allocate a buffer. A 2-second buffer is a reasonable start.
            audio_buffer: Vec::with_capacity((config.sample_rate * 2) as usize),
        }
    }

    /// Calculates the number of audio samples required to represent one bit.
    fn samples_per_bit(&self) -> f32 {
        self.config.sample_rate as f32 / self.config.baud_rate as f32
    }

    /// Calculates the number of audio samples for a full 10-bit character frame.
    fn samples_per_character(&self) -> usize {
        (self.samples_per_bit() * BITS_PER_CHARACTER as f32).round() as usize
    }

    /// Analyzes a slice of audio samples corresponding to one character frame.
    ///
    /// This is the core of the confidence-based decoder. It checks for correct
    /// start/stop bit framing and calculates a confidence score based on the
    /// signal-to-noise ratio and consistency of the bits.
    ///
    /// Returns a tuple: (confidence_score: f32, decoded_byte: u8).
    /// A confidence of 0.0 indicates an invalid or unrecognized frame.
    fn analyze_character_frame(&self, frame_samples: &[f32]) -> (f32, u8) {
        let samples_per_bit = self.samples_per_bit();
        let samples_per_bit_usize = samples_per_bit.round() as usize;

        let mut bits = [false; BITS_PER_CHARACTER];
        let mut signals = [0.0; BITS_PER_CHARACTER];
        let mut noises = [0.0; BITS_PER_CHARACTER];

        let mut bits_for_trace = String::new(); // For logging

        // 1. Demodulate each bit in the frame to get its value, signal, and noise levels.
        for i in 0..BITS_PER_CHARACTER {
            let start = (i as f32 * samples_per_bit).round() as usize;
            let end = start + samples_per_bit_usize;
            if end > frame_samples.len() {
                return (0.0, 0); // Not enough samples
            }

            if let Ok((mark_energy, space_energy)) =
                self.modem.analyze_bit(&frame_samples[start..end])
            {
                if mark_energy > space_energy {
                    bits[i] = true; // Mark (1)
                    signals[i] = mark_energy;
                    noises[i] = space_energy;
                    bits_for_trace.push('1'); // For logging
                } else {
                    bits[i] = false; // Space (0)
                    signals[i] = space_energy;
                    noises[i] = mark_energy;
                    bits_for_trace.push('0'); // For logging
                }
            } else {
                return (0.0, 0); // Modem error
            }
        }

        // --- ADDED TRACE LOG ---
        trace!(
            "Analyzing frame. Bits: {}, Signal[0]: {:.4}, Noise[0]: {:.4}",
            bits_for_trace,
            signals[0],
            noises[0]
        );

        // 2. Perform the critical framing check.
        // A valid frame must have a 'space' (0) start bit and a 'mark' (1) stop bit.
        // The start bit is the first bit, the stop bit is the last.
        let start_bit = bits[0];
        let stop_bit = bits[BITS_PER_CHARACTER - 1];

        if start_bit == true || stop_bit == false {
            trace!(" -> Frame REJECTED (Bad start/stop bits)"); // ADDED TRACE
            return (0.0, 0);
        }

        // 3. Calculate confidence score.
        let total_signal: f32 = signals.iter().sum();
        let total_noise: f32 = noises.iter().sum();

        // The overall signal-to-noise ratio for the frame.
        let snr = if total_noise > f32::EPSILON {
            total_signal / total_noise
        } else {
            1000.0 // Practically infinite SNR if noise is near zero.
        };

        // For now, our confidence is simply the SNR. More complex metrics
        // (like amplitude consistency) could be added later.
        let confidence = snr;

        // --- ADDED TRACE LOG ---
        trace!(" -> Frame ACCEPTED. Confidence: {:.2}", confidence);

        // 4. Assemble the 8 data bits (from index 1 to 8) into a byte.
        let mut byte = 0u8;
        for i in 1..=8 {
            if bits[i] {
                byte |= 1 << (i - 1);
            }
        }

        (confidence, byte)
    }
}

impl CodecTrait for SonarCodec {
    /// Encodes a payload into a stream of audio samples.
    /// This now includes a leader tone and per-character framing.
    fn encode(&self, payload: &[u8]) -> Result<Vec<f32>, Box<dyn Error>> {
        let mut bitstream = Vec::new();

        // 1. Add a leader tone (a series of mark bits) to stabilize the receiver.
        for _ in 0..(LEADER_TONE_CHARS * BITS_PER_CHARACTER) {
            bitstream.push(true); // Mark bit
        }

        // 2. For each byte, create a 10-bit character frame.
        for &byte in payload {
            // Start bit (space)
            bitstream.push(false);
            // 8 Data bits, LSB first
            for i in 0..8 {
                bitstream.push((byte >> i) & 1 == 1);
            }
            // Stop bit (mark)
            bitstream.push(true);
        }

        // 3. Modulate the entire bitstream into an audio signal.
        self.modem.modulate(&bitstream)
    }

    /// The new `decode` method, implementing the `FrameFinder` logic.
    fn decode(&mut self, samples: &[f32]) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
        // Add incoming audio to our internal buffer.
        self.audio_buffer.extend_from_slice(samples);
        trace!(
            "Added {} samples to buffer. New size: {}",
            samples.len(),
            self.audio_buffer.len()
        );

        let samples_per_char = self.samples_per_character();
        let mut found_bytes = Vec::new();

        // Keep processing the buffer as long as we have enough data for a character.
        loop {
            if self.audio_buffer.len() < samples_per_char {
                trace!("Buffer too small for a full character. Waiting for more data.");
                break; // Not enough data, wait for more.
            }

            // The 'FrameFinder' logic:
            // We search a small window of samples to find the best frame alignment.
            // This makes the decoder robust to minor timing drift.
            let search_window_samples = (self.samples_per_bit() * 1.5).round() as usize;

            let mut best_confidence = 0.0;
            let mut best_byte = 0;
            let mut best_frame_start_pos = 0;

            // Slide a window across the start of the buffer to find the best frame.
            for start_pos in 0..search_window_samples {
                if start_pos + samples_per_char > self.audio_buffer.len() {
                    break;
                }

                let window = &self.audio_buffer[start_pos..(start_pos + samples_per_char)];
                let (confidence, byte) = self.analyze_character_frame(window);

                if confidence > best_confidence {
                    best_confidence = confidence;
                    best_byte = byte;
                    best_frame_start_pos = start_pos;
                }
            }

            // --- ADDED DEBUG LOG ---
            debug!(
                "FrameFinder result: Best confidence {:.2} found at pos {}. Threshold is {}.",
                best_confidence, best_frame_start_pos, self.config.confidence_threshold
            );

            // Check if our best find is good enough.
            if best_confidence > self.config.confidence_threshold {
                // We found a valid character!
                info!(
                    "CHARACTER FOUND! Byte: 0x{:02X} ('{}'), Confidence: {:.2}",
                    best_byte,
                    if (best_byte as char).is_ascii_graphic() {
                        best_byte as char
                    } else {
                        '.'
                    },
                    best_confidence
                );
                found_bytes.push(best_byte);

                // Drain the used samples from the buffer. This includes the junk
                // before the frame and the frame itself.
                let samples_to_drain = best_frame_start_pos + samples_per_char;
                self.audio_buffer.drain(..samples_to_drain);
                trace!(
                    "Drained {} samples. Buffer size now: {}",
                    samples_to_drain,
                    self.audio_buffer.len()
                );
            } else {
                // No character found in the current window. Drain a small
                // amount of data to avoid getting stuck on noise and try again.
                let samples_to_drain = (samples_per_char / 2).max(1);
                self.audio_buffer.drain(..samples_to_drain);
                debug!(
                    "No confident frame found. Discarding {} samples to search again.",
                    samples_to_drain
                );
                break;
            }
        }

        if found_bytes.is_empty() {
            Ok(None)
        } else {
            Ok(Some(found_bytes))
        }
    }
}