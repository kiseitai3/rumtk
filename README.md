# rumtk

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
    - [ ] Network Core library
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
  - [ ] MLLP
    - [ ] Protocol
    - [ ] Client
    - [ ] Server
    - [ ] Tests
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

In its initial stages, I will be pushing code directly to the main branch. Once basic functionality has been stablished,
everyone including myself is required to open an issue for discussions, fork the project, and open a PR under your own
feature or main branch. I kindly ask you include a battery of unit tests with your PR to help protect the project
against regressions. Any contributions are very appreciated.
