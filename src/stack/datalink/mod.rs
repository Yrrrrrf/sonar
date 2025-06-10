mod frame;
pub use frame::*;

// Potentially others in the future like `flow_control.rs`
use std::error::Error;

use crate::modem::ModemTrait;

const PREAMBLE: [bool; 32] = [
    true, false, true, false, true, false, true, false, true, false, true, false, true, false,
    true, false, true, false, true, false, true, false, true, false, true, false, true, false,
    true, false, true, false,
];
const START_OF_FRAME: [bool; 8] = [false, true, true, true, true, true, true, false]; // 0x7E
const LENGTH_FIELD_BITS: usize = 16;
const CRC_FIELD_BITS: usize = 16;

// Core encoding/decoding methods
pub trait CodecTrait {
    // Takes USER DATA -> Returns AUDIO SAMPLES
    // * Encode: bits -> signal
    fn encode<T: AsRef<[u8]>>(&self, payload: T) -> Result<Vec<f32>, Box<dyn Error>>;

    // * Decode: signal -> bits
    // Takes AUDIO SAMPLES -> Returns USER DATA (if a full frame is found)
    fn decode(&mut self, samples: &[f32]) -> Result<Option<Vec<u8>>, Box<dyn Error>>;
}

pub struct SonarCodec {
    modem: Box<dyn ModemTrait>,
    decoder_state: DecoderState,
    bit_buffer: Vec<bool>,
    expected_payload_bits: usize,
}

// Define the private state enum
enum DecoderState {
    SearchingForSOF,
    ReadingHeader,
    ReadingPayload,
}

impl SonarCodec {
    pub fn new(modem: Box<dyn ModemTrait>) -> Self {
        Self {
            modem,
            decoder_state: DecoderState::SearchingForSOF,
            bit_buffer: Vec::new(),
            expected_payload_bits: 0,
        }
    }

    fn build_physical_frame(payload: &[u8]) -> Result<Vec<bool>, Box<dyn Error>> {
        use crc::{CRC_16_KERMIT, Crc};
        const KERMIT: Crc<u16> = Crc::<u16>::new(&CRC_16_KERMIT);

        // 1. Check payload size
        if payload.len() > u16::MAX as usize {
            return Err("Payload too large".into());
        }

        // 2. Calculate CRC
        let crc_val = KERMIT.checksum(payload);

        // 3. Assemble bitstream
        let mut bitstream = Vec::new();
        bitstream.extend_from_slice(&PREAMBLE);
        bitstream.extend_from_slice(&START_OF_FRAME);

        // Append length (as 16 bits, big-endian)
        for i in (0..16).rev() {
            bitstream.push((payload.len() >> i) & 1 == 1);
        }

        // Append payload
        for &byte in payload {
            for i in (0..8).rev() {
                bitstream.push((byte >> i) & 1 == 1);
            }
        }

        // Append CRC (as 16 bits, big-endian)
        for i in (0..16).rev() {
            bitstream.push((crc_val >> i) & 1 == 1);
        }

        Ok(bitstream)
    }

    fn process_buffer(&mut self) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
        // This is where the state machine loop goes.
        // loop { match self.decoder_state { ... } }
        // ... as designed in the previous blueprint ...

        // This function will need a helper to find a sub-slice pattern (the SOF)
        // and a helper to convert bits back to bytes/u16.
        Ok(None) // Default return
    }
}

impl CodecTrait for SonarCodec {
    fn encode<T: AsRef<[u8]>>(&self, payload: T) -> Result<Vec<f32>, Box<dyn Error>> {
        let physical_frame_bits = Self::build_physical_frame(payload.as_ref())?;
        self.modem.modulate(&physical_frame_bits)
    }

    fn decode(&mut self, samples: &[f32]) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
        let new_bits = self.modem.demodulate(samples)?;
        if !new_bits.is_empty() {
            self.bit_buffer.extend_from_slice(&new_bits);
        }
        self.process_buffer()
    }
}

// // same as above but using some macro to reduce boilerplate...
// // * macro to reduce boilerplate
// macro_rules! impl_codec {
//     (
//         $($codec:ident),* $(,)?
//     ) => {
//         pub enum Codec {
//             $(
//                 $codec($codec),
//             )*
//         }

//         impl Codec {
//             pub fn encode(&self, data: &[u8]) -> Result<Vec<f32>, Box<dyn Error>> {
//                 match self {
//                     $(
//                         Codec::$codec(codec) => codec.encode(data),
//                     )*
//                 }
//             }

//             pub fn decode(&self, samples: &[f32]) -> Result<Vec<u8>, Box<dyn Error>> {
//                 match self {
//                     $(
//                         Codec::$codec(codec) => codec.decode(samples),
//                     )*
//                 }
//             }
//         }
//     };
// }

// impl_codec!(
// add some modems to implement CodecTrait...
// );
