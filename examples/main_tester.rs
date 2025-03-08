#![allow(unused)] // silence unused warnings while developing

use std::time::Duration;
// todo: Add some prelude to the dev_utils crate...
use dev_utils::{
    app_dt,
    dlog::*,
    format::{Style, Stylize},
};

use bytes::{BufMut, Bytes, BytesMut};
use sonar::modem::{
    Frame, Header, Ipv4Address, MacAddress, Packet, PortAddress, Segment,
};

use rand::Rng;

fn main() {
    app_dt!(file!());
    // set_max_level(Level::Error);
    // set_max_level(Level::Warn);
    // set_max_level(Level::Info);
    set_max_level(Level::Debug);
    // set_max_level(Level::Trace);

    // Test with different sizes

    let frame_sizes = [
        57, // * default + 1 byte
        300, // * default + 243 bytes
            // 20  * 1024, // * 20 KB
    ];
    frame_sizes.iter().for_each(|size| test_frame(*size));
}

fn test_frame(size: usize) {
    println!();

    // let data = define_dt(size);
//     info!(
//         "Created data with size: ({})",
//         format!("{} Bytes", data.len())
//             .style(Style::Dim)
//             .style(Style::Italic)
//     );

//     // Create and test frame
//     let frame = frame_create(data.clone());
//     frame_extract(&frame, &data);

//     // Display frame details
//     debug!(
//         "Frame contains {} packets and {} segments",
//         frame.packet_count(),
//         frame.segment_count()
//     );

//     println!("{}", frame);
//     // println!("\n");
}

// /// Creates test data of the exact requested size
// ///
// /// If the default message is smaller than the requested size, random data is added
// /// If the default message is larger than the requested size, it is truncated
// fn define_dt(message_size: usize) -> Bytes {
//     // todo: Check the
//     // let default_msg = b"🚀0Some new text with weird characters: !@#$%^&*()_+{}|:<>?";
//     let default_msg = b"Some new text with weird characters: !@#$%^&*()_+{}|:<>?";

//     // Create a BytesMut with the exact requested capacity
//     let mut buffer = BytesMut::with_capacity(message_size);

//     // Add as much of the default message as we can fit
//     let copy_len = std::cmp::min(default_msg.len(), message_size);
//     buffer.put_slice(&default_msg[0..copy_len]);

//     // If we need more data to reach the requested size, fill with random values
//     if buffer.len() < message_size {
//         let remaining = message_size - buffer.len();
//         let mut rng = rand::rng(); // Fixed: use thread_rng() instead of rng()

//         for _ in 0..remaining {
//             // todo: Hadle the put u16...
//             buffer.put_u8(rng.random_range(0..=255)); // Fixed: use gen_range instead of random_range
//         }
//     }

//     // The buffer should now be exactly message_size in length
//     debug_assert_eq!(buffer.len(), message_size);

//     // Freeze the buffer into an immutable Bytes
//     buffer.freeze()
// }

// /// Creates a frame from the provided Bytes data
// fn frame_create(data: Bytes) -> Frame {
//     // Example MAC addresses
//     let src_mac = [0x00, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E];
//     let dst_mac = [0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA];

//     warn!("Creating frame...");
//     // Use the dedicated method for Bytes input
//     Frame::new_dt_bytes(src_mac, dst_mac, data)
// }

// /// Extracts data from a frame and compares it with the expected data
// fn frame_extract(frame: &Frame, expected_data: &Bytes) {
//     warn!("Extracting data from frame...");

//     // Use iterators to show frame structure
//     for (idx, packet) in frame.into_iter().enumerate() {
//         trace!("{idx} {packet}");
//     }

//     // Extract the data from the frame
//     let extracted_data = frame.extract_data();

//     // Compare the extracted data with the original data
//     if extracted_data == *expected_data {
//         info!(
//             "Data extracted successfully! ({})",
//             format!("{} Bytes", extracted_data.len())
//                 .style(Style::Dim)
//                 .style(Style::Italic)
//         );
//     } else {
//         error!("Data extraction failed!");
//         debug!(
//             "Expected length: {}, Actual length: {}",
//             expected_data.len(),
//             extracted_data.len()
//         );

//         // If lengths match but content doesn't, try to find where they differ
//         if extracted_data.len() == expected_data.len() {
//             for (i, (a, b)) in extracted_data.iter().zip(expected_data.iter()).enumerate() {
//                 if a != b {
//                     error!("First difference at byte {}: {} vs {}", i, a, b);
//                     break;
//                 }
//             }
//         }
//     }
// }
