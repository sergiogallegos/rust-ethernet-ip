#!/usr/bin/env python3
"""
Comprehensive Test for All Tags from PLC_TEST_TAG_DEFINITIONS.md

This test verifies that the Python wrapper can correctly:
1. Read all tags (controller and program-scoped)
2. Write new values to all tags
3. Read back and verify the writes were successful

Run with: python test_all_tags.py

Prerequisites:
- All tags from PLC_TEST_TAG_DEFINITIONS.md must exist in the PLC
- PLC must be accessible at 192.168.0.1:44818
- ControlLogix CPU in Slot 0 (or adjust CPU_SLOT constant)
"""

import asyncio
import sys
from typing import List, Tuple, Dict, Any, Optional

# Import the Python wrapper
try:
    from rust_ethernet_ip import EipClient, RoutePath, PlcValue
except ImportError:
    print("❌ Error: Could not import rust_ethernet_ip. Make sure the Python wrapper is installed.")
    print("   Install with: pip install -e ../pywrapper")
    sys.exit(1)

PLC_ADDRESS = "192.168.0.1:44818"
CPU_SLOT = 0  # ControlLogix CPU in Slot 0


class TestTag:
    def __init__(self, name: str, initial_value: Any, test_value: Any, description: str):
        self.name = name
        self.initial_value = initial_value
        self.test_value = test_value
        self.description = description


def values_match(actual: Any, expected: Any) -> bool:
    """Check if two values match (with tolerance for floats)."""
    if type(actual) != type(expected):
        return False
    
    if isinstance(actual, float) or isinstance(expected, float):
        return abs(float(actual) - float(expected)) < 0.001
    elif isinstance(actual, str):
        return actual == expected
    else:
        return actual == expected


async def read_tag(client: EipClient, tag_name: str, expected_type: Any) -> Any:
    """Read a tag from the PLC, trying to determine the type from the tag name."""
    if "DINT" in tag_name or "_DINT" in tag_name:
        value = await client.read_tag(tag_name)
        if hasattr(value, 'as_dint'):
            return value.as_dint()
        return value
    elif "REAL" in tag_name or "_REAL" in tag_name:
        value = await client.read_tag(tag_name)
        if hasattr(value, 'as_real'):
            return value.as_real()
        return value
    elif "BOOL" in tag_name or "_BOOL" in tag_name:
        value = await client.read_tag(tag_name)
        if hasattr(value, 'as_bool'):
            return value.as_bool()
        return value
    elif "INT[" in tag_name and "DINT" not in tag_name:
        value = await client.read_tag(tag_name)
        if hasattr(value, 'as_int'):
            return value.as_int()
        return value
    elif "STRING" in tag_name or "String" in tag_name:
        value = await client.read_tag(tag_name)
        if hasattr(value, 'as_string'):
            return value.as_string()
        return value
    else:
        # Default to DINT
        value = await client.read_tag(tag_name)
        if hasattr(value, 'as_dint'):
            return value.as_dint()
        return value


async def write_tag(client: EipClient, tag_name: str, value: Any) -> None:
    """Write a tag value to the PLC."""
    if isinstance(value, bool):
        await client.write_tag(tag_name, PlcValue.bool(value))
    elif isinstance(value, int):
        await client.write_tag(tag_name, PlcValue.dint(value))
    elif isinstance(value, float):
        await client.write_tag(tag_name, PlcValue.real(value))
    elif isinstance(value, str):
        await client.write_tag(tag_name, PlcValue.string(value))
    else:
        raise ValueError(f"Unsupported type: {type(value)}")


def create_test_tags() -> List[TestTag]:
    """Create all test tags from PLC_TEST_TAG_DEFINITIONS.md."""
    tags = []
    
    # Controller-Scoped Array Elements
    for i in range(10):
        tags.append(TestTag(
            name=f"gTestArray_DINT[{i}]",
            initial_value=(i + 1) * 10,
            test_value=1000 + (i * 111),
            description=f"Controller DINT array element {i}"
        ))
        
        tags.append(TestTag(
            name=f"gTestArray_REAL[{i}]",
            initial_value=(i + 1.0) * 1.1,
            test_value=10.0 + (i * 1.11),
            description=f"Controller REAL array element {i}"
        ))
        
        tags.append(TestTag(
            name=f"gTestArray_BOOL[{i}]",
            initial_value=(i % 2 == 0),
            test_value=(i % 2 == 1),
            description=f"Controller BOOL array element {i}"
        ))
        
        tags.append(TestTag(
            name=f"gTestArray_INT[{i}]",
            initial_value=(i + 1) * 100,
            test_value=1000 + (i * 111),
            description=f"Controller INT array element {i}"
        ))
    
    # Large DINT Array
    for idx in [100, 200, 300, 500, 999]:
        tags.append(TestTag(
            name=f"gTestArray_Large[{idx}]",
            initial_value=0,
            test_value=10000 + idx,
            description=f"Controller large DINT array element {idx} (16-bit index)"
        ))
    
    # Simple STRING tag
    tags.append(TestTag(
        name="gTest_STRING",
        initial_value="Initial String Value",
        test_value="Test String Write 789",
        description="Controller simple STRING tag (not UDT member)"
    ))
    
    # Controller-Scoped UDT Members
    tags.append(TestTag(
        name="gTestUDT.Member1_DINT",
        initial_value=100,
        test_value=7777,
        description="Controller UDT member: Member1_DINT"
    ))
    
    tags.append(TestTag(
        name="gTestUDT.Member2_REAL",
        initial_value=3.14159,
        test_value=77.77,
        description="Controller UDT member: Member2_REAL"
    ))
    
    tags.append(TestTag(
        name="gTestUDT.Member3_BOOL",
        initial_value=True,
        test_value=False,
        description="Controller UDT member: Member3_BOOL"
    ))
    
    tags.append(TestTag(
        name="gTestUDT.Member4_INT",
        initial_value=42,
        test_value=8888,
        description="Controller UDT member: Member4_INT"
    ))
    
    tags.append(TestTag(
        name="gTestUDT.Member5_String",
        initial_value="Hello PLC",
        test_value="Test String 123",
        description="Controller UDT member: Member5_String"
    ))
    
    # UDT Array_DINT - elements 0-9
    for i in range(10):
        tags.append(TestTag(
            name=f"gTestUDT.Array_DINT[{i}]",
            initial_value=i + 1,
            test_value=1000 + (i * 111),
            description=f"Controller UDT array member: Array_DINT[{i}]"
        ))
    
    # UDT Array elements 0-9 - Member1_DINT, Member2_REAL, Member3_BOOL, Member4_INT
    for i in range(10):
        tags.append(TestTag(
            name=f"gTestUDT_Array[{i}].Member1_DINT",
            initial_value=(i + 1) * 100,
            test_value=5000 + (i * 111),
            description=f"Controller UDT array element {i}, member Member1_DINT"
        ))
        
        tags.append(TestTag(
            name=f"gTestUDT_Array[{i}].Member2_REAL",
            initial_value=(i + 1.0) * 1.1,
            test_value=50.0 + (i * 1.11),
            description=f"Controller UDT array element {i}, member Member2_REAL"
        ))
        
        tags.append(TestTag(
            name=f"gTestUDT_Array[{i}].Member3_BOOL",
            initial_value=(i % 2 == 0),
            test_value=(i % 2 == 1),
            description=f"Controller UDT array element {i}, member Member3_BOOL"
        ))
        
        tags.append(TestTag(
            name=f"gTestUDT_Array[{i}].Member4_INT",
            initial_value=(i + 1) * 10,
            test_value=500 + (i * 11),
            description=f"Controller UDT array element {i}, member Member4_INT"
        ))
    
    # UDT Array elements 0-9 - Array_REAL[0-4] (sample a few)
    for i in range(10):
        for j in range(5):
            tags.append(TestTag(
                name=f"gTestUDT_Array[{i}].Array_REAL[{j}]",
                initial_value=(j + 1.0) * 1.1,
                test_value=10.0 + (i * 10.0) + (j * 1.11),
                description=f"Controller UDT array element {i}, nested array member Array_REAL[{j}]"
            ))
    
    # Program-Scoped Array Elements
    for i in range(10):
        tags.append(TestTag(
            name=f"Program:TestProgram.gTestArray_DINT[{i}]",
            initial_value=(i + 1) * 1000,
            test_value=10000 + (i * 1111),
            description=f"Program-scoped DINT array element {i}"
        ))
        
        tags.append(TestTag(
            name=f"Program:TestProgram.gTestArray_REAL[{i}]",
            initial_value=10.1 + (i * 10.1),
            test_value=100.0 + (i * 11.11),
            description=f"Program-scoped REAL array element {i}"
        ))
        
        tags.append(TestTag(
            name=f"Program:TestProgram.gTestArray_BOOL[{i}]",
            initial_value=(i % 2 == 1),
            test_value=(i % 2 == 0),
            description=f"Program-scoped BOOL array element {i}"
        ))
    
    # Simple STRING tag (program-scoped)
    tags.append(TestTag(
        name="Program:TestProgram.gTest_STRING",
        initial_value="Program Initial String",
        test_value="Program Test String Write 999",
        description="Program-scoped simple STRING tag (not UDT member)"
    ))
    
    # Program-Scoped UDT Members
    tags.append(TestTag(
        name="Program:TestProgram.gTestUDT.Member1_DINT",
        initial_value=500,
        test_value=55555,
        description="Program-scoped UDT member: Member1_DINT"
    ))
    
    tags.append(TestTag(
        name="Program:TestProgram.gTestUDT.Member2_REAL",
        initial_value=5.5,
        test_value=555.55,
        description="Program-scoped UDT member: Member2_REAL"
    ))
    
    tags.append(TestTag(
        name="Program:TestProgram.gTestUDT.Member3_BOOL",
        initial_value=False,
        test_value=True,
        description="Program-scoped UDT member: Member3_BOOL"
    ))
    
    tags.append(TestTag(
        name="Program:TestProgram.gTestUDT.Member4_INT",
        initial_value=24,
        test_value=9999,
        description="Program-scoped UDT member: Member4_INT"
    ))
    
    tags.append(TestTag(
        name="Program:TestProgram.gTestUDT.Member5_String",
        initial_value="Program UDT",
        test_value="Program Test String 456",
        description="Program-scoped UDT member: Member5_String"
    ))
    
    # Program UDT Array_DINT - elements 0-9
    for i in range(10):
        tags.append(TestTag(
            name=f"Program:TestProgram.gTestUDT.Array_DINT[{i}]",
            initial_value=i + 1,
            test_value=2000 + (i * 111),
            description=f"Program-scoped UDT array member: Array_DINT[{i}]"
        ))
    
    # Program UDT Array elements 0-4 - Member1_DINT, Member2_REAL, Member3_BOOL
    for i in range(5):
        tags.append(TestTag(
            name=f"Program:TestProgram.gTestUDT_Array[{i}].Member1_DINT",
            initial_value=(i + 1) * 200,
            test_value=6000 + (i * 111),
            description=f"Program-scoped UDT array element {i}, member Member1_DINT"
        ))
        
        tags.append(TestTag(
            name=f"Program:TestProgram.gTestUDT_Array[{i}].Member2_REAL",
            initial_value=(i + 1.0) * 2.2,
            test_value=60.0 + (i * 1.11),
            description=f"Program-scoped UDT array element {i}, member Member2_REAL"
        ))
        
        tags.append(TestTag(
            name=f"Program:TestProgram.gTestUDT_Array[{i}].Member3_BOOL",
            initial_value=(i % 2 == 1),
            test_value=(i % 2 == 0),
            description=f"Program-scoped UDT array element {i}, member Member3_BOOL"
        ))
    
    return tags


async def main():
    print("═══════════════════════════════════════════════════════════════════════════════")
    print("🔬 Comprehensive Test: All Tags from PLC_TEST_TAG_DEFINITIONS.md (Python Wrapper)")
    print("═══════════════════════════════════════════════════════════════════════════════")
    print()
    
    print(f"🔌 Connecting to ControlLogix PLC at {PLC_ADDRESS}...")
    print(f"   CPU Slot: {CPU_SLOT}")
    
    # Create route path for ControlLogix
    route = RoutePath().add_slot(CPU_SLOT)
    
    # Connect with route path
    client = await EipClient.connect_with_route(PLC_ADDRESS, route)
    print("✅ Connected successfully!\n")
    
    # Define all test tags
    test_tags = create_test_tags()
    
    total_tests = 0
    passed_tests = 0
    failed_tests = 0
    skipped_tests = 0
    
    # Track failures with error messages
    read_failures: List[Tuple[str, str]] = []
    write_failures: List[Tuple[str, str]] = []
    verify_failures: List[Tuple[str, str, Any, Any]] = []
    
    # Step 1: Read initial values
    print("═══════════════════════════════════════════════════════════════════════════════")
    print("📖 STEP 1: Reading Initial Values")
    print("═══════════════════════════════════════════════════════════════════════════════")
    print()
    
    initial_values: Dict[str, Any] = {}
    
    for tag in test_tags:
        total_tests += 1
        print(f"   Reading {tag.name}... ", end="", flush=True)
        try:
            value = await read_tag(client, tag.name, tag.initial_value)
            print(f"✅ {value}")
            initial_values[tag.name] = value
        except Exception as e:
            error_msg = str(e)
            print(f"❌ FAILED: {error_msg}")
            print("      ⚠️  Tag may not exist in PLC - skipping write/verify for this tag")
            read_failures.append((tag.name, error_msg))
            failed_tests += 1
            skipped_tests += 1
    
    print()
    print(f"📊 Step 1 Summary: {len(initial_values)} read, {len(test_tags) - len(initial_values)} failed")
    print()
    
    # Step 2: Write test values
    print("═══════════════════════════════════════════════════════════════════════════════")
    print("✏️  STEP 2: Writing Test Values")
    print("═══════════════════════════════════════════════════════════════════════════════")
    print()
    
    written_tags: List[str] = []
    
    for tag in test_tags:
        if tag.name not in initial_values:
            continue  # Skip tags that failed to read
        
        print(f"   Writing {tag.name} = {tag.test_value}... ", end="", flush=True)
        try:
            await write_tag(client, tag.name, tag.test_value)
            print("✅")
            written_tags.append(tag.name)
            
            # For STRING types, immediately read back to verify write
            if isinstance(tag.test_value, str):
                print(f"      Reading back {tag.name}... ", end="", flush=True)
                try:
                    read_value = await read_tag(client, tag.name, tag.test_value)
                    if values_match(read_value, tag.test_value):
                        print(f"✅ VERIFIED: {read_value}")
                    else:
                        print("⚠️  MISMATCH after write!")
                        print(f"         Expected: {tag.test_value}")
                        print(f"         Got:      {read_value}")
                except Exception as e:
                    print(f"❌ FAILED TO READ BACK: {e}")
        except Exception as ex:
            error_msg = str(ex)
            print(f"❌ FAILED: {error_msg}")
            write_failures.append((tag.name, error_msg))
            failed_tests += 1
    
    print()
    print(f"📊 Step 2 Summary: {len(written_tags)} written, {len(write_failures)} failed")
    print()
    
    # Step 3: Read back and verify writes
    print("═══════════════════════════════════════════════════════════════════════════════")
    print("🔍 STEP 3: Reading Back and Verifying Writes")
    print("═══════════════════════════════════════════════════════════════════════════════")
    print()
    
    for tag in test_tags:
        if tag.name not in written_tags:
            continue  # Skip tags that failed to write
        
        print(f"   Verifying {tag.name}... ", end="", flush=True)
        try:
            value = await read_tag(client, tag.name, tag.test_value)
            if values_match(value, tag.test_value):
                print(f"✅ {value}")
                passed_tests += 1
            else:
                print("❌ MISMATCH!")
                print(f"      Expected: {tag.test_value}")
                print(f"      Got:      {value}")
                verify_failures.append((tag.name, "Value mismatch", value, tag.test_value))
                failed_tests += 1
                
                # Enhanced STRING mismatch reporting
                if isinstance(value, str) and isinstance(tag.test_value, str):
                    print(f"      String Lengths: Expected {len(tag.test_value)}, Got {len(value)}")
                    expected_bytes = tag.test_value.encode('utf-8')
                    actual_bytes = value.encode('utf-8')
                    print(f"      Expected Bytes: {expected_bytes.hex()}")
                    print(f"      Actual Bytes:   {actual_bytes.hex()}")
        except Exception as ex:
            error_msg = str(ex)
            print(f"❌ FAILED: {error_msg}")
            verify_failures.append((tag.name, error_msg, None, tag.test_value))
            failed_tests += 1
    
    print()
    
    # Final Summary
    print("═══════════════════════════════════════════════════════════════════════════════")
    print("📊 FINAL RESULTS")
    print("═══════════════════════════════════════════════════════════════════════════════")
    print(f"   Total Tests:     {total_tests}")
    print(f"   ✅ Passed:         {passed_tests}")
    print(f"   ❌ Failed:         {failed_tests}")
    print(f"   ⏭️  Skipped:        {skipped_tests}")
    if total_tests > 0:
        print(f"   Success Rate:     {(passed_tests * 100.0 / total_tests):.1f}%")
    print()
    
    # Display failure summary
    if read_failures or write_failures or verify_failures:
        print("═══════════════════════════════════════════════════════════════════════════════")
        print("❌ FAILED TAGS SUMMARY")
        print("═══════════════════════════════════════════════════════════════════════════════")
        print()
        
        if read_failures:
            print(f"📖 READ FAILURES ({len(read_failures)} tags):")
            for tag_name, error in read_failures:
                print(f"   • {tag_name}: {error}")
            print()
        
        if write_failures:
            print(f"✏️  WRITE FAILURES ({len(write_failures)} tags):")
            # Group by error pattern
            error_groups: Dict[str, List[str]] = {}
            for tag_name, error in write_failures:
                if "0x2107" in error or "2107" in error:
                    if "_Array[" in tag_name and "." in tag_name:
                        error_key = "PLC does not support writing to UDT array element members directly (Error 0x2107)"
                    elif "Member5_String" in tag_name or tag_name.endswith(".Member5_String"):
                        error_key = "PLC does not support writing to STRING members in UDTs directly (Error 0x2107)"
                    elif tag_name == "gTest_STRING" or tag_name == "Program:TestProgram.gTest_STRING":
                        error_key = "PLC does not support writing to STRING tags directly (Error 0x2107)"
                    else:
                        error_key = f"Error 0x2107: {error}"
                else:
                    error_key = error
                
                if error_key not in error_groups:
                    error_groups[error_key] = []
                error_groups[error_key].append(tag_name)
            
            for error, tags in error_groups.items():
                print(f"   Error: {error}")
                print(f"   Affected tags ({len(tags)}):")
                if len(tags) <= 5:
                    for tag in tags:
                        print(f"     • {tag}")
                else:
                    for i in range(3):
                        print(f"     • {tags[i]}")
                    print(f"     • ... and {len(tags) - 3} more")
            print()
        
        if verify_failures:
            print(f"🔍 VERIFY FAILURES ({len(verify_failures)} tags):")
            for tag_name, error, actual, expected in verify_failures:
                print(f"   • {tag_name}: {error}")
                print(f"     Expected: {expected}, Got: {actual}")
            print()
        
        print("═══════════════════════════════════════════════════════════════════════════════")
        print()
    
    if failed_tests == 0 and skipped_tests == 0:
        print("🎉 ALL TESTS PASSED! The Python wrapper is working correctly.")
    elif failed_tests > 0:
        print("⚠️  Some tests failed. See the FAILED TAGS SUMMARY above for details.")
    else:
        print("ℹ️  Some tags were skipped (may not exist in PLC).")
    
    await client.unregister_session()


if __name__ == "__main__":
    asyncio.run(main())

