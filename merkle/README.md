# zkos-merkle

Efficient, generic Merkle-tree utilities powering the **Twilight / ZkOS** stack.

[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue)](/LICENSE)
<!-- CI badge example; uncomment when you have CI -->
<!-- [![CI](https://github.com/twilight-project/zkos-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/twilight-project/zkos-rust/actions/workflows/ci.yml) -->

**Status:** experimental ⛏️ – APIs may break before v1.0.

> **Origin:** Portions adapted from the  
> [`stellar/slingshot`](https://github.com/stellar/slingshot/tree/main/merkle) project (Apache-2.0).


## Features

- Generic over item type (`MerkleItem`)
- Efficient root calculation and incremental building
- Merkle path & proof generation and verification
- Optional **serde** serialization
- `no_std` compatible (requires `alloc`)

## Installation

```toml
[dependencies]
merkle = { path = "../merkle" }   # inside the zkos‑rust workspace
```

When the crate is published on crates.io you’ll be able to use:

```toml
zkos-merkle = "0.1"
```
## Example

```rust
use merkle::{MerkleItem, MerkleTree, Hash}; // adjust to `zkos_merkle` if renamed

// Your data type must implement `MerkleItem`
#[derive(Clone)]
struct MyLeaf(Vec<u8>);
impl MerkleItem for MyLeaf {
    fn hash(&self) -> Hash { blake3::hash(&self.0).into() }
}

let leaves = vec![
    MyLeaf(b"alice".to_vec()),
    MyLeaf(b"bob".to_vec()),
    MyLeaf(b"carol".to_vec()),
];

let tree = MerkleTree::from_leaves(b"my.domain", &leaves);

// Root commitment
let root = tree.root();

// Generate proof that `alice` is in the tree
let proof = tree.open(0).unwrap();
assert!(proof.verify(b"my.domain", &root));


```


## Minimum Supported Rust Version

Rust **1.70** or newer.

## License

Licensed under [`Apache-2.0`](../../LICENSE).  
Portions derived from Stellar’s **Slingshot** project; adapted © 2025 by Twilight Project Contributors.

