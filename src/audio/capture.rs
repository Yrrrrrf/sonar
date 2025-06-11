use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::error::Error;
use std::sync::{Arc, Mutex};

pub struct AudioCapture {
    pub device: cpal::Device, // Make public
    pub samples: Arc<Mutex<Vec<f32>>>,
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
            // .field("config", &self.config)
            .finish()
    }
}

impl AudioCapture {
    pub fn new_with_device(device: cpal::Device) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            device,
            samples: Arc::new(Mutex::new(Vec::new())),
        })
    }

    // UPDATED METHOD: Now takes config as an argument.
    pub fn start_listening(
        &self,
        config: &cpal::StreamConfig,
    ) -> Result<cpal::Stream, Box<dyn Error>> {
        let samples = Arc::clone(&self.samples);
        let stream = self.device.build_input_stream(
            config, // Use the provided config
            move |data: &[f32], _: &_| {
                samples.lock().unwrap().extend_from_slice(data);
            },
            |err| eprintln!("Error in input stream: {}", err),
            None,
        )?;
        Ok(stream)
    }

    pub fn get_samples(&self) -> Vec<f32> {
        let mut samples = self.samples.lock().unwrap();
        let result = samples.to_vec();
        samples.clear();
        result
    }
}

pub struct AudioPlayback {
    pub device: cpal::Device, // Make public
}

// Remove the `config` field from the struct.

impl AudioPlayback {
    pub fn new_with_device(device: cpal::Device) -> Result<Self, Box<dyn Error>> {
        Ok(Self { device })
    }

    // UPDATED METHOD: Now takes config as an argument.
    pub fn transmit(
        &self,
        config: &cpal::StreamConfig,
        samples: &[f32],
    ) -> Result<cpal::Stream, Box<dyn Error>> {
        let channels = config.channels as usize;
        let samples_arc = Arc::new(samples.to_vec());
        let mut sample_clock = 0;

        let stream = self.device.build_output_stream(
            config, // Use the provided config
            move |data: &mut [f32], _: &_| {
                for frame in data.chunks_mut(channels) {
                    let sample_value = *samples_arc.get(sample_clock).unwrap_or(&0.0);
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
