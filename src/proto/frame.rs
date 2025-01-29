use std::error::Error;
use bytes::{BytesMut, BufMut, Bytes, Buf};
use dev_utils::{
    app_dt, error, warn, info, debug, trace,
    dlog::*,
    format::*,
};

// Constants for frame structure
const SYNC_MARKER: u32 = 0xAAAAAAAA;
const END_MARKER: u16 = 0xFFFF;
const HEADER_SIZE: usize = 9;  // 4B sync + 1B version + 2B length + 1B sequence + 1B flags
const TRAILER_SIZE: usize = 8; // 2B CRC + 4B ECC + 2B end marker
const MAX_PAYLOAD_SIZE: usize = 1024;

// Frame flags
const FLAG_FRAGMENT: u8 = 0x01;     // Indicates frame is part of larger message
const FLAG_PRIORITY: u8 = 0x02;     // High priority frame
const FLAG_CONTROL: u8 = 0x04;      // Control frame (not data)
const FLAG_RETRANSMIT: u8 = 0x08;   // Frame is being retransmitted

#[derive(Debug, Clone)]
pub struct Frame {
    // ? Header fields
    version: u8,   // ? Protocol version
    sequence: u8,  // ? Frame sequence number
    flags: u8,     // ? Frame
    // * Data
    payload: Bytes,  // * Frame payload (the actual data)
    // ^ Trailer fields
    crc: u16,       // ^ CRC16 checksum
    ecc: [u8; 4],   // ^ Reed-Solomon ECC
}


impl Frame {
    /// Creates a new frame with given payload and sequence number
    pub fn new(payload: &[u8], sequence: u8) -> Result<Self, Box<dyn Error>> {
        if payload.len() > MAX_PAYLOAD_SIZE {
            return Err("Payload exceeds maximum size".into());
        }
        
        let mut frame = Frame {
            version: 1,  // Current protocol version
            sequence,
            flags: 0,    // Default flags
            payload: Bytes::copy_from_slice(payload),
            crc: 0,    // Will be calculated during encoding
            ecc: [0; 4], // Will be calculated during encoding
        };
        
        frame.crc = frame.calculate_crc();
        frame.calculate_ecc();
        
        Ok(frame)
    }

    pub fn serialize(&self) -> BytesMut {
        let mut buffer = BytesMut::with_capacity(
            HEADER_SIZE + self.payload.len() + TRAILER_SIZE
        );
        
        // Write header
        buffer.put_u32(SYNC_MARKER);
        buffer.put_u8(self.version);
        buffer.put_u16(self.payload.len() as u16);
        buffer.put_u8(self.sequence);
        buffer.put_u8(self.flags);
        
        // Write payload
        buffer.extend_from_slice(&self.payload);
        
        // Write trailer
        buffer.put_u16(self.crc);
        buffer.extend_from_slice(&self.ecc);
        buffer.put_u16(END_MARKER);
        
        buffer
    }

    pub fn deserialize(mut buffer: Bytes) -> Result<Option<Self>, Box<dyn Error>> {
        // Check minimum size
        if buffer.len() < HEADER_SIZE + TRAILER_SIZE {
            return Ok(None);  // Not enough data yet
        }
        
        // Look for sync marker
        let sync = buffer.get_u32();

        match sync != SYNC_MARKER {
            true => {
                info!("{}", "Invalid sync marker detected!".color(RED));
                return Ok(None);
            },
            false => info!("{}", "Valid sync marker found!".color(GREEN)),
        }
        
        // Read header fields
        let version = buffer.get_u8();
        let payload_len = buffer.get_u16() as usize;
        let sequence = buffer.get_u8();
        let flags = buffer.get_u8();
        
        // Validate total frame size
        if buffer.len() < payload_len + TRAILER_SIZE {
            return Ok(None);  // Incomplete frame
        }
        
        // Extract payload
        let payload = buffer.slice(..payload_len);
        buffer.advance(payload_len);
        
        // Read trailer
        let crc = buffer.get_u16();
        let mut ecc = [0u8; 4];
        buffer.copy_to_slice(&mut ecc);
        let end_marker = buffer.get_u16();
        
        // Validate end marker
        if end_marker != END_MARKER {
            info!("{}", "Invalid end marker detected!".color(RED));
            return Ok(None);
        } else {
            info!("{}", "Valid end marker found!".color(GREEN));
        }
        
        let frame = Frame {
            version,
            sequence,
            flags,
            payload,
            crc,
            ecc,
        };
        
        // Verify CRC
        if frame.calculate_crc() != crc {
            return Err("CRC mismatch".into());
        }
        
        Ok(Some(frame))
    }

    // TODO: Implement proper CRC16 calculation
    fn calculate_crc(&self) -> u16 {
        let mut sum: u16 = 0;
        sum = sum.wrapping_add(self.version as u16);
        sum = sum.wrapping_add(self.sequence as u16);
        sum = sum.wrapping_add(self.flags as u16);
        for byte in self.payload.iter() {
            sum = sum.wrapping_add(*byte as u16);
        }
        sum
    }

    // TODO: Implement proper Reed-Solomon ECC
    fn calculate_ecc(&mut self) {
        self.ecc = [0xAA, 0xBB, 0xCC, 0xDD];
    }

    // Getter methods
    pub fn sequence(&self) -> u8 { self.sequence }
    pub fn payload(&self) -> &Bytes { &self.payload }
    pub fn is_fragment(&self) -> bool { self.flags & FLAG_FRAGMENT != 0 }
    pub fn is_priority(&self) -> bool { self.flags & FLAG_PRIORITY != 0 }
    pub fn is_control(&self) -> bool { self.flags & FLAG_CONTROL != 0 }
    pub fn is_retransmit(&self) -> bool { self.flags & FLAG_RETRANSMIT != 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_creation() {
        let payload = b"Hello, World!";
        let frame = Frame::new(payload, 1).unwrap();
        assert_eq!(frame.sequence(), 1);
        assert_eq!(frame.payload().as_ref(), payload);
    }

    #[test]
    fn test_frame_encoding_decoding() {
        let payload = b"Test message";
        let original_frame = Frame::new(payload, 1).unwrap();
        
        let frame_serial = original_frame.serialize();
        let frame_deser= Frame::deserialize(frame_serial.freeze()).unwrap().unwrap();

        // Compare
        assert_eq!(frame_deser.sequence(), original_frame.sequence());
        assert_eq!(frame_deser.payload().as_ref(), payload);
    }

    #[test]
    fn test_max_payload_size() {
        let payload = vec![0u8; MAX_PAYLOAD_SIZE + 1];
        assert!(Frame::new(&payload, 1).is_err());
    }
}