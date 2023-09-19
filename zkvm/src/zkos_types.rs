#![allow(non_snake_case)]
#![allow(missing_docs)]

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

/// Identification of transaction in a block.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct TxPointer {
    /// block id
    block_height: u64,
    /// output index
    tx_index: u16,
}

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
}
///Default returns a Utxo with id = 0 and witness index = 0
///
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

/// IOtype: Coin, Memo, State
///
/// InputType implements [`Default`] and returns [`InputType::Coin`].
#[derive(Debug, PartialEq, Eq, Copy, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum InputData {
    Coin {
        /// txID, output Index  (Index of transaction output)
        utxo: Utxo,
        // Owning address or predicate root.
        // owner: String,
        // Elgamal encryption on amount of coins.
        //encryption: ElGamalCommitment,
        out_coin: OutputCoin,

        //coin: Coin,
        ///Index of witness that authorizes spending the coin.
        witness: u8,
        //Same value proof.  Pedersen will come from corresponding output MEMO
        //proof: Option<SigmaProof>,

        //program length
        // program_length: Option<u16>,
        //program
        // program: Option<Vec<u8>>,
        //arbitrary data
        //data: Option<ZkvmString>,
    },
    Memo {
        /// txID, output Index  (Index of transaction output)
        utxo: Utxo,
        // Script Address
        // script_address: String,
        // Owning address or predicate root.
        // owner: String,
        // Pedersen commitment on amount of coins.
        //commitment: CompressedRistretto,
        ///Additional varibales
        // data: ZkvmString,
        out_memo: OutputMemo,
        ///Index of witness that authorizes spending the coin.
        witness: u8,
        //UTXO being spent must have been created at least this many blocks ago.
        //timebounds: u32,
        ///Same value proof. Encryption will come from corresponding output COIN
        //proof: Option<SigmaProof>,

        ///same value proof Commitment Value. SHOULD BE REMOVED LATER
        /// ?????????????? Needed because the outout is encrypted with arbitrary value not necessarily the same as the Output commitment
        /// inside Input here
        commitment_proof_value: Commitment,
        //program length
        //  program_length: u16,
        //program
        //   program: Vec<u8>,
    },

    State {
        /// txID, output Index  (Index of transaction output)
        utxo: Utxo,
        /// Nonce. tracks all the interactuons with the state
        //nonce: u32,
        /// Script Address
        //script_address: String,
        /// Owning address or predicate root.
        // owner: String,
        /// Pedersen commitment on amount of coins.
        //commitment: CompressedRistretto,
        out_state: OutputState,
        ///Index of witness that authorizes spending the coin.
        witness: u8,
        //UTXO being spent must have been created at least this many blocks ago.
        //timebounds: u32,
        ///program length
        // program_length: u16,
        ///program
        // program: Vec<u8>,

        ///Additional varibales
        script_data: Option<ZkvmString>,
    },
}

impl InputData {
    //pub const fn coin(utxo: Utxo, coin: Coin, witness: u8) -> Self {
    //  Self::Coin { utxo, coin, witness}
    //}
    pub const fn coin(
        utxo: Utxo,
        // owner: String,
        // encryption: ElGamalCommitment,
        out_coin: OutputCoin,
        witness: u8,
        // proof: Option<SigmaProof>,
        // program_length: Option<u16>,
        // program: Option<Vec<u8>>,
        // data: Option<ZkvmString>,
    ) -> Self {
        Self::Coin {
            utxo,
            out_coin,
            witness,
            // proof,
            //   program_length,
            //  program,
            //data,
        }
    }

    pub const fn memo(
        utxo: Utxo,
        // script_address: String,
        // owner: String,
        // commitment: CompressedRistretto,
        // data: ZkvmString,
        out_memo: OutputMemo,
        witness: u8,
        // timebounds: u32,
        //proof: Option<SigmaProof>,
        commitment_proof_value: Commitment,
        // program_length: u16,
        // program: Vec<u8>,
    ) -> Self {
        Self::Memo {
            utxo,
            // script_address,
            // owner,
            // commitment,
            // data,
            witness,
            // timebounds,
            out_memo,
            //proof,
            commitment_proof_value,
            //  program_length,
            //  program,
        }
    }

    pub const fn state(
        utxo: Utxo,
        // nonce: u32,
        // script_address: String,
        // owner: String,
        // commitment: CompressedRistretto,
        out_state: OutputState,
        script_data: Option<ZkvmString>,
        witness: u8,
        // timebounds: u32,
        // program_length: u16,
        // program: Vec<u8>,
    ) -> Self {
        Self::State {
            utxo,
            out_state,
            //nonce,
            //script_address,
            //owner,
            //commitment,
            script_data,
            witness,
            // timebounds,
            // program_length,
            // program,
        }
    }
    // pub const fn memo()
    pub const fn as_utxo(&self) -> Option<&Utxo> {
        match self {
            Self::Coin { utxo, .. } => Some(utxo),
            Self::Memo { utxo, .. } => Some(utxo),
            Self::State { utxo, .. } => Some(utxo),
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
    pub const fn as_witness_index(&self) -> Option<&u8> {
        match self {
            InputData::Coin { witness, .. } => Some(witness),
            InputData::Memo { witness, .. } => Some(witness),
            InputData::State { witness, .. } => Some(witness),
        }
    }

    pub const fn as_timebounds(&self) -> Option<&u32> {
        match self {
            InputData::Memo { out_memo, .. } => Some(&out_memo.timebounds),
            InputData::State { out_state, .. } => Some(&out_state.timebounds),
            _ => None,
        }
    }

    /*  pub const fn as_sigma_proof(&self) -> Option<&SigmaProof> {
        match self {
            InputData::Coin { proof, .. } => proof.as_ref(),
            InputData::Memo { proof, .. } => proof.as_ref(),
            _ => None,
        }
    }*/

    pub const fn get_commitment_value_memo(&self) -> Option<&Commitment> {
        match self {
            InputData::Memo {
                commitment_proof_value,
                ..
            } => Some(commitment_proof_value),
            _ => None,
        }
    }

    // pub const fn as_program_length(&self) -> Option<&u16> {
    //     match self {
    //         InputData::Coin { program_length, .. } => program_length.as_ref(),
    //         InputData::Memo { program_length, .. } => Some(program_length),
    //         InputData::State { program_length, .. } => Some(program_length),
    //     }
    // }

    // pub const fn as_program(&self) -> Option<&Vec<u8>> {
    //     match self {
    //         InputData::Coin { program, .. } => program.as_ref(),
    //         InputData::Memo { program, .. } => Some(program),
    //         InputData::State { program, .. } => Some(program),
    //     }
    // }

    pub const fn as_memo_data(&self) -> Option<&ZkvmString> {
        match self {
            // InputData::Coin { data, .. } => data.as_ref(),
            InputData::Memo { out_memo, .. } => out_memo.data.as_ref(),
            _ => None,
            //InputData::State { script_data, .. } => Some(script_data),
        }
    }

    pub const fn as_state_script_data(&self) -> Option<&ZkvmString> {
        match self {
            InputData::State { script_data, .. } => script_data.as_ref(),
            _ => None,
            //InputData::State { script_data, .. } => Some(script_data),
        }
    }

    pub const fn as_state_variables(&self) -> Option<&Vec<ZkvmString>> {
        match self {
            InputData::State { out_state, .. } => out_state.state_variables.as_ref(),
            _ => None,
            //InputData::State { script_data, .. } => Some(script_data),
        }
    }

    // pub const fn as_script_data(&self) -> Option<&ZkvmString> {
    //     match self {
    //         InputData::State {
    //             script_data: ZkvmString,
    //             ..
    //         } => Some(script_data),
    //         _ => None,
    //     }
    //}
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

    pub fn as_witness_index(&self) -> Option<&u8> {
        self.input.as_witness_index()
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

    //return owner_address from input
    pub fn as_owner_address(&self) -> Option<&String> {
        match self.input {
            InputData::Coin { ref out_coin, .. } => Some(&out_coin.owner),
            InputData::Memo { ref out_memo, .. } => Some(&out_memo.owner),
            InputData::State { ref out_state, .. } => Some(&out_state.owner),
        }
    }

    // return Input with Witness = 0 for signing
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
                ref commitment_proof_value,
                ..
            } => Input::memo(InputData::memo(
                utxo.clone(),
                out_memo.clone(),
                0,
                commitment_proof_value.clone(),
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
            InputData::Coin { ref out_coin, .. } => Ok(out_coin.to_quisquis_account()),
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
    pub fn to_quisquis_account(&self) -> Account {
        let add: address::Address =
            address::Address::from_hex(&self.owner, address::AddressType::Standard).unwrap();
        let pub_key: RistrettoPublicKey = add.as_coin_address().public_key;
        Account::set_account(pub_key.clone(), self.encrypt.clone())
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
    ///Script Data (Address/Owner/Commitment)
    // pub contract_info: Contract,
    /// Script Address
    pub script_address: String,
    /// Owning address or predicate root.
    pub owner: String,
    /// Pedersen commitment on amount of coins.
    pub commitment: Commitment,
    ///Memo related data. e.g., Order Size
    pub data: Option<ZkvmString>,
    /// Timebounds
    pub timebounds: u32,
}
impl OutputMemo {
    pub fn new(
        script_address: String,
        owner: String,
        commitment: Commitment,
        data: Option<ZkvmString>,
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
        // let mut data_str: ZkvmString;
        // if self.data.is_some() {
        //     match self.data.unwrap() {
        //         ZkvmString::Commitment(commitment) => {
        //         data_str = Commitment::Closed(self.data.to_point());

        //     }

        //

        //         OutputMemo {
        //             script_address: self.script_address.clone(),
        //             owner: self.owner.clone(),
        //             commitment: Commitment::Closed(data_commitment.clone().to_point()),
        //             data: self.data.clone(),
        //             timebounds: self.timebounds,
        //         }
        //     }
        //     _ => OutputMemo {
        //         script_address: self.script_address.clone(),
        //         owner: self.owner.clone(),
        //         commitment: Commitment::Closed(self.commitment.clone().to_point()),
        //         data: self.data.clone(),
        //         timebounds: self.timebounds,
        //     },
        //   }
        // }
        OutputMemo {
            script_address: self.script_address.clone(),
            owner: self.owner.clone(),
            commitment: Commitment::Closed(self.commitment.clone().to_point()),
            data: self.data.clone(),
            timebounds: self.timebounds,
        }
    }
    //Use this function to create Output Memo in case of Trader/lend Order from Wasm
    pub fn new_from_wasm(
        script_address: String,
        owner_address: String,
        balance: u64,
        order_size: u64,
        scalar: Scalar,
    ) -> Self {
        //create ScalarWitness

        let commitment = crate::Commitment::blinded_with_factor(balance, scalar);
        // order size has to be in commitment
        let data_commitment = Commitment::blinded_with_factor(order_size, scalar);
        let data = Some(ZkvmString::Commitment(Box::new(data_commitment)));
        // create OutputMemo
        OutputMemo {
            script_address,
            owner: owner_address,
            commitment,
            data,
            timebounds: 0,
        }
    }
}
///Dummy values for testing Memo
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
    ///Contract Info Data (script Address/ Owner / Commitment value)
    //pub contract_info: Contract,
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
///Dummy value for state variable testing
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

/// A complete twilight typed Contract valid for a specific network.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Contract {
    // Script Address
    // pub script_address: String,
    // Owning address or predicate root.
    // pub owner: String,
    // Pedersen commitment on amount of coins.
    //  pub commitment: CompressedRistretto,
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

    pub const fn get_memo_data(&self) -> Option<&ZkvmString> {
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
    // pub const fn get_contract_info(&self) -> Option<&Contract> {
    //     match self {
    //         Self::Memo(memo) => Some(&memo.contract_info),
    //         Self::State(state) => Some(&state.contract_info),
    //         _ => None,
    //     }
    // }
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
            OutputData::Coin(ref out_coin) => Ok(out_coin.to_quisquis_account()),
            _ => Err("Error::InvalidOutputType. Only allowed for Coin type"),
        }
    }
}
impl From<Account> for Output {
    fn from(account: Account) -> Self {
        let (pk, comm) = account.get_account();
        let owner: String =
            address::Address::standard_address(address::Network::default(), pk).as_hex();
        let coin: OutputCoin = OutputCoin {
            encrypt: comm,
            owner,
        };
        Output::coin(OutputData::coin(coin))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Witness {
    // Signature in Script Tx
    Signature(Signature),
    // Signature ands Proof over state inputs ScriptTx
    State(StateWitness),
    // Signature and proof over Coin/Memo inputs for same value in ScriptTx
    ValueWitness(ValueWitness),
    // Zero balance proof for TransferTx
    Proof(SigmaProof),
}
use crate::VMError;
impl Witness {
    /// Downcasts Witness to `Signature` type.
    pub fn to_signature(self) -> Result<Signature, VMError> {
        match self {
            Witness::Signature(x) => Ok(x),
            _ => Err(VMError::TypeNotSignature),
        }
    }

    /// Downcasts Witness to `StateWitness` type.
    pub fn to_state_witness(self) -> Result<StateWitness, VMError> {
        match self {
            Witness::State(x) => Ok(x),
            _ => Err(VMError::TypeNotStateWitness),
        }
    }

    /// Downcasts Witness to `ValueWitness` type.
    /// This is used for the same value proof and signature. Used by CoinInput and MemoInput in script tx       
    pub fn to_value_witness(self) -> Result<ValueWitness, VMError> {
        match self {
            Witness::ValueWitness(x) => Ok(x),
            _ => Err(VMError::TypeNotValueWitness),
        }
    }
    // Downcasts Witness to `SigmaProof` type.
    pub fn to_sigma_proof(self) -> Result<SigmaProof, VMError> {
        match self {
            Witness::Proof(x) => Ok(x),
            _ => Err(VMError::TypeNotSigmaProof),
        }
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

//Upcast SigmaProof to Witness
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
        input: Input,
        secret_key: RistrettoSecretKey,

        pubkey: RistrettoPublicKey,
        zero_proof: Option<Vec<Scalar>>,
    ) -> Self {
        let in_state = input.input.clone();
        //create the Signature over the Input State with the secret key
        let commit: Commitment = Commitment::Closed(in_state.as_commitment().unwrap().to_point());
        // if in_state.as_state_variables().is_some(){
        let state_variables = in_state.as_state_variables().unwrap();
        let mut new_state_variables: Vec<ZkvmString> = Vec::new();

        for state in state_variables.iter() {
            match state {
                ZkvmString::Commitment(a) => {
                    let a_point = a.to_point();
                    let new_commitment = Commitment::Closed(a_point);
                    new_state_variables.push(ZkvmString::Commitment(Box::new(new_commitment)));
                }
                _ => new_state_variables.push(Clone::clone(&state)),
            }
        }
        //}
        // recreate the input state with the Verifier view values
        let out_state = OutputState {
            nonce: in_state.as_nonce().unwrap().clone(),

            script_address: in_state.as_script_address().unwrap().clone(),
            commitment: commit.clone(),
            owner: in_state.owner().unwrap().clone(),
            state_variables: Some(new_state_variables),
            timebounds: in_state.as_timebounds().unwrap().clone(),
        };

        //IGNORE Witness index at the time creating the signature
        let verifier_input = Input::state(InputData::state(
            input.as_utxo().unwrap().clone(),
            out_state,
            input.input.as_state_script_data().cloned(),
            0,
        ));
        //The sign must happen on the verifier View of the input so ghatg it can be verified acorrectly

        //create message bytes using input_state
        let message = bincode::serialize(&verifier_input).unwrap();

        let sign = pubkey.sign_msg(&message, &secret_key, ("StateSign").as_bytes());

        Self { sign, zero_proof }
    }

    pub fn verify_state_witness(
        &self,
        input: Input,
        pubkey: RistrettoPublicKey,
    ) -> Result<bool, &'static str> {
        //create message to verify the Signature over the Input State with the public key
        // The witness is 0 for the purposes of signature verification
        //recreate the input statewith the Witness as zero

        //verify the Signature over the Input state with the public key
        let verifier_input = Input::state(InputData::state(
            input.as_utxo().unwrap().clone(),
            input.as_out_state().unwrap().clone(),
            input.input.as_state_script_data().cloned(),
            0,
        ));
        let message = bincode::serialize(&verifier_input).unwrap();

        let verify_sig = pubkey.verify_msg(&message, &self.sign, ("StateSign").as_bytes());
        if verify_sig.is_err() {
            return Err("Signature verification failed");
        }
        let in_state = input.input;
        //get state_variables
        if in_state.as_state_variables().is_some() {
            //verify the zero proofs if any over the state variables
            let state_variables = in_state.as_state_variables().unwrap();
            let commitment_witness = self.zero_proof.as_ref().unwrap();
            if commitment_witness.len() > state_variables.len() {
                return Err("Error::There are more zero proofs than state variables");
            }
            //index is to go through the proofs against the committed variables
            let mut index: usize = 0;
            let gens = PedersenGens::default();
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
                            return Err("Error::The zero proof does not match the state variable");
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
//return iterator over zero proofs  (if any)
impl StateWitness {
    pub fn get_zero_proof(&self) -> Option<impl Iterator<Item = &Scalar>> {
        self.zero_proof.as_ref().map(|x| x.iter())
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

    pub fn create_value_witness(
        input: Input,
        secret_key: RistrettoSecretKey,
        enc_acc: Account,
        pubkey: RistrettoPublicKey,
        pedersen_commitment: CompressedRistretto,
        value: u64,
        rscalar: Scalar, //commitment scalar
    ) -> Self {
        //create the Signature over the Input Coin/Memo with the secret key
        //create message bytes using input
        //CONVERT INPUT TO VERIFIER VIEW
        let message = bincode::serialize(&input).unwrap();

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
        pubkey: RistrettoPublicKey,
        enc_acc: Account,
        commitment: CompressedRistretto,
    ) -> Result<bool, &'static str> {
        //create message to verify the Signature over the Input State with the public key
        let message = bincode::serialize(&input).unwrap();
        //verify the Signature over the InputData with the public key

        let verify_sig = pubkey.verify_msg(&message, &self.sign, ("ValueSign").as_bytes());
        if verify_sig.is_err() {
            return Err("Signature verification failed");
        }

        //verify the SigmaProof over the Input Coin/Memo with the public key
        let check = quisquislib::accounts::Verifier::verify_same_value_compact_verifier(
            enc_acc,
            commitment,
            self.value_proof.clone(),
        );
        if check.is_err() {
            return Err("Same Value SigmaProof verification failed");
        }
        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ZkosCreateOrder {
    pub input: Input,         //coin type input
    pub output: Output,       // memo type output
    pub signature: Signature, //quisquis signature
    pub proof: SigmaProof,
}
impl ZkosCreateOrder {
    pub fn new(input: Input, output: Output, vw: ValueWitness) -> Self {
        Self {
            input,
            output,
            signature: vw.get_signature().clone(),
            proof: vw.get_value_proof().clone(),
        }
    }
}
