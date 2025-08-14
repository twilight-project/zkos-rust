# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.1.0] - 2025-07-20

### Added
- **Initial public release** of `address` crate.
- Complete crate-level & API documentation (`#![deny(missing_docs)]` enabled).
- Apache 2.0 license.
- Comprehensive test suite (hex, script encoding, byte-serialisation).
- README with overview, examples and licence.

### Changed
- Parsing methods now return `Result` instead of panicking; all unwraps removed.
- Standardised error handling on `&'static str`.

### Fixed
- Type mismatches and memory-safety edge cases uncovered by new tests.

### Known Issues
- Script addresses cannot be recreated from hex (by design).  
- Some edge cases in public-key validation may need more coverage.
---

<!-- No compare link: initial release -->
