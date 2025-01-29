use std::time::{Duration, Instant};
use dev_utils::{app_dt, dlog::*, format::*};

use crate::{audio::{create_gradient_meter, format_signal_value, format_time, interpolate_color}, encoding::Encoder};

pub struct SignalMonitor {
    display_width: usize,
    peak_value: f32,
    samples_count: usize,
    last_peak_pos: Option<usize>,
    start_time: Instant,
    decoder: Box<dyn Encoder>,
}

impl SignalMonitor {
    pub fn new(display_width: usize, decoder: Box<dyn Encoder>) -> Self {
        Self {
            display_width,
            peak_value: 0.0,
            samples_count: 0,
            last_peak_pos: None,
            start_time: Instant::now(),
            decoder,
        }
    }

    pub fn print_header(&self) {
        print!("Signal Strength: ");
        println!("│{}│\n", 
            (0..self.display_width)
                .map(|i| "█".color(interpolate_color(i as f32 / self.display_width as f32, 0.0, 1.0)))
                .collect::<String>()
        );
    }

    pub fn process_samples(&mut self, samples: &[f32]) -> Option<Vec<u8>> {
        self.samples_count += samples.len();
        let mut decoded_data = None;

        // Try to decode if we have enough samples
        if let Ok(data) = self.decoder.decode(samples) {
            if !data.is_empty() {
                decoded_data = Some(data);
            }
        }

        // Update signal visualization
        if let Some(max_sample) = samples.iter().map(|s| s.abs()).max_by(|a, b| a.partial_cmp(b).unwrap()) {
            if max_sample > self.peak_value {
                self.peak_value = max_sample;
                self.last_peak_pos = Some((self.peak_value * self.display_width as f32 * 2.0) as usize);
            }

            if max_sample > 0.00001 {
                self.display_signal(max_sample);
            }
        }

        // Handle peak decay
        if self.samples_count > 48000 {
            self.peak_value *= 0.8;
            self.samples_count = 0;
            if self.peak_value < 0.001 {
                self.peak_value = 0.0;
                self.last_peak_pos = None;
            }
        }

        decoded_data
    }

    pub fn display_signal(&self, max_sample: f32) {
        print!("\x1B[2K"); // Clear line
        print!("\x1B[1G"); // Move to start of line

        let elapsed = format_time(self.start_time.elapsed());
        let meter = create_gradient_meter(max_sample, self.display_width, self.last_peak_pos);
        let value = format_signal_value(max_sample);
        let peak = format_signal_value(self.peak_value);
        
        println!("{} {} {} {} │ Peak: {}", 
            elapsed,
            "●".color(if self.samples_count % 2 == 0 { GREEN } else { YELLOW }),
            meter,
            value,
            peak
        );
    }
}
