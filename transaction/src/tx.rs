use bulletproofs::r1cs;
use bulletproofs::r1cs::R1CSProof;
use bulletproofs::PedersenGens;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use quisquis-rust::Transaction;
/// Transaction type: Transfer. Transition, Create, Vault
///
/// TransactionType implements [`Default`] and returns [`TransactionType::Transfer`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum TransactionType {
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

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Transaction {
    TransactionTransfer,
    TransactionTransition,
    TransactionCreate,
    TransactionVault,
}



/*pub(crate) version: u64,
    pub(crate) byte_price: u64,
    pub(crate) price_limit: u64,
    pub(crate) maturity: u64,
    pub(crate) input_count: u8,
    pub(crate) output_count: u8,
    pub(crate) input_account_vector: Vec<Account>,
    pub(crate) output_account_vector: Vec<Account>,
    //non shuffle proof
    pub(crate) proof: TransferProof,
    //input and output shuffle proof
    pub(crate) shuffle_proof: UnifiedShuffleProof,
    //required for lit to dark case. contains same value proof
    pub(crate) witness: Option<WitnessProof>,
    // pub(crate) updated_account_vector: Vec<Account>,
    // pub(crate) account_delta_vector: Vec<Account>,
    // pub(crate) account_epsilon_vector: Vec<Account>,
    //  pub(crate) account_updated_delta_vector: Vec<Account>,
    //  pub(crate) output_account_vector: Vec<Account>,
*/