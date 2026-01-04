package main

/*
Comprehensive Test for All Tags from PLC_TEST_TAG_DEFINITIONS.md

This test verifies that the Go wrapper can correctly:
1. Read all tags (controller and program-scoped)
2. Write new values to all tags
3. Read back and verify the writes were successful

Run with: go run main.go

Prerequisites:
- All tags from PLC_TEST_TAG_DEFINITIONS.md must exist in the PLC
- PLC must be accessible at 192.168.0.1:44818
- ControlLogix CPU in Slot 0 (or adjust CPU_SLOT constant)
*/

import (
	"fmt"
	"math"
	"strings"

	"github.com/rust-ethernet-ip/gowrapper/ethernetip"
)

const (
	PLC_ADDRESS = "192.168.0.1:44818"
	CPU_SLOT    = byte(0) // ControlLogix CPU in Slot 0
)

type TestTag struct {
	Name         string
	InitialValue interface{}
	TestValue    interface{}
	Description  string
}

func main() {
	fmt.Println("═══════════════════════════════════════════════════════════════════════════════")
	fmt.Println("🔬 Comprehensive Test: All Tags from PLC_TEST_TAG_DEFINITIONS.md (Go Wrapper)")
	fmt.Println("═══════════════════════════════════════════════════════════════════════════════")
	fmt.Println()

	fmt.Printf("🔌 Connecting to ControlLogix PLC at %s...\n", PLC_ADDRESS)
	fmt.Printf("   CPU Slot: %d\n", CPU_SLOT)

	client, err := ethernetip.NewClient(PLC_ADDRESS)
	if err != nil {
		fmt.Printf("❌ Failed to connect to PLC: %v\n", err)
		return
	}
	defer client.Close()

	// Set route path for ControlLogix (if supported)
	route := ethernetip.NewRoutePath().AddSlot(CPU_SLOT)
	err = client.SetRoutePath(route)
	if err != nil {
		fmt.Printf("⚠️  Warning: Route path support temporarily disabled: %v\n", err)
		fmt.Printf("   Continuing without route path (may work for CompactLogix)\n")
	} else {
		fmt.Printf("📍 Route path set: CPU Slot %d\n", CPU_SLOT)
	}

	fmt.Println("✅ Connected successfully!\n")

	// Define all test tags
	testTags := createTestTags()

	var totalTests, passedTests, failedTests, skippedTests int

	// Track failures with error messages
	var readFailures []struct {
		tagName string
		error   string
	}
	var writeFailures []struct {
		tagName string
		error   string
	}
	var verifyFailures []struct {
		tagName  string
		error    string
		actual   interface{}
		expected interface{}
	}

	// Step 1: Read initial values
	fmt.Println("═══════════════════════════════════════════════════════════════════════════════")
	fmt.Println("📖 STEP 1: Reading Initial Values")
	fmt.Println("═══════════════════════════════════════════════════════════════════════════════")
	fmt.Println()

	initialValues := make(map[string]interface{})

	for _, tag := range testTags {
		totalTests++
		fmt.Printf("   Reading %s... ", tag.Name)
		value, err := readTag(client, tag.Name, tag.InitialValue)
		if err != nil {
			errorMsg := err.Error()
			fmt.Printf("❌ FAILED: %s\n", errorMsg)
			fmt.Println("      ⚠️  Tag may not exist in PLC - skipping write/verify for this tag")
			readFailures = append(readFailures, struct {
				tagName string
				error   string
			}{tag.Name, errorMsg})
			failedTests++
			skippedTests++
		} else {
			fmt.Printf("✅ %v\n", value)
			initialValues[tag.Name] = value
		}
	}

	fmt.Println()
	fmt.Printf("📊 Step 1 Summary: %d read, %d failed\n", len(initialValues), len(testTags)-len(initialValues))
	fmt.Println()

	// Step 2: Write test values
	fmt.Println("═══════════════════════════════════════════════════════════════════════════════")
	fmt.Println("✏️  STEP 2: Writing Test Values")
	fmt.Println("═══════════════════════════════════════════════════════════════════════════════")
	fmt.Println()

	var writtenTags []string

	for _, tag := range testTags {
		if _, exists := initialValues[tag.Name]; !exists {
			continue // Skip tags that failed to read
		}

		fmt.Printf("   Writing %s = %v... ", tag.Name, tag.TestValue)
		err := writeTag(client, tag.Name, tag.TestValue)
		if err != nil {
			errorMsg := err.Error()
			fmt.Printf("❌ FAILED: %s\n", errorMsg)
			writeFailures = append(writeFailures, struct {
				tagName string
				error   string
			}{tag.Name, errorMsg})
			failedTests++
		} else {
			fmt.Println("✅")
			writtenTags = append(writtenTags, tag.Name)

			// For STRING types, immediately read back to verify write
			if _, ok := tag.TestValue.(string); ok {
				fmt.Printf("      Reading back %s... ", tag.Name)
				readValue, err := readTag(client, tag.Name, tag.TestValue)
				if err != nil {
					fmt.Printf("❌ FAILED TO READ BACK: %s\n", err.Error())
				} else if valuesMatch(readValue, tag.TestValue) {
					fmt.Printf("✅ VERIFIED: %v\n", readValue)
				} else {
					fmt.Println("⚠️  MISMATCH after write!")
					fmt.Printf("         Expected: %v\n", tag.TestValue)
					fmt.Printf("         Got:      %v\n", readValue)
				}
			}
		}
	}

	fmt.Println()
	fmt.Printf("📊 Step 2 Summary: %d written, %d failed\n", len(writtenTags), len(writeFailures))
	fmt.Println()

	// Step 3: Read back and verify writes
	fmt.Println("═══════════════════════════════════════════════════════════════════════════════")
	fmt.Println("🔍 STEP 3: Reading Back and Verifying Writes")
	fmt.Println("═══════════════════════════════════════════════════════════════════════════════")
	fmt.Println()

	for _, tag := range testTags {
		isWritten := false
		for _, written := range writtenTags {
			if written == tag.Name {
				isWritten = true
				break
			}
		}
		if !isWritten {
			continue // Skip tags that failed to write
		}

		fmt.Printf("   Verifying %s... ", tag.Name)
		value, err := readTag(client, tag.Name, tag.TestValue)
		if err != nil {
			errorMsg := err.Error()
			fmt.Printf("❌ FAILED: %s\n", errorMsg)
			verifyFailures = append(verifyFailures, struct {
				tagName  string
				error    string
				actual   interface{}
				expected interface{}
			}{tag.Name, errorMsg, nil, tag.TestValue})
			failedTests++
		} else if valuesMatch(value, tag.TestValue) {
			fmt.Printf("✅ %v\n", value)
			passedTests++
		} else {
			fmt.Println("❌ MISMATCH!")
			fmt.Printf("      Expected: %v\n", tag.TestValue)
			fmt.Printf("      Got:      %v\n", value)
			verifyFailures = append(verifyFailures, struct {
				tagName  string
				error    string
				actual   interface{}
				expected interface{}
			}{tag.Name, "Value mismatch", value, tag.TestValue})
			failedTests++
		}
	}

	fmt.Println()

	// Final Summary
	fmt.Println("═══════════════════════════════════════════════════════════════════════════════")
	fmt.Println("📊 FINAL RESULTS")
	fmt.Println("═══════════════════════════════════════════════════════════════════════════════")
	fmt.Printf("   Total Tests:     %d\n", totalTests)
	fmt.Printf("   ✅ Passed:         %d\n", passedTests)
	fmt.Printf("   ❌ Failed:         %d\n", failedTests)
	fmt.Printf("   ⏭️  Skipped:        %d\n", skippedTests)
	if totalTests > 0 {
		fmt.Printf("   Success Rate:     %.1f%%\n", float64(passedTests)*100.0/float64(totalTests))
	}
	fmt.Println()

	// Display failure summary
	if len(readFailures) > 0 || len(writeFailures) > 0 || len(verifyFailures) > 0 {
		fmt.Println("═══════════════════════════════════════════════════════════════════════════════")
		fmt.Println("❌ FAILED TAGS SUMMARY")
		fmt.Println("═══════════════════════════════════════════════════════════════════════════════")
		fmt.Println()

		if len(readFailures) > 0 {
			fmt.Printf("📖 READ FAILURES (%d tags):\n", len(readFailures))
			for _, failure := range readFailures {
				fmt.Printf("   • %s: %s\n", failure.tagName, failure.error)
			}
			fmt.Println()
		}

		if len(writeFailures) > 0 {
			fmt.Printf("✏️  WRITE FAILURES (%d tags):\n", len(writeFailures))
			// Group by error pattern
			errorGroups := make(map[string][]string)
			for _, failure := range writeFailures {
				errorKey := failure.error
				if strings.Contains(failure.error, "0x2107") || strings.Contains(failure.error, "2107") {
					if strings.Contains(failure.tagName, "_Array[") && strings.Contains(failure.tagName, ".") {
						errorKey = "PLC does not support writing to UDT array element members directly (Error 0x2107)"
					} else if strings.Contains(failure.tagName, "Member5_String") || strings.HasSuffix(failure.tagName, ".Member5_String") {
						errorKey = "PLC does not support writing to STRING members in UDTs directly (Error 0x2107)"
					} else if failure.tagName == "gTest_STRING" || failure.tagName == "Program:TestProgram.gTest_STRING" {
						errorKey = "PLC does not support writing to STRING tags directly (Error 0x2107)"
					}
				}
				errorGroups[errorKey] = append(errorGroups[errorKey], failure.tagName)
			}

			for error, tags := range errorGroups {
				fmt.Printf("   Error: %s\n", error)
				fmt.Printf("   Affected tags (%d):\n", len(tags))
				if len(tags) <= 5 {
					for _, tag := range tags {
						fmt.Printf("     • %s\n", tag)
					}
				} else {
					for i := 0; i < 3; i++ {
						fmt.Printf("     • %s\n", tags[i])
					}
					fmt.Printf("     • ... and %d more\n", len(tags)-3)
				}
			}
			fmt.Println()
		}

		if len(verifyFailures) > 0 {
			fmt.Printf("🔍 VERIFY FAILURES (%d tags):\n", len(verifyFailures))
			for _, failure := range verifyFailures {
				fmt.Printf("   • %s: %s\n", failure.tagName, failure.error)
				fmt.Printf("     Expected: %v, Got: %v\n", failure.expected, failure.actual)
			}
			fmt.Println()
		}

		fmt.Println("═══════════════════════════════════════════════════════════════════════════════")
		fmt.Println()
	}

	if failedTests == 0 && skippedTests == 0 {
		fmt.Println("🎉 ALL TESTS PASSED! The Go wrapper is working correctly.")
	} else if failedTests > 0 {
		fmt.Println("⚠️  Some tests failed. See the FAILED TAGS SUMMARY above for details.")
	} else {
		fmt.Println("ℹ️  Some tags were skipped (may not exist in PLC).")
	}
}

func readTag(client *ethernetip.EipClient, tagName string, expectedType interface{}) (interface{}, error) {
	if strings.Contains(tagName, "DINT") || strings.Contains(tagName, "_DINT") {
		return client.ReadDint(tagName)
	} else if strings.Contains(tagName, "REAL") || strings.Contains(tagName, "_REAL") {
		return client.ReadReal(tagName)
	} else if strings.Contains(tagName, "BOOL") || strings.Contains(tagName, "_BOOL") {
		return client.ReadBool(tagName)
	} else if strings.Contains(tagName, "INT[") && !strings.Contains(tagName, "DINT") {
		return client.ReadInt(tagName)
	} else if strings.Contains(tagName, "STRING") || strings.Contains(tagName, "String") {
		return client.ReadString(tagName)
	} else {
		// Default to DINT
		return client.ReadDint(tagName)
	}
}

func writeTag(client *ethernetip.EipClient, tagName string, value interface{}) error {
	switch v := value.(type) {
	case bool:
		return client.WriteBool(tagName, v)
	case int:
		return client.WriteDint(tagName, int32(v))
	case int32:
		return client.WriteDint(tagName, v)
	case float32:
		return client.WriteReal(tagName, float64(v))
	case float64:
		return client.WriteReal(tagName, v)
	case int16:
		return client.WriteInt(tagName, v)
	case string:
		return client.WriteString(tagName, v)
	default:
		return fmt.Errorf("unsupported type: %T", value)
	}
}

func valuesMatch(actual, expected interface{}) bool {
	switch a := actual.(type) {
	case bool:
		if b, ok := expected.(bool); ok {
			return a == b
		}
	case int:
		if b, ok := expected.(int); ok {
			return a == b
		} else if b, ok := expected.(int32); ok {
			return a == int(b)
		}
	case int32:
		if b, ok := expected.(int32); ok {
			return a == b
		} else if b, ok := expected.(int); ok {
			return int(a) == b
		}
	case float32:
		if b, ok := expected.(float32); ok {
			return math.Abs(float64(a-b)) < 0.001
		} else if b, ok := expected.(float64); ok {
			return math.Abs(float64(a)-b) < 0.001
		}
	case float64:
		if b, ok := expected.(float64); ok {
			return math.Abs(a-b) < 0.001
		} else if b, ok := expected.(float32); ok {
			return math.Abs(a-float64(b)) < 0.001
		}
	case int16:
		if b, ok := expected.(int16); ok {
			return a == b
		}
	case string:
		if b, ok := expected.(string); ok {
			return a == b
		}
	}
	return false
}

func createTestTags() []TestTag {
	var tags []TestTag

	// Controller-Scoped Array Elements
	for i := 0; i < 10; i++ {
		tags = append(tags, TestTag{
			Name:         fmt.Sprintf("gTestArray_DINT[%d]", i),
			InitialValue: (i + 1) * 10,
			TestValue:    1000 + (i * 111),
			Description:  fmt.Sprintf("Controller DINT array element %d", i),
		})

		tags = append(tags, TestTag{
			Name:         fmt.Sprintf("gTestArray_REAL[%d]", i),
			InitialValue: float32((i + 1)) * 1.1,
			TestValue:    float32(10.0 + float64(i)*1.11),
			Description:  fmt.Sprintf("Controller REAL array element %d", i),
		})

		tags = append(tags, TestTag{
			Name:         fmt.Sprintf("gTestArray_BOOL[%d]", i),
			InitialValue: i%2 == 0,
			TestValue:    i%2 == 1,
			Description:  fmt.Sprintf("Controller BOOL array element %d", i),
		})

		tags = append(tags, TestTag{
			Name:         fmt.Sprintf("gTestArray_INT[%d]", i),
			InitialValue: int16((i + 1) * 100),
			TestValue:    int16(1000 + (i * 111)),
			Description:  fmt.Sprintf("Controller INT array element %d", i),
		})
	}

	// Large DINT Array
	for _, idx := range []int{100, 200, 300, 500, 999} {
		tags = append(tags, TestTag{
			Name:         fmt.Sprintf("gTestArray_Large[%d]", idx),
			InitialValue: 0,
			TestValue:    10000 + idx,
			Description:  fmt.Sprintf("Controller large DINT array element %d (16-bit index)", idx),
		})
	}

	// Simple STRING tag
	tags = append(tags, TestTag{
		Name:         "gTest_STRING",
		InitialValue: "Initial String Value",
		TestValue:    "Test String Write 789",
		Description:  "Controller simple STRING tag (not UDT member)",
	})

	// Controller-Scoped UDT Members
	tags = append(tags, TestTag{
		Name:         "gTestUDT.Member1_DINT",
		InitialValue: 100,
		TestValue:    7777,
		Description:  "Controller UDT member: Member1_DINT",
	})

	tags = append(tags, TestTag{
		Name:         "gTestUDT.Member2_REAL",
		InitialValue: float32(3.14159),
		TestValue:    float32(77.77),
		Description:  "Controller UDT member: Member2_REAL",
	})

	tags = append(tags, TestTag{
		Name:         "gTestUDT.Member3_BOOL",
		InitialValue: true,
		TestValue:    false,
		Description:  "Controller UDT member: Member3_BOOL",
	})

	tags = append(tags, TestTag{
		Name:         "gTestUDT.Member4_INT",
		InitialValue: int16(42),
		TestValue:    int16(8888),
		Description:  "Controller UDT member: Member4_INT",
	})

	tags = append(tags, TestTag{
		Name:         "gTestUDT.Member5_String",
		InitialValue: "Hello PLC",
		TestValue:    "Test String 123",
		Description:  "Controller UDT member: Member5_String",
	})

	// UDT Array_DINT - elements 0-9
	for i := 0; i < 10; i++ {
		tags = append(tags, TestTag{
			Name:         fmt.Sprintf("gTestUDT.Array_DINT[%d]", i),
			InitialValue: i + 1,
			TestValue:    1000 + (i * 111),
			Description:  fmt.Sprintf("Controller UDT array member: Array_DINT[%d]", i),
		})
	}

	// UDT Array elements 0-9 - Member1_DINT, Member2_REAL, Member3_BOOL, Member4_INT
	for i := 0; i < 10; i++ {
		tags = append(tags, TestTag{
			Name:         fmt.Sprintf("gTestUDT_Array[%d].Member1_DINT", i),
			InitialValue: (i + 1) * 100,
			TestValue:    5000 + (i * 111),
			Description:  fmt.Sprintf("Controller UDT array element %d, member Member1_DINT", i),
		})

		tags = append(tags, TestTag{
			Name:         fmt.Sprintf("gTestUDT_Array[%d].Member2_REAL", i),
			InitialValue: float32((i + 1)) * 1.1,
			TestValue:    float32(50.0 + float64(i)*1.11),
			Description:  fmt.Sprintf("Controller UDT array element %d, member Member2_REAL", i),
		})

		tags = append(tags, TestTag{
			Name:         fmt.Sprintf("gTestUDT_Array[%d].Member3_BOOL", i),
			InitialValue: i%2 == 0,
			TestValue:    i%2 == 1,
			Description:  fmt.Sprintf("Controller UDT array element %d, member Member3_BOOL", i),
		})

		tags = append(tags, TestTag{
			Name:         fmt.Sprintf("gTestUDT_Array[%d].Member4_INT", i),
			InitialValue: int16((i + 1) * 10),
			TestValue:    int16(500 + (i * 11)),
			Description:  fmt.Sprintf("Controller UDT array element %d, member Member4_INT", i),
		})
	}

	// UDT Array elements 0-9 - Array_REAL[0-4] (sample a few)
	for i := 0; i < 10; i++ {
		for j := 0; j < 5; j++ {
			tags = append(tags, TestTag{
				Name:         fmt.Sprintf("gTestUDT_Array[%d].Array_REAL[%d]", i, j),
				InitialValue: float32((j + 1)) * 1.1,
				TestValue:    float32(10.0 + float64(i)*10.0 + float64(j)*1.11),
				Description:  fmt.Sprintf("Controller UDT array element %d, nested array member Array_REAL[%d]", i, j),
			})
		}
	}

	// Program-Scoped Array Elements
	for i := 0; i < 10; i++ {
		tags = append(tags, TestTag{
			Name:         fmt.Sprintf("Program:TestProgram.gTestArray_DINT[%d]", i),
			InitialValue: (i + 1) * 1000,
			TestValue:    10000 + (i * 1111),
			Description:  fmt.Sprintf("Program-scoped DINT array element %d", i),
		})

		tags = append(tags, TestTag{
			Name:         fmt.Sprintf("Program:TestProgram.gTestArray_REAL[%d]", i),
			InitialValue: float32(10.1 + float64(i)*10.1),
			TestValue:    float32(100.0 + float64(i)*11.11),
			Description:  fmt.Sprintf("Program-scoped REAL array element %d", i),
		})

		tags = append(tags, TestTag{
			Name:         fmt.Sprintf("Program:TestProgram.gTestArray_BOOL[%d]", i),
			InitialValue: i%2 == 1,
			TestValue:    i%2 == 0,
			Description:  fmt.Sprintf("Program-scoped BOOL array element %d", i),
		})
	}

	// Simple STRING tag (program-scoped)
	tags = append(tags, TestTag{
		Name:         "Program:TestProgram.gTest_STRING",
		InitialValue: "Program Initial String",
		TestValue:    "Program Test String Write 999",
		Description:  "Program-scoped simple STRING tag (not UDT member)",
	})

	// Program-Scoped UDT Members
	tags = append(tags, TestTag{
		Name:         "Program:TestProgram.gTestUDT.Member1_DINT",
		InitialValue: 500,
		TestValue:    55555,
		Description:  "Program-scoped UDT member: Member1_DINT",
	})

	tags = append(tags, TestTag{
		Name:         "Program:TestProgram.gTestUDT.Member2_REAL",
		InitialValue: float32(5.5),
		TestValue:    float32(555.55),
		Description:  "Program-scoped UDT member: Member2_REAL",
	})

	tags = append(tags, TestTag{
		Name:         "Program:TestProgram.gTestUDT.Member3_BOOL",
		InitialValue: false,
		TestValue:    true,
		Description:  "Program-scoped UDT member: Member3_BOOL",
	})

	tags = append(tags, TestTag{
		Name:         "Program:TestProgram.gTestUDT.Member4_INT",
		InitialValue: int16(24),
		TestValue:    int16(9999),
		Description:  "Program-scoped UDT member: Member4_INT",
	})

	tags = append(tags, TestTag{
		Name:         "Program:TestProgram.gTestUDT.Member5_String",
		InitialValue: "Program UDT",
		TestValue:    "Program Test String 456",
		Description:  "Program-scoped UDT member: Member5_String",
	})

	// Program UDT Array_DINT - elements 0-9
	for i := 0; i < 10; i++ {
		tags = append(tags, TestTag{
			Name:         fmt.Sprintf("Program:TestProgram.gTestUDT.Array_DINT[%d]", i),
			InitialValue: i + 1,
			TestValue:    2000 + (i * 111),
			Description:  fmt.Sprintf("Program-scoped UDT array member: Array_DINT[%d]", i),
		})
	}

	// Program UDT Array elements 0-4 - Member1_DINT, Member2_REAL, Member3_BOOL
	for i := 0; i < 5; i++ {
		tags = append(tags, TestTag{
			Name:         fmt.Sprintf("Program:TestProgram.gTestUDT_Array[%d].Member1_DINT", i),
			InitialValue: (i + 1) * 200,
			TestValue:    6000 + (i * 111),
			Description:  fmt.Sprintf("Program-scoped UDT array element %d, member Member1_DINT", i),
		})

		tags = append(tags, TestTag{
			Name:         fmt.Sprintf("Program:TestProgram.gTestUDT_Array[%d].Member2_REAL", i),
			InitialValue: float32((i + 1)) * 2.2,
			TestValue:    float32(60.0 + float64(i)*1.11),
			Description:  fmt.Sprintf("Program-scoped UDT array element %d, member Member2_REAL", i),
		})

		tags = append(tags, TestTag{
			Name:         fmt.Sprintf("Program:TestProgram.gTestUDT_Array[%d].Member3_BOOL", i),
			InitialValue: i%2 == 1,
			TestValue:    i%2 == 0,
			Description:  fmt.Sprintf("Program-scoped UDT array element %d, member Member3_BOOL", i),
		})
	}

	return tags
}
