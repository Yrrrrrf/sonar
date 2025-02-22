use cpal::traits::DeviceTrait;
use dev_utils::{app_dt, dlog::*, format::*, info, read_input};
use sonar::{
    audio::{capture::AudioCapture, list_audio_devices, signal::SignalMonitor},
    codec::FSK,
};
use std::time::Duration;

fn list_devices() -> Result<Vec<cpal::Device>, Box<dyn std::error::Error>> {
    let (input_devices, _) = list_audio_devices()?;

    println!(
        "\n{}",
        "Available Input Devices:".color(BLUE).style(Style::Bold)
    );
    println!("{}", "===================".color(BLUE));

    for (idx, device) in input_devices.iter().enumerate() {
        let config = device.default_input_config()?;

        println!(
            "{}. {} ({:?})",
            idx.to_string().color(GREEN),
            device.name()?.color(WHITE).style(Style::Bold),
            config.sample_format()
        );
        // Print additional device info
        if let Ok(config) = device.default_input_config() {
            println!(
                "   Sample Rate: {} Hz",
                config.sample_rate().0.to_string().color(YELLOW)
            );
            println!(
                "   Channels: {}",
                config.channels().to_string().color(YELLOW)
            );
        }
        println!();
    }

    Ok(input_devices)
}

fn select_device() -> Result<cpal::Device, Box<dyn std::error::Error>> {
    let devices = list_devices()?;

    if devices.is_empty() {
        return Err("No input devices found".into());
    }

    println!("\nSelect a device (0-{}):", devices.len() - 1);
    loop {
        let input = read_input::<u8>(None)?;
        if input < devices.len() as u8 {
            return Ok(devices[input as usize].clone());
        }
        println!("Invalid input. Please try again.");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    app_dt!(file!());
    // set_max_level(Level::Trace);
    set_max_level(Level::Debug);

    println!(
        "\n{}",
        "=== Audio Device Selector & Listener ==="
            .color(BLUE)
            .style(Style::Bold)
    );

    // Create capture with selected device
    let capture = AudioCapture::new_with_device(select_device()?)?;
    info!(
        "script::new({})",
        "AUDIO LISTENER".color(WHITE).style(Style::Bold)
    );
    info!(
        "Successfully started listening at {}",
        dev_utils::datetime::DateTime::now().time
    );

    // Initialize signal monitor with wider display
    let mut monitor = SignalMonitor::new(48, Box::new(FSK::default()));
    monitor.print_header();

    // Start listening
    let _stream = capture.start_listening()?;

    loop {
        std::thread::sleep(Duration::from_millis(100));
        if let Some(decoded) = monitor.process_samples(&capture.get_samples()) {
            trace!("{} {decoded:?}", "Decoded data:".color(GREEN));
        }
    }
}
