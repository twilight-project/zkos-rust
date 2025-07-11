# Project ZkOS 

_Step into the Enchanted Privacy Land_.

ZkOS (Zero Knowledge OS) is a new utxo-based blockchain architecture under active development, focused primaily on privacy, truslessness, and decentralization.  

## Specification

The specifications for ZkOS are defined in [docs](docs/)

The ZkOS project consists of the following components

### [Address](address)

An API module designed to compute blockchian addresses from Public keys and vice-versa. 

### [Merkle](merkle)

A Merkle tree API for computing Merkle roots, making and verifying Merkle proofs.
Used for program inclusion proof in ZkOS transaction and Taproot implementation.

Based on [RFC 6962 Section 2.1](https://tools.ietf.org/html/rfc6962#section-2.1) and implemented using [Merlin](https://merlin.cool).

### [Rangeproof](rangeproof)

[Bulletproofs](https://doc.dalek.rs/bulletproofs/index.html) zero-knowledge circuit proof system. 

### [Transaction](transaction)

A Transaction API module for creating and verifying ZkOS transactions. The detailed specifications and validations rules are defined 
[here](#./docs/transaction-spec.md).

The complete instructions for using the API are defined in the [Readme.md](#./transaction/readme.md). 

### [ZKVM](zkvm)

ZkVM is a virtual machine implementation for **zero-knowledge smart contract** execution/verification. 

* [README](zkvm/README.md)
* [ZkVM specification](docs/vm-spec.md)


### [TransactionApi](transactionapi)

This module defines the rpc-endpoints for querying utxos.  

### [UTXO](utxo-in-memory)

Utxo based State maintainance for ZkOS transactions

This module defines API for creating and maintaining an in-memory and archival utxo-set.  

### [Reader/Writer](readerwriter)

Simple encoding/decoding and reading/writing traits and utilities for blockchain data structures.


