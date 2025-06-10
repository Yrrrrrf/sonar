#![allow(unused)]

use std::error::Error;
use std::thread;
use std::time::Duration;

use cpal::traits::StreamTrait;
use dev_utils::{app_dt, debug, dlog::*, error, info, trace, warn};

use sonar::audio::{
    capture::AudioCapture, dev::AudioDev, playback::AudioPlayback, signal::SignalMonitor,
};
use sonar::*;
use sonar::modem::FSK;
use sonar::modem::ModemTrait;

const TEST_DATA: &[u8] = &[
    0xAA, 0xBB, 0xCC, 0xDD, // Test pattern
    b'H', b'e', b'l', b'l', b'o', // ASCII text
    0x01, 0x02, 0x03, 0x04, // Binary sequence
];

fn main() -> Result<(), Box<dyn Error>> {
    // Run tests
    // test_audio_loopback()?;
    // test_signal_strength()?;
    test_fsk_configs()?;

    Ok(())
}

// fn test_audio_loopback() -> Result<(), Box<dyn Error>> {
//     // Create FSK ModemTrait with default settings
//     let ModemTrait = Box::new(FSK::default());
//     let ModemTrait_clone = Box::new(FSK::default());

//     // Initialize audio devices
//     let capture = AudioCapture::new()?;
//     let playback = AudioPlayback::new(ModemTrait_clone)?;

//     // Create router
//     let router = AudioRouter::new(capture, playback, ModemTrait)?;

//     // Create signal monitor
//     let mut monitor = SignalMonitor::new(50, Box::new(FSK::default()));
//     monitor.print_header();

//     // Start the test sequence
//     println!("Starting audio loopback test...");
//     println!("Test data: {:?}", TEST_DATA);

//     // Send data through playback
//     let play_stream = router.send(TEST_DATA)?;

//     // Give some time for the signal to stabilize
//     thread::sleep(Duration::from_millis(100));

//     // Start listening
//     let (capture_stream, received_data) = router.listen()?;

//     // Monitor the signal for a short duration
//     for _ in 0..10 {
//         if let Some(decoded) = monitor.process_samples(&router.ModemTrait.encode(TEST_DATA)?) {
//             // println!("Decoded data: {:?}", decoded);
//             assert_eq!(decoded, TEST_DATA.to_vec(), "Decoded data doesn't match sent data");
//         }
//         thread::sleep(Duration::from_millis(100));
//     }

//     // Verify the received data
//     assert_eq!(received_data, TEST_DATA.to_vec(), "Received data doesn't match sent data");

//     // Clean up
//     play_stream.pause()?;
//     capture_stream.pause()?;

//     println!("Audio loopback test completed successfully!");
//     Ok(())
// }

fn test_signal_strength() -> Result<(), Box<dyn Error>> {
    let playback = AudioPlayback::new(Box::new(FSK::default()))?;
    let mut monitor = SignalMonitor::new(50, Box::new(FSK::default()));

    // Print monitor header
    monitor.print_header();

    // Test different signal strengths
    let test_signals = [
        (&[0x00], "0000 0000"), //   0
        (&[0x01], "0000 0001"), //   1
        (&[0x03], "0000 0011"), //   3
        (&[0x07], "0000 0111"), //   7
        (&[0x0F], "0000 1111"), //  15
        (&[0x1F], "0001 1111"), //  31
        (&[0x3F], "0011 1111"), //  63
        (&[0x7F], "0111 1111"), // 127
        (&[0xFF], "1111 1111"), // 255
    ];
    for (signal, description) in test_signals.iter() {
        println!("\nTesting {} ...", description);

        // let s = cast signal to &[u8]
        let s = signal.as_ref();
        // Send signal
        let stream = playback.transmit(s)?;
        // Monitor signal strength
        let samples = playback.modem.modulate(s)?;
        monitor.process_samples(&samples);

        thread::sleep(Duration::from_millis(250));
        stream.pause()?;
    }

    Ok(())
}

fn test_fsk_configs() -> Result<(), Box<dyn Error>> {
    let configs = [
        (1200.0, 2400.0, "Standard FSK"),
        (800.0, 1600.0, "Low Frequency FSK"),
        (2400.0, 4800.0, "High Frequency FSK"),
    ];

    for (freq_0, freq_1, description) in configs.iter() {
        println!(
            "\nTesting {} (f0={}, f1={})...",
            description, freq_0, freq_1
        );

        let codec = Box::new(FSK::new(48000, *freq_0, *freq_1, 480));
        let playback = AudioPlayback::new(codec)?;

        // Send test signal
        let stream = playback.transmit(TEST_DATA)?;
        thread::sleep(Duration::from_millis(500));
        stream.pause()?;
    }

    Ok(())
}

// // ? Some(test)

// fn test_frame_operations() {
//     info!("Testing Frame operations");

//     // Test frame creation
//     let test_data = b"Test Frame Data";
//     let frame = Frame::new(1, test_data.to_vec());

//     debug!("Created frame with sequence {}", frame.sequence);

//     // Test frame serialization
//     let frame_bytes = frame.to_bytes();
//     debug!("Frame serialized to {} bytes", frame_bytes.len());

//     // Test frame deserialization
//     match Frame::from_bytes(&frame_bytes) {
//         Ok(decoded_frame) => {
//             assert_eq!(decoded_frame.sequence, 1, "Sequence number mismatch");
//             assert_eq!(&decoded_frame.payload, test_data, "Payload mismatch");
//             info!("Frame serialization/deserialization test passed");
//         }
//         Err(e) => {
//             error!("Frame deserialization failed: {:?}", e);
//         }
//     }
// }

fn test_fsk_codec() {
    info!("Testing FSK codec");

    let fsk = FSK::new(48000, 1200.0, 2400.0, 480);

    let data = b"Hello, World!";

    match fsk.modulate(data) {
        Ok(encoded) => {
            debug!("Data encoded: {} samples", encoded.len());
            match fsk.demodulate(&encoded) {
                Ok(decoded) => {
                    trace!("\tOriginal data: {:?}", data);
                    trace!("\tDecoded data:  {:?}", decoded);
                    // if decoded == data {
                    //     info!("FSK codec test passed");
                    // } else {
                    //     error!("FSK codec test failed: data mismatch");
                    // }
                }
                Err(e) => error!("Decoding error: {}", e),
            }
        }
        Err(e) => error!("Encoding error: {}", e),
    }
}

fn test_codec() {
    info!("Main tester");

    let fsk = FSK::new(
        48000,  // 48kHz sample rate
        1200.0, // 1200 Hz for bit 0
        2400.0, // 2400 Hz for bit 1
        480,    // 480 samples per bit (100 bps)
    );

    // Encoding
    let data = b"Hello, World!";
    let encoded = match fsk.modulate(data) {
        Ok(encoded) => encoded,
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    };

    // Decoding
    // let decoded = fsk.decode(&encoded)?;
    let decoded = match fsk.demodulate(&encoded) {
        Ok(decoded) => decoded,
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    };

    // Compare original and decoded data
    println!("Original data: {:?}", data);
    println!("Decoded data: {:?}", decoded);
    assert_eq!(
        data.to_vec(),
        decoded,
        "Decoded data should match original data"
    );
}
