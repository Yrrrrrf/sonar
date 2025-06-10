// * std library imports
use std::error::Error;

// * module imports
pub mod fsk;
pub use fsk::FSK;

pub mod bpsk;
pub use bpsk::BPSK;

// pub mod qpsk;
// pub use qpsk::QPSK;


pub const SAMPLE_RATE: u32 = 48_000; // 48 kHz
pub const BAUD_RATE: u32 = 1_200; // 1.2 kbps
pub const SAMPLES_PER_BIT: u32 = SAMPLE_RATE / BAUD_RATE;

pub fn byte_to_bits<T>(byte: u8) -> Vec<T>
where
    T: From<bool>,
{
    (0..8)
        .map(|i| T::from(((byte >> (7 - i)) & 1) == 1))
        .collect()
}

pub fn bits_to_byte<T>(bits: &[T]) -> u8
where
    T: Into<bool> + Copy,
{
    bits.iter()
        .fold(0u8, |acc, &bit| (acc << 1) | if bit.into() { 1 } else { 0 })
}

// Core encoding/decoding methods
pub trait ModemTrait {
    // * Encode: bits -> signal
    fn modulate(&self, data: &[u8]) -> Result<Vec<f32>, Box<dyn Error>>;
    // * Decode: signal -> bits
    fn demodulate(&self, samples: &[f32]) -> Result<Vec<u8>, Box<dyn Error>>;
}

// same as above but using some macro to reduce boilerplate...
// * macro to reduce boilerplate
macro_rules! impl_codec {
    (
        $($codec:ident),* $(,)?
    ) => {
        pub enum Codec {
            $(
                $codec($codec),
            )*
        }

        impl Codec {
            pub fn modulate(&self, data: &[u8]) -> Result<Vec<f32>, Box<dyn Error>> {
                match self {
                    $(
                        Codec::$codec(codec) => codec.modulate(data),
                    )*
                }
            }

            // todo: receive some signal samples...
            pub fn demodulate(&self, samples: &[f32]) -> Result<Vec<u8>, Box<dyn Error>> {
                match self {
                    $(
                        Codec::$codec(codec) => codec.demodulate(samples),
                    )*
                }
            }
        }
    };
}

impl_codec!(
    FSK, BPSK,
    // QPSK,
);
