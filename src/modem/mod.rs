mod frame;
pub use frame::*;
mod segment;
pub use segment::*;

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
    ($($(#[$meta:meta])* $name:ident: $inner:ty, $default:expr),* $(,)?) => {

        $(
            $(#[$meta])*
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
