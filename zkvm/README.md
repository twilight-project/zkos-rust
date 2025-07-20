[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue)](/LICENSE)
<!-- CI badge example; uncomment when you have CI -->
<!-- [![CI](https://github.com/twilight-project/zkos-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/twilight-project/zkos-rust/actions/workflows/ci.yml) -->

**Status:** experimental ⛏️ – APIs may break before v1.0.

> **Origin:** Portions adapted from the  
> [`stellar/slingshot`](https://github.com/stellar/slingshot/tree/main/zkvm) project (Apache-2.0).

# ZkVM: Zero-Knowledge Virtual Machine for State Transition Proofs

A stack-based virtual machine for creating and verifying zero-knowledge proofs of state transitions in the ZkOS blockchain system. ZkVM provides a secure, programmable environment for executing smart contracts with full cryptographic verification.

## Overview

ZkVM is a specialized virtual machine that enables the creation and verification of zero-knowledge proofs for state transitions. Unlike traditional VMs that execute transactions directly, ZkVM focuses on proving the correctness of state changes using Bulletproofs R1CS constraint systems.

### Key Features

- **🔒 Zero-Knowledge Proofs**: Generate and verify proofs without revealing state details
- **📚 Stack-Based Architecture**: Simple, secure stack-based instruction set
- **⚡ R1CS Integration**: Built on Bulletproofs Rank-1 Constraint System
- **🛡️ Cryptographic Security**: All operations backed by cryptographic proofs
- **🔄 State Transition Verification**: Prove correctness of state changes
- **📦 Smart Contract Support**: Execute and verify smart contract logic

## Architecture

### Core Components

#### **Stack-Based VM**
- **Instruction Set**: Arithmetic, logical, and cryptographic operations
- **Stack Management**: Push, pop, duplicate, and roll operations
- **Memory Model**: Secure stack-based memory management

#### **R1CS Constraint System**
- **Constraint Generation**: Automatic constraint creation for all operations
- **Proof Generation**: Efficient zero-knowledge proof creation
- **Proof Verification**: Constant-time proof verification

#### **State Management**
- **Input/Output Types**: Coin, Memo, and State data types
- **Witness System**: Cryptographic proofs for state transitions
- **UTXO Integration**: Unspent transaction output management

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
zkvm = { git = "https://github.com/twilight-project/zkos-rust", package = "zkvm" }
bulletproofs = { git = "https://github.com/dalek-cryptography/bulletproofs", branch = "develop" }
curve25519-dalek = "3"
merlin = "2"
```

### Basic Usage

#### Creating a State Transition Proof

```rust
use zkvm::{VMScript, Prover, Verifier, Program, Input, Output};
use bulletproofs::r1cs::{Prover as R1CSProver, Verifier as R1CSVerifier};
use bulletproofs::{BulletproofGens, PedersenGens};
use merlin::Transcript;

// Create generators
let pc_gens = PedersenGens::default();
let bp_gens = BulletproofGens::new(128, 1);

// Define inputs and outputs
let inputs = vec![/* input data */];
let outputs = vec![/* output data */];

// Create program bytecode
let program = Program::from_instructions(vec![
    // Your program instructions here
]);

// Generate proof
let (proof, program_bytes) = {
    let mut prover_transcript = Transcript::new(b"StateTransition");
    let mut prover = R1CSProver::new(&pc_gens, &mut prover_transcript);
    
    let mut vm = VMScript::new(
        program.clone(),
        &mut prover,
        &inputs,
        &outputs,
        None, // tx_data
    );
    
    vm.run()?;
    let proof = prover.prove(&bp_gens)?;
    (proof, program.to_bytes())
};

// Verify proof
let mut verifier_transcript = Transcript::new(b"StateTransition");
let mut verifier = R1CSVerifier::new(&mut verifier_transcript);

let mut vm = VMScript::new(
    Program::from_bytes(&program_bytes)?,
    &mut verifier,
    &inputs,
    &outputs,
    None,
);

vm.run()?;
assert!(verifier.verify(&proof, &pc_gens, &bp_gens).is_ok());
```

#### Working with Different Data Types

```rust
use zkvm::zkos_types::{Input, Output, IOType, InputData, OutputData};

// Coin input/output
let coin_input = Input::coin(InputData::coin(utxo, output_coin, witness_index));
let coin_output = Output::coin(OutputData::coin(output_coin));

// Memo input/output
let memo_input = Input::memo(InputData::memo(utxo, output_memo, witness_index, coin_value));
let memo_output = Output::memo(OutputData::memo(output_memo));

// State input/output
let state_input = Input::state(InputData::state(utxo, output_state, script_data, witness_index));
let state_output = Output::state(OutputData::state(output_state));
```

## API Reference

### Core Types

#### **VM Components**
- **`VMScript`**: Main VM for script execution and proof generation
- **`Prover`**: R1CS prover for generating zero-knowledge proofs
- **`Verifier`**: R1CS verifier for proof verification
- **`Program`**: Bytecode representation of VM programs

#### **Data Types**
- **`Input`**: Transaction inputs with type-specific data
- **`Output`**: Transaction outputs with type-specific data
- **`IOType`**: Enumeration of input/output types (Coin, Memo, State)
- **`Witness`**: Cryptographic proofs for state transitions

#### **Instruction Set**
- **Arithmetic**: `add`, `mul`, `neg`, `eq`
- **Logical**: `and`, `or`, `not`
- **Stack**: `push`, `pop`, `dup`, `roll`, `drop`
- **Cryptographic**: `commit`, `verify`, `scalar`, `range`
- **State**: `input`, `output`, `log`, `fee`

### Key Functions

#### **Proof Generation**
```rust
// Create a proof for state transition
let proof = Prover::build_proof(
    program,
    inputs,
    outputs,
    contract_deploy_flag,
    tx_data,
)?;
```

#### **Proof Verification**
```rust
// Verify a state transition proof
let result = Verifier::verify_r1cs_proof(
    &proof,
    &program,
    &inputs,
    &outputs,
    contract_deploy_flag,
    tx_data,
)?;
```

## Instruction Set

### Stack Operations
- **`push`**: Push data onto stack
- **`pop`**: Remove top item from stack
- **`dup`**: Duplicate stack item
- **`roll`**: Rotate stack items
- **`drop`**: Remove top item

### Arithmetic Operations
- **`add`**: Add top two stack items
- **`mul`**: Multiply top two stack items
- **`neg`**: Negate top stack item
- **`eq`**: Check equality of top two items

### Logical Operations
- **`and`**: Logical AND of top two items
- **`or`**: Logical OR of top two items
- **`not`**: Logical NOT of top item

### Cryptographic Operations
- **`commit`**: Create Pedersen commitment
- **`verify`**: Verify cryptographic proof
- **`scalar`**: Handle scalar values
- **`range`**: Create range proof

### State Operations
- **`input`**: Load input data
- **`output`**: Create output data
- **`log`**: Log operation for debugging
- **`fee`**: Handle transaction fees

## Security Features

### Zero-Knowledge Properties
- **Complete Privacy**: State transitions proven without revealing details
- **Cryptographic Proofs**: All operations backed by mathematical proofs
- **Constant-Time Verification**: Secure verification without timing leaks



## Integration with ZkOS

ZkVM is designed to integrate seamlessly with the ZkOS transaction system:

- **Transaction Integration**: Works with ZkOS transaction types
- **UTXO Management**: Handles unspent transaction outputs
- **Witness System**: Provides cryptographic proofs for state transitions
- **Smart Contracts**: Executes and verifies smart contract logic

## Minimum Supported Rust Version

Rust **1.70** or newer.

## Documentation

- [API Documentation](https://docs.rs/zkvm)
- [VM Specs and Instruction Set](specs/vm.md)

\=]\[=]\[';.,/;[-=pm, ## License & Attribution
Licensed under [`Apache-2.0`](../../LICENSE).  
Portions derived from Stellar’s **Slingshot** project; adapted © 2025 by Twilight Project Contributors.