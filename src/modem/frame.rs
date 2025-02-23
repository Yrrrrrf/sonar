// * Common

// Frame flags
const FLAG_FRAGMENT: u8 = 0x01; // Indicates frame is part of larger message
const FLAG_PRIORITY: u8 = 0x02; // High priority frame
const FLAG_CONTROL: u8 = 0x04; // Control frame (not data)
const FLAG_RETRANSMIT: u8 = 0x08; // Frame is being retransmitted


macro_rules! define_addresses {
    ($($(#[$meta:meta])* $name:ident: $inner:ty),* $(,)?) => {
        $(
            $(#[$meta])*
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub struct $name($inner);
        )*
    };
}

// Using the macro to define multiple address types in a single call.
define_addresses!(
    /// Represents a MAC address.
    MacAddress: [u8; 6],
    /// Represents an IPv4 address.
    Ipv4Address: u32,
    /// Represents an IPv6 address.
    Ipv6Address: u128,
    /// Represents a PORT address.
    PortAddress: u16,
);

/// A generic container for a pair of addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddressPair<A> {
    pub src: A,
    pub dst: A,
}
// New generic Header struct to handle address pairs at any layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header<A> {pub addresses: AddressPair<A>,}
impl Header<MacAddress> {
    pub fn new(src: MacAddress, dst: MacAddress) -> Self {
        Self { addresses: AddressPair { src, dst } }
    }
}


// * Transport Layer (+mac)
#[derive(Debug)]
pub struct Segment {
    pub header: Header<PortAddress>,
    pub payload: Vec<u8>,
}

impl Segment {
    pub fn new(header: Header<PortAddress>, payload: Vec<u8>) -> Self {
        Self { header, payload }
    }
}


// * Network Layer (+ip)
#[derive(Debug)]
pub struct Packet {
    pub header: Header<Ipv4Address>, // IP header with source and destination addresses.
    pub payload: Segment,            // Transport layer segment.
}

impl Packet {
    pub fn new(header: Header<Ipv4Address>, payload: Segment) -> Self {
        Self { header, payload }
    }
    
    pub fn validate(&self) -> bool {unimplemented!()}
}


// * Data Link Layer (+ip)
/// Enum representing the frame types, with frame-specific data embedded.
#[derive(Debug, Clone, Copy)]
pub enum FrameKind {
    BitOriented { flag: u8 },
    BySync { sync: u8 },
    DDCMP { control: u8 },
    AsyncPPP { start_delim: u8, end_delim: u8 },
}

impl Default for FrameKind {
    fn default() -> Self {FrameKind::BitOriented { flag: 0b01111110 }}
    // fn default() -> Self {FrameKind::DDCMP { control: 0 }}
    // fn default() -> Self {FrameKind::AsyncPPP { start_delim: 0b01111110, end_delim: 0b01111110 }}
}

/// Represents a frame in the data link layer.
#[derive(Debug)]
pub struct Frame {
    pub header: Header<MacAddress>,
    pub kind: FrameKind,
    pub network_pdu: Vec<Packet>,
}

impl Frame {
    pub fn new(
        frame_header: Header<MacAddress>,
        network_header: Header<Ipv4Address>,
        transport_header: Header<PortAddress>,
        data: Vec<u8>,
        kind: Option<FrameKind>,
    ) -> Self {
        // Create the transport layer segment and network layer packet.
        let segment = Segment::new(transport_header, data);
        let packet = Packet::new(network_header, segment);
        let network_pdus = vec![packet];

        Self {
            header: frame_header,
            network_pdu: network_pdus,
            kind: kind.unwrap_or_default(), // Defaults to DDCMP if not provided.
        }
    }

    pub fn simple_frame(
        src: MacAddress,
        dst: MacAddress,
        data: Vec<u8>,
    ) -> Self {

        // todo: Impl this to be able to generate a simple frame...
        // todo: Later on then, improve the code to be able to generate a frame with multiple packets.
        Self {
            header: Header::new(src, dst),
            network_pdu: vec![],
            kind: FrameKind::default(),
        }

    }
}
