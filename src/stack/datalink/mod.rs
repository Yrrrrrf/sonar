// C:\...\sonar\src\stack\datalink\mod.rs

mod frame;
use dev_utils::{debug, info, trace, warn};
pub use frame::*;

use crate::modem::ModemTrait;
use crc::{CRC_16_KERMIT, Crc};
use std::error::Error;

// --- Constants ---
const PREAMBLE: [bool; 32] = [
    true, false, true, false, true, false, true, false, true, false, true, false, true, false,
    true, false, true, false, true, false, true, false, true, false, true, false, true, false,
    true, false, true, false,
];
const START_OF_FRAME: [bool; 8] = [false, true, true, true, true, true, true, false]; // 0x7E
const LENGTH_FIELD_BITS: usize = 16;
const CRC_FIELD_BITS: usize = 16; // 16 bits for CRC-16 Kermit
const HEADER_BITS: usize = LENGTH_FIELD_BITS; // For now, header is just the length

const KERMIT_CRC: Crc<u16> = Crc::<u16>::new(&CRC_16_KERMIT);

// --- Public API ---
pub trait CodecTrait {
    fn encode<T: AsRef<[u8]>>(&self, payload: T) -> Result<Vec<f32>, Box<dyn Error>>;
    fn decode(&mut self, samples: &[f32]) -> Result<Option<Vec<u8>>, Box<dyn Error>>;
}

pub struct SonarCodec {
    modem: Box<dyn ModemTrait>,
    decoder_state: DecoderState,
    bit_buffer: Vec<bool>,
    expected_payload_bits: usize,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DecoderState {
    SearchingForPreamble,
    SearchingForSOF,
    ReadingHeader,
    ReadingPayload,
}

// --- Implementation ---
impl SonarCodec {
    pub fn new(modem: Box<dyn ModemTrait>) -> Self {
        Self {
            modem,
            decoder_state: DecoderState::SearchingForPreamble,
            bit_buffer: Vec::new(),
            expected_payload_bits: 0,
        }
    }

    pub fn reset_decoder(&mut self) {
        self.decoder_state = DecoderState::SearchingForPreamble;
        self.decoder_state = DecoderState::SearchingForSOF;

        self.bit_buffer.clear();
        self.expected_payload_bits = 0;
        info!("Decoder state has been reset.");
    }

    // ADD THIS METHOD
    /// Returns the current state and buffer size for monitoring.
    pub fn get_decoder_status(&self) -> (DecoderState, usize) {
        (self.decoder_state.clone(), self.bit_buffer.len())
    }

    /// Assembles a full physical frame from a payload.
    fn build_physical_frame(payload: &[u8]) -> Result<Vec<bool>, Box<dyn Error>> {
        if payload.len() > u16::MAX as usize {
            return Err("Payload too large for 16-bit length field".into());
        }

        let crc_val = KERMIT_CRC.checksum(payload);

        // Pre-calculate capacity to avoid re-allocations
        let total_bits = PREAMBLE.len()
            + START_OF_FRAME.len()
            + LENGTH_FIELD_BITS
            + payload.len() * 8
            + CRC_FIELD_BITS;
        let mut bitstream = Vec::with_capacity(total_bits);

        bitstream.extend_from_slice(&PREAMBLE);
        bitstream.extend_from_slice(&START_OF_FRAME);

        // Append length (as 16 bits, big-endian)
        for i in (0..16).rev() {
            bitstream.push(((payload.len() as u16) >> i) & 1 == 1);
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

    /// The core state machine for processing the internal bit buffer.
    fn process_buffer(&mut self) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
        loop {
            trace!(
                "State: {:?}, Buffer size: {}",
                self.decoder_state,
                self.bit_buffer.len()
            );

            match self.decoder_state {
                DecoderState::SearchingForPreamble => {
                    const MIN_PREAMBLE_BITS: usize = 16; // Must detect at least 16 bits of preamble
                    const PREAMBLE_PATTERN: [bool; 2] = [true, false];

                    if let Some(index) =
                        self.bit_buffer
                            .windows(MIN_PREAMBLE_BITS)
                            .position(|window| {
                                // Check if the window consists of repeating '10' pattern
                                window.chunks(2).all(|chunk| chunk == PREAMBLE_PATTERN)
                            })
                    {
                        // Found a stable preamble!
                        // Discard everything before it.
                        self.bit_buffer.drain(..index);
                        debug!("Preamble locked. Now searching for SOF.");
                        self.decoder_state = DecoderState::SearchingForSOF;
                        // Continue loop to immediately check for SOF in the remaining buffer
                    } else {
                        // No stable preamble found yet.
                        // To prevent the buffer from growing forever with noise,
                        // we can keep it trimmed.
                        if self.bit_buffer.len() > 256 {
                            self.bit_buffer.drain(..self.bit_buffer.len() - 256);
                        }
                        return Ok(None);
                    }
                }

                DecoderState::SearchingForSOF => {
                    // Now we only look for SOF if a preamble was already found.
                    if let Some(index) = find_sof_pattern(&self.bit_buffer) {
                        self.bit_buffer.drain(..index + START_OF_FRAME.len());
                        debug!("SOF found. Moving to ReadingHeader.");
                        self.decoder_state = DecoderState::ReadingHeader;
                    } else {
                        // If we don't find the SOF within a reasonable number of bits
                        // after the preamble, assume it was a false preamble and reset.
                        if self.bit_buffer.len() > 64 {
                            // Arbitrary threshold
                            warn!("Found preamble but no SOF followed. Resetting search.");
                            self.reset_decoder();
                        }
                        return Ok(None);
                    }
                }

                DecoderState::ReadingHeader => {
                    if self.bit_buffer.len() >= HEADER_BITS {
                        let header_bits = self.bit_buffer.drain(..HEADER_BITS).collect::<Vec<_>>();
                        let payload_len_bytes = bits_to_u16(&header_bits);

                        // ANOTHER IMPORTANT CHECK: Is the length sane?
                        const MAX_SANE_PAYLOAD_BYTES: u16 = 4096; // 4KB
                        if payload_len_bytes == 0 || payload_len_bytes > MAX_SANE_PAYLOAD_BYTES {
                            warn!(
                                "Received insane payload length: {}. Likely noise. Resetting.",
                                payload_len_bytes
                            );
                            self.reset_decoder();
                            // Continue loop to restart search immediately
                            continue;
                        }

                        self.expected_payload_bits = payload_len_bytes as usize * 8;
                        debug!(
                            "Header read. Expecting payload of {} bytes.",
                            payload_len_bytes
                        );
                        self.decoder_state = DecoderState::ReadingPayload;
                    } else {
                        return Ok(None);
                    }
                }

                DecoderState::ReadingPayload => {
                    let total_frame_bits = self.expected_payload_bits + CRC_FIELD_BITS;
                    if self.bit_buffer.len() >= total_frame_bits {
                        // We have a full frame! Let's process it.
                        let payload_bits: Vec<bool> = self
                            .bit_buffer
                            .drain(..self.expected_payload_bits)
                            .collect();
                        let received_crc_bits: Vec<bool> =
                            self.bit_buffer.drain(..CRC_FIELD_BITS).collect();

                        let payload_bytes = bits_to_bytes(&payload_bits);
                        let calculated_crc = KERMIT_CRC.checksum(&payload_bytes);
                        let received_crc = bits_to_u16(&received_crc_bits);

                        // Reset state for the next frame search, regardless of CRC outcome.
                        self.decoder_state = DecoderState::SearchingForSOF;

                        if calculated_crc == received_crc {
                            info!(
                                "CRC MATCH! Calculated: {:04X}, Received: {:04X}. Frame is valid!",
                                calculated_crc, received_crc
                            );
                            return Ok(Some(payload_bytes));
                        } else {
                            warn!(
                                "CRC MISMATCH! Calculated: {:04X}, Received: {:04X}. Discarding corrupt frame.",
                                calculated_crc, received_crc
                            );
                            // The corrupt frame data is already drained, so we just loop again to search for the next SOF.
                        }
                    } else {
                        // Not enough bits for the full payload + CRC yet.
                        return Ok(None);
                    }
                }
            }
        }
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
            trace!("Demodulated {} new bits.", new_bits.len());
        }
        self.bit_buffer.extend_from_slice(&new_bits);
        self.process_buffer()
    }
}

// --- Helper Functions ---

/// Finds the first occurrence of the START_OF_FRAME pattern in a bit buffer.
fn find_sof_pattern(buffer: &[bool]) -> Option<usize> {
    buffer
        .windows(START_OF_FRAME.len())
        .position(|window| window == START_OF_FRAME)
}

/// Converts a slice of bools (bits, MSB first) to a Vec<u8>.
fn bits_to_bytes(bits: &[bool]) -> Vec<u8> {
    bits.chunks(8)
        .map(|chunk| chunk.iter().fold(0u8, |acc, &bit| (acc << 1) | (bit as u8)))
        .collect()
}

/// Converts a slice of up to 16 bools (MSB first) to a u16.
fn bits_to_u16(bits: &[bool]) -> u16 {
    bits.iter()
        .fold(0u16, |acc, &bit| (acc << 1) | (bit as u16))
}
