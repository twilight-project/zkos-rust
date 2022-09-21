# Signatures: Engineering design doc

This is a signature scheme for signing messages. 
This design doc describes the protocol for signing a single message with one public key 
(where the public key can be created from a single party's private key, 
or from the aggregation of multiple public keys),
and for signing multiple messages with multiple public keys.
The public key aggregation and multi-message signing protocols are implemented from the paper,
["Simple Schnorr Multi-Signatures with Applications to Bitcoin"](https://eprint.iacr.org/2018/068.pdf).



## Definitions

### Scalar

A _scalar_ is an integer modulo [Ristretto group](https://ristretto.group) order 
`|G| = 2^252 + 27742317777372353535851937790883648493`.

Scalars are encoded as 32-byte strings using little-endian convention.

Every scalar is required to be in a canonical (reduced) form.

### Point

A _point_ is an element in the [Ristretto group](https://ristretto.group).

Points are encoded as _compressed Ristretto points_ (32-byte strings).


### Base point

Ristretto base point in compressed form:

```
B = e2f2ae0a6abc4e71a884a961c500515f58e30b6aa582dd8db6a65945e08d2d76
```

### MusigContext

This is a public trait with functions:
- `commit(&self, &mut transcript)`: takes a mutable transcript, and commits the internal context to the transcript.
- `challenge(&self, index, &mut transcript) -> Scalar`: takes the index of a public key
and a mutable transcript, and returns the suitable challenge for that public key from the transcript. 
- `len(&self) -> usize`: returns the number of pubkeys associated with the context.
- `key(&self, index: usize)`: returns the key at `index`.


### Multimessage

Implements MusigContext

Fields:
- pairs: `Vec<(VerificationKey, &[u8])>`

Functions:
- `Multimessage::new(Vec<(VerificationKey, &[u8])>) -> Self`: creates a new Multimessage instance using the input.

- `Multimessage::commit(&self, &mut transcript)`: 
  It commits to the number of pairs, with label "Musig.Multimessage". 
  It then commits each of the pairs in `self.pairs`, by iterating through `self.pairs` and 
  committing the `VerificationKey` with label "X" and the message with label "m".

- `Multimessage::challenge(&self, i, &mut transcript) -> Scalar`: 
  Computes challenge `c_i = H(R, <S>, i)`, where `i` is the index of the public key 
  that it is getting a challenge for. The function expects that the nonce commitment sum `R`, 
  and the pairs `<S>`, have already been committed to the input `transcript`.

  It forks the input transcript by cloning it. It commits `i` to the forked transcript with label "i".
  It then gets and returns the challenge scalar `c_i` from the forked transcript with label "c".

- `Multimessage::len(&self) -> usize`: returns the length of `self.pairs`.

- `Multimessage:key(&self, index) -> VerificationKey`: returns the key at that index in `self.pairs`.


### Signature

A signature is comprised of a scalar `s`, and a RistrettoPoint `R`. 
In the simple Schnorr signature case, `s` represents the Schnorr signature scalar and `R` represents the nonce commitment. 
In the Musig signature case, `s` represents the sum of the Schnorr signature scalars of each party, or `s = sum_i (s_i)`. 
`R` represents the sum of the nonce commitments of each party, or `R = sum_i (R_i)`. 

Functions:
- `Signature::sign_single(...) -> Signature`
- `Signature::sign_multi(...) -> Result<Signature, MusigError>`
For more detail, see the [signing](#signing) section.

- `Signature::verify(...) -> DeferredVerification`
- `Signature::verify_multi(...) -> DeferredVerification`
For more detail, see the [verification](#verifying) section.

## Operations


### Signing

There are several paths to signing:
1. Make a Schnorr signature with one public key (derived from one private key).
    Function: `Signature::sign_single(...)`

    Input: 
    - transcript: `&mut Transcript` - a transcript to which the message to be signed has already been committed.
    - privkey: `Scalar`

    Operation:
    - Clone the transcript state, mix it with the privkey and system-provided RNG to generate the nonce `r`. 
    This makes the nonce uniquely bound to a message and private key, and also makes it non-deterministic to prevent "rowhammer" attacks.
    - Use the nonce to create a nonce commitment `R = r * G`
    - Make `c = H(X, R, m)`. Because `m` has already been fed into the transcript externally, 
    we do this by committing `X = privkey * G` to the transcript with label "X", 
    committing `R` to the transcript with label "R", and getting the challenge scalar `c` with label "c".
    - Make `s = r + c * x` where `x = privkey`

    Output:
    - Signature { `s`, `R` }


2. Make a Schnorr signature with multiple public keys and multiple messages, in a way that is safe from Russell's attack.
    - Create a `Multimessage` context by calling `Multimessage::new(...)`. 
      See the [multimessage](#multimessage) section for more details.

    For each signer that is taking part in the signing:
    - Call `Signer::new(transcript, privkey, multimessage)`.
    - All following steps are the same as in protocol #2.

### Verifying

There are several paths to verifying: 
1. Normal Schnorr signature verification (covers cases #1 and #2 in the [signing section](#signing)).
    Function: `Signature::verify(...)`

    Input: 
    - `&self`
    - transcript: `&mut Transcript` - a transcript to which the signed message has already been committed.
    - X: `VerificationKey`

    Operation:
    - Make `c = H(X, R, m)`. Since the transcript already has the message `m` committed to it, 
    the function only needs to commit `X` with label "X" and `R` with label "R", 
    and then get the challenge scalar `c` with label "c".
    - Make the `DeferredVerification` operation that checks if `s * G == R + c * P`. 
    `G` is the [base point](#base-point).

    Output:
    - `DeferredVerification` of the point operations to compute to check for validity.

2. Multi-message Schnorr signature verification (covers case #2 in [signing section](#signing)).
    Function: `Signature::verify_multi(...)`

    Input: 
    - `&self`
    - transcript: `&mut Transcript` - a transcript to which the signed message has already been committed.
    - messages: `Vec<(VerificationKey, &[u8])>` 

    Operation:
    - Make a `Multimessage` instance from `messages`, and call `commit()` on it to commit its state 
    to the transcript. 
    - Commit `self.R` to the transcript with label "R".
    - Use `multimessage.challenge(pubkey, &mut transcript)` to get the per-pubkey challenge `c_i` for each party.
    - Make the `DeferredVerification` operation that checks the linear combination: `s * G = R + sum{c_i * X_i}`.

    Output:
    - `DeferredVerification` of the point operations to compute to check for validity.
