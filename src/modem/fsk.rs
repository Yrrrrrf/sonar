use std::error::Error;
use std::f32::consts::PI;

use super::{ModemTrait, SAMPLE_RATE};

// FSK (Frequency-Shift Keying) modem implementation
#[derive(Debug, PartialEq)]
pub struct FSK {
    sample_rate: u32,     // Sampling rate in Hz
    freq_0: f32,          // Frequency for bit 0 (space)
    freq_1: f32,          // Frequency for bit 1 (mark)
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
        // Calculate energy (squared magnitude)
        let real = s0 - s1 * cos_omega;
        let imag = s1 * sin_omega;

        real * real + imag * imag
    }
}

// Implement the ModemTrait for FSK
impl ModemTrait for FSK {
    fn modulate(&self, data: &[bool]) -> Result<Vec<f32>, Box<dyn Error>> {
        let mut signal = Vec::new();
        // Generate corresponding sine waves for each bit
        for &bit in data {
            signal.extend(self.gen_wave(
                if bit { self.freq_1 } else { self.freq_0 }, // freq_1 is mark, freq_0 is space
                self.samples_per_bit,
            ));
        }
        Ok(signal)
    }

    /// (Legacy demodulation - no longer the primary method for the new SonarCodec)
    fn demodulate(&self, samples: &[f32]) -> Result<Vec<bool>, Box<dyn Error>> {
        let mut decoded_data = Vec::new();
        
        // Process samples in chunks of samples_per_bit
        for chunk in samples.chunks(self.samples_per_bit as usize) {
            let (mark_energy, space_energy) = self.analyze_bit(chunk)?;
            decoded_data.push(mark_energy > space_energy);
        }

        Ok(decoded_data)
    }

    // ================== NEW IMPLEMENTATION ==================
    /// Analyzes a chunk of audio, returning the energy at the mark and space frequencies.
    ///
    /// This is the core analysis function for the new confidence-based decoder.
    /// It uses the Goertzel algorithm to efficiently measure the energy of the two
    /// frequencies that represent our '1' (mark) and '0' (space) bits.
    ///
    /// # Arguments
    /// * `samples` - A slice of f32 audio samples, typically one bit's worth.
    ///
    /// # Returns
    /// A `Result` containing a tuple `(mark_energy, space_energy)`.
    // ========================================================
    fn analyze_bit(&self, samples: &[f32]) -> Result<(f32, f32), Box<dyn Error>> {
        // Calculate the energy for the 'mark' frequency (bit '1')
        let mark_energy = self.correlate(samples, self.freq_1);

        // Calculate the energy for the 'space' frequency (bit '0')
        let space_energy = self.correlate(samples, self.freq_0);

        Ok((mark_energy, space_energy))
    }
}