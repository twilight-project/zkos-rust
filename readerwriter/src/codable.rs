//! Traits for encoding and decoding structures using binary readers and writers.
//!
//! This module provides the [`Encodable`], [`Decodable`], and [`Codable`] traits for custom serialization logic.
//!
//! # Example
//! ```
//! use readerwriter::{Encodable, Writer};
//! struct MyType(u32);
//! impl Encodable for MyType {
//!     fn encode(&self, w: &mut impl Writer) -> Result<(), readerwriter::WriteError> {
//!         w.write_u32(b"mytype", self.0)
//!     }
//! }
//! ```

use crate::{ReadError, Reader, WriteError, Writer};

/// A trait for encoding structures using the [`Writer`] trait.
///
/// Implement this trait for your type to support custom binary serialization.
pub trait Encodable {
    /// Encodes receiver into bytes appending them to a provided buffer.
    fn encode(&self, w: &mut impl Writer) -> Result<(), WriteError>;
    /// If possible, returns an encoded size as a hint for allocating appropriate buffer.
    /// Default implementation returns None.
    fn encoded_size_hint(&self) -> Option<usize> {
        None
    }

    /// Encodes the receiver into a newly allocated vector of bytes.
    fn encode_to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.encoded_size_hint().unwrap_or(0));
        self.encode(&mut buf)
            .expect("Writing to a Vec never fails.");
        buf
    }
}

/// A trait for encoding structures with a known, exact encoded size.
pub trait ExactSizeEncodable: Encodable {
    /// Exact encoded size in bytes of the object.
    fn encoded_size(&self) -> usize;

    fn encoded_size_hint(&self) -> Option<usize> {
        Some(self.encoded_size())
    }
}

/// A trait for decoding bytes into structure using the [`Reader`] trait.
///
/// Implement this trait for your type to support custom binary deserialization.
pub trait Decodable: Sized {
    /// Decodes bytes into self by reading bytes from reader.
    fn decode(buf: &mut impl Reader) -> Result<Self, ReadError>;
}

/// Trait for structures which implement both [`Decodable`] and [`Encodable`] traits.
pub trait Codable: Encodable + Decodable {}

impl<T: Decodable + Encodable> Codable for T {}
