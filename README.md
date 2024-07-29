# rumtk
Rust's Medical Toolkit is a toolkit being developed to put together a set of tools and libraries to facilitate communication and automation in medicine. 

# Goal
+ To create a simple toolkit with the necessary libraries, dependencies, and utilities for managing standard tasks in Medical IT. 
+ Also, I would like the project to make it more accessible for hospital to enable interoperability between systems. 
+ The toolkit will focus on increasing security and simplicity with the first step taken by starting the project using Rust.
+ The toolkit shall foster reliability and will make attempts to be standards compliant.

# Milestones
- [ ] Network Core library
- [ ] Cryptologic Adapters
- [ ] Commandline Standard Argument Interface
- [ ] Toolkit Core Library
- [ ] Toolkit Core Tests
- [ ] HL7 v2 Interface Protocol
- [x] HL7 v2 Sanitize Segment Separator (\n\r, \n) => \r
- [x] HL7 v2 Escape Sequences Support
- [x] HL7 v2 Repeating Field Support
- [x] HL7 v2 UTF-8/Unicode Support
- [ ] HL7 v2 Custom Message Configuration
- [ ] HL7 v2 Endpoint Interface utility
- [ ] HL7 v2 Client Interface utility
- [ ] HL7 v2 Tests
- [ ] HL7 v2 to FHIR Basic Mapping.
- [ ] HL7 v2 to FHIR Custom Mappings
- [ ] HL7 v2 to FHIR Conversion utility
- [ ] HL7 v2 to FHIR Conversion Tests
- [ ] HL7 FHIR CRUD
- [ ] HL7 FHIR Client
- [ ] HL7 FHIR Client utility
- [ ] HL7 FHIR Tests
- [ ] HIFLAMES utility => HL7 Interface - FHIR Loader And Message Exporter System

# Contributing
In its initial stages, I will be pushing code directly to the main branch. Once basic functionality has been stablished, everyone including myself is required to open an issue for discussions, fork the project, and open a PR under your own feature or main branch. I kindly ask you include a battery of unit tests with your PR to help protect the project against regressions. Any contributions are very appreciated.
