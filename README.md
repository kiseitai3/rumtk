# rumtk

Rust's Universal Medical Toolkit is a toolkit being developed to put together a set of tools and libraries to facilitate
communication and automation in medicine.

# Components

## Rumtk-core

[![Build Status](https://github.com/kiseitai3/rumtk/actions/workflows/check.yml/badge.svg)](https://github.com/kiseitai3/rumtk/actions/workflows/check.yml) [![Crates.io](https://img.shields.io/crates/l/rumtk-core)](LICENSE-LGPL) [![Crates.io](https://img.shields.io/crates/v/rumtk-core)](https://crates.io/crates/rumtk-core) [![Released API docs](https://docs.rs/rumtk-core/badge.svg)](https://docs.rs/rumtk-core) [![Maintenance](https://img.shields.io/maintenance/yes/2025)](https://github.com/kiseitai3/rumtk)

## Rumtk-HL7-V2

[![Build Status](https://github.com/kiseitai3/rumtk/actions/workflows/check.yml/badge.svg)](https://github.com/kiseitai3/rumtk/actions/workflows/check.yml) [![Crates.io](https://img.shields.io/crates/l/rumtk-hl7-v2)](LICENSE-LGPL) [![Crates.io](https://img.shields.io/crates/v/rumtk-hl7-v2)](https://crates.io/crates/rumtk-hl7-v2) [![Released API docs](https://docs.rs/rumtk-hl7-v2/badge.svg)](https://docs.rs/rumtk-hl7-v2) [![Maintenance](https://img.shields.io/maintenance/yes/2025)](https://github.com/kiseitai3/rumtk)

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
    - [x] JSON Serialization Support
    - [ ] Cryptologic Adapters
    - [ ] Commandline Standard Argument Interface
    - [ ] Enable STDIN|Pipe Functionality
    - [ ] Toolkit Core Tests
    - [x] Async support
    - [x] Multithreaded support
    - [ ] SQLite Bindings
    - [ ] PyO3 for extending message processing
    - [ ] SIMD support?? [Maybe]
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
        - [ ] HL7 v2 Message to ASCII Message Generation
    - [x] MLLP
        - [x] Protocol
        - [x] Client
        - [x] Server
        - [x] Tests
    - [ ] Fuzz Targets
- [ ] MLLP Interfaces
    - [ ] HL7 v2 Endpoint Interface utility
    - [ ] HL7 v2 Client Interface utility
    - [ ] HL7 v2 Tests
- [ ] HL7 v2 to FHIR Conversion
    - [ ] HL7 v2 to FHIR Basic Mapping
    - [ ] HL7 v2 to FHIR Custom Mappings
    - [ ] HL7 v2 to FHIR Conversion utility
    - [ ] HL7 v2 to FHIR Conversion Tests
- [ ] HL7 v2 to DB [Maybe]
    - [ ] HL7 v2 Python bindings
    - [ ] HL7 v2 to DB utility
    - [ ] HL7 v2 to DB Tests
- [ ] HL7 v2 Dashboard [Maybe]
    - [ ] HL7 v2 Message History DB
    - [ ] HL7 v2 Dashboard service
    - [ ] HL7 v2 Dashboard Tests
- [ ] HL7 FHIR Interface
    - [ ] HL7 FHIR CRUD
    - [ ] HL7 FHIR Client
    - [ ] HL7 FHIR Client utility
    - [ ] HL7 FHIR Tests
- [ ] HIFLAMES utility => HL7 Interface - FHIR Loader And Message Exporter System
- [ ] Package [Maybe]
    - [ ] Fedora
    - [ ] Ubuntu
    - [ ] Arch
    - [ ] Docker

# Contributing

- [ ] Create an issue ticket if it has not been done.
- [ ] Fork this repository.
- [ ] Create your own feature.
- [ ] Include the ticket number in feature branch name.
- [ ] Open PR when ready to submit code for review.
- [ ] Include an appropriate set of tests to help protect against regressions. I don't require full TDD, but some basic
  testing is useful. Some of those tests should be doctests if applicable (introducing new APIs).

If coming from SIIM, welcome!
Follow the steps above to start contributing.

Any contributions are very appreciated. 
