use super::EipClient;
use crate::error::EtherNetIpError;
use crate::protocol::encap::{EncapsulationHeader, SEND_RR_DATA};
use crate::protocol::{Decode, Encode};
use crate::types::{ConnectedSession, ConnectionParameters, PlcValue};
use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::Instant;

impl EipClient {
    /// Writes a string value using Allen-Bradley UDT component access
    /// This writes to TestString.LEN and TestString.DATA separately
    pub async fn write_ab_string_components(
        &mut self,
        tag_name: &str,
        value: &str,
    ) -> crate::error::Result<()> {
        tracing::debug!(
            "[AB STRING] Writing string '{}' to tag '{}' using component access",
            value,
            tag_name
        );

        let string_bytes = value.as_bytes();
        let string_len = string_bytes.len() as i32;

        // Step 1: Write the length to TestString.LEN
        let len_tag = format!("{tag_name}.LEN");
        tracing::debug!("Step 1: Writing length {} to {}", string_len, len_tag);

        match self.write_tag(&len_tag, PlcValue::Dint(string_len)).await {
            Ok(_) => tracing::debug!("Length written successfully"),
            Err(e) => {
                tracing::error!("Length write failed: {}", e);
                return Err(e);
            }
        }

        // Step 2: Write the string data to TestString.DATA using array access
        tracing::debug!("Step 2: Writing string data to {}.DATA", tag_name);

        // We need to write each character individually to the DATA array
        for (i, &byte) in string_bytes.iter().enumerate() {
            let data_element = format!("{tag_name}.DATA[{i}]");
            match self
                .write_tag(&data_element, PlcValue::Sint(byte as i8))
                .await
            {
                Ok(_) => tracing::trace!("wrote STRING byte at position {}", i),
                Err(e) => {
                    tracing::error!("Failed to write byte {} to position {}: {}", byte, i, e);
                    return Err(e);
                }
            }
        }

        // Step 3: Clear remaining bytes (null terminate)
        if string_bytes.len() < 82 {
            let null_element = format!("{}.DATA[{}]", tag_name, string_bytes.len());
            match self.write_tag(&null_element, PlcValue::Sint(0)).await {
                Ok(_) => tracing::debug!("String null-terminated successfully"),
                Err(e) => tracing::warn!("Could not null-terminate: {}", e),
            }
        }

        tracing::info!("AB STRING component write completed!");
        Ok(())
    }

    /// Writes a string using a single UDT write with proper AB STRING format
    pub async fn write_ab_string_udt(
        &mut self,
        tag_name: &str,
        value: &str,
    ) -> crate::error::Result<()> {
        tracing::debug!(
            "[AB STRING UDT] Writing string '{}' to tag '{}' as UDT",
            value,
            tag_name
        );

        let string_bytes = value.as_bytes();
        if string_bytes.len() > 82 {
            return Err(EtherNetIpError::Protocol(
                "String too long for Allen-Bradley STRING (max 82 chars)".to_string(),
            ));
        }

        // Build a CIP request that writes the complete AB STRING structure
        let mut cip_request = Vec::new();

        // Service: Write Tag Service (0x4D)
        cip_request.push(0x4D);

        // Request Path
        let tag_path = self.build_tag_path(tag_name);
        cip_request.push((tag_path.len() / 2) as u8); // Path size in words
        cip_request.extend_from_slice(&tag_path);

        // Data Type: Allen-Bradley STRING (0x02A0) - but write as UDT components
        cip_request.extend_from_slice(&[0xA0, 0x00]); // UDT type
        cip_request.extend_from_slice(&[0x01, 0x00]); // Element count

        // AB STRING UDT structure:
        // - DINT .LEN (4 bytes)
        // - SINT .DATA[82] (82 bytes)

        // Write .LEN field (current string length)
        let len = string_bytes.len() as u32;
        cip_request.extend_from_slice(&len.to_le_bytes());

        // Write .DATA field (82 bytes total)
        cip_request.extend_from_slice(string_bytes); // Actual string data

        // Pad with zeros to reach 82 bytes
        let padding_needed = 82 - string_bytes.len();
        cip_request.extend_from_slice(&vec![0u8; padding_needed]);

        tracing::trace!("Built UDT write request: {} bytes total", cip_request.len());

        let response = self.send_cip_request(&cip_request).await?;

        if response.len() >= 3 {
            let general_status = response[2];
            if general_status == 0x00 {
                tracing::info!("AB STRING UDT write successful!");
                Ok(())
            } else {
                let error_msg = self.get_cip_error_message(general_status);
                Err(EtherNetIpError::Protocol(format!(
                    "AB STRING UDT write failed - CIP Error 0x{general_status:02X}: {error_msg}"
                )))
            }
        } else {
            Err(EtherNetIpError::Protocol(
                "Invalid AB STRING UDT write response".to_string(),
            ))
        }
    }

    /// Establishes a Class 3 connected session for STRING operations
    ///
    /// Connected sessions are required for certain operations like STRING writes
    /// in Allen-Bradley PLCs. This implements the Forward Open CIP service.
    /// Will try multiple connection parameter configurations until one succeeds.
    pub(super) async fn establish_connected_session(
        &mut self,
        session_name: &str,
    ) -> crate::error::Result<ConnectedSession> {
        tracing::debug!(
            "[CONNECTED] Establishing connected session: '{}'",
            session_name
        );
        tracing::debug!("[CONNECTED] Will try multiple parameter configurations...");

        // Generate unique connection parameters
        *self.connection_sequence.lock().await += 1;
        let connection_serial = (*self.connection_sequence.lock().await & 0xFFFF) as u16;

        // Try different configurations until one works
        for config_id in 0..=5 {
            tracing::debug!(
                "[ATTEMPT {}] Trying configuration {}:",
                config_id + 1,
                config_id
            );

            let mut session = if config_id == 0 {
                ConnectedSession::new(connection_serial)
            } else {
                ConnectedSession::with_config(connection_serial, config_id)
            };

            // Generate unique connection IDs for this attempt
            session.o_to_t_connection_id =
                0x2000_0000 + *self.connection_sequence.lock().await + (config_id as u32 * 0x1000);
            session.t_to_o_connection_id =
                0x3000_0000 + *self.connection_sequence.lock().await + (config_id as u32 * 0x1000);

            // Build Forward Open request with this configuration
            let forward_open_request = self.build_forward_open_request(&session)?;

            tracing::debug!(
                "[ATTEMPT {}] Sending Forward Open request ({} bytes)",
                config_id + 1,
                forward_open_request.len()
            );

            // Send Forward Open request
            match self.send_cip_request(&forward_open_request).await {
                Ok(response) => {
                    // Try to parse the response - DON'T clone, modify the session directly!
                    match self.parse_forward_open_response(&mut session, &response) {
                        Ok(()) => {
                            // Success! Store the session and return
                            tracing::info!("[SUCCESS] Configuration {} worked!", config_id);
                            tracing::debug!("Connection ID: 0x{:08X}", session.connection_id);
                            tracing::debug!("O->T ID: 0x{:08X}", session.o_to_t_connection_id);
                            tracing::debug!("T->O ID: 0x{:08X}", session.t_to_o_connection_id);
                            tracing::debug!(
                                "Using Connection ID: 0x{:08X} for messaging",
                                session.connection_id
                            );

                            session.is_active = true;
                            let mut sessions = self.connected_sessions.lock().await;
                            sessions.insert(session_name.to_string(), session.clone());
                            return Ok(session);
                        }
                        Err(e) => {
                            tracing::warn!(
                                "[ATTEMPT {}] Configuration {} failed: {}",
                                config_id + 1,
                                config_id,
                                e
                            );

                            // If it's a specific status error, log it
                            if e.to_string().contains("status: 0x") {
                                tracing::debug!(
                                    "Status indicates: parameter incompatibility or resource conflict"
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "[ATTEMPT {}] Network error with config {}: {}",
                        config_id + 1,
                        config_id,
                        e
                    );
                }
            }

            // Small delay between attempts
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        // If we get here, all configurations failed
        Err(EtherNetIpError::Protocol(
            "All connection parameter configurations failed. PLC may not support connected messaging or has reached connection limits.".to_string()
        ))
    }

    /// Builds a Forward Open CIP request for establishing connected sessions
    fn build_forward_open_request(
        &self,
        session: &ConnectedSession,
    ) -> crate::error::Result<Vec<u8>> {
        let mut request = Vec::with_capacity(50);

        // CIP Forward Open Service (0x54)
        request.push(0x54);

        // Request path length (Connection Manager object)
        request.push(0x02); // 2 words

        // Class ID: Connection Manager (0x06)
        request.push(0x20); // Logical Class segment
        request.push(0x06);

        // Instance ID: Connection Manager instance (0x01)
        request.push(0x24); // Logical Instance segment
        request.push(0x01);

        // Forward Open parameters

        // Connection Timeout Ticks (1 byte) + Timeout multiplier (1 byte)
        request.push(0x0A); // Timeout ticks (10)
        request.push(session.timeout_multiplier);

        // Originator -> Target Connection ID (4 bytes, little-endian)
        request.extend_from_slice(&session.o_to_t_connection_id.to_le_bytes());

        // Target -> Originator Connection ID (4 bytes, little-endian)
        request.extend_from_slice(&session.t_to_o_connection_id.to_le_bytes());

        // Connection Serial Number (2 bytes, little-endian)
        request.extend_from_slice(&session.connection_serial.to_le_bytes());

        // Originator Vendor ID (2 bytes, little-endian)
        request.extend_from_slice(&session.originator_vendor_id.to_le_bytes());

        // Originator Serial Number (4 bytes, little-endian)
        request.extend_from_slice(&session.originator_serial.to_le_bytes());

        // Connection Timeout Multiplier (1 byte) - repeated for target
        request.push(session.timeout_multiplier);

        // Reserved bytes (3 bytes)
        request.extend_from_slice(&[0x00, 0x00, 0x00]);

        // Originator -> Target RPI (4 bytes, little-endian, microseconds)
        request.extend_from_slice(&session.rpi.to_le_bytes());

        // Originator -> Target connection parameters (4 bytes)
        let o_to_t_params = self.encode_connection_parameters(&session.o_to_t_params);
        request.extend_from_slice(&o_to_t_params.to_le_bytes());

        // Target -> Originator RPI (4 bytes, little-endian, microseconds)
        request.extend_from_slice(&session.rpi.to_le_bytes());

        // Target -> Originator connection parameters (4 bytes)
        let t_to_o_params = self.encode_connection_parameters(&session.t_to_o_params);
        request.extend_from_slice(&t_to_o_params.to_le_bytes());

        // Transport type/trigger (1 byte) - Class 3, Application triggered
        request.push(0xA3);

        // Connection Path Size (1 byte)
        request.push(0x02); // 2 words for Message Router path

        // Connection Path - Target the Message Router
        request.push(0x20); // Logical Class segment
        request.push(0x02); // Message Router class (0x02)
        request.push(0x24); // Logical Instance segment
        request.push(0x01); // Message Router instance (0x01)

        Ok(request)
    }

    /// Encodes connection parameters into a 32-bit value
    fn encode_connection_parameters(&self, params: &ConnectionParameters) -> u32 {
        let mut encoded = 0u32;

        // Connection size (bits 0-15)
        encoded |= params.size as u32;

        // Variable flag (bit 25)
        if params.variable_size {
            encoded |= 1 << 25;
        }

        // Connection type (bits 29-30)
        encoded |= (params.connection_type as u32) << 29;

        // Priority (bits 26-27)
        encoded |= (params.priority as u32) << 26;

        encoded
    }

    /// Parses Forward Open response and updates session with connection info
    fn parse_forward_open_response(
        &self,
        session: &mut ConnectedSession,
        response: &[u8],
    ) -> crate::error::Result<()> {
        if response.len() < 2 {
            return Err(EtherNetIpError::Protocol(
                "Forward Open response too short".to_string(),
            ));
        }

        let service = response[0];
        let status = response[1];

        // Check if this is a Forward Open Reply (0xD4)
        if service != 0xD4 {
            return Err(EtherNetIpError::Protocol(format!(
                "Unexpected service in Forward Open response: 0x{service:02X}"
            )));
        }

        // Check status
        if status != 0x00 {
            let error_msg = match status {
                0x01 => "Connection failure - Resource unavailable or already exists",
                0x02 => "Invalid parameter - Connection parameters rejected",
                0x03 => "Connection timeout - PLC did not respond in time",
                0x04 => "Connection limit exceeded - Too many connections",
                0x08 => "Invalid service - Forward Open not supported",
                0x0C => "Invalid attribute - Connection parameters invalid",
                0x13 => "Path destination unknown - Target object not found",
                0x26 => "Invalid parameter value - RPI or size out of range",
                _ => &format!("Unknown status: 0x{status:02X}"),
            };
            return Err(EtherNetIpError::Protocol(format!(
                "Forward Open failed with status 0x{status:02X}: {error_msg}"
            )));
        }

        // Parse successful response
        if response.len() < 16 {
            return Err(EtherNetIpError::Protocol(
                "Forward Open response data too short".to_string(),
            ));
        }

        // CRITICAL FIX: The Forward Open response contains the actual connection IDs assigned by the PLC
        // Use the IDs returned by the PLC, not our requested ones
        let actual_o_to_t_id =
            u32::from_le_bytes([response[2], response[3], response[4], response[5]]);
        let actual_t_to_o_id =
            u32::from_le_bytes([response[6], response[7], response[8], response[9]]);

        // Update session with the actual assigned connection IDs
        session.o_to_t_connection_id = actual_o_to_t_id;
        session.t_to_o_connection_id = actual_t_to_o_id;
        session.connection_id = actual_o_to_t_id; // Use O->T as the primary connection ID

        tracing::info!("[FORWARD OPEN] Success!");
        tracing::debug!(
            "O->T Connection ID: 0x{:08X} (PLC assigned)",
            session.o_to_t_connection_id
        );
        tracing::debug!(
            "T->O Connection ID: 0x{:08X} (PLC assigned)",
            session.t_to_o_connection_id
        );
        tracing::debug!(
            "Using Connection ID: 0x{:08X} for messaging",
            session.connection_id
        );

        Ok(())
    }

    /// Writes a string using connected explicit messaging
    pub async fn write_string_connected(
        &mut self,
        tag_name: &str,
        value: &str,
    ) -> crate::error::Result<()> {
        let session_name = format!("string_write_{tag_name}");
        let mut sessions = self.connected_sessions.lock().await;

        if !sessions.contains_key(&session_name) {
            drop(sessions); // Release the lock before calling establish_connected_session
            self.establish_connected_session(&session_name).await?;
            sessions = self.connected_sessions.lock().await;
        }

        let session = sessions.get(&session_name).cloned().ok_or_else(|| {
            crate::error::EtherNetIpError::Connection(format!(
                "connected session '{session_name}' was not available after establishment"
            ))
        })?;
        let request = self.build_connected_string_write_request(tag_name, value, &session)?;

        drop(sessions); // Release the lock before sending the request
        let response = self
            .send_connected_cip_request(&request, &session, &session_name)
            .await?;

        // Check if write was successful
        if response.len() >= 2 {
            let status = response[1];
            if status == 0x00 {
                Ok(())
            } else {
                let error_msg = self.get_cip_error_message(status);
                Err(EtherNetIpError::Protocol(format!(
                    "CIP Error 0x{status:02X}: {error_msg}"
                )))
            }
        } else {
            Err(EtherNetIpError::Protocol(
                "Invalid connected string write response".to_string(),
            ))
        }
    }

    /// Builds a string write request for connected messaging
    fn build_connected_string_write_request(
        &self,
        tag_name: &str,
        value: &str,
        _session: &ConnectedSession,
    ) -> crate::error::Result<Vec<u8>> {
        let mut request = Vec::new();

        // For connected messaging, use direct CIP Write service
        // The connection is already established, so we can send the request directly

        // CIP Write Service Code
        request.push(0x4D);

        // Tag path - use simple ANSI format for connected messaging
        let tag_bytes = tag_name.as_bytes();
        let path_size_words = (2 + tag_bytes.len()).div_ceil(2); // +1 for potential padding, /2 for word count
        request.push(path_size_words as u8);

        request.push(0x91); // ANSI symbol segment
        request.push(tag_bytes.len() as u8); // Length of tag name
        request.extend_from_slice(tag_bytes);

        // Add padding byte if needed to make path even length
        if !(2 + tag_bytes.len()).is_multiple_of(2) {
            request.push(0x00);
        }

        // Data type for AB STRING
        request.extend_from_slice(&[0xCE, 0x0F]); // AB STRING data type (4046)

        // Number of elements (always 1 for a single string)
        request.extend_from_slice(&[0x01, 0x00]);

        // Build the AB STRING structure payload
        let string_bytes = value.as_bytes();
        let max_len: u16 = 82; // Standard AB STRING max length
        let current_len = string_bytes.len().min(max_len as usize) as u16;

        // STRING structure:
        // - Len (2 bytes) - number of characters used
        request.extend_from_slice(&current_len.to_le_bytes());

        // - MaxLen (2 bytes) - maximum characters allowed (typically 82)
        request.extend_from_slice(&max_len.to_le_bytes());

        // - Data[MaxLen] (82 bytes) - the character array, zero-padded
        let mut data_array = vec![0u8; max_len as usize];
        data_array[..current_len as usize].copy_from_slice(&string_bytes[..current_len as usize]);
        request.extend_from_slice(&data_array);

        tracing::trace!(
            "Built connected string write request ({} bytes) for '{}' = '{}' (len={}, maxlen={})",
            request.len(),
            tag_name,
            value,
            current_len,
            max_len
        );
        tracing::trace!("Request: {:02X?}", request);

        Ok(request)
    }

    /// Sends a CIP request using connected messaging
    async fn send_connected_cip_request(
        &mut self,
        cip_request: &[u8],
        session: &ConnectedSession,
        session_name: &str,
    ) -> crate::error::Result<Vec<u8>> {
        tracing::debug!(
            "[CONNECTED] Sending connected CIP request ({} bytes) using T->O connection ID 0x{:08X}",
            cip_request.len(),
            session.t_to_o_connection_id
        );

        let mut packet = BytesMut::new();
        EncapsulationHeader::new(SEND_RR_DATA, 0, self.session_handle).encode(&mut packet);

        // CPF (Common Packet Format) data starts here
        let cpf_start = packet.len();

        // Interface handle (4 bytes)
        packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        // Timeout (2 bytes) - 5 seconds
        packet.extend_from_slice(&[0x05, 0x00]);

        // Item count (2 bytes) - 2 items: Address + Data
        packet.extend_from_slice(&[0x02, 0x00]);

        // Item 1: Connected Address Item (specifies which connection to use)
        packet.extend_from_slice(&[0xA1, 0x00]); // Type: Connected Address Item (0x00A1)
        packet.extend_from_slice(&[0x04, 0x00]); // Length: 4 bytes
        // Use T->O connection ID (Target to Originator) for addressing
        packet.extend_from_slice(&session.t_to_o_connection_id.to_le_bytes());

        // Item 2: Connected Data Item (contains the CIP request + sequence)
        packet.extend_from_slice(&[0xB1, 0x00]); // Type: Connected Data Item (0x00B1)
        let data_length = cip_request.len() + 2; // +2 for sequence count
        packet.extend_from_slice(&(data_length as u16).to_le_bytes()); // Length

        // Clone session_name and session before acquiring the lock
        let session_name_clone = session_name.to_string();
        let _session_clone = session.clone();

        // Get the current session mutably to increment sequence counter
        let mut sessions = self.connected_sessions.lock().await;
        let current_sequence = if let Some(session_mut) = sessions.get_mut(&session_name_clone) {
            session_mut.sequence_count += 1;
            session_mut.sequence_count
        } else {
            1 // Fallback if session not found
        };

        // Drop the lock before sending the request
        drop(sessions);

        // Sequence count (2 bytes) - incremental counter for this connection
        packet.extend_from_slice(&current_sequence.to_le_bytes());

        // CIP request data
        packet.extend_from_slice(cip_request);

        // Update packet length in header (total CPF data size)
        let cpf_length = packet.len() - cpf_start;
        packet[2..4].copy_from_slice(&(cpf_length as u16).to_le_bytes());

        tracing::trace!(
            "[CONNECTED] Sending packet ({} bytes) with sequence {}",
            packet.len(),
            current_sequence
        );

        // Send packet
        let mut stream = self.stream.lock().await;
        stream
            .write_all(&packet)
            .await
            .map_err(EtherNetIpError::Io)?;

        // Read response header
        let mut header = [0u8; 24];
        stream
            .read_exact(&mut header)
            .await
            .map_err(EtherNetIpError::Io)?;

        // Check EtherNet/IP command status
        let mut header_bytes = &header[..];
        let response_header = EncapsulationHeader::decode(&mut header_bytes)?;
        if response_header.status != 0 {
            return Err(EtherNetIpError::Protocol(format!(
                "Connected message failed with status: 0x{:08X}",
                response_header.status
            )));
        }

        // Read response data
        let response_length = response_header.length as usize;
        let mut response_data = vec![0u8; response_length];
        stream
            .read_exact(&mut response_data)
            .await
            .map_err(EtherNetIpError::Io)?;

        let mut last_activity = self.last_activity.lock().await;
        *last_activity = Instant::now();

        tracing::trace!(
            "[CONNECTED] Received response ({} bytes)",
            response_data.len()
        );

        // Extract connected CIP response
        self.extract_connected_cip_from_response(&response_data)
    }

    /// Extracts CIP data from connected response
    fn extract_connected_cip_from_response(
        &self,
        response: &[u8],
    ) -> crate::error::Result<Vec<u8>> {
        tracing::trace!(
            "[CONNECTED] Extracting CIP from connected response ({} bytes): {:02X?}",
            response.len(),
            response
        );

        if response.len() < 12 {
            return Err(EtherNetIpError::Protocol(
                "Connected response too short for CPF header".to_string(),
            ));
        }

        // Parse CPF (Common Packet Format) structure
        // [0-3]: Interface handle
        // [4-5]: Timeout
        // [6-7]: Item count
        let item_count = u16::from_le_bytes([response[6], response[7]]) as usize;
        tracing::trace!("[CONNECTED] CPF item count: {}", item_count);

        let mut pos = 8; // Start after CPF header

        // Look for Connected Data Item (0x00B1)
        for _i in 0..item_count {
            if pos + 4 > response.len() {
                return Err(EtherNetIpError::Protocol(
                    "Response truncated while parsing items".to_string(),
                ));
            }

            let item_type = u16::from_le_bytes([response[pos], response[pos + 1]]);
            let item_length = u16::from_le_bytes([response[pos + 2], response[pos + 3]]) as usize;
            pos += 4; // Skip item header

            tracing::trace!(
                "[CONNECTED] Found item: type=0x{:04X}, length={}",
                item_type,
                item_length
            );

            if pos
                .checked_add(item_length)
                .is_none_or(|end| end > response.len())
            {
                return Err(EtherNetIpError::Protocol(
                    "Connected data item truncated".to_string(),
                ));
            }

            if item_type == 0x00B1 {
                // Connected Data Item
                // Connected Data Item contains [sequence_count(2)][cip_data]
                if item_length < 2 {
                    return Err(EtherNetIpError::Protocol(
                        "Connected data item too short for sequence".to_string(),
                    ));
                }

                let sequence_count = u16::from_le_bytes([response[pos], response[pos + 1]]);
                tracing::trace!("[CONNECTED] Sequence count: {}", sequence_count);

                // Extract CIP data (skip 2-byte sequence count)
                let cip_data = response[pos + 2..pos + item_length].to_vec();
                tracing::trace!(
                    "[CONNECTED] Extracted CIP data ({} bytes): {:02X?}",
                    cip_data.len(),
                    cip_data
                );

                return Ok(cip_data);
            } else {
                // Skip this item's data
                pos += item_length;
            }
        }

        Err(EtherNetIpError::Protocol(
            "Connected Data Item (0x00B1) not found in response".to_string(),
        ))
    }

    /// Closes a specific connected session
    async fn close_connected_session(&mut self, session_name: &str) -> crate::error::Result<()> {
        if let Some(session) = self.connected_sessions.lock().await.get(session_name) {
            let session = session.clone(); // Clone to avoid borrowing issues

            // Build Forward Close request
            let forward_close_request = self.build_forward_close_request(&session)?;

            // Send Forward Close request
            let _response = self.send_cip_request(&forward_close_request).await?;

            tracing::info!("[CONNECTED] Session '{}' closed successfully", session_name);
        }

        // Remove session from our tracking
        let mut sessions = self.connected_sessions.lock().await;
        sessions.remove(session_name);

        Ok(())
    }

    /// Builds a Forward Close CIP request for terminating connected sessions
    fn build_forward_close_request(
        &self,
        session: &ConnectedSession,
    ) -> crate::error::Result<Vec<u8>> {
        let mut request = Vec::with_capacity(21);

        // CIP Forward Close Service (0x4E)
        request.push(0x4E);

        // Request path length (Connection Manager object)
        request.push(0x02); // 2 words

        // Class ID: Connection Manager (0x06)
        request.push(0x20); // Logical Class segment
        request.push(0x06);

        // Instance ID: Connection Manager instance (0x01)
        request.push(0x24); // Logical Instance segment
        request.push(0x01);

        // Forward Close parameters

        // Connection Timeout Ticks (1 byte) + Timeout multiplier (1 byte)
        request.push(0x0A); // Timeout ticks (10)
        request.push(session.timeout_multiplier);

        // Connection Serial Number (2 bytes, little-endian)
        request.extend_from_slice(&session.connection_serial.to_le_bytes());

        // Originator Vendor ID (2 bytes, little-endian)
        request.extend_from_slice(&session.originator_vendor_id.to_le_bytes());

        // Originator Serial Number (4 bytes, little-endian)
        request.extend_from_slice(&session.originator_serial.to_le_bytes());

        // Connection Path Size (1 byte)
        request.push(0x02); // 2 words for Message Router path

        // Connection Path - Target the Message Router
        request.push(0x20); // Logical Class segment
        request.push(0x02); // Message Router class (0x02)
        request.push(0x24); // Logical Instance segment
        request.push(0x01); // Message Router instance (0x01)

        Ok(request)
    }

    /// Closes all connected sessions (called during disconnect)
    pub(super) async fn close_all_connected_sessions(&mut self) -> crate::error::Result<()> {
        let session_names: Vec<String> = self
            .connected_sessions
            .lock()
            .await
            .keys()
            .cloned()
            .collect();

        for session_name in session_names {
            let _ = self.close_connected_session(&session_name).await; // Ignore errors during cleanup
        }

        Ok(())
    }

    /// Writes a string using unconnected explicit messaging with proper AB STRING format
    ///
    /// This method uses standard unconnected messaging instead of connected messaging
    /// and implements the proper Allen-Bradley STRING structure as described in the
    /// provided information about `Len`, `MaxLen`, and `Data[82]` format.
    pub async fn write_string_unconnected(
        &mut self,
        tag_name: &str,
        value: &str,
    ) -> crate::error::Result<()> {
        tracing::debug!(
            "[UNCONNECTED] Writing string '{}' to tag '{}' using unconnected messaging",
            value,
            tag_name
        );

        self.validate_session().await?;

        let string_bytes = value.as_bytes();
        if string_bytes.len() > 82 {
            return Err(EtherNetIpError::Protocol(
                "String too long for Allen-Bradley STRING (max 82 chars)".to_string(),
            ));
        }

        // Build the CIP request with proper AB STRING structure
        let mut cip_request = Vec::new();

        // Service: Write Tag Service (0x4D)
        cip_request.push(0x4D);

        // Request Path Size (in words)
        let tag_bytes = tag_name.as_bytes();
        let path_len = if tag_bytes.len().is_multiple_of(2) {
            tag_bytes.len() + 2
        } else {
            tag_bytes.len() + 3
        } / 2;
        cip_request.push(path_len as u8);

        // Request Path: ANSI Extended Symbol Segment for tag name
        cip_request.push(0x91); // ANSI Extended Symbol Segment
        cip_request.push(tag_bytes.len() as u8); // Tag name length
        cip_request.extend_from_slice(tag_bytes); // Tag name

        // Pad to even length if necessary
        if !tag_bytes.len().is_multiple_of(2) {
            cip_request.push(0x00);
        }

        // For write operations, we don't include data type and element count
        // The PLC infers the data type from the tag definition

        // Build Allen-Bradley STRING structure based on what we see in read responses:
        // Looking at read response: [CE, 0F, 01, 00, 00, 00, 31, 00, ...]
        // Structure appears to be:
        // - Some header/identifier (2 bytes): 0xCE, 0x0F
        // - Length (2 bytes): number of characters
        // - MaxLength or padding (2 bytes): 0x00, 0x00
        // - Data array (variable length, null terminated)

        let _current_len = string_bytes.len().min(82) as u16;

        // Build the correct Allen-Bradley STRING structure to match what the PLC expects
        // Analysis of read response: [CE, 0F, 01, 00, 00, 00, 31, 00, 00, 00, ...]
        // Structure appears to be:
        // - Header (2 bytes): 0xCE, 0x0F (Allen-Bradley STRING identifier)
        // - Length (4 bytes, DINT): Number of characters currently used
        // - Data (variable): Character data followed by padding to complete the structure

        let current_len = string_bytes.len().min(82) as u32;

        // AB STRING header/identifier - this appears to be required
        cip_request.extend_from_slice(&[0xCE, 0x0F]);

        // Length (4 bytes) - number of characters used as DINT
        cip_request.extend_from_slice(&current_len.to_le_bytes());

        // Data bytes - the actual string content
        cip_request.extend_from_slice(&string_bytes[..current_len as usize]);

        // Add padding if the total structure needs to be a specific size
        // Based on reads, it looks like there might be additional padding after the data

        tracing::trace!(
            "Built Allen-Bradley STRING write request ({} bytes) for '{}' = '{}' (len={})",
            cip_request.len(),
            tag_name,
            value,
            current_len
        );
        tracing::trace!(
            "Request structure: Service=0x4D, Path={} bytes, Header=0xCE0F, Len={} (4 bytes), Data",
            path_len * 2,
            current_len
        );

        // Send the request using standard unconnected messaging
        let response = self.send_cip_request(&cip_request).await?;

        // Extract CIP response from EtherNet/IP wrapper
        let cip_response = self.extract_cip_from_response(&response)?;

        // Check if write was successful - use correct CIP response format
        if cip_response.len() >= 3 {
            let service_reply = cip_response[0]; // Should be 0xCD (0x4D + 0x80) for Write Tag reply
            let _additional_status_size = cip_response[1]; // Additional status size (usually 0)
            let status = cip_response[2]; // CIP status code at position 2

            tracing::trace!(
                "Write response - Service: 0x{:02X}, Status: 0x{:02X}",
                service_reply,
                status
            );

            if status == 0x00 {
                tracing::info!("[UNCONNECTED] String write completed successfully");
                Ok(())
            } else {
                let error_msg = self.get_cip_error_message(status);
                tracing::error!(
                    "[UNCONNECTED] String write failed: {} (0x{:02X})",
                    error_msg,
                    status
                );
                Err(EtherNetIpError::Protocol(format!(
                    "CIP Error 0x{status:02X}: {error_msg}"
                )))
            }
        } else {
            Err(EtherNetIpError::Protocol(
                "Invalid unconnected string write response - too short".to_string(),
            ))
        }
    }

    /// Write a string value to a PLC tag using unconnected messaging
    ///
    /// # Arguments
    ///
    /// * `tag_name` - The name of the tag to write to
    /// * `value` - The string value to write (max 82 characters)
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the write was successful
    /// * `Err(EtherNetIpError)` if the write failed
    ///
    /// # Errors
    ///
    /// * `StringTooLong` - If the string is longer than 82 characters
    /// * `InvalidString` - If the string contains invalid characters
    /// * `TagNotFound` - If the tag doesn't exist
    /// * `WriteError` - If the write operation fails
    pub async fn write_string(&mut self, tag_name: &str, value: &str) -> crate::error::Result<()> {
        // Validate string length
        if value.len() > 82 {
            return Err(crate::error::EtherNetIpError::StringTooLong {
                max_length: 82,
                actual_length: value.len(),
            });
        }

        // Validate string content (ASCII only)
        if !value.is_ascii() {
            return Err(crate::error::EtherNetIpError::InvalidString {
                reason: "String contains non-ASCII characters".to_string(),
            });
        }

        // Build the string write request
        let request = self.build_string_write_request(tag_name, value)?;

        // Send the request and get the response
        let response = self.send_cip_request(&request).await?;

        // Parse the response
        let cip_response = self.extract_cip_from_response(&response)?;

        // Check for errors in the response
        if cip_response.len() < 2 {
            return Err(crate::error::EtherNetIpError::InvalidResponse {
                reason: "Response too short".to_string(),
            });
        }

        let status = cip_response[0];
        if status != 0 {
            return Err(crate::error::EtherNetIpError::WriteError {
                status,
                message: self.get_cip_error_message(status),
            });
        }

        Ok(())
    }

    /// Build a string write request packet
    fn build_string_write_request(
        &self,
        tag_name: &str,
        value: &str,
    ) -> crate::error::Result<Vec<u8>> {
        let mut request = Vec::new();

        // CIP Write Service (0x4D)
        request.push(0x4D);

        // Tag path
        let tag_path = self.build_tag_path(tag_name);
        request.extend_from_slice(&tag_path);

        // AB STRING data structure
        request.extend_from_slice(&(value.len() as u16).to_le_bytes()); // Len
        request.extend_from_slice(&82u16.to_le_bytes()); // MaxLen

        // Data[82] with padding
        let mut data = [0u8; 82];
        let bytes = value.as_bytes();
        data[..bytes.len()].copy_from_slice(bytes);
        request.extend_from_slice(&data);

        Ok(request)
    }
}
