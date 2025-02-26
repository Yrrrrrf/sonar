// * Common

use std::fmt::{self, Display, Formatter};

use dev_utils::format::*;


// // Frame Flags:
// const F_FRAGMENT: u8 = 0x01; // Indicates frame is part of larger message
// const F_PRIORITY: u8 = 0x02; // High priority frame
// const F_CONTROL: u8 = 0x04; // Control frame (not data)
// const F_RETRANSMIT: u8 = 0x08; // Frame is being retransmitted

/// A generic container for a pair of addresses.
pub type AddressPair<A> = (A, A);

macro_rules! define_addresses {
    ($($(#[$meta:meta])* $name:ident: $inner:ty),* $(,)?) => {
        // New generic Header struct to handle address pairs at any layer.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct Header<A> {
            pub addresses: AddressPair<A>,
        }

        impl<A: Default> Default for Header<A> {
            fn default() -> Self {
                Self {
                    addresses: (A::default(), A::default()),
                }
            }
        }

        $(
            $(#[$meta])*
            pub type $name = $inner;

            impl Header<$name> {
                pub fn new(src: $inner, dst: $inner) -> Self {
                    Self {
                        addresses: (src, dst),
                    }
                }
            }

        )*
    };
}

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
            #[derive(Clone, PartialEq)]
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
        )*
    }
}

define_addresses! {
    /// Represents a MAC address.
    MacAddress: [u8; 6],
    /// Represents an IPv4 address.
    Ipv4Address: u32,
    /// Represents a PORT address.
    PortAddress: u16,
    // /// Represents an IPv6 address.
    // Ipv6Address: u128,
}

define_layer_struct! {
    // * Transport Layer (+mac)
    /// Represents a transport layer segment.
    Segment { header: PortAddress, payload: Vec<u8> },
    // * Network Layer (+ip)
    /// Represents a network layer packet.
    Packet { header: Ipv4Address, payload: Vec<Segment> },
    // * Data Link Layer
    /// Represents a data link layer frame.
    Frame { header: MacAddress, network_pdu: Vec<Packet> },
}

/// Display for a Segment: shows its payload (hex formatted) with dim styling.
impl Display for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Simply display the payload in hex with dim styling.
        write!(f, "{}", format!("{:X?}", self.payload).style(Style::Dim))
    }
}

/// Display for a Packet: shows header info in bold, then prints each Segment.
/// Note: This impl does not include the packet ID (since a Packet doesn’t know its own ID),
/// so the Frame impl will add the packet index.
impl Display for Packet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Print packet header info in bold.
        writeln!(
            f,
            "{}",
            format!(
                "src: {:X?} -> dst: {:X?} | segments: {}",
                self.header.addresses.0,
                self.header.addresses.1,
                self.payload.len()
            )
            .style(Style::Bold)
        )?;
        // Print each segment on its own line.
        for (seg_idx, segment) in self.payload.iter().enumerate() {
            writeln!(
                f,
                "    Segment {}: {}",
                // Here we print the segment id (without packet id) in dim style.
                format!("[id-{}]", seg_idx).style(Style::Dim),
                segment // This uses the Display for Segment.
            )?;
        }
        Ok(())
    }
}

/// Display for a Frame: prints its header in bold, then iterates each Packet,
/// prepending a packet id so that the full hierarchy is visible.
impl Display for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Print Frame header info in bold.
        writeln!(
            f,
            "{}",
            format!(
                "Frame [src: {:X?} -> dst: {:X?} | total_packets: {}]",
                self.header.addresses.0,
                self.header.addresses.1,
                self.network_pdu.len()
            )
            .style(Style::Bold)
        )?;
        // For each packet, print the packet id and then its Display output.
        for (pkt_idx, packet) in self.network_pdu.iter().enumerate() {
            writeln!(
                f,
                "  Packet {}:",
                format!("[id-{}]", pkt_idx).style(Style::Dim)
            )?;
            // Indent each line of the Packet’s output.
            let packet_str = format!("{}", packet);
            for line in packet_str.lines() {
                writeln!(f, "    {}", line)?;
            }
        }
        Ok(())
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
        FrameKind::BitOriented { flag: 0b01111110 }
    }
}


impl Frame {
    /// Creates a new data frame by splitting the provided data into segments and packets.
    /// Each segment holds 16 bytes, and each packet contains 8 segments.
    pub fn new_dt(src: MacAddress, dst: MacAddress, data: Vec<u8>) -> Self {
        // Create the MAC header for the frame
        let header = Header::<MacAddress>::new(src, dst);
        println!("Creating frame with src: {:X?} and dst: {:X?}", src, dst);

        let segments: Vec<Segment> = data.chunks(16).enumerate()
            .map(|(i, chunk)| {
                println!("\t-> Segment {i:>4} with data: {}", format!("{:?}", chunk).style(Style::Dim));
                Segment::new(Header::<PortAddress>::default(), chunk.to_vec())
            }).collect();

        let packets: Vec<Packet> = segments.chunks(8).enumerate()
            .map(|(i, seg_chunk)| {
                // println!("{:?}", seg_chunk);
                // format!("Creating Packet {} with {} segment(s)", i, seg_chunk.len()).style(Style::Dim);
                // Create a Packet with a default Ipv4Address header and the segments as payload.
                Packet::new(Header::<Ipv4Address>::default(), seg_chunk.to_vec())
            }).collect();

        // println!("Frame creation complete with {} packet(s)", packets.len());
        Self {
            header,
            network_pdu: packets,
        }
    }

    // Rename parameter to avoid unused variable warning
    // pub fn gen_frame(frame_type: FrameKind) -> Vec<u8> {
    //     println!("Generating frame of type: {:?}", frame_type);
    //     // SIMULATE THAT USES THE frame type if needed in future
    //     vec![0]
    // }
}
