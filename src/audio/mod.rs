// src/audio/mod.rs

use cpal::traits::{DeviceTrait, HostTrait};
use dev_utils::{format::*, read_input};
use std::{error::Error, time::Duration};

// * mod.rs
pub mod capture;
pub mod config;
pub mod playback;
pub mod signal;

pub fn list_audio_devices() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();

    println!("Input devices:");
    let input_devices = host.input_devices()?;
    for (i, device) in input_devices.enumerate() {
        println!("{}: {}", i, device.name()?);
    }

    println!("Output devices:");
    let output_devices = host.output_devices()?;
    for (i, device) in output_devices.enumerate() {
        println!("{}: {}", i, device.name()?);
    }

    Ok(())
}

pub fn select_device(input: bool) -> Result<cpal::Device, Box<dyn Error>> {
    let host = cpal::default_host();
    let mut devices = if input {
        host.input_devices()?
    } else {
        host.output_devices()?
    };

    println!(
        "\n{}",
        format!(
            "Available {} Devices:",
            if input { "Input" } else { "Output" }
        )
        .color(BLUE)
        .style(Style::Bold)
    );

    let device_list: Vec<_> = devices.collect();
    for (idx, device) in device_list.iter().enumerate() {
        println!(
            "{}. {}",
            idx.to_string().color(GREEN),
            device.name().unwrap_or_default().color(WHITE)
        );
    }

    loop {
        let choice = read_input::<usize>(Some("Select device number: "))?;
        if choice < device_list.len() {
            return Ok(device_list[choice].clone());
        }
        println!("Invalid selection. Try again.");
    }
}

// ? FORMAT RELATED FUNCTIONS
pub fn interpolate_color(value: f32, min: f32, max: f32) -> Color {
    let t = ((value - min) / (max - min)).clamp(0.0, 1.0);

    let colors = [
        (0.0, (0, 0, 255)),   // Blue
        (0.3, (0, 255, 0)),   // Green
        (0.6, (255, 255, 0)), // Yellow
        (1.0, (255, 0, 0)),   // Red
    ];

    let mut color1 = colors[0];
    let mut color2 = colors[1];

    for window in colors.windows(2) {
        if t >= window[0].0 && t <= window[1].0 {
            color1 = window[0];
            color2 = window[1];
            break;
        }
    }

    let factor = (t - color1.0) / (color2.0 - color1.0);

    let r = (color1.1.0 as f32 * (1.0 - factor) + color2.1.0 as f32 * factor) as u8;
    let g = (color1.1.1 as f32 * (1.0 - factor) + color2.1.1 as f32 * factor) as u8;
    let b = (color1.1.2 as f32 * (1.0 - factor) + color2.1.2 as f32 * factor) as u8;

    Color::from((r, g, b))
}

pub fn format_time(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    let millis = duration.subsec_millis();

    format!("[{:02}:{:02}:{:02}.{:03}]", hours, minutes, seconds, millis)
        .style(Style::Dim)
        .style(Style::Italic)
}

pub fn create_gradient_meter(value: f32, width: usize, peak_pos: Option<usize>) -> String {
    let meter_width = (value * width as f32 * 2.0).min(width as f32) as usize;
    let mut meter = String::with_capacity(width * 3);

    for i in 0..width {
        if i < meter_width {
            let segment_value = i as f32 / width as f32;
            let color = interpolate_color(segment_value, 0.0, 1.0);
            meter.push_str(&"█".color(color));
        } else if Some(i) == peak_pos {
            meter.push_str(&"╎".color(WHITE).style(Style::Bold)); // Peak indicator
        } else {
            meter.push(' ');
        }
    }
    format!("│{}│", meter)
}

pub fn format_signal_value(value: f32) -> String {
    format!("{:>10.8}", value)
        .color(interpolate_color(value, 0.0, 0.1))
        .style(Style::Bold)
        .to_string()
}
