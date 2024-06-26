//! Encoding utils for ZkVM
//! All methods err using VMError::InvalidFormat for convenience.

use crate::errors::VMError;

use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;
use quisquislib::elgamal::ElGamalCommitment;
pub use readerwriter::{
    Decodable, Encodable, ExactSizeEncodable, ReadError, Reader, WriteError, Writer,
};

/// Extension to the Reader interface for Ristretto points and scalars.
pub trait ReaderExt: Reader {
    /// Reads a u32-LE number used for encoding length prefixes in ZkVM.
    fn read_size(&mut self) -> Result<usize, ReadError> {
        Ok(self.read_u32()? as usize)
    }

    /// Reads a compressed Ristretto255 point (32 bytes).
    fn read_point(&mut self) -> Result<CompressedRistretto, ReadError> {
        let buf = self.read_u8x32()?;
        Ok(CompressedRistretto(buf))
    }

    /// Reads a Ristretto255 scalar (32 bytes).
    fn read_scalar(&mut self) -> Result<Scalar, ReadError> {
        let buf = self.read_u8x32()?;
        Scalar::from_canonical_bytes(buf).ok_or(ReadError::InvalidFormat)
    }

    /// Reads Encryption bytes as pair of Ristretto255 points (64 bytes).
    fn read_encryption(&mut self) -> Result<ElGamalCommitment, ReadError> {
        let buf = self.read_u8x64()?;
        let res = ElGamalCommitment::from_bytes(&buf);
        if res.is_ok() {
            Ok(res.unwrap())
        } else {
            Err(ReadError::InvalidFormat)
        }
    }

    /// Reads address bytes.
    fn read_address(&mut self) -> Result<String, ReadError> {
        let buf = self.read_u8x64()?;
        let res = String::from_utf8(buf.to_vec());
        if res.is_ok() {
            Ok(res.unwrap())
        } else {
            Err(ReadError::InvalidFormat)
        }
    }
}

/// Extension to the Writer interface for Ristretto points and scalars.
pub trait WriterExt: Writer {
    /// Writes a u32-LE number used for encoding length prefixes in ZkVM.
    fn write_size(&mut self, label: &'static [u8], x: usize) -> Result<(), WriteError> {
        self.write_u32(label, x as u32)
    }

    /// Writes a compressed Ristretto255 point.
    fn write_point(
        &mut self,
        label: &'static [u8],
        x: &CompressedRistretto,
    ) -> Result<(), WriteError> {
        self.write(label, &x.as_bytes()[..])
    }

    /// Writes a Ristretto255 scalar.
    fn write_scalar(&mut self, label: &'static [u8], x: &Scalar) -> Result<(), WriteError> {
        self.write(label, &x.as_bytes()[..])
    }

    /// Writes a encryption composed of Two compressed Ristretto255 points.
    fn write_encryption(
        &mut self,
        label: &'static [u8],
        x: &ElGamalCommitment,
    ) -> Result<(), WriteError> {
        // let b = x.to_bytes();
        self.write(label, &x.to_bytes()[..])
    }

    /// Writes an Address of account
    fn write_address(&mut self, label: &'static [u8], x: &String) -> Result<(), WriteError> {
        self.write(label, &x.as_bytes()[..])
    }
}

impl<T> ReaderExt for T where T: Reader {}
impl<T> WriterExt for T where T: Writer {}

impl From<ReadError> for VMError {
    fn from(_: ReadError) -> VMError {
        VMError::InvalidFormat
    }
}

impl From<WriteError> for VMError {
    fn from(_: WriteError) -> VMError {
        VMError::InvalidFormat
    }
}
