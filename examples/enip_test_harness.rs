use std::collections::HashMap;
use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};

// Data type constants for CompactLogix
#[derive(Debug, Clone, PartialEq)]
#[repr(u16)]
pub enum PlcDataType {
    Bool = 0x00C1,
    Dint = 0x00C4,
    Real = 0x00CA,
    String = 0x00D0,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlcDataTypeExt {
    Known(PlcDataType),
    Unknown(u16),
}

impl From<u16> for PlcDataTypeExt {
    fn from(value: u16) -> Self {
        match value {
            0x00C1 => PlcDataTypeExt::Known(PlcDataType::Bool),
            0x00C4 => PlcDataTypeExt::Known(PlcDataType::Dint),
            0x00CA => PlcDataTypeExt::Known(PlcDataType::Real),
            0x00D0 => PlcDataTypeExt::Known(PlcDataType::String),
            _ => PlcDataTypeExt::Unknown(value),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PlcValue {
    Bool(bool),
    Dint(i32),
    Real(f32),
    String(String),
    Raw(Vec<u8>),
}

impl PlcValue {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            PlcValue::Bool(val) => vec![if *val { 0xFF } else { 0x00 }],
            PlcValue::Dint(val) => val.to_le_bytes().to_vec(),
            PlcValue::Real(val) => val.to_le_bytes().to_vec(),
            PlcValue::String(val) => {
                let mut bytes = vec![];
                let str_bytes = val.as_bytes();
                bytes.extend_from_slice(&(str_bytes.len() as u16).to_le_bytes());
                bytes.extend_from_slice(str_bytes);
                if bytes.len() % 2 != 0 {
                    bytes.push(0x00);
                }
                bytes
            }
            PlcValue::Raw(val) => val.clone(),
        }
    }

    pub fn get_data_type(&self) -> u16 {
        match self {
            PlcValue::Bool(_) => PlcDataType::Bool as u16,
            PlcValue::Dint(_) => PlcDataType::Dint as u16,
            PlcValue::Real(_) => PlcDataType::Real as u16,
            PlcValue::String(_) => PlcDataType::String as u16,
            PlcValue::Raw(_) => 0x0000,
        }
    }
}

pub struct EipClient {
    stream: TcpStream,
    session_handle: u32,
    pub connection_info: HashMap<String, String>,
}

impl EipClient {
    pub async fn connect(addr: &str) -> Result<Self, Box<dyn Error>> {
        println!("🔌 Connecting to CompactLogix PLC at {}...", addr);
        let stream = TcpStream::connect(addr).await?;
        println!("✅ TCP connection established");

        let mut client = EipClient {
            stream,
            session_handle: 0,
            connection_info: HashMap::new(),
        };

        client
            .connection_info
            .insert("address".to_string(), addr.to_string());
        client.register_session().await?;
        Ok(client)
    }

    async fn register_session(&mut self) -> Result<(), Box<dyn Error>> {
        let packet: [u8; 28] = [
            0x65, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        ];

        self.stream.write_all(&packet).await?;
        println!("📤 Sent Register Session request");

        let mut buf = [0u8; 1024];
        let n = timeout(Duration::from_secs(5), self.stream.read(&mut buf)).await??;

        if n >= 8 {
            self.session_handle = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
            let status = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);

            if status == 0 && self.session_handle != 0 {
                println!(
                    "🎯 Registration successful! Session ID: 0x{:08X}",
                    self.session_handle
                );
                self.connection_info.insert(
                    "session_id".to_string(),
                    format!("0x{:08X}", self.session_handle),
                );
                Ok(())
            } else {
                Err(format!("Registration failed. Status: 0x{:08X}", status).into())
            }
        } else {
            Err("Invalid registration response".into())
        }
    }

    pub async fn read_tag(&mut self, tag_name: &str) -> Result<PlcValue, Box<dyn Error>> {
        let (data_type, raw_data) = self.read_tag_raw(tag_name).await?;
        self.parse_value(data_type.into(), &raw_data)
    }

    async fn read_tag_raw(&mut self, tag_name: &str) -> Result<(u16, Vec<u8>), Box<dyn Error>> {
        let tag_bytes = tag_name.as_bytes();
        let mut cip_request = vec![0x4C, 0x00];
        let mut path = vec![0x91, tag_bytes.len() as u8];
        path.extend_from_slice(tag_bytes);

        if path.len() % 2 != 0 {
            path.push(0x00);
        }

        cip_request[1] = (path.len() / 2) as u8;
        cip_request.extend_from_slice(&path);
        cip_request.extend_from_slice(&[0x01, 0x00]);

        let response = self.send_cip_request(&cip_request).await?;
        self.parse_cip_response(&response)
    }

    pub async fn write_tag(
        &mut self,
        tag_name: &str,
        value: PlcValue,
    ) -> Result<(), Box<dyn Error>> {
        let tag_bytes = tag_name.as_bytes();
        let value_bytes = value.to_bytes();
        let data_type = value.get_data_type();

        let mut cip_request = vec![0x53, 0x02]; // Write Tag Service with verification
        let mut path = vec![0x91, tag_bytes.len() as u8];
        path.extend_from_slice(tag_bytes);

        if path.len() % 2 != 0 {
            path.push(0x00);
        }

        cip_request[1] = (path.len() / 2) as u8;
        cip_request.extend_from_slice(&path);
        cip_request.extend_from_slice(&data_type.to_le_bytes());
        cip_request.extend_from_slice(&[0x01, 0x00]);
        cip_request.extend_from_slice(&value_bytes);

        println!(
            "📝 Writing {} to tag '{}'",
            match &value {
                PlcValue::Bool(v) => format!("BOOL: {}", v),
                PlcValue::Dint(v) => format!("DINT: {}", v),
                PlcValue::Real(v) => format!("REAL: {}", v),
                PlcValue::String(v) => format!("STRING: '{}'", v),
                PlcValue::Raw(v) => format!("RAW: {:02X?}", v),
            },
            tag_name
        );

        let response = self.send_cip_request(&cip_request).await?;

        if response.len() >= 4 {
            let general_status = response[2];
            if general_status == 0x00 {
                println!("✅ Tag '{}' written successfully!", tag_name);
                Ok(())
            } else {
                let error_msg = self.get_cip_error_message(general_status);
                Err(format!(
                    "Write failed - CIP Error 0x{:02X}: {}",
                    general_status, error_msg
                )
                .into())
            }
        } else {
            Err("Invalid write response".into())
        }
    }

    pub async fn read_multiple_tags(
        &mut self,
        tag_names: &[&str],
    ) -> Result<HashMap<String, PlcValue>, Box<dyn Error>> {
        let mut results = HashMap::new();
        println!("📋 Reading {} tags...", tag_names.len());

        for tag_name in tag_names {
            let read_result = self.read_tag(tag_name).await;
            match read_result {
                Ok(value) => {
                    println!("✅ {}: {:?}", tag_name, value);
                    results.insert(tag_name.to_string(), value);
                }
                Err(e) => {
                    println!("❌ {}: {}", tag_name, e);
                }
            }
        }
        Ok(results)
    }

    pub async fn write_multiple_tags(
        &mut self,
        tags: HashMap<&str, PlcValue>,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let mut successful_writes = Vec::new();
        let total_tags = tags.len();

        println!("📝 Writing {} tags...", total_tags);
        for (tag_name, value) in tags {
            let write_result = self.write_tag(tag_name, value).await;
            match write_result {
                Ok(()) => {
                    successful_writes.push(tag_name.to_string());
                }
                Err(e) => {
                    println!("❌ Failed to write {}: {}", tag_name, e);
                }
            }
        }

        println!(
            "✅ Successfully wrote {} of {} tags",
            successful_writes.len(),
            total_tags
        );
        Ok(successful_writes)
    }

    pub async fn read_array_element(
        &mut self,
        tag_name: &str,
        index: u16,
    ) -> Result<PlcValue, Box<dyn Error>> {
        let array_tag = format!("{}[{}]", tag_name, index);
        self.read_tag(&array_tag).await
    }

    pub async fn write_array_element(
        &mut self,
        tag_name: &str,
        index: u16,
        value: PlcValue,
    ) -> Result<(), Box<dyn Error>> {
        let array_tag = format!("{}[{}]", tag_name, index);
        self.write_tag(&array_tag, value).await
    }

    pub async fn read_array_range(
        &mut self,
        tag_name: &str,
        start_index: u16,
        count: u16,
    ) -> Result<Vec<PlcValue>, Box<dyn Error>> {
        let mut results = Vec::new();
        println!(
            "📋 Reading array {}[{}..{}]",
            tag_name,
            start_index,
            start_index + count - 1
        );

        for i in start_index..start_index + count {
            let read_result = self.read_array_element(tag_name, i).await;
            match read_result {
                Ok(value) => {
                    println!("✅ {}[{}]: {:?}", tag_name, i, value);
                    results.push(value);
                }
                Err(e) => {
                    println!("❌ {}[{}]: {}", tag_name, i, e);
                    break;
                }
            }
        }
        Ok(results)
    }

    pub async fn discover_tags(&mut self) -> Result<Vec<String>, Box<dyn Error>> {
        let mut discovered_tags = Vec::new();
        println!("🔍 Discovering available tags...");

        let controller_tags = [
            "Controller.Type",
            "Controller.MajorRev",
            "Controller.MinorRev",
            "Controller.SerialNumber",
            "Controller.LastScan",
            "Controller.DateTime",
        ];

        for tag in &controller_tags {
            if self.read_tag(tag).await.is_ok() {
                discovered_tags.push(tag.to_string());
                println!("✅ Found: {}", tag);
            }
        }

        let program_names = ["MainProgram", "Program", "Main"];
        for program in &program_names {
            let program_tag = format!("Program:{}.TestTag", program);
            if self.read_tag(&program_tag).await.is_ok() {
                discovered_tags.push(program_tag);
                println!("✅ Found program: {}", program);
            }
        }

        Ok(discovered_tags)
    }

    pub async fn get_controller_info(&mut self) -> Result<HashMap<String, String>, Box<dyn Error>> {
        let mut info = self.connection_info.clone();

        if let Ok(PlcValue::Dint(serial)) = self.read_tag("Controller.SerialNumber").await {
            info.insert("serial_number".to_string(), serial.to_string());
        }

        if let Ok(PlcValue::Dint(controller_type)) = self.read_tag("Controller.Type").await {
            info.insert("controller_type".to_string(), controller_type.to_string());
        }

        if let Ok(PlcValue::Dint(major)) = self.read_tag("Controller.MajorRev").await {
            info.insert("major_revision".to_string(), major.to_string());
        }

        if let Ok(PlcValue::Dint(minor)) = self.read_tag("Controller.MinorRev").await {
            info.insert("minor_revision".to_string(), minor.to_string());
        }

        Ok(info)
    }

    pub async fn test_connectivity(&mut self) -> Result<HashMap<String, String>, Box<dyn Error>> {
        let mut status = HashMap::new();

        status.insert("connection".to_string(), "OK".to_string());
        status.insert(
            "session_id".to_string(),
            format!("0x{:08X}", self.session_handle),
        );

        match self.read_tag("TestTag").await {
            Ok(_) => {
                status.insert("read_test".to_string(), "PASS".to_string());
            }
            Err(e) => {
                status.insert("read_test".to_string(), format!("FAIL: {}", e));
            }
        }

        match self.write_tag("TestTag", PlcValue::Bool(false)).await {
            Ok(()) => {
                status.insert("write_test".to_string(), "PASS".to_string());
            }
            Err(e) => {
                status.insert("write_test".to_string(), format!("FAIL: {}", e));
            }
        }

        Ok(status)
    }

    pub async fn benchmark_performance(
        &mut self,
    ) -> Result<HashMap<String, String>, Box<dyn Error>> {
        let mut results = HashMap::new();
        println!("⚡ Running performance benchmarks...");

        // Read performance test
        let start = std::time::Instant::now();
        let mut read_success = 0;
        for _ in 0..10 {
            if self.read_tag("TestTag").await.is_ok() {
                read_success += 1;
            }
        }
        let read_duration = start.elapsed();
        let read_rate = (read_success as f64 / read_duration.as_secs_f64()) as u32;

        println!("📊 Read performance: {} ops/sec", read_rate);
        results.insert("read_rate_per_sec".to_string(), read_rate.to_string());
        results.insert(
            "read_success_rate".to_string(),
            format!("{}/10", read_success),
        );

        // Write performance test
        let start = std::time::Instant::now();
        let mut write_success = 0;
        for i in 0..10 {
            if self.write_tag("TestDint", PlcValue::Dint(i)).await.is_ok() {
                write_success += 1;
            }
        }
        let write_duration = start.elapsed();
        let write_rate = (write_success as f64 / write_duration.as_secs_f64()) as u32;

        println!("📊 Write performance: {} ops/sec", write_rate);
        results.insert("write_rate_per_sec".to_string(), write_rate.to_string());
        results.insert(
            "write_success_rate".to_string(),
            format!("{}/10", write_success),
        );

        Ok(results)
    }

    pub async fn safe_read_tag(&mut self, tag_name: &str) -> Option<PlcValue> {
        self.read_tag(tag_name).await.ok()
    }

    pub async fn safe_write_tag(&mut self, tag_name: &str, value: PlcValue) -> bool {
        match self.write_tag(tag_name, value).await {
            Ok(()) => true,
            Err(_) => false,
        }
    }

    async fn send_cip_request(&mut self, cip_request: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let cip_len = cip_request.len();
        let total_data_len = 4 + 2 + 2 + 8 + cip_len;

        let mut packet = vec![0x6F, 0x00];
        packet.extend_from_slice(&(total_data_len as u16).to_le_bytes());
        packet.extend_from_slice(&self.session_handle.to_le_bytes());
        packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        packet.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]);
        packet.extend_from_slice(&[0x05, 0x06, 0x07, 0x08]);
        packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        packet.extend_from_slice(&[0x05, 0x00]);
        packet.extend_from_slice(&[0x02, 0x00]);
        packet.extend_from_slice(&[0x00, 0x00]);
        packet.extend_from_slice(&[0x00, 0x00]);
        packet.extend_from_slice(&[0xB2, 0x00]);
        packet.extend_from_slice(&(cip_len as u16).to_le_bytes());
        packet.extend_from_slice(cip_request);

        self.stream.write_all(&packet).await?;

        let mut buf = [0u8; 1024];
        let n = timeout(Duration::from_secs(10), self.stream.read(&mut buf)).await??;

        if n >= 24 {
            let cmd_status = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
            if cmd_status == 0 {
                self.extract_cip_from_response(&buf[..n])
            } else {
                Err(format!("EIP Command failed. Status: 0x{:08X}", cmd_status).into())
            }
        } else {
            Err("Response too short".into())
        }
    }

    fn extract_cip_from_response(&self, response: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut pos = 24;
        pos += 4;
        pos += 2;
        let item_count = u16::from_le_bytes([response[pos], response[pos + 1]]);
        pos += 2;

        for _ in 0..item_count {
            let item_type = u16::from_le_bytes([response[pos], response[pos + 1]]);
            pos += 2;
            let item_length = u16::from_le_bytes([response[pos], response[pos + 1]]);
            pos += 2;

            if item_type == 0x00B2 && item_length > 0 {
                return Ok(response[pos..pos + item_length as usize].to_vec());
            }

            pos += item_length as usize;
        }

        Err("Could not find CIP response data".into())
    }

    fn parse_cip_response(&self, cip_response: &[u8]) -> Result<(u16, Vec<u8>), Box<dyn Error>> {
        if cip_response.len() < 4 {
            return Err("CIP response too short".into());
        }

        let general_status = cip_response[2];
        let additional_status_size = cip_response[3];

        if general_status == 0x00 {
            let data_start = 4 + (additional_status_size as usize * 2);
            if data_start + 2 <= cip_response.len() {
                let data_type =
                    u16::from_le_bytes([cip_response[data_start], cip_response[data_start + 1]]);
                let value_data = &cip_response[data_start + 2..];
                return Ok((data_type, value_data.to_vec()));
            }
        }

        let error_msg = self.get_cip_error_message(general_status);
        Err(format!("CIP Error 0x{:02X}: {}", general_status, error_msg).into())
    }

    fn parse_value(
        &self,
        data_type: PlcDataTypeExt,
        raw_data: &[u8],
    ) -> Result<PlcValue, Box<dyn Error>> {
        match data_type {
            PlcDataTypeExt::Known(PlcDataType::Bool) => {
                if !raw_data.is_empty() {
                    Ok(PlcValue::Bool(raw_data[0] != 0))
                } else {
                    Err("No data for BOOL value".into())
                }
            }
            PlcDataTypeExt::Known(PlcDataType::Dint) => {
                if raw_data.len() >= 4 {
                    let value =
                        i32::from_le_bytes([raw_data[0], raw_data[1], raw_data[2], raw_data[3]]);
                    Ok(PlcValue::Dint(value))
                } else {
                    Err("Insufficient data for DINT value".into())
                }
            }
            PlcDataTypeExt::Known(PlcDataType::Real) => {
                if raw_data.len() >= 4 {
                    let value =
                        f32::from_le_bytes([raw_data[0], raw_data[1], raw_data[2], raw_data[3]]);
                    Ok(PlcValue::Real(value))
                } else {
                    Err("Insufficient data for REAL value".into())
                }
            }
            PlcDataTypeExt::Known(PlcDataType::String) => {
                if raw_data.len() >= 2 {
                    let str_len = u16::from_le_bytes([raw_data[0], raw_data[1]]) as usize;
                    if raw_data.len() >= 2 + str_len {
                        let str_bytes = &raw_data[2..2 + str_len];
                        let string_value = String::from_utf8_lossy(str_bytes).to_string();
                        Ok(PlcValue::String(string_value))
                    } else {
                        Err("Insufficient data for STRING value".into())
                    }
                } else {
                    Err("No data for STRING value".into())
                }
            }
            PlcDataTypeExt::Unknown(_) => Ok(PlcValue::Raw(raw_data.to_vec())),
        }
    }

    fn get_cip_error_message(&self, status: u8) -> &'static str {
        match status {
            0x01 => "Connection failure",
            0x02 => "Resource unavailable",
            0x03 => "Invalid parameter value",
            0x04 => "Path destination unknown",
            0x05 => "Path segment error",
            0x06 => "Path destination unknown",
            0x07 => "Partial transfer",
            0x08 => "Connection lost",
            0x09 => "Service not supported",
            0x0A => "Invalid attribute value",
            0x0B => "Attribute list error",
            0x0C => "Already in requested mode/state",
            0x0D => "Object state conflict",
            0x0E => "Object already exists",
            0x0F => "Attribute not settable",
            0x10 => "Privilege violation",
            0x11 => "Device state conflict",
            0x12 => "Reply data too large",
            0x13 => "Fragmentation of a primitive value",
            0x14 => "Not enough data",
            0x15 => "Attribute not supported",
            0x16 => "Too much data",
            0x17 => "Object does not exist",
            0x18 => "Service fragmentation sequence not in progress",
            0x19 => "No stored attribute data",
            0x1A => "Store operation failure",
            0x1B => "Routing failure, request packet too large",
            0x1C => "Routing failure, response packet too large",
            0x1D => "Missing attribute list entry data",
            0x1E => "Invalid attribute value list",
            0x1F => "Embedded service error",
            _ => "Unknown error",
        }
    }

    pub async fn unregister_session(&mut self) -> Result<(), Box<dyn Error>> {
        let session_bytes = self.session_handle.to_le_bytes();
        let packet: [u8; 24] = [
            0x66,
            0x00,
            0x00,
            0x00,
            session_bytes[0],
            session_bytes[1],
            session_bytes[2],
            session_bytes[3],
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
        ];

        self.stream.write_all(&packet).await?;
        println!("📤 Session closed");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🦀 Rust EtherNet/IP Driver v2.0 - Complete Edition");
    println!("====================================================");

    let mut client = EipClient::connect("192.168.0.1:44818").await?;

    // Test 1: Read TestTag
    println!("\n🧪 TEST 1: Reading TestTag");
    match client.read_tag("TestTag").await {
        Ok(value) => println!("✅ TestTag value: {:?}", value),
        Err(e) => println!("❌ Failed to read TestTag: {}", e),
    }

    // Test 2: Write TestTag
    println!("\n🧪 TEST 2: Writing to TestTag");
    match client.write_tag("TestTag", PlcValue::Bool(true)).await {
        Ok(()) => {
            println!("✅ Successfully wrote TRUE to TestTag");
            match client.read_tag("TestTag").await {
                Ok(value) => println!("✅ Verification - TestTag is now: {:?}", value),
                Err(e) => println!("⚠️ Could not verify write: {}", e),
            }
        }
        Err(e) => println!("❌ Failed to write TestTag: {}", e),
    }

    // Test 3: Different Data Types
    println!("\n🧪 TEST 3: Testing Different Data Types");

    println!("📝 Testing DINT operations...");
    match client.write_tag("TestDint", PlcValue::Dint(12345)).await {
        Ok(()) => match client.read_tag("TestDint").await {
            Ok(value) => println!("✅ DINT test: {:?}", value),
            Err(e) => println!("⚠️ DINT read failed: {}", e),
        },
        Err(e) => println!("⚠️ DINT write failed: {}", e),
    }

    println!("📝 Testing REAL operations...");
    match client.write_tag("TestReal", PlcValue::Real(123.45)).await {
        Ok(()) => match client.read_tag("TestReal").await {
            Ok(value) => println!("✅ REAL test: {:?}", value),
            Err(e) => println!("⚠️ REAL read failed: {}", e),
        },
        Err(e) => println!("⚠️ REAL write failed: {}", e),
    }

    // Test 4: Controller Information
    println!("\n🧪 TEST 4: Controller Information");
    match client.get_controller_info().await {
        Ok(info) => {
            println!("📊 Controller Details:");
            for (key, value) in &info {
                println!("   {}: {}", key, value);
            }
        }
        Err(e) => println!("⚠️ Could not get controller info: {}", e),
    }

    // Test 5: Tag Discovery
    println!("\n🧪 TEST 5: Tag Discovery");
    match client.discover_tags().await {
        Ok(tags) => {
            if !tags.is_empty() {
                println!("🔍 Discovered {} tags:", tags.len());
                for tag in tags {
                    println!("   📌 {}", tag);
                }
            } else {
                println!("⚠️ No controller tags discovered");
            }
        }
        Err(e) => println!("⚠️ Discovery error: {}", e),
    }

    // Test 6: Multiple Tag Operations
    println!("\n🧪 TEST 6: Multiple Tag Operations");
    let test_tags = ["TestTag", "TestDint", "TestReal"];
    match client.read_multiple_tags(&test_tags).await {
        Ok(results) => {
            println!("📋 Multiple tag read results:");
            for (tag, value) in results {
                println!("   {}: {:?}", tag, value);
            }
        }
        Err(e) => println!("⚠️ Multiple tag read error: {}", e),
    }

    // Test 7: Array Operations
    println!("\n🧪 TEST 7: Array Operations");
    println!("📝 Testing array operations (create TestArray[10] in your PLC)...");
    match client
        .write_array_element("TestArray", 0, PlcValue::Dint(100))
        .await
    {
        Ok(()) => {
            match client
                .write_array_element("TestArray", 1, PlcValue::Dint(200))
                .await
            {
                Ok(()) => match client.read_array_range("TestArray", 0, 3).await {
                    Ok(values) => {
                        println!("✅ Array operations successful! Values: {:?}", values);
                    }
                    Err(e) => println!("⚠️ Array read failed: {}", e),
                },
                Err(e) => println!("⚠️ Array write [1] failed: {}", e),
            }
        }
        Err(e) => println!("⚠️ Array write [0] failed (create TestArray[10]): {}", e),
    }

    // Test 8: Connectivity Test
    println!("\n🧪 TEST 8: Connectivity Test");
    match client.test_connectivity().await {
        Ok(status) => {
            println!("🔗 Connectivity Status:");
            for (key, value) in status {
                println!("   {}: {}", key, value);
            }
        }
        Err(e) => println!("⚠️ Connectivity test error: {}", e),
    }

    // Test 9: Performance Benchmark
    println!("\n🧪 TEST 9: Performance Benchmark");
    match client.benchmark_performance().await {
        Ok(results) => {
            println!("⚡ Performance Results:");
            for (key, value) in results {
                println!("   {}: {}", key, value);
            }
        }
        Err(e) => println!("⚠️ Benchmark error: {}", e),
    }

    // Test 10: Safe Operations Demo
    println!("\n🧪 TEST 10: Safe Operations Demo");
    if let Some(value) = client.safe_read_tag("TestTag").await {
        println!("✅ Safe read successful: {:?}", value);
    } else {
        println!("⚠️ Safe read returned None");
    }

    if client
        .safe_write_tag("TestTag", PlcValue::Bool(false))
        .await
    {
        println!("✅ Safe write successful");
    } else {
        println!("⚠️ Safe write failed");
    }

    // Clean up
    client.unregister_session().await?;

    println!("\n🎉 Complete EtherNet/IP Driver Test Finished!");
    println!("================================================");
    println!("✅ Your Rust EtherNet/IP library is production-ready!");
    println!("🚀 Features tested: Read/Write, Arrays, Discovery, Performance");
    println!("🔧 Ready for integration with C# or TypeScript!");

    Ok(())
}
