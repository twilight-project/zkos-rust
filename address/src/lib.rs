//#![deny(missing_docs)]
#![allow(non_snake_case)]

//! ZkOS Transaction Address implementation.
pub extern crate quisquislib;

use bs58;
use curve25519_dalek::ristretto::CompressedRistretto;
use quisquislib::{keys::PublicKey, ristretto::RistrettoPublicKey};
use ripemd::{Digest, Ripemd160};
use serde::{Deserialize, Serialize};
use sha3::Keccak256;
use std::fmt;

/// The list of the existing Twilight networks.
/// Network type: Mainnet, Testnet.
/// Network implements [`Default`] and returns [`Network::Mainnet`].
///
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Network {
    /// Mainnet is the "production" network and blockchain.
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
    /// The byte values should be taken from the blockchain config file. The same values should be used here. Sample values are used here
    pub fn from_u8(byte: u8) -> Result<Network, &'static str> {
        use Network::*;
        match byte {
            12 | 24 => Ok(Mainnet),
            44 | 66 => Ok(Testnet),
            _ => Err("Error::InvalidNteworkByte"),
        }
    }
}

impl Default for Network {
    fn default() -> Network {
        Network::Mainnet
    }
}
/// Address type: standard, contract.
///
/// AddressType implements [`Default`] and returns [`AddressType::Coin`].
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum AddressType {
    /// Standard twilight coin address.
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

impl Default for AddressType {
    fn default() -> AddressType {
        AddressType::Standard
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
/// Address implements [`Default`] and returns [`Address::Standard`].
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum Address {
    /// Standard twilight coin address.
    Standard(Standard),
    /// Script addresses.
    Script(Script),
}

impl Address {
    /// Recover the address type given an address bytes and the network.
    /// /// Create a standard address which is valid on the given network.
    pub fn standard_address(network: Network, public_key: RistrettoPublicKey) -> Address {
        Self::Standard(Standard{
            network,
            addr_type: AddressType::default(),
            public_key,
        })
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
    pub fn as_script_address(&self) -> Script {
        match *self {
            Address::Script(s) => s,
            _ => panic!("Not a script address"),
        }
    }
    pub fn as_c_address(&self) -> Standard {
        match *self {
            Address::Standard(c) => c,
            _ => panic!("Not a coin address"),
        }
    }
    pub fn from_hex(hex: &str, add_type:AddressType) -> Result<Address, &'static str> {
        let bytes = hex::decode(hex).map_err(|_| "Error::InvalidHex")?;
        match add_type {
            AddressType::Standard => Ok(Address::Standard(Standard::from_bytes(&bytes)?)),
            AddressType::Script => Err("Error::ScriptAddress can not be re-created from hex"),
        }
    }
    pub fn get_standard_address(&self) -> Result<Standard, &'static str> {
        match *self {
            Address::Standard(c) => Ok(c),
            _ => Err("Error::Not a coin address"),
        }
    }
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
    /// Parse an address from a vector of bytes, fail if the magic byte is incorrect, if public
    /// keys are not valid points, and if checksums missmatch.
    pub fn from_bytes(bytes: &[u8]) -> Result<Standard, &'static str> {
        use sha3::Digest;
        let network = Network::from_u8(bytes[0])?;
        let addr_type = AddressType::from_slice(&bytes, network)?;
        let gr = slice_to_pkpoint(&bytes[1..33])?;
        let grsk = slice_to_pkpoint(&bytes[33..65])?;
        let public_key = RistrettoPublicKey::new_from_pk(gr, grsk);
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

    /// Serialize the address as a vector of bytes.
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

    /// Serialize the address bytes as a hexadecimal string.
    pub fn as_hex(&self) -> String {
        hex::encode(self.as_bytes())
    }

    /// Serialize the address bytes as a BTC-Base58 string.
    pub fn as_base58(&self) -> String {
        bs58::encode(self.as_bytes()).into_string()
    }

    /// Convert Hex address string to Address
    pub fn from_hex(s: &str) -> Self {
        Self::from_bytes(&hex::decode(s).unwrap().as_slice()).unwrap()
    }

    /// Convert Base58 address string to Address
    pub fn from_base58(s: &str) -> Self {
        let decoded = bs58::decode(s).into_vec().unwrap();
        Self::from_bytes(&decoded).unwrap()
    }
}

impl Default for Standard {
    fn default() -> Standard {
        Standard {
            network: Network::Mainnet,
            addr_type: AddressType::default(),
            public_key: RistrettoPublicKey::new_from_pk(CompressedRistretto::default(),CompressedRistretto::default())
        }
    }
}

// A complete twilight typed address valid for a specific network.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Copy)]
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
    /// Byte Format : [magic byte, script tree root hash]  
    pub fn as_bytes(&self) -> [u8; 21] {
        let mut bytes: Vec<u8> = Vec::with_capacity(21);
        //add Network magic Byte
        bytes.push(self.network.as_u8(&self.addr_type));

        // create a RIPEMD-160 hasher instance
        let mut hasher = Ripemd160::new();
        // process input message i.e., RIPEMD-160 of tree root hash
        hasher.update(&self.root);
        // acquire hash digest in the form of GenericArray, which in this case is equivalent to [u8; 20]
        let result = hasher.finalize();
        //add RIP-160 hash bytes to byte array
        bytes.extend_from_slice(&result);
        bytes.try_into().unwrap()
    }

    /// Serialize the address as a vector of bytes using Ripemd160 hash for scripts.
    /// Byte Format : [magic byte, script tree root hash]  
    // pub fn from_bytes(bytes : &[u8]) -> ScriptAddress {

    /// Serialize the address bytes as a hexadecimal string.
    pub fn as_hex(&self) -> String {
        hex::encode(self.as_bytes())
    }

    /// Serialize the address bytes as a BTC-Base58 string.
    pub fn as_base58(&self) -> String {
        bs58::encode(self.as_bytes()).into_string()
    }
}
impl Default for Script {
    fn default() -> Script {
        Script { 
            network: Network::Testnet, 
            addr_type: AddressType::Script, 
            root: [b'0';32] 
        }

    }
}
/// Deserialize a public key from a slice. The input slice is 64 bytes
/// Utility Function
fn slice_to_pkpoint(data: &[u8]) -> Result<CompressedRistretto, &'static str> {
    if data.len() != 32 {
        return Err("Invalid Key Length");
    }
    let gr = CompressedRistretto::from_slice(&data);
    match gr.decompress() {
        Some(_) => (),
        None => {
            return Err("InvalidPoint");
        }
    };
    Ok(gr)
}
// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------
#[cfg(test)]
mod test {
    use super::*;
    use sha3::{Digest, Keccak256};
    #[test]
    fn hex_encoding_decoding_test() {}

    #[test]
    fn script_address_encoding_test() {
        let random_str = "I am a fool. Hardy Hardy fool fool";
        //Hasho fthe string
        let mut hasher = Keccak256::new();
        hasher.update(&random_str);
        let result = hasher.finalize();

        let root_hash = result.into();
        let sc_add = Address::script_address(Network::Mainnet, root_hash);
        let by = sc_add.as_bytes();
        println!("length: {:?}", by.len());
        println!("bytes: {:?}", by);
    }
}
