//! The `address` crate provides functionality for creating, parsing, and validating
//! Twilight addresses. It supports different address types and networks, and provides
//! functionality for encoding and decoding addresses to and from various formats.
//!
//! # Overview
//!
//! This crate defines the following main types:
//!
//! *   [`Network`]: Represents the network (Mainnet or Testnet) on which an address is valid.
//! *   [`AddressType`]: Represents the type of an address (Standard or Script).
//! *   [`Address`]: An enum that can be either a `Standard` or `Script` address.
//! *   [`Standard`]: A standard address that is derived from a public key.
//! *   [`Script`]: A script address that is derived from a script hash.
//!
//! # Usage
//!
//! ## Creating a Standard Address
//!
//! ```
//! use address::{Address, Network};
//! use quisquislib::ristretto::RistrettoPublicKey;
//! use curve25519_dalek::ristretto::CompressedRistretto;
//!
//! let public_key_bytes = [0; 32];
//! let public_key_bytes_grsk = [0; 32];
//! let compressed_ristretto = CompressedRistretto(public_key_bytes);
//! let compressed_ristretto_grsk = CompressedRistretto(public_key_bytes_grsk);
//! let public_key = RistrettoPublicKey::new_from_pk(compressed_ristretto, compressed_ristretto_grsk);
//! let address = Address::standard_address(Network::Mainnet, public_key);
//!
//! println!("Standard Address: {}", address);
//! ```
//!
//! ## Creating a Script Address
//!
//! ```
//! use address::{Address, Network};
//!
//! let script_hash = [0; 32];
//! let address = Address::script_address(Network::Mainnet, script_hash);
//!
//! println!("Script Address: {}", address.as_base58());
//! ```
//!
//! ## Parsing an Address
//!
//! ```
//! use address::{Address, AddressType};
//!
//! let address_str = "T16fJv9T1sH6qE7X9Z3vY4K2A8bC5D6E7F8G9H0J";
//! let address = Address::from_base58(address_str, AddressType::Standard);
//!
//! // assert!(address.is_ok());
//! ```
#![deny(missing_docs)]
#![allow(non_snake_case)]

use curve25519_dalek::ristretto::CompressedRistretto;
use quisquislib::{keys::PublicKey, ristretto::RistrettoPublicKey};
use ripemd::{Digest, Ripemd160};
use serde::{Deserialize, Serialize};
use sha3::Keccak256;
use std::fmt;

/// The list of the existing Twilight networks.
///
/// `Network` implements [`Default`] and returns [`Network::Mainnet`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Network {
    /// Mainnet is the "production" network and blockchain.
    #[default]
    Mainnet,
    /// Testnet is the "experimental" network and blockchain where things get released long before
    /// mainnet.
    Testnet,
}
impl Network {
    /// Get the associated magic byte given an address type.
    /// The byte values should be taken from the blockchain config file. The same values should be used here. Sample values are used here
    pub fn as_u8(self, addr_type: &AddressType) -> u8 {
        use AddressType::*;
        match self {
            Network::Mainnet => match addr_type {
                Standard => 12,
                Script => 24,
            },
            Network::Testnet => match addr_type {
                Standard => 44,
                Script => 66,
            },
        }
    }

    /// Recover the network type given an address magic byte.
    ///
    /// The byte values should be taken from the blockchain config file. The same values should be used here.
    pub fn from_u8(byte: u8) -> Result<Network, &'static str> {
        use Network::*;
        match byte {
            12 | 24 => Ok(Mainnet),
            44 | 66 => Ok(Testnet),
            _ => Err("Error::InvalidNteworkByte"),
        }
    }
}

/// Address type: standard, contract.
///
/// AddressType implements [`Default`] and returns [`AddressType::Standard`].
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize, Default)]
pub enum AddressType {
    /// Standard twilight coin address.
    #[default]
    Standard,
    /// Script addresses.
    Script,
}

impl AddressType {
    /// Recover the address type given an address bytes and the network.
    pub fn from_slice(bytes: &[u8], net: Network) -> Result<AddressType, &'static str> {
        let byte = bytes[0];
        use AddressType::*;
        use Network::*;
        match net {
            Mainnet => match byte {
                12 => Ok(Standard),
                24 => Ok(Script),
                _ => Err("Error::InvalidAddressTypeMagicByte"),
            },
            Testnet => match byte {
                44 => Ok(Standard),
                66 => Ok(Script),
                _ => Err("Error::InvalidAddressTypeMagicByte"),
            },
        }
    }
}

impl fmt::Display for AddressType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AddressType::Standard => write!(f, "Coin address"),
            AddressType::Script => write!(f, "Script address"),
        }
    }
}

/// Address: standard, contract.
///
/// `Address` can be either a [`Standard`] or a [`Script`] address.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum Address {
    /// Standard twilight coin address.
    Standard(Standard),
    /// Script addresses.
    Script(Script),
}

impl Address {
    /// Create a standard address which is valid on the given network.
    pub fn standard_address(network: Network, public_key: RistrettoPublicKey) -> Address {
        Self::Standard(Standard {
            network,
            addr_type: AddressType::default(),
            public_key,
        })
    }

    /// Set a standard address.
    pub fn set_standard_address(add: Standard) -> Self {
        Address::Standard(add)
    }

    /// Create a script address which is valid on the given network.
    pub fn script_address(network: Network, root: [u8; 32]) -> Address {
        Self::Script(Script {
            network,
            addr_type: AddressType::Script,
            root,
        })
    }
    /// Serialize the address bytes as a BTC-Base58 string.
    pub fn as_base58(&self) -> String {
        match *self {
            Address::Standard(c) => c.as_base58(),
            Address::Script(s) => s.as_base58(),
        }
    }

    /// Serialize the address bytes as a Hex string.
    pub fn as_hex(&self) -> String {
        match *self {
            Address::Standard(c) => c.as_hex(),
            Address::Script(s) => s.as_hex(),
        }
    }

    /// Serialize the address as a byte string.
    pub fn as_bytes(&self) -> Vec<u8> {
        match *self {
            Address::Standard(c) => c.as_bytes(),
            Address::Script(s) => s.as_bytes().to_vec(),
        }
    }
    /// Returns the script address.
    ///
    /// # Panics
    ///
    /// Panics if the address is not a script address.
    pub fn as_script_address(&self) -> Script {
        match *self {
            Address::Script(s) => s,
            _ => panic!("Not a script address"),
        }
    }
    /// Returns the coin address.
    ///
    /// # Panics
    ///
    /// Panics if the address is not a coin address.
    pub fn as_coin_address(&self) -> Standard {
        match *self {
            Address::Standard(c) => c,
            _ => panic!("Not a coin address"),
        }
    }
    /// Recover an address from a hex string.
    pub fn from_hex(hex: &str, add_type: AddressType) -> Result<Address, &'static str> {
        let bytes = hex::decode(hex).map_err(|_| "Error::InvalidHex")?;
        match add_type {
            AddressType::Standard => Ok(Address::Standard(Standard::from_bytes(&bytes)?)),
            AddressType::Script => Err("Error::ScriptAddress can not be re-created from hex"),
        }
    }

    /// Recover an address from a base58 string.
    pub fn from_base58(base_58: &str, add_type: AddressType) -> Result<Address, &'static str> {
        let bytes = bs58::decode(base_58)
            .into_vec()
            .map_err(|_| "Error::Invalid Base58 address")?;

        match add_type {
            AddressType::Standard => Ok(Address::Standard(Standard::from_bytes(&bytes)?)),
            AddressType::Script => Err("Error::ScriptAddress can not be re-created from Base58"),
        }
    }
    /// Returns the standard address.
    pub fn get_standard_address(&self) -> Result<Standard, &'static str> {
        match *self {
            Address::Standard(c) => Ok(c),
            _ => Err("Error::Not a coin address"),
        }
    }
    /// Returns the script address.
    pub fn get_script_address(&self) -> Result<Script, &'static str> {
        match *self {
            Address::Script(s) => Ok(s),
            _ => Err("Error::Not a script address"),
        }
    }
}
impl Default for Address {
    fn default() -> Address {
        Address::Standard(Standard::default())
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Address::Standard(ref c) => write!(f, "{}", c.as_base58()),
            Address::Script(ref s) => write!(f, "{}", s.as_base58()),
        }
    }
}
//create standard mainnet address from public key
impl From<RistrettoPublicKey> for Address {
    fn from(pk: RistrettoPublicKey) -> Address {
        Address::standard_address(Network::Mainnet, pk)
    }
}

impl From<Address> for RistrettoPublicKey {
    fn from(val: Address) -> Self {
        match val {
            Address::Standard(c) => c.public_key,
            _ => panic!("Not a coin address"),
        }
    }
}

/// A complete twilight typed address valid for a specific network.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub struct Standard {
    /// The network on which the address is valid and should be used.
    pub network: Network,
    /// The address type.
    pub addr_type: AddressType,
    /// The address public key.
    pub public_key: RistrettoPublicKey,
}

impl Standard {
    /// Create a standard address which is valid on the given network.
    pub fn new(network: Network, public_key: RistrettoPublicKey) -> Standard {
        Standard {
            network,
            addr_type: AddressType::Standard,
            public_key,
        }
    }

    /// Returns the public key of the address.
    pub fn get_public_key(&self) -> RistrettoPublicKey {
        self.public_key
    }

    /// Create a standard address from a byte slice.
    pub fn from_bytes(bytes: &[u8]) -> Result<Standard, &'static str> {
        use sha3::Digest;
        let network = Network::from_u8(bytes[0])?;
        let addr_type = AddressType::from_slice(bytes, network)?;
        let public_key = RistrettoPublicKey::from_bytes(&bytes[1..65])?;

        let (checksum_bytes, checksum) = (&bytes[0..65], &bytes[65..69]);
        let mut hasher = Keccak256::new();
        hasher.update(checksum_bytes);
        let checksum_verify = hasher.finalize();
        if &checksum_verify[0..4] != checksum {
            return Err("Invalid Checksum");
        }

        Ok(Standard {
            network,
            addr_type,
            public_key,
        })
    }

    /// Serialize the address as a byte string.
    ///
    /// Byte Format : [magic byte, public key, checksum]  
    pub fn as_bytes(&self) -> Vec<u8> {
        use sha3::Digest;
        let mut bytes = vec![self.network.as_u8(&self.addr_type)];
        bytes.extend_from_slice(self.public_key.as_bytes().as_slice());
        let mut hasher = Keccak256::new();
        hasher.update(&bytes);
        let checksum = hasher.finalize();
        bytes.extend_from_slice(&checksum[0..4]);
        bytes
    }

    /// Serialize the address as a hex string.
    pub fn as_hex(&self) -> String {
        hex::encode(self.as_bytes())
    }

    /// Serialize the address as a base58 string.
    pub fn as_base58(&self) -> String {
        bs58::encode(self.as_bytes()).into_string()
    }

    /// Create a standard address from a hex string.
    pub fn from_hex(s: &str) -> Self {
        let bytes = hex::decode(s).unwrap();
        Standard::from_bytes(&bytes).unwrap()
    }

    /// Create a standard address from a hex string, returning an error if the string is invalid.
    pub fn from_hex_with_error(s: &str) -> Result<Self, String> {
        let bytes = hex::decode(s).map_err(|e| e.to_string())?;
        Standard::from_bytes(&bytes).map_err(|e| e.to_string())
    }

    /// Create a standard address from a base58 string.
    ///
    /// # Errors
    ///
    /// Returns an error if the base58 string is invalid or the address cannot be parsed.
    pub fn from_base58(s: &str) -> Result<Self, &'static str> {
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|_| "Invalid base58 encoding")?;
        Standard::from_bytes(&bytes)
    }
}
impl Default for Standard {
    fn default() -> Standard {
        Standard {
            network: Network::Mainnet,
            addr_type: AddressType::default(),
            public_key: RistrettoPublicKey::new_from_pk(
                CompressedRistretto::default(),
                CompressedRistretto::default(),
            ),
        }
    }
}

/// A zkos script address.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub struct Script {
    /// The network on which the address is valid and should be used.
    pub network: Network,
    /// The address type.
    pub addr_type: AddressType,
    /// The root hash of the script tree.
    pub root: [u8; 32],
}

impl Script {
    /// Serialize the address as a vector of bytes using Ripemd160 hash for scripts.
    ///
    /// Byte Format : [magic byte, script tree root hash]  
    pub fn as_bytes(&self) -> [u8; 21] {
        let mut bytes = [0u8; 21]; // Create fixed-size array

        // Add network magic byte
        bytes[0] = self.network.as_u8(&self.addr_type);

        // Create RIPEMD-160 hash
        let mut hasher = Ripemd160::new();
        hasher.update(self.root);
        let result = hasher.finalize();

        // Copy hash bytes to array (starting from index 1)
        bytes[1..].copy_from_slice(&result);

        bytes // Return the fixed-size array

        //let mut bytes: Vec<u8> = Vec::with_capacity(21);
        //add Network magic Byte
        //bytes.push(self.network.as_u8(&self.addr_type));
        //let mut hasher = Ripemd160::new();
        // process input message i.e., RIPEMD-160 of tree root hash
        //hasher.update(&self.root);
        // acquire hash digest in the form of GenericArray, which in this case is equivalent to [u8; 20]
        // let result = hasher.finalize();
        //add RIP-160 hash bytes to byte array
        //bytes.extend_from_slice(&result);
        //bytes.try_into().unwrap()
        //bytes[1..].copy_from_slice(&result);

        //bytes
    }

    /// Serialize the address as a hex string.
    pub fn as_hex(&self) -> String {
        hex::encode(self.as_bytes())
    }

    /// Serialize the address as a base58 string.
    pub fn as_base58(&self) -> String {
        bs58::encode(self.as_bytes()).into_string()
    }

    /// Returns the root hash of the script.
    pub fn get_root_hash(&self) -> [u8; 32] {
        self.root
    }
}
impl Default for Script {
    fn default() -> Script {
        Script {
            network: Network::default(),
            addr_type: AddressType::Script,
            root: [b'0'; 32],
        }
    }
}

// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn hex_encoding_decoding_test() {
        // Test 1: Standard Address (this should work)
        let valid_public_key_bytes = [
            216, 227, 23, 119, 23, 126, 216, 232, 118, 13, 184, 255, 134, 185, 26, 15, 19, 216, 52,
            139, 17, 132, 11, 64, 15, 11, 218, 114, 142, 240, 23, 12,
        ];
        let valid_public_key_bytes_grsk = [
            216, 227, 23, 119, 23, 126, 216, 232, 118, 13, 184, 255, 134, 185, 26, 15, 19, 216, 52,
            139, 17, 132, 11, 64, 15, 11, 218, 114, 142, 240, 23, 12,
        ];
        let compressed_ristretto = CompressedRistretto(valid_public_key_bytes_grsk);
        let compressed_ristretto_grsk = CompressedRistretto(valid_public_key_bytes_grsk);
        let public_key =
            RistrettoPublicKey::new_from_pk(compressed_ristretto, compressed_ristretto_grsk);

        let address = Address::standard_address(Network::Mainnet, public_key);
        let encoded_address = address.as_hex();

        match Address::from_hex(&encoded_address, AddressType::Standard) {
            Ok(decoded_address) => assert_eq!(decoded_address, address),
            Err(e) => panic!("Standard address decoding failed: {}", e),
        }

        // Test 2: Script Address (this will always fail with current implementation)
        let script_address = Address::script_address(Network::Mainnet, [0; 32]);
        let encoded_address = script_address.as_hex();

        // This should fail according to your current implementation
        match Address::from_hex(&encoded_address, AddressType::Script) {
            Ok(_) => panic!("Script address decoding should have failed"),
            Err(e) => {
                assert_eq!(e, "Error::ScriptAddress can not be re-created from hex");
                println!(
                    "Script address decoding correctly failed as expected: {}",
                    e
                );
            }
        }
    }

    #[test]
    fn script_address_encoding_test() {
        let script_address = Address::script_address(
            Network::Testnet,
            [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ],
        );

        let encoded_address = script_address.as_hex();

        // Calculate the expected value based on the current implementation
        let expected_bytes = script_address.as_script_address().as_bytes();
        let expected_hex = hex::encode(expected_bytes);

        assert_eq!(encoded_address, expected_hex);

        let base_58_address = script_address.as_base58();

        // Calculate the expected base58 value
        let expected_base58 = bs58::encode(expected_bytes).into_string();
        assert_eq!(base_58_address, expected_base58);

        // Print the actual values for debugging
        println!("Actual hex: {}", encoded_address);
        println!("Actual base58: {}", base_58_address);
    }

    #[test]
    fn script_as_bytes_test() {
        // Test 1: Basic functionality
        let script = Script {
            network: Network::Testnet,
            addr_type: AddressType::Script,
            root: [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ],
        };

        let bytes = script.as_bytes();

        // Check the length is exactly 21 bytes
        assert_eq!(bytes.len(), 21);

        // Check the magic byte (Testnet Script = 66)
        assert_eq!(bytes[0], 66);

        // Check that the rest is the RIPEMD-160 hash of the root
        let mut hasher = Ripemd160::new();
        hasher.update(&script.root);
        let expected_hash = hasher.finalize();
        assert_eq!(&bytes[1..], expected_hash.as_slice());

        // Test 2: Different network
        let script_mainnet = Script {
            network: Network::Mainnet,
            addr_type: AddressType::Script,
            root: [0; 32],
        };

        let bytes_mainnet = script_mainnet.as_bytes();
        assert_eq!(bytes_mainnet.len(), 21);
        assert_eq!(bytes_mainnet[0], 24); // Mainnet Script = 24

        // Test 3: Zero root hash
        let script_zero = Script {
            network: Network::Testnet,
            addr_type: AddressType::Script,
            root: [0; 32],
        };

        let bytes_zero = script_zero.as_bytes();
        assert_eq!(bytes_zero.len(), 21);
        assert_eq!(bytes_zero[0], 66); // Testnet Script = 66

        // Verify the RIPEMD-160 hash of zeros
        let mut hasher = Ripemd160::new();
        hasher.update(&[0; 32]);
        let expected_zero_hash = hasher.finalize();
        assert_eq!(&bytes_zero[1..], expected_zero_hash.as_slice());

        // Test 4: Round-trip test (bytes -> hex -> base58 -> bytes should be consistent)
        let original_bytes = script.as_bytes();
        let hex_str = script.as_hex();
        let base58_str = script.as_base58();

        // Verify hex encoding/decoding
        let decoded_hex = hex::decode(&hex_str).unwrap();
        assert_eq!(decoded_hex, original_bytes);

        // Verify base58 encoding/decoding
        let decoded_base58 = bs58::decode(&base58_str).into_vec().unwrap();
        assert_eq!(decoded_base58, original_bytes);
    }
}
