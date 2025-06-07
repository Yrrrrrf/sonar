// First, modify the Encoder trait in encoding/mod.rs
pub trait Encoder: Send {
    // Add Send requirement
    fn encode(&self, data: &[u8]) -> Result<Vec<f32>, Box<dyn Error>>;
    fn decode(&self, samples: &[f32]) -> Result<Vec<u8>, Box<dyn Error>>;
}

// Then update the AudioDev implementation:
use bytes::BytesMut;
use cpal::Stream;
use cpal::traits::StreamTrait;
use dev_utils::{dlog::*, format::*};
use std::{
    error::Error,
    sync::{Arc, Mutex},
    time::Duration,
};

use super::{capture::AudioCapture, playback::AudioPlayback, signal::SignalMonitor};
use crate::modem::FSK; // Added FSK import

pub struct AudioDev {
    capture: AudioCapture,
    playback: AudioPlayback,
    buffer: Arc<Mutex<Vec<f32>>>,
    sequence: Arc<Mutex<u8>>,
    decoded_buffer: Arc<Mutex<BytesMut>>, // Add buffer for decoded data
}

impl AudioDev {
    pub fn new(capture: AudioCapture, playback: AudioPlayback) -> Result<Self, Box<dyn Error>> {
        let buffer = Arc::default();
        let sequence = Arc::new(Mutex::new(0));
        let decoded_buffer = Arc::new(Mutex::new(BytesMut::new()));
        Ok(Self {
            capture,
            playback,
            buffer,
            sequence,
            decoded_buffer,
        })
    }

    pub fn monitor(&self) -> Result<Stream, Box<dyn Error>> {
        let stream = self.capture.start_listening()?;
        info!(
            "{}",
            "🎧 Started listening for incoming signals...".color(GREEN)
        );

        let samples = Arc::clone(&self.capture.samples);
        let decoded_buffer = Arc::clone(&self.decoded_buffer);

        std::thread::spawn(move || {
            let monitor = SignalMonitor::new(48, Box::new(FSK::default()));
            monitor.print_header();

            let mut monitor = monitor;
            let mut total_samples_processed = 0;

            loop {
                std::thread::sleep(Duration::from_millis(100));

                let current_samples = {
                    let mut samples_lock = samples.lock().unwrap();
                    let result = samples_lock.clone();
                    samples_lock.clear();
                    result
                };

                if !current_samples.is_empty() {
                    total_samples_processed += current_samples.len();
                    // trace!("Processing {} samples (Total: {})",
                    //     current_samples.len().to_string().color(YELLOW),
                    //     total_samples_processed.to_string().color(BLUE)
                    // );

                    // Process samples and accumulate decoded data
                    if let Some(decoded_data) = monitor.process_samples(&current_samples) {
                        // info!("📦 Raw decoded data: {} bytes", decoded_data.len().to_string().color(GREEN));

                        // Append to our decoded buffer
                        let mut buffer = decoded_buffer.lock().unwrap();
                        buffer.extend_from_slice(&decoded_data);

                        // Try to extract frames from accumulated data
                        let buffer_bytes = buffer.clone().freeze();

                        // match Frame::deserialize(buffer_bytes) {
                        //     Ok(Some(frame)) => {
                        //         // info!("✅ Successfully decoded frame:");
                        //         // info!("   Sequence: {}", frame.sequence().to_string().color(GREEN));
                        //         // info!("   Length: {}", frame.payload().len().to_string().color(GREEN));

                        //         // Print the actual message
                        //         // if let Ok(message) = String::from_utf8(frame.payload().to_vec()) {
                        //         //     info!("📨 Message content: {}", message.color(GREEN).style(Style::Bold));
                        //         // }

                        //         // Clear the processed data from buffer
                        //         buffer.clear();  // Clear after successful frame extraction
                        //     },
                        //     Ok(None) => {
                        //         debug!("🔄 Buffering data, waiting for complete frame...");
                        //         // Keep accumulating data
                        //     },
                        //     Err(e) => {
                        //         error!("❌ Error deserializing frame: {}", e);
                        //         buffer.clear();  // Clear on error to prevent buffer bloat
                        //     }
                        // }
                    }
                }
            }
        });

        Ok(stream)
    }

    // pub fn send(&self, data: &[u8]) -> Result<Stream, Box<dyn Error>> {
    //     // Create a new frame with incrementing sequence number
    //     let mut seq = self.sequence.lock().unwrap();
    //     let frame = Frame::new(data, *seq)?;
    //     *seq = seq.wrapping_add(1);

    //     // Serialize frame and transmit
    //     let frame_bytes = frame.serialize();
    //     info!("📤 Sending frame with sequence: {}", frame.sequence());
    //     self.playback.transmit(&frame_bytes)
    // }

    pub fn stop(&self, streams: &[Stream]) -> Result<(), Box<dyn Error>> {
        for stream in streams {
            stream.pause()?;
        }
        Ok(())
    }
}
