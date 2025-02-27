// // todo: Improve the cli by adding some:
// // - 'store' data when listing or just listing the data
// // - 'plot' data when listing or just listing the data
use clap::{Parser, Subcommand};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dev_utils::app_dt;
use sonar::audio::capture::AudioCapture;
use sonar::audio::playback::AudioPlayback;
use sonar::audio::signal::SignalMonitor;
use sonar::codec::FSK;
use std::error::Error;
/// Listeuse ctrlc;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

/// Command-line interface for the Sonar crate.
#[derive(Parser)]
#[command(name = "Sonar CLI")]
#[command(about = "A command-line interface for Sonar", long_about = None)]
struct Cli {
    /// Increase logging verbosity (use multiple times for more detailed logs)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands for the CLI.
#[derive(Subcommand)]
enum Commands {
    /// List available audio devices with their indices
    Devices,

    /// Transmit a message using audio signals
    Transmit {
        /// Index of the output device to use (optional, defaults to default output device)
        #[arg(short = 'o', long)]
        output_device: Option<usize>,

        /// Transmission volume (0.0 to 1.0, default: 1.0)
        #[arg(short = 'V', long)]
        volume: Option<f32>,

        /// The message to transmit
        message: String,
    },

    /// Listen for incoming audio signals
    Listen {
        /// Index of the input device to use (optional, defaults to default input device)
        #[arg(short = 'i', long)]
        input_device: Option<usize>,

        /// Enable signal monitoring visualization
        #[arg(short = 'm', long)]
        monitor: bool,
    },
}

/// List all available input and output audio devices with their indices.
fn list_devices() {
    let host = cpal::default_host();
    let input_devices: Vec<_> = host.input_devices().unwrap().collect();
    let output_devices: Vec<_> = host.output_devices().unwrap().collect();

    println!("Input devices:");
    for (i, device) in input_devices.iter().enumerate() {
        println!("\t{}: {}", i, device.name().unwrap());
    }
    println!("Output devices:");
    for (i, device) in output_devices.iter().enumerate() {
        println!("\t{}: {}", i, device.name().unwrap());
    }
}

/// Select an audio device by index from a list of devices, or use the default device.
fn select_device_by_index(
    index: Option<usize>,
    devices: &[cpal::Device],
    default_device: &cpal::Device,
) -> cpal::Device {
    index
        .and_then(|i| devices.get(i).cloned())
        .unwrap_or_else(|| default_device.clone())
}

/// Transmit a message using the specified or default output device.
fn transmit(
    message: &str,
    output_device_index: Option<usize>,
    volume: Option<f32>,
) -> Result<(), Box<dyn Error>> {
    let host = cpal::default_host();
    let output_devices: Vec<_> = host.output_devices()?.collect();
    let default_output = host
        .default_output_device()
        .ok_or("No default output device")?;
    let output_device =
        select_device_by_index(output_device_index, &output_devices, &default_output);

    let encoder = Box::new(FSK::default());
    let playback = AudioPlayback::new_with_device(output_device, encoder)?;

    let volume = volume.unwrap_or(1.0);
    // Calculate transmission duration based on encoded sample count and sample rate.
    let samples = playback.encoder.encode(message.as_bytes())?;
    let duration =
        Duration::from_secs_f32(samples.len() as f32 / playback.config.sample_rate.0 as f32);
    let stream = playback.transmit_with_volume(message.as_bytes(), volume)?;

    std::thread::sleep(duration);
    stream.pause()?;
    println!("Transmission complete.");

    Ok(())
}

fn listen(input_device_index: Option<usize>, _monitor: bool) -> Result<(), Box<dyn Error>> {
    let host = cpal::default_host();
    let input_devices: Vec<_> = host.input_devices()?.collect();
    let default_input = host
        .default_input_device()
        .ok_or("No default input device")?;
    let input_device = select_device_by_index(input_device_index, &input_devices, &default_input);

    let capture = AudioCapture::new_with_device(input_device)?;
    let decoder = Box::new(FSK::default());
    let mut signal_monitor = SignalMonitor::new(48, decoder);

    signal_monitor.print_header();
    let _stream = capture.start_listening()?;

    // Create an atomic flag to indicate when to stop.
    let running = Arc::new(AtomicBool::new(true));
    {
        let running = running.clone();
        ctrlc::set_handler(move || {
            running.store(false, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");
    }

    println!("Listening for incoming signals... (Press Ctrl+C to stop)");

    while running.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(100));
        let samples = capture.get_samples();
        if let Some(_) = signal_monitor.process_samples(&samples) {
            // TODO: This is where it gets interesting...
            // if let Some(decoded) = signal_monitor.process_samples(&samples) {
            // todo: We have a decoded message, what do we do with it?
            // todo: But also, first of all we need to read the message to know if its just 'noise' or a real message
            // todo: We can do this using some kind of 'threshold' or 'confidence' level

            // match String::from_utf8(decoded) {
            //     // Ok(message) => println!("Received: {}", message),
            //     Ok(message) => continue,
            //     Err(_) => continue,
            // }
        }
    }

    // .start_time.elapsed()
    println!("Total time: {:?}", signal_monitor.start_time.elapsed());
    println!("Listening complete.\n");
    Ok(())
}

/// Run the CLI, parsing arguments and executing the appropriate subcommand.
pub fn main() {
    let cli = Cli::parse();
    app_dt!("sonar-cli");
    // set_logging_level(cli.verbose);

    match cli.command {
        Commands::Devices => list_devices(),
        Commands::Transmit {
            output_device,
            volume,
            message,
        } => {
            if let Err(e) = transmit(&message, output_device, volume) {
                eprintln!("Error during transmission: {}", e);
            }
        }
        Commands::Listen {
            input_device,
            monitor,
        } => {
            if let Err(e) = listen(input_device, monitor) {
                eprintln!("Error during listening: {}", e);
            }
        }
    }
}
