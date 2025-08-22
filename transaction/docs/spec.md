# ZKOS Transaction Specification

## 1. Introduction

This document provides the technical specification for the ZkOS transaction structure. It defines the data types, validation rules, and serialization formats required to create, process, and verify transactions on the ZkOS blockchain.

The ZkOS transaction model is designed with three primary goals:

*   **Confidentiality:** To enable private and anonymous transfers and interactions through zero-knowledge proofs.
*   **Programmability:** To support complex state transitions and smart contracts via an integrated Zero-Knowledge Virtual Machine (ZkVM).
*   **Extensibility:** To provide a flexible framework with distinct transaction types (`Transfer`, `Script`, `Message`) that can support a variety of application protocols.

This specification should be read in conjunction with the **[ZkVM Specification](../zkvm/specs/README.md)**, which details the virtual machine that executes `TransactionScript` programs.






## Constants

| name                        | type     | value   | description                                   |
|-----------------------------|----------|-------  |-----------------------------------------------|
| `GAS_PER_BYTE`              | `uint64` |  `10`   | Gas charged per byte of the transaction.      |
| `MAX_INPUTS`                | `uint64` |   `8`   | Maximum number of inputs.                     |
| `MAX_OUTPUTS`               | `uint64` |   `8`   | Maximum number of outputs.                    |
| `MAX_SCRIPT_LENGTH`         | `uint64` |`1024 * 1024`   | Maximum length of script, in instructions.    |
| `MAX_SCRIPT_DATA_LENGTH`    | `uint64` |`1024 * 1024`   | Maximum length of script data, in bytes.      |
| `MAX_WITNESSES`             | `uint64` | `9`  | Maximum number of witnesses.                  |

## ZkOS Types

ZkOS defines the following types

## Addresses

### Network
ZkOS supports two types of Networks. The first byte of address represents the intended network.
    1. Mainnet -> reprersented by 12 for Standard and 24 for Script addresses
    2. Testnet -> represented by 44 for Standard and 66 for Script addresses

### Standard
[Standard](#standard) address are used to identify the owner of the assets on the blockchain. The address is derived based on the El-Gamal public key encoded as hexadecimal string.
The address is derived as follows
    1. The first byte represents the Network
    2. 64 byte El-Gamal public key
    3. 4 bytes of the Hashed address as checksum
    4. Encode the bytes representation as hexadecimal string

### Script
[Script](#script) address represents the merkle root of the program merkle tree. It is used to ensure the integrity and authenticity of the deployed programs on the blockchian.
The script is constructed as follows
    1. The first byte represents the Network
    2. 32 byte merkle tree root is hashed using RIPEMD-160 
    3. The resultant bytes are encoded as hexadecimal string

## Contract 
A ZkOS [Contract](#contract-type) is expressed as a collection of ZkVM programs stored in a binary merkle tree and associated [state](state-type). Each 
leaf in the merkle tree represents a program containing computational constraints or execution instructions for state change. 

## TransactionType

```
enum  TransactionType : uint8 {
    Transfer = 0,             | Supports direct asset transfers among users (Private / Anonymous)                              
    Script = 1,               | Supports interaction with Contracts using Zkvm Programs  
    Vault = 2,                | Supports bridginf assets among blockchains
    Message = 3,              | Supports auxiliary message passing on chain   
}
```

## Transaction

[Transaction](#transaction) type definition

| name   | type                                                                                      | description       |
|--------|-------------------------------------------------------------------------------------------|-------------------|
| `type` | [TransactionType](#transactiontype)                                                       | Transaction type. |
| `data` | One of [TransactionType](#transactiontype)                                                | Transaction data. |

Transaction is invalid if:

- `type > TransactionType.Message`
- `gasLimit > MAX_GAS_PER_TX`
- `blockheight() < maturity`
- `inputsCount > MAX_INPUTS`
- `outputsCount > MAX_OUTPUTS`
- `witnessesCount > MAX_WITNESSES`

### TransactionTransfer

[TransactionTransfer](#transfertransaction) type definition

| name                 | type                               | description                              |
|----------------------|------------------------------------|------------------------------------------|
| `version`            | `uint64`                           | Version type.                            |
| `maturity`           | `uint64`                           | Block until which tx cannot be included. |
| `fee`                | `uint64`                           | fee for the tx.                          |
| `inputCounts`        | `uint8`                            | Number of inputs.                        |
| `outputCounts`       | `uint8`                            | Number of outputs.                       |
| `witnessCounts`      | `uint8`                            | Number of witnesses.                     |
| `inputs`             | [Input](#input)`[]`                | List of inputs.                          |
| `outputs`            | [Output](#output)`[]`              | List of outputs.                         |
| `proof`              | [PrivateProof](#privateproof)      | Private transfer proof                   |
| `shuffleProof`       | [ShuffleProof](#shuffleproof)      | optional anonimity proof.                |
| `witnesses`          | [Witness](#witness)`[]`            | List of witnesses.                       |

-TransactionTransfer supports QuisQuis(Anonymous) and Private transfer of assets

Transaction is invalid if:
- `inputsCount >= 9`
- `outputsCount >= 9`
- inputs are of any type other than `InputType.Coin`
- outputs are of any type other than `OutputType.Coin`


### TransactionScript

[TransactionScript](#transactionscript) type definition

| name                   | type                      | description                                       |
|------------------------|---------------------------|---------------------------------------------------|
| `version`              | `uint64`                  | Version type.                                     |
| `fee`                  | `uint64`                  | Max price for transaction.                        |  
| `maturity`             | `uint64`                  |Block until which tx cannot be included.           |
| `inputsCount`          | `uint8`                   | Number of inputs.                                 |   
| `outputsCount`         | `uint8`                   | Number of outputs.                                |
| `witnessCount`         | `uint8`                   | Number of witnesses.                              |
| `inputs`               | [Input](#input)`[]`       | List of inputs.                                   |
| `outputs`              | [Output](#output)`[]`     | List of outputs.                                  |
| `program`              | `byte[]`                  | ZKVM program instructions for proof construction. |
| `callproof`            | `byte[]`                  | Program call proof in the contract tree.          |
| `proof`                | [R1CSProof](#r1csproof)   | R1CS proof for computations.                      |
| `witnesses`            | [Witness](#witness)`[]`   | List of witnesses.                                |                       
| `txdata`               | [data](#data)`[]`         | Optional. Arbitrary tx data for ZkVM stack.       |

- TransactionScript supports Contract(program-based) Transactions only.
- Contract deployment and interactions are handled through this tx type.
- It can only be used to interact with contracts deployed on the blockchain.     

Transaction is invalid if:

- number of `programs` is > 1
- `InputType.Coin` does not have a matching `OutputType.Memo` 
- `InputType.Memo` does not have a matching `OutputType.Coin`
- `InputType.State` does not have a matching `OutputType.State` and vice versa


### TransactionVault [TBD]

| name                   | type                      | description                                       |
|------------------------|---------------------------|---------------------------------------------------|
| `version`              | `uint64`                  | Version type.                                     |
| `fee`                  | `uint64`                  | Max price for transaction.                        |
| `maturity`             | `uint32`                  | Block until which tx cannot be included.          |
| `inputsCount`          | `uint8`                   | Number of inputs.                                 |   
| `outputsCount`         | `uint8`                   | Number of outputs.                                |
| `witnessCounts`        | `uint8`                   | Number of witnesses.                              |
| `inputs`               | [Input](#input)`[]`       | List of inputs.                                   |
| `outputs`              | [Output](#output)`[]`     | List of outputs.                                  |
| `witnesses`            | [Witness](#witness)`[]`   | List of witnesses.                                |

### TransactionMessage

```
enum  MessageType : uint8 {
    burn = 0, // destroys asset utxo
    app = 1,  // carries app specific message
}
```

### Message

| name                   | type                      | description                                       |
|------------------------|---------------------------|---------------------------------------------------|
| `version`              | `uint64`                  | Version type.                                     |
| `fee`                  | `uint64`                  | Max price for transaction.                        |  
| `input`                | [Input](#input)           | Input asset.                                      |
| `type`                 | [type](#type)             | type for message   (burn or app)                  |
| `data`                 | [data](#data)             | data for message                                  |
| `proof`                | [SigmaProof](#proof)      | Optional. reveal proof for burn.                  |
| `witness`              | [Witness](#witness)       | Witness carrying Authorization signature.         |


## IOType
ZkOS defines three variants of [input](#input) and [output](#output).

[IOType](#io-type)  type definition
```
enum  IOType : uint8 {
    Coin = 0,
    Memo = 1,
    State = 2,
}
```

## Input

[Input](#input) type definition

| name   | type                                                                                              | description    |
|--------|---------------------------------------------------------------------------------------------------|----------------|
| `type` | [IOType](#io-type)                                                                                | Type of input. |
| `data` | One of [InputCoin](#inputcoin), [InputMemo](#inputmemo) or [InputState](#inputstate)            | Input data.    |

Transaction is invalid if:

- `type > InputType.State`


### InputCoin

[InputCoin](#inputcoin) type definition

| name                  | type                    | description                                                            |
|-----------------------|-------------------------|------------------------------------------------------------------------|
| `txID`                | `byte[32]`              | Hash of transaction.                                                   |
| `outputIndex`         | `uint8`                 | Index of transaction output. UtxoID                                    |
| `OutCoin`             | `OutputCoin`            | [OutputCoin](#outputcoin)|
| `witnessIndex`        | `uint8`                 | Optional. Index of witness that authorizes spending the coin.          |


### InputMemo

[InputMemo](#inputmemo) type definition

| name                  | type                    | description                                                            |
|-----------------------|-------------------------|------------------------------------------------------------------------|
| `txID`                | `byte[32]`              | Hash of transaction.                                                   |
| `outputIndex`         | `uint8`                 | Index of transaction output.                                           |
| `OutMemo`             | `OutputMemo`            | [OutputMemo](#outputmemo).                                             |
| `data`                | `ZKVMString`            | Optional. Additional Information for memo to coin conversion           |
| `witnessIndex`        | `uint8`                 | Index of witness that authorizes memo interaction with contract.       |


### InputState

[InputState](#inputstate) type definition

| name                  | type                    | description                                                            |
|-----------------------|-------------------------|------------------------------------------------------------------------|
| `txID`                | `byte[32]`              | Hash of transaction.                                                   |
| `outputIndex`         | `uint8`                 | Index of transaction output.                                           |
| `OutState`            | `OutputState`           | [OutputState](#outputstate).                                           |
| `witnessIndex`        | `uint8`                 | Index of witness that authorizes state transition.                     |
| `scriptData`          | `ZKVMString[]`          | Optional. Addition script data for state transition.                   |


## Output
[Output](#output) type definition

| name   | type                                                                                              | description    |
|--------|---------------------------------------------------------------------------------------------------|----------------|
| `type` | [IOType](#io-type)                                                                                | Type of output.|
| `data` | One of [OutputCoin](#outputcoin), [OutputState](#outputstate), [OutputMemo](#outputmemo),         | Output data.   |

### OutputCoin

[OutputCoin](#outputcoin) type definition

| name                  | type                    | description                                                            |
|-----------------------|-------------------------|------------------------------------------------------------------------|
| `owner`               | `String`                | Owner [Standard](#standard) coin address encoded as hex.               |
| `encrypt`             | `Encryption`            | Elgamal encryption on value/amount of coin asset.                      |
| `timeBounds`          | `uint32`                | Block until which the Output cannot be consumed.                       |


### OutputMemo

[OutputMemo](#outputmemo) type definition

| name                  | type                    | description                                                            |
|-----------------------|-------------------------|------------------------------------------------------------------------|
| `ScriptAddress`       | `String`                | [Script](#script) address of [Contract](#contract-type).               |
| `owner`               | `String`                | Owner [Standard](#standard) address of originating coin                |
| `amount`              | `Commitment`            | Pedersen commitment of value amount.                                   |
| `data`                | `ZKVMString`            | Memo auxiliary data. u64/u32/Scalar/CompressedRistretto.               |
| `timeBounds`          | `uint32`                | Block until which the Output cannot be consumed.                       |


### OutputState

[OutputState](#outputstate) type definition

| name                  | type                    | description                                                            |
|-----------------------|-------------------------|------------------------------------------------------------------------|
| `Nonce`               | `uint32`                | Nonce of the state change value                                        |
| `ScriptAddress`       | `String`                | [Script](#script) address of [Contract](#contract-type) state.         |
| `owner`               | `String`                | Contract Owner [Standard](#standard) address                           |
| `value`               | `Value`                 | Pedersen commitment or Cleartext value of assets                       |
| `stateData`           | `ZKVMString[]`          | Optional. Additional state variables                                   |
| `timeBounds`          | `uint32`                | Block until which the Output cannot be consumed.                       |

##Transaction Validation

In case of a [TransactionTransfer](#transfertransaction), the following steps are performed
* performs basic checks on input/Output count and [IOType](#io-type)
* parses [TransactionTransfer](#transfertransaction) type to identify Confidential versus Anonymous Transfer. Existance of `ShuffleProof` signals Anonymous Transfers. 
* Verifies [PrivateProof](#privateproof) and Witness in case of Confidential Tranfer
* Verifies [PrivateProof](#privateproof), [ShuffleProof](#shuffleproof) and Witness for Anonymous transfer
Transaction fails if any of the above steps fail

 For a [TransactionScript](#transactionscript), the verifier performs the following steps,

1. Verifies `callproof` for the `program` in the [TransactionScript](#transactionscript). This step insures that the program used by the Tx is the one already deployed on blockchain. Terminates if the `callproof` verification fails
2. Initiates the virtual VM
3. Executes the `program` to verify `proof`. Terminates if the `proof` verification fails
4. Verifies the [Witness](#witness) for all [Inputs](#input) 
5. Verification fails if any of the above step returns fails


## Contract Deployment

[Contract](#contract-type) can be deployed using [TransactionScript](#transactionscript). The process to successfully deploy a contract is as follows

1. Write a set of ZkVm programs 
2. create a Merkle binary tree using the programs as leaves
3. Derive a script address
4. Create [InputState](#inputstate) with zero values. This should be accompanied by a reveal proof in case of encrypted state variables.  
5. Create [OutputState](#outputstate) with inital deposit.
6. Use an [InputCoin](#inputcoin) to initiate the state.
7. Get the initialization program for the Contract.
8. create a `callproof` for the initialization program.
9. Create [ValueWitness](#value-witness) and [StateWitness](#state-witness).
10. Create the [Transaction](#transaction) for broadcast to the blockchain.

## Witness

Witness carrries authorization for spending the [Input](#input) and/or zero-knowledge Sigma proofs for establishing relationships among 
el-gamal encryptions and pedersen commitments. Every [Input](#input) in [Transaction](#transaction) carries a corresponding Witness. Following four types of winesses are supported in ZkOS.
* [`Signature`](#sinature)
* [`SigmaProof`](#sigmaproof)
* [`ValueWitness`](#value-witness)
* [`StateWitness`](#state-witness)

### Signature
[Signature](#signature) are created using [ZkSchnorr](#https://github.com/twilight-project/ZkSchnorr/). It is a single-message Schnorr signature protocol implemented with multi-point [Ristretto](https://ristretto.group) verification key and [Merlin transcripts](https://merlin.cool). Signature is performed on [Input](#input) and setting the `outputIndex` to 0  before taking the hash and signing message with public keys in [Standard](#standard) address. It is used to authorize [InputCoin](#inputcoin) and InputState(#inputstate) spending in [TransactionScript](#transactionscript). 

### SigmaProof

[SigmaProof](#sigmaproof) is a compact zero-knowledge based [Sigma Protocol](#https://ir.cwi.nl/pub/21438) implementation. It is used to authorize the spending of [InputMemo](#inputmemo) in a [TransactionScript](#transactionscript) and to carry zero-value proofs in [TransactionTransfer](#transfertransaction). 

### ValueWitness
[ValueWitness](#value-witness) constitutes same value [SigmaProof](#sigmaproof) between [inputCoin](#inputcoin) `encryption` and [OutputMemo](#outputmemo) `commitment` and [Signature](#signature) over the [Input](#input).

### StateWitness
[StateWitness](#state-witness) constitutes [Signature](#signature) over the [Input](#input) and zero-value reveal proof. The reveal proof is used during [Contract](#contract-type) initialization to prove zero balance state variables. 

