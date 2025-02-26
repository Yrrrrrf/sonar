#![allow(unused)] // silence unused warnings while developing

use std::time::Duration;
// todo: Add some prelude to the dev_utils crate...
use dev_utils::{app_dt, dlog::*};
use sonar::modem::{Frame, MacAddress};

fn main() {
    app_dt!(file!());
    set_max_level(Level::Trace);

    // Sample MAC addresses for source and destination.
    let src: MacAddress = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    let dst: MacAddress = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66];

    // Generate 4000 bytes of data by cycling through 0-255.
    const DATA_SIZE: usize = 1280;
    // const DATA_SIZE: usize = 128*16;
    let data: Vec<u8> = (0..DATA_SIZE).map(|i| (i % 256) as u8).collect();

    // Create the frame with the sample data.
    let frame = Frame::new_dt(src, dst, data);

    // Print the generated frame with pretty (and colored) formatting.
    println!("Generated Frame:\n{}", frame);
    // println!("Generated Frame:\n{:#?}", frame);
}
