// C:\...\sonar\examples\loopback.rs

use std::error::Error;
use std::io::Write;
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
use sonar::stack::datalink::{CodecTrait, SonarCodec, SonarCodecConfig};

// --- Constants for the new architecture ---
const SAMPLE_RATE: u32 = 48000;
const BAUD_RATE: u32 = 300;
// This is the "squelch" control. Higher values require a clearer signal.
// Good values are typically between 1.5 and 5.0.
const CONFIDENCE_THRESHOLD: f32 = 2.0;

// --- Define frequencies consistently ---
const FREQ_SPACE: f32 = 1200.0; // Bit '0'
const FREQ_MARK: f32 = 2400.0; // Bit '1'

fn main() -> Result<(), Box<dyn Error>> {
    app_dt!(file!());
    // Set to `Trace` for deep debugging, `Info` for normal use.
    set_max_level(Level::Info);
    // set_max_level(Level::Warn);

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
    // Use consistent frequency constants
    let fsk_modem = Box::new(FSK::new(
        SAMPLE_RATE,
        FREQ_SPACE, // freq_0
        FREQ_MARK,  // freq_1
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
    // Instruct the user to press Enter to send the newline character
    let mut message = read_input::<String>(Some("Enter the message to send (press Enter to transmit): "))?;
    message.push('\n'); // Append newline to signal end of message
    info!("Preparing to send message...");

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
    // Use consistent frequency constants
    let fsk_modem = Box::new(FSK::new(
        SAMPLE_RATE,
        FREQ_SPACE, // freq_0
        FREQ_MARK,  // freq_1
        SAMPLE_RATE / BAUD_RATE,
    ));
    let codec_config = SonarCodecConfig {
        sample_rate: SAMPLE_RATE,
        baud_rate: BAUD_RATE,
        confidence_threshold: CONFIDENCE_THRESHOLD,
    };
    let mut codec = SonarCodec::new(fsk_modem, codec_config);

    // Ensure our capture device matches the config
    let mut capture = AudioCapture::new_with_device(input_device)?;
    capture.config.sample_rate = cpal::SampleRate(SAMPLE_RATE);
    capture.config.channels = input_config.channels();

    // --- Start Listening ---
    let stream = capture.start_listening()?;
    stream.play()?;

    info!("Listening for incoming signals... Press Ctrl+C to stop.");
    info!("Using confidence threshold: {}", CONFIDENCE_THRESHOLD);

    // Add a buffer to assemble the final message
    let mut message_buffer: Vec<u8> = Vec::new();

    // The main listening loop
    loop {
        let samples = capture.get_samples();
        if samples.is_empty() {
            // Sleep briefly to avoid busy-waiting
            thread::sleep(Duration::from_millis(10));
            continue;
        }

        // Feed the captured audio samples to the decoder.
        match codec.decode(&samples) {
            Ok(Some(bytes)) => {
                for byte in bytes {
                    // Check for the newline character, which signals end of message
                    if byte == b'\n' {
                        // --- Message Complete! ---
                        // Convert buffer to string and print
                        if let Ok(message) = String::from_utf8(message_buffer.clone()) {
                            println!(); // Print a newline to separate from our real-time printing
                            warn!(
                                "{}",
                                "--- MESSAGE RECEIVED ---"
                                    .style(Style::Bold)
                                    .color(dev_utils::format::GREEN)
                            );
                            println!("{}", message);
                            println!();
                        }
                        // Clear the buffer for the next message
                        message_buffer.clear();
                    } else {
                        // --- Add byte to buffer ---
                        message_buffer.push(byte);
                        // Optional: Print character in real-time for teletype effect
                        let char_to_print = (byte as char).to_string();
                        print!("{}", char_to_print.color(dev_utils::format::CYAN));
                        std::io::stdout().flush()?;
                    }
                }
            }
            Ok(None) => {
                // No complete character found yet. This is normal.
            }
            Err(e) => {
                // This would indicate a more serious, unrecoverable error in the modem.
                warn!("An unrecoverable error occurred during decoding: {}", e);
            }
        }
    }
}