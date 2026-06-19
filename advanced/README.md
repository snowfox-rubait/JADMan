# JADM Advanced Features

This directory contains advanced features for JADM that are legally sensitive in some jurisdictions.

**DISCLAIMER: These tools are for personal use and interoperability testing only. Ensure that using these tools is legal in your jurisdiction before proceeding. The authors of JADM are not responsible for any misuse of these tools.**

## Contents

- `cdm/`: Frida-based Content Decryption Module (CDM) key extractor.
- `extension-hooks/`: Browser extension hooks for advanced stream interception.

## Installation

1. Copy scripts from `cdm/` to the `scripts/` directory in the root of the JADM project if you want to use CDM features.
2. Build `jadm-daemon` with the `cdm` feature: `cargo build --features cdm`.
