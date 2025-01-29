use std::error::Error;
use std::f32::consts::PI;

use super::Encoder;

// FSK (Frequency-Shift Keying) encoder implementation
#[derive(Debug, PartialEq)]
pub struct FSKEncoder {
    sample_rate: u32,      // Sampling rate in Hz
    freq_0: f32,          // Frequency for bit 0 in Hz
    freq_1: f32,          // Frequency for bit 1 in Hz
    samples_per_bit: u32, // Number of samples per bit
}

impl Default for FSKEncoder {
    fn default() -> Self {Self::new(48_000, 1_200.0, 2_400.0, 480)}
}

impl FSKEncoder {
    pub fn new(sample_rate: u32, freq_0: f32, freq_1: f32, samples_per_bit: u32) -> Self {
        Self { sample_rate, freq_0, freq_1, samples_per_bit }
    }

    // Helper method to generate a sine wave for a given frequency and number of samples
    fn generate_sine_wave(&self, frequency: f32, num_samples: u32) -> Vec<f32> {
        let sample_period = 1.0 / self.sample_rate as f32;
        (0..num_samples).map(|i| (2.0 * PI * frequency * (i as f32 * sample_period)).sin()).collect()
    }

    // Goertzel algorithm for frequency detection
    fn goertzel_energy(&self, samples: &[f32], target_freq: f32) -> f32 {
        let omega = 2.0 * PI * target_freq / self.sample_rate as f32;
        let cos_omega = omega.cos();
        let sin_omega = omega.sin();
        
        let mut s0 = 0.0;
        let mut s1 = 0.0;
        let mut s2;

        // Process all samples
        for &sample in samples {
            s2 = s1;
            s1 = s0;
            s0 = 2.0 * cos_omega * s1 - s2 + sample;
        };
        // Calculate energy
        let real = s0 - s1 * cos_omega;
        let imag = s1 * sin_omega;
        
        real * real + imag * imag
    }

    fn byte_to_bits(byte: u8) -> Vec<bool> {(0..8).map(|i| ((byte >> (7 - i)) & 1) == 1).collect()}

    fn bits_to_byte(bits: &[bool]) -> u8 {bits.iter().fold(0u8, |acc, &bit| (acc << 1) | if bit { 1 } else { 0 })}
}

impl Encoder for FSKEncoder {
    fn encode(&self, data: &[u8]) -> Result<Vec<f32>, Box<dyn Error>> {
        let mut signal = Vec::new();
        // Convert each byte to bits and generate corresponding sine waves
        for &byte in data {            
            for bit in Self::byte_to_bits(byte) {
                let frequency = if bit { self.freq_1 } else { self.freq_0 };
                let wave = self.generate_sine_wave(frequency, self.samples_per_bit);
                signal.extend(wave);
            }
        }

        Ok(signal)
    }

    fn decode(&self, samples: &[f32]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut decoded_data = Vec::new();
        let mut current_bits = Vec::new();

        // Process samples in chunks of samples_per_bit
        for chunk in samples.chunks(self.samples_per_bit as usize) {
            // Use Goertzel algorithm to detect which frequency is present
            let energy_0 = self.goertzel_energy(chunk, self.freq_0);
            let energy_1 = self.goertzel_energy(chunk, self.freq_1);

            // The frequency with higher energy represents the bit
            current_bits.push(energy_1 > energy_0);

            // When we have 8 bits, convert them to a byte
            if current_bits.len() == 8 {
                decoded_data.push(Self::bits_to_byte(&current_bits));
                current_bits.clear();
            }
        }

        Ok(decoded_data)
    }
}

// Example usage and test implementation
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fsk_encoding_decoding() {
        // * FSKEncoder::new(sample_rate, freq_0, freq_1, samples_per_bit)
        let encoder: FSKEncoder = FSKEncoder::new(48_000, 1_200.0, 2_400.0, 480);

        let test_data = vec![0b10101010, 0b11001100];  // * (0xAA:170,  0xCC:204)

        let enc = encoder.encode(&test_data).unwrap();
        let dec = encoder.decode(&enc).unwrap();

        assert_eq!(test_data, dec, "Decoded data should match original data");
    }
}