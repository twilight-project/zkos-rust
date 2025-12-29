[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue)](/LICENSE)
<!-- CI badge example; uncomment when you have CI -->
<!-- [![CI](https://github.com/twilight-project/zkos-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/twilight-project/zkos-rust/actions/workflows/ci.yml) -->

**Status:** experimental ⛏️ – APIs may break before v1.0.

> **Origin:** Portions adapted from the  
> [`stellar/slingshot`](https://github.com/stellar/slingshot/tree/main/spacesuit) project (Apache-2.0).

# Rangeproof: Programmable Range Proof Construction using Bulletproofs R1CS

A pure Rust implementation of range proofs based on the [Bulletproofs](https://crypto.stanford.edu/bulletproofs/) zero-knowledge proof system, enabling confidential value verification without revealing actual amounts.

## Overview

This crate provides efficient range proof construction using Bulletproofs' Rank-1 Constraint System (R1CS) framework. It allows proving that a committed value lies within a specified range [0, 2^n) without revealing the actual value, making it ideal for confidential transactions and privacy-preserving applications.

## Features

- **🔒 Range Proofs**: Prove values are in range [0, 2^n) for any n ≤ 64
- **⚡ R1CS Integration**: Built on Bulletproofs R1CS constraint system
- **🔄 Signed Integer Support**: Handle both positive and negative values with overflow protection
- **🔐 Pedersen Commitments**: Secure value commitment without revealing secrets
- **🛡️ Constant-time Operations**: Side-channel resistant implementations
- **🔄 Batch Verification**: Efficient verification of multiple proofs

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rangeproof = { git = "https://github.com/twilight-project/zkos-rust", package = "rangeproof" }
bulletproofs = { git = "https://github.com/dalek-cryptography/bulletproofs", branch = "develop" }
curve25519-dalek = "3"
merlin = "2"
```

### Basic Usage

```rust
use rangeproof::{range_proof, BitRange, SignedInteger};
use bulletproofs::r1cs::{Prover, Verifier};
use bulletproofs::{BulletproofGens, PedersenGens};
use merlin::Transcript;
use curve25519_dalek::scalar::Scalar;
use rand::thread_rng;

// Create generators
let pc_gens = PedersenGens::default();
let bp_gens = BulletproofGens::new(128, 1);

// Prover creates a range proof
let (proof, commitment) = {
    let mut prover_transcript = Transcript::new(b"RangeProofExample");
    let mut rng = thread_rng();
    let mut prover = Prover::new(&pc_gens, &mut prover_transcript);

    // Commit to a value in range [0, 2^32)
    let value = SignedInteger::from(12345u64);
    let (com, var) = prover.commit(value.into(), Scalar::random(&mut rng));
    
    // Create range proof for 32-bit range
    let bit_range = BitRange::new(32).unwrap();
    range_proof(&mut prover, var.into(), Some(value), bit_range).unwrap();

    let proof = prover.prove(&bp_gens).unwrap();
    (proof, com)
};

// Verifier checks the range proof
let mut verifier_transcript = Transcript::new(b"RangeProofExample");
let mut verifier = Verifier::new(&mut verifier_transcript);

let var = verifier.commit(commitment);
let bit_range = BitRange::new(32).unwrap();
range_proof(&mut verifier, var.into(), None, bit_range).unwrap();

assert!(verifier.verify(&proof, &pc_gens, &bp_gens).is_ok());
```

### Advanced Usage

```rust
use rangeproof::{Value, ProverCommittable, VerifierCommittable};
use rangeproof::{range_proof, BitRange};

// Create a value with quantity and flavor
let value = Value {
    q: 100u64.into(),  // quantity
    f: Scalar::from(1u64),  // flavor (asset type)
};

// Commit and create range proof
let (commitment, allocated) = value.commit(&mut prover, &mut rng);
let bit_range = BitRange::new(16).unwrap(); // 16-bit range [0, 65536)
range_proof(&mut prover, allocated.q.into(), Some(value.q), bit_range).unwrap();
```

## API Reference

### Core Types

- **`BitRange`**: Specifies bit width for range proofs (1-64 bits)
- **`SignedInteger`**: Signed integers with overflow protection
- **`Value`**: Secret value with quantity and flavor
- **`CommittedValue`**: Pedersen commitments to a value
- **`AllocatedValue`**: R1CS variables representing a value

### Key Functions

- **`range_proof()`**: Create R1CS constraints for range proofs
- **`BitRange::new()`**: Create a new bit range specification
- **`SignedInteger::from()`**: Convert u64 to signed integer

### Traits

- **`ProverCommittable`**: Commit values to prover's constraint system
- **`VerifierCommittable`**: Commit values to verifier's constraint system

## Algorithm

The range proof works by:

1. **Binary Decomposition**: Express value v as sum of bits: v = Σ(b_i * 2^i)
2. **Bit Constraints**: Ensure each bit b_i is either 0 or 1
3. **Range Enforcement**: Prove v equals the sum of its bits

For each bit position i, we create:
- Multiplier: a_i * b_i = 0 (ensures one of a_i, b_i is zero)
- Constraint: a_i = 1 - b_i (ensures a_i, b_i are complementary)
- Contribution: v -= b_i * 2^i (builds the sum)

## Security

This implementation follows cryptographic best practices:

- ✅ Uses Bulletproofs for efficient zero-knowledge proofs
- ✅ Implements proper domain separation with Merlin transcripts
- ✅ Provides constant-time operations where possible
- ✅ Built on the secure Ristretto255 curve
- ✅ Includes comprehensive test coverage

## Performance

- **Proof Size**: Logarithmic in the bit range
- **Verification Time**: Constant-time verification
- **Memory Usage**: Minimal overhead for constraint systems

## Minimum Supported Rust Version

Rust **1.70** or newer.

## Documentation

create documentation using 
```
cargo doc --open
```
- [Specification](spec.md)


## Contributing

Contributions are welcome! Please ensure all code is properly documented and tested.

## License & Attribution

Licensed under [`Apache-2.0`](../../LICENSE).  
Portions derived from Stellar's **Slingshot** project (Apache-2.0).

## References

- [Bulletproofs Paper](https://crypto.stanford.edu/bulletproofs/)
- [Bulletproofs Implementation](https://github.com/dalek-cryptography/bulletproofs)
- [Stellar Slingshot](https://github.com/stellar/slingshot)
- [Ristretto255](https://ristretto.group/)