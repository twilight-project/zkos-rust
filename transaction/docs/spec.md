# ZKOS Transaction specification

This is a technical specification for defining ZKOS transaction. ZKOS supports the following

## Constants

| name                        | type     | value | description                                   |
| --------------------------- | -------- | ----- | --------------------------------------------- |
| `GAS_PER_BYTE`              | `uint64` |       | Gas charged per byte of the transaction.      |
| `MAX_GAS_PER_TX`            | `uint64` |       | Maximum gas per transaction.                  |
| `MAX_INPUTS`                | `uint64` | `?`   | Maximum number of inputs.                     |
| `MAX_OUTPUTS`               | `uint64` | `?`   | Maximum number of outputs.                    |
| `MAX_PREDICATE_LENGTH`      | `uint64` |       | Maximum length of predicate, in instructions. |
| `MAX_PREDICATE_DATA_LENGTH` | `uint64` |       | Maximum length of predicate data, in bytes.   |
| `MAX_SCRIPT_LENGTH`         | `uint64` |       | Maximum length of script, in instructions.    |
| `MAX_SCRIPT_DATA_LENGTH`    | `uint64` |       | Maximum length of script data, in bytes.      |
| `MAX_STATIC_CONTRACTS`      | `uint64` | `255` | Maximum number of static contracts.           |
| `MAX_WITNESSES`             | `uint64` | `16`  | Maximum number of witnesses.                  |

## TransactionType

```
enum  TransactionType : uint8 {
    Transfer = 0,
    Transition = 1,
    Create = 2,
    Vault = 3,
}
```

## Transaction

| name   | type                                       | description       |
| ------ | ------------------------------------------ | ----------------- |
| `type` | [TransactionType](#transactiontype)        | Transaction type. |
| `data` | One of [TransactionType](#transactiontype) | Transaction data. |

Transaction is invalid if:

- `type > TransactionType.Shuffle`
- `gasLimit > MAX_GAS_PER_TX`
- `blockheight() < maturity`
- `inputsCount > MAX_INPUTS`
- `outputsCount > MAX_OUTPUTS`
- `witnessesCount > MAX_WITNESSES`
- More than one output is of type `OutputType.Change` for any asset ID in the input set (??CHANGE type??)
- Any output is of type `OutputType.Change` for any asset ID not in the input set

PLACEHOLDER for selialization Information regarding Tx

### TransactionTransfer

| name                 | type                              | description                              |
| -------------------- | --------------------------------- | ---------------------------------------- |
| `version`            | `uint64`                          | Version type.                            |
| `bytePrice`          | `uint64`                          | Price per transaction byte.              |
| `PriceLimit`         | `uint64`                          | Max price for transaction.               |
| `timeBounds`         | `uint64`                          | Block until which tx cannot be included. |
| `proofLength`        | `uint16`                          | Length of proof.                         |
| `shuffleProofLength` | `uint16`                          | Length of shuffle proof.                 |
| `inputCounts`        | `uint8`                           | Number of inputs.                        |
| `outputCounts`       | `uint8`                           | Number of outputs.                       |
| `witnessCounts`      | `uint8`                           | Number of witnesses.                     |
| `inputs`             | [Input](#input)`[]`               | List of inputs.                          |
| `outputs`            | [Output](#output)`[]`             | List of outputs.                         |
| `proof`              | [Proof](#proof)`[]`               | proof for dark transfer.                 |
| `shuffleProof`       | [ShuffleProof](#shuffleproof)`[]` | proof for shuffle.                       |
| `witnesses`          | [Witness](#witness)`[]`           | List of witnesses.                       |

Transaction is invalid if:

- `inputsCount != 9`
- `outputsCount != 9`
- inputs are of any type other than `InputType.DarkCoin`
- outputs are of any type other than `OutputType.DarkCoin`

## InputType

```
enum  InputType : uint8 {
    Lit = 0,
    Dark = 1,
    Record = 2,
    State = 3,
    InterState = 4,
}
```

## Input

| name   | type                           | description    |
| ------ | ------------------------------ | -------------- |
| `type` | [InputType](#inputtype)        | Type of input. |
| `data` | One of [InputType](#inputtype) | Input data.    |

Transaction is invalid if:

- `type > InputType.InterState`

### InputLit

| name           | type       | description                                         |
| -------------- | ---------- | --------------------------------------------------- |
| `txID`         | `byte[32]` | Hash of transaction.                                |
| `outputIndex`  | `uint8`    | Index of transaction output.                        |
| `owner`        | `byte[32]` | Owning address or predicate root.                   |
| `amount`       | `uint64`   | Amount of coins.                                    |
| `witnessIndex` | `uint8`    | Index of witness that authorizes spending the coin. |

### InputDark

| name           | type       | description                                         |
| -------------- | ---------- | --------------------------------------------------- |
| `txID`         | `byte[32]` | Hash of transaction.                                |
| `outputIndex`  | `uint8`    | Index of transaction output.                        |
| `owner`        | `byte[68]` | Owning address or predicate root.                   |
| `encryption`   | `byte[64]` | Elgamal encryption on amount of coins.              |
| `witnessIndex` | `uint8`    | Index of witness that authorizes spending the coin. |

## OutputType

```
enum  OutputType : uint8 {
    Lit = 0,
    Dark = 1,
    Record = 2,
    State = 3,
    InterState = 4,
}
```

## Output

| name   | type                                                       | description     |
| ------ | ---------------------------------------------------------- | --------------- |
| `type` | [OutputType](#outputtype)                                  | Type of output. |
| `data` | One of [OutputDark](#outputdark), [OutputLit](#outputlit). | Output data.    |
