# ZkVM Specification: Operations

> Part of the ZkVM Specification.  
> &laquo; [VM Instructions](./INSTRUCTIONS.md) | [Home](./README.md) | [Next: Security &raquo;](./SECURITY.md)

## VM operation

This document outlines the operational model of the ZkVM, including its internal state, execution lifecycle, and how it handles deferred cryptographic operations for efficiency.

### VM state

The ZkVM state consists of the static attributes and the state machine attributes.

1. [Script Transaction](add link here):
    * `program`
    * `cs_proof`
    * `Input`
    * `Output`
2. Data stack (array of [items](#types))
3. Program stack (array of [programs](#program-type) with their offsets)
4. Current [program](#program-type) with its offset
5. [Deferred point operations](#deferred-point-operations)
6. [Constraint system](#constraint-system)


### VM execution

The VM is initialized with the following state:

1. [Transaction](#....) as provided by the user.
2. Data stack is loaded with
   * Parse the [Inputs](#...) and the corresponding [Outputs](#...) to detect types
   * If the `Input` is `Coin`, load the data stack with corresponding `Memo` Output
   * If the `Input` is `Memo`, the stack is loaded with the input values and data
   * If the `Input` is `State`, load stack with `OutputState`(#...) followed by `InputState`(#...)
   * `tx_data` is loaded if present. 

3. Current program set to the transaction program; with zero offset.
4. Array of deferred point operations is empty.
5. Constraint system: empty, with [transcript](#transcript) initialized with label `ZkVM.r1cs`:
    ```
    r1cs_transcript = Transcript("ZkVM.r1cs")
    ```

Then, the VM executes the current program till completion:

1. Each instruction is read at the current program offset, including its immediate data (if any).
2. Program offset is advanced immediately after reading the instruction to the next instruction.
3. The instruction is executed per [specification below](#instructions). If the instruction fails, VM exits early with an error result.
4. If the offset is less than the current program’s length, a new instruction is read (go back to step 1).
5. If the program stack is empty, the execution is considered _finalized_ and VM successfully exits.

If the execution finishes successfully, VM performs the finishing tasks:

1. Checks if the stack is empty; fails otherwise.
2. Computes a verification statement for [constraint system proof](#constraint-system-proof).
3. Executes all [deferred point operations](#deferred-point-operations) using a single multi-scalar multiplication. Fails if the result is not an identity point.


### Deferred point operations

VM defers operations on [points](#point) till the end of the VM execution in order
to batch them with the verification of [constraint system proof](#constraint-system-proof).

Each deferred operation at index `i` represents a statement:
```
0  ==  sum{s[i,j]·P[i,j], for all j}  +  a[i]·B  +  b[i]·B2
```
where:
1. `{s[i,j],P[i,j]}` is an array of ([scalar](#scalar-value),[point](#point)) tuples,
2. `a[i]` is a [scalar](#scalar-value) weight of a [primary base point](#base-points) `B`,
3. `b[i]` is a [scalar](#scalar-value) weight of a [secondary base point](#base-points) `B2`.

All such statements are combined using the following method:

1. For each statement, a random [scalar](#scalar-value) `x[i]` is sampled.
2. Each weight `s[i,j]` is multiplied by `x[i]` for all weights per statement `i`:
    ```
    z[i,j] = x[i]·s[i,j]
    ```
3. All weights `a[i]` and `b[i]` are independently added up with `x[i]` factors:
    ```
    a = sum{a[i]·x[i]}
    b = sum{b[i]·x[i]}
    ```
4. A single multi-scalar multiplication is performed to verify the combined statement:
    ```
    0  ==  sum{z[i,j]·P[i,j], for all i,j}  +  a·B  +  b·B2
    ```

---
> &laquo; [Instructions](./INSTRUCTIONS.md) | [Home](./README.md) | [Next: Security &raquo;](./SECURITY.md)    

