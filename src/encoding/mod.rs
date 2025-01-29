
// * std library imports
use std::error::Error;

// * module imports
pub mod fsk;
pub use fsk::FSKEncoder;

pub trait Encoder {
    // Core encoding/decoding methods    // * Encode: bits -> signal
    fn encode(&self, data: &[u8]) -> Result<Vec<f32>, Box<dyn Error>>;
    // * Decode: signal -> bits
    fn decode(&self, samples: &[f32]) -> Result<Vec<u8>, Box<dyn Error>>;

    // * In Digital Logic, the Encoder & Decoder are some circuit that
    // * converts the input data into a format that is suitable for
    // * transmission over a communication channel.
    // & src -> [Encoder] -> dst (transmission) -> [Decoder] -> src
    // ^ 'dst' is information that is transmitted over a communication channel
    // ^ 'src' is the data that is to be transmitted
}


pub fn bytes_to_bits(byte: u8) -> Vec<bool> {
    (0..8).map(|i| ((byte >> (7 - i)) & 1) == 1).collect()
}

pub fn bits_to_bytes(bits: &[bool]) -> u8 {
    bits.iter().fold(0u8, |acc, &bit| (acc << 1) | if bit { 1 } else { 0 })
}
