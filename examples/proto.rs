#![allow(unused)]  // silence unused warnings while developing

use std::{default, error::Error, thread, time::Duration};
use cpal::traits::StreamTrait;
use cpal::{traits::{DeviceTrait, HostTrait}, Device};
use dev_utils::{
    app_dt, debug, dlog::*, error, format::*, info, read_input, trace, warn
};
use sonar::encoding::Encoder;
use sonar::{
    audio::{
        capture::AudioCapture, dev::AudioDev, playback::AudioPlayback, select_device
    }, encoding::FSKEncoder, proto::Frame 
};    


fn get_capture_device() -> AudioCapture {
    AudioCapture::new_with_device(select_device(true).unwrap()).unwrap()
}

fn get_playback_device(encoder: Box<dyn Encoder>) -> AudioPlayback {
    AudioPlayback::new_with_device(select_device(false).unwrap(), encoder).unwrap()
}
fn get_audio_dev(
    default_devices: bool
) -> Result<(AudioCapture, AudioPlayback), Box<dyn Error>> {
    Ok(match default_devices {
        true => {info!("Using default devices");
            (AudioCapture::default(),AudioPlayback::new(Box::new(FSKEncoder::default()))?)
        },
        false => {info!("Using selected devices");
            (get_capture_device(), get_playback_device(Box::new(FSKEncoder::default())))
        }
    })
}

fn start_sender(dev: &AudioDev) -> Result<(), Box<dyn Error>> {
    println!("\n{}", "Ready to send messages! Type 'q' to quit".color(YELLOW).style(Style::Dim));

    loop {
        let input = read_input::<String>(Some(&"Send: ".style(Style::Bold)))?;
        if input.trim() == "q" { break; }

        // Send data and get stream
        let stream = dev.send(input.as_bytes())?;
        
        // Wait for transmission and cleanup
        thread::sleep(Duration::from_millis(100));
        stream.pause()?;
    }
    Ok(())
}

fn start_listener(dev: &AudioDev) -> Result<(), Box<dyn Error>> {
    println!("\n{}", "Starting listener mode... Press Ctrl+C to quit".color(YELLOW).style(Style::Dim));
    
    // Start monitoring for incoming frames
    let stream = dev.monitor()?;

    // Keep the main thread alive
    loop {
        thread::sleep(Duration::from_millis(100));
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    app_dt!(file!());
    set_max_level(Level::Trace);

    // let default_devices = true;
    let default_devices = false;
    let (capture, playback) = get_audio_dev(default_devices)?;

    trace!("{:#?}", capture);
    trace!("{:#?}", playback);

    let dev = AudioDev::new(capture, playback)?;

    // Ask user for mode
    println!("\n{}", "Select mode:".color(BLUE).style(Style::Bold));
    println!("1. Sender");
    println!("2. Listener");
    
    loop {
        let mode = read_input::<String>(Some("Choose mode (1/2): "))?;
        match mode.trim() {
            "1" => return start_sender(&dev),
            "2" => return start_listener(&dev),
            _ => println!("Invalid selection. Please choose 1 or 2."),
        }
    }
}
