# CIP Protocol Reference - Based on 1756-PM020

This document contains key information extracted from Allen-Bradley Publication 1756-PM020: Logix Controller Access Data, specifically focusing on array element addressing, tag services, and CIP protocol structure.

## Table of Contents

1. [CIP Service Request/Response Format](#cip-service-requestresponse-format)
2. [Segment Encoding](#segment-encoding)
3. [Read Tag Service](#read-tag-service)
4. [Write Tag Service](#write-tag-service)
5. [Read Tag Fragmented Service](#read-tag-fragmented-service)
6. [Write Tag Fragmented Service](#write-tag-fragmented-service)
7. [Array Element Addressing](#array-element-addressing)

---

## CIP Service Request/Response Format

### Request Format

All CIP services follow the Message Router Request/Response format defined in the CIP Networks Library, Volume 1, Chapter 2.

**Message Request Fields:**

| Field | Description |
|-------|-------------|
| **Request Service** | Indicates to the object referenced in the request path to perform a task. The CIP or device manufacturer define these tasks. Most services covered in this manual are defined by Rockwell Automation vendor-specific objects. |
| **Request Path Size** | A byte value that indicates the number of 16-bit words in the Request Path. |
| **Request Path** | A variable-sized field that consists of one or more segments. The path references the item that services operate on in the controller. The path contains Logical or Symbolic segments or both. |
| **Request Data** | The service-specific data that is delivered to the object referenced in the Request Path. This field only appears in the message frame if a service has service-specific data. |

**Notes:**
- This same form is used for ControlNet and EtherNet/IP communication CIP-based networks
- Use the CIP service format for CIP-explicit messages and to deliver connected or unconnected messages to the controller
- The mechanisms for doing this are CIP-network specific
- For EtherNet/IP access, see CIP Networks Library, Volume 1 unconnected, Chapter 3 and EtherNet/IP Adaptation of CIP, Volume 2

### Response Format

**Message Response Fields:**

| Field | Description |
|-------|-------------|
| **Reply Service** | The request service with the MSB set to 1 (e.g., 0x4C → 0xCC, 0x4D → 0xCD, 0x52 → 0xD2, 0x53 → 0xD3) |
| **Reserved** | 0x00 (reserved byte) |
| **General Status** | An 8-bit value that indicates whether the service request was successful or resulted in an error. The CIP Networks Library, Volume 1, Appendix B has a list of the general status codes. The object class specified in the request path defines any extended status codes for each service defined for that class. |
| **Extended Status Size** | An 8-bit value that specifies the number of 16-bit values that follow in the "Extended Status" field. For a successful status (status=0), this value is 0. |
| **Extended Status** | This field contains an array of 16-bit values that provide more detailed information about the general status code. It is only present in the message response when the "Extended Status Size" field has a value greater than 0. |
| **Reply Data** | This field contains the actual data returned by the service request. It is only included in the message frame if the specific service has data to return. |

---

## Segment Encoding

### Logical Segments

These tables explain the Logical Segments. Not all segment types defined by CIP are supported by Logix 5000 controllers.

#### Element ID Segments

**Table: Byte Order Representation of Element ID Value (low byte first)**

| Segment Type | Value | Byte Order Representation |
|--------------|-------|---------------------------|
| **8-bit Element ID** | `0x28` | Byte 0: `Value` (8-bit element ID value) |
| **16-bit Element ID** | `0x29` | Byte 0: `00`, Byte 1: `Low`, Byte 2: `High` |
| **32-bit Element ID** | `0x2A` | Byte 0: `00`, Byte 1: `Lowest`, Byte 2: `Low`, Byte 3: `High`, Byte 4: `Highest` |

**Key Points:**
- `0x28` is used for 8-bit element IDs (single byte value)
- `0x29` is used for 16-bit element IDs (2 bytes, little-endian)
- `0x2A` is used for 32-bit element IDs (4 bytes, little-endian)
- All values use **low byte first** (little-endian) byte order

#### Other Logical Segments

**Class ID Segments:**
- `0x20`: 8-bit Class ID
- `0x21`: 16-bit Class ID (Byte 0: `00`, Byte 1: `Low`, Byte 2: `High`)

**Instance ID Segments:**
- `0x24`: 8-bit Instance ID
- `0x25`: 16-bit Instance ID (Byte 0: `00`, Byte 1: `Low`, Byte 2: `High`)

**Attribute ID Segments:**
- `0x30`: 8-bit Attribute ID
- `0x31`: 16-bit Attribute ID (Byte 0: `00`, Byte 1: `Low`, Byte 2: `High`)

### Symbolic Segments

CIP uses the ANSI Extended Symbol Segment, defined in the CIP Networks Library, Volume 1, Appendix C, to reference items by their symbolic names.

**ANSI Extended Symbolic Segment:**

| Segment Type | Value | Byte Order Representation |
|-------------|-------|---------------------------|
| **ANSI Extended Symbolic** | `0x91` | Byte 0: `Length` (length of symbolic name), Byte 1: `1st char`, ..., Byte N: `Nth Char`, Byte N+1: `(1)` (padding/word count) |

**Key Points:**
- `0x91` is the segment type for ANSI Extended Symbolic
- First byte after `0x91` is the length of the symbolic name
- Followed by ASCII characters of the tag name
- May include padding byte to ensure word alignment

**Important:** When addressing an arrayed tag, the Logical Segment for Element ID is also used in conjunction with the Symbolic Segment.

---

## Read Tag Service

**Service Code:** `0x4C` (Request), `0xCC` (Reply)

### Example 1: Symbolic Segment Addressing Method

Reading a single INT tag named "parts" with value 42.

**Message Request Field:**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | `4C` | Read Tag Service (Request) |
| Request Path Size | `04` | Request Path is 4 words (8 bytes) long |
| Request Path | `91 05 70 61 72 74 73 00` | ANSI Ext. Symbolic Segment for *parts*<br>`91`: Symbolic segment type<br>`05`: Length (5 words)<br>`70 61 72 74 73`: ASCII "parts"<br>`00`: Padding |
| Request Data | `01 00` | Number of elements to read (1) |

**Message Reply Field:**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Reply Service | `CC` | Read Tag Service (Reply) |
| Reserved | `00` | (Empty) |
| General Status | `00` | Success |
| Extended Status Size | `00` | No extended status |
| Reply Data | `C3 00` | INT Tag Type Value (0x00C3 = INT) |
| Reply Data | `2A 00` | 0x002A = 42 decimal (little-endian) |

### Example 2: Symbol Instance Addressing Method

Reading a single INT tag named "parts" with value 42 using instance addressing.

**Message Request Field:**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | `4C` | Read Tag Service (Request) |
| Request Path Size | `03` | Request Path is 3 words (6 bytes) long |
| Request Path | `20 6B 25 00 82 25` | Logical Segment for Symbol Class ID (`20 6B`)<br>Logical Segment for Instance ID for tag parts (`25 00 82 25`) |
| Request Data | `01 00` | Number of elements to read (1) |

**Message Reply Field:** Same as Example 1

### Example: Reading DINT Tag "rate" (value 534)

**Message Request Field:**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | `4C` | Read Tag Service (Request) |
| Request Path Size | `06` | Request Path is 6 words (12 bytes) long |
| Request Path | `91 0A 54 6F 74 61 6C 43 6F 75 6E 74` | ANSI Ext. Symbolic Segment for TotalCount<br>`91`: Symbolic segment<br>`0A`: Length (10 bytes)<br>`54 6F 74 61 6C 43 6F 75 6E 74`: ASCII "TotalCount" |
| Request Data | `01 00` | Number of elements to read (1) |

**Message Reply Field:**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Reply Service | `CC` | Read Tag Service (Reply) |
| Reserved | `00` | (Empty) |
| General Status | `00` | Success |
| Extended Status Size | `00` | No extended status |
| Reply Data | `C4 00` | DINT Tag Type Value (0x00C4 = DINT) |
| Reply Data | `16 02 00 00` | 0x00000216 = 534 decimal (little-endian) |

---

## Write Tag Service

**Service Code:** `0x4D` (Request), `0xCD` (Reply)

### Example: Writing DINT Tag "CartonSize" (value 14)

**Message Request Field:**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | `4D` | Write Tag Service (Request) |
| Request Path Size | `06` | Request Path is 6 words (12 bytes) long |
| Request Path | `91 0A 43 61 72 74 6F 6E 53 69 7A 65` | ANSI Ext. Symbolic Segment for CartonSize<br>`91`: Symbolic segment type<br>`0A`: Length (10 bytes)<br>`43 61 72 74 6F 6E 53 69 7A 65`: ASCII "CartonSize" |
| Request Data | `C4 00` | DINT Tag Type Value (0x00C4 = DINT) |
| Request Data | `01 00` | Number of elements to write (1) |
| Request Data | `0E 00 00 00` | Data 0x0000000E = 14 decimal (little-endian) |

**Message Reply Field:**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Reply Service | `CD` | Write Tag Service (Reply) |
| Reserved | `00` | (Empty) |
| General Status | `00` | Success |
| Extended Status Size | `00` | No extended status |

---

## Read Tag Fragmented Service

**Service Code:** `0x52` (Request), `0xD2` (Reply)

### Purpose

Enables client applications to read data from a tag that exceeds the size of a single packet (approximately 500 bytes). The client must send a series of requests to the controller. For each subsequent request, the client must update the `Offset` field value based on the number of bytes transferred in the response to the previous request.

**Key Points:**
- The `Byte Offset` field is always expressed in bytes, regardless of the data type being read
- For SINT (1 byte), elements and offset are in the same units
- For other data types, the units differ

### Example: Reading 1750 SINTs from "TotalCount"

**1st Message Request Field:**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | `52` | Read Tag Fragmented Service (Request) |
| Request Path Size | `06` | Request Path is 6 words (12 bytes) long |
| Request Path | `91 0A 54 6F 74 61 6C 43 6F 75 6E 74` | ANSI Ext. Symbolic Segment for TotalCount |
| Request Data | `D6 06` | Number of elements to read (1750)<br>`0x06D6` = 1750 decimal (little-endian) |
| Request Data | `00 00 00 00` | Start at this byte offset (0) and return as much as will fit |

**1st Message Reply Field:**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Reply Service | `D2` | Read Tag Fragmented Service (Reply) |
| Reserved | `00` | (Empty) |
| General Status | `06` | Reply Data Too Large |
| Extended Status Size | `00` | No extended status |
| Reply Data | `C2 00` | SINT Tag Type Value (0x00C2 = SINT) |
| Reply Data | `nn nn nn...nn` | Data for Elements 0 through 489 |

**2nd Message Request Field:**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | `52` | Read Tag Fragmented Service (Request) |
| Request Path Size | `06` | Request Path is 6 words (12 bytes) long |
| Request Path | `91 0A 54 6F 74 61 6C 43 6F 75 6E 74` | ANSI Ext. Symbolic Segment for TotalCount |
| Request Data | `D6 06` | Number of elements to read (1750) |
| Request Data | `EA 01 00 00` | Start at this byte offset (490)<br>`0x000001EA` = 490 decimal (little-endian) |

**2nd Message Reply Field:**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Reply Service | `D2` | Read Tag Fragmented Service (Reply) |
| Reserved | `00` | (Empty) |
| General Status | `06` | Reply Data Too Large |
| Extended Status Size | `00` | No extended status |
| Reply Data | `C2 00` | SINT Tag Type Value |
| Reply Data | `nn nn nn...nn` | Data for Elements 490 through 979 |

**3rd Message Request Field:**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | `52` | Read Tag Fragmented Service (Request) |
| Request Path Size | `06` | Request Path is 6 words (12 bytes) long |
| Request Path | `91 0A 54 6F 74 61 6C 43 6F 75 6E 74` | ANSI Ext. Symbolic Segment for TotalCount |
| Request Data | `D6 06` | Number of elements to read (1750) |
| Request Data | `D4 03 00 00` | Start at this byte offset (980)<br>`0x000003D4` = 980 decimal (little-endian) |

**3rd Message Reply Field:**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Reply Service | `D2` | Read Tag Fragmented Service (Reply) |
| Reserved | `00` | (Empty) |
| General Status | `06` | Reply Data Too Large |
| Extended Status Size | `00` | No extended status |
| Reply Data | `C2 00` | SINT Tag Type Value |
| Reply Data | `nn nn nn...nn` | Data for Elements 980 through 1469 |

**4th Message Request Field:**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | `52` | Read Tag Fragmented Service (Request) |
| Request Path Size | `06` | Request Path is 6 words (12 bytes) long |
| Request Path | `91 0A 54 6F 74 61 6C 43 6F 75 6E 74` | ANSI Ext. Symbolic Segment for TotalCount |
| Request Data | `D6 06` | Number of elements to read (1750) |
| Request Data | `BE 05 00 00` | Start at this byte offset (1470)<br>`0x000005BE` = 1470 decimal (little-endian) |

**4th Message Reply Field:**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Reply Service | `D2` | Read Tag Fragmented Service (Reply) |
| Reserved | `00` | (Empty) |
| General Status | `00` | Success |
| Extended Status Size | `00` | No extended status |
| Reply Data | `C2 00` | SINT Tag Type Value |
| Reply Data | `nn nn nn...nn` | Data for Elements 1470 through 1749 |

**Key Observations:**
- Each request specifies the total number of elements (1750) and the byte offset to start from
- The offset increases with each request: 0 → 490 → 980 → 1470
- Each reply contains approximately 490 elements worth of data
- The last reply has status `00` (Success) instead of `06` (Reply Data Too Large)

---

## Write Tag Fragmented Service

**Service Code:** `0x53` (Request), `0xD3` (Reply)

### Purpose

Enables client applications to write data to a controller tag when the data is too large to fit into a single packet (approximately 500 bytes). The client must issue a series of requests to the controller to write all the data.

**Key Points:**
- The "Request Service", "Request Path Size", "Request Path", and "Number of Elements" fields remain the same for each request in a fragmented write sequence
- The client must update the "byte offset" field value with each subsequent request, based on the number of bytes transferred in the previous request
- The "Byte Offset" field is always expressed in bytes, regardless of the data type being written
- For SINT (1 byte), elements and offset are in the same units
- For other data types, the units differ

### Example: Writing 1750 SINTs to "TotalCount"

**1st Message Request Field:**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | `53` | Write Tag Fragmented Service (Request) |
| Request Path Size | `06` | Request Path is 6 words (12 bytes) long |
| Request Path | `91 0A 54 6F 74 61 6C 43 6F 75 6E 74` | ANSI Ext. Symbolic Segment for TotalCount |
| Request Data | `C2 00` | SINT Tag Type Value (0x00C2 = SINT) |
| Request Data | `D6 06` | Total number of elements to write (1750)<br>`0x06D6` = 1750 decimal (little-endian) |
| Request Data | `00 00 00 00` | Start at this byte offset (0) |
| Request Data | `nn nn nn...nn` | Data for elements 0 through 489 |

**Subsequent Requests:**
- 2nd request: Offset = 490, Data for elements 490-979
- 3rd request: Offset = 980, Data for elements 980-1469
- 4th request: Offset = 1470, Data for elements 1470-1749

---

## Array Element Addressing

### Correct Implementation

According to the CIP specification (1756-PM020, Page 603), when addressing array elements, the Request Path should include:

1. **Symbolic Segment** (0x91) for the base array tag name
2. **Element ID Segment** (0x28) for element addressing

### Example 1: Accessing Single Array Element

**From Page 603 of 1756-PM020:**

Access 5th element of array named `count`:
```
91 05 63 6F 75 6E 74 00 28 05
```

**Breakdown:**
- `91 05 63 6F 75 6E 74 00` = ANSI Extended Symbol Segment for "count"
  - `91`: Symbolic segment type
  - `05`: Length (5 bytes)
  - `63 6F 75 6E 74`: ASCII "count"
  - `00`: Padding
- `28 05` = 8-bit Element ID segment (element 5)
  - `28`: Element segment type (8-bit Element ID)
  - `05`: Element index (single byte, value 5)

### Element Addressing Format

**For Single Element Access:**
```
[0x28] [Index: 1 byte]
```

**For Multiple Elements:**
The element count is specified in **Request Data**, not in the path. The path contains the starting element index using `0x28` segment.

**Example: Reading elements 5-10 of array "count":**
```
Request Path: 91 05 63 6F 75 6E 74 00 28 05  // "count" + element 5
Request Data: 06 00  // Number of elements to read (6)
```

**Note:** The `0x28` segment uses a single byte for the element index (8-bit Element ID). For indices > 255, you would need to use `0x29` (16-bit) or `0x2A` (32-bit) Element ID segments.

### Alternative: Using Fragmented Service for Arrays

For large arrays, the **Read Tag Fragmented Service** (0x52) can be used:

**Request Structure:**
```
[Service: 0x52]
[Path Size: N words]
[Request Path: Symbolic segment for array name]
[Request Data: Element Count (2 bytes)]
[Request Data: Byte Offset (4 bytes)]
```

**Key Differences from Current Implementation:**
- Uses service code `0x52` instead of `0x4C`
- Specifies byte offset (4 bytes) instead of element index
- Can read specific ranges by specifying offset
- Returns status `06` (Reply Data Too Large) when more data is available

### Recommended Approach

1. **For single element access:** Use element addressing segment (0x28) in the path
2. **For multiple elements:** Use Read Tag Fragmented Service (0x52) with byte offset
3. **For specific ranges:** Use fragmented service with calculated byte offsets

---

## Data Type Codes

Common CIP data type codes used in tag operations:

| Data Type | Code (hex) | Size (bytes) |
|-----------|-----------|--------------|
| BOOL | `0x00C1` | 1 |
| SINT | `0x00C2` | 1 |
| INT | `0x00C3` | 2 |
| DINT | `0x00C4` | 4 |
| LINT | `0x00C5` | 8 |
| USINT | `0x00C6` | 1 |
| UINT | `0x00C7` | 2 |
| UDINT | `0x00C8` | 4 |
| ULINT | `0x00C9` | 8 |
| REAL | `0x00CA` | 4 |
| LREAL | `0x00CB` | 8 |
| STRING | `0x00CE` | Variable |
| UDT | `0x00A0` | Variable |
| DWORD (BOOL array) | `0x00D3` | 4 |

---

## Key Takeaways for Array Implementation

1. **Element Addressing:** Use `0x28` segment in Request Path for array element access
2. **Fragmented Service:** Use service `0x52` for reading large arrays with byte offset
3. **Byte Offset:** Always expressed in bytes, regardless of data type
4. **Element Count:** Specified in Request Data, not in the path
5. **Response Status:** `06` means "Reply Data Too Large" - more data available
6. **Little-Endian:** All multi-byte values use little-endian byte order

---

## References

- Allen-Bradley Publication 1756-PM020: Logix Controller Access Data
- CIP Networks Library, Volume 1, Chapter 2 (Message Router Request/Response format)
- CIP Networks Library, Volume 1, Appendix C (ANSI Extended Symbol Segment)
- CIP Networks Library, Volume 1, Appendix B (General status codes)
- EtherNet/IP Adaptation of CIP, Volume 2

---

**Document Status:** Partial - Awaiting additional sections on array element addressing examples

