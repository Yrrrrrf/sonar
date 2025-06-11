// C:\...\sonar\examples\loopback.rs

use std::error::Error;
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dev_utils::{
    app_dt,
    dlog::*,
    format::{Style, Stylize},
    read_input,
};
use sonar::audio::{self, capture::AudioCapture, playback::AudioPlayback};
use sonar::modem::fsk::FSK;
use sonar::stack::datalink::{CodecTrait, SonarCodec, SonarCodecConfig};

// --- Constants ---
const BAUD_RATE: u32 = 300;
// Note: Confidence is not just SNR anymore. It's scaled by signal strength.
// A higher value may be needed. Start with a low value like 10.0 and tune up.
const CONFIDENCE_THRESHOLD: f32 = 4.0;
const FREQ_SPACE: f32 = 1200.0; // Bit '0'
const FREQ_MARK: f32 = 2400.0;  // Bit '1'

fn main() -> Result<(), Box<dyn Error>> {
    app_dt!(file!());
    set_max_level(Level::Info);
    println!(
        "{}",
        "Sonar Acoustic Modem Test".style(Style::Bold).color(dev_utils::format::YELLOW)
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

fn run_sender() -> Result<(), Box<dyn Error>> {
    info!("Starting in TRANSMIT mode.");
    let device = audio::select_device(false)?;
    info!("Selected output device: {}", device.name()?);

    let supported_config = device.supported_output_configs()?.next().ok_or("No supported output config")?.with_max_sample_rate();
    let sample_rate = supported_config.sample_rate().0;
    let config = supported_config.config();
    info!("Using sample rate: {} Hz", sample_rate);

    let fsk_modem = Box::new(FSK::new(sample_rate, FREQ_SPACE, FREQ_MARK, sample_rate / BAUD_RATE));
    let codec_config = SonarCodecConfig { sample_rate, baud_rate: BAUD_RATE, confidence_threshold: CONFIDENCE_THRESHOLD };
    let codec = SonarCodec::new(fsk_modem, codec_config);
    let playback = AudioPlayback::new_with_device(device)?;

    let mut message = read_input::<String>(Some("Enter message (press Enter to transmit): "))?;
    message.push('\n');
    info!("Preparing to send message...");

    let audio_samples = codec.encode(message.as_bytes())?;
    let duration_secs = audio_samples.len() as f32 / sample_rate as f32;
    info!("Estimated transmission time: {:.2} seconds.", duration_secs);

    let stream = playback.transmit(&config, &audio_samples)?;
    stream.play()?;
    thread::sleep(Duration::from_secs_f32(duration_secs + 0.5));
    info!("{}", "Transmission complete.".color(dev_utils::format::GREEN));
    Ok(())
}

fn run_listener() -> Result<(), Box<dyn Error>> {
    info!("Starting in LISTEN mode.");
    let device = audio::select_device(true)?;
    info!("Selected input device: {}", device.name()?);

    let supported_config = device.supported_input_configs()?.next().ok_or("No supported input config")?.with_max_sample_rate();
    let sample_rate = supported_config.sample_rate().0;
    let config = supported_config.config();
    info!("Using sample rate: {} Hz", sample_rate);

    let fsk_modem = Box::new(FSK::new(sample_rate, FREQ_SPACE, FREQ_MARK, sample_rate / BAUD_RATE));
    let codec_config = SonarCodecConfig { sample_rate, baud_rate: BAUD_RATE, confidence_threshold: CONFIDENCE_THRESHOLD };
    let mut codec = SonarCodec::new(fsk_modem, codec_config);
    let capture = AudioCapture::new_with_device(device)?;

    let stream = capture.start_listening(&config)?;
    stream.play()?;
    info!("Listening for incoming signals... Press Ctrl+C to stop.");
    info!("Using confidence threshold: {}", CONFIDENCE_THRESHOLD);

    let mut message_buffer: Vec<u8> = Vec::new();
    let mut last_char_time = Instant::now();
    const RECEPTION_TIMEOUT: Duration = Duration::from_secs(2);

    loop {
        if last_char_time.elapsed() > RECEPTION_TIMEOUT {
            codec.reset_state();
        }

        let samples = capture.get_samples();
        if samples.is_empty() {
            thread::sleep(Duration::from_millis(10));
            continue;
        }

        if let Ok(Some(bytes)) = codec.decode(&samples) {
            last_char_time = Instant::now();
            for byte in bytes {
                if byte == b'\n' {
                    if !message_buffer.is_empty() {
                        if let Ok(message) = String::from_utf8(message_buffer.clone()) {
                            println!();
                            info!("{}", "--- MESSAGE RECEIVED ---".style(Style::Bold).color(dev_utils::format::GREEN));
                            println!("{}", message);
                            println!();
                        }
                    }
                    message_buffer.clear();
                } else {
                    message_buffer.push(byte);
                    let char_to_print = (byte as char).to_string();
                    print!("{}", char_to_print.color(dev_utils::format::CYAN));
                    std::io::stdout().flush()?;
                }
            }
        }
    }
}