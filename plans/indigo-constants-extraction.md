# INDIGO Constants Extraction

## Overview

Extract well-known Device and Property names from the upstream INDIGO library (sys/externals/indigo) to replace hardcoded constants in the codebase. This work is split into two phases: first, automating the generation of props.rs from INDIGO headers, and second, replacing all hardcoded property name strings throughout the codebase with references to the generated constants.

## Current State

- A ~1255 line [`props.rs`](../props.rs:1) file exists at the project root containing property name constants
- Hardcoded property name strings exist in various files (e.g., [`src/strategies/rs/`](../src/strategies/rs/), [`src/strategies/ffi.rs`](../src/strategies/ffi.rs:1))
- Manual maintenance required when INDIGO adds new standard properties or devices
- Risk of inconsistency with upstream INDIGO naming conventions and between hardcoded strings

## Goals

### Phase 1: Automated props.rs Generation

1. Automatically extract standard property names from INDIGO C headers
2. Generate props.rs as part of the build process (likely in build.rs)
3. Keep props.rs under version control (generate once, commit, regenerate when INDIGO updates)
4. Ensure generated constants match INDIGO's naming conventions exactly

### Phase 2: Replace Hardcoded Strings

1. Audit all code in [`src/strategies/rs/`](../src/strategies/rs/) and [`src/strategies/ffi.rs`](../src/strategies/ffi.rs:1) for hardcoded property name strings
2. Replace hardcoded strings with references to constants from props.rs
3. Ensure type safety and consistency across the codebase
4. Verify all property references use the generated constants

## Implementation Approach

### Phase 1: Automated props.rs Generation

This phase focuses on extracting property names from INDIGO C headers and generating props.rs automatically.

#### Step 1: Analysis

- [ ] Identify all INDIGO header files containing standard property names (likely in sys/externals/indigo/indigo_libs/)
- [ ] Document the naming patterns and conventions used in INDIGO for property definitions
- [ ] Analyze the current props.rs structure to understand the desired output format
- [ ] Determine which property names, device interface names, and property item names need extraction

#### Step 2: Build Script Development

- [ ] Create or extend build.rs to parse INDIGO C headers
- [ ] Extract device interface names (e.g., INDIGO_INTERFACE_CAMERA, INDIGO_INTERFACE_MOUNT)
- [ ] Extract standard property names (e.g., CONNECTION, INFO, CONFIG, CCD_EXPOSURE, MOUNT_EQUATORIAL_COORDINATES)
- [ ] Extract standard property item names (e.g., CONNECTED, DISCONNECTED, RA, DEC)
- [ ] Generate Rust constant declarations with proper documentation and formatting
- [ ] Handle edge cases and special naming conventions

#### Step 3: Generation Strategy

- [ ] Implement props.rs generation as part of build.rs
- [ ] Generate props.rs in the project root (current location)
- [ ] Keep props.rs under version control (commit after generation)
- [ ] Add build script logic to detect when INDIGO headers change
- [ ] Document the regeneration process for when INDIGO is updated

#### Step 4: Validation

- [ ] Add tests to verify generated constant values match INDIGO exactly
- [ ] Compare generated props.rs with current manual version
- [ ] Ensure no constants are lost in the transition
- [ ] Verify formatting and documentation quality

### Phase 2: Replace Hardcoded Strings

This phase focuses on replacing all hardcoded property name strings with references to the generated constants.

#### Step 1: Audit Codebase

- [ ] Search [`src/strategies/rs/`](../src/strategies/rs/) for hardcoded property name strings
- [ ] Search [`src/strategies/ffi.rs`](../src/strategies/ffi.rs:1) for hardcoded property name strings
- [ ] Document all locations where property names are hardcoded
- [ ] Identify patterns of property name usage (e.g., in match statements, property lookups, etc.)

#### Step 2: Systematic Replacement

- [ ] Replace hardcoded strings in [`src/strategies/rs/client.rs`](../src/strategies/rs/client.rs:1)
- [ ] Replace hardcoded strings in [`src/strategies/rs/protocol.rs`](../src/strategies/rs/protocol.rs:1)
- [ ] Replace hardcoded strings in [`src/strategies/rs/protocol_json.rs`](../src/strategies/rs/protocol_json.rs:1)
- [ ] Replace hardcoded strings in [`src/strategies/rs/protocol_negotiation.rs`](../src/strategies/rs/protocol_negotiation.rs:1)
- [ ] Replace hardcoded strings in [`src/strategies/rs/transport.rs`](../src/strategies/rs/transport.rs:1)
- [ ] Replace hardcoded strings in [`src/strategies/ffi.rs`](../src/strategies/ffi.rs:1)
- [ ] Replace hardcoded strings in [`src/strategies/async_ffi.rs`](../src/strategies/async_ffi.rs:1)

#### Step 3: Add Imports

- [ ] Add `use crate::props::*;` or specific imports where needed
- [ ] Ensure all files have access to the generated constants
- [ ] Verify no naming conflicts with existing code

#### Step 4: Testing and Verification

- [ ] Run all existing tests to ensure functionality is preserved
- [ ] Add tests to verify constants are used correctly
- [ ] Check for any remaining hardcoded property name strings
- [ ] Verify type safety improvements from using constants

#### Step 5: Documentation

- [ ] Document the replacement process and rationale
- [ ] Update code comments to reference constants instead of literal strings
- [ ] Add migration notes for future developers

## Key Files

### Phase 1 Files

- [`props.rs`](../props.rs:1) - Current manual constants file (~1255 lines)
- [`build.rs`](../build.rs:1) - Build script for generation logic
- `sys/externals/indigo/indigo_libs/indigo_bus.h` - INDIGO header with standard names
- `sys/externals/indigo/indigo_libs/indigo_driver.h` - Additional INDIGO definitions

### Phase 2 Files

- [`src/strategies/rs/client.rs`](../src/strategies/rs/client.rs:1)
- [`src/strategies/rs/protocol.rs`](../src/strategies/rs/protocol.rs:1)
- [`src/strategies/rs/protocol_json.rs`](../src/strategies/rs/protocol_json.rs:1)
- [`src/strategies/rs/protocol_negotiation.rs`](../src/strategies/rs/protocol_negotiation.rs:1)
- [`src/strategies/rs/transport.rs`](../src/strategies/rs/transport.rs:1)
- [`src/strategies/ffi.rs`](../src/strategies/ffi.rs:1)
- [`src/strategies/async_ffi.rs`](../src/strategies/async_ffi.rs:1)

## Success Criteria

### Phase 1 Success Criteria

- ✅ All standard INDIGO property names are automatically extracted from headers
- ✅ Generated props.rs matches the structure and content of the current manual version
- ✅ Build process successfully generates props.rs from INDIGO headers
- ✅ props.rs is kept under version control
- ✅ Documentation explains how to regenerate when INDIGO is updated
- ✅ Tests verify generated constant values match INDIGO exactly

### Phase 2 Success Criteria

- ✅ No hardcoded property name strings remain in [`src/strategies/rs/`](../src/strategies/rs/)
- ✅ No hardcoded property name strings remain in [`src/strategies/ffi.rs`](../src/strategies/ffi.rs:1)
- ✅ All property references use constants from props.rs
- ✅ All existing tests pass with the new constants
- ✅ Type safety is improved through consistent constant usage
- ✅ Code is more maintainable with centralized property name definitions

## Dependencies

- Phase 2 depends on Phase 1 being completed first
- The trait-based device API (planned for v0.3.0) depends on both phases being completed

## References

- INDIGO Protocol Documentation: `sys/externals/indigo/indigo_docs/PROTOCOLS.md`
- INDIGO Source: `sys/externals/indigo/`
- Current props.rs: [`props.rs`](../props.rs:1)
- Build script: [`build.rs`](../build.rs:1)
