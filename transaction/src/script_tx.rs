use std::borrow::Borrow;

use address::{Address, Network};
use merlin::Transcript;
use quisquislib::{
    accounts::Account,
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
};
//use quisquislib::{keys::PublicKey, ristretto::RistrettoSecretKey};
use serde::{Deserialize, Serialize};
use zkvm::{
    zkos_types::{Input, Output, OutputCoin, OutputMemo, StateWitness, ValueWitness, Witness}, // OutputCoin, Utxo},
    Commitment,
    IOType,
    Program,
};

use bulletproofs::r1cs::R1CSProof;
// use bulletproofs::BulletproofGens;
use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;

// use bincode;
// use std::fmt;
use zkschnorr::{Signature, VerificationKey};
use zkvm::merkle::CallProof; //, Hash, MerkleItem, MerkleTree};

///
/// Store for TransactionScript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptTransaction {
    //transaction header
    pub(crate) version: u64,
    pub(crate) fee: u64,
    pub(crate) maturity: u64,

    //lengths of vectors to come
    pub(crate) input_count: u8,
    pub(crate) output_count: u8,
    pub(crate) witness_count: u8,

    //List of inputs and outputs
    pub(crate) inputs: Vec<Input>,
    pub(crate) outputs: Vec<Output>,

    //Script program to be executed by the VM
    pub(crate) program: Vec<u8>,
    //Call Proof for program Merkle tree inclusion
    pub(crate) call_proof: CallProof,

    //Script proof for computations in tx
    pub(crate) proof: R1CSProof,

    //required for lit to dark case. contains same value proof
    pub(crate) witness: Vec<Witness>,
    // Transaction data. e.g., supporting data needed for a script transaction at the top level.
    pub(crate) tx_data: Option<zkvm::String>,
}

impl ScriptTransaction {
    /// Set a script transaction
    ///
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
    ///DUMMY TX FOR UTXO SET VERIFICATION
    /// Done only for verifying the utxo set in block processing module
    ///
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
    /// create a script transaction
    /// run the program and create a proof and Witnesses for all inputs and corresponding outputs
    /// convert inputs and outputs to hide the encrypted data using verifier view
    /// adjust the witness index of each input to match the input index in the transaction
    pub fn create_script_transaction(
        sk_list: &[RistrettoSecretKey],
        prog: Program,
        call_proof: CallProof,
        inputs: &[Input],
        outputs: &[Output],
        tx_data: Option<zkvm::String>,
        contract_deploy_flag: bool,
    ) -> Result<ScriptTransaction, zkvm::VMError> {
        // execute the program and create a proof
        let (program, proof) = crate::vm_run::Prover::build_proof(
            prog,
            &inputs,
            &outputs,
            contract_deploy_flag,
            tx_data.clone(),
        )?;

        // create signatures and witness proofs for all inputs and corresponding outputs
        let witness: Vec<Witness> = ScriptTransaction::create_witness_for_script_tx(
            sk_list,
            inputs,
            outputs,
            contract_deploy_flag,
        );
        // converts inputs and outputs to hide the encrypted data using verifier view and update witness index
        let (inputs, outputs, tx_data) =
            ScriptTransaction::create_verifier_view(inputs, outputs, tx_data);
        Ok(ScriptTransaction::set_script_transaction(
            0u64,
            0u64,
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
    // create verifier view for the transaction
    // Should be replace with Encoding function for Tx which should do this automatically
    // Currrently bincode is used for encoding which shall be replaced with a custom encoding function
    fn create_verifier_view(
        inputs: &[Input],
        outputs: &[Output],
        tx_data: Option<zkvm::String>,
    ) -> (Vec<Input>, Vec<Output>, Option<zkvm::String>) {
        //iterate over inputs and create verifier view for each input
        let mut input_vec: Vec<Input> = Vec::with_capacity(inputs.len());
        let mut output_vec: Vec<Output> = Vec::with_capacity(outputs.len());
        for (i, inp) in inputs.iter().enumerate() {
            let mut verifier_input = inp.verifier_view();
            verifier_input.replace_witness_index(i as u8);
            input_vec.push(verifier_input);
            output_vec.push(outputs[i].to_verifier_view());
        }

        // check if tx_ data needs encyption
        let verifier_tx_data = match tx_data {
            Some(data) => {
                // check if tx_data is a commitment
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

    ///create signatures and zero balance proofs for all inputs and corresponding outputs
    pub fn create_witness_for_script_tx(
        sk_list: &[RistrettoSecretKey],
        inputs: &[Input],
        outputs: &[Output],
        contract_deploy_flag: bool,
    ) -> Vec<Witness> {
        let mut witness: Vec<Witness> = Vec::with_capacity(inputs.len());

        //iterate over Inputs and build the corresponding witness
        // Coin <-> Memo always carry ValueWitness
        // State <-> State
        //  1. Deploy Contract: State -> StateWitness
        //  2. Call Contract: Signature -> SignatureWitness
        for (i, inp) in inputs.iter().enumerate() {
            match inp.in_type {
                IOType::Coin => {
                    // get corresponding OutputMemo
                    let out_memo: Output = outputs[i].clone();
                    let acc: Account = inp
                        .to_quisquis_account()
                        .expect("Input is not a quisquis account");
                    // get the public key from account
                    let (pk, _) = acc.get_account();
                    // get Pedersen commitment value from Memo
                    let memo_commitment = out_memo
                        .output
                        .get_commitment()
                        .expect("Memo is not a coin");
                    // get commitment value and scalar
                    let (memo_value, memo_scalar) = memo_commitment.witness().unwrap();
                    let memo_commit = memo_commitment.to_point();
                    let value = memo_value
                        .to_integer()
                        .expect("Can not cconvert to signed int")
                        .to_u64()
                        .expect("Value is not a u64");
                    // create coin input witness
                    let input_coin = inp.clone();
                    let sk = sk_list[i].clone();
                    let coin_witness = zkvm::zkos_types::ValueWitness::create_value_witness(
                        input_coin,
                        sk,
                        out_memo,
                        acc,
                        pk,
                        memo_commit,
                        value,
                        memo_scalar,
                    );
                    witness.push(Witness::ValueWitness(coin_witness));
                }
                IOType::Memo => {
                    let in_memo: &OutputMemo = inp
                        .as_out_memo()
                        .expect("OutputMemo can not be extracted from Input Memo");
                    // get corresponding OutputCoin
                    let out_coin: Output = outputs[i].clone();
                    let acc: Account = out_coin
                        .to_quisquis_account()
                        .expect("Output is not a quisquis account");
                    // get the public key from account
                    let (pk, _) = acc.get_account();
                    // get Pedersen commitment from Memo for same value proof. this value is coming from coin value
                    let memo_commitment = in_memo.commitment.clone();
                    // get commitment value and scalar
                    let (memo_value, memo_scalar) = memo_commitment.witness().unwrap();
                    let memo_commit = memo_commitment.to_point();
                    let value = memo_value
                        .to_integer()
                        .expect("Can not cconvert to signed int")
                        .to_u64()
                        .expect("Value is not a u64");

                    let sk = sk_list[i].clone();
                    let memo_witness = zkvm::zkos_types::ValueWitness::create_value_witness(
                        inp.clone(),
                        sk,
                        out_coin,
                        acc,
                        pk,
                        memo_commit,
                        value,
                        memo_scalar,
                    );
                    witness.push(Witness::ValueWitness(memo_witness));
                }
                IOType::State => {
                    // get the input
                    let input = inp.clone();
                    let sk = sk_list[i].clone();
                    let output = outputs[i].clone();
                    let owner = input
                        .as_owner_address()
                        .expect("Owner address does not exist");
                    // extract pk from owner string
                    let address: Address = Address::from_hex(owner, address::AddressType::Standard)
                        .expect("Hex address is not decodable");
                    let pk: RistrettoPublicKey = address.into();
                    let state_witness = StateWitness::create_state_witness(
                        &input,
                        &output,
                        sk.clone(),
                        pk.clone(),
                        contract_deploy_flag,
                    );
                    witness.push(Witness::State(state_witness));
                }
            }
        }
        witness
    }

    /// verify the script tx
    pub fn verify(&self) -> Result<(), &'static str> {
        //assume that the Utxo Ids have been verified already

        // Differentiate between contract deploy and contract call
        let contract_initialize = self.is_contract_deploy();

        //verify the witnesses and the proofs of same value and zero balance proof as required
        self.verify_witnesses(contract_initialize)?;

        // verify the call proof for the program to check the authenticity of the program
        // Checking authenticity of the program is not required for contract deploy

        self.verify_call_proof()?;

        // verify the r1cs proof

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
    // verify call proof for the tx program
    // assuming a single tx can only have one program and can only interact with single state
    pub fn verify_call_proof(&self) -> Result<(), &'static str> {
        // verify the call proof for the program to check the authenticity of the program
        let hasher: zkvm::Hasher<Program> = zkvm::Hasher::<Program>::new(b"ZkOS.MerkelTree");
        let bytecode = self.program.clone();
        // recreate ProgramItem from Vec[u8]
        let prog = match Program::parse(&bytecode) {
            Ok(prog) => prog,
            Err(_e) => {
                return Err("Program is not a valid bytecode");
            }
        };
        // get the script address from Inputs and Outputs
        // the first input will always be a Coin or a Memo
        let inp = self.inputs[0].clone();
        let mut script_address = String::new();
        if inp.in_type == IOType::Coin {
            // get corresponding OutputMemo
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
        // verify the call proof
        let verify_call_proof = self
            .call_proof
            .verify_call_proof(script_address, &prog, &hasher);
        match verify_call_proof {
            true => Ok(()),
            false => Err("Call Proof Verification Failed"),
        }
    }
    // check if script is deploying contract
    // can also use Utxo existance to check this but this is more efficient
    pub fn is_contract_deploy(&self) -> bool {
        // loop over inputs and find if any input is of type state
        for inp in self.inputs.iter() {
            if inp.in_type == zkvm::IOType::State {
                // get the witness for the input
                //  match &self.witness {
                //      Some(wit) => {
                let wit = self.witness.clone();
                // check if witness is of type ZeroBalanceProof
                // get the witness from index
                let witness = wit[inp.get_witness_index() as usize].clone();
                let state_witness = witness.to_state_witness().unwrap();
                // check if state witness is carrying a zero balance proof
                let zero_proof = state_witness.get_zero_proof();
                match zero_proof {
                    Some(_x) => {
                        return true;
                    }
                    None => {
                        return false;
                    }
                }
                //        }
                //        None => {
                //            return false;
                //        }
                // }
            }
        }
        false
    }

    // verify the witnesses and the proofs of same value and zero balance proof as required
    pub fn verify_witnesses(&self, contract_deploy_flag: bool) -> Result<(), &'static str> {
        // get the witness vector
        let witness_vector: Vec<Witness> = self.witness.clone();
        // loop over inputs and extract their corresponding witnesses
        for (i, inp) in self.inputs.iter().enumerate() {
            match inp.in_type {
                IOType::Coin => {
                    // get corresponding OutputMemo
                    let out_memo: Output = self.outputs[i].clone();
                    // get coin input witness
                    let coin_witness: zkvm::zkos_types::ValueWitness = witness_vector
                        [inp.get_witness_index() as usize]
                        .clone()
                        .to_value_witness()
                        .map_err(|_| "Invalid ValueWitness for Input")?;

                    // verify the witness
                    // get account from input
                    let acc: Account = inp.to_quisquis_account()?;
                    // get the public key from account
                    let (pk, _) = acc.get_account();
                    // get Pedersen commitment value from Memo
                    let memo_value = out_memo.output.get_commitment();

                    let memo_value = match memo_value {
                        Some(memo) => memo,
                        None => {
                            return Err("VerificationError::MemoComitment does not exist");
                        }
                    };
                    let witness_verify = coin_witness.verify_value_witness(
                        inp.clone(),
                        out_memo.clone(),
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
                    // get corresponding OutputCoin
                    let out_coin: Output = self.outputs[i].clone();

                    // get memo input witness
                    let memo_witness: zkvm::zkos_types::ValueWitness = witness_vector
                        [inp.get_witness_index() as usize]
                        .clone()
                        .to_value_witness()
                        .map_err(|_| "VerificationError::Invalid ValueWitness for Input")?;

                    // verify the witness
                    // get account from output
                    let acc: Account = out_coin.to_quisquis_account()?;
                    // get public key from input
                    let (pk, _) = acc.get_account();
                    // get pedersen commitment value from input
                    let memo_value = inp.as_commitment();
                    let memo_value = match memo_value {
                        Some(memo) => memo,
                        None => {
                            return Err("VerificationError::MemoComitment does not exist");
                        }
                    };
                    if !memo_witness.verify_value_witness(
                        inp.clone(),
                        out_coin.clone(),
                        pk,
                        acc,
                        memo_value.to_point(),
                    )? {
                        return Err("Value Witness Verification Failed");
                    }
                }
                IOType::State => {
                    // get the witness for the input
                    let state_witness = witness_vector[inp.get_witness_index() as usize]
                        .clone()
                        .to_state_witness()
                        .map_err(|_| "VerificationEroor::Invalid StateWitness")?;

                    let owner_address_str = inp.as_owner_address();
                    // extract pk from owner string
                    let owner_address: Address = match owner_address_str {
                        Some(owner) => Address::from_hex(owner, address::AddressType::Standard)?,
                        None => {
                            return Err("Owner address does not exist");
                        }
                    };
                    let pk: RistrettoPublicKey = owner_address.into();
                    // verify the witness
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
    //created for utxo-in-memory
    pub fn get_input_values(&self) -> Vec<Input> {
        self.inputs.clone()
    }
    pub fn get_output_values(&self) -> Vec<Output> {
        self.outputs.clone()
    }
}
