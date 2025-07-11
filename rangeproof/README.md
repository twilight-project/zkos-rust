# RangeProof 

RangeProof is a gadget for generating R1CS constrainted range proof assets based on the [Bulletproofs](https://crypto.stanford.edu/bulletproofs/) zero-knowledge proof system.

## WARNING

This code is still research-quality. It is not (yet) suitable for deployment.

## Tests 

Run tests with `cargo test`.

## Benchmarks

This crate uses [criterion.rs][criterion] for benchmarks. Run
benchmarks with `cargo bench`.

## About

This is a research project sponsored by [Interstellar][interstellar],
developed by Henry de Valence, Cathie Yun, and Oleg Andreev.


[bp_website]: https://crypto.stanford.edu/bulletproofs/
[bp_repo]: https://github.com/dalek-cryptography/bulletproofs/
[interstellar]: https://interstellar.com/
[criterion]: https://github.com/japaric/criterion.rs
