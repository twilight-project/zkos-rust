#![allow(non_snake_case)]
//#![deny(missing_docs)]

use crate::util::Address;
use quisquislib::{accounts::Account, elgamal::ElGamalCommitment};
use serde::{Deserialize, Serialize};

/// Transaction ID is a unique 32-byte identifier of a transaction effects represented by `TxLog`.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxId(pub [u8; 32]);

/// Transaction type: Transfer. Transition, Create, Vault
///
/// TransactionType implements [`Default`] and returns [`TransactionType::Transfer`].
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    Transition,
    Create,
    Vault,
}

impl TransactionType {
    pub fn from_u8(byte: u8) -> Result<TransactionType, &'static str> {
        use TransactionType::*;
        match byte {
            0 => Ok(Transfer),
            1 => Ok(Transition),
            2 => Ok(Create),
            3 => Ok(Vault),
            _ => Err("Error::InvalidTransactionType"),
        }
    }
}
impl Default for TransactionType {
    fn default() -> TransactionType {
        TransactionType::Transfer
    }
}

/// Identification of transaction in a block.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxPointer {
    /// block id
    block_height: u64,
    /// output index
    tx_index: u16,
}

/// Identification of unspend transaction output.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Utxo {
    /// transaction id
    txid: TxId,
    /// output index
    output_index: u8,
}

impl Utxo {
    pub const fn new(txid: TxId, output_index: u8) -> Self {
        Self { txid, output_index }
    }

    pub const fn tx_id(&self) -> &TxId {
        &self.txid
    }

    pub const fn output_index(&self) -> u8 {
        self.output_index
    }

    pub fn replace_tx_id(&mut self, tx_id: TxId) {
        self.txid = tx_id;
    }
}

/// Input type: Dark, Record,
///
/// InputType implements [`Default`] and returns [`InputType::Dark`].
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum InputType {
    Dark,
    Record,
}

impl InputType {
    pub fn from_u8(byte: u8) -> Result<InputType, &'static str> {
        use InputType::*;
        match byte {
            0 => Ok(Dark),
            1 => Ok(Record),
            _ => Err("Error::InvalidInputType"),
        }
    }
}
impl Default for InputType {
    fn default() -> InputType {
        InputType::Dark
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputData {
    Coin { utxo: Utxo, account: Account },
}

impl InputData {
    pub const fn coin_dark(utxo: Utxo, account: Account) -> Self {
        Self::Coin { utxo, account }
    }

    pub const fn utxo_id(&self) -> Option<&Utxo> {
        match self {
            Self::Coin { utxo, .. } => Some(utxo),
            _ => None,
        }
    }

    /*pub const fn tx_pointer(&self) -> Option<&TxPointer> {
        match self {
            InputData::Coin { tx_pointer, .. } => Some(tx_pointer),
            _ => None,
        }
    }*/

    pub const fn account(&self) -> Option<&Account> {
        match self {
            InputData::Coin { account, .. } => Some(account),
            _ => None,
        }
    }
}

/// A complete twilight typed Input valid for a specific network.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Input {
    /// Defines the input type.
    pub in_type: InputType,
    /// The input data corresponding to the input type.
    pub input: InputData,
}

impl Input {
    /// Create a input of Dark Coin which is valid on the given network.
    pub fn coin(data: InputData) -> Input {
        Input {
            in_type: InputType::default(),
            input: data,
        }
    }
}

/// Output type: Dark, Record,
///
/// OutputType implements [`Default`] and returns [`OutputType::Dark`].
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum OutputType {
    Dark,
    Record,
}

impl OutputType {
    pub fn from_u8(byte: u8) -> Result<OutputType, &'static str> {
        use OutputType::*;
        match byte {
            0 => Ok(Dark),
            1 => Ok(Record),
            _ => Err("Error::InvalidInputType"),
        }
    }
}
impl Default for OutputType {
    fn default() -> OutputType {
        OutputType::Dark
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OutputData {
    Coin {
        address: Address,
        comm: ElGamalCommitment,
    },
}

// impl Default for Output {
//     fn default() -> Self {
//         Self::Coin {
//             account: Default::default(),
//         }
//     }
// }
impl OutputData {
    pub const fn coin(address: Address, comm: ElGamalCommitment) -> Self {
        Self::Coin { address, comm }
    }

    pub const fn adress(&self) -> Option<Address> {
        match self {
            Self::Coin { address, .. } => Some(*address),
            _ => None,
        }
    }

    pub const fn commitment(&self) -> Option<ElGamalCommitment> {
        match self {
            Self::Coin { comm, .. } => Some(*comm),
            _ => None,
        }
    }
}

/// A complete twilight typed Output valid for a specific network.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Output {
    /// Defines the input type.
    pub out_type: OutputType,
    /// The input data corresponding to the input type.
    pub output: OutputData,
}

impl Output {
    /// Create a input of Dark Coin which is valid on the given network.
    pub fn coin(data: OutputData) -> Output {
        Output {
            out_type: OutputType::default(),
            output: data,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Witness {
    data: Vec<u8>,
}

impl Witness {
    pub const fn as_vec(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn as_vec_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }
}

impl From<Vec<u8>> for Witness {
    fn from(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl From<&[u8]> for Witness {
    fn from(data: &[u8]) -> Self {
        data.to_vec().into()
    }
}

impl AsRef<[u8]> for Witness {
    fn as_ref(&self) -> &[u8] {
        self.data.as_ref()
    }
}

impl AsMut<[u8]> for Witness {
    fn as_mut(&mut self) -> &mut [u8] {
        self.data.as_mut()
    }
}

impl Extend<u8> for Witness {
    fn extend<T: IntoIterator<Item = u8>>(&mut self, iter: T) {
        self.data.extend(iter);
    }
}

/// Transaction log, a list of all effects of a transaction called [entries](TxEntry).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxLog(Vec<TxEntry>);

/// Entry in a transaction log. All entries are hashed into a [transaction ID](TxID).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TxEntry {
    /// Transaction [header](self::TxHeader).
    /// This entry is not present in the [transaction log](TxLog), but used only for computing a [TxID](TxID) hash.
    // Header(TxHeader),
    ///Input entry that signals that a input utxo was consumed
    Input(Utxo),
    /// Output entry that signals that a output utxo was created. Contains the Output::Coin.
    Output(Output),
    /// Amount of fee being paid (transaction may have multiple fee entries).
    Fee(u64),
    /// Plain data entry created by [`log`](crate::ops::Instruction::Log) instruction. Contains an arbitrary binary string.
    Data(Vec<u8>),
}
// //TODO Can be a good approach
// /// Header metadata for the transaction
// #[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
// pub struct TxHeader {
//     /// Version of the transaction
//     pub version: u64,

//     /// Timestamp before which tx is invalid (in milliseconds since the Unix epoch)
//     pub mintime_ms: u64,

//     /// Timestamp after which tx is invalid (in milliseconds since the Unix epoch)
//     pub maxtime_ms: u64,
// }

impl TxEntry {
    /// Converts entry to the input and provides its contract ID.
    pub fn as_input(&self) -> Option<Utxo> {
        match self {
            TxEntry::Input(cid) => Some(*cid),
            _ => None,
        }
    }

    /// Converts entry to the output and provides a reference to its contract.
    pub fn as_output(&self) -> Option<&Output> {
        match self {
            TxEntry::Output(c) => Some(c),
            _ => None,
        }
    }
}

impl TxLog {
    /// Total amount of fees paid in the transaction
    pub fn fee(&self) -> u64 {
        self.0
            .iter()
            .map(|e| if let TxEntry::Fee(f) = e { *f } else { 0 })
            .sum()
    }

    /// Adds an entry to the txlog.
    pub fn push(&mut self, item: TxEntry) {
        self.0.push(item);
    }

    /// Iterator over the input entries
    pub fn inputs(&self) -> impl Iterator<Item = &Utxo> {
        self.0.iter().filter_map(|entry| match entry {
            TxEntry::Input(utxo) => Some(utxo),
            _ => None,
        })
    }

    /// Iterator over the output entries
    pub fn outputs(&self) -> impl Iterator<Item = &Output> {
        self.0.iter().filter_map(|entry| match entry {
            TxEntry::Output(out) => Some(out),
            _ => None,
        })
    }
}

impl From<Vec<TxEntry>> for TxLog {
    fn from(v: Vec<TxEntry>) -> TxLog {
        TxLog(v)
    }
}

impl Into<Vec<TxEntry>> for TxLog {
    fn into(self) -> Vec<TxEntry> {
        self.0
    }
}
