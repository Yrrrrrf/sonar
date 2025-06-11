// C:\...\sonar\examples\loopback_test.rs

#![allow(unused)]

use std::error::Error;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dev_utils::{
    app_dt,
    dlog::*,
    format::{Style, Stylize},
    read_input,
};

// Import our core library components
use sonar::audio::{self, capture::AudioCapture, playback::AudioPlayback, signal::SignalMonitor};
use sonar::modem::{ModemTrait, fsk::FSK};
use sonar::stack::SonarCodec;
use sonar::stack::{CodecTrait, DecoderState};

fn main() -> Result<(), Box<dyn Error>> {
    app_dt!(file!());
    // set_max_level(Level::Info); // Start with Info, can be changed to Debug for more verbosity
    set_max_level(Level::Debug); // Start with Info, can be changed to Debug for more verbosity

    // --- Role Selection ---
    println!(
        "{}",
        "Sonar Two-Way Communication Test"
            .style(Style::Bold)
            .color(dev_utils::format::YELLOW)
    );
    println!("Choose the role for this computer:");
    println!("  1. Send a message");
    println!("  2. Listen for a message");

    let choice = read_input::<u32>(Some("Enter your choice (1 or 2): "))?;

    match choice {
        1 => run_sender()?,
        2 => run_listener()?,
        _ => error!("Invalid choice. Please run the script again and select 1 or 2."),
    }

    Ok(())
}

/// Handles the logic for the sending computer.
fn run_sender() -> Result<(), Box<dyn Error>> {
    info!("Starting in SEND mode.");

    // --- Device Selection ---
    let output_device = audio::select_device(false)?; // false for output
    info!("Selected output device: {}", output_device.name()?);

    // --- Codec and Audio Setup ---
    let fsk_modem = Box::new(FSK::default());
    let codec = SonarCodec::new(fsk_modem);
    let playback = AudioPlayback::new_with_device(output_device)?;

    // --- Message Input ---
    let message = read_input::<String>(Some("Enter the message to send: "))?;
    info!("Preparing to send message: '{}'", message);

    // --- Encoding and Transmission ---
    // 1. Datalink layer prepares the audio signal.
    let audio_samples = codec.encode(message)?;
    info!(
        "Message encoded into {} audio samples.",
        audio_samples
            .len()
            .to_string()
            .color(dev_utils::format::CYAN)
    );

    // Calculate approximate duration for user feedback
    let duration_secs = audio_samples.len() as f32 / playback.config.sample_rate.0 as f32;
    info!("Estimated transmission time: {:.2} seconds.", duration_secs);

    // 2. Audio layer plays the prepared signal.
    let stream = playback.transmit(&audio_samples)?;
    stream.play()?;

    info!("{}", "Transmission in progress...".style(Style::Italic));
    // Wait for the transmission to complete
    std::thread::sleep(Duration::from_secs_f32(duration_secs + 0.5)); // Add a small buffer

    info!(
        "{}",
        "Transmission complete.".color(dev_utils::format::GREEN)
    );
    Ok(())
}

/// Handles the logic for the listening computer.
fn run_listener() -> Result<(), Box<dyn Error>> {
    info!("Starting in LISTEN mode.");

    // --- Device Selection ---
    let input_device = audio::select_device(true)?;
    info!("Selected input device: {}", input_device.name()?);

    // --- Codec and Audio Setup ---
    let fsk_modem = Box::new(FSK::default());
    let mut codec = SonarCodec::new(fsk_modem);

    let capture = AudioCapture::new_with_device(input_device)?;
    // Monitor is disabled for clarity, but you can re-enable it.
    // let mut monitor = SignalMonitor::new(50);

    // --- Start Listening ---
    let stream = capture.start_listening()?;
    stream.play()?;

    info!("Listening for incoming signals... Press Ctrl+C to stop.");

    // --- Watchdog Variables ---
    // If the buffer grows beyond this many bits while trying to read a payload,
    // we'll assume the frame was lost and reset. 8000 bits = 1 KB.
    const STUCK_BUFFER_THRESHOLD: usize = 8000;

    // The main listening loop
    loop {
        let samples = capture.get_samples();
        if samples.is_empty() {
            thread::sleep(Duration::from_millis(50));
            continue;
        }

        // You can re-enable the monitor here if you wish.
        // monitor.process_samples(&samples);
        // io::stdout().flush()?;

        // Give samples to the Datalink layer to find a message.
        match codec.decode(&samples) {
            Ok(Some(payload)) => {
                let message = String::from_utf8_lossy(&payload);
                println!();
                info!(
                    "{}",
                    "--- MESSAGE RECEIVED ---"
                        .style(Style::Bold)
                        .color(dev_utils::format::GREEN)
                );
                println!("{}", message);
                println!();
            }
            Ok(None) => {
                // No complete frame found yet, this is normal.
                // Now we add the watchdog logic.
                let (current_state, buffer_size) = codec.get_decoder_status();

                if current_state == DecoderState::ReadingPayload
                    && buffer_size > STUCK_BUFFER_THRESHOLD
                {
                    warn!(
                        "Decoder seems stuck with a large buffer ({} bits). Frame likely lost. Resetting decoder.",
                        buffer_size
                    );
                    codec.reset_decoder(); // We need to add this method!
                }
            }
            Err(e) => {
                warn!(
                    "A frame was detected but could not be decoded: {}. Resetting decoder.",
                    e
                );
                codec.reset_decoder(); // Also reset on errors.
            }
        }
    }
}
