# Rex Development Tasks

## Recently Completed ✅

### Platform Filtering for Multi-Architecture Images
- [x] Added `parse_platform()` helper function for parsing platform strings
- [x] Implemented platform filtering in `get_image_inspect()`
- [x] Updated `handle_image_inspect()` to pass platform parameter
- [x] Added 5 unit tests for `parse_platform()` function
- [x] Added 3 integration tests for multi-platform filtering scenarios
- [x] Committed implementation (commit: b543553)
- [ ] **PENDING**: Commit test additions

**Testing Coverage:**
- Test: Multi-platform image without --platform (helpful error)
- Test: Multi-platform image with valid --platform (correct fetch)
- Test: Multi-platform image with invalid platform (error listing)
- Test: Platform parsing (os/arch, os/arch/variant formats)

**Test Results:**
- 128 rex tests passing (added 3 new integration tests)
- 359 total tests passing across project
- All clippy checks passing
- Code properly formatted

---

## Recently Completed ✅ (Continued)

### Raw Manifest/Config Display Flags
- [x] Implemented `--raw-manifest` flag for raw OCI manifest JSON output
- [x] Implemented `--raw-config` flag for raw config JSON output
- [x] Added optional fields to ImageInspect struct with skip_serializing
- [x] Updated get_image_inspect() to accept and handle raw flags
- [x] Updated handle_image_inspect() to output raw JSON when requested
- [x] Added 3 integration tests for raw flag scenarios
- [x] Committed implementation (commit: 2932882)

**Test Results:**
- 131 rex tests passing (added 3 new tests)
- 362 total tests passing across project
- All clippy checks passing
- Code properly formatted

---

## Pending Features (From Design Docs)

### 1. TUI Mode (Large Feature)
**Priority:** Medium
**Effort:** Large (~2000 lines)

Complete terminal UI implementation:
- Main TUI loop with ratatui + crossterm
- Threading model (UI thread + worker threads)
- Message passing via mpsc channels
- Views: images, tags, details, registry selector, help
- Keybindings (standard + vim mode)
- Real-time fuzzy search
- Theme support (dark/light)
- Non-blocking I/O via workers

**Reference:** rex/design.md Part 3 (lines 2337-2610)

### 2. OS Keyring Integration (Medium)
**Priority:** Low
**Effort:** ~200 lines

Currently uses file-based credential storage only.

**Implementation:**
- Add keyring crate dependency
- Integrate macOS Keychain
- Integrate Linux Secret Service
- Integrate Windows Credential Manager
- Fallback to file-based storage

**Reference:** librex/src/auth/notes.md:110

### 3. HTTP Retry-After Header (Small)
**Priority:** Low
**Effort:** ~20 lines

Parse and respect Retry-After header for rate limiting.

**Location:** librex/src/client/mod.rs:783

---

## Documentation Tasks

- [ ] Update README with platform filtering examples
- [ ] Add usage examples for multi-platform images
- [ ] Document platform string format (os/arch, os/arch/variant)

---

## Code Quality

### Current Status
- ✅ All tests passing (362 total)
- ✅ Zero clippy warnings
- ✅ Code properly formatted
- ✅ Comprehensive test coverage for new features

### Bisectability
- ✅ Latest commit builds successfully
- ✅ Latest commit passes all tests
- ✅ Latest commit passes linting
- ✅ All features fully committed

---

## Next Steps

### Immediate
1. Add documentation/examples for platform filtering
2. Add documentation/examples for raw manifest/config flags

### Short Term
1. Implement HTTP Retry-After header parsing
2. Consider OS keyring integration for production use

### Long Term
1. Evaluate TUI mode implementation priority
2. Plan and scope TUI architecture

---

## Notes

- Platform filtering is fully functional and well-tested
- Raw manifest/config flags are fully functional and well-tested
- All core CLI features from design docs are implemented except TUI mode
- Only small enhancement features remain (HTTP Retry-After, OS keyring)
- Project follows dev.md guidelines (TDD, incremental commits, bisectability)
- Code coverage is excellent with both unit and integration tests
- CLI is feature-complete for basic registry operations
