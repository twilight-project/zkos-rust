# Use Case: Order Transaction

## 1. Overview

The Order Transaction is a `ScriptTransaction` designed to facilitate the placing of an order within the ZkOS exchange protocol. It allows a user to commit a certain amount of a confidential asset (`InputCoin`) towards an order, while proving that their remaining balance is non-negative.

The core of this transaction is a ZkVM program that generates a **range proof** on the difference between the user's initial coin value and the order amount, ensuring solvency without revealing either value.

## 2. Transaction Structure

The transaction is composed of a confidential input and a data-carrying output.

| Component | Type | Purpose |
| :--- | :--- | :--- |
| **Input** | `InputCoin` | Represents the confidential asset (e.g., 100 TOKEN_A) that the user owns and is using to back the order. It is consumed in this transaction. |
| **Output** | `OutputMemo` | A new data-carrying UTXO. Its commitment matches the `InputCoin`'s commitment, but its auxiliary data (`MemoData`) stores the specific `order_amount` for off-chain systems to index and process. |

## 3. VM Program Logic (`program`)

The ZkVM program executed by this transaction is designed to perform one critical task: prove the user's solvency.

**Conceptual Logic:**

```
// Pseudocode for the ZkVM program
let initial_value = input[0].value;
let order_amount = output[0].data.order_amount;

// This is the core constraint:
// Prove that the user's remaining balance is valid.
range_proof(initial_value - order_amount);
```

This ensures that `initial_value >= order_amount` without revealing either value to the public blockchain. The proof is generated via the `range` instruction within the ZkVM.

## 4. Verification Flow

A validator verifying this transaction performs the following steps:

1.  **Witness Verification:**
    *   The `ValueWitness` for the `InputCoin` is checked. This involves verifying the signature to prove ownership and the DLEQ proof to ensure the commitment in the corresponding `OutputMemo` is for the same value.

2.  **Call Proof Verification:**
    *   The `CallProof` is verified against the exchange contract's Merkle root to ensure the correct, authorized `program` is being executed.

3.  **R1CS Proof Verification:**
    *   The ZkVM `Verifier` is initialized with the transaction's `Inputs`, `Outputs`, and `program`.
    *   The `Verifier` executes the program, reconstructing the constraints.
    *   Finally, it verifies the `R1CSProof` provided in the transaction. A successful verification confirms that the range proof (and any other constraints) was satisfied.

## 5. Key Design Decisions

-   **Why `InputCoin` and `OutputMemo`?** This pattern allows the system to "tag" a confidential value with public data. The `InputCoin` is consumed, and a new `OutputMemo` UTXO is created with the same value commitment, but with added public metadata (the order amount). This makes the order discoverable to the exchange's matching engine.
-   **No State Change:** This specific transaction is stateless; it does not interact with an `InputState` or `OutputState`. It only proves a condition on a user's balance.
