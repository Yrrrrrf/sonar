use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{error::Error, sync::Arc};

use crate::modem::ModemTrait;


pub struct AudioPlayback {
    pub config: cpal::StreamConfig,   // Device configuration
    pub device: cpal::Device,         // The physical output device (speakers)
    pub modem: Box<dyn ModemTrait>, // The modem instance for signal processing
}

impl std::fmt::Debug for AudioPlayback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "AudioPlayback {{ device: {:?}, config: {:?} }}", self.device.name(), self.config)
        f.debug_struct("AudioPlayback")
            .field("device", &self.device.name().unwrap())
            .field("config", &self.config)
            .finish()
    }
}

impl AudioPlayback {
    /// Creates a new AudioPlayback with the default output device and modem
    pub fn new(modem: Box<dyn ModemTrait>) -> Result<Self, Box<dyn Error>> {
        Self::new_with_device(
            cpal::default_host()
                .default_output_device()
                .ok_or("No output device found")?,
            modem,
        )
    }

    /// Creates a new AudioPlayback with a specific output device and modem
    pub fn new_with_device(
        device: cpal::Device,
        modem: Box<dyn ModemTrait>,
    ) -> Result<Self, Box<dyn Error>> {
        let config = device.default_output_config()?.config();
        Ok(Self {
            device,
            config,
            modem,
        })
    }

    /// Send data through the modem and play it with volume control
    pub fn transmit_with_volume(
        &self,
        data: &[u8],
        volume: f32,
    ) -> Result<cpal::Stream, Box<dyn Error>> {
        // Encode the data into audio samples
        let channels = self.config.channels as usize;
        let samples = Arc::new(self.modem.modulate(data)?);
        let samples_clone = Arc::clone(&samples);

        let stream = self.build_output_stream(samples_clone, channels, volume)?;
        stream.play()?;

        Ok(stream)
    }

    /// Send data through the modem and play it (with default volume = 1.0)
    pub fn transmit(&self, data: &[u8]) -> Result<cpal::Stream, Box<dyn Error>> {
        self.transmit_with_volume(data, 1.0)
    }

    // Private helper methods
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
                    let sample = if sample_clock >= samples.len() {
                        0.0 // Output silence
                    } else {
                        samples[sample_clock] * volume // Apply volume
                    };
                    // Copy sample to all channels
                    frame.iter_mut().for_each(|s| *s = sample);
                    sample_clock += 1;
                }
            },
            |err| eprintln!("Error in output stream: {}", err),
            None,
        )?;
        Ok(stream)
    }
}
