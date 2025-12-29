# Changelog

All notable changes to the `zkvm` crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] – 2025-12-29
### Maintenance
- Updated internal dependency `quisquis-rust` to `Testnet-v1.0.1`.
- No public API or observable behavior changes are intended.

## [0.1.0] - 2025-01-01

### Summary
Initial public release of the ZkVM stack-based virtual machine for zero-knowledge state transitions.

### ✨ Added

#### Core Virtual Machine
- **Stack-Based Architecture**: Complete stack-based VM implementation
- **R1CS Integration**: Bulletproofs Rank-1 Constraint System integration
- **Zero-Knowledge Proofs**: Proof generation and verification for state transitions
- **Instruction Set**: Comprehensive instruction set for smart contract execution

#### State Management
- **Input/Output Types**: Support for Coin, Memo, and State data types
- **UTXO Integration**: Unspent transaction output management
- **Witness System**: Cryptographic proofs for state transitions
- **State Transitions**: Secure state change verification

#### Smart Contract Support
- **Program Execution**: Bytecode-based program execution
- **Contract Deployment**: Support for contract deployment and initialization
- **State Variables**: Management of contract state variables
- **Script Data**: Handling of script-specific data

#### Cryptographic Components
- **Pedersen Commitments**: Secure value commitment system
- **Range Proofs**: Zero-knowledge range proof integration
- **Signature Verification**: Cryptographic signature handling
- **Merkle Tree Integration**: Merkle tree for program verification



### Attribution
Portions derived from the **Slingshot** project by Stellar Development Foundation (archived June 6 2024).  
Licensed under the Apache-2.0 license. See [LICENSE](../../LICENSE) for details.

---

## Version History

- **0.1.0** - Initial release with comprehensive VM system 

<!-- No compare link: initial release -->