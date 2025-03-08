// src/modem/mod.rs
mod frame;
use bytes::Bytes;
pub use frame::*;
mod segment;
pub use segment::*;

use std::fmt::{self, Display, Formatter};
use dev_utils::format::*;

/// A generic container for a pair of addresses.
pub type AddressPair<A> = (A, A);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header<A> {
    pub addresses: AddressPair<A>,
}

impl<A> Header<A> {
    pub fn new(src: A, dst: A) -> Self {
        Self {
            addresses: (src, dst),
        }
    }
    pub fn src(&self) -> &A {
        &self.addresses.0
    }
    pub fn dst(&self) -> &A {
        &self.addresses.1
    }
}

macro_rules! define_addresses {
    ($($(#[$meta_d:meta])* $name:ident: $inner:ty, $default:expr),* $(,)?) => {

        $(
            $(#[$meta_d])*
            pub type $name = $inner;

            impl Default for Header<$name> {
                fn default() -> Self {
                    let default_addr: $name = $default;
                    Self {addresses: (default_addr, default_addr),}
                }
            }

            // Implement conversion methods for specific address types if needed
            impl Header<$name> {
                // Additional type-specific methods can go here
            }
        )*

    };
}

define_addresses! {
    /// Represents a MAC address.
    MacAddress: [u8; 6], [0, 0, 0, 0, 0, 0],
    /// Represents an IPv4 address.
    Ipv4Address: u32, 0x7F000001, // 127.0.0.1
    /// Represents a PORT address.
    PortAddress: u16, 80,
}


macro_rules! impl_iterator_trait {
    ($name:ident, $payload_field:ident, $payload_ty:ty) => {
        // * for item in layer_struct { ... }
        impl IntoIterator for $name {
            type Item = <$payload_ty as IntoIterator>::Item;
            type IntoIter = <$payload_ty as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                self.$payload_field.into_iter()
            }
        }

        // * for item in &layer_struct { ... }
        impl<'a> IntoIterator for &'a $name {
            type Item = &'a <$payload_ty as IntoIterator>::Item;
            type IntoIter = std::slice::Iter<'a, <$payload_ty as IntoIterator>::Item>;

            fn into_iter(self) -> Self::IntoIter {
                self.$payload_field.iter()
            }
        }        
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
            #[derive(Clone, PartialEq, Debug)]
            pub struct $name {
                pub header: Header<$header_ty>,
                pub $payload_field: $payload_ty,
            }

            impl $name {
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

            impl_iterator_trait!($name, $payload_field, $payload_ty);
        )*
    }
}

define_layer_struct! {
    // * Transport Layer (+any-byte-stream)
    /// Represents a transport layer segment.
    Segment { header: PortAddress, payload: Bytes },
    // * Network Layer (+ip)
    /// Represents a network layer packet.
    Packet { header: Ipv4Address, pdu: Vec<Segment> },
    // * Data Link Layer
    /// Represents a data link layer frame.
    Frame { header: MacAddress, network_pdu: Vec<Packet> },
}



// todo: Implement the following modules
// todo: Implement the following modules
// todo: Implement the following modules

// Flow control modules (commented for future implementation)
// mod flow_control;
// -> flow-control
//     - sliding-window
//     - congestion-control
//     - rate-control
// - stop-and-wait
// - go-back-n
// - selective-repeat

// Error handling modules (commented for future implementation)
// mod error_handling;
// -> error-control
//     - crc
//     - hamming
//     - reed-solomon
//     - convolutional-coding
//     - turbo-coding
//     - ldpc
//     - polar
//     - fountain
//     - etc...
