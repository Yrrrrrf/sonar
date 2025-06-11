use std::error::Error;
use std::f32::consts::PI;

use super::{ModemTrait, SAMPLE_RATE};

// FSK (Frequency-Shift Keying) modem implementation
#[derive(Debug, PartialEq)]
pub struct FSK {
    sample_rate: u32,     // Sampling rate in Hz
    freq_0: f32,          // Frequency for bit 0 in Hz
    freq_1: f32,          // Frequency for bit 1 in Hz
    samples_per_bit: u32, // Number of samples per bit
}

impl Default for FSK {
    fn default() -> Self {
        const SAMPLE_RATE: u32 = 48_000;
        const BAUD_RATE: u32 = 300; // SLOWED DOWN from 1200
        const SAMPLES_PER_BIT: u32 = SAMPLE_RATE / BAUD_RATE; // NOW = 160 samples per bit

        Self::new(SAMPLE_RATE, 1_200.0, 2_400.0, SAMPLES_PER_BIT)
    }
}

impl FSK {
    pub fn new(sample_rate: u32, freq_0: f32, freq_1: f32, samples_per_bit: u32) -> Self {
        Self {
            sample_rate,
            freq_0,
            freq_1,
            samples_per_bit,
        }
    }

    // Helper method to generate a sine wave for a given frequency and number of samples
    fn gen_wave(&self, frequency: f32, num_samples: u32) -> Vec<f32> {
        let sample_period = 1.0 / self.sample_rate as f32;
        (0..num_samples)
            .map(|i| (2.0 * PI * frequency * (i as f32 * sample_period)).sin())
            .collect()
    }

    // Goertzel algorithm for frequency detection
    fn correlate(&self, samples: &[f32], target_freq: f32) -> f32 {
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
        }
        // Calculate energy
        let real = s0 - s1 * cos_omega;
        let imag = s1 * sin_omega;

        real * real + imag * imag
    }
}

impl ModemTrait for FSK {
    fn modulate(&self, data: &[bool]) -> Result<Vec<f32>, Box<dyn Error>> {
        let mut signal = Vec::new();
        // Generate corresponding sine waves for each bit
        for &bit in data {
            signal.extend(self.gen_wave(
                if bit { self.freq_1 } else { self.freq_0 },
                self.samples_per_bit,
            ));
        }

        Ok(signal)
    }

    fn demodulate(&self, samples: &[f32]) -> Result<Vec<bool>, Box<dyn Error>> {
        let mut decoded_data = Vec::new();
        let mut current_bits = Vec::new();

        // Process samples in chunks of samples_per_bit
        for chunk in samples.chunks(self.samples_per_bit as usize) {
            // Use Goertzel algorithm to detect which frequency is present
            let energy_0 = self.correlate(chunk, self.freq_0);
            let energy_1 = self.correlate(chunk, self.freq_1);

            // The frequency with higher energy represents the bit
            current_bits.push(energy_1 > energy_0);

            // When we have 8 bits, convert them to a byte
            if current_bits.len() == 8 {
                decoded_data.push(super::bits_to_byte(&current_bits));
                current_bits.clear();
            }
        }

        Ok(decoded_data.iter().map(|&b| b != 0).collect())
    }

    fn samples_per_bit(&self) -> usize {
        self.samples_per_bit as usize
    }

    fn get_bit_metrics(&self, samples: &[f32]) -> ((f32, f32), (f32, f32)) {
        let mark_energy = self.correlate(samples, self.freq_1);
        let space_energy = self.correlate(samples, self.freq_0);
        ((mark_energy, space_energy), (mark_energy, space_energy))
    }
}
