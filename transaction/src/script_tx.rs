// Copyright © 2025 Twilight Project Contributors
// SPDX-License-Identifier: Apache-2.0

//! Script transaction implementation for program execution on ZkOS.
//!
//! This module provides the core functionality for script transactions that execute
//! programs on the ZkOS blockchain. Script transactions support contract deployment,
//! contract calls, and general program execution with zero-knowledge proof verification.
//!
//! # Overview
//!
//! Script transactions enable:
//! - **Contract Deployment**: Deploy new smart contracts to the blockchain
//! - **Contract Calls**: Execute existing smart contracts with parameters
//! - **Program Execution**: Run arbitrary programs with R1CS proof verification
//! - **Witness Management**: Handle different types of input/output witnesses
//!
//! # Transaction Input/Output Types
//!
//! - **Coin → Memo**: Value transfers with commitment proofs
//! - **Memo → Coin**: Value reveals with signature proofs  
//! - **State → State**: Contract operations with state witnesses
//!
//! # Example
//! ```
//! use transaction::script_tx::ScriptTransaction;
//! use zkvm::{Program, IOType};
//! use quisquislib::ristretto::RistrettoSecretKey;
//!
//! // Create a script transaction for contract deployment
//! let script_tx = ScriptTransaction::create_script_transaction(
//!     &secret_keys,
//!     program,
//!     call_proof,
//!     &inputs,
//!     &outputs,
//!     Some(tx_data),
//!     true, // contract deploy
//!     100,  // fee
//! )?;
//!
//! // Verify the transaction
//! assert!(script_tx.verify().is_ok());
//! ```

use address::Address;
use quisquislib::{
    accounts::Account,
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
};
use serde::{Deserialize, Serialize};
use zkvm::{
    zkos_types::{Input, Output, StateWitness, Witness},
    Commitment, IOType, Program,
};

use bulletproofs::r1cs::R1CSProof;
use zkvm::merkle::CallProof;

/// Script transaction for program execution on ZkOS.
///
/// Script transactions enable the execution of programs on the blockchain
/// with full zero-knowledge proof verification. They support contract deployment,
/// contract calls, and general program execution with various input/output types.
///
/// # Structure
///
/// - **Header**: Version, fee, maturity, and vector lengths
/// - **IO**: Inputs and outputs with their respective types
/// - **Program**: Bytecode to be executed by the VM
/// - **Proofs**: R1CS proof and call proof for verification
/// - **Witnesses**: Cryptographic proofs for input validation
///
/// # Input/Output Types
///
/// - **Coin**: Value-bearing inputs with commitment proofs
/// - **Memo**: Value-revealing inputs with signature proofs
/// - **State**: Contract state inputs with state witnesses
///
/// # Example
/// ```
/// use transaction::script_tx::ScriptTransaction;
/// use zkvm::{Program, IOType};
///
/// let script_tx = ScriptTransaction::set_script_transaction(
///     0,      // version
///     100,    // fee
///     0,      // maturity
///     2,      // input_count
///     2,      // output_count
///     2,      // witness_count
///     inputs,
///     outputs,
///     program_bytes,
///     call_proof,
///     r1cs_proof,
///     witnesses,
///     Some(tx_data),
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptTransaction {
    // Transaction header
    pub(crate) version: u64,
    pub(crate) fee: u64,
    pub(crate) maturity: u64,

    // Lengths of vectors to come
    pub(crate) input_count: u8,
    pub(crate) output_count: u8,
    pub(crate) witness_count: u8,

    // List of inputs and outputs
    pub(crate) inputs: Vec<Input>,
    pub(crate) outputs: Vec<Output>,

    // Script program to be executed by the VM
    pub(crate) program: Vec<u8>,
    // Call proof for program Merkle tree inclusion
    pub(crate) call_proof: CallProof,

    // Script proof for computations in tx
    pub(crate) proof: R1CSProof,

    // Required for lit to dark case. Contains same value proof
    pub(crate) witness: Vec<Witness>,
    // Transaction data. e.g., supporting data needed for a script transaction at the top level.
    pub(crate) tx_data: Option<zkvm::String>,
}

impl ScriptTransaction {
    /// Creates a new script transaction with all components.
    ///
    /// # Arguments
    /// * `version` - Transaction version number
    /// * `fee` - Transaction fee in base units
    /// * `maturity` - Block height when transaction becomes valid
    /// * `input_count` - Number of inputs
    /// * `output_count` - Number of outputs
    /// * `witness_count` - Number of witnesses
    /// * `inputs` - Vector of transaction inputs
    /// * `outputs` - Vector of transaction outputs
    /// * `program` - Program bytecode to execute
    /// * `call_proof` - Merkle proof for program inclusion
    /// * `proof` - R1CS proof for program execution
    /// * `witness` - Vector of cryptographic witnesses
    /// * `tx_data` - Optional transaction data
    ///
    /// # Returns
    /// A new `ScriptTransaction` instance
    pub fn set_script_transaction(
        version: u64,
        fee: u64,
        maturity: u64,
        input_count: u8,
        output_count: u8,
        witness_count: u8,
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        program: Vec<u8>,
        call_proof: CallProof,
        proof: R1CSProof,
        witness: Vec<Witness>,
        tx_data: Option<zkvm::String>,
    ) -> Self {
        ScriptTransaction {
            version,
            fee,
            maturity,
            input_count,
            output_count,
            witness_count,
            inputs,
            outputs,
            program,
            call_proof,
            proof,
            witness,
            tx_data,
        }
    }

    /// Sets the outputs for this transaction.
    pub fn set_outputs(&mut self, outputs: Vec<Output>) {
        self.outputs = outputs;
    }

    /// Sets the inputs for this transaction.
    pub fn set_inputs(&mut self, inputs: Vec<Input>) {
        self.inputs = inputs;
    }

    /// Sets the transaction data.
    pub fn set_data(&mut self, data: Option<zkvm::String>) {
        self.tx_data = data;
    }

    /// Sets the transaction fee.
    pub fn set_fee(&mut self, fee: u64) {
        self.fee = fee;
    }

    /// Creates a dummy script transaction for UTXO set verification.
    ///
    /// This method creates a minimal transaction structure used only for
    /// verifying the UTXO set during block processing. It should not be
    /// used for actual transactions.
    ///
    /// # Arguments
    /// * `inputs` - Input references for verification
    /// * `outputs` - Output references for verification
    ///
    /// # Returns
    /// A dummy `ScriptTransaction` for UTXO verification
    pub fn create_utxo_dummy_script_transaction(
        inputs: &[Input],
        outputs: &[Output],
    ) -> ScriptTransaction {
        let program: Vec<u8> = vec![b'0'; 32];
        let witness_vec: Vec<Witness> = vec![];
        ScriptTransaction::set_script_transaction(
            0u64,
            0u64,
            0u64,
            inputs.len() as u8,
            outputs.len() as u8,
            0u8,
            inputs.to_vec(),
            outputs.to_vec(),
            program,
            CallProof::default(),
            R1CSProof::from_bytes(&[0u8; 32]).unwrap(),
            witness_vec,
            None,
        )
    }

    /// Creates a complete script transaction with proof generation.
    ///
    /// This method executes the program, generates R1CS proofs, creates witnesses,
    /// and constructs a verifiable script transaction. Use this when the prover
    /// holds private keys for all inputs.
    ///
    /// # Arguments
    /// * `sk_list` - Secret keys for all inputs
    /// * `prog` - Program to execute
    /// * `call_proof` - Merkle proof for program inclusion
    /// * `inputs` - Transaction inputs
    /// * `outputs` - Transaction outputs
    /// * `tx_data` - Optional transaction data
    /// * `contract_deploy_flag` - Whether this is a contract deployment
    /// * `fee` - Transaction fee
    ///
    /// # Returns
    /// A complete `ScriptTransaction` with all proofs and witnesses
    ///
    /// # Errors
    /// * `zkvm::VMError` if program execution or proof generation fails
    ///
    /// # Note
    /// For manual construction, follow these steps:
    /// 1. Create inputs and outputs
    /// 2. Create program proof
    /// 3. Create call proof
    /// 4. Create value and state witnesses
    /// 5. Create script transaction
    pub fn create_script_transaction(
        sk_list: &[RistrettoSecretKey],
        prog: Program,
        call_proof: CallProof,
        inputs: &[Input],
        outputs: &[Output],
        tx_data: Option<zkvm::String>,
        contract_deploy_flag: bool,
        fee: u64,
    ) -> Result<ScriptTransaction, zkvm::VMError> {
        // Execute the program and create a proof
        let (program, proof) = crate::vm_run::Prover::build_proof(
            prog,
            inputs,
            outputs,
            contract_deploy_flag,
            tx_data.clone(),
        )?;

        // Create signatures and witness proofs for all inputs and corresponding outputs
        let witness: Vec<Witness> = ScriptTransaction::create_witness_for_script_tx(
            sk_list,
            inputs,
            outputs,
            contract_deploy_flag,
        );

        // Convert inputs and outputs to hide encrypted data using verifier view
        let (inputs, outputs, tx_data) =
            ScriptTransaction::create_verifier_view(inputs, outputs, tx_data);

        Ok(ScriptTransaction::set_script_transaction(
            0u64,
            fee,
            0u64,
            inputs.len() as u8,
            outputs.len() as u8,
            witness.len() as u8,
            inputs,
            outputs,
            program,
            call_proof,
            proof,
            witness,
            tx_data,
        ))
    }

    /// Creates verifier view for transaction encoding.
    ///
    /// This method converts inputs and outputs to their verifier view representation,
    /// hiding encrypted data and updating witness indices. This should be replaced
    /// with a proper encoding function in the future.
    ///
    /// # Arguments
    /// * `inputs` - Original transaction inputs
    /// * `outputs` - Original transaction outputs
    /// * `tx_data` - Original transaction data
    ///
    /// # Returns
    /// Tuple of (verifier_inputs, verifier_outputs, verifier_tx_data)
    pub fn create_verifier_view(
        inputs: &[Input],
        outputs: &[Output],
        tx_data: Option<zkvm::String>,
    ) -> (Vec<Input>, Vec<Output>, Option<zkvm::String>) {
        // Iterate over inputs and create verifier view for each input
        let mut input_vec: Vec<Input> = Vec::with_capacity(inputs.len());
        let mut output_vec: Vec<Output> = Vec::with_capacity(outputs.len());

        for (i, inp) in inputs.iter().enumerate() {
            let mut verifier_input = inp.verifier_view();
            verifier_input.replace_witness_index(i as u8);
            input_vec.push(verifier_input);
            output_vec.push(outputs[i].to_verifier_view());
        }

        // Check if tx_data needs encryption
        let verifier_tx_data = match tx_data {
            Some(data) => {
                // Check if tx_data is a commitment
                match data {
                    zkvm::String::Commitment(commit) => Some(zkvm::String::Commitment(Box::new(
                        Commitment::Closed(commit.to_point()),
                    ))),
                    _ => Some(data),
                }
            }
            None => None,
        };

        (input_vec, output_vec, verifier_tx_data)
    }

    /// Creates signatures and zero balance proofs for all inputs and outputs.
    ///
    /// This method generates the appropriate witnesses for each input type:
    /// - **Coin inputs**: Value witnesses with commitment proofs
    /// - **Memo inputs**: Signature witnesses for value reveals
    /// - **State inputs**: State witnesses for contract operations
    ///
    /// # Arguments
    /// * `sk_list` - Secret keys for all inputs
    /// * `inputs` - Transaction inputs
    /// * `outputs` - Transaction outputs
    /// * `contract_deploy_flag` - Whether this is a contract deployment
    ///
    /// # Returns
    /// Vector of witnesses for all inputs
    pub fn create_witness_for_script_tx(
        sk_list: &[RistrettoSecretKey],
        inputs: &[Input],
        outputs: &[Output],
        contract_deploy_flag: bool,
    ) -> Vec<Witness> {
        let mut witness: Vec<Witness> = Vec::with_capacity(inputs.len());

        // Iterate over inputs and build corresponding witnesses
        // Coin <-> Memo always carry ValueWitness
        // State <-> State
        //  1. Deploy Contract: State -> StateWitness
        //  2. Call Contract: Signature -> SignatureWitness
        for (i, inp) in inputs.iter().enumerate() {
            match inp.in_type {
                IOType::Coin => {
                    // Get corresponding OutputMemo
                    let out_memo: Output = outputs[i].clone();
                    let acc: Account = inp
                        .to_quisquis_account()
                        .expect("Input is not a quisquis account");

                    // Get the public key from account
                    let (pk, _) = acc.get_account();

                    // Get Pedersen commitment value from Memo
                    let memo_commitment = out_memo
                        .output
                        .get_commitment()
                        .expect("Memo is not a coin");

                    // Get commitment value and scalar
                    let (memo_value, memo_scalar) = memo_commitment.witness().unwrap();
                    let memo_commit = memo_commitment.to_point();
                    let value = memo_value
                        .to_integer()
                        .expect("Cannot convert to signed int")
                        .to_u64()
                        .expect("Value is not a u64");

                    // Create coin input witness
                    let input_coin = inp.clone();
                    let sk = sk_list[i];
                    let coin_witness = zkvm::zkos_types::ValueWitness::create_value_witness(
                        input_coin,
                        sk,
                        acc,
                        pk,
                        memo_commit,
                        value,
                        memo_scalar,
                    );
                    witness.push(Witness::ValueWitness(coin_witness));
                }
                IOType::Memo => {
                    // Get corresponding OutputCoin
                    let out_coin: Output = outputs[i].clone();

                    let memo_witness = zkvm::zkos_types::Witness::create_witness_for_memo_input(
                        out_coin,
                        inp.clone(),
                    )
                    .expect("Memo Witness cannot be created");
                    witness.push(memo_witness);
                }
                IOType::State => {
                    // Get the input
                    let input = inp.clone();
                    let sk = sk_list[i];
                    let output = outputs[i].clone();
                    let owner = input
                        .as_owner_address()
                        .expect("Owner address does not exist");

                    // Extract pk from owner string
                    let address: Address = Address::from_hex(owner, address::AddressType::Standard)
                        .expect("Hex address is not decodable");
                    let pk: RistrettoPublicKey = address.into();

                    let state_witness = StateWitness::create_state_witness(
                        &input,
                        &output,
                        sk,
                        pk,
                        contract_deploy_flag,
                    );
                    witness.push(Witness::State(state_witness));
                }
            }
        }
        witness
    }

    /// Verifies the complete script transaction.
    ///
    /// This method performs comprehensive verification of all transaction components:
    /// - Witness verification for all inputs
    /// - Call proof verification for program authenticity
    /// - R1CS proof verification for program execution
    ///
    /// # Returns
    /// `Ok(())` if verification succeeds, `Err` otherwise
    ///
    /// # Errors
    /// * Various verification errors if any component fails
    pub fn verify(&self) -> Result<(), &'static str> {
        // Assume that the UTXO IDs have been verified already

        // Differentiate between contract deploy and contract call
        let contract_initialize = self.is_contract_deploy();

        // Verify the witnesses and proofs of same value and zero balance proof as required
        self.verify_witnesses(contract_initialize)?;

        // Verify the call proof for the program to check authenticity
        // Checking authenticity is not required for contract deploy
        self.verify_call_proof()?;

        // Verify the R1CS proof
        let verify = crate::vm_run::Verifier::verify_r1cs_proof(
            &self.proof,
            &self.program,
            &self.inputs,
            &self.outputs,
            contract_initialize,
            self.tx_data.clone(),
        );

        match verify {
            Ok(_x) => Ok(()),
            Err(_e) => Err("R1CS Proof Verification Failed"),
        }
    }

    /// Verifies the call proof for the transaction program.
    ///
    /// This method verifies that the program is authentic by checking its
    /// inclusion in the Merkle tree. A single transaction can only have
    /// one program and interact with a single state.
    ///
    /// # Returns
    /// `Ok(())` if verification succeeds, `Err` otherwise
    ///
    /// # Errors
    /// * Various errors if program parsing or proof verification fails
    pub fn verify_call_proof(&self) -> Result<(), &'static str> {
        // Verify the call proof for program authenticity
        let hasher: zkvm::Hasher<Program> = zkvm::Hasher::<Program>::new(b"ZkOS.MerkelTree");
        let bytecode = self.program.clone();

        // Recreate ProgramItem from Vec[u8]
        let prog = match Program::parse(&bytecode) {
            Ok(prog) => prog,
            Err(_e) => {
                return Err("Program is not a valid bytecode");
            }
        };

        // Get the script address from inputs and outputs
        // The first input will always be a Coin or a Memo
        let inp = self.inputs[0].clone();
        let mut script_address = String::new();

        if inp.in_type == IOType::Coin {
            // Get corresponding OutputMemo
            let out_memo = match self.outputs[0].as_out_memo() {
                Some(out_memo) => out_memo,
                None => {
                    return Err("First Output is not a Memo");
                }
            };
            script_address = out_memo.script_address.clone();
        }

        if inp.in_type == IOType::Memo {
            script_address = match inp.as_script_address() {
                Some(addr) => addr.to_owned(),
                None => {
                    return Err("Script Address does not exist");
                }
            }
        }

        if inp.in_type == IOType::State {
            return Err("First Input is not a Coin or a Memo");
        }

        // Verify the call proof
        let verify_call_proof = self
            .call_proof
            .verify_call_proof(script_address, &prog, &hasher);

        match verify_call_proof {
            true => Ok(()),
            false => Err("Call Proof Verification Failed"),
        }
    }

    /// Checks if the script is deploying a contract.
    ///
    /// This method determines whether the transaction is a contract deployment
    /// by checking if any input is of type State and has a zero balance proof.
    /// This is more efficient than checking UTXO existence.
    ///
    /// # Returns
    /// `true` if this is a contract deployment, `false` otherwise
    pub fn is_contract_deploy(&self) -> bool {
        // Loop over inputs and find if any input is of type state
        for inp in self.inputs.iter() {
            if inp.in_type == zkvm::IOType::State {
                let wit = self.witness.clone();
                // Check if witness is of type ZeroBalanceProof
                let witness = wit[inp.get_witness_index() as usize].clone();
                let state_witness = witness.to_state_witness().unwrap();

                // Check if state witness is carrying a zero balance proof
                let zero_proof = state_witness.get_zero_proof();
                match zero_proof {
                    Some(_x) => {
                        return true;
                    }
                    None => {
                        return false;
                    }
                }
            }
        }
        false
    }

    /// Verifies all witnesses and their associated proofs.
    ///
    /// This method verifies witnesses for all input types:
    /// - **Coin inputs**: Value witness verification with commitment proofs
    /// - **Memo inputs**: Signature witness verification for value reveals
    /// - **State inputs**: State witness verification for contract operations
    ///
    /// # Arguments
    /// * `contract_deploy_flag` - Whether this is a contract deployment
    ///
    /// # Returns
    /// `Ok(())` if all witnesses verify successfully, `Err` otherwise
    ///
    /// # Errors
    /// * Various verification errors if any witness fails
    pub fn verify_witnesses(&self, contract_deploy_flag: bool) -> Result<(), &'static str> {
        // Get the witness vector
        let witness_vector: Vec<Witness> = self.witness.clone();

        // Loop over inputs and extract their corresponding witnesses
        for (i, inp) in self.inputs.iter().enumerate() {
            match inp.in_type {
                IOType::Coin => {
                    // Get corresponding OutputMemo
                    let out_memo: Output = self.outputs[i].clone();

                    // Get coin input witness
                    let coin_witness: zkvm::zkos_types::ValueWitness = witness_vector
                        [inp.get_witness_index() as usize]
                        .clone()
                        .to_value_witness()
                        .map_err(|_| "Invalid ValueWitness for Input")?;

                    // Verify the witness
                    let acc: Account = inp.to_quisquis_account()?;
                    let (pk, _) = acc.get_account();
                    let memo_value = out_memo.output.get_commitment();

                    let memo_value = match memo_value {
                        Some(memo) => memo,
                        None => {
                            return Err("VerificationError::MemoCommitment does not exist");
                        }
                    };

                    let witness_verify = coin_witness.verify_value_witness(
                        inp.clone(),
                        pk,
                        acc,
                        memo_value.to_point(),
                    );

                    match witness_verify {
                        Ok(_x) => {}
                        Err(_e) => {
                            return Err("Value Witness Verification Failed");
                        }
                    }
                }
                IOType::Memo => {
                    // Get corresponding OutputCoin
                    let out_coin: Output = self.outputs[i].clone();

                    // Get memo input witness
                    let memo_witness = witness_vector[inp.get_witness_index() as usize].clone();

                    if !memo_witness.verify_witness_for_memo_input(out_coin.clone(), inp.clone())? {
                        return Err("Value Witness Verification Failed");
                    }
                }
                IOType::State => {
                    // Get the witness for the input
                    let state_witness = witness_vector[inp.get_witness_index() as usize]
                        .clone()
                        .to_state_witness()
                        .map_err(|_| "VerificationError::Invalid StateWitness")?;

                    let owner_address_str = inp.as_owner_address();
                    // Extract pk from owner string
                    let owner_address: Address = match owner_address_str {
                        Some(owner) => Address::from_hex(owner, address::AddressType::Standard)?,
                        None => {
                            return Err("Owner address does not exist");
                        }
                    };
                    let pk: RistrettoPublicKey = owner_address.into();

                    // Verify the witness
                    if !state_witness.verify_state_witness(
                        inp.clone(),
                        self.outputs[i].clone(),
                        pk,
                        contract_deploy_flag,
                    )? {
                        return Err("State Witness Verification Failed");
                    };
                }
            }
        }
        Ok(())
    }

    /// Returns the input values for UTXO-in-memory compatibility.
    pub fn get_input_values(&self) -> Vec<Input> {
        self.inputs.clone()
    }

    /// Returns the output values for UTXO-in-memory compatibility.
    pub fn get_output_values(&self) -> Vec<Output> {
        self.outputs.clone()
    }

    /// Returns the transaction data.
    pub fn get_tx_data(&self) -> Option<zkvm::String> {
        self.tx_data.clone()
    }
}
