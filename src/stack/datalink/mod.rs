// src/stack/datalink/mod.rs

use crate::modem::{ModemTrait, fsk::FSK};
use dev_utils::{debug, info, trace, warn};
use std::error::Error;

// --- Constants for the new Frame-based approach ---

/// A leader tone of pure mark frequency to help the receiver's audio levels stabilize.
/// Duration is in number of bits. A 16-bit duration is a good starting point.
const LEADER_TONE_BITS: usize = 16;

/// A trailer tone (also mark frequency) to signify the end of transmission.
const TRAILER_TONE_BITS: usize = 8;

// --- Public API & Main Codec Struct ---

/// The primary trait for encoding/decoding data payloads into audio signals.
pub trait CodecTrait {
    /// Encodes a byte payload into a complete audio signal (Vec<f32>).
    fn encode<T: AsRef<[u8]>>(&self, payload: T) -> Result<Vec<f32>, Box<dyn Error>>;

    /// Processes a new chunk of audio samples and attempts to decode bytes.
    /// Returns `Ok(Some(Vec<u8>))` if one or more bytes were successfully decoded.
    /// Returns `Ok(None)` if no complete bytes were found in the current buffer.
    fn decode(&mut self, samples: &[f32]) -> Result<Option<Vec<u8>>, Box<dyn Error>>;
}

/// The main struct that implements the `CodecTrait` using the robust,
/// confidence-based decoding strategy inspired by `minimodem`.
pub struct SonarCodec {
    modem: Box<dyn ModemTrait>,
    sample_buffer: Vec<f32>,
    pub confidence_threshold: f32, // The "squelch" level for decoding.
    // Configuration for the character frame structure
    start_bits: usize,
    data_bits: usize,
    stop_bits: usize,
}

// --- Implementation ---

impl SonarCodec {
    /// Creates a new `SonarCodec`.
    ///
    /// # Arguments
    /// * `modem` - A boxed `ModemTrait` object (e.g., `Box::new(FSK::default())`).
    pub fn new(modem: Box<dyn ModemTrait>) -> Self {
        Self {
            modem,
            sample_buffer: Vec::with_capacity(48000 * 2), // Buffer for 2s of audio
            confidence_threshold: 1.5, // A reasonable default, similar to minimodem
            start_bits: 1,
            data_bits: 8,
            stop_bits: 1,
        }
    }

    /// Resets the decoder's internal buffer. Useful for clearing state after an error.
    pub fn reset_decoder(&mut self) {
        self.sample_buffer.clear();
        info!("Decoder buffer has been reset.");
    }

    /// The core signal processing logic that analyzes a single character frame's worth
    /// of audio samples and calculates a confidence score. This is our `FrameAnalyzer`.
    ///
    /// # Returns
    /// A tuple of `(confidence, decoded_byte)`.
    /// - `confidence`: A score > 0 indicating how likely this is a valid frame.
    /// - `decoded_byte`: The `u8` value of the data bits if confidence is > 0.
    fn analyze_character_frame(&self, frame_samples: &[f32]) -> (f32, u8) {
        let samples_per_bit = self.modem.samples_per_bit();
        let total_bits = self.start_bits + self.data_bits + self.stop_bits;
        let mut data_byte = 0u8;

        let mut total_signal = 0.0;
        let mut total_noise = 0.0;

        // 1. Analyze the Start Bit
        let start_bit_chunk = &frame_samples[0..samples_per_bit];
        let (signal, noise) = self.modem.get_bit_metrics(start_bit_chunk);
        let start_bit = if signal.0 > signal.1 { 1 } else { 0 }; // 0 for space, 1 for mark
        // A valid start bit MUST be a space tone (0).
        if start_bit != 0 {
            return (0.0, 0);
        }
        total_signal += signal.1; // Energy of the expected space tone
        total_noise += noise.1; // Energy of the unexpected mark tone

        // 2. Analyze the 8 Data Bits
        for i in 0..self.data_bits {
            let start = (self.start_bits + i) * samples_per_bit;
            let end = start + samples_per_bit;
            let data_bit_chunk = &frame_samples[start..end];
            let (signal, noise) = self.modem.get_bit_metrics(data_bit_chunk);

            let bit = if signal.0 > signal.1 { 1 } else { 0 };
            data_byte |= bit << i; // LSB first
            total_signal += signal.0.max(signal.1);
            total_noise += noise.0.min(noise.1);
        }

        // 3. Analyze the Stop Bit
        let stop_bit_chunk_start = (self.start_bits + self.data_bits) * samples_per_bit;
        let stop_bit_chunk = &frame_samples[stop_bit_chunk_start..];
        let (signal, noise) = self.modem.get_bit_metrics(stop_bit_chunk);
        let stop_bit = if signal.0 > signal.1 { 1 } else { 0 };
        // A valid stop bit MUST be a mark tone (1).
        if stop_bit != 1 {
            return (0.0, 0);
        }
        total_signal += signal.0; // Energy of the expected mark tone
        total_noise += noise.0; // Energy of the unexpected space tone

        // 4. Calculate Final Confidence Score (SNR-based)
        // Avoid division by zero. If noise is negligible, confidence is effectively infinite.
        let confidence = if total_noise > f32::EPSILON {
            total_signal / total_noise
        } else {
            f32::INFINITY
        };

        (confidence, data_byte)
    }
}

impl CodecTrait for SonarCodec {
    /// Encodes a payload into a full signal with leader and trailer tones.
    fn encode<T: AsRef<[u8]>>(&self, payload: T) -> Result<Vec<f32>, Box<dyn Error>> {
        let payload_bytes = payload.as_ref();
        let samples_per_bit = self.modem.samples_per_bit();

        // Calculate total bits for capacity pre-allocation
        let total_data_frames = payload_bytes.len();
        let bits_per_frame = self.start_bits + self.data_bits + self.stop_bits;
        let total_bits =
            LEADER_TONE_BITS + (total_data_frames * bits_per_frame) + TRAILER_TONE_BITS;
        let mut bitstream = Vec::with_capacity(total_bits);

        // --- Build the Bitstream ---
        // 1. Leader Tone (Mark)
        bitstream.resize(LEADER_TONE_BITS, true);

        // 2. Character Frames
        for &byte in payload_bytes {
            // Start Bit (Space)
            bitstream.extend(vec![false; self.start_bits]);
            // Data Bits (LSB first)
            for i in 0..self.data_bits {
                bitstream.push((byte >> i) & 1 == 1);
            }
            // Stop Bit (Mark)
            bitstream.extend(vec![true; self.stop_bits]);
        }

        // 3. Trailer Tone (Mark)
        bitstream.resize(bitstream.len() + TRAILER_TONE_BITS, true);

        // --- Modulate the complete bitstream into an audio signal ---
        self.modem.modulate(&bitstream)
    }

    /// The `FrameFinder` implementation. It uses a sliding window over the internal
    /// sample buffer to find the character with the highest confidence.
    fn decode(&mut self, samples: &[f32]) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
        self.sample_buffer.extend_from_slice(samples);
        let mut decoded_payload = Vec::new();

        let total_bits_per_frame = self.start_bits + self.data_bits + self.stop_bits;
        let samples_per_frame = self.modem.samples_per_bit() * total_bits_per_frame;

        // Keep processing the buffer as long as there's enough data for a full frame
        'outer: while self.sample_buffer.len() >= samples_per_frame {
            let mut best_confidence = 0.0;
            let mut best_byte = 0;
            let mut best_start_index = 0;

            // The sliding window scans for the best possible frame.
            // We don't need to scan the whole buffer, just a bit's worth of samples,
            // to find the optimal alignment.
            let scan_width = self.modem.samples_per_bit();

            for i in 0..scan_width {
                if i + samples_per_frame > self.sample_buffer.len() {
                    break; // Window would exceed buffer
                }

                let window = &self.sample_buffer[i..i + samples_per_frame];
                let (confidence, byte) = self.analyze_character_frame(window);

                if confidence > best_confidence {
                    best_confidence = confidence;
                    best_byte = byte;
                    best_start_index = i;
                }
            }

            if best_confidence >= self.confidence_threshold {
                // We found a high-confidence character!
                trace!(
                    "Found byte '{}' (0x{:02X}) with confidence {:.2}",
                    best_byte as char, best_byte, best_confidence
                );
                decoded_payload.push(best_byte);

                // Advance the buffer past the consumed frame
                let advance_by = best_start_index + samples_per_frame;
                self.sample_buffer.drain(..advance_by);

                // Continue the 'outer loop to immediately search for the next character
            } else {
                // No high-confidence character found in the scannable area.
                // Discard a small amount of data to prevent an infinite loop on noise
                // and to keep the buffer from growing indefinitely.
                let advance_by = scan_width / 2;
                self.sample_buffer.drain(..advance_by);

                // Break the loop and wait for more samples.
                break 'outer;
            }
        }

        if decoded_payload.is_empty() {
            Ok(None)
        } else {
            Ok(Some(decoded_payload))
        }
    }
}
