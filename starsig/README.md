[![Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue)](/LICENSE)
<!-- CI badge example; uncomment when you have CI -->
<!-- [![CI](https://github.com/twilight-project/zkos-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/twilight-project/zkos-rust/actions/workflows/ci.yml) -->

**Status:** experimental ⛏️ – APIs may break before v1.0.

> **Origin:** Portions adapted from the  
> [`stellar/slingshot`](https://github.com/stellar/slingshot/tree/main/starsig) project (Apache-2.0).

# Starsig: schnorr signatures on Ristretto

Implementation of a simple Schnorr signature protocol
implemented with [Ristretto](https://ristretto.group) and [Merlin transcripts](https://merlin.cool).

* [Specification](docs/spec.md)

## Features

* Simple message-based API.
* Flexible [transcript](https://merlin.cool)-based API.
* Single signature verification.
* Batch signature verification.
* Compatible with [Musig](../musig) API.
* Compatible with [Keytree](../keytree) key derivation API.
* VRF (aka “HMAC verifiable by a public key”) is in development.

## Authors

* [Oleg Andreev](https://github.com/oleganza)
* [Cathie Yun](https://github.com/cathieyun)



## Minimum Supported Rust Version

Rust **1.70** or newer.

---

## License & Attribution

Licensed under [`Apache-2.0`](../../LICENSE).  
Portions derived from Stellar’s **Slingshot** project (Apache-2.0).
