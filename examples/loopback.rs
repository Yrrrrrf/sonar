// C:\...\sonar\examples\loopback.rs

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
use sonar::audio::{self, capture::AudioCapture, playback::AudioPlayback};
use sonar::modem::fsk::FSK;
use sonar::stack::{CodecTrait, SonarCodec};

fn main() -> Result<(), Box<dyn Error>> {
    app_dt!(file!());
    set_max_level(Level::Info); // Start with Info for cleaner output
    // set_max_level(Level::Debug); // Use Debug for more verbosity

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
    // 1. Datalink layer prepares the audio signal with leader/trailer tones and character frames.
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

    // --- Start Listening ---
    let stream = capture.start_listening()?;
    stream.play()?;

    info!("Listening for incoming signals... Press Ctrl+C to stop.");
    println!("{}", "--- MESSAGE START ---".style(Style::Bold).color(dev_utils::format::GREEN));

    // The main listening loop
    loop {
        let samples = capture.get_samples();
        if samples.is_empty() {
            thread::sleep(Duration::from_millis(50));
            continue;
        }

        // Give samples to the Datalink layer to find a message.
        // The new `decode` uses a sliding window and confidence scoring.
        match codec.decode(&samples) {
            Ok(Some(payload)) => {
                // `decode` can return multiple bytes at once if they are
                // processed quickly from the buffer.
                let message_chunk = String::from_utf8_lossy(&payload);
                // Print without a newline to allow characters to appear as they are decoded.
                print!("{}", message_chunk);
                io::stdout().flush()?; // Ensure the character is displayed immediately.
            }
            Ok(None) => {
                // No character with high enough confidence was found. This is normal
                // during silence or noise. The loop will continue, processing more samples.
            }
            Err(e) => {
                // An unrecoverable error occurred during demodulation or analysis.
                warn!(
                    "An error occurred during decoding: {}. Resetting decoder.",
                    e
                );
                codec.reset_decoder();
            }
        }
    }
}