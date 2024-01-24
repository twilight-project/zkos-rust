#![allow(non_snake_case)]
#![allow(missing_docs)]

use core::panic;

//use crate::readerwriter::{Encodable, ExactSizeEncodable, Writer, WriteError};
use crate::constraints::Commitment;
use crate::encoding::*;
use crate::tx::TxID;
use crate::types::String as ZkvmString;
use bincode;
use bincode::{deserialize, serialize};
use bulletproofs::PedersenGens;
use curve25519_dalek::{ristretto::CompressedRistretto, scalar::Scalar};
use merkle::Hash;
use quisquislib::accounts::{Account, SigmaProof};
use quisquislib::elgamal::ElGamalCommitment;
use quisquislib::keys::PublicKey;
use quisquislib::ristretto::RistrettoPublicKey;
use quisquislib::ristretto::RistrettoSecretKey;
use serde::{Deserialize, Serialize};
use zkschnorr::Signature;

/// Identification of unspend transaction output.
#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct Utxo {
    /// Hash of the transaction
    txid: TxID,
    /// Index of transaction output.
    output_index: u8,
}

impl Utxo {
    pub const fn new(txid: TxID, output_index: u8) -> Self {
        Self { txid, output_index }
    }

    pub const fn from_hash(hash: Hash, output_index: u8) -> Self {
        Self {
            txid: TxID(hash),
            output_index,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serialize(self).unwrap()
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }

    // Create a Utxo struct from Vec<u8>
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        deserialize(bytes).ok()
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let bytes = hex::decode(hex).ok()?;
        Self::from_bytes(&bytes)
    }

    pub const fn tx_id(&self) -> &TxID {
        &self.txid
    }

    pub const fn output_index(&self) -> u8 {
        self.output_index
    }

    pub fn replace_tx_id(&mut self, tx_id: TxID) {
        self.txid = tx_id;
    }
    pub fn tx_id_to_hex(&self) -> String {
        hex::encode(self.txid.0 .0)
    }

    pub fn tx_id_to_vec(&self) -> Vec<u8> {
        self.txid.0 .0.to_vec()
    }
    /// Used in test suites to create a random UtxoID
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let txid: [u8; 32] = rand::Rng::gen(&mut rng);
        let hash: Hash = merkle::Hash(txid);
        Utxo {
            txid: TxID(hash),
            output_index: 0,
        }
    }
}
///Default returns a Utxo with id = 0 and witness index = 0
impl Default for Utxo {
    fn default() -> Utxo {
        let id: [u8; 32] = [0; 32];
        let hash: Hash = merkle::Hash(id);
        Utxo {
            txid: TxID(hash),
            output_index: 0,
        }
    }
}
/// Messagetype: burn, App
/// Message implements [`Default`] and returns [`MessageType::App`].
#[derive(Debug, Eq, Copy, Clone, Deserialize, Serialize)]
pub enum MessageType {
    Burn,
    App,
}
impl MessageType {
    pub fn from_u8(byte: u8) -> Result<MessageType, &'static str> {
        use MessageType::*;
        match byte {
            0 => Ok(Burn),
            1 => Ok(App),
            _ => Err("Error::InvalidMessageType"),
        }
    }
    pub fn is_burn(&self) -> bool {
        match *self {
            MessageType::Burn => true,
            _ => false,
        }
    }
    pub fn is_app(&self) -> bool {
        match *self {
            MessageType::App => true,
            _ => false,
        }
    }
    pub fn to_u8(&self) -> u8 {
        match *self {
            MessageType::Burn => 0,
            MessageType::App => 1,
        }
    }
}
impl Default for MessageType {
    fn default() -> MessageType {
        MessageType::App
    }
}
impl PartialEq for MessageType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MessageType::Burn, MessageType::Burn) => true,
            (MessageType::App, MessageType::App) => true,
            _ => false,
        }
    }
}
/// IOtype: Coin, Memo, State
///
/// InputType implements [`Default`] and returns [`InputType::Coin`].
#[derive(Debug, Eq, Copy, Clone, Deserialize, Serialize)]
pub enum IOType {
    Coin,
    Memo,
    State,
}

impl IOType {
    pub fn from_u8(byte: u8) -> Result<IOType, &'static str> {
        use IOType::*;
        match byte {
            0 => Ok(Coin),
            1 => Ok(Memo),
            2 => Ok(State),
            _ => Err("Error::InvalidInputType"),
        }
    }
    pub fn to_usize(&self) -> usize {
        match *self {
            IOType::Coin => 0,
            IOType::Memo => 1,
            IOType::State => 2,
        }
    }
    pub fn is_coin(&self) -> bool {
        match *self {
            IOType::Coin => true,
            _ => false,
        }
    }
    pub fn is_memo(&self) -> bool {
        match *self {
            IOType::Memo => true,
            _ => false,
        }
    }
    pub fn is_state(&self) -> bool {
        match *self {
            IOType::State => true,
            _ => false,
        }
    }
}
impl Default for IOType {
    fn default() -> IOType {
        IOType::Coin
    }
}
impl PartialEq for IOType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (IOType::Coin, IOType::Coin) => true,
            (IOType::Memo, IOType::Memo) => true,
            (IOType::State, IOType::State) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum InputData {
    Coin {
        /// txID, output Index  (Index of transaction output)
        utxo: Utxo,
        /// Output Coin containing address and encryption
        out_coin: OutputCoin,
        ///Index of witness that authorizes spending the coin.
        witness: u8,
    },
    Memo {
        /// txID, output Index  (Index of transaction output)
        utxo: Utxo,
        /// OutputMemo carrying script address/owner address/commitment value/ etc that already exist as output
        out_memo: OutputMemo,
        ///Index of witness that authorizes spending the coin. Value witness is expected
        witness: u8,
        /// Coin value to be created for the input memo
        coin_value: Option<Commitment>,
    },

    State {
        /// txID, output Index  (Index of transaction output)
        utxo: Utxo,
        //OutputState: type holding script address/owner address/commitment value/ state variable list
        out_state: OutputState,
        ///Index of witness that authorizes spending the coin.
        witness: u8,
        ///Additional varibales needed for state transition
        script_data: Option<Vec<ZkvmString>>,
    },
}

impl InputData {
    pub const fn coin(utxo: Utxo, out_coin: OutputCoin, witness: u8) -> Self {
        Self::Coin {
            utxo,
            out_coin,
            witness,
        }
    }

    pub const fn memo(
        utxo: Utxo,
        out_memo: OutputMemo,
        witness: u8,
        coin_value: Option<Commitment>,
    ) -> Self {
        Self::Memo {
            utxo,
            witness,
            out_memo,
            coin_value,
        }
    }

    pub const fn state(
        utxo: Utxo,
        out_state: OutputState,
        script_data: Option<Vec<ZkvmString>>,
        witness: u8,
    ) -> Self {
        Self::State {
            utxo,
            out_state,
            script_data,
            witness,
        }
    }
    pub const fn as_utxo(&self) -> Option<&Utxo> {
        match self {
            Self::Coin { utxo, .. } => Some(utxo),
            Self::Memo { utxo, .. } => Some(utxo),
            Self::State { utxo, .. } => Some(utxo),
        }
    }

    pub fn get_utxo(&self) -> Utxo {
        match self {
            Self::Coin { utxo, .. } => utxo.clone(),
            Self::Memo { utxo, .. } => utxo.clone(),
            Self::State { utxo, .. } => utxo.clone(),
        }
    }
    pub const fn as_utxo_id(&self) -> Option<&TxID> {
        match self {
            Self::Coin { utxo, .. } => Some(&utxo.txid),
            Self::Memo { utxo, .. } => Some(&utxo.txid),
            Self::State { utxo, .. } => Some(&utxo.txid),
        }
    }
    pub const fn owner(&self) -> Option<&String> {
        match self {
            Self::Coin { out_coin, .. } => Some(&out_coin.owner),
            Self::Memo { out_memo, .. } => Some(&out_memo.owner),
            Self::State { out_state, .. } => Some(&out_state.owner),
        }
    }
    pub const fn as_encryption(&self) -> Option<ElGamalCommitment> {
        match self {
            Self::Coin { out_coin, .. } => Some(out_coin.encrypt),
            _ => None,
        }
    }

    pub const fn as_commitment(&self) -> Option<&Commitment> {
        match self {
            Self::Memo { out_memo, .. } => Some(&out_memo.commitment),
            Self::State { out_state, .. } => Some(&out_state.commitment),
            _ => None,
        }
    }
    pub const fn as_script_address(&self) -> Option<&String> {
        match self {
            InputData::Memo { out_memo, .. } => Some(&out_memo.script_address),
            InputData::State { out_state, .. } => Some(&out_state.script_address),
            _ => None,
        }
    }

    pub const fn as_nonce(&self) -> Option<&u32> {
        match self {
            InputData::State { out_state, .. } => Some(&out_state.nonce),
            _ => None,
        }
    }

    pub fn get_witness_index(&self) -> u8 {
        match self {
            InputData::Coin { witness, .. } => *witness,
            InputData::Memo { witness, .. } => *witness,
            InputData::State { witness, .. } => *witness,
        }
    }

    pub const fn as_timebounds(&self) -> Option<&u32> {
        match self {
            InputData::Memo { out_memo, .. } => Some(&out_memo.timebounds),
            InputData::State { out_state, .. } => Some(&out_state.timebounds),
            _ => None,
        }
    }

    pub const fn get_coin_value_from_memo(&self) -> &Option<Commitment> {
        match self {
            InputData::Memo { coin_value, .. } => coin_value,
            _ => &None,
        }
    }

    pub const fn as_memo_data(&self) -> Option<&Vec<ZkvmString>> {
        match self {
            InputData::Memo { out_memo, .. } => out_memo.data.as_ref(),
            _ => None,
        }
    }

    pub const fn as_state_script_data(&self) -> Option<&Vec<ZkvmString>> {
        match self {
            InputData::State { script_data, .. } => script_data.as_ref(),
            _ => None,
        }
    }

    pub const fn as_state_variables(&self) -> Option<&Vec<ZkvmString>> {
        match self {
            InputData::State { out_state, .. } => out_state.state_variables.as_ref(),
            _ => None,
        }
    }
}

impl PartialEq for InputData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (InputData::Coin { utxo, .. }, InputData::Coin { utxo: utxo2, .. }) => utxo == utxo2,
            (InputData::Memo { utxo, .. }, InputData::Memo { utxo: utxo2, .. }) => utxo == utxo2,
            (InputData::State { utxo, .. }, InputData::State { utxo: utxo2, .. }) => utxo == utxo2,
            _ => false,
        }
    }
}

/// A complete twilight typed Input valid for a specific network.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    /// Defines the input type.
    pub in_type: IOType,
    /// The input data corresponding to the input type.
    pub input: InputData,
}

impl Input {
    /// Create a input of Coin which is valid on the given network.
    pub fn coin(data: InputData) -> Input {
        Input {
            in_type: IOType::default(),
            input: data,
        }
    }

    /// Create a input of Memo which is valid on the given network.
    pub fn memo(data: InputData) -> Input {
        Input {
            in_type: IOType::Memo,
            input: data,
        }
    }

    /// Create a input of State which is valid on the given network.
    pub fn state(data: InputData) -> Input {
        Input {
            in_type: IOType::State,
            input: data,
        }
    }
    pub fn as_utxo(&self) -> Option<&Utxo> {
        self.input.as_utxo()
    }

    pub fn get_utxo(&self) -> Utxo {
        self.input.get_utxo()
    }
    pub fn as_utxo_id(&self) -> Option<&TxID> {
        self.input.as_utxo_id()
    }

    pub fn as_encryption(&self) -> Option<ElGamalCommitment> {
        self.input.as_encryption()
    }

    pub fn as_commitment(&self) -> Option<&Commitment> {
        self.input.as_commitment()
    }

    pub fn as_script_address(&self) -> Option<&String> {
        self.input.as_script_address()
    }

    pub fn as_nonce(&self) -> Option<&u32> {
        self.input.as_nonce()
    }

    pub fn get_witness_index(&self) -> u8 {
        self.input.get_witness_index()
    }

    pub fn as_timebounds(&self) -> Option<&u32> {
        self.input.as_timebounds()
    }

    // pub fn as_sigma_proof(&self) -> Option<&SigmaProof> {
    //     self.input.as_sigma_proof()
    // }

    // return out_coin from input
    pub fn as_out_coin(&self) -> Option<&OutputCoin> {
        match self.input {
            InputData::Coin { ref out_coin, .. } => Some(out_coin),
            _ => None,
        }
    }

    // return out_memo from input
    pub fn as_out_memo(&self) -> Option<&OutputMemo> {
        match self.input {
            InputData::Memo { ref out_memo, .. } => Some(out_memo),
            _ => None,
        }
    }

    // return out_state from input
    pub fn as_out_state(&self) -> Option<&OutputState> {
        match self.input {
            InputData::State { ref out_state, .. } => Some(out_state),
            _ => None,
        }
    }

    pub fn as_input_data(&self) -> &InputData {
        &self.input
    }

    //return owner_address from input
    pub fn as_owner_address(&self) -> Option<&String> {
        match self.input {
            InputData::Coin { ref out_coin, .. } => Some(&out_coin.owner),
            InputData::Memo { ref out_memo, .. } => Some(&out_memo.owner),
            InputData::State { ref out_state, .. } => Some(&out_state.owner),
        }
    }

    /// replace the witness index of each input with the new index
    pub fn replace_witness_index(&mut self, witness_index: u8) {
        match self.input {
            InputData::Coin {
                ref mut witness, ..
            } => *witness = witness_index,
            InputData::Memo {
                ref mut witness, ..
            } => *witness = witness_index,
            InputData::State {
                ref mut witness, ..
            } => *witness = witness_index,
        }
    }
    /// function to return the encrypted values for the input to be placed in transaction
    ///
    pub fn verifier_view(&self) -> Input {
        match self.input {
            InputData::Memo {
                ref utxo,
                ref out_memo,
                ref coin_value,
                ref witness,
                ..
            } => {
                let commit: Option<Commitment>;
                if let Some(value) = coin_value {
                    commit = Some(Commitment::Closed(value.to_point()));
                } else {
                    commit = None;
                }
                Input::memo(InputData::memo(
                    utxo.clone(),
                    out_memo.verifier_view(),
                    witness.clone(),
                    commit,
                ))
            }
            InputData::State {
                ref utxo,
                ref out_state,
                ref script_data,
                ref witness,
                ..
            } => {
                let verifier_script_data: Option<Vec<ZkvmString>>;
                if let Some(data) = script_data {
                    let mut script_data: Vec<ZkvmString> = Vec::new();
                    for x in data.iter() {
                        match x {
                            ZkvmString::Commitment(commitment) => {
                                script_data.push(Commitment::Closed(commitment.to_point()).into());
                            }
                            _ => {
                                script_data.push(x.clone());
                            }
                        }
                    }
                    verifier_script_data = Some(script_data);
                } else {
                    verifier_script_data = None;
                }
                Input::state(InputData::state(
                    utxo.clone(),
                    out_state.verifier_view(),
                    verifier_script_data,
                    witness.clone(),
                ))
            }
            _ => self.clone(),
        }
    }
    /// return Input with Witness = 0
    pub fn as_input_for_signing(&self) -> Input {
        match self.input {
            InputData::Coin {
                ref utxo,
                ref out_coin,
                ..
            } => Input::coin(InputData::coin(utxo.clone(), out_coin.clone(), 0)),
            InputData::Memo {
                ref utxo,
                ref out_memo,
                ref coin_value,
                ..
            } => Input::memo(InputData::memo(
                utxo.clone(),
                out_memo.clone(),
                0,
                coin_value.clone(),
            )),
            InputData::State {
                ref utxo,
                ref out_state,
                ref script_data,
                ..
            } => Input::state(InputData::state(
                utxo.clone(),
                out_state.clone(),
                script_data.clone(),
                0,
            )),
        }
    }
    // Works only for Coin Input Type
    pub fn to_quisquis_account(&self) -> Result<Account, &'static str> {
        match self.input {
            InputData::Coin { ref out_coin, .. } => Ok(out_coin.to_quisquis_account()?),
            _ => Err("Error::InvalidInputType. Only allowed for Coin type"),
        }
    }
    // Works only for Coin Input Type
    pub fn input_from_quisquis_account(
        account: &Account,
        utxo: Utxo,
        witness_index: u8,
        net: address::Network,
    ) -> Self {
        let (pk, encryption) = account.get_account();
        // create address from publickey
        let address = address::Address::standard_address(net, pk);
        let out_coin = OutputCoin::new(encryption, address.as_hex());
        Input::coin(InputData::coin(utxo, out_coin, witness_index))
    }
}

/// Define the != operator for Input
///
impl PartialEq for Input {
    fn eq(&self, other: &Self) -> bool {
        self.in_type == other.in_type && self.input == other.input
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OutputData {
    Coin(OutputCoin),
    Memo(OutputMemo),
    State(OutputState),
}

/// A complete twilight typed Coin Output valid for a specific network.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutputCoin {
    /// Encryption to value's quantity
    pub encrypt: ElGamalCommitment,
    /// Owners Address
    pub owner: String,
}
impl OutputCoin {
    pub fn new(encrypt: ElGamalCommitment, owner: String) -> Self {
        Self { encrypt, owner }
    }
    pub fn to_output(&self) -> Output {
        Output::coin(OutputData::Coin(self.clone()))
    }
    pub fn to_input_data(&self, utxo: Utxo, witness_index: u8) -> InputData {
        InputData::coin(utxo, self.clone(), witness_index)
    }
    pub fn to_input(&self, utxo: Utxo, witness_index: u8) -> Input {
        Input::coin(InputData::coin(utxo, self.clone(), witness_index))
    }
    pub fn to_quisquis_account(&self) -> Result<Account, &'static str> {
        let add: address::Address =
            address::Address::from_hex(&self.owner, address::AddressType::Standard)?;
        let pub_key: RistrettoPublicKey = add.as_coin_address().public_key;
        Ok(Account::set_account(pub_key.clone(), self.encrypt.clone()))
    }
}

impl Encodable for OutputCoin {
    fn encode(&self, w: &mut impl Writer) -> Result<(), WriteError> {
        w.write_encryption(b"encrypt", &self.encrypt)?;
        w.write_address(b"address", &self.owner)?;
        Ok(())
    }
}
impl ExactSizeEncodable for OutputCoin {
    fn encoded_size(&self) -> usize {
        64 + 69
    }
}

/// A complete twilight typed Memo Output valid for a specific network.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutputMemo {
    /// Script Address
    pub script_address: String,
    /// Owning address or predicate root.
    pub owner: String,
    /// Pedersen commitment on amount of coins.
    pub commitment: Commitment,
    ///Memo related data. e.g., Order Size / Deposit / PoolSize
    pub data: Option<Vec<ZkvmString>>,
    /// Timebounds
    pub timebounds: u32,
}
impl OutputMemo {
    pub fn new(
        script_address: String,
        owner: String,
        commitment: Commitment,
        data: Option<Vec<ZkvmString>>,
        timebounds: u32,
    ) -> Self {
        Self {
            script_address,
            owner,
            commitment,
            data,
            timebounds,
        }
    }
    pub fn verifier_view(&self) -> OutputMemo {
        // check if any of the data is a comm
        let data_str: Option<Vec<ZkvmString>>;
        match &self.data {
            None => data_str = None,
            Some(x) => {
                // loop over the state variables and convert the commitments to points
                let mut data_string: Vec<ZkvmString> = Vec::new();
                for data in x.iter() {
                    match data {
                        ZkvmString::Commitment(commitment) => {
                            data_string.push(ZkvmString::Commitment(Box::new(Commitment::Closed(
                                commitment.to_point(),
                            ))));
                        }
                        _ => {
                            data_string.push(data.clone());
                        }
                    }
                }
                data_str = Some(data_string);
            }
        }
        OutputMemo {
            script_address: self.script_address.clone(),
            owner: self.owner.clone(),
            commitment: Commitment::Closed(self.commitment.clone().to_point()),
            data: data_str,
            timebounds: self.timebounds,
        }
    }
}

/// Empty OutputMemo for testing
impl Default for OutputMemo {
    fn default() -> Self {
        Self {
            script_address: String::new(),
            owner: String::new(),
            commitment: Commitment::Closed(CompressedRistretto::default()),
            data: None,
            timebounds: 0,
        }
    }
}

/// A complete twilight typed State Output valid for a specific network.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutputState {
    /// nonce for tracking the interations with state
    pub nonce: u32,
    /// Script Address
    pub script_address: String,
    /// Owning address or predicate root.
    pub owner: String,
    /// Pedersen commitment on amount of coins.
    pub commitment: Commitment,
    ///Additional state variable data. Additional data needed as state u64/u32/Scalar/CompressedRistretto
    pub state_variables: Option<Vec<ZkvmString>>,
    /// Timebounds
    pub timebounds: u32,
}
/// Empty OutputState for testing
impl Default for OutputState {
    fn default() -> Self {
        Self {
            nonce: 0,
            script_address: String::new(),
            owner: String::new(),
            commitment: Commitment::Closed(CompressedRistretto::default()),
            state_variables: None,
            timebounds: 0,
        }
    }
}
impl OutputState {
    /// needed at the time of signing and creating the input vector for tx
    pub fn verifier_view(&self) -> Self {
        // convert the value commitmen to point
        // conver the state commitments to point if any
        // witness = 0  for signing
        let state_var;
        match &self.state_variables {
            Some(x) => {
                // loop over the state variables and convert the commitments to points
                let mut state_variables: Vec<ZkvmString> = Vec::new();
                for state_variable in x.iter() {
                    match state_variable {
                        ZkvmString::Commitment(commitment) => {
                            state_variables.push(ZkvmString::Commitment(Box::new(
                                Commitment::Closed(commitment.to_point()),
                            )));
                        }
                        _ => {
                            state_variables.push(state_variable.clone());
                        }
                    }
                }
                state_var = Some(state_variables);
            }
            None => {
                state_var = None;
            }
        }
        OutputState {
            nonce: self.nonce,
            script_address: self.script_address.clone(),
            owner: self.owner.clone(),
            commitment: Commitment::Closed(self.commitment.clone().to_point()),
            state_variables: state_var,
            timebounds: self.timebounds,
        }
    }
}

impl OutputData {
    pub const fn coin(c: OutputCoin) -> Self {
        Self::Coin(c)
    }
    pub const fn memo(c: OutputMemo) -> Self {
        Self::Memo(c)
    }
    pub const fn state(c: OutputState) -> Self {
        Self::State(c)
    }
    pub const fn get_owner_address(&self) -> Option<&String> {
        match self {
            Self::Coin(coin) => Some(&coin.owner),
            Self::Memo(memo) => Some(&memo.owner),
            Self::State(state) => Some(&state.owner),
        }
    }
    pub const fn get_script_address(&self) -> Option<&String> {
        match self {
            Self::Memo(memo) => Some(&memo.script_address),
            Self::State(state) => Some(&state.script_address),
            _ => None,
        }
    }

    pub const fn get_encryption(&self) -> Option<ElGamalCommitment> {
        match self {
            Self::Coin(coin) => Some(coin.encrypt),
            _ => None,
        }
    }

    pub const fn get_commitment(&self) -> Option<&Commitment> {
        match self {
            Self::Memo(memo) => Some(&memo.commitment),
            Self::State(state) => Some(&state.commitment),
            _ => None,
        }
    }

    pub const fn get_nonce(&self) -> Option<&u32> {
        match self {
            Self::State(state) => Some(&state.nonce),
            _ => None,
        }
    }
    pub const fn get_state_data(&self) -> Option<&Vec<ZkvmString>> {
        match self {
            Self::State(state) => state.state_variables.as_ref(),
            _ => None,
        }
    }

    pub const fn get_memo_data(&self) -> Option<&Vec<ZkvmString>> {
        match self {
            Self::Memo(memo) => memo.data.as_ref(),
            _ => None,
        }
    }

    pub const fn get_timebounds(&self) -> Option<&u32> {
        match self {
            Self::Memo(memo) => Some(&memo.timebounds),
            Self::State(state) => Some(&state.timebounds),
            _ => None,
        }
    }
    pub fn get_output_coin(&self) -> Option<&OutputCoin> {
        match self {
            Self::Coin(coin) => Some(coin),
            _ => None,
        }
    }
    pub fn get_output_memo(&self) -> Option<&OutputMemo> {
        match self {
            Self::Memo(memo) => Some(memo),
            _ => None,
        }
    }
    pub fn get_output_state(&self) -> Option<&OutputState> {
        match self {
            Self::State(state) => Some(state),
            _ => None,
        }
    }
}
/// A complete twilight typed Output valid for a specific network.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Output {
    /// Defines the output type.
    pub out_type: IOType,
    /// The input data corresponding to the output type.
    pub output: OutputData,
}

impl Output {
    /// Create a output of Coin which is valid on the given network.
    pub fn coin(data: OutputData) -> Output {
        Output {
            out_type: IOType::default(),
            output: data,
        }
    }

    /// Create a output of Memo which is valid on the given network.
    pub fn memo(data: OutputData) -> Output {
        Output {
            out_type: IOType::Memo,
            output: data,
        }
    }

    /// Create a output of State which is valid on the given network.
    pub fn state(data: OutputData) -> Output {
        Output {
            out_type: IOType::State,
            output: data,
        }
    }

    //get output type
    pub fn get_out_type(&self) -> IOType {
        self.out_type
    }

    /// get output data
    /// return OutputData
    pub fn as_output_data(&self) -> &OutputData {
        &self.output
    }

    //return out_coin from output
    pub fn as_out_coin(&self) -> Option<&OutputCoin> {
        match self.output {
            OutputData::Coin(ref out_coin) => Some(out_coin),
            _ => None,
        }
    }

    //return out_memo from output
    pub fn as_out_memo(&self) -> Option<&OutputMemo> {
        match self.output {
            OutputData::Memo(ref out_memo) => Some(out_memo),
            _ => None,
        }
    }

    //return out_state from output
    pub fn as_out_state(&self) -> Option<&OutputState> {
        match self.output {
            OutputData::State(ref out_state) => Some(out_state),
            _ => None,
        }
    }

    pub fn to_quisquis_account(&self) -> Result<Account, &'static str> {
        match self.output {
            OutputData::Coin(ref out_coin) => Ok(out_coin.to_quisquis_account()?),
            _ => Err("Error::InvalidOutputType. Only allowed for Coin type"),
        }
    }
    /// Create a output of Coin which is valid on the given network.
    pub fn from_quisquis_account(account: Account, net: address::Network) -> Self {
        let (pk, comm) = account.get_account();
        let owner: String = address::Address::standard_address(net, pk).as_hex();
        let coin: OutputCoin = OutputCoin {
            encrypt: comm,
            owner,
        };
        Output::coin(OutputData::coin(coin))
    }
    pub fn to_verifier_view(&self) -> Self {
        match self.output {
            OutputData::Memo(ref out_memo) => {
                let out_memo = out_memo.verifier_view();
                Output::memo(OutputData::Memo(out_memo))
            }
            OutputData::State(ref out_state) => {
                let out_state = out_state.verifier_view();
                Output::state(OutputData::State(out_state))
            }
            _ => self.clone(),
        }
    }
    /// Create a output of Coin which is valid on the given Network
    fn output_from_account(account: Account, net: address::Network) -> Self {
        let (pk, comm) = account.get_account();
        let owner: String = address::Address::standard_address(net, pk).as_hex();
        let coin: OutputCoin = OutputCoin {
            encrypt: comm,
            owner,
        };
        Output::coin(OutputData::coin(coin))
    }
}

//Upcast OutputCoin to Output
impl From<OutputCoin> for Output {
    fn from(x: OutputCoin) -> Self {
        Output::state(OutputData::Coin(x))
    }
}

//Upcast OutputMemo to Output
impl From<OutputMemo> for Output {
    fn from(x: OutputMemo) -> Self {
        Output::state(OutputData::Memo(x))
    }
}
//Upcast OutputState to Output
impl From<OutputState> for Output {
    fn from(x: OutputState) -> Self {
        Output::state(OutputData::State(x))
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Witness {
    // ZkSchnorr Signature
    Signature(Signature),
    //Zero balance proof for TransferTx. Same value proof in case of memo input
    Proof(SigmaProof),
    // Signature and proof over Coin<->Memo pairs for same value in ScriptTx
    ValueWitness(ValueWitness),
    // Signature and Proof over state inputs<-> ScriptTx
    State(StateWitness),
}
use crate::VMError;
impl Witness {
    /// Downcasts Witness to `Signature` type.
    pub fn to_signature(&self) -> Result<Signature, VMError> {
        match self {
            Witness::Signature(x) => Ok(x.clone()),
            _ => Err(VMError::TypeNotSignature),
        }
    }

    /// Downcasts Witness to `StateWitness` type.
    pub fn to_state_witness(&self) -> Result<StateWitness, VMError> {
        match self {
            Witness::State(x) => Ok(x.clone()),
            _ => Err(VMError::TypeNotStateWitness),
        }
    }

    /// Downcasts Witness to `ValueWitness` type.
    /// This is used for the same value proof and signature.
    //Used by CoinInput and MemoInput in script tx
    pub fn to_value_witness(&self) -> Result<ValueWitness, VMError> {
        match self {
            Witness::ValueWitness(x) => Ok(x.clone()),
            _ => Err(VMError::TypeNotValueWitness),
        }
    }
    // Downcasts Witness to `SigmaProof` type.
    pub fn to_sigma_proof(&self) -> Result<SigmaProof, VMError> {
        match self {
            Witness::Proof(x) => Ok(x.clone()),
            _ => Err(VMError::TypeNotSigmaProof),
        }
    }
    /// used for creating the value witness when the input to the tx is a memo
    /// returns the Same value proof and Signature 
    /// @param enc_acc: Account
    /// @param pedersen_commitment: CompressedRistretto
    /// @param value: u64
    /// @param rscalar: Scalar
    /// @return ValueWitness
    pub fn create_witness_for_memo_input(
        // signature: Signature, // Signature over the OutputMemo to be used as input in this tx 
         coin_output: Output, 
         memo_input: Input, 
         ) -> Result<Self, &'static str> {
         // Signature is provided by the owner of the Memo 
         //let sign = signature;
         //create account from the coin output
         let account = coin_output.to_quisquis_account()?;
         
         // extract the commitment and value from the memo input
          // get Pedersen commitment from Memo for same value proof. this value is coming from coin value
          let memo_commitment = match memo_input.as_input_data().get_coin_value_from_memo(){
                 Some(memo) => memo.clone(),
                 None => return Err("Memo commitment does not exist"),
             };
          
          
          // get commitment value and scalar
          let (memo_value, memo_scalar) = match memo_commitment.witness(){
                 Some(x) => x,
                 None => return Err("Memo commitment witness does not exist"),
             };
         
          let perdersen_commitment = memo_commitment.to_point();
          let value_signed_int = match memo_value
              .to_integer(){
                     Ok(x) => x,
                     Err(_) => return Err("Memo commitment value is not an integer"),
              };
              let value = match value_signed_int.to_u64(){
                     Some(x) => x,
                     None => return Err("Memo commitment value is not a u64"),
              };
         //create the SigmaProof over the Input Coin/Memo with the secret key
         let value_proof = quisquislib::accounts::Prover::same_value_compact_prover(
             account,
             memo_scalar,
             Scalar::from(value),
             perdersen_commitment,
         );
         Ok(Witness::from(value_proof))
     }  // Verify Value Witness for Memo Input/ Coin Output
    pub fn verify_witness_for_memo_input(
        &self,
        coin_output: Output,  // coin value account
       // commitment: CompressedRistretto, // commitment of the coin value as provided in the Memo Input
        memo: Input,  

    ) -> Result<bool, &'static str> {
        //verify the Signature over the InputData with the public key
        // Signature is provided by the owner of the Memo
        // Extract OutputMemo from the Input
        // let output_memo = match memo.as_out_memo() {
        //     Some(x) => x,
        //     None => return Err("ValueWitnessVerification Failed: Input has no Output Memo"),
        // };
        // //extract publickey from account
        // let (pk, _) = enc_acc.get_account();
        // // recreate the signature message
        // // create output from OutputMemo
        // let output = Output::from(output_memo.clone()); 
        // //serialize the output for sign verification
        // let message = match bincode::serialize(&output) {
        //     Ok(message) => message,
        //     Err(_) => return Err("ValueWitnessVerification Failed: Memo Output serialization failed"),
        // };
        // extract account from Output Coin
        let account = coin_output.to_quisquis_account()?;
        // extract the commitment of value from Input Memo
        let commitment = match memo.as_input_data().get_coin_value_from_memo(){
            Some(memo) => memo.clone(),
            None => return Err("Memo commitment does not exist"),
        };
        // pk.verify_msg(&message, &self.sign, ("PublicKeySign").as_bytes())?;
        let same_value_proof = self.to_sigma_proof().map_err(|_|"Invalid SigmaProof")?;
        //verify the SigmaProof over the Input Memo/ Output Coin with the public key
        quisquislib::accounts::Verifier::verify_same_value_compact_verifier(
            account,
            commitment.to_point(),
            same_value_proof,
        )?;
        Ok(true)
    }
}
//Upcast Signature to Witness
impl From<Signature> for Witness {
    fn from(x: Signature) -> Self {
        Witness::Signature(x)
    }
}

//Upcast StateWitness to Witness
impl From<StateWitness> for Witness {
    fn from(x: StateWitness) -> Self {
        Witness::State(x)
    }
}

//Upcast Valuewitness to Witness
impl From<ValueWitness> for Witness {
    fn from(x: ValueWitness) -> Self {
        Witness::ValueWitness(x)
    }
}

// Upcast SigmaProof to Witness
impl From<SigmaProof> for Witness {
    fn from(x: SigmaProof) -> Self {
        Witness::Proof(x)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueWitness {
    sign: Signature,
    value_proof: SigmaProof,
}

impl ValueWitness {
    pub fn set_value_witness(sign: Signature, value_proof: SigmaProof) -> Self {
        Self { sign, value_proof }
    }

    pub fn get_signature(&self) -> &Signature {
        &self.sign
    }

    pub fn get_value_proof(&self) -> &SigmaProof {
        &self.value_proof
    }
    /// assuming the inputs passed are already converted to represent verifier view of commitments
    pub fn create_value_witness(
        input: Input,
        secret_key: RistrettoSecretKey,
        output: Output,
        enc_acc: Account,
        pubkey: RistrettoPublicKey,
        pedersen_commitment: CompressedRistretto,
        value: u64,
        rscalar: Scalar, //commitment scalar
    ) -> Self {
        //create the Signature over the Input Coin/Memo with the secret key
        let mut input_verifier_view = input.verifier_view();
        input_verifier_view = input_verifier_view.as_input_for_signing();
        let output_verifier_view = output.to_verifier_view();

        //create message bytes using input and output verifier view
        let mut message: Vec<u8>;
        message = bincode::serialize(&input_verifier_view).unwrap();
        message.extend(bincode::serialize(&output_verifier_view).unwrap());

        //create the signature over the input
        let sign = pubkey.sign_msg(&message, &secret_key, ("ValueSign").as_bytes());
        //create the SigmaProof over the Input Coin/Memo with the secret key
        let value_proof = quisquislib::accounts::Prover::same_value_compact_prover(
            enc_acc,
            rscalar,
            Scalar::from(value),
            pedersen_commitment,
        );
        Self { sign, value_proof }
    }

    pub fn verify_value_witness(
        &self,
        input: Input,
        output: Output,
        pubkey: RistrettoPublicKey,
        enc_acc: Account,
        commitment: CompressedRistretto,
    ) -> Result<bool, &'static str> {
        //create message to verify the Signature over the Input and Output with the public key
        let mut message: Vec<u8>;
        message = match bincode::serialize(&input){
            Ok(x) => x,
            Err(_) => return Err("Serialization Error::Failed to serialize the input for signature verification"),
        
        };
        let output_binary_string = match bincode::serialize(&output){
            Ok(x) => x,
            Err(_) => return Err("Serialization Error::Failed to serialize the output for signature verification"),
        
        };
        message.extend(output_binary_string);
        //verify the Signature over the InputData with the public key

        pubkey.verify_msg(&message, &self.sign, ("ValueSign").as_bytes())?;

        //verify the SigmaProof over the Input Coin/ Output Memo with the public key
        quisquislib::accounts::Verifier::verify_same_value_compact_verifier(
            enc_acc,
            commitment,
            self.value_proof.clone(),
        )?;
        Ok(true)
    }
  
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateWitness {
    sign: Signature,
    zero_proof: Option<Vec<Scalar>>,
}

impl StateWitness {
    pub fn set_state_witness(sign: Signature, zero_proof: Option<Vec<Scalar>>) -> Self {
        Self { sign, zero_proof }
    }

    pub fn get_sign(&self) -> &Signature {
        &self.sign
    }

    pub fn create_state_witness(
        input: &Input,
        output: &Output,
        secret_key: RistrettoSecretKey,
        pubkey: RistrettoPublicKey,
        contract_deploy_flag: bool,
    ) -> Self {
        //create the Signature over the Input State with the secret key

        //create input State for verifier view
        let mut verifier_input = input.verifier_view();
        // set witness = 0
        verifier_input = verifier_input.as_input_for_signing();

        //The sign must happen on the verifier View of the input so that it can be verified acorrectly

        //create verifier view of the output
        let verifier_output = output.to_verifier_view();

        //create message bytes using input_state + output_state
        let mut message: Vec<u8>;
        message = bincode::serialize(&verifier_input).unwrap();
        message.extend(bincode::serialize(&verifier_output).unwrap());

        //println!("message  {:?}", message);
        let sign = pubkey.sign_msg(&message, &secret_key, ("StateSign").as_bytes());
        // Contract calling
        if !contract_deploy_flag {
            return Self {
                sign,
                zero_proof: None,
            };
        } else {
            // create the zero proof over the input state commitment and state variables
            let state_var = input.as_out_state().unwrap();
            let state_commitment = state_var.clone().commitment;
            // get value, witness for state commitment
            let (state_value, state_value_blinding) = state_commitment.witness().unwrap();
            // check if value is zero
            if state_value != 0.into() {
                panic!("Error::The value of the state commitment is not zero");
            }
            // create the zero proof over the state commitment and state variables
            let mut zero_proof: Vec<Scalar> = Vec::new();
            zero_proof.push(state_value_blinding);
            // get the state variables
            let state_variables = state_var.state_variables.as_ref();
            match state_variables {
                Some(x) => {
                    // loop over the state variables and create the zero proofs
                    for state_variable in x.iter() {
                        match state_variable {
                            ZkvmString::Commitment(commitment) => {
                                let (value, blinding) = commitment.witness().unwrap();
                                if value != 0.into() {
                                    panic!("Error::The value of the state variable is not zero");
                                }
                                zero_proof.push(blinding);
                            }
                            _ => {}
                        }
                    }
                    return Self {
                        sign,
                        zero_proof: Some(zero_proof),
                    };
                }
                None => {
                    return Self {
                        sign,
                        zero_proof: Some(zero_proof),
                    };
                }
            }
        }
    }
    /// verify_state_witness verifies the zero value proof and signature
    /// invoked if a new contract has to be deployed.
    /// fails if and value or state witness is not zero
    pub fn verify_state_witness(
        &self,
        input: Input,
        output: Output,
        pubkey: RistrettoPublicKey,
        contract_deploy_flag: bool,
    ) -> Result<bool, &'static str> {
        //create message to verify the Signature over the Input State with the public key

        // The witness is 0 for the purposes of signature verification
        //recreate the input state with the Witness as zero
        let verifier_input = input.as_input_for_signing();

        //create message bytes using input_state + output_state
        let mut message: Vec<u8>;
        message = bincode::serialize(&verifier_input).map_err(|_| {
            " Serialization Error::Failed to serialize the input for signature verification"
        })?;

        message.extend(bincode::serialize(&output).map_err(|_| {
            "Serialization Error::Failed to serialize the output for signature verification"
        })?);
        //verify the Signature over the Input state with the public key

        // println!("\n \nmessage  {:?}", message);
        let verify_sig = pubkey.verify_msg(&message, &self.sign, ("StateSign").as_bytes());
        if verify_sig.is_err() {
            return Err("Input State Signature verification failed");
        }
        // Contract calling
        if !contract_deploy_flag {
            return Ok(true);
        } else {
            // Contract is being deployed so verify the zero proofs
            // Both for state commitments and state variables
            let in_state = input.input;
            // the first index of zero_proof is the Zero commitment on Value state
            let zero_witness_proof_scalar = self.zero_proof.as_ref().unwrap()[0].clone();
            // recreate the commitment using the zero scalar
            let gens = PedersenGens::default();
            let proof = gens.commit(0u64.into(), zero_witness_proof_scalar);
            let state_commit = in_state.as_commitment().unwrap().to_point();
            // compare the zero committed points
            if state_commit != proof.compress() {
                return Err("Error::The zero proof does not match the state commitment");
            }
            //get extra state_variables if available
            if in_state.as_state_variables().is_some() {
                //verify the zero proofs if any over the state variables
                let state_variables = in_state.as_state_variables().unwrap();
                let commitment_witness = self.zero_proof.as_ref().unwrap();
                if commitment_witness.len() - 1 > state_variables.len() {
                    return Err("Error::There are more zero proofs than state variables");
                }
                //index is to go through the proofs against the committed variables
                let mut index: usize = 1;

                for variable in state_variables {
                    match variable {
                        ZkvmString::Commitment(x) => {
                            let state_comit = x.to_point();
                            //recreate commitment using 0 as value and scalar from the proof
                            let proof_point = gens
                                .commit(0u64.into(), commitment_witness[index].clone())
                                .compress();
                            //verify the proof
                            if state_comit != proof_point {
                                return Err(
                                    "Error::The zero proof does not match the state variable",
                                );
                            }
                            index += 1;
                        }
                        _ => {}
                    }
                }
            }

            Ok(true)
        }
    }
}
//return iterator over zero proofs  (if any)
impl StateWitness {
    pub fn get_zero_proof(&self) -> Option<impl Iterator<Item = &Scalar>> {
        self.zero_proof.as_ref().map(|x| x.iter())
    }
}
