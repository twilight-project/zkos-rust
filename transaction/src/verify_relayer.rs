#![allow(non_snake_case)]
//#![deny(missing_docs)]

use ::zkschnorr::Signature;
use address::{Address, AddressType, Network};
use curve25519_dalek::{ristretto::CompressedRistretto, scalar::Scalar};
use merlin::Transcript;
use quisquislib::{
    accounts::prover::{Prover, SigmaProof},
    accounts::verifier::Verifier,
    accounts::Account,
    elgamal::ElGamalCommitment,
    keys::PublicKey,
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
};
use serde::{Deserialize, Serialize};
use zkvm::merkle::{CallProof, Hasher, MerkleTree, Path};
use zkvm::{
    zkos_types::{Input, InputData, Output, OutputCoin, OutputData, Utxo, ValueWitness, Witness},
    IOType, Program,
};

use crate::{ScriptTransaction, Transaction, TransactionData};

///Verifies the create_trade_order or create_lend_order
/// Input = Coin Input carrying the ZkosAccount
/// Output = Memo with the order details
/// Signature = Signature over input as Verifier view
/// proof = Sigma proof of same value committed in Coin and Memo
///
pub fn verify_trade_lend_order(
    input: Input,
    output: Output,
    signature: Signature,
    proof: SigmaProof,
) -> Result<bool, &'static str> {
    //check owner address on Coin and Memo are same
    if input.as_owner_address().unwrap().to_owned() != output.as_out_memo().unwrap().owner {
        return Err("Owner address on Coin and Memo are different");
    }
    //extract publickey from owner address of input coin
    let owner: String = input.as_owner_address().unwrap().to_owned();
    let address: Address = Address::from_hex(&owner, AddressType::default()).unwrap();
    let pk: RistrettoPublicKey = address.as_c_address().public_key;

    //create the Verifier View of the Coin and set the Witness to 0
    let input_sign = input.as_input_for_signing();

    //get Pk from input owner address and create account
    let enc_acc: Account = Account::set_account(pk.clone(), input.as_encryption().unwrap().clone());

    //get verifier view from the output memo
    let memo = output.as_out_memo().unwrap().to_owned();
    let memo_verifier = memo.verifier_view();

    //extract the commitment value from the memo
    let commitment: CompressedRistretto = memo_verifier.commitment.into();

    // verify the Signature over input and Same value Sigma Proof
    let value_witness = ValueWitness::set_value_witness(signature.clone(), proof.clone());
    let verify = value_witness.verify_value_witness(input_sign, pk, enc_acc, commitment);
    if verify.is_ok() {
        Ok(true)
    } else {
        Err("Signature and Sigma Proof verification failed")
    }
}

/// Verifies the settlement request for Trader or lend order
/// Input = Memo carrying the order details
/// Signature = Signature over the input as Verifier view
pub fn verify_settle_requests(input: Input, signature: Signature) -> Result<(), &'static str> {
    //extract publickey from owner address of input memo
    let owner: String = input.as_owner_address().unwrap().to_owned();
    let address: Address = Address::from_hex(&owner, AddressType::default()).unwrap();
    let pk: RistrettoPublicKey = address.as_c_address().public_key;

    // create verifier and signature view for the input
    // Verifier view is created by converting the Input Commitment to a  Compressed point
    // Signature view is created by setting the 'witness' in input as 0
    let mut message: Vec<u8> = Vec::new();

    if input.in_type == IOType::Coin {
        //create the Verifier View of the Coin and set the Witness to 0
        let input_sign = input.as_input_for_signing();
        //serialize the input for sign verification
        message = bincode::serialize(&input_sign).unwrap();
    } else if input.in_type == IOType::Memo {
        // Create the Verifier View of the Memo and set the Witness to 0
        let memo = input.as_out_memo().unwrap().to_owned();
        //convert commitment into point
        let memo_verifier = memo.verifier_view();

        // create signer view over the resultant verifier view neno
        let input_sign = Input::memo(InputData::memo(
            input.as_utxo().unwrap().to_owned(),
            memo_verifier,
            0,
            input.input.get_commitment_value_memo().unwrap().to_owned(),
        ));
        //serialize the input for sign verification
        message = bincode::serialize(&input_sign).unwrap();
    }
    let verify = pk.verify_msg(&message, &signature, ("PublicKeySign").as_bytes());
    if verify.is_ok() {
        Ok(())
    } else {
        Err("Signature verification failed")
    }
}

/// Verifies the query request for Trader or lend order
/// Verifies the cancel order request
/// address = Hex Address string of the trader or lender zkosAccount used for creating the order
/// signature = Signature over the standard request (QueryTraderOrder/ QueryLendOrder/ CancelTraderOrder)
/// message = Message used for signing the query request. Bincode Serialized (QueryTraderOrder/ QueryLendOrder /CancelTraderOrder) type
pub fn verify_query_order(
    address: String,
    signature: Signature,
    message: &[u8],
) -> Result<(), &'static str> {
    //extract Address from hex
    let add: Address = Address::from_hex(&address, AddressType::default()).unwrap();
    //extract the public key from address
    let pk: RistrettoPublicKey = add.as_c_address().public_key;
    //verify the signature
    let verify = pk.verify_msg(message, &signature, ("PublicKeySign").as_bytes());

    if verify.is_ok() {
        Ok(())
    } else {
        Err("Signature verification failed")
    }
}
/// Creates the ScriptTransaction for creating the trade order on relayer for chain
// input_coin :  Input received from the trader
// output_memo : Output Memo created by the trader
// signature : Signature over the input_coin as Verifier view sent by trader
// proof : Sigma proof of same value committed in Coin and Memo sent by the trader
// order_msg: order message serialized. CreateTraderOrder struct should be passed here. Ideally this information should be Encrypted

pub fn create_trade_order(
    input_coin: Input,
    output_memo: Output,
    signature: Signature,
    proof: SigmaProof,
    order_msg: Vec<u8>, //order message serialized
) -> Transaction {
    //assuming there is no verifier at the chain yet

    //creating the witness
    let witness = Witness::ValueWitness(ValueWitness::set_value_witness(
        signature.clone(),
        proof.clone(),
    ));

    let witness_vec = vec![witness];

    //create input vector
    let inputs = vec![input_coin];

    //create output vector
    let outputs = vec![output_memo];

    //get the program
    let correct_program = self::get_trader_order_program();

    //creating the program proof
    //cretae unsigned Tx with program proof
    let result = crate::vm_run::Prover::build_proof(correct_program, &inputs, &outputs);
    // println!("result:{:?}", result);
    let (program, proof) = result.unwrap();
    // println!("program:{:?}, /n proof: {:?}", program, proof);

    //get program call proof and address
    let (call_proof, address) = create_call_proof();
    let script_tx = ScriptTransaction::set_script_transaction(
        0u64,
        0u64,
        0u64,
        inputs.len() as u8,
        outputs.len() as u8,
        witness_vec.len() as u8,
        inputs.to_vec(),
        outputs.to_vec(),
        program,
        call_proof,
        proof,
        Some(witness_vec.to_vec()),
        order_msg.to_vec(),
    );
    // println!("{:?}", result);
    Transaction::from(script_tx)
}

/// Creates the ScriptTransaction for creating the Lend order on relayer for chain
// input_coin :  Input received from the trader
// output_memo : Output Memo created by the trader
// signature : Signature over the input_coin as Verifier view sent by trader
// proof : Sigma proof of same value committed in Coin and Memo sent by the trader
// old_total_locked_value: TLV_0. if in BTC convert to SATS
// old_total_pool_share: u64, TPS_0. should always be a decimal number
// new_total_locked_value:  TLV_1  if in BTC convert to SATS
// new_total_poolshare_value: TPS_1 should always be a decimal number
pub fn create_lend_order(
    input_coin: Input,
    output_memo: Output,
    signature: Signature,
    proof: SigmaProof,
    old_total_locked_value: u64,
    old_total_pool_share: u64,
    new_total_locked_value: u64,
    new_total_pool_share: u64,
    rscalar: Scalar,
) /*-> Transaction, Output, Scalar */
{
    /*Outputs */
    //Transaction:: Complete chain transaction with the program and proof to be relayed to the Chain
    //Output: Output Memo to be sent to the trader. Store in the order_id -> (Output, TxId) DB
    //Scalar: rscalar to be used for creating the proof for the next state update. Most recent Stored by relayer
}

/// Creates the ScriptTransaction for Sttlement request for Trader or Liquidation
// input_memo : Input Memo created and sent by the trader
// Signature : Signature over the input_memo as Prover view sent by trader
// Available_margin:  Available margin of the trader for the current order  in SATS
// Payment: Payment amount to be sent to the trader in CENTS
// Liquidation Price:: liquidation price of the trader in CENTS
// old_total_locked_value: TLV_0 if in BTC convert to SATS
// new_total_locked_value:  TLV_1  if in BTC convert to SATS
//  total_pool_share: u64, TPS_0    should always be a decimal number. No calculation is done. ONLY NEEDED FOR CREATING THE PROOF
pub fn settle_trader_order(
    input_memo: Input,
    signature: Signature,
    available_margin: u64,
    payment: u64,
    liquidation_price: Option<u64>,
    old_total_locked_value: u64,
    new_total_locked_value: u64,
    total_pool_share: u64,
    rscalar: Scalar,
) /*-> (Transaction, Output, Scalar)*/
{
    /*Outputs */
    //Transaction:: Complete chain transaction with the program and proof to be relayed to the Chain
    //Output: Output Coin to be sent to the trader. Store in the order_id -> (Output, TxId) DB
    //Scalar: rscalar to be used for creating the proof for the next state update. Most recent Stored by relayer

    //create coin and same value proof for the output based on the payment amount
    let out_address: String = input_memo.as_owner_address().unwrap().to_owned();
    let out_pk: RistrettoPublicKey = Address::from_hex(&out_address, AddressType::default())
        .unwrap()
        .as_c_address()
        .public_key;
    let (_, out_encryption_scalar) = input_memo
        .input
        .get_commitment_value_memo()
        .unwrap()
        .to_owned()
        .witness()
        .unwrap();

    match liquidation_price {
        Some(liquidation_price) => {
            //Liquidation
            let out_encryption = ElGamalCommitment::generate_commitment(
                &out_pk,
                out_encryption_scalar,
                Scalar::from(0u64),
            );

            let out_coin = Output::coin(OutputData::coin(OutputCoin::new(
                out_encryption,
                out_address.clone(),
            )));
        }
        None => {
            //Settlement
            //create coin and same value proof for the output based on the payment amount
            let out_encryption = ElGamalCommitment::generate_commitment(
                &out_pk,
                out_encryption_scalar,
                Scalar::from(payment),
            );

            let out_coin = Output::coin(OutputData::coin(OutputCoin::new(
                out_encryption,
                out_address.clone(),
            )));
        }
    }
}

// pub fn settle_lend_order()-> Transaction {

// }
pub fn get_trader_order_program() -> Program {
    let order_prog = Program::build(|p| {
        p.commit()
            .expr()
            .neg()
            .roll(1)
            .commit()
            .expr()
            .add()
            .range()
            .drop();
    });
    return order_prog;
}

pub fn create_call_proof() -> (CallProof, Address) {
    // create a tree of programs
    let hasher = Hasher::new(b"ZkOS.MerkelTree");

    let prog1 = self::get_trader_order_program();
    let progs = vec![prog1.clone()];
    //create tree root
    let root = MerkleTree::root(b"ZkOS.MerkelTree", progs.iter());
    //convert root to address
    let address = Address::script_address(Network::Mainnet, root.0);
    //script address as hex
    let address_hex = address.as_hex();

    // create path for program3
    // let _path = Path::new(&progs, 2 as usize, &hasher).unwrap();

    // create call proof for program3
    let call_proof =
        CallProof::create_call_proof(&progs, 0 as usize, &hasher, address_hex).unwrap();
    (call_proof, address)
}

//
// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------
#[cfg(test)]
mod test {
    use super::*;
    use quisquislib::elgamal::ElGamalCommitment;
    use quisquislib::keys::SecretKey;
    use rand::rngs::OsRng;
    use zkvm::zkos_types::OutputMemo;
    #[test]
    fn test_verify_query_order() {
        let (acc, prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
        let (pk, enc) = acc.get_account();
        let message = ("0a000000000000006163636f756e745f6964040000008c0000000000000022306366363661623465306432373239626538373835333366376663313866336364313862316337383764396230336262343163303263326235316561353239373437326330633433323934646131653035643736353235633234393336383234303636356565353632353363656435333466656362616536313437336130343737663631613866616634224000000000000000180bdfbb82e758e70684c3125b011a10b2205db929867c7507e3b156ff96be2f6a2aaeb522576b54743fdf5f10bc7ecb88328d15d35c98a2b0512b60fc0da405").as_bytes();
        let signature: Signature = pk.sign_msg(&message, &prv, ("PublicKeySign").as_bytes());
        //Verification
        let address: Address = Address::standard_address(Network::default(), pk.clone());
        let add_hex: String = address.as_hex();
        let verify_query = verify_query_order(add_hex, signature, &message);
        println!("verify_query: {:?}", verify_query);
        assert!(verify_query.is_ok());
    }

    #[test]
    fn TEST_verify_settle_requests() {
        let (acc, prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
        let (pk, enc) = acc.get_account();
        let message = ("0a000000000000006163636f756e745f6964040000008c0000000000000022306366363661623465306432373239626538373835333366376663313866336364313862316337383764396230336262343163303263326235316561353239373437326330633433323934646131653035643736353235633234393336383234303636356565353632353363656435333466656362616536313437336130343737663631613866616634224000000000000000180bdfbb82e758e70684c3125b011a10b2205db929867c7507e3b156ff96be2f6a2aaeb522576b54743fdf5f10bc7ecb88328d15d35c98a2b0512b60fc0da405").as_bytes();
        let signature: Signature = pk.sign_msg(&message, &prv, ("PublicKeySign").as_bytes());
        //Verification
        let address: Address = Address::standard_address(Network::default(), pk.clone());
        let add_hex: String = address.as_hex();
        let verify_query = verify_query_order(add_hex, signature, &message);
        println!("verify_query: {:?}", verify_query);
        assert!(verify_query.is_ok());
    }
    use zkvm::Commitment;
    #[test]
    fn test_verify_trade_lend_order() {
        //create the input coin
        let mut rng = rand::thread_rng();
        let sk: RistrettoSecretKey = SecretKey::random(&mut rng);
        let pk = RistrettoPublicKey::from_secret_key(&sk, &mut rng);
        let comm_scalar = Scalar::random(&mut OsRng);
        let enc =
            ElGamalCommitment::generate_commitment(&pk, comm_scalar.clone(), Scalar::from(20u64));

        let address: Address = Address::standard_address(Network::default(), pk.clone());
        let add_hex: String = address.as_hex();

        let coin = OutputCoin::new(enc.clone(), add_hex.clone());
        let utxo: Utxo = Utxo::default();
        let coin_input = Input::coin(InputData::coin(utxo, coin, 0));

        //create the output memo for the input coin
        let commit = Commitment::blinded_with_factor(20u64, comm_scalar.clone());

        let out_memo: OutputMemo =
            OutputMemo::new(add_hex.clone(), add_hex.clone(), commit, None, 0);
        let output = Output::memo(OutputData::memo(out_memo.clone()));
        let enc_acc = Account::set_account(pk.clone(), enc.clone());
        let out_memo_verifier = out_memo.verifier_view();
        let input_sign = coin_input.as_input_for_signing();
        //Verification
        let witness = ValueWitness::create_value_witness(
            input_sign.clone(),
            sk,
            enc_acc,
            pk.clone(),
            out_memo_verifier.commitment.into(),
            20u64,
            comm_scalar.clone(),
        );

        //Verification

        let signature = witness.get_signature().to_owned();
        let proof = witness.get_value_proof().to_owned();

        let verify_query = verify_trade_lend_order(coin_input, output, signature, proof);
        println!("verify_query: {:?}", verify_query);
        assert!(verify_query.is_ok());
    }
}
