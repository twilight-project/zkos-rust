#![allow(non_snake_case)]
//#![deny(missing_docs)]

use ::zkschnorr::Signature;
use address::{Address, AddressType, Network};
use curve25519_dalek::{ristretto::CompressedRistretto, scalar::Scalar};
//use merlin::Transcript;
use quisquislib::{
    accounts::prover::SigmaProof,
    // accounts::verifier::Verifier,
    accounts::Account,
    elgamal::ElGamalCommitment,
    keys::PublicKey,
    ristretto::RistrettoPublicKey,
};
//use serde::{Deserialize, Serialize};
//use rangeproof::signed_integer::SignedInteger;
use zkvm::merkle::{CallProof, Hasher, MerkleTree};
use zkvm::{
    zkos_types::{Input, InputData, Output, OutputCoin, OutputData, ValueWitness, Witness},
    IOType, Program, String as ZString,
};

use crate::{ScriptTransaction, Transaction};

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
    let pk: RistrettoPublicKey = address.as_coin_address().public_key;

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
    let pk: RistrettoPublicKey = address.as_coin_address().public_key;

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
        let coin_value = input
            .input
            .get_coin_value_input_memo()
            .as_ref()
            .unwrap()
            .to_owned();
        // create signer view over the resultant verifier view neno
        let input_sign = Input::memo(InputData::memo(
            input.as_utxo().unwrap().to_owned(),
            memo_verifier,
            0,
            Some(coin_value),
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
    let pk: RistrettoPublicKey = add.as_coin_address().public_key;
    //verify the signature
    let verify = pk.verify_msg(message, &signature, ("PublicKeySign").as_bytes());

    if verify.is_ok() {
        Ok(())
    } else {
        Err("Signature verification failed")
    }
}
/// Chain Transaction to deploy Relayer Contract
/// Initializes the state.
/// This transaction should be run to fire the relayer on chain. No relayer operation can happen without doing this.
/// Pre-req:: Create a Trading to funding account transaction to create ZkosAccount first.
/// The amount inside the ZkosAccount will be used to initialize the Relayer Pool and deploy Relayer Contract on chain
///
/// Locks inital Amount into the TVL and TPS.
///
/*******  Inputs *******/
// Address:: Hex qq account Address of the relayer
// Amount:: Amount to be locked in the TVL and TPS. Assuming the relayer wants to lock the whole amount
// Scalar:: Scalar to be used for creating the ecryption(Coin) and commitment(Memo) for the SigmaProof
///
/*******  Ouputs *******/
//Transaction :: Complete chain transaction with the program and proof to be relayed to the Chain
//Scalar:: rscalar to be used for creating the proof for the next state update. Most recent Stored by relayer
//Output:: OutputMemo to be stored by the relayer for future use. If he loses it. He loses all his money
///
pub fn deploy_relayer_contract(
    _utxo: String,         //get from ZkosOracle
    owner_address: String, //Hex string
    amount: u64,
    scalar_hex: String, //Hex string. Get from chain
) /*-> (Transaction, Scalar, Output)*/
{
    //recreate scalar from hex
    let scalar_bytes = hex::decode(&scalar_hex).unwrap();
    let scalar = Scalar::from_bytes_mod_order(scalar_bytes.try_into().unwrap());
    // recreate utxo from json
    //let utxo: Utxo = serde_json::from_str(&utxo).unwrap();

    //create the input coin
    //get pk from address
    let address: Address = Address::from_hex(&owner_address, AddressType::default()).unwrap();
    let pk: RistrettoPublicKey = address.as_coin_address().public_key;
    //create commitment
    let enc = ElGamalCommitment::generate_commitment(&pk, scalar.clone(), Scalar::from(amount));
    //create coin
    let _out_coin = OutputCoin::new(enc.clone(), owner_address.clone());
    //create Coin input with witness index = 0
    //let input: Input = Input::coin(InputData::coin(utxo, out_coin, 0));
    //create script address

    //create output Memo
    // let output_memo =
    //   OutputMemo::new_from_wasm(script_address, owner_address, balance, order_size, scalar);
}

/// Creates the ScriptTransaction for creating the trade order on relayer for chain
///
/*******  Inputs *******/
// input_coin :  Input received from the trader
// output_memo : Output Memo created by the trader
// signature : Signature over the input_coin as Verifier view sent by trader
// proof : Sigma proof of same value committed in Coin and Memo sent by the trader
// order_msg: order message serialized. CreateTraderOrder struct should be passed here. Ideally this information should be Encrypted

/*******  Ouputs *******/
//Transaction :: Complete chain transaction with the program and proof to be relayed to the Chain
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
    println!("Program \n{:?}", correct_program);

    //creating the program proof
    //cretae unsigned Tx with program proof
    let result =
        crate::vm_run::Prover::build_proof(correct_program, &inputs, &outputs, false, None);
    let (program, proof) = result.unwrap();
    //get program call proof and address
    let _address_str = create_script_address(Network::default());
    let call_proof = create_call_proof(Network::default());
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
        witness_vec.to_vec(),
        None,
    );
    // println!("{:?}", result);
    Transaction::from(script_tx)
}

/// Creates the ScriptTransaction for creating the Lend order on relayer for chain
///
/*******  Inputs *******/
// input_coin :  Input received from the trader
// output_memo : Output Memo created by the trader
// signature : Signature over the input_coin as Verifier view sent by trader
// proof : Sigma proof of same value committed in Coin and Memo sent by the trader
// old_total_locked_value: TLV_0. if in BTC convert to SATS
// old_total_pool_share: u64, TPS_0. should always be a decimal number
// new_total_locked_value:  TLV_1  if in BTC convert to SATS
// new_total_poolshare_value: TPS_1 should always be a decimal number
///
/*******  Ouputs *******/
//Transaction:: Complete chain transaction with the program and proof to be relayed to the Chain
//Output: Output Memo to be sent to the trader. Store in the order_id -> (Output, TxId) DB
//Scalar: rscalar to be used for creating the proof for the next state update. Most recent Stored by relayer
pub fn create_lend_order(
    _input_coin: Input,
    _output_memo: Output,
    _signature: Signature,
    _proof: SigmaProof,
    _old_total_locked_value: u64,
    _old_total_pool_share: u64,
    _new_total_locked_value: u64,
    _new_total_pool_share: u64,
    _rscalar: Scalar,
) /*-> Transaction, Output, Scalar */
{
}

/// Creates the ScriptTransaction for Sttlement request for Trader or Liquidation
///
/*******  Inputs *******/
// input_memo : Input Memo created and sent by the trader
// Signature : Signature over the input_memo as Prover view sent by trader
// Available_margin:  Available margin of the trader for the current order  in SATS
// Payment: Payment amount to be sent to the trader in CENTS
// Liquidation Price:: liquidation price of the trader in CENTS
// old_total_locked_value: TLV_0 if in BTC convert to SATS
// new_total_locked_value:  TLV_1  if in BTC convert to SATS
//  total_pool_share: u64, TPS_0    should always be a decimal number. No calculation is done. ONLY NEEDED FOR CREATING THE PROOF
///
/*******  Ouputs *******/
//Transaction:: Complete chain transaction with the program and proof to be relayed to the Chain
//Output: Output Coin to be sent to the trader. Store in the order_id -> (Output, TxId) DB
//Scalar: rscalar to be used for creating the proof for the next state update. Most recent Stored by relayer
pub fn settle_trader_order(
    input_memo: Input,
    _signature: Signature,
    _available_margin: u64,
    payment: u64,
    liquidation_price: Option<u64>,
    _old_total_locked_value: u64,
    _new_total_locked_value: u64,
    _total_pool_share: u64,
    _rscalar: Scalar,
) /*-> (Transaction, Output, Scalar)*/
{
    //create coin and same value proof for the output based on the payment amount
    let out_address: String = input_memo.as_owner_address().unwrap().to_owned();
    let out_pk: RistrettoPublicKey = Address::from_hex(&out_address, AddressType::default())
        .unwrap()
        .as_coin_address()
        .public_key;
    let (_, out_encryption_scalar) = input_memo
        .input
        .get_coin_value_input_memo()
        .as_ref()
        .unwrap()
        .to_owned()
        .witness()
        .unwrap();

    match liquidation_price {
        Some(_liquidation_price) => {
            //Liquidation
            let out_encryption = ElGamalCommitment::generate_commitment(
                &out_pk,
                out_encryption_scalar,
                Scalar::from(0u64),
            );

            let _out_coin = Output::coin(OutputData::coin(OutputCoin::new(
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

            let _out_coin = Output::coin(OutputData::coin(OutputCoin::new(
                out_encryption,
                out_address.clone(),
            )));
        }
    }
}

// pub fn settle_lend_order()-> Transaction {

// }
pub fn get_trader_order_program_old() -> Program {
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
/// Program to prove the PositionSize calculation
/// PositionSize = IM * Leverage * EntryPrice
/// Memo Commitmtent = IM
/// Data -> PositionSize, Leverage, EntryPrice
pub fn get_trader_order_program() -> Program {
    let order_prog = Program::build(|p| {
        p.roll(3) // Get IM to top of stack
            .commit()
            .expr()
            .roll(1) // Get EntryPrice to top of stack
            .scalar()
            .mul() // EntryPrice * IM
            .roll(1) // Get Leverage to top of stack
            .commit()
            .expr()
            .mul() // Leverage * EntryPrice * IM
            .roll(1)
            .scalar()
            .eq()
            .verify();
    });
    return order_prog;
}

/// Program to prove the Settlement calculations
/// (AM - IM - mD) * EntryPrice * SettlePrice = PositionSize * (EntryPrice - SettlePrice) * OrderSide(-1/1) + Error
/// Error = (AM - IM - mD) * EntryPrice * SettlePrice - (PositionSize * (EntryPrice - SettlePrice) * OrderSide)
/// and TVL1 = TVL0 - Payment  where payment = Am -IM
/// Memo and State used
/// Memo = IM , Data -> PositionSize, Leverage, EntryPrice, OrderSide
/// InputMemo provides AM
/// State = TVL , TPS InputState -> mD, Error  
/// SettlePrice - > tx_data
/*  Stack
    IM ->PositionSize->Leverage->EntryPrice->OrderSide->AM
    ->TVL_0->TVL_1->TPS0->TPS1->mD->Error->SettlePrice -> IM

    PositionSize->Leverage->EntryPrice->OrderSide
    ->TVL_0->TPS0 -> TVL_1->TPS1->mD->Error->SettlePrice -> AM-IM -> AM-IM

     PositionSize->Leverage->EntryPrice->OrderSide
    ->TVL_0->TPS0 -> TVL_1->TPS1Error->SettlePrice -> AM-IM -> AM-IM-mD

    PositionSize->Leverage->EntryPrice->OrderSide
    ->TVL_0->TPS0 -> TVL_1->TPS1->Error->SettlePrice -> AM-IM -> AM-IM-mD * SettlePrice
*/
pub fn get_settle_trader_order_program() -> Program {
    let order_prog = Program::build(|p| {
        p.roll(3) //drop TPS1
            .drop()
            .roll(3)
            .drop() //drop TPS0
            .roll(10) // Get IM to top of stack
            .commit()
            .dup(0) // duplicate IM
            .expr()
            .neg() // -IM
            .roll(7) // Get AM to top of stack
            .commit()
            .dup(0) // duplicate AM
            .expr()
            .roll(2) // get -IM to top
            .add() // AM - IM = Payment
            .roll(2) // get IM
            .expr()
            .neg() // -IM
            .roll(2) //get AM
            .expr()
            .add() //AM -IM
            .roll(4) //marginDifference
            .commit()
            .expr()
            .neg() // -mD
            .add() // AM - IM - mD
            .dup(2) //duplicate  SettlePrice
            .scalar()
            .mul() // SettlePrice * (AM - IM - mD)
            .dup(7) //duplicate entryprice
            .scalar()
            .mul() // EntryPrice * SettlePrice * (AM - IM - mD)
            .roll(7) // get EntryPrice
            .scalar()
            .roll(3) //get SettlePrice
            .scalar()
            .neg() //-settlePrice
            .add() // entryPrice - settlePrice
            .roll(6) // get Order Side (-1 for Long / 1 for short)
            .scalar()
            .mul() // OrderSide * (EntryPrice - SettlePrice)
            .roll(7) // get PositionSize
            .scalar()
            .mul() // PositionSize * OrderSide * (EntryPrice - SettlePrice)
            .roll(3) //get Error
            .scalar()
            .add() // Error + PositionSize * OrderSide * (EntryPrice - SettlePrice)
            .eq() //(Payment - marginDifference)*EntryPrice*SettlePrice = Error + PositionSize * OrderSide * (EntryPrice - SettlePrice)
            .roll(1) // get Payment = AM - IM as expression
            .neg()
            .roll(3) //get TVL0
            .commit()
            .expr()
            .add() //TVL0 - Payment
            .roll(2) // get TVL1
            .commit()
            .expr()
            .eq() // TVL1 = TVL0 - Payment
            .and() // Bind both constraints
            .verify()
            .drop(); // drop leverage
    });
    return order_prog;
}

pub fn contract_initialize_program_with_stack_short() -> Program {
    let order_prog = Program::build(|p| {
        p.drop()
            .commit()
            .expr()
            .roll(1)
            .commit()
            .expr()
            .neg()
            .add()
            .range()
            .drop();
    });
    return order_prog;
}
pub fn lend_order_deposit_program() -> Program {
    let order_prog = Program::build(|p| {
        // TPS1 - TPS0 = PS or TPS1 = PS + TPS0
        p.roll(1) //TPS1
            .commit()
            .expr()
            .dup(2) // TPS0
            .commit()
            .expr()
            .dup(6) // nPoolshare
            .commit()
            .expr()
            .add() //
            .eq() //  TPS0 + nPoolShare = TPS1
            .roll(3) //TLV1
            .commit()
            .expr()
            .dup(4) //TLV0
            .commit()
            .expr()
            .dup(7) // Deposit
            .commit()
            .expr()
            .add() //Deposit + tlv
            .eq() // TLV1 = Deposit + TLV0
            .and() // TPS== &&  TLV== &&
            .roll(1) // error
            .scalar()
            .roll(2) // TPS0
            .commit()
            .expr()
            .roll(5) //Deposit
            .commit()
            .expr()
            .mul() //Deposit * TPS0
            .add() // Deposit * TPS0 + error
            .roll(2) // TVL0
            .commit()
            .expr()
            .roll(3) // nPoolshare
            .commit()
            .expr()
            .mul() // TVL0 * nPoolshare
            .eq()
            .and()
            .verify();
    });
    return order_prog;
}

pub fn lend_order_settle_program() -> Program {
    let order_prog = Program::build(|p| {
        // TPS1 - TPS0 = PS or TPS1 = PS + TPS0
        p.scalar() // Error
            //.neg() // -Error
            .dup(4) //TLV0
            .commit()
            .expr()
            .dup(7) // nPoolShare
            .commit()
            .expr()
            .mul() //nPoolShare * TLV0
            .add() // nPoolShare * TLV0 - Error
            .dup(2) // TPS0
            .commit()
            .expr()
            .dup(6) // nWithdraw
            .commit()
            .expr()
            .mul() // TPS0 * nWithdraw
            .eq() //  TPS0 * nWithdraw = nPoolShare * TLV0 + Error
            .roll(6) //nPoolShare
            .commit()
            .expr()
            .neg() // -nPoolShare
            .roll(3) //TPS0
            .commit()
            .expr()
            .add() // TPS0 -nPoolShare
            .roll(2) //TPS1
            .commit()
            .expr()
            .eq() //  TPS1 = TPS0 - nPoolShare
            .and() // Adding 2 Equalities together
            .roll(1) //TVL1
            .commit()
            .expr()
            .roll(2) //TVL0
            .commit()
            .expr()
            .roll(3) //nWithdraw
            .commit()
            .expr()
            .neg()
            .add() // TLV0- nWithdraw
            .eq() // TLV1 = TLV0 - nWithdraw
            .and() // rolling all constraints together
            .verify()
            // .drop() //dropping deposit off stack
            // .drop()
            // .drop()
            // .drop()
            // .drop()
            // .drop()
            .drop();
    });
    return order_prog;
}

fn create_program_tree() -> Vec<Program> {
    let prog1 = self::get_trader_order_program();
    let prog2: Program = self::contract_initialize_program_with_stack_short();
    vec![prog1.clone(), prog2.clone()]
}
pub fn create_call_proof(network: Network) -> CallProof {
    // create a tree of programs
    let hasher = Hasher::new(b"ZkOS.MerkelTree");
    let prog_list = create_program_tree();
    // create call proof for program3
    let call_proof =
        CallProof::create_call_proof(&prog_list, 1 as usize, &hasher, network).unwrap();
    call_proof
}

pub fn create_script_address(network: Network) -> String {
    // create a tree of programs
    let prog_list = create_program_tree();
    //create tree root
    let root = MerkleTree::root(b"ZkOS.MerkelTree", prog_list.iter());
    //convert root to address
    let address = Address::script_address(network, root.0);
    //script address as hex
    address.as_hex()
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
    use zkvm::zkos_types::{OutputMemo, OutputState};

    #[test]
    fn test_verify_query_order() {
        let (acc, prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
        let (pk, _enc) = acc.get_account();
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
    fn Test_verify_settle_requests() {
        let (acc, prv) = Account::generate_random_account_with_value(Scalar::from(20u64));
        let (pk, _enc) = acc.get_account();
        let message = ("0a000000000000006163636f756e745f6964040000008c0000000000000022306366363661623465306432373239626538373835333366376663313866336364313862316337383764396230336262343163303263326235316561353239373437326330633433323934646131653035643736353235633234393336383234303636356565353632353363656435333466656362616536313437336130343737663631613866616634224000000000000000180bdfbb82e758e70684c3125b011a10b2205db929867c7507e3b156ff96be2f6a2aaeb522576b54743fdf5f10bc7ecb88328d15d35c98a2b0512b60fc0da405").as_bytes();
        let signature: Signature = pk.sign_msg(&message, &prv, ("PublicKeySign").as_bytes());
        //Verification
        let address: Address = Address::standard_address(Network::default(), pk.clone());
        let add_hex: String = address.as_hex();
        let verify_query = verify_query_order(add_hex, signature, &message);
        println!("verify_query: {:?}", verify_query);
        assert!(verify_query.is_ok());
    }
    use zkvm::{Commitment, Utxo};
    #[test]
    fn test_verify_trade_lend_order() {
        //create the input coin
        let mut rng = rand::thread_rng();
        let sk: quisquislib::ristretto::RistrettoSecretKey = SecretKey::random(&mut rng);
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
    use crate::vm_run::{Prover, Verifier};
    #[test]
    fn test_trade_order_tx_new() {
        let correct_program = self::get_trader_order_program();
        println!("\n Program \n{:?}", correct_program);

        //create InputCoin and OutputMemo

        let mut rng = rand::thread_rng();
        let sk_in: quisquislib::ristretto::RistrettoSecretKey = SecretKey::random(&mut rng);
        let pk_in = RistrettoPublicKey::from_secret_key(&sk_in, &mut rng);
        let commit_in = ElGamalCommitment::generate_commitment(
            &pk_in,
            Scalar::random(&mut rng),
            Scalar::from(100u64),
        );
        let add: Address = Address::standard_address(Network::default(), pk_in.clone());
        let out_coin = OutputCoin {
            encrypt: commit_in,
            owner: add.as_hex(),
        };
        let in_data: InputData = InputData::coin(Utxo::default(), out_coin, 0);
        let coin_in: Input = Input::coin(in_data);
        let input: Vec<Input> = vec![coin_in];

        //*****  OutputMemo  *********/
        //****************************/
        let script_address =
            Address::script_address(Network::Mainnet, *Scalar::random(&mut rng).as_bytes());
        //IM
        let commit_memo = Commitment::blinded(100u64);
        //Leverage committed
        let leverage = Commitment::blinded(5u64);
        // entryprice in cents
        let entry_price = 50u64;
        // PositionSize
        let position_size = 25000u64;

        let data: Vec<ZString> = vec![
            ZString::from(Scalar::from(position_size)),
            ZString::from(leverage),
            ZString::from(Scalar::from(entry_price)),
        ];
        let memo_out = OutputMemo {
            script_address: script_address.as_hex(),
            owner: add.as_hex(),
            commitment: commit_memo,
            data: Some(data),
            timebounds: 0,
        };
        let out_data = OutputData::Memo(memo_out);
        let memo = Output::memo(out_data);
        let output: Vec<Output> = vec![memo];

        //cretae unsigned Tx with program proof
        let result = Prover::build_proof(correct_program, &input, &output, false, None);
        println!("{:?}", result);
        let (prog_bytes, proof) = result.unwrap();
        let verify = Verifier::verify_r1cs_proof(&proof, &prog_bytes, &input, &output, false, None);
        println!("{:?}", verify);
    }

    #[test]
    fn test_trade_settle_order_stack_new() {
        let correct_program = self::get_settle_trader_order_program();
        println!("\n Program \n{:?}", correct_program);

        //create InputMemo and OutputCoin

        let mut rng = rand::thread_rng();
        let sk_in: quisquislib::ristretto::RistrettoSecretKey = SecretKey::random(&mut rng);
        let pk_in = RistrettoPublicKey::from_secret_key(&sk_in, &mut rng);
        let commit_in = ElGamalCommitment::generate_commitment(
            &pk_in,
            Scalar::random(&mut rng),
            Scalar::from(82u64),
        );
        let add: Address = Address::standard_address(Network::default(), pk_in.clone());
        let out_coin = OutputCoin {
            encrypt: commit_in,
            owner: add.as_hex(),
        };
        let coin_out: Output = Output::coin(OutputData::coin(out_coin));

        //*****  InputMemo  *********/
        //****************************/
        let script_address =
            Address::script_address(Network::Mainnet, *Scalar::random(&mut rng).as_bytes());
        //IM
        let commit_memo = Commitment::blinded(50u64);
        //Leverage committed
        let leverage = Commitment::blinded(5u64);
        // entryprice in cents
        let entry_price = 500u64;
        // PositionSize
        let position_size = 125000u64;
        let order_side: u64 = 1u64;
        let data: Vec<ZString> = vec![
            ZString::from(Scalar::from(position_size)),
            ZString::from(leverage),
            ZString::from(Scalar::from(entry_price)),
            ZString::from(Scalar::from(order_side)),
        ];
        let memo_out = OutputMemo {
            script_address: script_address.as_hex(),
            owner: add.as_hex(),
            commitment: commit_memo,
            data: Some(data),
            timebounds: 0,
        };
        let coin_value: Commitment = Commitment::blinded(82u64); // AM to be pushed back to the user
        let memo_in = Input::memo(InputData::memo(
            Utxo::default(),
            memo_out,
            0,
            Some(coin_value),
        ));

        //create output state
        let tvl_1: Commitment = Commitment::blinded(118u64);
        let tps_1: Commitment = Commitment::blinded(10u64);
        let s_var: ZString = ZString::from(tps_1.clone());
        let s_var_vec: Vec<ZString> = vec![s_var];
        // create Output state
        let out_state: OutputState = OutputState {
            nonce: 2,
            script_address: script_address.as_hex(),
            owner: add.as_hex(),
            commitment: tvl_1,
            state_variables: Some(s_var_vec),
            timebounds: 0,
        };

        let output: Vec<Output> = vec![coin_out, Output::state(OutputData::State(out_state))];
        // create Input State
        let tvl_0: Commitment = Commitment::blinded(150u64);
        let tps_0: Commitment = Commitment::blinded(10u64);
        let s_var: ZString = ZString::from(tps_0.clone());
        let in_state_var_vec: Vec<ZString> = vec![s_var];
        let temp_out_state = OutputState {
            nonce: 1,
            script_address: script_address.as_hex(),
            owner: add.as_hex(),
            commitment: tvl_0.clone(),
            state_variables: Some(in_state_var_vec),
            timebounds: 0,
        };

        let margin_difference = Commitment::blinded(5u64);
        let error_int = -zkvm::ScalarWitness::Integer(175000u64.into());
        let error_scalar: Scalar = error_int.into();
        let pay_string: Vec<ZString> = vec![
            ZString::from(margin_difference),
            ZString::from(error_scalar),
        ];
        // convert to input
        let input_state: Input = Input::state(InputData::state(
            Utxo::default(),
            temp_out_state.clone(),
            Some(pay_string),
            1,
        ));
        let input: Vec<Input> = vec![memo_in, input_state];
        //tx_date i.e., settle price
        let tx_data: zkvm::String = zkvm::String::from(Scalar::from(450u64));

        //cretae unsigned Tx with program proof
        let result = Prover::build_proof(
            correct_program,
            &input,
            &output,
            false,
            Some(tx_data.clone()),
        );
        println!("{:?}", result);
        let (prog_bytes, proof) = result.unwrap();
        let verify =
            Verifier::verify_r1cs_proof(&proof, &prog_bytes, &input, &output, false, Some(tx_data));
        println!("{:?}", verify);
    }
    #[test]
    fn test_lend_order_deposit_program_stack() {
        let correct_program = self::lend_order_deposit_program();
        println!("\n Program \n{:?}", correct_program);

        //create InputCoin and OutputMemo

        let mut rng = rand::thread_rng();
        let sk_in: quisquislib::ristretto::RistrettoSecretKey = SecretKey::random(&mut rng);
        let pk_in = RistrettoPublicKey::from_secret_key(&sk_in, &mut rng);
        let commit_in = ElGamalCommitment::generate_commitment(
            &pk_in,
            Scalar::random(&mut rng),
            Scalar::from(10u64),
        );
        let add: Address = Address::standard_address(Network::default(), pk_in.clone());
        let out_coin = OutputCoin {
            encrypt: commit_in,
            owner: add.as_hex(),
        };
        let in_data: InputData = InputData::coin(Utxo::default(), out_coin, 0);
        let coin_in: Input = Input::coin(in_data);

        //outputMemo
        let script_address =
            Address::script_address(Network::Mainnet, *Scalar::random(&mut rng).as_bytes());
        // create deposit amount
        let memo_deposit = Commitment::blinded(10000u64);
        //order size
        let pool_share_normalized = Commitment::blinded(10000u64);
        let data: Vec<ZString> = vec![ZString::from(pool_share_normalized)];
        let memo_out = OutputMemo {
            script_address: script_address.as_hex(),
            owner: add.as_hex(),
            commitment: memo_deposit,
            data: Some(data),
            timebounds: 0,
        };
        let out_data = OutputData::Memo(memo_out);
        let memo = Output::memo(out_data);

        //create output state
        let tvl_1: Commitment = Commitment::blinded(110000u64);
        let tps_1: Commitment = Commitment::blinded(10010u64);
        let s_var: ZString = ZString::from(tps_1.clone());
        let s_var_vec: Vec<ZString> = vec![s_var];
        // create Output state
        let out_state: OutputState = OutputState {
            nonce: 2,
            script_address: script_address.as_hex(),
            owner: add.as_hex(),
            commitment: tvl_1,
            state_variables: Some(s_var_vec),
            timebounds: 0,
        };

        let output: Vec<Output> = vec![memo, Output::state(OutputData::State(out_state))];
        // create Input State
        let tvl_0: Commitment = Commitment::blinded(100000u64);
        let tps_0: Commitment = Commitment::blinded(10u64);
        let s_var: ZString = ZString::from(tps_0.clone());
        let in_state_var_vec: Vec<ZString> = vec![s_var];
        let temp_out_state = OutputState {
            nonce: 1,
            script_address: script_address.as_hex(),
            owner: add.as_hex(),
            commitment: tvl_0.clone(),
            state_variables: Some(in_state_var_vec),
            timebounds: 0,
        };
        let error = Scalar::from(999900000u64);
        let err_string = vec![ZString::from(error)];
        // convert to input
        let input_state: Input = Input::state(InputData::state(
            Utxo::default(),
            temp_out_state.clone(),
            Some(err_string),
            1,
        ));
        let input: Vec<Input> = vec![coin_in, input_state];

        //cretae unsigned Tx with program proof
        let result = Prover::build_proof(correct_program, &input, &output, false, None);
        println!("{:?}", result);
        let (prog_bytes, proof) = result.unwrap();
        let verify = Verifier::verify_r1cs_proof(&proof, &prog_bytes, &input, &output, false, None);
        println!("{:?}", verify);
    }

    #[test]
    fn test_lend_settle_order_stack() {
        let correct_program = self::lend_order_settle_program();
        println!("\n Program \n{:?}", correct_program);

        //create InputMemo and OutputCoin

        let mut rng = rand::thread_rng();
        let sk_in: quisquislib::ristretto::RistrettoSecretKey = SecretKey::random(&mut rng);
        let pk_in = RistrettoPublicKey::from_secret_key(&sk_in, &mut rng);
        let commit_in = ElGamalCommitment::generate_commitment(
            &pk_in,
            Scalar::random(&mut rng),
            Scalar::from(219u64),
        );
        let add: Address = Address::standard_address(Network::default(), pk_in.clone());
        let out_coin = OutputCoin {
            encrypt: commit_in,
            owner: add.as_hex(),
        };
        let coin_out: Output = Output::coin(OutputData::coin(out_coin));

        //*****  InputMemo  *********/
        //****************************/
        let script_address =
            Address::script_address(Network::Mainnet, *Scalar::random(&mut rng).as_bytes());
        // Initial Deposit
        let commit_memo = Commitment::blinded(300u64);
        //Poolsize committed
        let pool_share = Commitment::blinded(245u64);

        let data: Vec<ZString> = vec![ZString::from(pool_share)];
        let memo_out = OutputMemo {
            script_address: script_address.as_hex(),
            owner: add.as_hex(),
            commitment: commit_memo,
            data: Some(data),
            timebounds: 0,
        };
        let withdraw: Commitment = Commitment::blinded(33u64); // Withdraw to be pushed back to the user
        let memo_in = Input::memo(InputData::memo(
            Utxo::default(),
            memo_out,
            0,
            Some(withdraw),
        ));

        //create output state
        let tvl_1: Commitment = Commitment::blinded(3194u64);
        let tps_1: Commitment = Commitment::blinded(23030u64);
        let s_var: ZString = ZString::from(tps_1.clone());
        let s_var_vec: Vec<ZString> = vec![s_var];
        // create Output state
        let out_state: OutputState = OutputState {
            nonce: 2,
            script_address: script_address.as_hex(),
            owner: add.as_hex(),
            commitment: tvl_1,
            state_variables: Some(s_var_vec),
            timebounds: 0,
        };

        let output: Vec<Output> = vec![coin_out, Output::state(OutputData::State(out_state))];
        // create Input State
        let tvl_0: Commitment = Commitment::blinded(3227u64);
        let tps_0: Commitment = Commitment::blinded(23275u64);
        let s_var: ZString = ZString::from(tps_0.clone());
        let in_state_var_vec: Vec<ZString> = vec![s_var];
        let temp_out_state = OutputState {
            nonce: 1,
            script_address: script_address.as_hex(),
            owner: add.as_hex(),
            commitment: tvl_0.clone(),
            state_variables: Some(in_state_var_vec),
            timebounds: 0,
        };
        let error_int = -zkvm::ScalarWitness::Integer(22540u64.into());
        let error_scalar: Scalar = error_int.into();
        let pay_string: Vec<ZString> = vec![ZString::from(error_scalar)];
        // convert to input
        let input_state: Input = Input::state(InputData::state(
            Utxo::default(),
            temp_out_state.clone(),
            Some(pay_string),
            1,
        ));
        let input: Vec<Input> = vec![memo_in, input_state];

        //cretae unsigned Tx with program proof
        let result = Prover::build_proof(correct_program, &input, &output, false, None);
        println!("{:?}", result);
        let (prog_bytes, proof) = result.unwrap();
        let verify = Verifier::verify_r1cs_proof(&proof, &prog_bytes, &input, &output, false, None);
        println!("{:?}", verify);
    }
}
