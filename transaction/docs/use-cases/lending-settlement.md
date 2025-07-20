# Use Case: Lending and Settlement Transactions

## 1. Overview

This document specifies two related `ScriptTransaction` flows for interacting with a deployed ZkOS smart contract: **Lending** (depositing funds) and **Settlement** (withdrawing funds). Both are stateful transactions that update the contract's `InputState` to a new `OutputState`.

-   **Lend Transaction:** A user deposits a confidential `InputCoin` into the contract, receiving an `OutputMemo` as a record and updating the contract's state.
-   **Settlement Transaction:** A user redeems an `InputMemo` (representing a claim or settlement), receiving a confidential `OutputCoin` and updating the contract's state.

## 2. Lend Transaction

### 2.1. Transaction Structure

| Component | Type | Purpose |
| :--- | :--- | :--- |
| **Input** | `InputCoin` | The user's confidential asset to be lent/deposited. |
| **Input** | `InputState` | The current state of the smart contract being interacted with. |
| **Output**| `OutputMemo` | A record of the transaction, containing the lend amount and potentially other data (e.g., a TPS commitment) in its `MemoData`. |
| **Output**| `OutputState`| The contract's new state after the deposit, with updated liquidity and nonce. |

### 2.2. VM Program Logic

The program proves that the contract state was updated correctly based on the user's deposit.

**Conceptual Logic:**

```
// Pseudocode for the ZkVM program
let deposit_value = input_coin.value;
let old_contract_value = input_state.value;
let new_contract_value = output_state.value;

// Constrain the new contract value
constrain_eq(new_contract_value, old_contract_value + deposit_value);

// Update other state variables (e.g., user shares)
...
```

### 2.3. Verification Flow

1.  **Witness Verification:**
    *   The `ValueWitness` for the `InputCoin` is verified.
    *   The `StateWitness` for the `InputState` is verified (this time with a signature, as it's an update).

2.  **State Transition Validation:**
    *   The `nonce` of the `OutputState` must be `InputState.nonce + 1`.
    *   The `script_address` must be unchanged.

3.  **Call Proof & R1CS Proof Verification:**
    *   The `CallProof` and `R1CSProof` are verified to ensure the correct state update logic was executed.

## 3. Settlement Transaction

### 3.1. Transaction Structure

| Component | Type | Purpose |
| :--- | :--- | :--- |
| **Input** | `InputMemo` | The user's claim to be settled. The commitment represents the value to be paid out. |
| **Input** | `InputState` | The current state of the smart contract. |
| **Output**| `OutputCoin` | The new confidential asset paid out to the user. |
| **Output**| `OutputState`| The contract's new state after the settlement. |

### 3.2. VM Program Logic

The program proves the contract can afford the settlement and that the state is updated correctly.

**Conceptual Logic:**

```
// Pseudocode for the ZkVM program
let settlement_value = input_memo.commitment.value;
let old_contract_value = input_state.value;
let new_contract_value = output_state.value;

// Constrain the new contract value
constrain_eq(old_contract_value, new_contract_value + settlement_value);

// Prove the contract had sufficient funds
range_proof(new_contract_value); // Proves new_contract_value >= 0

// Update other state variables
...
```

### 3.3. Verification Flow

The verification flow is nearly identical to the Lend Transaction, but the roles of the `InputMemo` and `OutputCoin` are reversed. The DLEQ proof in the `OutputCoin`'s witness links its encrypted value back to the `InputMemo`'s public commitment.

## 4. Key Design Decisions

-   **Vector Data in Memos:** The `MemoData` field is a `Vec<String>`, allowing for flexible and extensible data storage. This is used to hold order amounts, TPS commitments, and other auxiliary data without changing the core transaction structure.
-   **State Immutability:** While state variables can change, their *structure* can be enforced by having the ZkVM program check a hash of the state data. The program can take the `InputState` data, apply transformations, and constrain the result against the `OutputState` data, ensuring only valid transformations occur.
-   **Stateful vs. Stateless Scripts:** The system cleanly separates stateless actions (like the basic Order a Transaction) from stateful ones (Lend/Settle). The presence of an `InputState` and `OutputState` is the clear differentiator.