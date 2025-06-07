use std::error::Error;
use std::f32::consts::PI;

use super::{CodecTrait, SAMPLE_RATE};

/// BPSK (Binary Phase-Shift Keying) encoder/decoder implementation.
///
/// In BPSK:
/// - Bit 0 is represented by a sine wave with phase 0.
/// - Bit 1 is represented by a sine wave with phase π (i.e. inverted sine wave).
#[derive(Debug, PartialEq)]
pub struct BPSK {
    sample_rate: u32,     // Sampling rate in Hz
    carrier_freq: f32,    // Carrier frequency in Hz
    samples_per_bit: u32, // Number of samples used to represent one bit
}

impl Default for BPSK {
    fn default() -> Self {
        Self::new(SAMPLE_RATE, 1_200.0, SAMPLE_RATE / 1_200)
    }
}

impl BPSK {
    /// Creates a new BPSK encoder/decoder with the given parameters.
    pub fn new(sample_rate: u32, carrier_freq: f32, samples_per_bit: u32) -> Self {
        Self {
            sample_rate,
            carrier_freq,
            samples_per_bit,
        }
    }

    /// Generates a BPSK modulated sine wave for a given bit.
    ///
    /// For bit `false` (0), the sine wave has no phase shift.
    /// For bit `true` (1), the sine wave is shifted by π (inverted).
    fn gen_wave(&self, bit: bool) -> Vec<f32> {
        let phase = if bit { PI } else { 0.0 };
        let sample_period = 1.0 / self.sample_rate as f32;
        (0..self.samples_per_bit)
            .map(|i| {
                // Calculate the sine wave sample for the current time instant
                (2.0 * PI * self.carrier_freq * (i as f32 * sample_period) + phase).sin()
            })
            .collect()
    }

    /// Correlates a chunk of samples with a reference sine wave (phase 0) to detect the bit.
    ///
    /// The correlation is computed as the dot product between the sample chunk and a reference
    /// sine wave. A positive result indicates alignment with phase 0 (bit 0), while a negative
    /// result indicates a phase shift of π (bit 1).
    fn correlate(&self, chunk: &[f32]) -> f32 {
        let sample_period = 1.0 / self.sample_rate as f32;
        chunk.iter().enumerate().fold(0.0, |acc, (i, &sample)| {
            let reference = (2.0 * PI * self.carrier_freq * (i as f32 * sample_period)).sin();
            acc + sample * reference
        })
    }
}

impl CodecTrait for BPSK {
    /// Encodes raw data into a BPSK modulated signal.
    ///
    /// Each bit in the input data is converted into a BPSK-modulated sine wave.
    fn encode(&self, data: &[u8]) -> Result<Vec<f32>, Box<dyn Error>> {
        let mut signal = Vec::new();
        // Convert each byte into bits and generate the corresponding BPSK wave
        for &byte in data {
            for bit in super::byte_to_bits(byte) {
                signal.extend(self.gen_wave(bit));
            }
        }
        Ok(signal)
    }

    /// Decodes a BPSK modulated signal back into raw data.
    ///
    /// The signal is processed in chunks of `samples_per_bit` and a correlation with a reference
    /// sine wave (phase 0) is computed. A negative correlation implies a bit value of 1,
    /// while a positive correlation implies 0.
    fn decode(&self, samples: &[f32]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut decoded_data = Vec::new();
        let mut current_bits = Vec::new();

        // Process the signal chunk by chunk (each chunk represents one bit)
        for chunk in samples.chunks(self.samples_per_bit as usize) {
            let correlation = self.correlate(chunk);
            // Determine the bit: if correlation is negative, we treat it as bit 1.
            current_bits.push(correlation < 0.0);

            // Once we have 8 bits, convert them into a byte.
            if current_bits.len() == 8 {
                decoded_data.push(super::bits_to_byte(&current_bits));
                current_bits.clear();
            }
        }
        Ok(decoded_data)
    }
}
