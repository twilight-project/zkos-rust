//! Encoding utils for ZKOS Transaction
//! All methods err using VMError::InvalidFormat for convenience.

// pub use readerwriter::{
//     Decodable, Encodable, ExactSizeEncodable, ReadError, Reader, WriteError, Writer,
// };
// use crate::util::Address;

// use quisquislib::elgamal::ElGamalCommitment;

// /// Extension to the Reader interface for Encryption and Address.
// pub trait ReaderExt: Reader {
    
//     /// Reads Encryption bytes as pair of Ristretto255 points (64 bytes).
//     fn read_encryption(&mut self) -> Result<ElGamalCommitment, ReadError> {
//         let buf = self.read_u8x64()?;
//         ElGamalCommitment::from_bytes(buf).ok_or(ReadError::InvalidFormat)

//     }

//     /// Reads address bytes.
//     fn read_address(&mut self) -> Result<Address, ReadError> {
//         let buf = self.read_u8x64()?;
//         let res = Address::from_bytes(&buf);
//         if res.is_ok(){
//             Ok(res.unwrap())
//         }else{
//             Err(ReadError::InvalidFormat)
//         }
//     }
// }

// /// Extension to the Writer interface for Encryption and Address.
// pub trait WriterExt: Writer {
    
//      /// Writes a encryption composed of Two compressed Ristretto255 points.
//       fn write_encryption(
//         &mut self,
//         label: &'static [u8],
//         x: &ElGamalCommitment,
//     ) -> Result<(), WriteError> {
//         self.write(label, &x.to_bytes()[..])
//     }
//     /// Writes an Address of account
//     fn write_address(
//         &mut self,
//         label: &'static [u8],
//         x: &Address,
//     ) -> Result<(), WriteError> {
//         self.write(label, &x.as_bytes()[..])
//     }
    
// }

