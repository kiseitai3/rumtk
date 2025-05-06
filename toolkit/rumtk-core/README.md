# rumtk

[![Build Status](https://github.com/kiseitai3/rumtk/actions/workflows/rust.yaml/badge.svg)](https://github.com/kiseitai3/rumtk/actions/workflows/rust.yaml) [![Crates.io](https://img.shields.io/crates/l/rumtk-core)](LICENSE-LGPL) [![Crates.io](https://img.shields.io/crates/v/rumtk-core)](https://crates.io/crates/rumtk-core) [![Released API docs](https://docs.rs/rumtk-core/badge.svg)](https://docs.rs/rumtk-core) [![Maintenance](https://img.shields.io/maintenance/yes/2025)](https://github.com/kiseitai3/rumtk)

Rust's Universal Medical Toolkit is a toolkit being developed to put together a set of tools and libraries to facilitate
communication and automation in medicine.

# Goal

+ To create a simple toolkit with the necessary libraries, dependencies, and utilities for managing bridging HL7 V2
  Medical IT infrastructure to FHIR based systems.
+ Also, I would like the project to be accessible to hospitals to enable interoperability between systems. I plan to
  package it for package managers and containers.
+ The toolkit will focus on increasing security and simplicity with the first step taken by starting the project using
  Rust.
+ The toolkit shall foster reliability and will make attempts to be as strictly standards compliant as possible.
  Strictness may be relaxed later once the project sees use in the wild.

# Features

- [ ] Toolkit Core Library
    - [x] UTF-8 Support
    - [x] Small String Optimization Support
    - [x] String Encodings to UTF-8 Support
    - [x] RegEx Support
    - [x] Memory Cache Support
    - [x] Network Core library
    - [ ] Cryptologic Adapters
    - [ ] Commandline Standard Argument Interface
    - [ ] Enable STDIN|Pipe Functionality
    - [ ] Toolkit Core Tests
    - [x] Async support
    - [x] Multithreaded support
    - [ ] SQLite Bindings
    - [ ] PyO3 for extending message processing
    - [ ] SIMD support?? [Maybe]

# Contributing

In its initial stages, I will be pushing code directly to the main branch. Once basic functionality has been stablished,
everyone including myself is required to open an issue for discussions, fork the project, and open a PR under your own
feature or main branch. I kindly ask you include a battery of unit tests with your PR to help protect the project
against regressions. Any contributions are very appreciated.
