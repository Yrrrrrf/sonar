use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::fmt::{self, Display, Formatter};

use dev_utils::format::*;

use crate::modem::{Header, PortAddress};

/// Represents a segment at the transport layer with efficient byte handling
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    /// Port address represents the source port
    pub header: PortAddress,
    /// Payload data stored in an efficient Bytes container
    pub(crate) payload: Bytes,
}

impl Segment {
    /// Creates a new segment with the given header and payload
    pub fn new(header: Header<PortAddress>, payload: impl Into<Bytes>) -> Self {
        Self {
            header: header.addresses.0, // Using the source address
            payload: payload.into(),
        }
    }

    /// Creates a segment from raw bytes
    pub fn from_bytes(header: Header<PortAddress>, bytes: &[u8]) -> Self {
        Self {
            header: header.addresses.0,
            payload: Bytes::copy_from_slice(bytes),
        }
    }

    /// Creates a segment builder for efficiently constructing segments
    pub fn builder(header: Header<PortAddress>) -> SegmentBuilder {
        SegmentBuilder {
            header: header.addresses.0,
            payload: BytesMut::new(),
        }
    }

    /// Returns the length of the payload in bytes
    pub fn len(&self) -> usize {
        self.payload.len()
    }

    /// Checks if the segment has an empty payload
    pub fn is_empty(&self) -> bool {
        self.payload.is_empty()
    }

    /// Provides a slice view of the payload
    pub fn as_slice(&self) -> &[u8] {
        &self.payload
    }

    /// Returns a reference to the segment's payload as Bytes
    pub fn payload(&self) -> &Bytes {
        &self.payload
    }

    /// Converts the segment's payload to a Vec<u8> for compatibility
    pub fn to_vec(&self) -> Vec<u8> {
        self.payload.to_vec()
    }

    /// Returns an iterator over the bytes in the segment
    pub fn iter(&self) -> impl Iterator<Item = u8> + '_ {
        self.payload.iter().copied()
    }
}

impl Display for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self.payload).style(Style::Dim))
    }
}

/// Builder pattern for creating segments with efficient memory management
pub struct SegmentBuilder {
    /// The port address to use for the segment
    header: PortAddress,
    /// A mutable buffer for constructing the payload
    payload: BytesMut,
}

impl SegmentBuilder {
    /// Adds a byte slice to the payload
    pub fn put_slice(&mut self, slice: &[u8]) -> &mut Self {
        self.payload.put_slice(slice);
        self
    }

    /// Adds bytes from a Bytes object to the payload
    pub fn put_bytes(&mut self, bytes: &Bytes) -> &mut Self {
        self.payload.put_slice(bytes.as_ref());
        self
    }

    /// Reserves capacity in the buffer for additional bytes
    pub fn reserve(&mut self, additional: usize) -> &mut Self {
        self.payload.reserve(additional);
        self
    }

    /// Finalizes the builder and returns a Segment
    pub fn build(self) -> Segment {
        Segment {
            header: self.header,
            payload: self.payload.freeze(),
        }
    }
}
