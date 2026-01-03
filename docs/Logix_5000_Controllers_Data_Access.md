# Logix 5000 Controllers Data Access

**Programming Manual - Original Instructions**

**Publication 1756-PM020I-EN-P - September 2025**

Applicable Controllers:
- 1756 ControlLogix
- 1756 GuardLogix
- 1769 CompactLogix
- 1769 Compact GuardLogix
- 1789 SoftLogix
- 5069 CompactLogix
- 5069 Compact GuardLogix

---

## Important User Information

Read this document and the documents listed in the additional resources section about installation, configuration, and operation of this equipment before you install, configure, operate, or maintain this product. Users are required to familiarize themselves with installation and wiring instructions in addition to requirements of all applicable codes, laws, and standards.

Activities including installation, adjustments, putting into service, use, assembly, disassembly, and maintenance are required to be carried out by suitably trained personnel in accordance with applicable code of practice.

If this equipment is used in a manner not specified by the manufacturer, the protection provided by the equipment may be impaired.

In no event will Rockwell Automation, Inc. be responsible or liable for indirect or consequential damages resulting from the use or application of this equipment.

---

## Preface

Before using this document:
- Have a thorough understanding of CIP and EtherNet/IP
- Have purchased a copy of the pertinent volumes of the CIP Networks Library
- Be properly licensed through ODVA to use the CIP technology

For more information on the CIP Networks Library and CIP technologies, contact ODVA at http://www.odva.org/

### Studio 5000 Environment

The Studio 5000 Automation Engineering & Design Environment® combines engineering and design elements into a common environment. The first element is the Studio 5000 Logix Designer® application. The Logix Designer application is the rebranding of RSLogix 5000® software and will continue to be the product to program Logix 5000™ controllers for discrete, process, batch, motion, safety, and drive-based solutions.

### Additional Resources

| Resource | Description |
|----------|-------------|
| Industrial Automation Wiring and Grounding Guidelines, publication 1770-4.1 | Provides general guidelines for installing a Rockwell Automation industrial system |
| Logix 5000™ Controllers Design Considerations, publication 1756-RM094 | Provides information to help design and plan Logix 5000 systems |

---

# Chapter 1: CIP Services

Communicating with Logix 5000 controllers requires using CIP explicit messaging. This chapter describes the subset of the CIP explicit messaging constructs for understanding the service explanations that follow.

## CIP Services Overview

Before using CIP services, review introductory information:
- CIP data types
- Logix 5000 data
- Tag Type Service parameter
- Segment encoding
- CIP Service Request/Response format

## CIP Data Types

Data type information is very important in all aspects of CIP communication. The type information is used for reading, writing, and, if necessary, deciphering structures. The Logix 5000 controller supports these data types:

- **Atomic**: A bit, byte, 16-bit word, or 32-bit word, each of which stores a single value. (CIP refers to these as Elementary Data Types.)
- **Structure**: A grouping of different data types that functions as a single unit and serves a specific purpose. Depending on the needs of the application, create additional structures, which are referred to as user-defined structures.
- **Array**: A sequence of elements, each of which is the same data type.
  - Define data in one, two, or three dimensions, as required (one dimension is the most common)
  - Use atomic or structure data types

Data in the controller is organized as tags. The tags come in two basic types: atomic and structure. Atomic types can be arrayed or singular, and are very easy to work with. Structure types provide a great deal of flexibility, but are more challenging to access.

### Atomic Data Type Sizes

| To Store | Use This Data Type |
|----------|-------------------|
| Bit | BOOL |
| Bit array | DWORD (32-bit boolean array) |
| 8-bit integer | SINT |
| 16-bit integer | INT |
| 32-bit integer | DINT |
| 32-bit float | REAL |
| 64-bit integer | LINT |

## Logix 5000 Data

The Logix 5000 controller stores data in tags, in contrast to a PLC-5 or SLC controller, which stores data in data files. Logix 5000 tags have these properties:

- **Name** that identifies the data (up to 40 characters in length)
- **Scope**:
  - Controller (global), accessed directly
  - Program (local), which cannot be directly accessed, but can be copied to a controller scope tag
- **Data type**, which defines the organization of the data

> **Note:** In the Logix Designer application, version 21.00.00 and later, and in RSLogix 5000 software, version 18.00.00 and later, external access to controller scoped tags is user selectable. If a tag's External Access attribute is set to None, then the tag cannot be accessed from outside the controller.

## Tag Type Service Parameter

The Read tag, Write Tag, Read Tag Fragmented, Write Tag Fragmented, and Read-Modify-Write Tag services require a service parameter that identifies the data type of the tag being referenced. This tag type parameter is:

- A 16-bit value for atomic tags
- Two 16-bit values for structured tags

### Tag Type Service Parameter Values Used with Logix Controllers

| Data Type | Tag Type Value | Size of Transmitted Data |
|-----------|---------------|-------------------------|
| BOOL | 0x0nc1 (n = bit position 0-7) | 1 byte |
| SINT | 0x00C2 | 1 byte |
| INT | 0x00C3 | 2 bytes |
| DINT | 0x00C4 | 4 bytes |
| REAL | 0x00CA | 4 bytes |
| DWORD | 0x00D3 | 4 bytes |
| LINT | 0x00C5 | 8 bytes |

**Note:** Multi-byte data values are transmitted low-byte first.

### Tag Type Service Parameters for Structures

Structures use a Tag Type Service Parameter that is different from the one used with atomic tags. The Tag Type Service Parameter for structures is a 4-byte sequence:

```
A0 02 Structure_Handle_Low_Byte Structure_Handle_High_Byte
```

The Structure Handle comes from Template instance attribute 1 of the template instance associated with the tag.

> **IMPORTANT:** Reading a structure before writing to it is one way to obtain the value for this parameter, but that does not provide any understanding of the structure makeup, which is critical information when manipulating structure data. The correct way to access structures as a whole is to first read their template information and understand the data packing.

## Segment Encoding

The Request Path in a CIP explicit message contains addressing information indicating which internal resource in the target node directs the service. This addressing information is organized by using Logical Segments, Symbolic Segments, or both.

### Logical Segments

#### Element ID Segments

| Segment Type | Value | Byte Order |
|--------------|-------|------------|
| 8-bit Element ID | 0x28 | Value |
| 16-bit Element ID | 0x29 | 00, Low, High |
| 32-bit Element ID | 0x2A | 00, Lowest, Low, High, Highest |

#### Class ID Segments

| Segment Type | Value | Byte Order |
|--------------|-------|------------|
| 8-bit Class ID | 0x20 | Value |
| 16-bit Class ID | 0x21 | 00, Low, High |

#### Instance ID Segments

| Segment Type | Value | Byte Order |
|--------------|-------|------------|
| 8-bit Instance ID | 0x24 | Value |
| 16-bit Instance ID | 0x25 | 00, Low, High |

#### Attribute ID Segments

| Segment Type | Value | Byte Order |
|--------------|-------|------------|
| 8-bit Attribute ID | 0x30 | Value |
| 16-bit Attribute ID | 0x31 | 00, Low, High |

### Symbolic Segments

CIP defines a way to reference items by their symbolic name using the ANSI Extended Symbol Segment.

| Segment Type | Value | Byte Order |
|--------------|-------|------------|
| ANSI Extended Symbolic | 0x91 | Length, 1st char, ..., Nth Char |

## CIP Service Request/Response Format

All CIP services follow the Message Router Request/Response format. All requests take this form:

| Message Request Field | Description |
|----------------------|-------------|
| Request Service | Indicates to the object referenced in the request path to perform a task |
| Request Path Size | A byte value that indicates the number of 16-bit words in the Request Path |
| Request Path | A variable sized field that consists of one or more segments |
| Request Data | The service-specific data that is delivered to the object |

All responses take this form:

| Message Response Field | Description |
|-----------------------|-------------|
| Reply Service | The request service with the MSB set to 1 |
| Reserved | 00 |
| General Status | An 8-bit value indicating success or error status |
| Extended Status Size | An 8-bit value that indicates how many 16-bit values follow |
| Extended Status | The array of 16-bit values that describe the general status code |
| Reply Data | The data returned by the service request |

---

## Services Supported by Logix 5000 Controllers

The following vendor-specific services operate on tags in the controller using symbolic addressing:

- Read Tag Service (0x4c)
- Read Tag Fragmented Service (0x52)
- Write Tag Service (0x4d)
- Write Tag Fragmented Service (0x53)
- Read Modify Write Tag Service (0x4e)

### Addressing Methods

| Addressing Method | How it Works | When to Use |
|-------------------|--------------|-------------|
| **Symbolic Segment** | Uses the name of the tag in ASCII format using ANSI Extended Symbolic Segments. Allows direct access to tags as displayed in Data Monitor. | For best performance in applications that access small to moderate amounts of data |
| **Symbol Instance** | Uses the instance ID of the symbol class for the tag. Client must retrieve symbol instance information to associate name with instance ID. | For best performance in applications that access a large number of tags |

---

## Read Tag Service (0x4c)

The Read Tag Service reads the data associated with the tag specified in the path.

- Any data that fits into the reply packet is returned, even if it does not all fit
- If all the data does not fit into the packet, the error 0x06 is returned along with the data
- When reading a two or three dimensional array of data, all dimensions must be specified
- When reading a BOOL tag:
  - For 5570 and 5370 controller versions 29 and earlier: values returned for 0 and 1 are 0 and 0xFF
  - For versions 30 and later: values returned for 0 and 1 are 0 and 1

### Example Using Symbolic Segment Addressing

Read a single tag named `rate` with data type DINT and value 534:

**Request:**
| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | 4C | Read Tag Service (Request) |
| Request Path Size | 06 | Request Path is 6 words (12 bytes) long |
| Request Path | 91 0A 54 6F 74 61 6C 43 6F 75 6E 74 | ANSI Ext. Symbolic Segment for TotalCount |
| Request Data | 01 00 | Number of elements to read (1) |

**Reply:**
| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Reply Service | CC | Read Tag Service (Reply) |
| Reserved | 00 | - |
| General Status | 00 | Success |
| Extended Status Size | 00 | No extended status |
| Reply Data | C4 00 | DINT Tag Type Value |
| | 16 02 00 00 | 0x00000216 = 534 decimal |

### Example Using Symbol Instance Addressing

**Request:**
| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | 4C | Read Tag Service (Request) |
| Request Path Size | 03 | Request Path is 3 words (6 bytes) long |
| Request Path | 20 6B 25 00 8F F6 | Logical Segment for Symbol Class ID and Instance ID |
| Request Data | 01 00 | Number of elements to read (1) |

### Read Tag Service Error Codes

| Error Code (Hex) | Extended Error (Hex) | Description |
|-----------------|---------------------|-------------|
| 0x04 | 0x0000 | A syntax error was detected decoding the Request Path |
| 0x05 | 0x0000 | Request Path destination unknown: Probably instance number is not present |
| 0x06 | N/A | Insufficient Packet Space: Not enough room in the response buffer for all the data |
| 0x13 | N/A | Insufficient Request Data: Data too short for expected parameters |
| 0x26 | N/A | The Request Path Size received was shorter or longer than expected |
| 0xFF | 0x2105 | General Error: Access beyond end of the object |

---

## Read Tag Fragmented Service (0x52)

The Read Tag Fragmented Service enables client applications to read a tag with data that does not fit into a single packet (approximately 500 bytes). The client must issue a series of requests to the controller to retrieve the data using this service.

The Byte Offset field is expressed in number of bytes regardless of the data type being read.

### Example: Reading 1750 SINTs

Reading the tag `TotalCount` that has 1750 SINTs consists of four service requests:

**1st Message Request:**
| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | 52 | Read Tag Fragmented Service (Request) |
| Request Path Size | 06 | Request Path is 6 words (12 bytes) long |
| Request Path | 91 0A 54 6F 74 61 6C 43 6F 75 6E 74 | ANSI Ext. Symbolic Segment for TotalCount |
| Request Data | D6 06 | Number of elements to read (1750) |
| | 00 00 00 00 | Start at this byte offset (0) |

**1st Message Reply:**
| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Reply Service | D2 | Read Tag Fragmented Service (Reply) |
| General Status | 06 | Reply Data Too Large |
| Reply Data | C2 00 | SINT Tag Type Value |
| | nn nn nn...nn | Data for Elements 0 through 489 |

Continue with subsequent requests, adjusting the offset (490, 980, 1470) until General Status = 00 (Success) is returned.

### Read Tag Fragmented Service Error Codes

| Error Code (Hex) | Extended Error (Hex) | Description |
|-----------------|---------------------|-------------|
| 0x04 | 0x0000 | A syntax error was detected decoding the Request Path |
| 0x05 | 0x0000 | Request Path destination unknown |
| 0x06 | N/A | Insufficient Packet Space |
| 0x13 | N/A | Insufficient Request Data |
| 0x26 | N/A | Request Path Size was shorter or longer than expected |
| 0xFF | 0x2105 | General Error: Number of Elements or Byte Offset is beyond end of tag |

---

## Write Tag Service (0x4d)

The Write Tag Service writes the data associated with the tag specified in the path. The tag type must match for the write to occur.

- When writing a two or three dimensional array of data, all dimensions must be specified
- When writing to a BOOL tag, any non-zero value is interpreted as 1

### Example Using Symbolic Segment Addressing

Write the value of 14 to a DINT tag named `CartonSize`:

**Request:**
| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | 4D | Write Tag Service (Request) |
| Request Path Size | 06 | Request Path is 6 words (12 bytes) long |
| Request Path | 91 0A 43 61 72 74 6F 6E 53 69 7A 65 | ANSI Ext. Symbolic Segment for CartonSize |
| | C4 00 | DINT Tag Type Value |
| | 01 00 | Number of elements to write (1) |
| Request Data | 0E 00 00 00 | Data 0x0000000E = 14 decimal |

**Reply:**
| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Reply Service | CD | Write Tag Service (Reply) |
| General Status | 00 | Success |

### Write Tag Service Error Codes

| Error Code (Hex) | Extended Error (Hex) | Description |
|-----------------|---------------------|-------------|
| 0x04 | 0x0000 | A syntax error was detected decoding the Request Path |
| 0x05 | 0x0000 | Request Path destination unknown |
| 0x10 | 0x2101 | Device state conflict: keyswitch position |
| 0x10 | 0x2802 | Device state conflict: Safety Status |
| 0x13 | N/A | Insufficient Request Data |
| 0x26 | N/A | Request Path Size was shorter or longer than expected |
| 0xFF | 0x2105 | General Error: Number of Elements extends beyond end of tag |
| 0xFF | 0x2107 | General Error: Tag type used in request does not match target tag's data type |

---

## Write Tag Fragmented Service (0x53)

The Write Tag Fragmented Service enables client applications to write to a tag whose data will not fit into a single packet (approximately 500 bytes).

The Request Service, Request Path Size, Request Path, and Number of Elements fields remain the same for each request. The client must change the byte offset field value with each request.

### Write Tag Fragmented Service Error Codes

| Error Code (Hex) | Extended Error (Hex) | Description |
|-----------------|---------------------|-------------|
| 0x04 | 0x0000 | A syntax error was detected decoding the Request Path |
| 0x05 | 0x0000 | Request Path destination unknown |
| 0x10 | 0x2101 | Device state conflict: keyswitch position |
| 0x10 | 0x2802 | Device state conflict: Safety Status |
| 0x13 | N/A | Insufficient Request Data |
| 0x26 | N/A | Request Path Size was shorter or longer than expected |
| 0xFF | 0x2104 | General Error: Offset is beyond end of tag |
| 0xFF | 0x2105 | General Error: Offset plus Number of Elements extends beyond end of tag |
| 0xFF | 0x2107 | General Error: Data type used in request does not match target tag |

---

## Read Modify Write Tag Service (0x4e)

The Read Modify Write Tag Service modifies Tag data with individual bit resolution. The controller reads the Tag data, applies the logical OR/AND modification masks, and writes the data back.

### Service Request Parameters

| Name | Description | Semantics of Values |
|------|-------------|---------------------|
| Size of masks | Size in bytes of modify masks | Only 1, 2, 4, 8, 12 accepted |
| OR masks | Array of OR modify masks | 1 mask sets bit to 1 |
| AND masks | Array of AND modify masks | 0 mask resets bit to 0 |

### Example: Set bit 2 and reset bit 5 of DINT named ControlWord

**Request:**
| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | 4E | Read Modify Write Tag Service (Request) |
| Request Path Size | 07 | Request Path is 7 words (14 bytes) long |
| Request Path | 91 0B 43 6F 6E 74 72 6F 6C 57 6F 72 64 00 | ANSI Ext. Symbolic Segment for ControlWord |
| | 04 00 | Size of Masks (4) |
| | 04 00 00 00 | Array of OR modify masks |
| Request Data | DF FF FF FF | Array of AND modify masks |

### Read Modify Write Tag Service Error Codes

| Error Code (Hex) | Extended Error (Hex) | Description |
|-----------------|---------------------|-------------|
| 0x03 | N/A | Bad parameter, size > 12 or size greater than size of element |
| 0x04 | 0x0000 | A syntax error was detected decoding the Request Path |
| 0x05 | 0x0000 | Request Path destination unknown |
| 0x10 | 0x2101 | Device state conflict: keyswitch position |
| 0x10 | 0x2802 | Device state conflict: Safety Status |
| 0x13 | N/A | Insufficient Request Data |
| 0x26 | N/A | Request Path Size was shorter or longer than expected |

---

## Multiple Service Packet Service (0x0a)

The Multiple Service Packet Service conducts more than one CIP request in a single CIP explicit-message frame. Use this service to optimize CIP reads and writes by grouping service requests together for faster request processing.

### Example: Multiple Service

**Request:**
| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | 0A | Multiple Service Packet Service (Request) |
| Request Path Size | 02 | Request Path is 2 words (4 bytes) long |
| Request Path | 20 02 24 01 | Logical Segment: Class 0x02, Instance 01 (Message Router) |
| | 02 00 | Number of Services |
| | 06 00, 12 00 | Offsets for each Service |
| Request Data | 4C 04 91 05 70 61 72 74 73 00 01 00 | First Request: Read Tag "parts" |
| | 4C 07 91 0B ... 01 00 | Second Request: Read Tag "ControlWord" |

**Reply:**
| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Reply Service | 8A | Multiple Service Packet Service (Reply) |
| General Status | 00 | Success |
| | 02 00 | Number of Service Replies |
| | 06 00, 10 00 | Offsets for each Service Reply |
| Reply Data | CC 00 00 00 C4 00 2A 00 00 00 | First Reply: DINT value 42 |
| | CC 00 00 00 C4 00 DC 01 00 00 | Second Reply: DINT value 476 |

---

# Logix Data Structures

A structure is a compound data type that stores a group of possibly different data types that function as a single unit and serve a specific purpose.

- A structure contains one or more members
- Each member can be an atomic data type, another structured data type, or a single dimension array

The controller contains these basic types of structures:
- **Module-Defined data types** - created by adding modules to the I/O tree
- **Predefined data types** - created by default in the controller (e.g., TON, CTU, Motion)
- **Add-On-Defined data types**
- **User-Defined data types (UDT)** - created by the user

## Work with Data Structures

Guidelines for working with data structures:

1. Complete user-defined structure tags, or individual members can be accessed
2. Do not access complete UDT tags that contain nested system structures (Module-Defined, Predefined, or Add-On Defined)
3. For Predefined, Module-Defined, and Add-On-Defined structure tags, access atomic members instead of complete structures

**Access Methods:**
- Create an alias of the atomic member and access the alias instead
- Create an atomic tag or UDT structure tag with an atomic member, copy data to/from it

## Understanding Structure Boundaries and Member Order

### Structure Boundaries

Structures begin and end on 32-bit boundaries. A Logix5000 controller aligns every data type along:
- An 8-bit boundary for SINTs
- A 16-bit boundary for INTs
- A 32-bit boundary for DINTs, REALs, arrays and BOOL arrays (BOOL[nx32])

In a UDT structure, BOOLs are mapped to a host SINT member. If placing BOOLs next to each other in a UDT structure, they share the same SINT.

### Member Order

For normal UDTs that do not include any nested system structures, the size and member order is shown in the Data Monitor view of the structure definition in the Logix Designer application with padding to meet the alignment rules.

---

# Chapter 2: CIP Services and User-created Tags

This chapter describes processes that CIP clients use when interacting with Logix 5000 controller data:

1. Finding the controller-scope tags created in a Logix 5000 controller
2. Isolating user created tags from system tags and identify structured tags
3. Locating the structure template for a specific structure
4. Determining the structure makeup of a specific structure
5. Determining the data packing of the members of a structure when accessed as a whole
6. Determining when to refresh the list of tags and structure information

## How Tags Are Organized in the Controller

A client application interacts with the **Symbol** and **Template** objects associated with tags in Logix 5000 controllers.

### Symbol Object

Creating a tag creates an instance of the Symbol class (Class ID 0x6B) in the controller:
- The name of the tag is stored in attribute 1
- The data type of the tag is stored in attribute 2

The `Get_Instance_Attribute_List` service helps find instances and retrieve name and type attributes.

### Template Object

When creating a user-defined data type, an instance of the Template object (Class ID 0x6C) is created to hold information about the structure makeup:
- Its name
- The member list
- The number of members
- The size of the structure when read or written
- The Structure Handle

## Create and Maintain a Symbol Object List

### Step 1: Find User-Created Controller Scope Tags

Use the `Get_Instance_Attribute_List` (0x55) service to retrieve Symbol Name and Symbol Type attributes for each instance.

**Process:**
1. Set initial instance to zero
2. Send request
3. When General Status = 06 is returned, there is more data to read
4. Parse data to find last instance ID returned
5. Add one to the instance number
6. Repeat until General Status = 00 (Success)

### Step 2: Isolate User-Created Tags from System Tags

**Symbol Type Attribute Decoding:**

| Tag Type | Condition | Description |
|----------|-----------|-------------|
| Atomic tag | Bit 15 = 0, Bit 12 = 0 | Bits 0-11 are the Tag Type Service Parameter |
| Structured tag | Bit 15 = 1, Bit 12 = 0 | Bits 0-11 are the instance ID of the template object |

**Array Dimensions (Bits 13-14):**
| Bit 14 | Bit 13 | Meaning |
|--------|--------|---------|
| 0 | 0 | 0 dimensions (not an array) |
| 0 | 1 | 1 dimension array |
| 1 | 0 | 2 dimension array |
| 1 | 1 | 3 dimension array |

**Rules to Eliminate Non-User Tags:**

1. Discard tags not in these ranges:
   - Atomics: Bit 12=0, Bit 15=0, Bits 0-11 range 0x001-0x0FF
   - Structures: Bit 12=0, Bit 15=1, Bits 0-11 range 0x100-0xEFF
2. Discard tags with:
   - Leading double underscores (e.g., `__ABC`)
   - A colon (`:`) in the name
3. For remaining structures, check Template Name and first member name for leading double underscores or colons

### Step 3: Determine Structure Makeup

Read Template Instance attributes using `Get_Attribute_List` service (0x03):

| Attribute | Description | Data Type |
|-----------|-------------|-----------|
| 1 | Structure Handle (Tag Type Parameter) | UINT |
| 2 | Template Member Count | UINT |
| 4 | Template Object Definition Size (32-bit words) | UDINT |
| 5 | Template Structure Size (bytes) | UDINT |

**Template Read Service (0x4C):**

Number of bytes to read: `(Template Object Definition Size * 4) – 23`

**Member Information Format:**

For each member:
- Lower 16-bits (INFO value):
  - Atomic: 0
  - Array: array size (max 65535)
  - Boolean: bit location (0-31)
- Upper 16-bits: data type
- Second 32-bit value: offset location in the UDT structure

### Step 4: Determine Data Packing When Accessing Structure as a Whole

The data returned when reading a complete structure follows the member order shown in the Data Monitor view with appropriate padding for alignment.

### Step 5: Determine When to Refresh Tags List

Use `Get_Attribute_List` service to periodically retrieve attributes 1, 2, 3, 4, and 10 of class 0xAC. Changes in these values indicate changes to symbol or template instances.

---

# CIP Addressing Examples

This section provides detailed examples of how to construct request paths for accessing array elements, structure members, and combinations thereof using both Symbolic Segment Addressing and Symbol Instance Addressing methods.

## Array Element Addressing - Key Concepts

When addressing array elements, append a **Logical Segment for Element ID** after the tag name (symbolic) or instance ID:

| Element Value Range | Segment Type | Format |
|---------------------|--------------|--------|
| 0-255 | 8-bit Element ID | `0x28` + 1 byte value |
| 256-65535 | 16-bit Element ID | `0x29 0x00` + 2 bytes (low, high) |
| 65536+ | 32-bit Element ID | `0x2A 0x00` + 4 bytes (lowest to highest) |

---

## Atomic Members of Predefined Data Types

### Example 1: Symbolic Segment Addressing - Array Element Access

**Objective:** Access element 5 of a DINT array named `count`

**Request Path:**
```
91 05 63 6F 75 6E 74 00 28 05
```

**Breakdown:**
| Bytes | Description |
|-------|-------------|
| `91` | ANSI Extended Symbol Segment identifier |
| `05` | Length of tag name (5 characters) |
| `63 6F 75 6E 74` | ASCII for "count" |
| `00` | Pad byte (names with odd length need padding to word boundary) |
| `28` | 8-bit Element ID segment identifier |
| `05` | Element index = 5 |

**Full Read Request:**
| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | 4C | Read Tag Service |
| Request Path Size | 05 | 5 words (10 bytes) |
| Request Path | 91 05 63 6F 75 6E 74 00 28 05 | Tag "count", element 5 |
| Request Data | 01 00 | Read 1 element |

---

### Example 2: Symbol Instance Addressing - Array Element Access

**Objective:** Access element 5 of an array using Symbol Instance ID 0x0E51

**Request Path:**
```
20 6B 25 00 51 0E 28 05
```

**Breakdown:**
| Bytes | Description |
|-------|-------------|
| `20 6B` | 8-bit Class ID segment, Class 0x6B (Symbol) |
| `25 00` | 16-bit Instance ID segment identifier |
| `51 0E` | Instance ID = 0x0E51 (little-endian) |
| `28` | 8-bit Element ID segment identifier |
| `05` | Element index = 5 |

---

### Example 3: Symbolic Segment - 16-bit Element Index (Element > 255)

**Objective:** Access element 300 of array `LargeArray`

**Request Path:**
```
91 0A 4C 61 72 67 65 41 72 72 61 79 29 00 2C 01
```

**Breakdown:**
| Bytes | Description |
|-------|-------------|
| `91` | ANSI Extended Symbol Segment identifier |
| `0A` | Length = 10 characters |
| `4C 61 72 67 65 41 72 72 61 79` | ASCII for "LargeArray" |
| `29` | 16-bit Element ID segment identifier |
| `00` | Pad byte |
| `2C 01` | Element index = 0x012C = 300 (little-endian) |

---

### Example 4: Symbolic Segment - Structure Member Access

**Objective:** Access member `Temp` of structure tag `Machine`

**Request Path:**
```
91 07 4D 61 63 68 69 6E 65 00 91 04 54 65 6D 70
```

**Breakdown:**
| Bytes | Description |
|-------|-------------|
| `91 07` | Symbol segment, 7 chars |
| `4D 61 63 68 69 6E 65 00` | "Machine" + pad |
| `91 04` | Symbol segment, 4 chars |
| `54 65 6D 70` | "Temp" |

**Note:** Structure member access chains multiple symbolic segments together.

---

### Example 5: Symbol Instance - Structure Member by Offset

**Objective:** Access structure member using instance addressing

When using Symbol Instance addressing for structures, you typically access the entire structure and parse members client-side based on template information, OR use symbolic segment for the member name after the instance path.

---

### Example 6: Symbolic Segment - Array of Structures, Element Access

**Objective:** Access element 3 of structure array `Stations`

**Request Path:**
```
91 08 53 74 61 74 69 6F 6E 73 28 03
```

**Breakdown:**
| Bytes | Description |
|-------|-------------|
| `91 08` | Symbol segment, 8 chars |
| `53 74 61 74 69 6F 6E 73` | "Stations" |
| `28 03` | 8-bit Element ID, element 3 |

---

### Example 7: Symbolic Segment - Array of Structures, Member of Element

**Objective:** Access member `Count` of element 3 in structure array `Stations`

**Request Path:**
```
91 08 53 74 61 74 69 6F 6E 73 28 03 91 05 43 6F 75 6E 74 00
```

**Breakdown:**
| Bytes | Description |
|-------|-------------|
| `91 08` | Symbol segment, 8 chars |
| `53 74 61 74 69 6F 6E 73` | "Stations" |
| `28 03` | 8-bit Element ID, element 3 |
| `91 05` | Symbol segment, 5 chars |
| `43 6F 75 6E 74 00` | "Count" + pad |

This accesses `Stations[3].Count`

---

### Example 8: Both Addressing Methods - 2D Array Access

**Objective:** Access element [2,5] of 2D array `Grid`

**Symbolic Segment Method:**
```
91 04 47 72 69 64 28 02 28 05
```

**Breakdown:**
| Bytes | Description |
|-------|-------------|
| `91 04` | Symbol segment, 4 chars |
| `47 72 69 64` | "Grid" |
| `28 02` | 8-bit Element ID, first dimension = 2 |
| `28 05` | 8-bit Element ID, second dimension = 5 |

**Note:** Multi-dimensional arrays use multiple consecutive Element ID segments.

---

### Example 9: Both Addressing Methods - 3D Array Access

**Objective:** Access element [1,2,3] of 3D array `Cube`

**Request Path:**
```
91 04 43 75 62 65 28 01 28 02 28 03
```

**Breakdown:**
| Bytes | Description |
|-------|-------------|
| `91 04` | Symbol segment, 4 chars |
| `43 75 62 65` | "Cube" |
| `28 01` | Element ID, dim 0 = 1 |
| `28 02` | Element ID, dim 1 = 2 |
| `28 03` | Element ID, dim 2 = 3 |

---

### Example 10: Symbolic Segment Addressing with BOOLs

**Objective:** Access BOOL tag or BOOL array element

**For standalone BOOL tag `Running`:**
```
91 07 52 75 6E 6E 69 6E 67 00
```

**For BOOL array `Flags[5]`:**
```
91 05 46 6C 61 67 73 00 28 05
```

**Note:** When reading a BOOL, the controller returns 1 byte. The Tag Type in response will be `0x00C1` for BOOL at bit 0, `0x01C1` for bit 1, etc.

---

## Complete Read Tag Request Examples

### Reading Array Element - Full Message

**Read `MyArray[10]` (DINT array):**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | 4C | Read Tag Service |
| Request Path Size | 05 | 5 words (10 bytes) |
| Request Path | 91 07 4D 79 41 72 72 61 79 00 28 0A | "MyArray" + element 10 |
| Request Data | 01 00 | Read 1 element |

**Response:**
| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Reply Service | CC | Read Tag Reply |
| Reserved | 00 | - |
| General Status | 00 | Success |
| Extended Status Size | 00 | - |
| Reply Data | C4 00 | DINT type (0x00C4) |
| | xx xx xx xx | 4-byte DINT value (little-endian) |

---

### Reading Multiple Array Elements Starting at Index

**Read 5 elements starting at `MyArray[10]`:**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | 4C | Read Tag Service |
| Request Path Size | 05 | 5 words (10 bytes) |
| Request Path | 91 07 4D 79 41 72 72 61 79 00 28 0A | "MyArray" + element 10 |
| Request Data | 05 00 | Read 5 elements |

**Response contains:** Tag type + 5 consecutive DINT values (20 bytes of data)

---

### Writing to Array Element - Full Message

**Write value 0x12345678 to `MyArray[10]`:**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | 4D | Write Tag Service |
| Request Path Size | 05 | 5 words (10 bytes) |
| Request Path | 91 07 4D 79 41 72 72 61 79 00 28 0A | "MyArray" + element 10 |
| Request Data | C4 00 | DINT type |
| | 01 00 | Write 1 element |
| | 78 56 34 12 | Value 0x12345678 (little-endian) |

---

## Element ID Segment Size Selection

Use the appropriate Element ID segment based on the index value:

```
if (index <= 255):
    # 8-bit Element ID: 0x28 + index
    segment = [0x28, index]
    
elif (index <= 65535):
    # 16-bit Element ID: 0x29, 0x00, low_byte, high_byte
    segment = [0x29, 0x00, index & 0xFF, (index >> 8) & 0xFF]
    
else:
    # 32-bit Element ID: 0x2A, 0x00, byte0, byte1, byte2, byte3
    segment = [0x2A, 0x00, 
               index & 0xFF, 
               (index >> 8) & 0xFF,
               (index >> 16) & 0xFF,
               (index >> 24) & 0xFF]
```

---

## ANSI Extended Symbol Segment Construction

```
def build_symbol_segment(tag_name: str) -> bytes:
    name_bytes = tag_name.encode('ascii')
    length = len(name_bytes)
    
    # Start with segment type and length
    segment = bytes([0x91, length]) + name_bytes
    
    # Pad to word boundary if odd length
    if length % 2 == 1:
        segment += b'\x00'
    
    return segment
```

**Examples:**
| Tag Name | Length | Segment Bytes |
|----------|--------|---------------|
| "rate" | 4 | `91 04 72 61 74 65` |
| "count" | 5 | `91 05 63 6F 75 6E 74 00` (padded) |
| "TotalCount" | 10 | `91 0A 54 6F 74 61 6C 43 6F 75 6E 74` |

---

# Access User-Defined Structures

This section provides examples for accessing User-Defined Types (UDTs) including complete structures, individual members, and arrays within structures.

## UDT Access Patterns

### Example 1: Read Entire UDT Structure

**Objective:** Read complete UDT tag `MachineData` of type `MACHINE_UDT`

**Request:**
| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | 4C | Read Tag Service |
| Request Path Size | 06 | 6 words (12 bytes) |
| Request Path | 91 0B 4D 61 63 68 69 6E 65 44 61 74 61 00 | "MachineData" |
| Request Data | 01 00 | Read 1 element (entire structure) |

**Response:**
| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Reply Service | CC | Read Tag Reply |
| General Status | 00 | Success |
| Reply Data | A0 02 xx xx | Structure Tag Type (0x02A0 + Structure Handle) |
| | ... | Structure data bytes per template layout |

**Note:** The Structure Handle (xx xx) comes from Template Instance Attribute 1.

---

### Example 2: Read Single Member of UDT

**Objective:** Read member `Speed` from UDT tag `MachineData`

**Request Path:**
```
91 0B 4D 61 63 68 69 6E 65 44 61 74 61 00 91 05 53 70 65 65 64 00
```

**Breakdown:**
| Bytes | Description |
|-------|-------------|
| `91 0B` | Symbol segment, 11 chars |
| `4D 61 63 68 69 6E 65 44 61 74 61 00` | "MachineData" + pad |
| `91 05` | Symbol segment, 5 chars |
| `53 70 65 65 64 00` | "Speed" + pad |

This accesses `MachineData.Speed`

---

### Example 3: Read Array Member Within UDT

**Objective:** Read element 5 of array member `Counts` from UDT tag `MachineData`

**Request Path:**
```
91 0B 4D 61 63 68 69 6E 65 44 61 74 61 00 91 06 43 6F 75 6E 74 73 28 05
```

**Breakdown:**
| Bytes | Description |
|-------|-------------|
| `91 0B ... 00` | "MachineData" |
| `91 06` | Symbol segment, 6 chars |
| `43 6F 75 6E 74 73` | "Counts" |
| `28 05` | Element 5 |

This accesses `MachineData.Counts[5]`

---

### Example 4: Read Array of UDTs - Single Element

**Objective:** Read element 3 from array `Stations` where each element is a UDT

**Request Path:**
```
91 08 53 74 61 74 69 6F 6E 73 28 03
```

This accesses `Stations[3]` (entire UDT structure at index 3)

---

### Example 5: Read Member from Array of UDTs

**Objective:** Read member `Status` from element 3 of UDT array `Stations`

**Request Path:**
```
91 08 53 74 61 74 69 6F 6E 73 28 03 91 06 53 74 61 74 75 73
```

**Breakdown:**
| Bytes | Description |
|-------|-------------|
| `91 08` | Symbol segment, 8 chars |
| `53 74 61 74 69 6F 6E 73` | "Stations" |
| `28 03` | Element 3 |
| `91 06` | Symbol segment, 6 chars |
| `53 74 61 74 75 73` | "Status" |

This accesses `Stations[3].Status`

---

### Example 6: Nested UDT Member Access

**Objective:** Access nested structure member `Config.Setpoint` from tag `Controller`

**Request Path:**
```
91 0A 43 6F 6E 74 72 6F 6C 6C 65 72 91 06 43 6F 6E 66 69 67 91 08 53 65 74 70 6F 69 6E 74
```

**Breakdown:**
| Segment | Description |
|---------|-------------|
| `91 0A 43...72` | "Controller" |
| `91 06 43...67` | "Config" |
| `91 08 53...74` | "Setpoint" |

This accesses `Controller.Config.Setpoint`

---

## Writing to UDT Members

### Write to Single Member

**Write value 1500 to `MachineData.Speed` (INT type):**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | 4D | Write Tag Service |
| Request Path Size | 0A | 10 words |
| Request Path | 91 0B ... 91 05 53 70 65 65 64 00 | "MachineData.Speed" |
| Request Data | C3 00 | INT type (0x00C3) |
| | 01 00 | Write 1 element |
| | DC 05 | Value 1500 = 0x05DC |

### Write to Array Element Within UDT

**Write value 100 to `MachineData.Counts[5]` (DINT type):**

| Field | Bytes (hex) | Description |
|-------|-------------|-------------|
| Request Service | 4D | Write Tag Service |
| Request Path | ... | "MachineData.Counts" + element 5 |
| Request Data | C4 00 | DINT type |
| | 01 00 | Write 1 element |
| | 64 00 00 00 | Value 100 |

---

## UDT Data Layout Considerations

When reading/writing complete UDTs:

1. **Structure Handle Required:** Write operations require the correct Tag Type Service Parameter (0xA002 + Structure Handle from Template Attribute 1)

2. **Data Alignment:** Members are aligned per their data type:
   - SINT: 1-byte boundary
   - INT: 2-byte boundary  
   - DINT/REAL: 4-byte boundary
   - LINT: 8-byte boundary

3. **Padding:** The controller adds padding bytes between members to maintain alignment

4. **BOOL Mapping:** BOOLs are packed into hidden SINT host members (up to 8 BOOLs per SINT)

5. **Total Size:** Use Template Attribute 5 (Template Structure Size) to know exact byte count

---

## Unconnected Messaging (UCMM) through PCCC

Unconnected messages are used for CIP explicit messages that do not require a connection to be established first.

## Connected Explicit Messages through PCCC

Connected messages provide a dedicated communication path between devices.

## PCCC Commands

### Initial Fields of All PCCC Commands

| Field | Description |
|-------|-------------|
| DST | Destination node |
| SRC | Source node |
| CMD | Command byte |
| STS | Status |
| TNS | Transaction number |
| FNC | Function (if present) |

### PLC-2 Communication Commands

| Command | Description |
|---------|-------------|
| CMD=01, 41 | Unprotected Read |
| CMD=00, 40 | Protected Write |
| CMD=08, 48 | Unprotected Write |
| CMD=02, 42 | Protected Bit Write |
| CMD=05, 45 | Unprotected Bit Write |

### PLC-5 Communication Commands

| Command | Function | Description |
|---------|----------|-------------|
| CMD=0F, 4F | FNC=79 | Read Modify Write N |
| CMD=0F, 4F | FNC=68 | Typed Read |
| CMD=0F, 4F | FNC=67 | Typed Write |
| CMD=0F, 4F | FNC=01 | Word Range Read |
| CMD=0F, 4F | FNC=00 | Word Range Write |
| CMD=0F, 4F | FNC=02 | Bit Write |

### SLC Communication Commands

| Command | Function | Description |
|---------|----------|-------------|
| CMD=0F, 4F | FNC=A2 | SLC Protected Typed Logical Read with 3 Address Fields |
| CMD=0F, 4F | FNC=AA | SLC Protected Typed Logical Write with 3 Address Fields |
| CMD=0F, 4F | FNC=A1 | SLC Protected Typed Logical Read with 2 Address Fields |
| CMD=0F, 4F | FNC=A9 | SLC Protected Typed Logical Write with 2 Address Fields |

---

## Quick Reference Tables

### CIP Service Codes

| Service | Code | Description |
|---------|------|-------------|
| Read Tag | 0x4C | Read data from a tag |
| Read Tag Fragmented | 0x52 | Read large tag data in fragments |
| Write Tag | 0x4D | Write data to a tag |
| Write Tag Fragmented | 0x53 | Write large tag data in fragments |
| Read Modify Write Tag | 0x4E | Modify individual bits |
| Multiple Service Packet | 0x0A | Combine multiple requests |
| Get_Instance_Attribute_List | 0x55 | Get attributes for object instances |
| Get_Attribute_List | 0x03 | Get attributes from a class |
| Template Read | 0x4C | Read template structure information |

### Class IDs

| Class | ID | Description |
|-------|-----|-------------|
| Message Router | 0x02 | Routes CIP messages |
| Symbol | 0x6B | Tag definitions |
| Template | 0x6C | Structure definitions |
| Controller | 0xAC | Controller attributes |

### Common General Status Codes

| Code | Description |
|------|-------------|
| 0x00 | Success |
| 0x04 | Path syntax error |
| 0x05 | Path destination unknown |
| 0x06 | Insufficient packet space (more data available) |
| 0x10 | Device state conflict |
| 0x13 | Insufficient request data |
| 0x26 | Request path size error |
| 0xFF | General error (check extended status) |

---

*© Rockwell Automation, Inc. All rights reserved.*

*Publication 1756-PM020I-EN-P - September 2025*