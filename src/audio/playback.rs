// src/audio/playback.rs

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{error::Error, sync::Arc};

pub struct AudioPlayback {
    pub config: cpal::StreamConfig, // Device configuration
    pub device: cpal::Device,       // The physical output device (speakers)
}

impl std::fmt::Debug for AudioPlayback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioPlayback")
            .field("device", &self.device.name().unwrap_or_default())
            .field("config", &self.config)
            .finish()
    }
}

impl AudioPlayback {
    /// Creates a new AudioPlayback with the default output device.
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("No default output device available")?;
        Self::new_with_device(device)
    }

    /// Creates a new AudioPlayback with a specific output device.
    pub fn new_with_device(device: cpal::Device) -> Result<Self, Box<dyn Error>> {
        let config = device.default_output_config()?.config();
        Ok(Self { device, config })
    }

    /// Plays the given audio samples with a specific volume.
    ///
    /// # Arguments
    /// * `samples` - A slice of f32 audio samples to be played.
    /// * `volume` - A volume multiplier (e.g., 1.0 for full volume).
    pub fn transmit_with_volume(
        &self,
        samples: &[f32],
        volume: f32,
    ) -> Result<cpal::Stream, Box<dyn Error>> {
        let channels = self.config.channels as usize;
        // The samples are already modulated, so we just wrap them for the audio thread.
        let samples_arc = Arc::new(samples.to_vec());

        self.build_output_stream(samples_arc, channels, volume)
    }

    /// Plays the given audio samples with default volume (1.0).
    pub fn transmit(&self, samples: &[f32]) -> Result<cpal::Stream, Box<dyn Error>> {
        self.transmit_with_volume(samples, 1.0)
    }

    // Private helper method to build the cpal output stream.
    fn build_output_stream(
        &self,
        samples: Arc<Vec<f32>>,
        channels: usize,
        volume: f32,
    ) -> Result<cpal::Stream, Box<dyn Error>> {
        let mut sample_clock = 0;

        let stream = self.device.build_output_stream(
            &self.config,
            move |data: &mut [f32], _: &_| {
                for frame in data.chunks_mut(channels) {
                    let sample_value = if sample_clock < samples.len() {
                        samples[sample_clock] * volume // Apply volume
                    } else {
                        0.0 // Output silence when samples are finished
                    };

                    // Write the same sample to all channels
                    for sample in frame.iter_mut() {
                        *sample = sample_value;
                    }

                    sample_clock += 1;
                }
            },
            |err| eprintln!("Error in output stream: {}", err),
            None,
        )?;
        Ok(stream)
    }
}
