# zkos-address Crate

The `address` crate provides functionality for creating, parsing, and validating
ZkOS addresses. It supports different address types and networks, and provides
functionality for encoding and decoding addresses to and from various formats.

## Overview

This crate defines the following main types:

*   [`Network`](https://docs.rs/zkos-address/latest/zkos_address/enum.Network.html): Represents the network (Mainnet or Testnet) on which an address is valid.
*   [`AddressType`](https://docs.rs/zkos-address/latest/zkos_address/enum.AddressType.html): Represents the type of an address (Standard or Script).
*   [`Address`](https://docs.rs/zkos-address/latest/zkos_address/enum.Address.html): An enum that can be either a `Standard` or `Script` address.
*   [`Standard`](https://docs.rs/zkos-address/latest/zkos_address/struct.Standard.html): A standard address that is derived from a public key.
*   [`Script`](https://docs.rs/zkos-address/latest/zkos_address/struct.Script.html): A script address that is derived from a script hash.

## Usage

### Creating a Standard Address

```rust
use zkos_address::{Address, Network};
use quisquislib::ristretto::RistrettoPublicKey;
use curve25519_dalek::ristretto::CompressedRistretto;

let public_key_bytes = [0; 32];
let compressed_ristretto = CompressedRistretto(public_key_bytes);
let public_key = RistrettoPublicKey::from_compressed(compressed_ristretto).unwrap();
let address = Address::standard_address(Network::Mainnet, public_key);

println!("Standard Address: {}", address);
```

### Creating a Script Address

```rust
use zkos_address::{Address, Network};

let script_hash = [0; 32];
let address = Address::script_address(Network::Mainnet, script_hash);

println!("Script Address: {}", address.as_base58());
```

### Parsing an Address

```rust
use zkos_address::{Address, AddressType};

let address_str = "T16fJv9T1sH6qE7X9Z3vY4K2A8bC5D6E7F8G9H0J";
let address = Address::from_base58(address_str, AddressType::Standard);

// assert!(address.is_ok());
```

## Minimum Supported Rust Version

Rust **1.70** or newer.

## License

Licensed under [`Apache-2.0`](../../LICENSE). 
