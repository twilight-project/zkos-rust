#![allow(non_snake_case)]
#![deny(missing_docs)]
//! Definition of the proof struct.

use bulletproofs::{PedersenGens, RangeProof};
use curve25519_dalek::scalar::Scalar;
use quisquislib::{
    accounts::prover::{Prover, SigmaProof},
    accounts::verifier::Verifier,
    accounts::Account,
    keys::PublicKey,
    pedersen::vectorpedersen::VectorPedersenGens,
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
    shuffle::{shuffle::ROWS, Shuffle, ShuffleProof, ShuffleStatement},
};

use serde::{Deserialize, Serialize};

/// Used in Dark Transaction and Quisquis Tx
/// Store Dark Tx Proof
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DarkTxProof {
    pub(super) delta_accounts: Vec<Account>,
    pub(super) epsilon_accounts: Vec<Account>,
    pub(super) updated_delta_accounts: Vec<Account>,
    pub(super) delta_dleq: SigmaProof,
    pub(super) updated_sender_epsilon_accounts: Vec<Account>,
    pub(super) sender_account_dleq: SigmaProof,
    pub(super) range_proof: Vec<RangeProof>,
    // Proof only needed for Dark Tx. None in case of Quisquis Tx
    pub(super) updated_output_proof: Option<SigmaProof>,
    //ONLY FOR TESTING PURPOSES
    pub(super) receivers_count: usize, //SHOULD BE REMOVED LATER
}

///
/// Store the shuffle proof and missing info for QuisQuis TX
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShuffleTxProof {
    pub(super) input_dash_accounts: Vec<Account>, //Updated input accounts
    pub(super) input_shuffle_proof: ShuffleProof,
    pub(super) input_shuffle_statement: ShuffleStatement,
    pub(super) updated_delta_dlog: SigmaProof,
    pub(super) zero_balance_dlog: Option<SigmaProof>,
    // pub(super) updated_delta_accounts: Vec<Account>,
    pub(super) output_shuffle_proof: ShuffleProof,
    pub(super) output_shuffle_statement: ShuffleStatement,
}
impl DarkTxProof {
    /// Serializes the proof into a byte array
    ///
    /// # Layoutec<>
    ///
    /// The layout of the darktx proof encoding is:
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(64);
        //DESIGN BYTE STREAM FOR PROOF CATERING FOR VECTORS
        let (pk, _enc) = self.delta_accounts[0].get_account();
        buf.extend_from_slice(&pk.as_bytes());
        // buf.extend_from_slice(self.);
        // buf.extend_from_slice(self.);
        // buf.extend_from_slice(self.);
        // buf.extend_from_slice(self.);
        // buf.extend_from_slice(self.);
        // buf.extend_from_slice(self.);
        // buf.extend_from_slice(self.);
        // buf.extend(self.);
        buf
    }

    /// Deserializes the proof from a byte slice.
    ///
    /// Returns an error if the byte slice cannot be parsed into a `DarkTxProof`.
    pub fn from_bytes(_slice: &[u8]) /*-> Result<DarkTxProof, &'static str >*/
    {
        // if slice.len() < 1 {
        //     return Err("DarkTxProofError::FormatError");
        // }
        // let version = slice[0];
        // let mut slice = &slice[1..];

        // if slice.len() % 32 != 0 {
        //     return Err(R1CSError::FormatError);
        // }

        // let minlength = match version {
        //     ONE_PHASE_COMMITMENTS => 11 * 32,
        //     TWO_PHASE_COMMITMENTS => 14 * 32,
        //     _ => return Err(R1CSError::FormatError),
        // };

        // if slice.len() < minlength {
        //     return Err(R1CSError::FormatError);
        // }

        // // This macro takes care of counting bytes in the slice
        // macro_rules! read32 {
        //     () => {{
        //         let tmp = util::read32(slice);
        //         slice = &slice[32..];
        //         tmp
        //     }};
        // }

        // let A_I1 = CompressedRistretto(read32!());
        // let A_O1 = CompressedRistretto(read32!());
        // let S1 = CompressedRistretto(read32!());
        // let T_5 = CompressedRistretto(read32!());
        // let T_6 = CompressedRistretto(read32!());
        // let t_x = Scalar::from_canonical_bytes(read32!()).ok_or(R1CSError::FormatError)?;
        // let t_x_blinding = Scalar::from_canonical_bytes(read32!()).ok_or(R1CSError::FormatError)?;
        // let e_blinding = Scalar::from_canonical_bytes(read32!()).ok_or(R1CSError::FormatError)?;

        // // XXX: IPPProof from_bytes gives ProofError.
        // let ipp_proof = InnerProductProof::from_bytes(slice).map_err(|_| R1CSError::FormatError)?;

        // Ok(R1CSProof {
        //     ipp_proof,
        // })
    }
    ///
    /// create Dark transaction proof for Prover
    pub fn create_dark_tx_proof(
        prover: &mut quisquislib::accounts::Prover,
        value_vector: &[Scalar],
        delta_accounts: &[Account],
        epsilon_accounts: &[Account],
        delta_rscalar: &[Scalar],
        sender_updated_delta_account: &[Account],
        sender_updated_balance: &[u64],
        reciever_updated_balance: &[u64],
        sender_sk: &[RistrettoSecretKey],
        senders_count: usize,
        receivers_count: usize,
        base_pk: RistrettoPublicKey,
        update_outputs_statement: Option<(&[Account], &[Account], Scalar, Scalar)>, //updated_outputs: Option<&[Account]>,
                                                                                    //delta_updated_accounts: Option<&[Account]>,
                                                                                    //updated_out_pk_rscalar: Option<Scalar>,
                                                                                    //updated_out_comm_rscalar: Option<Scalar>,
    ) -> DarkTxProof {
        //enerate DLEQ proof for same balance value committed based on Epsilon and delta account
        let delta_dleq = Prover::verify_delta_compact_prover(
            &delta_accounts,
            &epsilon_accounts,
            &delta_rscalar,
            &value_vector,
            prover,
        );
        // let updated_delta_accounts_sender_slice = &delta_accounts[..senders_count];
        let (updated_sender_epsilon_accounts, epsilon_sender_rscalar_vector, sender_account_dleq) =
            Prover::verify_account_prover(
                &sender_updated_delta_account,
                sender_updated_balance,
                sender_sk,
                prover,
                base_pk,
            );
        //create rangeproof on senders and receivers
        //create sender_final + reciver balance vector
        let bl_rp_vector: Vec<u64> = sender_updated_balance
            .into_iter()
            .cloned()
            .chain(reciever_updated_balance.iter().cloned())
            .collect();
        //create rscalar vector for sender and reciver epsilon accounts.
        //extract rscalars for reciever epsilon accounts
        let rec_rscalars_slice = &delta_rscalar[senders_count..senders_count + receivers_count];
        //receiver rscalars are extracted from rscalars vector returned in create_delta_and_epsilon_accounts

        let scalars_bp_vector: Vec<Scalar> = epsilon_sender_rscalar_vector
            .iter()
            .cloned()
            .chain(rec_rscalars_slice.iter().cloned())
            .collect();
        //Generate range proof over sender/reciever account values. i.,e balance >=0 for all
        let range_proof =
            prover.verify_non_negative_sender_reciver_prover(&bl_rp_vector, &scalars_bp_vector);

        // check if is is dark or quisquis tx
        match update_outputs_statement {
            Some((
                updated_outputs,
                delta_updated_accounts,
                updated_out_pk_rscalar,
                updated_out_comm_rscalar,
            )) => {
                let updated_output_proof = Prover::verify_update_account_dark_tx_prover(
                    delta_updated_accounts,
                    updated_outputs,
                    updated_out_pk_rscalar,
                    updated_out_comm_rscalar,
                    prover,
                );
                DarkTxProof {
                    delta_accounts: delta_accounts.to_vec(),
                    epsilon_accounts: epsilon_accounts.to_vec(),
                    delta_dleq,
                    updated_sender_epsilon_accounts,
                    sender_account_dleq,
                    range_proof,
                    updated_output_proof: Some(updated_output_proof),
                    receivers_count,
                }
            }
            None => DarkTxProof {
                delta_accounts: delta_accounts.to_vec(),
                epsilon_accounts: epsilon_accounts.to_vec(),
                delta_dleq,
                updated_sender_epsilon_accounts,
                sender_account_dleq,
                range_proof,
                updated_output_proof: None,
                receivers_count,
            },
        }

        // if updated_outputs.is_some() && delta_updated_accounts.is_some() {
        //     // Dark TX
        //     // if dark tx then create updated output proof

        //     let updated_output_proof = prover.verify_update_account_dark_tx_prover(
        //         delta_updated_accounts,
        //         updated_outputs,
        //         updated_out_pk_rscalar,
        //         updated_out_comm_rscalar,
        //     );
        //     DarkTxProof {
        //         delta_accounts: delta_accounts.to_vec(),
        //         epsilon_accounts: epsilon_accounts.to_vec(),
        //         delta_dleq,
        //         updated_sender_epsilon_accounts,
        //         sender_account_dleq,
        //         range_proof,
        //         updated_output_proof: Some(updated_output_proof),
        //         receivers_count,
        //     }
        // } else {
        //     // QuisQuis TX
        //     DarkTxProof {
        //         delta_accounts: delta_accounts.to_vec(),
        //         epsilon_accounts: epsilon_accounts.to_vec(),
        //         delta_dleq,
        //         updated_sender_epsilon_accounts,
        //         sender_account_dleq,
        //         range_proof,
        //         updated_output_proof: None,
        //         receivers_count,
        //     }
        // }
    }

    ///
    /// Verify the DarkTx Proof
    pub fn verify(
        &self,
        verifier: &mut Verifier,
        updated_input: &Vec<Account>, //Updated_input = input in case of Dark Tx.
        //     updated_delta_account_sender: &[Account],
        updated_delta_account: &Vec<Account>, //updated_delta = Output in the cas eof dark Tx
        update_output_proof_statement: Option<(&[Account], &[Account])>,
    ) -> Result<(), &'static str> {
        let base_pk = RistrettoPublicKey::generate_base_pk();
        //identity check function to verify the construction of epsilon accounts using correct rscalars

        Verifier::verify_delta_identity_check(&self.epsilon_accounts)?;
        // Verify Update Delta.
        // checks if pk_input' = pk_delta =pk_output'
        // checks if com_output' = com_input' * com_delta

        Account::verify_delta_update(updated_delta_account, &self.delta_accounts, updated_input)?;
        let delta_dleq = self.delta_dleq.clone();
        let (zv_vector, zr1_vector, zr2_vector, x) = delta_dleq.get_dleq();
        // verify dleq proof
        Verifier::verify_delta_compact_verifier(
            &self.delta_accounts,
            &self.epsilon_accounts,
            &zv_vector,
            &zr1_vector,
            &zr2_vector,
            &x,
            verifier,
        )?;

        let (zv_sender_acc, zsk_sender_acc, zr_sender_acc, x_sender_acc) =
            self.sender_account_dleq.clone().get_dleq();

        let senders_count: usize = self.updated_sender_epsilon_accounts.len();
        let updated_delta_account_sender = &updated_delta_account[..senders_count];
        //verify sender account signature and remaining balance.
        let senders_count: usize = self.updated_sender_epsilon_accounts.len();
        let updated_delta_account_sender = &updated_delta_account[..senders_count];
        Verifier::verify_account_verifier_bulletproof(
            &updated_delta_account_sender,
            &self.updated_sender_epsilon_accounts,
            &base_pk,
            &zv_sender_acc,
            &zsk_sender_acc,
            &zr_sender_acc,
            x_sender_acc,
            verifier,
        )?;
        // let senders_count: usize = updated_delta_account_sender.len();
        //let total_count : usize = self.epsilon_accounts.len();
        //Verify the bulletproofs

        let reciever_epsilon_accounts_slice =
            &self.epsilon_accounts[senders_count..senders_count + self.receivers_count].to_vec();
        //prepare epsilon account vector for sender + reciver
        let bp_epsilon_vec: Vec<Account> = self
            .updated_sender_epsilon_accounts
            .iter()
            .cloned()
            .chain(reciever_epsilon_accounts_slice.iter().cloned())
            .collect();

        //TODO
        //check if batched bulletproof or vector proof
        //CHECK IF RANGE_PROOF LENGTH IS A MULTIPLE OF 2
        //only test in the case of 1
        if self.range_proof.len() == 1 {
            assert!(verifier
                .verify_non_negative_sender_receiver_bulletproof_batch_verifier(
                    &bp_epsilon_vec,
                    &self.range_proof[0],
                )
                .is_ok())
        } else {
            assert!(verifier
                .verify_non_negative_sender_receiver_bulletproof_vector_verifier(
                    &bp_epsilon_vec,
                    &self.range_proof,
                )
                .is_ok())
        }
        // check if verifying the proof for Dark Tx or Quisquis Tx
        match update_output_proof_statement {
            Some((updated_outputs, delta_updated_accounts)) => {
                // Dark TX
                // verify the updated output proof
                let updated_output_proof = self.updated_output_proof.clone().unwrap();
                let (z_vector, x) = updated_output_proof.get_dlog();
                let check = Verifier::verify_update_account_dark_tx_verifier(
                    delta_updated_accounts,
                    updated_outputs,
                    &z_vector,
                    &x,
                    verifier,
                );
                match check {
                    Ok(()) => Ok(()),
                    Err(e) => Err("Updated Output Account: DLOG Proof Verify: Failed"),
                }
            }
            None => {
                Ok(())
                // Quisquis TX
                // do nothing
            }
        }

        //  Ok(())
    }
}

// impl Serialize for DarkTxProof {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         serializer.serialize_bytes(&self.to_bytes()[..])
//     }
// }

//impl<'de> Deserialize<'de> for DarkTxProof {
//    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//  where
//    D: Deserializer<'de>,
// {
//     struct R1CSProofVisitor;

//     impl<'de> Visitor<'de> for R1CSProofVisitor {
//         type Value = R1CSProof;

//         fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
//             formatter.write_str("a valid R1CSProof")
//         }

//         fn visit_bytes<E>(self, v: &[u8]) -> Result<R1CSProof, E>
//         where
//             E: serde::de::Error,
//         {
//             // Using Error::custom requires T: Display, which our error
//             // type only implements when it implements std::error::Error.
//             #[cfg(feature = "std")]
//             return R1CSProof::from_bytes(v).map_err(serde::de::Error::custom);
//             // In no-std contexts, drop the error message.
//             #[cfg(not(feature = "std"))]
//             return R1CSProof::from_bytes(v)
//                 .map_err(|_| serde::de::Error::custom("deserialization error"));
//         }
//     }

//     deserializer.deserialize_bytes(R1CSProofVisitor)
//  }
//}

impl ShuffleTxProof {
    ///
    ///create shuffle proof
    pub fn create_shuffle_proof(
        prover: &mut quisquislib::accounts::Prover,
        updated_accounts_slice: &[Account],
        updated_delta_accounts_slice: &[Account],
        rscalars_slice: &[Scalar],
        input_anonymity_account_slice: &[Account],
        anonymity_comm_scalar: &[Scalar],
        input_shuffle: &Shuffle,
        output_shuffle: &Shuffle,
    ) -> ShuffleTxProof {
        //Step 2. create proof for Input shuffle
        //generate Xcomit generator points of length m+1
        let xpc_gens = VectorPedersenGens::new(ROWS + 1);
        // Prepare the constraint system
        let pc_gens = PedersenGens::default();

        let (input_shuffle_proof, input_shuffle_statement) =
            ShuffleProof::create_shuffle_proof(prover, input_shuffle, &pc_gens, &xpc_gens);

        //generate DLOG proof on Anonymity accounts in Updated Delta accounts
        let updated_delta_dlog = Prover::verify_update_account_prover(
            &updated_accounts_slice,
            &updated_delta_accounts_slice,
            &rscalars_slice,
            prover,
        );

        //if annoymity accounts are created on the fly.
        //create zero balance proof for all the anonymity accounts
        /* NEEDS SUPPORT OF UTXO SET TO DETERMINE THE CORRECT COMBINATION OF ANONYMITY INPUT
         ** All inputs with no UtxoId will be gathered as new anonymity set and a zero balance proof will have to be provided
         ** since we are doing compact batch proof we need to collect the anonymity set before we can run the proof
        let zero_balance_dlog = Prover::zero_balance_account_prover(
            &input_anonymity_account_slice,
            &input_anonymity_account_slice,
            &anonymity_comm_scalar,
            prover,
        );*/

        let (output_shuffle_proof, output_shuffle_statement) =
            ShuffleProof::create_shuffle_proof(prover, output_shuffle, &pc_gens, &xpc_gens);

        ShuffleTxProof {
            input_dash_accounts: input_shuffle.get_outputs_vector(),

            input_shuffle_proof,
            input_shuffle_statement,
            updated_delta_dlog,
            zero_balance_dlog: None,
            updated_delta_accounts: output_shuffle.get_inputs_vector(),
            output_shuffle_proof,
            output_shuffle_statement,
        }
    }
    ///
    /// verify the shuffle proof
    pub fn verify(
        &self,
        verifier: &mut Verifier,
        input_accounts: &[Account],
        output_accounts: &[Account],
        anonymity_index: usize,
    ) -> Result<(), &'static str> {
        //Recreate Pedersen Commitment (PC) Genarater and Xtended PC (XPC) Gens
        //generate Xcomit generator points of length m+1
        let xpc_gens = VectorPedersenGens::new(ROWS + 1);
        // Prepare the constraint system
        let pc_gens = PedersenGens::default();

        //verify the input shuffle
        self.input_shuffle_proof.verify(
            verifier,
            &self.input_shuffle_statement,
            &input_accounts,
            &self.input_dash_accounts,
            &pc_gens,
            &xpc_gens,
        )?;

        // Verify DLOG proof on Anonymity accounts in Updated Delta accounts
        let (z_vector, x) = self.updated_delta_dlog.clone().get_dlog();
        let updated_accounts_slice = &self.input_dash_accounts[anonymity_index..9];
        let updated_delta_accounts_slice = &self.updated_delta_accounts[anonymity_index..9];
        //verify dlog proof
        println!("BEFORE Anony index {:?}", anonymity_index);
        Verifier::verify_update_account_verifier(
            &updated_accounts_slice,
            &updated_delta_accounts_slice,
            &z_vector,
            &x,
            verifier,
        )?;
        /* NEEDS SUPPORT OF UTXO SET TO DETERMINE THE CORRECT COMBINATION OF ANONYMITY INPUT
         //Step 7. if annoymity accounts are created on the fly.
         //create zero balance proof for all the anonymity accounts
         if self.zero_balance_dlog.is_some(){
        // let input_anonymity_account_slice =
         let (z_zero_balance, x_zero_balance) = self.zero_balance_dlog.clone().unwrap().get_dlog();
         println!("In verifier");
         //verify zero balance proof for anonymity set
         Verifier::zero_balance_account_verifier(
             &updated_accounts_slice,
             &z_zero_balance,
             x_zero_balance,
             verifier,
         )?;*/

        //verify the output shuffle
        self.output_shuffle_proof.verify(
            verifier,
            &self.output_shuffle_statement,
            &self.updated_delta_accounts,
            output_accounts,
            &pc_gens,
            &xpc_gens,
        )?;

        Ok(())
    }
    /// Serializes the proof into a byte array
    ///
    /// # Layoutec<>
    ///
    /// The layout of the darktx proof encoding is:
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(64);
        //DESIGN BYTE STREAM FOR PROOF CATERING FOR VECTORS
        let (pk, _enc) = self.updated_delta_accounts[0].get_account();
        buf.extend_from_slice(&pk.as_bytes());
        // buf.extend_from_slice(self.);
        // buf.extend_from_slice(self.);
        // buf.extend(self.);
        buf
    }

    /// Deserializes the proof from a byte slice.
    ///
    /// Returns an error if the byte slice cannot be parsed into a `DarkTxProof`.
    pub fn from_bytes(_slice: &[u8]) /*-> Result<DarkTxProof, &'static str >*/
    {
        // let e_blinding = Scalar::from_canonical_bytes(read32!()).ok_or(R1CSError::FormatError)?;

        // // XXX: IPPProof from_bytes gives ProofError.
        // let ipp_proof = InnerProductProof::from_bytes(slice).map_err(|_| R1CSError::FormatError)?;

        // Ok(R1CSProof {
        //     ipp_proof,
        // })
    }
}

// impl Serialize for ShuffleTxProof {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         serializer.serialize_bytes(&self.to_bytes()[..])
//     }
// }

//impl<'de> Deserialize<'de> for ShuffleTxProof {
//  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//  where
//    D: Deserializer<'de>,
// {
// struct R1CSProofVisitor;

// impl<'de> Visitor<'de> for R1CSProofVisitor {
//     type Value = R1CSProof;

//     fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
//         formatter.write_str("a valid R1CSProof")
//     }

//     fn visit_bytes<E>(self, v: &[u8]) -> Result<R1CSProof, E>
//     where
//         E: serde::de::Error,
//     {
//         // Using Error::custom requires T: Display, which our error
//         // type only implements when it implements std::error::Error.
//         #[cfg(feature = "std")]
//         return R1CSProof::from_bytes(v).map_err(serde::de::Error::custom);
//         // In no-std contexts, drop the error message.
//         #[cfg(not(feature = "std"))]
//         return R1CSProof::from_bytes(v)
//             .map_err(|_| serde::de::Error::custom("deserialization err or"));
//     }
// }

// deserializer.deserialize_bytes(R1CSProofVisitor)
// }
//}

// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use merlin::Transcript;
    #[test]
    fn create_dark_tx_proof_test() {
        let base_pk = RistrettoPublicKey::generate_base_pk();

        let value_vector: Vec<Scalar> = vec![
            -Scalar::from(-(-5i64) as u64),
            -Scalar::from(-(-3i64) as u64),
            -Scalar::from(-(-5i64) as u64),
            -Scalar::from(-(-3i64) as u64),
            Scalar::from(5u64),
            Scalar::from(3u64),
        ];
        let mut updated_accounts: Vec<Account> = Vec::new();
        let mut sender_sk: Vec<RistrettoSecretKey> = Vec::new();

        for i in 0..4 {
            let (updated_account, sk) =
                Account::generate_random_account_with_value(Scalar::from(10u64));

            updated_accounts.push(updated_account);

            // lets save the first and second sk as sender's sk as we discard the rest
            if i == 0 || i == 1 {
                sender_sk.push(sk);
            }
        }

        let (delta_accounts, epsilon_accounts, delta_rscalar_vector) =
            Account::create_delta_and_epsilon_accounts(&updated_accounts, &value_vector, base_pk);

        let updated_delta_accounts =
            Account::update_delta_accounts(&updated_accounts, &delta_accounts).unwrap();

        // balance that we want to prove should be sender balance - the balance user is trying to send
        let sender_updated_balance: Vec<u64> = vec![5u64, 7u64];
        let reciever_updated_balance: Vec<u64> = vec![5u64, 3u64];

        let base_pk = RistrettoPublicKey::generate_base_pk();
        //get sender_updated delta accounts for verify account proof
        let sender_updated_delta_account = &updated_delta_accounts[..2];
        //create DarkTx Prover merlin transcript
        let mut transcript = Transcript::new(b"TxProof");
        let mut prover = Prover::new(b"DarkTx", &mut transcript);
        // create proof for QuisQuis variant
        // let dark_tx_proof = DarkTxProof::create_dark_tx_proof(
        //     &mut prover,
        //     &value_vector,
        //     &delta_accounts,
        //     &epsilon_accounts,
        //     &delta_rscalar_vector,
        //     &sender_updated_delta_account,
        //     &sender_updated_balance,
        //     &reciever_updated_balance,
        //     &sender_sk,
        //     2,
        //     2,
        //     base_pk,
        //     None,
        // );

        // update the delta_updated_accounts to create output for dark tx
        let pk_update_scalar = Scalar::random(&mut rand::rngs::OsRng);
        let comm_update_scalar = Scalar::random(&mut rand::rngs::OsRng);

        let outputs = updated_delta_accounts
            .iter()
            .map(|account| {
                Account::update_account(
                    *account,
                    Scalar::zero(),
                    pk_update_scalar,
                    comm_update_scalar,
                )
            })
            .collect::<Vec<Account>>();
        // create proof for Dark Tx variant
        let dark_tx_proof = DarkTxProof::create_dark_tx_proof(
            &mut prover,
            &value_vector,
            &delta_accounts,
            &epsilon_accounts,
            &delta_rscalar_vector,
            &sender_updated_delta_account,
            &sender_updated_balance,
            &reciever_updated_balance,
            &sender_sk,
            2,
            2,
            base_pk,
            Some((
                &outputs,
                &updated_delta_accounts,
                pk_update_scalar,
                comm_update_scalar,
            )),
        );

        //verify the proof

        //create DarkTx Prover merlin transcript
        let mut v_transcript = Transcript::new(b"TxProof");
        let mut verifier = Verifier::new(b"DarkTx", &mut v_transcript);

        //create updated senders delta account slice
        // let updated_senders_delta_account = &dark_tx_proof.delta_accounts[..2];

        // Standard verification in case of Quisquis Tx
        // let verify = dark_tx_proof.verify(
        //     &mut verifier,
        //     &updated_accounts,
        //     &updated_delta_accounts,
        //     None,
        // );
        // Veification in case of Dark Tx

        let verify = dark_tx_proof.verify(
            &mut verifier,
            &updated_accounts,
            &updated_delta_accounts,
            Some((&outputs, &updated_delta_accounts)),
        );
        println!("{:?}", verify);
        assert!(verify.is_ok())
    }
    #[test]
    fn create_scalar_test() {
        let iin: i64 = -5;
        let uin: i64 = 5;

        println!(
            "0 - Scalar {:#?}",
            Scalar::zero() - Scalar::from(-iin as u64)
        );
        println!(" -Scalar {:#?}", -Scalar::from(iin as u64));
        println!("Scalar {:#?}", Scalar::from(uin as u64));
        println!("Scalar {:#?}", -Scalar::from(uin as u64));
        println!("Scalar {:#?}", Scalar::from(-iin as u64));
    }
}
