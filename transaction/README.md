# ZKOS Transaction

- [Specification](docs/spec.md)

## Features

- QuisQuis Transaction
- Lit to Lit Transfer
- Dark to Dark Transfer
- Template based Transaction

## Contract Deployment  
[Contract](#contract-type) can be deployed using [TransactionScript](#transactionscript). 
The process to successfully deploy a contract is as follows  
1. Write a set of ZkVm programs
2. Create a Merkle binary tree using the programs as leaves
3.  Derive a script address
4.  Create [InputState](#inputstate) with zero values. This should be accompanied by a reveal proof in case of encrypted state variables.
5. Create [OutputState](#outputstate) with inital deposit.
6. Use an [InputCoin](#inputcoin) to initiate the state.
7. Get the initialization program for the Contract.
8. Create a `callproof` for the initialization program.
9. Create [ValueWitness](#value-witness) and [StateWitness](#state-witness).
10.  Create the [Transaction](#transaction) for broadcast to the blockchain.
