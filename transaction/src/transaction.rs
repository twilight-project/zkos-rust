//! Core transaction types and structures for the ZkOS blockchain.
//!
//! This module defines the fundamental transaction types and data structures
//! used throughout the ZkOS transaction system, including transfer, script,
//! and message transactions.

use zkvm::zkos_types::{Input, Output};

use crate::{Message, ScriptTransaction, TransferTransaction};
use serde::{Deserialize, Serialize};

/// Transaction types supported by the ZkOS blockchain.
///
/// Each transaction type has different validation rules and processing
/// requirements. The default transaction type is `Transfer`.
///
/// # Example
/// ```
/// use transaction::TransactionType;
///
/// let transfer_type = TransactionType::Transfer;
/// let script_type = TransactionType::Script;
/// let message_type = TransactionType::Message;
///
/// // Convert from byte representation
/// let from_byte = TransactionType::from_u8(0).unwrap();
/// assert_eq!(from_byte, TransactionType::Transfer);
/// ```
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize, Default)]
pub enum TransactionType {
    /// Standard transfer transaction (default)
    #[default]
    Transfer,
    /// Smart contract script execution
    Script,
    /// Vault operations (reserved for future use)
    Vault,
    /// Message-based operations (burn, etc.)
    Message,
}

impl TransactionType {
    /// Converts a byte value to a transaction type.
    ///
    /// # Arguments
    /// * `byte` - The byte value representing the transaction type
    ///
    /// # Returns
    /// `Ok(TransactionType)` if the byte is valid, `Err` otherwise
    ///
    /// # Example
    /// ```
    /// use transaction::TransactionType;
    ///
    /// assert_eq!(TransactionType::from_u8(0).unwrap(), TransactionType::Transfer);
    /// assert_eq!(TransactionType::from_u8(1).unwrap(), TransactionType::Script);
    /// assert!(TransactionType::from_u8(255).is_err());
    /// ```
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

/// Transaction data payload for different transaction types.
///
/// This enum contains the specific data structures for each transaction type,
/// allowing type-safe handling of different transaction payloads.
///
/// # Example
/// ```
/// use transaction::{TransactionData, TransferTransaction, ScriptTransaction, Message};
/// use transaction::TransactionType;
///
/// // Transfer transaction data
/// let transfer_data = TransactionData::TransactionTransfer(TransferTransaction::default());
///
/// // Script transaction data
/// let script_data = TransactionData::TransactionScript(ScriptTransaction::default());
///
/// // Message transaction data
/// let message_data = TransactionData::Message(Message::default());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionData {
    /// Transfer transaction data
    TransactionTransfer(TransferTransaction),
    /// Script transaction data
    TransactionScript(ScriptTransaction),
    /// Message transaction data
    Message(Message),
    //TransactionCreate,
}

impl TransactionData {
    /// Attempts to downcast to a `TransferTransaction`.
    ///
    /// # Returns
    /// `Ok(TransferTransaction)` if the data is a transfer transaction, `Err` otherwise
    ///
    /// # Example
    /// ```
    /// use transaction::{TransactionData, TransferTransaction};
    ///
    /// let transfer_tx = TransferTransaction::default();
    /// let data = TransactionData::TransactionTransfer(transfer_tx.clone());
    /// assert_eq!(data.to_transfer().unwrap(), transfer_tx);
    /// ```
    pub fn to_transfer(self) -> Result<TransferTransaction, &'static str> {
        match self {
            TransactionData::TransactionTransfer(x) => Ok(x),
            _ => Err("Invalid Transfer Transaction"),
        }
    }

    /// Attempts to downcast to a `ScriptTransaction`.
    ///
    /// # Returns
    /// `Ok(ScriptTransaction)` if the data is a script transaction, `Err` otherwise
    pub fn to_script(self) -> Result<ScriptTransaction, &'static str> {
        match self {
            TransactionData::TransactionScript(x) => Ok(x),
            _ => Err("Invalid Script Transaction"),
        }
    }

    /// Attempts to downcast to a `Message`.
    ///
    /// # Returns
    /// `Ok(Message)` if the data is a message transaction, `Err` otherwise
    pub fn to_message(self) -> Result<Message, &'static str> {
        match self {
            TransactionData::Message(x) => Ok(x),
            _ => Err("Invalid Message Transaction"),
        }
    }
}

/// A complete ZkOS transaction valid for a specific network.
///
/// This is the main transaction structure that contains both the transaction
/// type and the corresponding data payload. All transactions must be verified
/// before being accepted by the network.
///
/// # Example
/// ```
/// use transaction::{Transaction, TransactionType, TransactionData, TransferTransaction};
///
/// let transfer_tx = TransferTransaction::default();
/// let transaction = Transaction::new(
///     TransactionType::Transfer,
///     TransactionData::TransactionTransfer(transfer_tx),
/// );
///
/// // Verify the transaction
/// assert!(transaction.verify().is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// The transaction type
    pub tx_type: TransactionType,
    /// The transaction data corresponding to the type
    pub tx: TransactionData,
}

impl Transaction {
    /// Creates a new transaction with the specified type and data.
    ///
    /// # Arguments
    /// * `tx_type` - The type of transaction
    /// * `tx` - The transaction data
    ///
    /// # Example
    /// ```
    /// use transaction::{Transaction, TransactionType, TransactionData, TransferTransaction};
    ///
    /// let transfer_tx = TransferTransaction::default();
    /// let transaction = Transaction::new(
    ///     TransactionType::Transfer,
    ///     TransactionData::TransactionTransfer(transfer_tx),
    /// );
    /// ```
    pub fn new(tx_type: TransactionType, tx: TransactionData) -> Transaction {
        Transaction { tx_type, tx }
    }

    /// Creates a transfer transaction with default type.
    ///
    /// # Arguments
    /// * `data` - The transfer transaction data
    ///
    /// # Example
    /// ```
    /// use transaction::{Transaction, TransactionData, TransferTransaction};
    ///
    /// let transfer_tx = TransferTransaction::default();
    /// let transaction = Transaction::transaction_transfer(
    ///     TransactionData::TransactionTransfer(transfer_tx),
    /// );
    /// ```
    pub fn transaction_transfer(data: TransactionData) -> Transaction {
        Transaction {
            tx_type: TransactionType::default(),
            tx: data,
        }
    }

    /// Creates a script transaction.
    ///
    /// # Arguments
    /// * `data` - The script transaction data
    pub fn transaction_script(data: TransactionData) -> Transaction {
        Transaction {
            tx_type: TransactionType::Script,
            tx: data,
        }
    }

    /// Creates a message transaction.
    ///
    /// # Arguments
    /// * `data` - The message transaction data
    pub fn transaction_message(data: TransactionData) -> Transaction {
        Transaction {
            tx_type: TransactionType::Message,
            tx: data,
        }
    }

    /// Returns the input values for this transaction.
    ///
    /// # Returns
    /// Vector of input values from the transaction
    ///
    /// # Example
    /// ```
    /// use transaction::{Transaction, TransactionType, TransactionData, TransferTransaction};
    ///
    /// let transfer_tx = TransferTransaction::default();
    /// let transaction = Transaction::new(
    ///     TransactionType::Transfer,
    ///     TransactionData::TransactionTransfer(transfer_tx),
    /// );
    ///
    /// let inputs = transaction.get_tx_inputs();
    /// ```
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

    /// Returns the output values for this transaction.
    ///
    /// # Returns
    /// Vector of output values from the transaction
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

    /// Returns the transaction fee.
    ///
    /// # Returns
    /// The fee amount for this transaction
    pub fn get_tx_fee(&self) -> u64 {
        match self.tx.clone() {
            TransactionData::TransactionTransfer(transfer_transaction) => transfer_transaction.fee,
            TransactionData::TransactionScript(script_transaction) => script_transaction.fee,
            TransactionData::Message(message) => message.fee,
        }
    }

    /// Verifies the transaction validity.
    ///
    /// This method delegates verification to the specific transaction type
    /// implementation, ensuring all cryptographic proofs and constraints
    /// are valid.
    ///
    /// # Returns
    /// `Ok(())` if verification succeeds, `Err` otherwise
    ///
    /// # Example
    /// ```
    /// use transaction::{Transaction, TransactionType, TransactionData, TransferTransaction};
    ///
    /// let transfer_tx = TransferTransaction::default();
    /// let transaction = Transaction::new(
    ///     TransactionType::Transfer,
    ///     TransactionData::TransactionTransfer(transfer_tx),
    /// );
    ///
    /// match transaction.verify() {
    ///     Ok(()) => println!("Transaction is valid"),
    ///     Err(e) => println!("Transaction verification failed: {}", e),
    /// }
    /// ```
    pub fn verify(&self) -> Result<(), &'static str> {
        match self.tx.clone() {
            TransactionData::TransactionTransfer(transfer_transaction) => {
                transfer_transaction.verify()
            }
            TransactionData::TransactionScript(script_transaction) => script_transaction.verify(),
            TransactionData::Message(message) => message.verify(),
        }
    }
}

// Conversion implementations for convenience

impl From<ScriptTransaction> for Transaction {
    fn from(tx_script: ScriptTransaction) -> Transaction {
        Transaction {
            tx_type: TransactionType::Script,
            tx: TransactionData::TransactionScript(tx_script),
        }
    }
}

impl From<TransferTransaction> for Transaction {
    fn from(tx_transfer: TransferTransaction) -> Transaction {
        Transaction {
            tx_type: TransactionType::Transfer,
            tx: TransactionData::TransactionTransfer(tx_transfer),
        }
    }
}

impl From<Message> for Transaction {
    fn from(message: Message) -> Transaction {
        Transaction {
            tx_type: TransactionType::Message,
            tx: TransactionData::Message(message),
        }
    }
}
