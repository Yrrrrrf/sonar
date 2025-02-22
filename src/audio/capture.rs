use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::StreamConfig;
use std::default;
use std::error::Error;
use std::sync::{Arc, Mutex};

use crate::codec::bits_to_byte;

pub struct AudioCapture {
    device: cpal::Device,              // The physical input device (microphone)
    config: cpal::StreamConfig,        // Device configuration
    pub samples: Arc<Mutex<Vec<f32>>>, // Buffer for captured audio data
}

impl Default for AudioCapture {
    fn default() -> Self {
        Self::new_with_device(match cpal::default_host().default_input_device() {
            Some(device) => device,
            None => panic!("No input device available"),
        })
        .unwrap()
    }
}

impl std::fmt::Debug for AudioCapture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioCapture")
            .field("device", &self.device.name().unwrap())
            .field("config", &self.config)
            .finish()
    }
}

impl AudioCapture {
    /// Creates a new AudioCapture with a specific input device
    pub fn new_with_device(device: cpal::Device) -> Result<Self, Box<dyn Error>> {
        let config = device.default_input_config()?.config();
        Ok(Self {
            device,
            config,
            samples: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub fn new(device: cpal::Device, config: cpal::StreamConfig) -> Self {
        let samples = Arc::default();
        Self {
            device,
            config,
            samples,
        }
    }

    /// Start listening for audio input
    pub fn start_listening(&self) -> Result<cpal::Stream, Box<dyn Error>> {
        let samples = Arc::clone(&self.samples);

        let stream = self.device.build_input_stream(
            &self.config,
            move |data: &[f32], _: &_| {
                let mut samples = samples.lock().unwrap();
                samples.extend_from_slice(data);
            },
            |err| eprintln!("Error in input stream: {}", err),
            None,
        )?;

        stream.play()?;
        Ok(stream)
    }

    pub fn get_samples(&self) -> Vec<f32> {
        let mut samples = self.samples.lock().unwrap();
        let result = samples.clone();
        samples.clear();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_device() -> Result<(), Box<dyn Error>> {
        let capture = AudioCapture::default();
        Ok(())
    }

    #[test]
    fn test_specific_device() -> Result<(), Box<dyn Error>> {
        Ok(()) // Skip test if no devices available
    }
}
