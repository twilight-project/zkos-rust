#![allow(non_snake_case)]
//#![deny(missing_docs)]


//use crate::readerwriter::{Encodable, ExactSizeEncodable, Writer, WriteError};
use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;
use quisquislib::elgamal::ElGamalCommitment;
use serde::{Deserialize, Serialize};
use crate::util::Address;
use quisquislib::accounts::Account;


/// Transaction ID is a unique 32-byte identifier of a transaction effects represented by `TxLog`.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxId(pub [u8; 32]);

impl TxId {
    // Convert the TxId to a hexadecimal string
    pub fn to_hex_string(&self) -> String {
        let hex_chars: Vec<String> = self.0.iter().map(|byte| format!("{:02x}", byte)).collect();
        hex_chars.join("")
    }

    pub fn from_vec(vec_u8: Vec<u8>) -> TxId {
        let mut array_u8: [u8; 32] = [0; 32];
        if vec_u8.len() == 32 {
            array_u8.copy_from_slice(&vec_u8);
        } else {
            panic!("The Vec<u8> must contain exactly 32 bytes");
        }
        TxId(array_u8)
    }

}


/// Transaction type: Transfer. Script, Vault
///
/// TransactionType implements [`Default`] and returns [`TransactionType::Transfer`].
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    Script,
    Vault,
}

impl TransactionType {
    pub fn from_u8(byte: u8) -> Result<TransactionType, &'static str> {
        use TransactionType::*;
        match byte {
            0 => Ok(Transfer),
            1 => Ok(Script),
            2 => Ok(Vault),
            _ => Err("Error::InvalidTransactionType"),
        }
    }
    pub fn is_transfer(&self) -> bool {
        match *self {
            TransactionType::Transfer => true,
            _ => false,
        }
    }
    pub fn is_script(&self) -> bool {
        match *self {
            TransactionType::Script => true,
            _ => false,
        }
    }
}
impl Default for TransactionType {
    fn default() -> TransactionType {
        TransactionType::Transfer
    }
}

/// Identification of transaction in a block.
// #[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
// pub struct TxPointer {
//     /// block id
//     block_height: u64,
//     /// output index
//     tx_index: u16,
// }

/// Identification of unspend transaction output.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Utxo {
    /// Hash of the transaction
    txid: TxId,
    /// Index of transaction output.
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
///Default returns a Utxo with id = 0 and witness index = 0
/// 
impl Default for Utxo {
    fn default() -> Utxo {
        let id: [u8;32]= [0 ; 32]; 
        Utxo { 
            txid: TxId(id),
            output_index: 0,
        }
    }
}

/// Input type: Coin, Memo, State
///
/// InputType implements [`Default`] and returns [`InputType::Coin`].
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum InputType {
    Coin,
    Memo,
    State,
}

impl InputType {
    pub fn from_u8(byte: u8) -> Result<InputType, &'static str> {
        use InputType::*;
        match byte {
            0 => Ok(Coin),
            1 => Ok(Memo),
            2 => Ok(State),
            _ => Err("Error::InvalidInputType"),
        }
    }
    pub fn is_coin(&self) -> bool {
        match *self {
            InputType::Coin => true,
            _ => false,
        }
    }
    pub fn is_memo(&self) -> bool {
        match *self {
            InputType::Memo => true,
            _ => false,
        }
    }
    pub fn is_state(&self) -> bool {
        match *self {
            InputType::State => true,
            _ => false,
        }
    }
}
impl Default for InputType {
    fn default() -> InputType {
        InputType::Coin
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputData {
    Coin {
        /// txID, output Index  (Index of transaction output) 
        utxo: Utxo, 
        /// Owning address or predicate root.
        owner: Address, 
        /// Elgamal encryption on amount of coins.
        encryption: ElGamalCommitment,
        //coin: Coin,
        ///Index of witness that authorizes spending the coin.
        witness: u8,

        account: Account

      //  Pedersenproof	byte[]	pedersena and corresponsing sigma proof.
//calldata	byte[]	call proof.
//program	byte[]	Call data for state execution.
//data	byte[]	Arbitrary data
    },
    Memo {
        /// txID, output Index  (Index of transaction output) 
        utxo: Utxo,
        /// Script Address
        script_address: Address, 
        /// Owning address or predicate root.
        owner: Address, 
       /// Pedersen commitment on amount of coins.
        commitment: CompressedRistretto, 
        ///Additional varibales
        data: Option<u32>,	
        ///Index of witness that authorizes spending the coin.
        witness: u8,
        //UTXO being spent must have been created at least this many blocks ago.
        //timebounds:	u32,	
        //Index of witness that contains the program
        //program_index: u8, There is no program with memo
    },

    State {
          /// txID, output Index  (Index of transaction output) 
          utxo: Utxo,
          /// nonce for tracking the interations with state
          nonce: u32, 
          /// Script Address
          script_address: Address, 
          /// Owning address or predicate root.
          owner: Address, 
         /// Pedersen commitment on amount of coins.
          commitment: CompressedRistretto, 
          ///Script data. Additional data needed as state u64/u32/Scalar/CompressedRistretto
          script_data: Option<Scalar>,	
          ///Index of witness that authorizes spending the coin.
          witness: u8,
          //UTXO being spent must have been created at least this many blocks ago.
          //timebounds:	u32,	
          ///Index of witness that contains the program
          program_index: u8,
    },

}

impl InputData {
    //pub const fn coin(utxo: Utxo, coin: Coin, witness: u8) -> Self {
      //  Self::Coin { utxo, coin, witness}
    //}
    pub const fn coin(utxo: Utxo, owner: Address, encryption: ElGamalCommitment, witness: u8, account: Account) -> Self {
        Self::Coin { utxo, owner, encryption, witness, account}
    }

    pub const fn memo(utxo: Utxo, script_address: Address, owner: Address, commitment: CompressedRistretto, data: Option<u32>, witness: u8, ) -> Self {
        Self::Memo { utxo, script_address, owner, commitment, data, witness}
    }

    pub const fn state(utxo: Utxo, nonce:u32, script_address: Address, owner: Address, commitment: CompressedRistretto, script_data: Option<Scalar>, witness: u8, program_index:u8) -> Self {
        Self::State { utxo, nonce, script_address, owner, commitment, script_data, witness, program_index}
    }
   // pub const fn memo()
    pub const fn as_utxo_id(&self) -> Option<&Utxo> {
        match self {
            Self::Coin { utxo, .. } => Some(utxo),
            Self::Memo { utxo, .. } => Some(utxo),
            Self::State { utxo, .. } => Some(utxo),
            _ => None,
        }
    }

    pub const fn as_owner(&self) -> Option<&Address> {
        match self {
            Self::Coin { owner, .. } => Some(owner),
            Self::Memo { owner, .. } => Some(owner),
            Self::State { owner, .. } => Some(owner),
            _ => None,
        }
    }

    pub const fn account(&self) -> Option<Account> {
        match self {
            Self::Coin {account , ..} => Some(*account),
            _ => None,
        }
    }

    pub const fn as_encryption(&self) -> Option<&ElGamalCommitment> {
        match self {
            Self::Coin { encryption, .. } => Some(encryption),
            _ => None,
        }
    }

    pub const fn as_commitment(&self) -> Option<&CompressedRistretto> {
        match self {
            Self::Memo { commitment, .. } => Some(commitment),
            Self::State { commitment, .. } => Some(commitment),
            _ => None,
        }
    }
    pub const fn as_script_address(&self) -> Option<&Address> {
        match self {
            InputData::Memo { script_address, .. } => Some(script_address),
            InputData::State { script_address, .. } => Some(script_address),
            _ => None,
        }
    }

   pub const fn as_nonce(&self) -> Option<&u32> {
        match self {
            InputData::State { nonce, .. } => Some(nonce),
            _ => None,
        }
    }
    pub const fn as_witness_index(&self) -> Option<&u8> {
        match self {
            InputData::Coin { witness, .. } => Some(witness),
            InputData::Memo { witness, .. } => Some(witness),
            InputData::State { witness, .. } => Some(witness),
            _ => None,
        }
    }

}

/// A complete twilight typed Input valid for a specific network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    /// Defines the input type.
    pub in_type: InputType,
    /// The input data corresponding to the input type.
    pub input: InputData,
}

impl Input {
    /// Create a input of Coin which is valid on the given network.
    pub fn coin(data: InputData) -> Input {
        Input {
            in_type: InputType::default(),
            input: data,
        }
    }

    /// Create a input of Memo which is valid on the given network.
    pub fn memo(data: InputData) -> Input {
        Input {
            in_type: InputType::Memo,
            input: data,
        }
    }

    /// Create a input of State which is valid on the given network.
    pub fn state(data: InputData) -> Input {
        Input {
            in_type: InputType::State,
            input: data,
        }
    }
}

/// Output type: Dark, Record,
///
/// OutputType implements [`Default`] and returns [`OutputType::Dark`].
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum OutputType {
    Coin,
    Memo,
    State,
}

impl OutputType {
    pub fn from_u8(byte: u8) -> Result<OutputType, &'static str> {
        use OutputType::*;
        match byte {
            0 => Ok(Coin),
            1 => Ok(Memo),
            2 => Ok(State),
            _ => Err("Error::InvalidInputType"),
        }
    }
    pub fn is_coin(&self) -> bool{
        match *self {
            OutputType::Coin => true,
            _ => false,
        }
    }

    pub fn is_memo(&self) -> bool{
        match *self {
            OutputType::Memo => true,
            _ => false,
        }
    }
    pub fn is_state(&self) -> bool{
        match *self {
            OutputType::State => true,
            _ => false,
        }
    }
}
impl Default for OutputType {
    fn default() -> OutputType {
        OutputType::Coin
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OutputData {
    Coin(Coin), 
    Memo (Memo),
    State (State),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Coin {
    /// Encryption to value's quantity
    pub encrypt: ElGamalCommitment,
    /// Owners Address
    pub address: Address,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Memo {
   pub contract: CData,
   ///Additional varibales
   pub data: Option<u32>,	
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct State {
    /// nonce for tracking the interations with state
    pub nonce: u32, 
    pub contract: CData,
    ///Script data. Additional data needed as state u64/u32/Scalar/CompressedRistretto
    pub script_data: Option<Scalar>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CData {
    /// Script Address
    pub script_address: Address, 
    /// Owning address or predicate root.
    pub owner: Address, 
    /// Pedersen commitment on amount of coins.
    pub commitment: CompressedRistretto, 
}

impl OutputData {
    pub const fn coin(c: Coin) -> Self {
        Self::Coin(c)
    }
    pub const fn memo(c: Memo) -> Self {
        Self::Memo(c)
    }
    pub const fn state(c: State ) -> Self {
        Self::State(c)
    }
    pub const fn get_owner_address(&self) -> Option<&Address> {
         match self {
             Self::Coin(coin) => Some(&coin.address),
             Self::Memo(memo) => Some(&memo.contract.owner),
             Self::State(state) => Some(&state.contract.owner),   
             _ => None,
         }
    }
    pub const fn get_script_address(&self) -> Option<&Address> {
        match self {
            Self::Memo(memo) => Some(&memo.contract.script_address),
            Self::State(state) => Some(&state.contract.script_address),   
            _ => None,
        }
   }
    pub const fn get_encryption(&self) -> Option<&ElGamalCommitment> {
        match self {
            Self::Coin(coin) => Some(&coin.encrypt),
            _ => None,
        }
   }

    pub const fn get_commitment(&self) -> Option<&CompressedRistretto> {
        match self {
            Self::Memo(memo) => Some(&memo.contract.commitment),
            Self::State(state) => Some(&state.contract.commitment),
            _ => None,
        }
    }

    pub const fn adress(&self) -> Option<Address> {
        match self {
            Self::Coin(coin) => Some(coin.address),
            _ => None,
        }
    }


    pub const fn get_nonce(&self) -> Option<&u32> {
        match self {
            Self::State(state) => Some(&state.nonce),
            _ => None,
        }
    }

    pub const fn commitment(&self) -> Option<ElGamalCommitment> {
        match self {
            Self::Coin(coin) => Some(coin.encrypt),
            _ => None,
        }
    }
    pub const fn get_script_data(&self) -> Option<Scalar> {
        match self {
            Self::State(state) => state.script_data,
            _ => None,
        }
    }

    pub const fn get_data(&self) -> Option<u32> {
        match self {
            Self::Memo(memo) => memo.data,
            _ => None,
        }
    }

}

/// A complete twilight typed Output valid for a specific network.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Output {
    /// Defines the output type.
    pub out_type: OutputType,
    /// The input data corresponding to the output type.
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

    pub fn memo(data: OutputData) -> Output {
        Output {
            out_type: OutputType::Memo,
            output: data,
        }
    }
    pub fn state(data: OutputData) -> Output {
        Output {
            out_type: OutputType::State,
            output: data,
        }
    }
    

}
// impl Default for Output {
//     fn default() -> Self {
//         let out_type = OutputType::default();

//         let c: Coin = 
//         Self::Coin {
//             account: Default::default(),
//         }
//     }
// }

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
#[derive(Clone, Debug)]
pub struct TxLog(Vec<TxEntry>);

/// Entry in a transaction log. All entries are hashed into a [transaction ID](TxID).
#[derive(Clone, Debug)]
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

/// DataEntry in Memo/State.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DataEntry {
    ///Input entry that signals that a input utxo was consumed
    Scalar(Scalar),
    /// Output entry that signals that a output utxo was created. Contains the Output::Coin.
    Commitment(CompressedRistretto),
   /// Plain data entry 
    Data(u64),
}

impl DataEntry {
    /// Converts dataentry to the Scalar.
    pub fn as_scalar(&self) -> Option<Scalar> {
        match self {
            DataEntry::Scalar(cid) => Some(*cid),
            _ => None,
        }
    }

    /// Converts dataentry to the Commitment.
    pub fn as_commitment(&self) -> Option<CompressedRistretto> {
        match self {
            DataEntry::Commitment(cid) => Some(*cid),
            _ => None,
        }
    }

    /// Converts data entry to Plain Data.
    pub fn as_plain_data(&self) -> Option<&u64> {
        match self {
            DataEntry::Data(c) => Some(c),
            _ => None,
        }
    }
}

// impl TxLog {
//     /// Total amount of fees paid in the transaction
//     pub fn fee(&self) -> u64 {
//         self.0
//             .iter()
//             .map(|e| if let TxEntry::Fee(f) = e { *f } else { 0 })
//             .sum()
//     }

//     /// Adds an entry to the txlog.
//     pub fn push(&mut self, item: TxEntry) {
//         self.0.push(item);
//     }

//     /// Iterator over the input entries
//     pub fn inputs(&self) -> impl Iterator<Item = &Utxo> {
//         self.0.iter().filter_map(|entry| match entry {
//             TxEntry::Input(utxo) => Some(utxo),
//             _ => None,
//         })
//     }

//     /// Iterator over the output entries
//     pub fn outputs(&self) -> impl Iterator<Item = &Output> {
//         self.0.iter().filter_map(|entry| match entry {
//             TxEntry::Output(out) => Some(out),
//             _ => None,
//         })
//     }
// }

// impl From<Vec<TxEntry>> for TxLog {
//     fn from(v: Vec<TxEntry>) -> TxLog {
//         TxLog(v)
//     }
// }

// impl Into<Vec<TxEntry>> for TxLog {
//     fn into(self) -> Vec<TxEntry> {
//         self.0
//     }
// }
