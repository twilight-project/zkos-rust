[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue)](/LICENSE)
<!-- CI badge example; uncomment when you have CI -->
<!-- [![CI](https://github.com/twilight-project/zkos-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/twilight-project/zkos-rust/actions/workflows/ci.yml) -->

**Status:** experimental ⛏️ – APIs may break before v1.0.

> **Origin:** Portions adapted from the  
> [`stellar/slingshot`](https://github.com/stellar/slingshot/tree/main/musig) project (Apache-2.0).

 mulmsgsig

Multi-message multi-signature scheme for Ristretto255 with batch verification support.

## Overview

This crate provides a pure Rust implementation of multi-message multi-signatures over the Ristretto255 curve. It allows a single signature to verify multiple messages, each signed by a different key, with support for efficient batch verification.

## Features

- **Multi-message signatures**: Sign multiple messages with different keys in a single signature
- **Batch verification**: Efficiently verify multiple signatures together
- **Transcript-based API**: Uses Merlin transcripts for domain separation and security
- **Ristretto255 curve**: Built on the secure Ristretto255 elliptic curve
- **Integration**: Seamlessly integrates with the `starsig` crate


## Usage

### Basic Multi-Message Signing

```rust
use mulmsgsig::Multisignature;
use starsig::{Signature, VerificationKey};
use merlin::Transcript;
use curve25519_dalek::scalar::Scalar;

// Create signing keys
let privkey1 = Scalar::from(1u64);
let privkey2 = Scalar::from(2u64);
let pubkey1 = VerificationKey::from_secret(&privkey1);
let pubkey2 = VerificationKey::from_secret(&privkey2);

// Create messages for each key
let messages = vec![
    (pubkey1, b"message1"),
    (pubkey2, b"message2"),
];

// Sign multiple messages with multiple keys
let mut transcript = Transcript::new(b"example");
let signature = Signature::sign_multi(
    vec![privkey1, privkey2],
    messages.clone(),
    &mut transcript,
).unwrap();

// Verify the multi-message signature
let mut verify_transcript = Transcript::new(b"example");
assert!(signature.verify_multi(&mut verify_transcript, messages).is_ok());
```

### Batch Verification

```rust
use mulmsgsig::Multisignature;
use starsig::{BatchVerifier, Signature, VerificationKey};
use merlin::Transcript;
use curve25519_dalek::scalar::Scalar;

// Create multiple signatures
let mut batch = BatchVerifier::new();
let mut transcript = Transcript::new(b"batch_example");
// Add signatures to batch
for (i, (privkey, message)) in privkeys.iter().zip(messages.iter()).enumerate() {
    let pubkey = VerificationKey::from_secret(privkey);
    let signature = Signature::sign_multi(
        vec![*privkey],
        vec![(pubkey, *message)],
        &mut transcript.clone(),
    ).unwrap();
    
    signature.verify_multi_batched(
        &mut transcript.clone(),
        vec![(pubkey, *message)],
        &mut batch,
    );
}

// Verify all signatures at once
assert!(batch.verify().is_ok());
```

## API Reference

### Traits

- `Multisignature`: Extension trait for multi-message signature operations
- `MusigContext`: Context management for multi-signature schemes
- `TranscriptProtocol`: Transcript extensions for multi-message protocols

### Types

- `Multimessage<M>`: Multi-message context implementation
- `MusigError`: Error types for multi-message operations

### Key Functions

- `Signature::sign_multi()`: Create multi-message signatures
- `Signature::verify_multi()`: Verify multi-message signatures
- `Signature::verify_multi_batched()`: Add to batch verification


## Minimum Supported Rust Version

Rust **1.70** or newer.

---

## License & Attribution

Licensed under [`Apache-2.0`](../../LICENSE).  
Portions derived from Stellar’s **Slingshot** project (Apache-2.0).
