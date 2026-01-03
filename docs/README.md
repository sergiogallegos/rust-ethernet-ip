# Documentation Index

This directory contains documentation for the Rust EtherNet/IP library, including protocol references, implementation guides, and testing documentation.

## Protocol Reference Documentation

### CIP Protocol Reference
- **`CIP_PROTOCOL_REFERENCE_1756-PM020.md`** - Complete CIP protocol reference extracted from Allen-Bradley Publication 1756-PM020
  - CIP Service Request/Response Format
  - Segment Encoding (Logical and Symbolic)
  - Read/Write Tag Services
  - Fragmented Services
  - Data Type Codes

### PDF Extraction Summary
- **`PDF_EXTRACTION_SUMMARY.md`** - Summary of information extracted from 1756-PM020 PDF
  - Pages documented
  - Key findings
  - Open questions
  - Next steps

## Implementation Guides

### Array Implementation
- **`ARRAY_ELEMENT_ADDRESSING_GUIDE.md`** - Detailed guide for implementing proper array element addressing
  - Current issues
  - Correct implementation approaches
  - Code examples
  - Testing requirements

### Implementation Status
- **`IMPLEMENTATION_STATUS.md`** - Current status of UDT and Array implementations
  - UDT: ✅ Complete
  - Array: ❌ Needs fixes
  - Open questions
  - Priority list

## Review and Analysis Documents

### Array Review
- **`../ARRAY_IMPLEMENTATION_REVIEW.md`** - Detailed analysis of current array implementation issues
- **`../ARRAY_FIX_IMPLEMENTATION_PLAN.md`** - Step-by-step plan for fixing array issues
- **`../COMPREHENSIVE_REVIEW_SUMMARY.md`** - Executive summary of library review

### UDT Documentation
- **`../TESTING_UDT_CHANGES.md`** - Comprehensive testing guide for UDT changes
- **`../QUICK_TEST_GUIDE.md`** - Quick reference for testing

## Key Information Quick Reference

### Service Codes
| Service | Request | Reply |
|---------|---------|-------|
| Read Tag | `0x4C` | `0xCC` |
| Write Tag | `0x4D` | `0xCD` |
| Read Tag Fragmented | `0x52` | `0xD2` |
| Write Tag Fragmented | `0x53` | `0xD3` |

### Segment Types
| Segment | Value | Purpose |
|---------|-------|---------|
| ANSI Extended Symbolic | `0x91` | Tag names |
| 8-bit Element ID | `0x28` | Array element addressing |
| 16-bit Element ID | `0x29` | Array element addressing |
| 32-bit Element ID | `0x2A` | Array element addressing |

### Data Type Codes
| Type | Code | Size |
|------|------|------|
| BOOL | `0x00C1` | 1 byte |
| SINT | `0x00C2` | 1 byte |
| INT | `0x00C3` | 2 bytes |
| DINT | `0x00C4` | 4 bytes |
| REAL | `0x00CA` | 4 bytes |
| UDT | `0x00A0` | Variable |

## Quick Links

- **Protocol Reference:** `CIP_PROTOCOL_REFERENCE_1756-PM020.md`
- **Array Implementation Guide:** `ARRAY_ELEMENT_ADDRESSING_GUIDE.md`
- **Implementation Status:** `IMPLEMENTATION_STATUS.md`
- **PDF Summary:** `PDF_EXTRACTION_SUMMARY.md`

## Related Files

- **Source Code:** `../src/lib.rs`, `../src/tag_path.rs`, `../src/udt.rs`
- **Tests:** `../tests/udt_data_tests.rs`
- **Examples:** `../examples/rust_examples/test_udt_data_format.rs`

---

**Note:** This documentation is based on Allen-Bradley Publication 1756-PM020: Logix Controller Access Data (pages 13-29, 63 partial). Additional sections may be needed to complete the array element addressing format specification.

