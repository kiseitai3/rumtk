# Rumtk-HL7-V2

[![Build Status](https://github.com/kiseitai3/rumtk/actions/workflows/check.yml/badge.svg)](https://github.com/kiseitai3/rumtk/actions/workflows/check.yml) [![Crates.io](https://img.shields.io/crates/l/rumtk-hl7-v2)](LICENSE-LGPL) [![Crates.io](https://img.shields.io/crates/v/rumtk-hl7-v2)](https://crates.io/crates/rumtk-hl7-v2) [![Released API docs](https://docs.rs/rumtk-hl7-v2/badge.svg)](https://docs.rs/rumtk-hl7-v2) [![Maintenance](https://img.shields.io/maintenance/yes/2025)](https://github.com/kiseitai3/rumtk)

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

- [ ] HL7 v2 Library
    - [ ] Parser
        - [x] HL7 v2 Sanitize Segment Separator (\n\r, \n) => \r
        - [x] HL7 v2 Escape Sequences Support
        - [x] HL7 v2 Repeating Field Support
        - [x] HL7 v2 UTF-8/Unicode Support
        - [x] HL7 v2 Encodings to UTF-8 Conversion
        - [x] Hl7 v2 Message to JSON Serialization Support
        - [x] HL7 v2 Component Searching (\<segment\>(\<subgroup\>)\<field\>\[sub_field\].\<component\>)
        - [x] HL7 v2 Primitive Types
        - [ ] HL7 v2 Component Optionality
        - [ ] HL7 v2 Complex Types (aka structs)
        - [ ] HL7 v2 Tables + Validation
        - [ ] HL7 v2 Complex traits
        - [ ] HL7 v2 Base Message to HL7 Type casting
        - [ ] HL7 v2 Custom Message Overrides
        - [ ] HL7 v2 Message Validator
        - [x] HL7 v2 Message to ASCII Message Generation
    - [x] MLLP
        - [x] Protocol
        - [x] Client
        - [x] Server
        - [x] Tests
    - [ ] Fuzz Targets

# Contributing

In its initial stages, I will be pushing code directly to the main branch. Once basic functionality has been stablished,
everyone including myself is required to open an issue for discussions, fork the project, and open a PR under your own
feature or main branch. I kindly ask you include a battery of unit tests with your PR to help protect the project
against regressions. Any contributions are very appreciated.
