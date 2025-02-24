#![allow(unused)] // silence unused warnings while developing

use std::time::Duration;

use dev_utils::{app_dt, debug, dlog::*, error, format::*, info, trace, warn};
use sonar::modem::{Frame, MacAddress};

// import some::*; from parent dir

// Example usage in main
fn main() {
    app_dt!(file!());
    set_max_level(Level::Trace);

    // Sample MAC addresses for source and destination.
    let src: MacAddress = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    let dst: MacAddress = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66];

    // Generate some sample data (for example, 256 bytes).
    let data: Vec<u8> = (0..=255).collect();
    // now a 4000 bytes data ()jusrt rememeber u8 just goes from 0..255, so 4000 is a lot of data)
    // let data: Vec<u8> = (0..=4000).collect(); so this won't work, we need to use a loop to generate the data
    let mut data: Vec<u8> = Vec::new();
    for i in 0..4000 {
        // add mod of 256 to keep the data in the range of u8
        data.push(i as u8);
    }

    // Create the frame with the sample data.
    let frame = Frame::new_dt(src, dst, data);

    // Print the generated frame with pretty formatting.
    // println!("Generated Frame: {:?}", frame);
}
