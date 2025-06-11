use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::error::Error;
use std::sync::{Arc, Mutex};

pub struct AudioCapture {
    pub device: cpal::Device,              // The physical input device (microphone)
    pub config: cpal::StreamConfig,        // Device configuration
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

                // if let Some(max) = samples.iter().cloned().map(f32::abs).max() {
                //     if max > 0.1 {
                //         eprintln!("Clipping detected: {}", max);
                //     }
                // }

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
