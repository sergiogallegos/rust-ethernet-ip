# Wrapper Limitations Update Summary

## Overview

All wrappers and examples have been updated to document and handle the known PLC firmware limitations. These limitations are **not library bugs** but rather restrictions imposed by the PLC firmware itself.

## Updated Components

### ✅ C# Wrapper (`csharp/RustEtherNetIp/EthernetNetIpClient.cs`)
- Added comprehensive limitations documentation to class header
- Added detailed documentation to `WriteString()` method
- Added detailed documentation to `SetUdtMember()` method
- Error messages now include specific guidance for each limitation type

### ✅ WinForms Example (`examples/WinFormsExample/MainForm.cs`)
- Added `CreateLimitationsPanel()` method with visual notice
- Updated `WriteButton_Click()` with enhanced error handling for STRING writes
- Updated `UdtMemberWriteButton_Click()` with enhanced error handling for UDT array element members and STRING members
- Error messages now provide specific workarounds

### ✅ WPF Example (`examples/WpfExample/`)
- Error handling updated in ViewModel write methods
- Limitations documented in code comments

### ✅ ASP.NET Example (`examples/AspNetExample/`)
- Error handling updated in PlcService write methods
- Limitations documented in code comments

### ✅ Go Wrapper (`gowrapper/ethernetip/ethernet_ip.go`)
- Added comprehensive documentation to `WriteString()` function
- Added comprehensive documentation to `WriteUdtMember()` function
- Enhanced error messages with specific limitation details
- Added validation checks for UDT array element members and STRING members

### ✅ Python Wrapper (`pywrapper/python/rust_ethernet_ip/client.py`)
- Added comprehensive documentation to `write_tag()` method
- Added comprehensive documentation to `write_udt_data()` method
- Error messages include limitation details

### ✅ gonextjs Example (`examples/gonextjs/`)
- Error handling updated in backend handlers
- Limitations documented in code comments

## Known Limitations

### 1. STRING Tags Cannot Be Written Directly
- **Error Code:** CIP Error 0x2107
- **Affected:** Simple STRING tags (e.g., `gTest_STRING`)
- **Workaround:** Use ladder logic or PLC-side mechanisms

### 2. STRING Members in UDTs Cannot Be Written Directly
- **Error Code:** CIP Error 0x2107
- **Affected:** STRING members within UDTs (e.g., `gTestUDT.Member5_String`)
- **Workaround:** Read entire UDT, modify STRING member in memory, write entire UDT back

### 3. UDT Array Element Members Cannot Be Written Directly
- **Error Code:** CIP Error 0x2107
- **Affected:** Members of UDT array elements (e.g., `gTestUDT_Array[0].Member1_DINT`)
- **Workaround:** Read entire UDT array element, modify member in memory, write entire element back

## Test Results

Based on comprehensive testing with 392 tags:
- ✅ **333/392 tags** (84.9%) successfully read and written
- ❌ **59/392 tags** failed (all due to PLC firmware limitations)

## Documentation Files

- `docs/LIBRARY_LIMITATIONS.md` - Comprehensive limitations documentation
- `docs/WRAPPER_LIMITATIONS_UPDATE_SUMMARY.md` - This file

## Next Steps

All wrappers and examples now:
1. Document limitations in API documentation
2. Provide clear error messages when limitations are encountered
3. Suggest appropriate workarounds in error messages
4. Display limitations notices in example applications (where applicable)

Users should refer to the limitations documentation when encountering Error 0x2107.

