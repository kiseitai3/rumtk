# rumtk-v2-interface

[![Build Status](https://github.com/kiseitai3/rumtk/actions/workflows/check.yml/badge.svg)](https://github.com/kiseitai3/rumtk/actions/workflows/check.yml) [![Crates.io](https://img.shields.io/crates/l/rumtk-v2-interface)](LICENSE-GPL3) [![Crates.io](https://img.shields.io/crates/v/rumtk-v2-interface)](https://crates.io/crates/rumtk-v2-interface) [![Released API docs](https://docs.rs/rumtk-v2-interface/badge.svg)](https://docs.rs/rumtk-v2-interface) [![Maintenance](https://img.shields.io/maintenance/yes/2025)](https://github.com/kiseitai3/rumtk)

Using RUMTK, this is a utility that implements the steps for receiving and sending HL7 v2 messages. This utility does
not cast the incoming message to the data types defined by the HL7 specification nor does it apply the validation rules
and tables. That will be handled by a different utility.

# Goal

+ To provide a basic interface utility that serves as client and server for transacting v2 messages.
+ Have a utility that can be used on the terminal as part of other projects or a more complex pipeline.
+ Provide basic parsing of HL7 v2 messages.

# Features

- [ ] HL7 v2 Interface
    - [x] Listener
    - [x] Client
    - [x] Basic parsing of v2 message from pipes to `V2Message` type.
    - [x] Basic generation of v2 message from `V2Message` to pipes format.
    - [ ] Allow reading of JSON or HL7 messages
    - [ ] Tests
    - [ ] Fuzz Targets

# Contributing

In its initial stages, I will be pushing code directly to the main branch. Once basic functionality has been stablished,
everyone including myself is required to open an issue for discussions, fork the project, and open a PR under your own
feature or main branch. I kindly ask you include a battery of unit tests with your PR to help protect the project
against regressions. Any contributions are very appreciated.
