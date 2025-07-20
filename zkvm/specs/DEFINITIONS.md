# ZkVM Specification: Types

> Part of the ZkVM Specification.  
> &laquo; [ZkVM Types](./TYPES.md) | [Home](./README.md) | [Next:Instructions &raquo;](./INSTRUCTIONS.md)
---

## Definitions

This document defines the core cryptographic primitives and data structures that form the foundation of the ZkVM. These definitions are essential for understanding the security and operation of the virtual machine.

### LE32

A non-negative 32-bit integer encoded using little-endian convention.
Used to encode lengths of [strings](#string-type),  e.g., stack indices.

### LE64

A non-negative 64-bit integer encoded using little-endian convention.
Used to encode [value quantities](#value-type).


### Scalar value

A _scalar_ is an integer modulo [Ristretto group](https://ristretto.group) order `|G| = 2^252 + 27742317777372353535851937790883648493`.

Scalars are encoded as 32-byte [strings](#string-type) using little-endian convention.

Every scalar in the VM is guaranteed to be in a canonical (reduced) form: an instruction that operates on a scalar
checks if the scalar is canonical.


### Point

A _point_ is an element in the [Ristretto group](https://ristretto.group).

Points are encoded as 32-byte [strings](#string-type) in _compressed Ristretto form_.

Each point in the VM is guaranteed to be a valid Ristretto point.


### Base points

ZkVM defines two base points: primary `B` and secondary `B2`.

```
B  = e2f2ae0a6abc4e71a884a961c500515f58e30b6aa582dd8db6a65945e08d2d76
B2 = hash-to-ristretto255(SHA3-512(B))
```

Both base points are orthogonal (the discrete log between them is unknown)
and used in [Pedersen commitments](#pedersen-commitment).


### Pedersen commitment

Pedersen commitment to a secret [scalar](#scalar-value)
is defined as a point with the following structure:

```
P = Com(v, f) = v·B + f·B2
```

where:

* `P` is a point representing commitment,
* `v` is a secret scalar value being committed to,
* `f` is a secret blinding factor (scalar),
* `B` and `B2` are [base points](#base-points).

Pedersen commitments can be used to allocate new [variables](#variable-type) using the [`commit`](#commit) instruction.

Pedersen commitments can be opened using the [`unblind`](#unblind) instruction.


### Transcript

Transcript is an instance of the [Merlin](https://doc.dalek.rs/merlin/) construction,
which is itself based on [STROBE](https://strobe.sourceforge.io/) and [Keccak-f](https://keccak.team/keccak.html)
with 128-bit security parameter.

Transcript is used throughout ZkVM to generate challenge [scalars](#scalar-value) and commitments.

Transcripts have the following operations, each taking a label for domain separation:

1. **Initialize** transcript:
    ```
    T := Transcript(label)
    ```
2. **Append bytes** of arbitrary length prefixed with a label:
    ```
    T.append(label, bytes)
    ```
3. **Challenge bytes**
    ```    
    T.challenge_bytes<size>(label) -> bytes
    ```
4. **Challenge scalar** is defined as generating 64 challenge bytes and reducing the 512-bit little-endian integer modulo Ristretto group order `|G|`:
    ```    
    T.challenge_scalar(label) -> scalar
    T.challenge_scalar(label) == T.challenge_bytes<64>(label) mod |G|
    ```

Labeled instances of the transcript can be precomputed
to reduce number of Keccak-f permutations to just one per challenge.


### Constraint system

The constraint system is the part of the [VM state](#vm-state) that implements
[Bulletproof's rank-1 constraint system](https://doc-internal.dalek.rs/develop/bulletproofs/notes/r1cs_proof/index.html).

It also keeps track of the [variables](#variable-type) and [constraints](#constraint-type),
and is used to verify the [constraint system proof](#constraint-system-proof).


### Constraint system proof

A proof of satisfiability of a [constraint system](#constraint-system) built during the VM execution.

The proof is provided to the VM at the beginning of execution and verified when the VM is [finished](#vm-execution).


### Merkle binary tree

The construction of a merkle binary tree is based on the [RFC 6962 Section 2.1](https://tools.ietf.org/html/rfc6962#section-2.1)
with hash function replaced with a [transcript](#transcript).

Leafs and nodes in the tree use the same instance of a transcript provided by the upstream protocol:

```
T = Transcript(<label>)
```

The hash of an empty list is a 32-byte challenge string with the label `merkle.empty`:

```
MerkleHash(T, {}) = T.challenge_bytes("merkle.empty")
```

The hash of a list with one entry (also known as a leaf hash) is
computed by committing the entry to the transcript (defined by the item type),
and then generating 32-byte challenge string the label `merkle.leaf`:

```
MerkleHash(T, {item}) = {
    T.append(<field1 name>, item.field1)
    T.append(<field2 name>, item.field2)
    ...
    T.challenge_bytes("merkle.leaf")
}
```

For n > 1, let k be the largest power of two smaller than n (i.e., k < n ≤ 2k). The merkle hash of an n-element list is then defined recursively as:

```
MerkleHash(T, list) = {
    T.append("L", MerkleHash(list[0..k]))
    T.append("R", MerkleHash(list[k..n]))
    T.challenge_bytes("merkle.node")
}
```

Note that we do not require the length of the input list to be a power of two.
The resulting merkle binary tree may thus not be balanced; however,
its shape is uniquely determined by the number of leaves.

The Merkle binary tree is used to construct a Contract 

---
> &laquo; [VM Types](./TYPES.md) | [Home](./README.md) | [Next: Definitions &raquo;](./INSTRUCTIONS.md)
