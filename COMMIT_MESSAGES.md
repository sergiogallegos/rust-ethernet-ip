# Git Commit Messages for v0.6.0 Release

## Commit 1: Version bump and core library updates

```
chore: bump version to 0.6.0 and update core library documentation

- Update Cargo.toml version from 0.5.5 to 0.6.0
- Update src/lib.rs header and changelog to reflect v0.6.0
- Add comprehensive documentation for new UdtData format
- Update read_tag(), write_tag(), and read_udt_chunked() documentation
- Add v0.6.0 migration notes and usage examples
- Fix clippy warnings (manual range contains)

Breaking Changes:
- UDT format changed from HashMap<String, PlcValue> to UdtData struct
- All UDT operations now use generic UdtData with symbol_id and raw bytes
```

## Commit 2: Update Rust examples for new UDT API

```
refactor(examples): update all Rust examples to use new UdtData format

- Update test_udt_structure.rs to use UdtData instead of HashMap
- Update all rust_examples/*.rs files for new UDT API:
  * test_gtracking_udt.rs
  * test_part_data_enhanced.rs
  * test_part_data_chunked.rs
  * test_part_data_udt.rs
  * test_udt_multiple_members.rs
  * test_real_udt.rs
  * test_udt_chunked.rs
  * test_program_tag_out_fuse.rs
  * enhanced_udt_demo.rs
  * generic_udt_demo.rs
  * data_types_showcase.rs
- Replace HashMap-based UDT access with UdtData parsing
- Update examples to show symbol_id and raw data bytes
- Add notes about using UDT definitions for member access
```

## Commit 3: Update integration tests for new UDT API

```
refactor(tests): update integration tests for new UdtData format

- Update integration_test.rs to use UdtData instead of HashMap
- Update comprehensive_test.rs for new UDT format
- Update udt_enhanced_parsing_tests.rs for UdtData
- Update udt_enhanced_tests.rs for new API
- Update udt_data_tests.rs (already uses UdtData correctly)
- Fix test methods visibility (build_element_id_segment, etc.)
- All tests now verify symbol_id and data bytes instead of HashMap access
```

## Commit 4: Update README and documentation

```
docs: update README and documentation for v0.6.0

- Update README.md version badge to 0.6.0
- Add v0.6.0 features section highlighting generic UDT format
- Update changelog section with v0.6.0 as current version
- Update roadmap to reflect Phase 3 completion
- Update performance section with v0.6.0 improvements
- Create docs/VERSION_0.6.0_CHANGELOG.md with migration guide
- Update docs/LIBRARY_CHECK_SUMMARY.md with current status
- Update all dates to 2026-01-03
```

## Commit 5: Update example applications (C#, WPF, ASP.NET, Go+Next.js)

```
feat(examples): update all example applications with new features

C# Examples:
- Update WinForms example with RoutePath, Array, and UDT support
- Update WPF example with MVVM pattern and new features
- Update ASP.NET example with new API endpoints

Go + Next.js Example:
- Update Go backend with array and UDT endpoints
- Update Next.js frontend with new API integration
- Add CORS support and fix API field naming

All examples now support:
- RoutePath configuration
- Array element read/write
- UDT read/write operations
- UDT member access
```

## Alternative: Single comprehensive commit

If you prefer a single commit for the entire v0.6.0 release:

```
release: v0.6.0 - Generic UDT Format and Library Health Improvements

Major Changes:
- Introduce generic UdtData format replacing HashMap-based UDTs
- Update all examples and tests for new UDT API
- Comprehensive documentation updates
- All 31 unit tests passing

Breaking Changes:
- UDT format changed from HashMap<String, PlcValue> to UdtData struct
- UdtData contains symbol_id and raw bytes instead of parsed members
- Use UdtData::parse() with UDT definition to access members

New Features:
- Generic UDT handling works with any UDT without prior knowledge
- Better performance with raw bytes format
- Enhanced documentation with migration guide

Updated Files:
- Core library (src/lib.rs): Updated documentation and UDT handling
- All Rust examples: Updated for new UDT API
- All integration tests: Updated for new format
- README and docs: Comprehensive v0.6.0 documentation
- Example applications: C#, WPF, ASP.NET, Go+Next.js updated

See docs/VERSION_0.6.0_CHANGELOG.md for detailed migration guide.
```

