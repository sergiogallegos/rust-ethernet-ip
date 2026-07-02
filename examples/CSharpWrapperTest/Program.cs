using System;
using System.Collections.Generic;
using System.Diagnostics;
using RustEtherNetIp;

namespace CSharpWrapperTest
{
    /// <summary>
    /// Comprehensive Test for All Tags from PLC_TEST_TAG_DEFINITIONS.md
    /// 
    /// This test verifies that the C# wrapper can correctly:
    /// 1. Read all tags (controller and program-scoped)
    /// 2. Write new values to all tags
    /// 3. Read back and verify the writes were successful
    /// 
    /// Run with: dotnet run
    /// 
    /// Prerequisites:
    /// - All tags from PLC_TEST_TAG_DEFINITIONS.md must exist in the PLC
    /// - PLC must be accessible at 192.168.0.1:44818
    /// - ControlLogix CPU in Slot 0 (or adjust CPU_SLOT constant)
    /// </summary>
    class Program
    {
        struct TestTag
        {
            public string Name;
            public PlcValue InitialValue;
            public PlcValue TestValue;
            public string Description;
        }

        static void Main(string[] args)
        {
            var plcAddress = Environment.GetEnvironmentVariable("TEST_PLC_ADDRESS") ?? "192.168.0.1:44818";
            var cpuSlot = byte.TryParse(Environment.GetEnvironmentVariable("TEST_PLC_SLOT"), out var parsedSlot)
                ? parsedSlot
                : (byte)0;
            Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
            Console.WriteLine("🔬 Comprehensive Test: All Tags from PLC_TEST_TAG_DEFINITIONS.md (C# Wrapper)");
            Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
            Console.WriteLine();

            Console.WriteLine($"🔌 Connecting to ControlLogix PLC at {plcAddress}...");
            Console.WriteLine($"   CPU Slot: {cpuSlot}");

            using var client = new EtherNetIpClient();
            
            // Connect to PLC with RoutePath (for ControlLogix with CPU in specific slot)
            var routePath = new RoutePath().AddSlot(cpuSlot);
            if (!client.ConnectWithRoute(plcAddress, routePath))
            {
                Console.WriteLine("❌ Failed to connect to PLC");
                return;
            }

            Console.WriteLine($"✅ Connected successfully with RoutePath (CPU Slot {cpuSlot})!\n");

            // Define all test tags
            var testTags = CreateTestTags();

            int totalTests = 0;
            int passedTests = 0;
            int failedTests = 0;
            int skippedTests = 0;

            // Track failures with error messages
            var readFailures = new List<(string, string)>();
            var writeFailures = new List<(string, string)>();
            var verifyFailures = new List<(string, string, PlcValue, PlcValue)>();

            // Step 1: Read initial values
            Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
            Console.WriteLine("📖 STEP 1: Reading Initial Values");
            Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
            Console.WriteLine();

            var initialValues = new Dictionary<string, PlcValue>();

            foreach (var tag in testTags)
            {
                totalTests++;
                Console.Write($"   Reading {tag.Name}... ");
                try
                {
                    var value = ReadTag(client, tag.Name, tag.InitialValue);
                    Console.WriteLine($"✅ {value}");
                    initialValues[tag.Name] = value;
                }
                catch (Exception ex)
                {
                    var errorMsg = ex.Message;
                    Console.WriteLine($"❌ FAILED: {errorMsg}");
                    Console.WriteLine("      ⚠️  Tag may not exist in PLC - skipping write/verify for this tag");
                    readFailures.Add((tag.Name, errorMsg));
                    failedTests++;
                    skippedTests++;
                }
            }

            Console.WriteLine();
            Console.WriteLine($"📊 Step 1 Summary: {initialValues.Count} read, {testTags.Count - initialValues.Count} failed");
            Console.WriteLine();

            // Step 2: Write test values
            Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
            Console.WriteLine("✏️  STEP 2: Writing Test Values");
            Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
            Console.WriteLine();

            var writtenTags = new List<string>();

            foreach (var tag in testTags)
            {
                if (!initialValues.ContainsKey(tag.Name))
                {
                    continue; // Skip tags that failed to read
                }

                Console.Write($"   Writing {tag.Name} = {tag.TestValue}... ");
                try
                {
                    WriteTag(client, tag.Name, tag.TestValue);
                    Console.WriteLine("✅");
                    writtenTags.Add(tag.Name);

                    // For STRING types, immediately read back to verify write
                    if (tag.TestValue.Type == PlcValueType.String)
                    {
                        Console.Write($"      Reading back {tag.Name}... ");
                        try
                        {
                            var readValue = ReadTag(client, tag.Name, tag.TestValue);
                            if (ValuesMatch(readValue, tag.TestValue))
                            {
                                Console.WriteLine($"✅ VERIFIED: {readValue}");
                            }
                            else
                            {
                                Console.WriteLine("⚠️  MISMATCH after write!");
                                Console.WriteLine($"         Expected: {tag.TestValue}");
                                Console.WriteLine($"         Got:      {readValue}");
                            }
                        }
                        catch (Exception e)
                        {
                            Console.WriteLine($"❌ FAILED TO READ BACK: {e.Message}");
                        }
                    }
                }
                catch (Exception ex)
                {
                    var errorMsg = ex.Message;
                    Console.WriteLine($"❌ FAILED: {errorMsg}");
                    writeFailures.Add((tag.Name, errorMsg));
                    failedTests++;
                }
            }

            Console.WriteLine();
            Console.WriteLine($"📊 Step 2 Summary: {writtenTags.Count} written, {writeFailures.Count} failed");
            Console.WriteLine();

            // Step 3: Read back and verify writes
            Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
            Console.WriteLine("🔍 STEP 3: Reading Back and Verifying Writes");
            Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
            Console.WriteLine();

            foreach (var tag in testTags)
            {
                if (!writtenTags.Contains(tag.Name))
                {
                    continue; // Skip tags that failed to write
                }

                Console.Write($"   Verifying {tag.Name}... ");
                try
                {
                    var value = ReadTag(client, tag.Name, tag.TestValue);
                    if (ValuesMatch(value, tag.TestValue))
                    {
                        Console.WriteLine($"✅ {value}");
                        passedTests++;
                    }
                    else
                    {
                        Console.WriteLine("❌ MISMATCH!");
                        Console.WriteLine($"      Expected: {tag.TestValue}");
                        Console.WriteLine($"      Got:      {value}");
                        verifyFailures.Add((tag.Name, "Value mismatch", value, tag.TestValue));
                        failedTests++;

                        // Enhanced STRING mismatch reporting
                        if (value.Type == PlcValueType.String && tag.TestValue.Type == PlcValueType.String)
                        {
                            var actualStr = value.As<string>();
                            var expectedStr = tag.TestValue.As<string>();
                            Console.WriteLine($"      String Lengths: Expected {expectedStr.Length}, Got {actualStr.Length}");
                            var expectedBytes = System.Text.Encoding.UTF8.GetBytes(expectedStr);
                            var actualBytes = System.Text.Encoding.UTF8.GetBytes(actualStr);
                            Console.WriteLine($"      Expected Bytes: {BitConverter.ToString(expectedBytes)}");
                            Console.WriteLine($"      Actual Bytes:   {BitConverter.ToString(actualBytes)}");
                        }
                    }
                }
                catch (Exception ex)
                {
                    var errorMsg = ex.Message;
                    Console.WriteLine($"❌ FAILED: {errorMsg}");
                    verifyFailures.Add((tag.Name, errorMsg, PlcValue.Dint(0), tag.TestValue));
                    failedTests++;
                }
            }

            Console.WriteLine();

            // Final Summary
            Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
            Console.WriteLine("📊 FINAL RESULTS");
            Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
            Console.WriteLine($"   Total Tests:     {totalTests}");
            Console.WriteLine($"   ✅ Passed:         {passedTests}");
            Console.WriteLine($"   ❌ Failed:         {failedTests}");
            Console.WriteLine($"   ⏭️  Skipped:        {skippedTests}");
            Console.WriteLine($"   Success Rate:     {(passedTests * 100.0 / totalTests):F1}%");
            Console.WriteLine();

            // Display failure summary
            if (readFailures.Count > 0 || writeFailures.Count > 0 || verifyFailures.Count > 0)
            {
                Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
                Console.WriteLine("❌ FAILED TAGS SUMMARY");
                Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
                Console.WriteLine();

                if (readFailures.Count > 0)
                {
                    Console.WriteLine($"📖 READ FAILURES ({readFailures.Count} tags):");
                    foreach (var (tagName, error) in readFailures)
                    {
                        Console.WriteLine($"   • {tagName}: {error}");
                    }
                    Console.WriteLine();
                }

                if (writeFailures.Count > 0)
                {
                    Console.WriteLine($"✏️  WRITE FAILURES ({writeFailures.Count} tags):");
                    // Group by error pattern
                    var errorGroups = new Dictionary<string, List<string>>();
                    foreach (var (tagName, error) in writeFailures)
                    {
                        string errorKey;
                        if (error.Contains("0x2107") || error.Contains("2107"))
                        {
                            if (tagName.Contains("_Array[") && tagName.Contains('.'))
                            {
                                errorKey = "PLC does not support writing to UDT array element members directly (Error 0x2107)";
                            }
                            else if (tagName.Contains("Member5_String") || tagName.EndsWith(".Member5_String"))
                            {
                                errorKey = "PLC does not support writing to STRING members in UDTs directly (Error 0x2107)";
                            }
                            else
                            {
                                errorKey = $"Error 0x2107: {error}";
                            }
                        }
                        else
                        {
                            errorKey = error;
                        }

                        if (!errorGroups.ContainsKey(errorKey))
                        {
                            errorGroups[errorKey] = new List<string>();
                        }
                        errorGroups[errorKey].Add(tagName);
                    }

                    foreach (var (error, tags) in errorGroups)
                    {
                        Console.WriteLine($"   Error: {error}");
                        Console.WriteLine($"   Affected tags ({tags.Count}):");
                        if (tags.Count <= 5)
                        {
                            foreach (var tag in tags)
                            {
                                Console.WriteLine($"     • {tag}");
                            }
                        }
                        else
                        {
                            for (int i = 0; i < 3; i++)
                            {
                                Console.WriteLine($"     • {tags[i]}");
                            }
                            Console.WriteLine($"     • ... and {tags.Count - 3} more");
                        }
                    }
                    Console.WriteLine();
                }

                if (verifyFailures.Count > 0)
                {
                    Console.WriteLine($"🔍 VERIFY FAILURES ({verifyFailures.Count} tags):");
                    foreach (var (tagName, error, actual, expected) in verifyFailures)
                    {
                        Console.WriteLine($"   • {tagName}: {error}");
                        Console.WriteLine($"     Expected: {expected}, Got: {actual}");
                    }
                    Console.WriteLine();
                }

                Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
                Console.WriteLine();
            }

            if (failedTests == 0 && skippedTests == 0)
            {
                Console.WriteLine("🎉 ALL TESTS PASSED! The C# wrapper is working correctly.");
            }
            else if (failedTests > 0)
            {
                Console.WriteLine("⚠️  Some tests failed. See the FAILED TAGS SUMMARY above for details.");
            }
            else
            {
                Console.WriteLine("ℹ️  Some tags were skipped (may not exist in PLC).");
            }
        }

        static PlcValue ReadTag(EtherNetIpClient client, string tagName, PlcValue expectedType)
        {
            // Try to determine type from tag name - check data type suffixes FIRST
            // This ensures UDT members like "gTestUDT.Member1_DINT" are read as DINT, not UDT
            
            if (tagName.Contains("STRING") || tagName.Contains("String") || tagName.EndsWith(".Member5_String"))
            {
                return PlcValue.String(client.ReadString(tagName));
            }
            else if (tagName.Contains("_DINT") || tagName.Contains("DINT["))
            {
                return PlcValue.Dint(client.ReadDint(tagName));
            }
            else if (tagName.Contains("_REAL") || tagName.Contains("REAL["))
            {
                return PlcValue.Real(client.ReadReal(tagName));
            }
            else if (tagName.Contains("_BOOL") || tagName.Contains("BOOL["))
            {
                return PlcValue.Bool(client.ReadBool(tagName));
            }
            else if ((tagName.Contains("_INT") || tagName.Contains("INT[")) && !tagName.Contains("DINT"))
            {
                return PlcValue.Int(client.ReadInt(tagName));
            }
            else if (tagName.Contains("UDT") && !tagName.Contains(".") && !tagName.Contains("["))
            {
                // Only read as UDT if it's a full UDT tag (not a member, not an array element)
                return client.ReadUdt(tagName);
            }
            else
            {
                // Default to DINT for unknown types
                return PlcValue.Dint(client.ReadDint(tagName));
            }
        }

        static void WriteTag(EtherNetIpClient client, string tagName, PlcValue value)
        {
            switch (value.Type)
            {
                case PlcValueType.Bool:
                    client.WriteBool(tagName, value.As<bool>());
                    break;
                case PlcValueType.Dint:
                    client.WriteDint(tagName, value.As<int>());
                    break;
                case PlcValueType.Real:
                    client.WriteReal(tagName, value.As<float>());
                    break;
                case PlcValueType.Int:
                    client.WriteInt(tagName, value.As<short>());
                    break;
                case PlcValueType.String:
                    client.WriteString(tagName, value.As<string>());
                    break;
                case PlcValueType.Udt:
                    client.WriteUdt(tagName, value);
                    break;
                default:
                    throw new Exception($"Unsupported type: {value.Type}");
            }
        }

        static bool ValuesMatch(PlcValue actual, PlcValue expected)
        {
            if (actual.Type != expected.Type)
            {
                return false;
            }

            switch (actual.Type)
            {
                case PlcValueType.Bool:
                    return actual.As<bool>() == expected.As<bool>();
                case PlcValueType.Dint:
                    return actual.As<int>() == expected.As<int>();
                case PlcValueType.Real:
                    var a = actual.As<float>();
                    var e = expected.As<float>();
                    return Math.Abs(a - e) < 0.001f;
                case PlcValueType.Int:
                    return actual.As<short>() == expected.As<short>();
                case PlcValueType.String:
                    return actual.As<string>() == expected.As<string>();
                default:
                    return actual.ToString() == expected.ToString();
            }
        }

        static List<TestTag> CreateTestTags()
        {
            var tags = new List<TestTag>();

            // Controller-Scoped Array Elements
            for (int i = 0; i < 10; i++)
            {
                tags.Add(new TestTag
                {
                    Name = $"gTestArray_DINT[{i}]",
                    InitialValue = PlcValue.Dint((i + 1) * 10),
                    TestValue = PlcValue.Dint(1000 + (i * 111)),
                    Description = $"Controller DINT array element {i}"
                });

                tags.Add(new TestTag
                {
                    Name = $"gTestArray_REAL[{i}]",
                    InitialValue = PlcValue.Real((i + 1.0f) * 1.1f),
                    TestValue = PlcValue.Real(10.0f + (i * 1.11f)),
                    Description = $"Controller REAL array element {i}"
                });

                tags.Add(new TestTag
                {
                    Name = $"gTestArray_BOOL[{i}]",
                    InitialValue = PlcValue.Bool(i % 2 == 0),
                    TestValue = PlcValue.Bool(i % 2 == 1),
                    Description = $"Controller BOOL array element {i}"
                });

                tags.Add(new TestTag
                {
                    Name = $"gTestArray_INT[{i}]",
                    InitialValue = PlcValue.Int((short)((i + 1) * 100)),
                    TestValue = PlcValue.Int((short)(1000 + (i * 111))),
                    Description = $"Controller INT array element {i}"
                });
            }

            // Large DINT Array
            foreach (var idx in new[] { 100, 200, 300, 500, 999 })
            {
                tags.Add(new TestTag
                {
                    Name = $"gTestArray_Large[{idx}]",
                    InitialValue = PlcValue.Dint(0),
                    TestValue = PlcValue.Dint(10000 + idx),
                    Description = $"Controller large DINT array element {idx} (16-bit index)"
                });
            }

            // Simple STRING tag
            tags.Add(new TestTag
            {
                Name = "gTest_STRING",
                InitialValue = PlcValue.String("Initial String Value"),
                TestValue = PlcValue.String("Test String Write 789"),
                Description = "Controller simple STRING tag (not UDT member)"
            });

            // Controller-Scoped UDT Members
            tags.Add(new TestTag
            {
                Name = "gTestUDT.Member1_DINT",
                InitialValue = PlcValue.Dint(100),
                TestValue = PlcValue.Dint(7777),
                Description = "Controller UDT member: Member1_DINT"
            });

            tags.Add(new TestTag
            {
                Name = "gTestUDT.Member2_REAL",
                InitialValue = PlcValue.Real(3.14159f),
                TestValue = PlcValue.Real(77.77f),
                Description = "Controller UDT member: Member2_REAL"
            });

            tags.Add(new TestTag
            {
                Name = "gTestUDT.Member3_BOOL",
                InitialValue = PlcValue.Bool(true),
                TestValue = PlcValue.Bool(false),
                Description = "Controller UDT member: Member3_BOOL"
            });

            tags.Add(new TestTag
            {
                Name = "gTestUDT.Member4_INT",
                InitialValue = PlcValue.Int(42),
                TestValue = PlcValue.Int(8888),
                Description = "Controller UDT member: Member4_INT"
            });

            tags.Add(new TestTag
            {
                Name = "gTestUDT.Member5_String",
                InitialValue = PlcValue.String("Hello PLC"),
                TestValue = PlcValue.String("Test String 123"),
                Description = "Controller UDT member: Member5_String"
            });

            // UDT Array_DINT - elements 0-9
            for (int i = 0; i < 10; i++)
            {
                tags.Add(new TestTag
                {
                    Name = $"gTestUDT.Array_DINT[{i}]",
                    InitialValue = PlcValue.Dint(i + 1),
                    TestValue = PlcValue.Dint(1000 + (i * 111)),
                    Description = $"Controller UDT array member: Array_DINT[{i}]"
                });
            }

            // UDT Array_REAL - elements 0-4
            for (int i = 0; i < 5; i++)
            {
                tags.Add(new TestTag
                {
                    Name = $"gTestUDT.Array_REAL[{i}]",
                    InitialValue = PlcValue.Real((i + 1.0f) * 1.1f),
                    TestValue = PlcValue.Real(10.0f + (i * 1.11f)),
                    Description = $"Controller UDT array member: Array_REAL[{i}]"
                });
            }

            // UDT Array_BOOL - elements 0-19
            for (int i = 0; i < 20; i++)
            {
                tags.Add(new TestTag
                {
                    Name = $"gTestUDT.Array_BOOL[{i}]",
                    InitialValue = PlcValue.Bool(i % 2 == 0),
                    TestValue = PlcValue.Bool(i % 2 == 1),
                    Description = $"Controller UDT array member: Array_BOOL[{i}]"
                });
            }

            // UDT Array elements 0-9 - Member1_DINT, Member2_REAL, Member3_BOOL, Member4_INT
            for (int i = 0; i < 10; i++)
            {
                tags.Add(new TestTag
                {
                    Name = $"gTestUDT_Array[{i}].Member1_DINT",
                    InitialValue = PlcValue.Dint((i + 1) * 100),
                    TestValue = PlcValue.Dint(5000 + (i * 111)),
                    Description = $"Controller UDT array element {i}, member Member1_DINT"
                });

                tags.Add(new TestTag
                {
                    Name = $"gTestUDT_Array[{i}].Member2_REAL",
                    InitialValue = PlcValue.Real((i + 1.0f) * 1.1f),
                    TestValue = PlcValue.Real(50.0f + (i * 1.11f)),
                    Description = $"Controller UDT array element {i}, member Member2_REAL"
                });

                tags.Add(new TestTag
                {
                    Name = $"gTestUDT_Array[{i}].Member3_BOOL",
                    InitialValue = PlcValue.Bool(i % 2 == 0),
                    TestValue = PlcValue.Bool(i % 2 == 1),
                    Description = $"Controller UDT array element {i}, member Member3_BOOL"
                });

                tags.Add(new TestTag
                {
                    Name = $"gTestUDT_Array[{i}].Member4_INT",
                    InitialValue = PlcValue.Int((short)((i + 1) * 10)),
                    TestValue = PlcValue.Int((short)(500 + (i * 11))),
                    Description = $"Controller UDT array element {i}, member Member4_INT"
                });
            }

            // UDT Array elements 0-9 - Array_DINT[0-9]
            for (int i = 0; i < 10; i++)
            {
                for (int j = 0; j < 10; j++)
                {
                    tags.Add(new TestTag
                    {
                        Name = $"gTestUDT_Array[{i}].Array_DINT[{j}]",
                        InitialValue = PlcValue.Dint(j + 1),
                        TestValue = PlcValue.Dint(1000 + (i * 100) + (j * 11)),
                        Description = $"Controller UDT array element {i}, nested array member Array_DINT[{j}]"
                    });
                }
            }

            // UDT Array elements 0-9 - Array_REAL[0-4] (sample a few)
            for (int i = 0; i < 10; i++)
            {
                for (int j = 0; j < 5; j++)
                {
                    tags.Add(new TestTag
                    {
                        Name = $"gTestUDT_Array[{i}].Array_REAL[{j}]",
                        InitialValue = PlcValue.Real((j + 1.0f) * 1.1f),
                        TestValue = PlcValue.Real(10.0f + (i * 10.0f) + (j * 1.11f)),
                        Description = $"Controller UDT array element {i}, nested array member Array_REAL[{j}]"
                    });
                }
            }

            // Program-Scoped Array Elements
            for (int i = 0; i < 10; i++)
            {
                tags.Add(new TestTag
                {
                    Name = $"Program:TestProgram.gTestArray_DINT[{i}]",
                    InitialValue = PlcValue.Dint((i + 1) * 1000),
                    TestValue = PlcValue.Dint(10000 + (i * 1111)),
                    Description = $"Program-scoped DINT array element {i}"
                });

                tags.Add(new TestTag
                {
                    Name = $"Program:TestProgram.gTestArray_REAL[{i}]",
                    InitialValue = PlcValue.Real(10.1f + (i * 10.1f)),
                    TestValue = PlcValue.Real(100.0f + (i * 11.11f)),
                    Description = $"Program-scoped REAL array element {i}"
                });

                tags.Add(new TestTag
                {
                    Name = $"Program:TestProgram.gTestArray_BOOL[{i}]",
                    InitialValue = PlcValue.Bool(i % 2 == 1),
                    TestValue = PlcValue.Bool(i % 2 == 0),
                    Description = $"Program-scoped BOOL array element {i}"
                });
            }

            // Simple STRING tag (program-scoped)
            tags.Add(new TestTag
            {
                Name = "Program:TestProgram.gTest_STRING",
                InitialValue = PlcValue.String("Program Initial String"),
                TestValue = PlcValue.String("Program Test String Write 999"),
                Description = "Program-scoped simple STRING tag (not UDT member)"
            });

            // Program-Scoped UDT Members
            tags.Add(new TestTag
            {
                Name = "Program:TestProgram.gTestUDT.Member1_DINT",
                InitialValue = PlcValue.Dint(500),
                TestValue = PlcValue.Dint(55555),
                Description = "Program-scoped UDT member: Member1_DINT"
            });

            tags.Add(new TestTag
            {
                Name = "Program:TestProgram.gTestUDT.Member2_REAL",
                InitialValue = PlcValue.Real(5.5f),
                TestValue = PlcValue.Real(555.55f),
                Description = "Program-scoped UDT member: Member2_REAL"
            });

            tags.Add(new TestTag
            {
                Name = "Program:TestProgram.gTestUDT.Member3_BOOL",
                InitialValue = PlcValue.Bool(false),
                TestValue = PlcValue.Bool(true),
                Description = "Program-scoped UDT member: Member3_BOOL"
            });

            tags.Add(new TestTag
            {
                Name = "Program:TestProgram.gTestUDT.Member4_INT",
                InitialValue = PlcValue.Int(24),
                TestValue = PlcValue.Int(9999),
                Description = "Program-scoped UDT member: Member4_INT"
            });

            tags.Add(new TestTag
            {
                Name = "Program:TestProgram.gTestUDT.Member5_String",
                InitialValue = PlcValue.String("Program UDT"),
                TestValue = PlcValue.String("Program Test String 456"),
                Description = "Program-scoped UDT member: Member5_String"
            });

            // Program UDT Array_DINT - elements 0-9
            for (int i = 0; i < 10; i++)
            {
                tags.Add(new TestTag
                {
                    Name = $"Program:TestProgram.gTestUDT.Array_DINT[{i}]",
                    InitialValue = PlcValue.Dint(i + 1),
                    TestValue = PlcValue.Dint(2000 + (i * 111)),
                    Description = $"Program-scoped UDT array member: Array_DINT[{i}]"
                });
            }

            // Program UDT Array_REAL - elements 0-4
            for (int i = 0; i < 5; i++)
            {
                tags.Add(new TestTag
                {
                    Name = $"Program:TestProgram.gTestUDT.Array_REAL[{i}]",
                    InitialValue = PlcValue.Real((i + 1.0f) * 1.1f),
                    TestValue = PlcValue.Real(20.0f + (i * 1.11f)),
                    Description = $"Program-scoped UDT array member: Array_REAL[{i}]"
                });
            }

            // Program UDT Array elements 0-4 - Member1_DINT, Member2_REAL, Member3_BOOL
            for (int i = 0; i < 5; i++)
            {
                tags.Add(new TestTag
                {
                    Name = $"Program:TestProgram.gTestUDT_Array[{i}].Member1_DINT",
                    InitialValue = PlcValue.Dint((i + 1) * 200),
                    TestValue = PlcValue.Dint(6000 + (i * 111)),
                    Description = $"Program-scoped UDT array element {i}, member Member1_DINT"
                });

                tags.Add(new TestTag
                {
                    Name = $"Program:TestProgram.gTestUDT_Array[{i}].Member2_REAL",
                    InitialValue = PlcValue.Real((i + 1.0f) * 2.2f),
                    TestValue = PlcValue.Real(60.0f + (i * 1.11f)),
                    Description = $"Program-scoped UDT array element {i}, member Member2_REAL"
                });

                tags.Add(new TestTag
                {
                    Name = $"Program:TestProgram.gTestUDT_Array[{i}].Member3_BOOL",
                    InitialValue = PlcValue.Bool(i % 2 == 1),
                    TestValue = PlcValue.Bool(i % 2 == 0),
                    Description = $"Program-scoped UDT array element {i}, member Member3_BOOL"
                });
            }

            // Program UDT Array elements 0-4 - Array_DINT[0-9]
            for (int i = 0; i < 5; i++)
            {
                for (int j = 0; j < 10; j++)
                {
                    tags.Add(new TestTag
                    {
                        Name = $"Program:TestProgram.gTestUDT_Array[{i}].Array_DINT[{j}]",
                        InitialValue = PlcValue.Dint(j + 1),
                        TestValue = PlcValue.Dint(2000 + (i * 100) + (j * 11)),
                        Description = $"Program-scoped UDT array element {i}, nested array member Array_DINT[{j}]"
                    });
                }
            }

            return tags;
        }
    }
}

