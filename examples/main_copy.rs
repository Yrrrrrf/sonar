#![allow(unused)] // silence unused warnings while developing

use std::time::Duration;
// todo: Add some prelude to the dev_utils crate...
use dev_utils::{
    app_dt,
    dlog::*,
    format::{Style, Stylize},
};

use bytes::{BufMut, Bytes, BytesMut};
use sonar::stack::{Frame, Header, Ipv4Address, MacAddress, Packet, PortAddress, Segment};

use rand::Rng;

// * ON BYTES!
const SEGMENT_SIZE: usize = 16;
const PACKET_SIZE: usize = 8;

fn main() {
    app_dt!(file!());
    // set_max_level(Level::Error);
    // set_max_level(Level::Warn);
    // set_max_level(Level::Info);
    set_max_level(Level::Debug);
    // set_max_level(Level::Trace);

    // // Test with different sizes
    // let frame_sizes = [
    //     // 57, // * default + 1 byte
    //     300, // * default + 243 bytes
    //     // 20  * 1024, // * 20 KB
    // ];
    // frame_sizes.iter().for_each(|size| test_frame(*size));
}

// fn test_frame(size: usize) {
//     println!();

//     let data = define_dt(size);
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
//         frame.network_pdu.len(),
//         count_segments(&frame)
//     );

//     println!("{}", frame);
// }

// /// Creates test data of the exact requested size
// ///
// /// If the default message is smaller than the requested size, random data is added
// /// If the default message is larger than the requested size, it is truncated
// fn define_dt(message_size: usize) -> Bytes {
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

//     // Create segments from the data
//     let segments = create_segments(data);

//     // Create packets containing the segments
//     let packets = create_packets(segments);

//     // Create the frame with the packets
//     Frame {
//         header: Header::new(src_mac, dst_mac),
//         network_pdu: packets,
//     }
// }

// /// Creates segments from the provided data
// fn create_segments(data: Bytes) -> Vec<Segment> {
//     // Divide the data into segments of SEGMENT_SIZE
//     let mut segments = Vec::new();
//     let mut remaining = data;

//     while !remaining.is_empty() {
//         let chunk_size = std::cmp::min(SEGMENT_SIZE, remaining.len());
//         let chunk = remaining.slice(0..chunk_size);
//         remaining = remaining.slice(chunk_size..);

//         // Create a segment with source and destination port addresses
//         let src_port = 8080;
//         let dst_port = 80;
//         let segment = Segment {
//             header: Header::new(src_port, dst_port),
//             payload: chunk,
//         };

//         segments.push(segment);
//     }

//     segments
// }

// /// Creates packets containing the segments
// fn create_packets(segments: Vec<Segment>) -> Vec<Packet> {
//     // Divide the segments into packets, each containing PACKET_SIZE segments
//     let mut packets = Vec::new();

//     for chunk in segments.chunks(PACKET_SIZE) {
//         // Create a packet with source and destination IP addresses
//         let src_ip = 0xC0A80001; // 192.168.0.1
//         let dst_ip = 0xC0A80002; // 192.168.0.2
//         let packet = Packet {
//             header: Header::new(src_ip, dst_ip),
//             pdu: chunk.to_vec(),
//         };

//         packets.push(packet);
//     }

//     packets
// }

// /// Count the total number of segments in a frame
// fn count_segments(frame: &Frame) -> usize {
//     frame.network_pdu.iter().map(|packet| packet.pdu.len()).sum()
// }

// /// Extracts data from a frame and compares it with the expected data
// fn frame_extract(frame: &Frame, expected_data: &Bytes) {
//     warn!("Extracting data from frame...");

//     // Use iterators to show frame structure
//     for (idx, packet) in frame.into_iter().enumerate() {
//         trace!("{idx} {packet}");
//     }

//     // Extract the data from the frame
//     let extracted_data = extract_data(frame);

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

// /// Extracts data from a frame by concatenating all segment payloads
// fn extract_data(frame: &Frame) -> Bytes {
//     // Collect all segment payloads from all packets
//     let mut data = BytesMut::new();

//     for packet in &frame.network_pdu {
//         for segment in &packet.pdu {
//             data.put_slice(&segment.payload);
//         }
//     }

//     data.freeze()
// }
