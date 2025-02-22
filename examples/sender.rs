use cpal::traits::StreamTrait;
use dev_utils::{format::*, read_input};
use sonar::{
    audio::{playback::AudioPlayback, select_device},
    codec::FSK,
};
use std::{error::Error, thread, time::Duration};

fn main() -> Result<(), Box<dyn Error>> {
    // Setup devices
    let output_device = select_device(false)?;

    let playback = AudioPlayback::new_with_device(output_device, Box::new(FSK::default()))?;

    // Start capture
    println!(
        "\n{}",
        "Ready to transfer data! Type 'q' to quit"
            .color(YELLOW)
            .style(Style::Dim)
    );

    loop {
        let input = read_input::<String>(Some(&"Send: ".style(Style::Bold)))?;
        if input.trim() == "q" {
            break;
        }
        let stream = playback.transmit(input.as_bytes())?; // Send the data

        // Wait a bit and get the captured samples
        thread::sleep(Duration::from_millis(100)); // 100ms cooldown
        stream.pause()?; // Stop the output stream
    }
    Ok(())
}
