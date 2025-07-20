
s

---

```markdown:transaction/docs/use-cases/initialization.md
# Use Case: Contract Initialization

## 1. Overview

The Contract Initialization transaction is a specialized `ScriptTransaction` used to deploy a new smart contract to the ZkOS blockchain. This process involves creating the contract's initial state from scratch and seeding it with an initial deposit from the deployer's `InputCoin`.

A key feature of this transaction is the use of a zero-balance proof for the initial `InputState`, which signals to the system that this is a contract creation event rather than an update to an existing contract.

## 2. Transaction Structure

This transaction is composed of multiple inputs and outputs that work together to establish the new contract.

| Component | Type | Purpose |
| :--- | :--- | :--- |
| **Input** | `InputCoin` | The deployer's confidential asset, which will be consumed to provide the initial deposit for the contract's state. |
| **Input** | `InputState` | A placeholder for the initial state. Its UTXO does not exist on-chain. Crucially, its corresponding `StateWitness` contains a **zero-balance proof** instead of a signature. |
| **Output**| `OutputMemo` | A data-carrying UTXO, typically used to record metadata about the deployment, such as the initial pool share issued to the deployer. |
| **Output**| `OutputState`| The newly created contract state UTXO. Its value is constrained to match the `InputCoin`'s value, and its state variables are initialized. |

## 3. VM Program Logic (`initialization_program`)

The ZkVM program is responsible for constraining the relationship between the inputs and the newly created outputs.

**Conceptual Logic:**

```
// Pseudocode for the ZkVM program
let deposit_value = input_coin.value;
let initial_state_value = input_state.value; // Constrained to be zero by witness
let new_state_value = output_state.value;

// 1. Prove the input state was empty (value is zero)
// This is handled by the zero-balance proof in the StateWitness,
// which is verified outside the main R1CS proof.

// 2. Constrain the new state's value to the deposit amount
constrain_eq(new_state_value, deposit_value);

// 3. Initialize the contract's state variables
// (e.g., setting total liquidity, issuing initial shares)
let initial_liquidity = ...
let initial_shares = ...
constrain_eq(output_state.state_variables.total_liquidity, initial_liquidity);
constrain_eq(output_state.state_variables.shares, initial_shares);
```
<code_block_to_apply_changes_from>
```

## 4. Verification Flow

1.  **Witness Verification:**
    *   The `ValueWitness` for the `InputCoin` is verified to prove ownership.
    *   The `StateWitness` for the `InputState` is verified. The system checks for the presence of the **zero-balance proof**, which confirms this is a valid deployment and that the state starts with a value of zero.

2.  **Call Proof Verification:**
    *   The `CallProof` is verified against the Merkle root of the contract's program suite to ensure the authorized `initialization_program` is being executed.

3.  **R1CS Proof Verification:**
    *   The ZkVM `Verifier` executes the program and verifies the `R1CSProof`. This confirms that the `OutputState` was correctly initialized based on the `InputCoin` deposit and the program's logic.

## 5. Key Design Decisions

-   **Zero-Balance Proof for State:** Using a zero-balance proof within the `StateWitness` is an elegant solution to distinguish contract creation from contract updates. When the `nonce` of an `InputState` is zero, the system expects a zero-balance proof. For any `nonce > 0`, it expects a valid signature. This avoids needing a separate transaction type for deployment.
-   **Merkle Tree of Programs:** The contract's `script_address` is derived from the Merkle root of all its possible programs (`call_proof`). This ensures that only authorized code can ever be run for that contract.
```

