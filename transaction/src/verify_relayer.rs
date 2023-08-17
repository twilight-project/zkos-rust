#![allow(non_snake_case)]
//#![deny(missing_docs)]

use address::{Address, Network};
use merlin::Transcript;
use zkvm::zkos_types::{Input, InputData, Output, OutputCoin, OutputData, Utxo, Witness};

use serde::{Deserialize, Serialize};

use curve25519_dalek::scalar::Scalar;
use quisquislib::{
    accounts::prover::{Prover, SigmaProof},
    accounts::verifier::Verifier,
    accounts::Account,
    keys::PublicKey,
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
};