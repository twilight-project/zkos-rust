# ZkVM Specification

**Version:** 0.1.0  
**Status:** Experimental  
**Last Updated:** 2025-01-XX

> **Origin:** This specification is adapted from the ZkVM design in Stellar's Slingshot project (Apache-2.0).

## 1. Introduction

This document specifies the ZkVM: a stack-based virtual machine designed to create and verify zero-knowledge proofs for state transitions within the ZkOS ecosystem.

Unlike a general-purpose blockchain VM, the ZkOS ZkVM is not a transaction processing engine. Instead, it serves as a specialized **proof-generation engine**. Its primary role is to execute a program that builds a Rank-1 Constraint System (R1CS), which is then used to generate a Bulletproof attesting to the correctness of a state transition without revealing the underlying data.

This specification is divided into the following parts:

- **[Types](./TYPES.md):** The data types manipulated by the VM.
- **[Definitions](./DEFINITIONS.md):** Core cryptographic primitives and concepts.
- **[VM Operation](./OPERATIONS.md):** The internal state and execution model of the VM.
- **[Instruction Set](./INSTRUCTIONS.md):** The complete reference for all VM opcodes.
- **[Security](./SECURITY.md):** Security considerations for the VM and its proofs.

## 2. Core Concepts

A ZkOS `ScriptTransaction` encapsulates a `Program` that is executed by the ZkVM. During execution, the VM manipulates various data types to construct a zero-knowledge proof of the program’s computations.

- **Values & State:** The VM operates on abstract `Value` types. In ZkOS, these are mapped to concrete types like `Coin`, `Memo`, and `State` which are loaded onto the stack during initialization. The VM's purpose is to prove valid transformations between these input and output types.

- **Programmable Constraints:** Custom logic is represented via programmable `Constraints`. These form a single `Constraint System` which is proven to be satisfied after the VM has finished execution.

- **Proof of Correctness:** A ZkVM proof is valid only if the VM program executes to completion without errors and leaves the stack empty. This guarantees that all constraints were met and the state transition is valid.

## 3. ZkOS Integration

The ZkVM provides the generic execution environment, while the ZkOS `transaction` crate provides the concrete data structures loaded into the VM.

### Data Mapping

| ZkVM Generic Type | ZkOS Concrete Type (`zkos_types.rs`) | Purpose in ZkOS |
| :--- | :--- | :--- |
| `Input` / `Output` | `Input` / `Output` | Represents the inputs and outputs of a state transition. |
| `Value` | `Coin` | A confidential asset with an encrypted amount. |
| `Value` | `Memo` | Data-carrying output, often paired with a `Coin`. |
| `Value` | `State` | A smart contract's persistent state. |
| `Constraint` | `Witness` (`ValueWitness`, `StateWitness`) | Cryptographic proof authorizing the use of an input. |

---
**Next:** [VM Types &raquo;](./TYPES.md)


