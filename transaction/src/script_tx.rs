use address::{Address, Network};
use merlin::Transcript;
use quisquislib::accounts::Account;
//use quisquislib::{keys::PublicKey, ristretto::RistrettoSecretKey};
use serde::{Deserialize, Serialize};
use zkvm::{
    zkos_types::{Input, Output, OutputCoin, OutputMemo, Witness}, // OutputCoin, Utxo},
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
    pub(crate) witness: Option<Vec<Witness>>,
    // Transaction data. e.g., supporting data for a script transaction.
    pub(crate) data: Vec<u8>,
}

// /// Represents a precomputed, but not verified transaction.
// pub struct PrecomputedTx {
//     /// Transaction header
//     pub header: TxHeader,

//     /// Transaction ID
//     pub id: TxID,

//     /// Transaction log: a list of changes to the blockchain state (UTXOs to delete/insert, etc.)
//     pub log: TxLog,

//     /// Fee rate of the transaction
//     pub feerate: FeeRate,

//     /// Verifier to continue verification of the transaction
//     pub(crate) verifier: Verifier,

//     /// Schnorr signature
//     pub(crate) signature: Signature,

//     /// R1CS proof
//     pub(crate) proof: R1CSProof,
// }

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
        witness: Option<Vec<Witness>>,
        data: Vec<u8>,
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
            data,
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
            None,
            vec![b'0'; 32],
        )
    }
    /// run the program and create a proof
    pub fn create_script_tx_without_witness(
        _prog: Program,
        _inputs: &[Input],
        _outputs: &[Output],
    ) {

        //Run the program and create a proof
    }

    ///create signatures and zero balance proofs for all inputs
    pub fn create_witness_without_tx(inputs: &[Input], sk_list: &[Scalar]) -> Vec<Witness> {
        let mut witness: Vec<Witness> = Vec::with_capacity(inputs.len());
        //iterate over Inputs and check its type
        for (i, inp) in inputs.iter().enumerate() {
            // create signature over input
            //extract public key of input
            let pk = address::Standard::from_hex(inp.input.owner().unwrap());
            //serialize input
            let inp_bytes: Vec<u8> = bincode::serialize(inp).unwrap();
            //create signature
            let sign = Signature::sign_message(
                ("ZKOS.Sign").as_bytes(),
                &inp_bytes,
                VerificationKey::from_bytes(pk.public_key.as_bytes().as_slice()).unwrap(),
                sk_list[i],
            );
            //if coin mark witness as Signature
            match inp.in_type {
                IOType::Coin => {
                    witness.push(Witness::Signature(sign));
                }
                //if data mark witness as ZeroBalanceProof
                IOType::Memo => {
                    witness.push(Witness::Signature(sign));
                }
                IOType::State => {
                    witness.push(Witness::Signature(sign));
                }
            }
        }
        witness
    }

    /// verify the script tx
    pub fn verify(&self) -> Result<(), &'static str> {
        //assume that the Utxo Ids have been verified already

        // Differentiate between contract deploy and contract call

        //verify the witnesses and the proofs of same value and zero balance proof as required
        self.verify_witnesses()?;

        // verify the call proof for the program to check the authenticity of the program
        let hasher = Hasher::new(b"ZkOS.MerkelTree");
        let prog = self.program.clone();
        // identify address from input state

        let address = Address::from_string("ZkOS.MerkelTree", Network::Testnet).unwrap();
        self.call_proof.verify_call_proof(&address, &prog, &hasher);

        // verify the r1cs proof
        let bp_gens = BulletproofGens::new(256, 1);
        let pc_gens = PedersenGens::default();
        let verifier = r1cs::Verifier::new(Transcript::new(b"ZkVM.r1cs"));
        let mut verifier = Verifier { cs: verifier };
        let mut vm = VMScript::new(
            VerifierRun::new(self.program.clone()),
            &bp_gens,
            &pc_gens,
            &mut verifier,
        );
        // initialize the Stack with inputs and outputs
        let _init_result = vm.initialize_stack()?;
        // run the program to create a proof
        let _run_result = vm.run()?;
        // verify the proof
        let _verify_result = vm.verify(self.proof.clone())?;

        Ok(())
    }

    // check if script is deploying contract
    // can also use Utxo existance to check this but this is more efficient
    pub fn is_contract_deploy(&self) -> bool {
        // loop over inputs and find if any inout is of type state
        for inp in self.inputs.iter() {
            if inp.in_type == zkvm::IOType::State {
                // get the witness for the input
                match &self.witness {
                    Some(wit) => {
                        // check if witness is of type ZeroBalanceProof
                        // get the witness from index
                        let state_witness = &wit[inp.get_witness_index() as usize]
                            .to_state_witness()
                            .unwrap();
                        // check if state witness is carrying a zero balance proof
                        match state_witness.get_zero_proof() {
                            Some(x) => {
                                return true;
                            }
                            None => {
                                return false;
                            }
                        }
                    }
                    None => {
                        return false;
                    }
                }
            }
        }
        false
    }

    // verify the witnesses and the proofs of same value and zero balance proof as required
    pub fn verify_witnesses(&self) -> Result<(), &'static str> {
        // get the witness vector
        let witness_vector: Vec<Witness> = self.witness.clone().expect("Witness Array is empty");
        // loop over inputs and extract their corresponding witnesses
        for (i, inp) in self.inputs.iter().enumerate() {
            match inp.in_type {
                IOType::Coin => {
                    let in_coin: &OutputCoin = inp.as_out_coin().expect("Input is not a coin");
                    // get corresponding OutputMemo
                    let out_memo: Output = self.outputs[i];
                    // get coin input witness
                    let coin_witness: zkvm::zkos_types::ValueWitness = witness_vector
                        [inp.get_witness_index() as usize]
                        .to_value_witness()
                        .expect("Witness is not a value witness for Input Coin");
                    // verify the witness
                    // get account from input
                    let acc: Account = inp.to_quisquis_account().expect("Input is not an account");
                    // get the public key from account
                    let (pk, _) = acc.get_account();
                    // get Pedersen commitment value from Memo
                    let memo_value = out_memo
                        .output
                        .get_commitment()
                        .expect("Memo is not a coin");

                    if !coin_witness.verify_value_witness(
                        inp.clone(),
                        pk,
                        acc,
                        memo_value.to_point(),
                    )? {
                        return Err("Value Witness Verification Failed");
                    }
                }
                IOType::Memo => {
                    let in_memo: &OutputMemo = inp.as_out_memo().unwrap();
                    // get corresponding OutputCoin
                    let out_coin: Output = self.outputs[i];

                    // get memo input witness
                    let memo_witness: zkvm::zkos_types::ValueWitness = witness_vector
                        [inp.get_witness_index() as usize]
                        .to_value_witness()
                        .expect("Witness is not a value witness for Input Memo");
                    // verify the witness
                    // get account from output
                    let acc: Account = out_coin
                        .to_quisquis_account()
                        .expect("Output is not an account");
                    // get public key from input
                    let (pk, _) = acc.get_account();
                    // get pedersen commitment value from input
                    let memo_value = inp.as_commitment().expect("Input is not a Memo commitment");
                    if !memo_witness.verify_value_witness(
                        inp.clone(),
                        pk,
                        acc,
                        memo_value.to_point(),
                    )? {
                        return Err("Value Witness Verification Failed");
                    }
                }
                IOType::State => {
                    // get the witness for the input

                    match &self.witness {
                        Some(wit) => {
                            // check if witness is of type ZeroBalanceProof
                            // get the witness from index
                            let state_witness = &wit[inp.get_witness_index() as usize]
                                .to_state_witness()
                                .unwrap();
                            // check if state witness is carrying a zero balance proof
                            match state_witness.get_zero_proof() {
                                Some(x) => {
                                    // verify the zero balance proof
                                    if !x.verify() {
                                        return Err("Zero Balance Proof Verification Failed");
                                    }
                                }
                                None => {
                                    return Err("Zero Balance Proof Not Found");
                                }
                            }
                            // check if state witness is carrying a same value proof
                            match state_witness.get_same_value_proof() {
                                Some(x) => {
                                    // verify the same value proof
                                    if !x.verify() {
                                        return Err("Same Value Proof Verification Failed");
                                    }
                                }
                                None => {
                                    return Err("Same Value Proof Not Found");
                                }
                            }
                        }
                        None => {
                            return Err("Witness Not Found");
                        }
                    }
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
