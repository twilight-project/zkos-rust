//use merlin::Transcript;
use zkvm::zkos_types::{Input, Output};

use crate::{Message, ScriptTransaction, TransferTransaction};
use serde::{Deserialize, Serialize};

/// Transaction type: Transfer. Script, Vault
/// TransactionType implements [`Default`] and returns [`TransactionType::Transfer`].
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    Script,
    Vault,
    Message,
}

impl TransactionType {
    pub fn from_u8(byte: u8) -> Result<TransactionType, &'static str> {
        use TransactionType::*;
        match byte {
            0 => Ok(Transfer),
            1 => Ok(Script),
            2 => Ok(Vault),
            3 => Ok(Message),
            _ => Err("Error::InvalidTransactionType"),
        }
    }
}
impl Default for TransactionType {
    fn default() -> TransactionType {
        TransactionType::Transfer
    }
}
/// Transaction data: Transfer, Script, Vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionData {
    TransactionTransfer(TransferTransaction),
    TransactionScript(ScriptTransaction),
    //TransactionCreate,
    Message(Message),
}

impl TransactionData {
    /// Downcasts Transaction to `Transfer` type.
    pub fn to_transfer(self) -> Result<TransferTransaction, &'static str> {
        match self {
            TransactionData::TransactionTransfer(x) => Ok(x),
            _ => Err("Invalid Transfer Transaction"),
        }
    }

    /// Downcasts Transaction to `Script` type.
    pub fn to_script(self) -> Result<ScriptTransaction, &'static str> {
        match self {
            TransactionData::TransactionScript(x) => Ok(x),
            _ => Err("Invalid Script Transaction"),
        }
    }
    /// Downcasts Transaction to `Message` type.
    pub fn to_message(self) -> Result<Message, &'static str> {
        match self {
            TransactionData::Message(x) => Ok(x),
            _ => Err("Invalid Message Transaction"),
        }
    }
}

/// A complete twilight Transactiont valid for a specific network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Defines the Tx type.
    pub tx_type: TransactionType,
    /// The Tx data corresponding to the Tx type.
    pub tx: TransactionData,
}

impl Transaction {
    /// set a new transaction
    pub fn new(tx_type: TransactionType, tx: TransactionData) -> Transaction {
        Transaction { tx_type, tx }
    }

    /// Create a transfer tx .
    pub fn transaction_transfer(data: TransactionData) -> Transaction {
        Transaction {
            tx_type: TransactionType::default(),
            tx: data,
        }
    }
    /// Create a Script tx .
    pub fn transaction_script(data: TransactionData) -> Transaction {
        Transaction {
            tx_type: TransactionType::Script,
            tx: data,
        }
    }
    /// Create a Message tx .
    pub fn transaction_message(data: TransactionData) -> Transaction {
        Transaction {
            tx_type: TransactionType::Message,
            tx: data,
        }
    }

    /// return tx Input values
    pub fn get_tx_inputs(&self) -> Vec<Input> {
        match self.tx.clone() {
            TransactionData::TransactionTransfer(transfer_transaction) => {
                transfer_transaction.get_input_values().clone()
            }
            TransactionData::TransactionScript(script_transaction) => {
                script_transaction.get_input_values().clone()
            }
            TransactionData::Message(message) => vec![message.input.clone()],
        }
    }
    /// return tx Output values
    pub fn get_tx_outputs(&self) -> Vec<Output> {
        match self.tx.clone() {
            TransactionData::TransactionTransfer(transfer_transaction) => {
                transfer_transaction.get_output_values()
            }
            TransactionData::TransactionScript(script_transaction) => {
                script_transaction.get_output_values()
            }
            _ => vec![],
        }
    }
    pub fn verify(&self) -> Result<(), &'static str> {
        match self.tx.clone() {
            TransactionData::TransactionTransfer(transfer_transaction) => {
                transfer_transaction.verify()
            }
            TransactionData::TransactionScript(script_transaction) => script_transaction.verify(
                self.get_tx_inputs().as_slice(),
                self.get_tx_outputs().as_slice(),
            ),
            TransactionData::Message(message) => message.verify(),
        }
    }
}

impl From<ScriptTransaction> for Transaction {
    fn from(tx_script: ScriptTransaction) -> Transaction {
        Transaction {
            tx_type: TransactionType::Script,
            tx: TransactionData::TransactionScript(tx_script),
        }
    }
}
/// from transfer transaction to transaction
impl From<TransferTransaction> for Transaction {
    fn from(tx_transfer: TransferTransaction) -> Transaction {
        Transaction {
            tx_type: TransactionType::Transfer,
            tx: TransactionData::TransactionTransfer(tx_transfer),
        }
    }
}

/// from message to transaction
impl From<Message> for Transaction {
    fn from(message: Message) -> Transaction {
        Transaction {
            tx_type: TransactionType::Message,
            tx: TransactionData::Message(message),
        }
    }
}
