// * Common
use bytes::{BufMut, Bytes, BytesMut};

use dev_utils::{format::*, info, trace};

use crate::modem::{Frame, Packet, Segment, Header, PortAddress };

use super::{Ipv4Address, MacAddress};

// // Frame Flags:
// const F_FRAGMENT: u8 = 0x01; // Indicates frame is part of larger message
// const F_PRIORITY: u8 = 0x02; // High priority frame
// const F_CONTROL: u8 = 0x04; // Control frame (not data)
// const F_RETRANSMIT: u8 = 0x08; // Frame is being retransmitted




impl Frame {
    pub fn get_message(&self) -> &str {
        panic!("Not implemented")
    }
}


/// Enum representing the frame types, with frame-specific data embedded.
#[derive(Debug, Clone, Copy)]
pub enum FrameKind {
    BitOriented { flag: u8 },
    BySync { sync: u8 },
    DDCMP { control: u8 },
    AsyncPPP { start_delim: u8, end_delim: u8 },
}

impl Default for FrameKind {
    fn default() -> Self {
        // & 0b01111110  == 0x7E == 126 == 0
        FrameKind::BitOriented { flag: 0x7E }
        // FrameKind::BySync { sync: 0x7E }
        // FrameKind::DDCMP { control: 0x03 }
        // FrameKind::AsyncPPP { start_delim: 0x7E, end_delim: 0x7E }
    }
}

// * ON BYTES!
const SEGMENT_SIZE: usize = 32;
const PACKET_SIZE: usize = 4;


impl Frame {
}

impl Packet {
}

impl Segment {
}
