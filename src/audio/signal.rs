// src/audio/signal.rs

use dev_utils::format::*;
use std::time::Instant;

use crate::audio::{create_gradient_meter, format_signal_value, format_time, interpolate_color};

pub struct SignalMonitor {
    display_width: usize,
    peak_value: f32,
    samples_count: usize,
    last_peak_pos: Option<usize>,
    pub start_time: Instant,
}

impl SignalMonitor {
    /// Creates a new SignalMonitor for visualizing audio stream properties.
    ///
    /// # Arguments
    /// * `display_width` - The character width of the signal strength meter.
    pub fn new(display_width: usize) -> Self {
        Self {
            display_width,
            peak_value: 0.0,
            samples_count: 0,
            last_peak_pos: None,
            start_time: Instant::now(),
        }
    }

    pub fn print_header(&self) {
        let gradient_bar: String = (0..self.display_width)
            .map(|i| {
                "█".color(interpolate_color(
                    i as f32 / self.display_width as f32,
                    0.0,
                    1.0,
                ))
            })
            .collect();

        println!("Signal Strength: │{}│\n", gradient_bar);
    }

    /// Processes a chunk of audio samples to update the signal visualization.
    /// This function no longer decodes data.
    pub fn process_samples(&mut self, samples: &[f32]) {
        self.samples_count += samples.len();

        // Find the maximum absolute sample value in the current chunk.
        if let Some(max_sample) = samples
            .iter()
            .map(|s| s.abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        {
            // If the current max is a new peak, update the peak value and position.
            if max_sample > self.peak_value {
                self.peak_value = max_sample;
                self.last_peak_pos = Some(
                    (self.peak_value * self.display_width as f32 * 2.0)
                        .min(self.display_width as f32 - 1.0) as usize,
                );
            }

            // Only display the meter if the signal is above a minimal threshold.
            if max_sample > 0.00001 {
                self.display_signal(max_sample);
            }
        }

        // Handle peak value decay over time to make the meter responsive.
        // Assuming ~48kHz sample rate, decay every second.
        if self.samples_count > 48000 {
            self.peak_value *= 0.8; // Decay by 20%
            self.samples_count = 0;
            if self.peak_value < 0.001 {
                self.peak_value = 0.0;
                self.last_peak_pos = None;
            }
        }
    }

    /// Renders the signal strength meter to the console.
    pub fn display_signal(&self, max_sample: f32) {
        print!("\x1B[2K"); // Clear the current line
        print!("\x1B[1G"); // Move cursor to the beginning of the line

        let live_indicator = "●".color(if self.samples_count % 2 == 0 {
            GREEN
        } else {
            YELLOW
        });
        let meter = create_gradient_meter(max_sample, self.display_width, self.last_peak_pos);
        let current_value_str = format_signal_value(max_sample);
        let peak_value_str = format_signal_value(self.peak_value);

        println!(
            "{} {} {} {} │ Peak: {}",
            format_time(self.start_time.elapsed()),
            live_indicator,
            meter,
            current_value_str,
            peak_value_str
        );
    }
}
