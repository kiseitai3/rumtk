# Build Instructions

This file provides instructions for building the project. 

# General Requirements

For the project to successfully compile, you need:

- rustup/ Rust toolchain
- C++ toolchain
- libfuzzer installed
- Python 3.x 

# Notes

- In the future, Python support will be moved behind a feature guard. The Python support here is to enable exposing the 
`V2Message` objects in Python so scripts can process them and return results.
- `libfuzzer` enables the fuzzer targets to test the code for vulnerabilities.
- C++ toolchain is needed because Rust itself is a frontend to `LLVM` and the project uses `MiMalloc` as an alternative, performance oriented allocator to the system allocator.
- On Windows, use PowerShell to run the cargo commands.
- On Windows, you can test the parser by running `cat .\examples\hl7\path_report_enterprisehealth.hl7 | .\target\release\rumtk-hl7-v2-parse` in a PowerShell instance.
- On Linux, you can test the parser by running `cat ./examples/hl7/path_report_enterprisehealth.hl7 | ./target/release/rumtk-hl7-v2-parse` in a terminal.

# Instructions

1. Download [rustup](https://rust-lang.org/tools/install/)
2. On Windows, install the C++ toolchain from [Visual Studio](https://visualstudio.microsoft.com/downloads/).
3. On Linux, install CLang from package manager.
4. Install Python 3.x from package manager or download it from [Python.org](https://www.python.org/downloads/).
5. Install the `nightly` toolchain using `rustup install nightly` or `rustup toolchain install nightly`.
6. Install the `llvm-tools` with `rustup component add llvm-tools --toolchain nightly`.
7. Install `cargo-fuzz` with `cargo install cargo-fuzz`.
8. On Ubuntu systems, install `python3-dev` with `sudo apt install python3-dev`.
9. Navigate to the `rumtk` project directory.
10. Run `cargo build --release` to build the project.

