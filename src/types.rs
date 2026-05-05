use std::collections::HashMap;
use tokio::time::Instant;

/// Connected session information for Class 3 explicit messaging.
///
/// This structure tracks the state needed for connected-message flows used by
/// some internal operations and experiments. It does not imply that connected
/// messaging bypasses documented PLC firmware restrictions such as direct
/// standalone `STRING` writes.
#[derive(Debug, Clone)]
pub struct ConnectedSession {
    /// Connection ID assigned by the PLC
    pub connection_id: u32,

    /// Our connection ID (originator -> target)
    pub o_to_t_connection_id: u32,

    /// PLC's connection ID (target -> originator)
    pub t_to_o_connection_id: u32,

    /// Connection serial number for this session
    pub connection_serial: u16,

    /// Originator vendor ID (our vendor ID)
    pub originator_vendor_id: u16,

    /// Originator serial number (our serial number)
    pub originator_serial: u32,

    /// Connection timeout multiplier
    pub timeout_multiplier: u8,

    /// Requested Packet Interval (RPI) in microseconds
    pub rpi: u32,

    /// Connection parameters for O->T direction
    pub o_to_t_params: ConnectionParameters,

    /// Connection parameters for T->O direction
    pub t_to_o_params: ConnectionParameters,

    /// Timestamp when connection was established
    pub established_at: Instant,

    /// Whether this connection is currently active
    pub is_active: bool,

    /// Sequence counter for connected messages (increments with each message)
    pub sequence_count: u16,
}

/// Connection parameters for EtherNet/IP connections
#[derive(Debug, Clone)]
pub struct ConnectionParameters {
    /// Connection size in bytes
    pub size: u16,

    /// Connection type (0x02 = Point-to-point, 0x01 = Multicast)
    pub connection_type: u8,

    /// Priority (0x00 = Low, 0x01 = High, 0x02 = Scheduled, 0x03 = Urgent)
    pub priority: u8,

    /// Variable size flag
    pub variable_size: bool,
}

impl Default for ConnectionParameters {
    fn default() -> Self {
        Self {
            size: 500,             // 500 bytes default
            connection_type: 0x02, // Point-to-point
            priority: 0x01,        // High priority
            variable_size: false,
        }
    }
}

impl ConnectedSession {
    /// Creates a new connected session with default parameters
    pub fn new(connection_serial: u16) -> Self {
        Self {
            connection_id: 0,
            o_to_t_connection_id: 0,
            t_to_o_connection_id: 0,
            connection_serial,
            originator_vendor_id: 0x1337,   // Custom vendor ID
            originator_serial: 0x1234_5678, // Custom serial number
            timeout_multiplier: 0x05,       // 32 seconds timeout
            rpi: 100_000,                   // 100ms RPI
            o_to_t_params: ConnectionParameters::default(),
            t_to_o_params: ConnectionParameters::default(),
            established_at: Instant::now(),
            is_active: false,
            sequence_count: 0,
        }
    }

    /// Creates a connected session with alternative parameters for different PLCs
    pub fn with_config(connection_serial: u16, config_id: u8) -> Self {
        let mut session = Self::new(connection_serial);

        match config_id {
            1 => {
                // Config 1: Conservative Allen-Bradley parameters
                session.timeout_multiplier = 0x07; // 256 seconds timeout
                session.rpi = 200_000; // 200ms RPI (slower)
                session.o_to_t_params.size = 504; // Standard packet size
                session.t_to_o_params.size = 504;
                session.o_to_t_params.priority = 0x00; // Low priority
                session.t_to_o_params.priority = 0x00;
                tracing::debug!("CONFIG 1: Conservative: 504 bytes, 200ms RPI, low priority");
            }
            2 => {
                // Config 2: Compact parameters
                session.timeout_multiplier = 0x03; // 8 seconds timeout
                session.rpi = 50000; // 50ms RPI (faster)
                session.o_to_t_params.size = 256; // Smaller packet size
                session.t_to_o_params.size = 256;
                session.o_to_t_params.priority = 0x02; // Scheduled priority
                session.t_to_o_params.priority = 0x02;
                tracing::debug!("CONFIG 2: Compact: 256 bytes, 50ms RPI, scheduled priority");
            }
            3 => {
                // Config 3: Minimal parameters
                session.timeout_multiplier = 0x01; // 4 seconds timeout
                session.rpi = 1_000_000; // 1000ms RPI (very slow)
                session.o_to_t_params.size = 128; // Very small packets
                session.t_to_o_params.size = 128;
                session.o_to_t_params.priority = 0x03; // Urgent priority
                session.t_to_o_params.priority = 0x03;
                tracing::debug!("CONFIG 3: Minimal: 128 bytes, 1000ms RPI, urgent priority");
            }
            4 => {
                // Config 4: Standard Rockwell parameters (from documentation)
                session.timeout_multiplier = 0x05; // 32 seconds timeout
                session.rpi = 100_000; // 100ms RPI
                session.o_to_t_params.size = 500; // Standard size
                session.t_to_o_params.size = 500;
                session.o_to_t_params.connection_type = 0x01; // Multicast
                session.t_to_o_params.connection_type = 0x01;
                session.originator_vendor_id = 0x001D; // Rockwell vendor ID
                tracing::debug!(
                    "CONFIG 4: Rockwell standard: 500 bytes, 100ms RPI, multicast, Rockwell vendor"
                );
            }
            5 => {
                // Config 5: Large buffer parameters
                session.timeout_multiplier = 0x0A; // Very long timeout
                session.rpi = 500_000; // 500ms RPI
                session.o_to_t_params.size = 1024; // Large packets
                session.t_to_o_params.size = 1024;
                session.o_to_t_params.variable_size = true; // Variable size
                session.t_to_o_params.variable_size = true;
                tracing::debug!("CONFIG 5: Large buffer: 1024 bytes, 500ms RPI, variable size");
            }
            _ => {
                // Default config
                tracing::debug!("CONFIG 0: Default parameters");
            }
        }

        session
    }
}

/// Represents the different data types supported by Allen-Bradley PLCs
///
/// These correspond to the CIP data type codes used in EtherNet/IP
/// communication. Each variant maps to a specific 16-bit type identifier
/// that the PLC uses to describe tag data.
///
/// # Supported Data Types
///
/// ## Integer Types
/// - **SINT**: 8-bit signed integer (-128 to 127)
/// - **INT**: 16-bit signed integer (-32,768 to 32,767)
/// - **DINT**: 32-bit signed integer (-2,147,483,648 to 2,147,483,647)
/// - **LINT**: 64-bit signed integer (-9,223,372,036,854,775,808 to 9,223,372,036,854,775,807)
///
/// ## Unsigned Integer Types
/// - **USINT**: 8-bit unsigned integer (0 to 255)
/// - **UINT**: 16-bit unsigned integer (0 to 65,535)
/// - **UDINT**: 32-bit unsigned integer (0 to 4,294,967,295)
/// - **ULINT**: 64-bit unsigned integer (0 to 18,446,744,073,709,551,615)
///
/// ## Floating Point Types
/// - **REAL**: 32-bit IEEE 754 float (±1.18 × 10^-38 to ±3.40 × 10^38)
/// - **LREAL**: 64-bit IEEE 754 double (±2.23 × 10^-308 to ±1.80 × 10^308)
///
/// ## Other Types
/// - **BOOL**: Boolean value (true/false)
/// - **STRING**: Variable-length string
/// - **UDT**: User Defined Type (structured data)
///
/// Represents raw UDT (User Defined Type) data
///
/// This structure stores UDT data in a generic format that works for any UDT
/// without requiring knowledge of member names. The `symbol_id` (template instance ID)
/// is required for writing UDTs back to the PLC, and the raw bytes can be parsed
/// later when the UDT definition is available.
///
/// # Usage
///
/// To write a UDT, you typically need to read it first to get the `symbol_id`.
/// While it's technically possible to calculate the symbol_id, it's much safer
/// to enforce a read of the UDT before writing to it.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UdtData {
    /// The template instance ID (symbol_id) from the PLC
    /// This is required for writing UDTs back to the PLC
    pub symbol_id: i32,
    /// Raw UDT data bytes
    /// This can be parsed into member values when the UDT definition is known
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PlcValue {
    /// Boolean value (single bit)
    ///
    /// Maps to CIP type 0x00C1. In CompactLogix PLCs, BOOL tags
    /// are stored as single bits but transmitted as bytes over the network.
    Bool(bool),

    /// 8-bit signed integer (-128 to 127)
    ///
    /// Maps to CIP type 0x00C2. Used for small numeric values,
    /// status codes, and compact data storage.
    Sint(i8),

    /// 16-bit signed integer (-32,768 to 32,767)
    ///
    /// Maps to CIP type 0x00C3. Common for analog input/output values,
    /// counters, and medium-range numeric data.
    Int(i16),

    /// 32-bit signed integer (-2,147,483,648 to 2,147,483,647)
    ///
    /// Maps to CIP type 0x00C4. This is the most common integer type
    /// in Allen-Bradley PLCs, used for counters, setpoints, and numeric values.
    Dint(i32),

    /// 64-bit signed integer (-9,223,372,036,854,775,808 to 9,223,372,036,854,775,807)
    ///
    /// Maps to CIP type 0x00C5. Used for large counters, timestamps,
    /// and high-precision calculations.
    Lint(i64),

    /// 8-bit unsigned integer (0 to 255)
    ///
    /// Maps to CIP type 0x00C6. Used for byte data, small counters,
    /// and status flags.
    Usint(u8),

    /// 16-bit unsigned integer (0 to 65,535)
    ///
    /// Maps to CIP type 0x00C7. Common for analog values, port numbers,
    /// and medium-range unsigned data.
    Uint(u16),

    /// 32-bit unsigned integer (0 to 4,294,967,295)
    ///
    /// Maps to CIP type 0x00C8. Used for large counters, memory addresses,
    /// and unsigned calculations.
    Udint(u32),

    /// 64-bit unsigned integer (0 to 18,446,744,073,709,551,615)
    ///
    /// Maps to CIP type 0x00C9. Used for very large counters, timestamps,
    /// and high-precision unsigned calculations.
    Ulint(u64),

    /// 32-bit IEEE 754 floating point number
    ///
    /// Maps to CIP type 0x00CA. Used for analog values, calculations,
    /// and any data requiring decimal precision.
    /// Range: ±1.18 × 10^-38 to ±3.40 × 10^38
    Real(f32),

    /// 64-bit IEEE 754 floating point number
    ///
    /// Maps to CIP type 0x00CB. Used for high-precision calculations,
    /// scientific data, and extended-range floating point values.
    /// Range: ±2.23 × 10^-308 to ±1.80 × 10^308
    Lreal(f64),

    /// String value.
    ///
    /// The library uses Allen-Bradley `STRING` encoding (`0x00CE`) for normal tag
    /// operations. Some controller responses may surface related string
    /// encodings, but callers should treat this variant as the standard Logix
    /// `STRING` representation.
    String(String),

    /// User Defined Type instance.
    ///
    /// The public API represents UDT values as [`UdtData`], which carries the
    /// template instance ID (`symbol_id`) plus the raw bytes required for
    /// read-modify-write flows. The raw bytes can be parsed once a UDT
    /// definition is available via [`UdtData::parse`].
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /// # let mut client = rust_ethernet_ip::EipClient::connect("192.168.1.100:44818").await?;
    /// use rust_ethernet_ip::PlcValue;
    /// let value = client.read_tag("MyUDT").await?;
    /// if let PlcValue::Udt(udt_data) = value {
    ///     let udt_def = client.get_udt_definition("MyUDT").await?;
    ///     // Convert UdtDefinition to UserDefinedType
    ///     let mut user_def = rust_ethernet_ip::udt::UserDefinedType::new(udt_def.name.clone());
    ///     for member in &udt_def.members {
    ///         user_def.add_member(member.clone());
    ///     }
    ///     let members = udt_data.parse(&user_def)?;
    ///     // Access members via HashMap
    /// }
    /// # Ok(())
    /// # }
    /// ```
    Udt(UdtData),
}

impl UdtData {
    /// Parses raw UDT bytes into member values using a UDT definition.
    ///
    /// This method converts generic `UdtData` bytes into a structured map of
    /// member names to values. It requires a UDT definition to interpret the
    /// layout correctly.
    ///
    /// Use `EipClient::get_udt_definition()` to obtain the definition from the PLC first.
    ///
    /// # Arguments
    ///
    /// * `definition` - The UDT definition containing member information (offsets, types, etc.)
    ///
    /// # Returns
    ///
    /// A HashMap mapping member names to their parsed `PlcValue` values
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /// # let mut client = rust_ethernet_ip::EipClient::connect("192.168.1.100:44818").await?;
    /// use rust_ethernet_ip::PlcValue;
    /// let udt_value = client.read_tag("MyUDT").await?;
    /// if let PlcValue::Udt(udt_data) = udt_value {
    ///     let udt_def = client.get_udt_definition("MyUDT").await?;
    ///     // Convert UdtDefinition to UserDefinedType
    ///     let mut user_def = rust_ethernet_ip::udt::UserDefinedType::new(udt_def.name.clone());
    ///     for member in &udt_def.members {
    ///         user_def.add_member(member.clone());
    ///     }
    ///     let members = udt_data.parse(&user_def)?;
    ///     
    ///     if let Some(PlcValue::Dint(value)) = members.get("Member1") {
    ///         println!("Member1 value: {}", value);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse(
        &self,
        definition: &crate::udt::UserDefinedType,
    ) -> crate::error::Result<HashMap<String, PlcValue>> {
        definition.to_hash_map(&self.data)
    }

    /// Creates `UdtData` from member values and a UDT definition.
    ///
    /// This method serializes member values back into raw bytes according to
    /// the supplied UDT definition. It is intended for read-modify-write flows
    /// where you need to update members and then write the whole UDT value back.
    ///
    /// # Arguments
    ///
    /// * `members` - HashMap of member names to `PlcValue` values
    /// * `definition` - The UDT definition containing member information (offsets, types, etc.)
    /// * `symbol_id` - The template instance ID (symbol_id) for this UDT. Typically obtained
    ///   by reading the UDT first.
    ///
    /// # Returns
    ///
    /// `UdtData` containing the serialized bytes and symbol_id, ready to be written back
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /// # let mut client = rust_ethernet_ip::EipClient::connect("192.168.1.100:44818").await?;
    /// use rust_ethernet_ip::{PlcValue, UdtData};
    /// // Read existing UDT to get symbol_id
    /// let udt_value = client.read_tag("MyUDT").await?;
    /// let udt_def = client.get_udt_definition("MyUDT").await?;
    ///
    /// if let PlcValue::Udt(mut udt_data) = udt_value {
    ///     // Convert UdtDefinition to UserDefinedType
    ///     let mut user_def = rust_ethernet_ip::udt::UserDefinedType::new(udt_def.name.clone());
    ///     for member in &udt_def.members {
    ///         user_def.add_member(member.clone());
    ///     }
    ///     // Parse to modify members
    ///     let mut members = udt_data.parse(&user_def)?;
    ///     members.insert("Member1".to_string(), PlcValue::Dint(42));
    ///
    ///     // Serialize back to UdtData
    ///     let modified_udt = UdtData::from_hash_map(&members, &user_def, udt_data.symbol_id)?;
    ///     client.write_tag("MyUDT", PlcValue::Udt(modified_udt)).await?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_hash_map(
        members: &HashMap<String, PlcValue>,
        definition: &crate::udt::UserDefinedType,
        symbol_id: i32,
    ) -> crate::error::Result<Self> {
        let data = definition.from_hash_map(members)?;
        Ok(UdtData { symbol_id, data })
    }
}

impl PlcValue {
    /// Converts the PLC value to its byte representation for network transmission
    ///
    /// This function handles the little-endian byte encoding required by
    /// the EtherNet/IP protocol. Each data type has specific encoding rules:
    ///
    /// - BOOL: Single byte (0x00 = false, 0xFF = true)
    /// - SINT: Single signed byte
    /// - INT: 2 bytes in little-endian format
    /// - DINT: 4 bytes in little-endian format
    /// - LINT: 8 bytes in little-endian format
    /// - USINT: Single unsigned byte
    /// - UINT: 2 bytes in little-endian format
    /// - UDINT: 4 bytes in little-endian format
    /// - ULINT: 8 bytes in little-endian format
    /// - REAL: 4 bytes IEEE 754 little-endian format
    /// - LREAL: 8 bytes IEEE 754 little-endian format
    ///
    /// # Returns
    ///
    /// A vector of bytes ready for transmission to the PLC
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            PlcValue::Bool(val) => vec![if *val { 0xFF } else { 0x00 }],
            PlcValue::Sint(val) => val.to_le_bytes().to_vec(),
            PlcValue::Int(val) => val.to_le_bytes().to_vec(),
            PlcValue::Dint(val) => val.to_le_bytes().to_vec(),
            PlcValue::Lint(val) => val.to_le_bytes().to_vec(),
            PlcValue::Usint(val) => val.to_le_bytes().to_vec(),
            PlcValue::Uint(val) => val.to_le_bytes().to_vec(),
            PlcValue::Udint(val) => val.to_le_bytes().to_vec(),
            PlcValue::Ulint(val) => val.to_le_bytes().to_vec(),
            PlcValue::Real(val) => val.to_le_bytes().to_vec(),
            PlcValue::Lreal(val) => val.to_le_bytes().to_vec(),
            PlcValue::String(val) => {
                // Try minimal approach - just length + data without padding
                // Testing if the PLC accepts a simpler format

                let mut bytes = Vec::new();

                // Length field (4 bytes as DINT) - number of characters currently used
                let length = val.len().min(82) as u32;
                bytes.extend_from_slice(&length.to_le_bytes());

                // String data - just the actual characters, no padding
                let string_bytes = val.as_bytes();
                let data_len = string_bytes.len().min(82);
                bytes.extend_from_slice(&string_bytes[..data_len]);

                bytes
            }
            PlcValue::Udt(udt_data) => {
                // Return the raw UDT data bytes
                udt_data.data.clone()
            }
        }
    }

    /// Returns the CIP data type code for this value
    ///
    /// These codes are defined by the CIP specification and must match
    /// exactly what the PLC expects for each data type.
    ///
    /// # Returns
    ///
    /// The 16-bit CIP type code for this value type
    pub fn get_data_type(&self) -> u16 {
        match self {
            PlcValue::Bool(_) => 0x00C1,   // BOOL
            PlcValue::Sint(_) => 0x00C2,   // SINT (signed char)
            PlcValue::Int(_) => 0x00C3,    // INT (short)
            PlcValue::Dint(_) => 0x00C4,   // DINT (int)
            PlcValue::Lint(_) => 0x00C5,   // LINT (long long)
            PlcValue::Usint(_) => 0x00C6,  // USINT (unsigned char)
            PlcValue::Uint(_) => 0x00C7,   // UINT (unsigned short)
            PlcValue::Udint(_) => 0x00C8,  // UDINT (unsigned int)
            PlcValue::Ulint(_) => 0x00C9,  // ULINT (unsigned long long)
            PlcValue::Real(_) => 0x00CA,   // REAL (float)
            PlcValue::Lreal(_) => 0x00CB,  // LREAL (double)
            PlcValue::String(_) => 0x00CE, // Allen-Bradley STRING type
            PlcValue::Udt(_) => 0x00A0,    // UDT placeholder
        }
    }
}
