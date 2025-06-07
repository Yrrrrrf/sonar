// * std library imports
use std::error::Error;

// * module imports
pub mod fsk;
pub use fsk::FSK;

pub mod bpsk;
pub use bpsk::BPSK;

// pub mod qpsk;
// pub use qpsk::QPSK;

use crate::codec::{
    CodecTrait, SAMPLE_RATE, BAUD_RATE, SAMPLES_PER_BIT,
    byte_to_bits, bits_to_byte,
};