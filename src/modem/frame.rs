// * Common
use bytes::{BufMut, Bytes, BytesMut};
use std::fmt::{self, Display, Formatter};

use dev_utils::{format::*, info, trace};

use crate::modem::{Header, PortAddress, Segment};
// Fix: Don't try to import segment_utils module that doesn't exist yet

use super::{Ipv4Address, MacAddress};

// // Frame Flags:
// const F_FRAGMENT: u8 = 0x01; // Indicates frame is part of larger message
// const F_PRIORITY: u8 = 0x02; // High priority frame
// const F_CONTROL: u8 = 0x04; // Control frame (not data)
// const F_RETRANSMIT: u8 = 0x08; // Frame is being retransmitted

macro_rules! define_layer_struct {
    (
        $(
            $(#[$meta:meta])*
            $name:ident { header: $header_ty:ty, $payload_field:ident: $payload_ty:ty $(,)? }
        ),* $(,)?
    ) => {
        $(
            // Apply any doc comments or attributes provided.
            $(#[$meta])*
            #[derive(Clone, PartialEq, Debug)]
            pub struct $name {
                pub header: Header<$header_ty>,
                pub $payload_field: $payload_ty,
            }

            impl $name {
                /// Creates a new instance of the struct.
                pub fn new(header: Header<$header_ty>, $payload_field: $payload_ty) -> Self {
                    Self {
                        header,
                        $payload_field,
                    }
                }
            }

            impl Default for $name {
                fn default() -> Self {
                    Self {
                        header: Header::<$header_ty>::default(),
                        $payload_field: Default::default(),
                    }
                }
            }

            impl Display for $name {
                fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                    writeln!( f, "{}", format!(
                        "src: {:X?} -> dst: {:X?} | {:>4} {}'s |",
                        self.header.addresses.0,
                        self.header.addresses.1,
                        self.$payload_field.len(),
                        stringify!($payload_field)
                    ).style(Style::Italic)
                    )?;
                    for (idx, item) in self.$payload_field.iter().enumerate() {
                        writeln!(f, "\t{} {}: {}", stringify!($payload_field), idx, item)?;
                    }
                    Ok(())
                }
            }

            // * for item in frame { ... }
            // Iterator implementation for owned type
            impl IntoIterator for $name {
                type Item = <$payload_ty as IntoIterator>::Item;
                type IntoIter = <$payload_ty as IntoIterator>::IntoIter;

                fn into_iter(self) -> Self::IntoIter {
                    self.$payload_field.into_iter()
                }
            }

            // * for item in &frame { ... }
            // Iterator implementation for borrowed type
            impl<'a> IntoIterator for &'a $name {
                type Item = &'a <$payload_ty as IntoIterator>::Item;
                type IntoIter = std::slice::Iter<'a, <$payload_ty as IntoIterator>::Item>;

                fn into_iter(self) -> Self::IntoIter {
                    self.$payload_field.iter()
                }
            }

            // * for item in &mut frame { ... }
            // Iterator implementation for mutably borrowed type
            impl<'a> IntoIterator for &'a mut $name {
                type Item = &'a mut <$payload_ty as IntoIterator>::Item;
                type IntoIter = std::slice::IterMut<'a, <$payload_ty as IntoIterator>::Item>;

                fn into_iter(self) -> Self::IntoIter {
                    self.$payload_field.iter_mut()
                }
            }
        )*
    }
}

define_layer_struct! {
    // * Network Layer (+ip)
    /// Represents a network layer packet.
    Packet { header: Ipv4Address, pdu: Vec<Segment> },
    // * Data Link Layer
    /// Represents a data link layer frame.
    Frame { header: MacAddress, network_pdu: Vec<Packet> },
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
    /// Creates a new data frame by splitting the provided data into segments and packets.
    /// Takes any type that can be referenced as a byte slice.
    pub fn new_dt(src: MacAddress, dst: MacAddress, data: impl AsRef<[u8]>) -> Self {
        // Create the MAC header for the frame
        let header = Header::<MacAddress>::new(src, dst);
        info!("Creating frame with src: {:X?} and dst: {:X?}", src, dst);

        // Get a reference to the data slice
        let data_slice = data.as_ref();

        // Process data into segments using builders for efficiency
        let segments: Vec<Segment> = data_slice
            .chunks(SEGMENT_SIZE)
            .map(|chunk| {
                let mut builder = Segment::builder(Header::<PortAddress>::default());
                builder.put_slice(chunk);
                builder.build()
            })
            .collect();

        // Group segments into packets
        let packets: Vec<Packet> = segments
            .chunks(PACKET_SIZE)
            .map(|seg_chunk| Packet::new(Header::<Ipv4Address>::default(), seg_chunk.to_vec()))
            .collect();

        let frame = Self {
            header,
            network_pdu: packets,
        };

        trace!("Frame created with {} packets", frame.network_pdu.len());
        for (i, packet) in frame.network_pdu.iter().enumerate() {
            trace!("  Packet {}: {} segments", i, packet.pdu.len());
        }

        frame
    }

    /// Creates a new data frame directly from Bytes
    pub fn new_dt_bytes(src: MacAddress, dst: MacAddress, data: Bytes) -> Self {
        Self::new_dt(src, dst, data.as_ref())
    }

    /// Extract all data from the frame as a single Bytes object
    pub fn extract_data(&self) -> Bytes {
        // Calculate total size to preallocate buffer with enough capacity
        let estimated_size = self.calculate_total_data_size();
        let mut buffer = BytesMut::with_capacity(estimated_size);

        // Extract data from all segments in all packets
        for packet in &self.network_pdu {
            for segment in &packet.pdu {
                buffer.extend_from_slice(segment.as_slice());
            }
        }

        buffer.freeze()
    }

    /// Calculate the total size of all data in the frame
    pub fn calculate_total_data_size(&self) -> usize {
        self.network_pdu
            .iter()
            .flat_map(|packet| packet.pdu.iter())
            .map(|segment| segment.len())
            .sum()
    }

    /// Get the number of packets in the frame
    pub fn packet_count(&self) -> usize {
        self.network_pdu.len()
    }

    /// Get the total number of segments in the frame
    pub fn segment_count(&self) -> usize {
        self.network_pdu.iter().map(|packet| packet.pdu.len()).sum()
    }

    /// Get a flattened iterator over all segments in the frame
    pub fn segments(&self) -> impl Iterator<Item = &Segment> + '_ {
        self.network_pdu.iter().flat_map(|packet| packet.pdu.iter())
    }
}

impl Packet {
    /// Calculate the total size of all data in the packet
    pub fn calculate_data_size(&self) -> usize {
        self.pdu.iter().map(|segment| segment.len()).sum()
    }

    /// Get a segment by index
    pub fn get_segment(&self, index: usize) -> Option<&Segment> {
        self.pdu.get(index)
    }

    /// Get a mutable segment by index
    pub fn get_segment_mut(&mut self, index: usize) -> Option<&mut Segment> {
        self.pdu.get_mut(index)
    }
}
