# ZkVM Specification: Security Considerations

> Part of the ZkVM Specification.  
> &laquo; [Previous: Instruction Set](./INSTRUCTIONS.md) | [Home](./README.md)
---

This document outlines the security model, guarantees, and considerations for the ZkOS ZkVM.

## 1. Security Model & Guarantees

The ZkVM is designed to provide the following security guarantees:

1.  **Confidentiality:** The ZkVM's proofs of computation (via Bulletproofs) do not reveal any secret information about the inputs or intermediate state of the program. This ensures that values, flavors, and other sensitive data remain private.

2.  **Integrity:** A valid ZkVM proof guarantees that the associated computation was performed correctly according to the program's instructions. It is computationally infeasible to create a valid proof for an invalid state transition.

3.  **Soundness:** It is computationally infeasible for a malicious party to create a proof for a false statement (e.g., proving a constraint that is not actually satisfied).

## 2. Key Security Mechanisms

### Cryptographic Primitives

-   **Ristretto255 Curve:** All elliptic curve operations are performed on the Ristretto255 group, which is prime-order and provides protection against many classes of side-channel attacks and invalid-curve attacks.
-   **Merlin Transcripts:** The use of Merlin for cryptographic transcripts ensures domain separation for all generated challenges, preventing cross-protocol attacks and replay attacks.
-   **Pedersen Commitments:** Provide perfect hiding and computational binding, ensuring that committed values cannot be revealed but also cannot be changed after commitment.

### Constraint System

-   **Rank-1 Constraint System (R1CS):** The foundation of the proof system. Every operation in the VM that affects a value is translated into one or more constraints. A valid final proof implies the entire system of constraints is satisfied.
-   **Range Proofs:** The `range` instruction is critical for preventing value overflows and ensuring that quantities remain within their defined bit-range (e.g., non-negative `u64`).

### VM Execution

-   **Linear Type System:** The enforcement of linear types for `Values`, `Constraints`, and other items is a critical security feature. It prevents a wide range of logical bugs, such as the duplication (double-spending) of a value or the reuse of a constraint.
-   **Stack Discipline:** The requirement that the stack must be empty at the end of execution ensures that all created values and constraints have been properly consumed and verified.

## 3. Potential Vulnerabilities & Mitigations

-   **Malleability of Proofs:** The VM design uses non-interactive proofs (Bulletproofs), which are generally not subject to malleability. The use of Merlin transcripts further binds the proof to the context in which it was created.

-   **Side-Channel Attacks:** While the underlying `curve25519-dalek` library is designed to be constant-time, care must be taken in higher-level logic to avoid data-dependent branching or memory access patterns.

-   **Incorrect Constraint Construction:** This is the most significant risk. A poorly written VM program could fail to correctly constrain the relationships between inputs and outputs, leading to a valid proof for a logically invalid state transition (e.g., creating money out of thin air). **Security of the ZkVM relies on the correctness of the programs executed within it.**

## 4. Auditing & Verification Checklist

When reviewing a ZkVM program for security, auditors should verify that:

-   Every input `Value` is properly consumed.
-   The total value of outputs is correctly constrained to be equal to the total value of inputs (unless issuance or burning is explicitly and correctly modeled).
-   All secret variables are properly constrained (e.g., with range proofs).
-   There are no logical paths that could result in a non-empty stack at the end of execution.

---
> &laquo; [Previous: Instruction Set](./INSTRUCTIONS.md) | [Home](./README.md)
