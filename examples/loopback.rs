// C:\...\sonar\examples\loopback.rs

use std::error::Error;
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, StreamTrait};
use dev_utils::{
    app_dt,
    dlog::*,
    format::{Style, Stylize},
    read_input,
};

// Import our core library components
use sonar::audio::{self, capture::AudioCapture, playback::AudioPlayback};
use sonar::modem::fsk::FSK;
use sonar::stack::datalink::{CodecTrait, SonarCodec, SonarCodecConfig};

// --- Constants for the new architecture ---
const SAMPLE_RATE: u32 = 48000;
const BAUD_RATE: u32 = 300;
// This is the "squelch" control. Higher values require a clearer signal.
// Good values are typically between 1.5 and 5.0.
const CONFIDENCE_THRESHOLD: f32 = 2.0;

fn main() -> Result<(), Box<dyn Error>> {
    app_dt!(file!());
    set_max_level(Level::Trace); // Start with Info for a cleaner user experience

    // --- Role Selection ---
    println!(
        "{}",
        "Sonar Acoustic Modem Test"
            .style(Style::Bold)
            .color(dev_utils::format::YELLOW)
    );
    println!("Choose the role for this computer:");
    println!("  1. Transmit a message (TX)");
    println!("  2. Listen for a message (RX)");

    let choice = read_input::<u32>(Some("Enter your choice (1 or 2): "))?;

    match choice {
        1 => run_sender()?,
        2 => run_listener()?,
        _ => error!("Invalid choice. Please run the script again and select 1 or 2."),
    }

    Ok(())
}

/// Handles the logic for the transmitting computer.
fn run_sender() -> Result<(), Box<dyn Error>> {
    info!("Starting in TRANSMIT mode.");

    // --- Device Selection ---
    let output_device = audio::select_device(false)?; // false for output
    let output_config = output_device.default_output_config()?;
    info!("Selected output device: {}", output_device.name()?);

    // --- Codec and Audio Setup ---
    // Ensure the modem and codec config use the same parameters
    let fsk_modem = Box::new(FSK::new(
        SAMPLE_RATE,
        1200.0,
        2400.0,
        SAMPLE_RATE / BAUD_RATE,
    ));
    let codec_config = SonarCodecConfig {
        sample_rate: SAMPLE_RATE,
        baud_rate: BAUD_RATE,
        confidence_threshold: CONFIDENCE_THRESHOLD, // Not used by sender, but good practice
    };
    let codec = SonarCodec::new(fsk_modem, codec_config);

    // Ensure our playback device matches the config
    let mut playback = AudioPlayback::new_with_device(output_device)?;
    playback.config.sample_rate = cpal::SampleRate(SAMPLE_RATE);
    playback.config.channels = output_config.channels();

    // --- Message Input ---
    let message = read_input::<String>(Some("Enter the message to send: "))?;
    info!("Preparing to send message: '{}'", message);

    // --- Encoding and Transmission ---
    let audio_samples = codec.encode(message.as_bytes())?;
    info!(
        "Message encoded into {} audio samples.",
        audio_samples.len().to_string().color(dev_utils::format::CYAN)
    );

    // Calculate approximate duration for user feedback
    let duration_secs = audio_samples.len() as f32 / SAMPLE_RATE as f32;
    info!("Estimated transmission time: {:.2} seconds.", duration_secs);

    // Play the generated audio signal.
    let stream = playback.transmit(&audio_samples)?;
    stream.play()?;

    info!("{}", "Transmission in progress...".style(Style::Italic));
    // Wait for the transmission to complete, adding a small buffer
    thread::sleep(Duration::from_secs_f32(duration_secs + 0.5));

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
    let input_device = audio::select_device(true)?; // true for input
    let input_config = input_device.default_input_config()?;
    info!("Selected input device: {}", input_device.name()?);

    // --- Codec and Audio Setup ---
    let fsk_modem = Box::new(FSK::new(
        SAMPLE_RATE,
        1200.0,
        2400.0,
        SAMPLE_RATE / BAUD_RATE,
    ));
    let codec_config = SonarCodecConfig {
        sample_rate: SAMPLE_RATE,
        baud_rate: BAUD_RATE,
        confidence_threshold: CONFIDENCE_THRESHOLD,
    };
    let mut codec = SonarCodec::new(fsk_modem, codec_config);

    // Ensure our capture device matches the config
    let mut capture = AudioCapture::new_with_device(input_device)?; // <--- MAKE THIS MUTABLE
    capture.config.sample_rate = cpal::SampleRate(SAMPLE_RATE);
    capture.config.channels = input_config.channels();

    // --- Start Listening ---
    let stream = capture.start_listening()?;
    stream.play()?;

    // ================== CORRECTED BLOCK ==================
    // Query the config from the `capture` object itself.
    info!(
        "{}",
        format!("Actual stream sample rate: {} Hz", capture.config.sample_rate.0)
            .style(Style::Bold)
            .color(dev_utils::format::MAGENTA)
    );
    if capture.config.sample_rate.0 != SAMPLE_RATE {
        warn!(
            "WARNING: Actual sample rate does not match configured rate of {} Hz!",
            SAMPLE_RATE
        );
    }
    // =====================================================

    info!("Listening for incoming signals... Press Ctrl+C to stop.");
    info!("Using confidence threshold: {}", CONFIDENCE_THRESHOLD);

    // The main listening loop - now much simpler!
    loop {
        let samples = capture.get_samples();
        if samples.is_empty() {
            // Sleep briefly to avoid busy-waiting
            thread::sleep(Duration::from_millis(10));
            continue;
        }

        // Feed the captured audio samples to the decoder.
        // The decoder will internally buffer and process them.
        match codec.decode(&samples) {
            Ok(Some(bytes)) => {
                // The decoder found one or more valid bytes!
                let message_chunk = String::from_utf8_lossy(&bytes);
                // Print without a newline to allow characters to appear as they are decoded.
                print!("{}", message_chunk.color(dev_utils::format::GREEN));
                // We need to flush stdout to make sure the characters appear immediately.
                use std::io::Write;
                std::io::stdout().flush()?;
            }
            Ok(None) => {
                // No complete character found yet. This is normal.
                // The codec is waiting for more audio or a clearer signal.
            }
            Err(e) => {
                // This would indicate a more serious, unrecoverable error in the modem.
                warn!("An unrecoverable error occurred during decoding: {}", e);
            }
        }
    }
}