use super::EipClient;
use crate::batch::{BatchConfig, BatchError, BatchOperation, BatchResult};
use crate::protocol::values;
use crate::types::PlcValue;
use tokio::time::Instant;

#[derive(Clone)]
struct PreparedBatchOperation {
    operation: BatchOperation,
    service_request: Vec<u8>,
}

impl EipClient {
    // =========================================================================
    // BATCH OPERATIONS IMPLEMENTATION
    // =========================================================================

    /// Executes a batch of read and write operations
    ///
    /// This is the main entry point for batch operations. It takes a slice of
    /// `BatchOperation` items and executes them efficiently by grouping them
    /// into optimal CIP packets based on the current `BatchConfig`.
    ///
    /// # Arguments
    ///
    /// * `operations` - A slice of operations to execute
    ///
    /// # Returns
    ///
    /// A vector of [`BatchResult`] items, one per executed operation.
    ///
    /// When `optimize_packet_packing` is enabled, operations may be regrouped
    /// by type for execution, so result order is not guaranteed to match the
    /// original mixed-operation input order. Use [`BatchResult::operation`] to
    /// correlate each result.
    ///
    /// # Performance
    ///
    /// Batch execution primarily reduces round trips by combining multiple
    /// operations into fewer requests. Observed throughput varies significantly
    /// between simulator and real hardware, and also depends on packet sizing,
    /// controller model, route path, and tag mix.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rust_ethernet_ip::{EipClient, BatchOperation, PlcValue};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ///     let mut client = EipClient::connect("192.168.1.100:44818").await?;
    ///
    ///     let operations = vec![
    ///         BatchOperation::Read { tag_name: "Motor1_Speed".to_string() },
    ///         BatchOperation::Read { tag_name: "Motor2_Speed".to_string() },
    ///         BatchOperation::Write {
    ///             tag_name: "SetPoint".to_string(),
    ///             value: PlcValue::Dint(1500)
    ///         },
    ///     ];
    ///
    ///     let results = client.execute_batch(&operations).await?;
    ///
    ///     for result in results {
    ///         match result.result {
    ///             Ok(Some(value)) => println!("Read value: {:?}", value),
    ///             Ok(None) => println!("Write successful"),
    ///             Err(e) => println!("Operation failed: {}", e),
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn execute_batch(
        &mut self,
        operations: &[BatchOperation],
    ) -> crate::error::Result<Vec<BatchResult>> {
        if operations.is_empty() {
            return Ok(Vec::new());
        }

        let start_time = Instant::now();
        tracing::debug!(
            "[BATCH] Starting batch execution with {} operations",
            operations.len()
        );

        // Group operations based on configuration
        let operation_groups = if self.batch_config.optimize_packet_packing {
            self.optimize_operation_groups(operations).await?
        } else {
            self.sequential_operation_groups(operations).await?
        };

        let mut all_results = Vec::with_capacity(operations.len());

        // Execute each group
        for (group_index, group) in operation_groups.iter().enumerate() {
            tracing::debug!(
                "[BATCH] Processing group {} with {} operations",
                group_index + 1,
                group.len()
            );

            match self.execute_operation_group(group).await {
                Ok(mut group_results) => {
                    all_results.append(&mut group_results);
                }
                Err(e) => {
                    if !self.batch_config.continue_on_error {
                        return Err(e);
                    }

                    // Create error results for this group
                    for op in group {
                        let error_result = BatchResult {
                            operation: op.operation.clone(),
                            result: Err(BatchError::NetworkError(e.to_string())),
                            execution_time_us: 0,
                        };
                        all_results.push(error_result);
                    }
                }
            }
        }

        let total_time = start_time.elapsed();
        tracing::info!(
            "[BATCH] Completed batch execution in {:?} - {} operations processed",
            total_time,
            all_results.len()
        );

        Ok(all_results)
    }

    /// Reads multiple tags in a single batch operation
    ///
    /// This is a convenience method for read-only batch operations.
    /// It's optimized for reading many tags at once.
    ///
    /// # Arguments
    ///
    /// * `tag_names` - A slice of tag names to read
    ///
    /// # Returns
    ///
    /// A vector of tuples containing `(tag_name, result)` pairs
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rust_ethernet_ip::EipClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ///     let mut client = EipClient::connect("192.168.1.100:44818").await?;
    ///
    ///     let tags = ["Motor1_Speed", "Motor2_Speed", "Temperature", "Pressure"];
    ///     let results = client.read_tags_batch(&tags).await?;
    ///
    ///     for (tag_name, result) in results {
    ///         match result {
    ///             Ok(value) => println!("{}: {:?}", tag_name, value),
    ///             Err(e) => println!("{}: Error - {}", tag_name, e),
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn read_tags_batch(
        &mut self,
        tag_names: &[&str],
    ) -> crate::error::Result<Vec<(String, std::result::Result<PlcValue, BatchError>)>> {
        let operations: Vec<BatchOperation> = tag_names
            .iter()
            .map(|&name| BatchOperation::Read {
                tag_name: name.to_string(),
            })
            .collect();

        let results = self.execute_batch(&operations).await?;

        Ok(results
            .into_iter()
            .map(|result| {
                let tag_name = match &result.operation {
                    BatchOperation::Read { tag_name } => tag_name.clone(),
                    BatchOperation::Write { tag_name, .. } => {
                        return (
                            tag_name.clone(),
                            Err(BatchError::Other(
                                "Internal batch error: write result returned from read-only helper"
                                    .to_string(),
                            )),
                        );
                    }
                };

                let value_result = match result.result {
                    Ok(Some(value)) => Ok(value),
                    Ok(None) => Err(BatchError::Other(
                        "Unexpected None result for read operation".to_string(),
                    )),
                    Err(e) => Err(e),
                };

                (tag_name, value_result)
            })
            .collect())
    }

    /// Writes multiple tag values in a single batch operation
    ///
    /// This is a convenience method for write-only batch operations.
    /// It's optimized for writing many values at once.
    ///
    /// # Arguments
    ///
    /// * `tag_values` - A slice of `(tag_name, value)` tuples to write
    ///
    /// # Returns
    ///
    /// A vector of tuples containing `(tag_name, result)` pairs
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rust_ethernet_ip::{EipClient, PlcValue};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ///     let mut client = EipClient::connect("192.168.1.100:44818").await?;
    ///
    ///     let writes = vec![
    ///         ("SetPoint1", PlcValue::Bool(true)),
    ///         ("SetPoint2", PlcValue::Dint(2000)),
    ///         ("EnableFlag", PlcValue::Bool(true)),
    ///     ];
    ///
    ///     let results = client.write_tags_batch(&writes).await?;
    ///
    ///     for (tag_name, result) in results {
    ///         match result {
    ///             Ok(_) => println!("{}: Write successful", tag_name),
    ///             Err(e) => println!("{}: Write failed - {}", tag_name, e),
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn write_tags_batch(
        &mut self,
        tag_values: &[(&str, PlcValue)],
    ) -> crate::error::Result<Vec<(String, std::result::Result<(), BatchError>)>> {
        let operations: Vec<BatchOperation> = tag_values
            .iter()
            .map(|(name, value)| BatchOperation::Write {
                tag_name: name.to_string(),
                value: value.clone(),
            })
            .collect();

        let results = self.execute_batch(&operations).await?;

        Ok(results
            .into_iter()
            .map(|result| {
                let tag_name = match &result.operation {
                    BatchOperation::Write { tag_name, .. } => tag_name.clone(),
                    BatchOperation::Read { tag_name } => {
                        return (
                            tag_name.clone(),
                            Err(BatchError::Other(
                                "Internal batch error: read result returned from write-only helper"
                                    .to_string(),
                            )),
                        );
                    }
                };

                let write_result = match result.result {
                    Ok(None) => Ok(()),
                    Ok(Some(_)) => Err(BatchError::Other(
                        "Unexpected value result for write operation".to_string(),
                    )),
                    Err(e) => Err(e),
                };

                (tag_name, write_result)
            })
            .collect())
    }

    /// Configures batch operation settings
    ///
    /// This method allows fine-tuning of batch operation behavior,
    /// including performance optimizations and error handling.
    ///
    /// # Arguments
    ///
    /// * `config` - The new batch configuration to use
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rust_ethernet_ip::{EipClient, BatchConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ///     let mut client = EipClient::connect("192.168.1.100:44818").await?;
    ///
    ///     let config = BatchConfig {
    ///         max_operations_per_packet: 50,
    ///         max_packet_size: 1500,
    ///         packet_timeout_ms: 5000,
    ///         continue_on_error: false,
    ///         optimize_packet_packing: true,
    ///     };
    ///
    ///     client.configure_batch_operations(config);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn configure_batch_operations(&mut self, config: BatchConfig) {
        self.batch_config = config;
        tracing::debug!(
            "[BATCH] Updated batch configuration: max_ops={}, max_size={}, timeout={}ms",
            self.batch_config.max_operations_per_packet,
            self.batch_config.max_packet_size,
            self.batch_config.packet_timeout_ms
        );
    }

    /// Gets current batch operation configuration
    pub fn get_batch_config(&self) -> &BatchConfig {
        &self.batch_config
    }

    // =========================================================================
    // INTERNAL BATCH OPERATION HELPERS
    // =========================================================================

    /// Groups operations optimally for batch processing
    async fn optimize_operation_groups(
        &mut self,
        operations: &[BatchOperation],
    ) -> crate::error::Result<Vec<Vec<PreparedBatchOperation>>> {
        let mut reads = Vec::new();
        let mut writes = Vec::new();

        // Separate reads and writes
        for op in operations {
            match op {
                BatchOperation::Read { .. } => reads.push(op.clone()),
                BatchOperation::Write { .. } => writes.push(op.clone()),
            }
        }

        let mut groups = self.prepare_and_pack_operations(&reads).await?;
        groups.extend(self.prepare_and_pack_operations(&writes).await?);

        Ok(groups)
    }

    /// Groups operations sequentially (preserves order)
    async fn sequential_operation_groups(
        &mut self,
        operations: &[BatchOperation],
    ) -> crate::error::Result<Vec<Vec<PreparedBatchOperation>>> {
        self.prepare_and_pack_operations(operations).await
    }

    async fn prepare_and_pack_operations(
        &mut self,
        operations: &[BatchOperation],
    ) -> crate::error::Result<Vec<Vec<PreparedBatchOperation>>> {
        let mut prepared = Vec::with_capacity(operations.len());
        for operation in operations {
            prepared.push(PreparedBatchOperation {
                operation: operation.clone(),
                service_request: self.build_batch_service_request(operation).await?,
            });
        }

        Ok(self.pack_prepared_operations(prepared))
    }

    fn pack_prepared_operations(
        &self,
        operations: Vec<PreparedBatchOperation>,
    ) -> Vec<Vec<PreparedBatchOperation>> {
        let max_operations = self.batch_config.max_operations_per_packet.max(1);
        let max_packet_size = self.batch_config.max_packet_size;
        let mut groups = Vec::new();
        let mut current_group = Vec::new();

        for operation in operations {
            let exceeds_operation_count = current_group.len() >= max_operations;
            let exceeds_packet_size = !current_group.is_empty()
                && max_packet_size > 0
                && self.group_wire_len_with_candidate(&current_group, &operation) > max_packet_size;

            if exceeds_operation_count || exceeds_packet_size {
                groups.push(std::mem::take(&mut current_group));
            }

            current_group.push(operation);
        }

        if !current_group.is_empty() {
            groups.push(current_group);
        }

        groups
    }

    fn group_wire_len_with_candidate(
        &self,
        group: &[PreparedBatchOperation],
        candidate: &PreparedBatchOperation,
    ) -> usize {
        let service_bytes = self.group_service_bytes(group) + candidate.service_request.len();
        let operation_count = group.len() + 1;
        let msp_len = 8 + (operation_count * 2) + service_bytes;

        self.unconnected_send_len_for_embedded(msp_len)
    }

    #[cfg(test)]
    fn group_wire_len(&self, group: &[PreparedBatchOperation]) -> usize {
        let msp_len = 8 + (group.len() * 2) + self.group_service_bytes(group);
        self.unconnected_send_len_for_embedded(msp_len)
    }

    fn group_service_bytes(&self, group: &[PreparedBatchOperation]) -> usize {
        group
            .iter()
            .map(|operation| operation.service_request.len())
            .sum()
    }

    fn unconnected_send_len_for_embedded(&self, embedded_len: usize) -> usize {
        let route_path_len = self
            .route_path_snapshot()
            .map(|route_path| route_path.to_cip_bytes().len())
            .unwrap_or(0);
        let pad_len = embedded_len % 2;

        // Unconnected Send request path/timeout/message-length fields (10 bytes)
        // plus optional pad, route-size/reserved fields (2 bytes), and route path.
        12 + embedded_len + pad_len + route_path_len
    }

    /// Executes a single group of operations as a CIP Multiple Service Packet
    async fn execute_operation_group(
        &mut self,
        operations: &[PreparedBatchOperation],
    ) -> crate::error::Result<Vec<BatchResult>> {
        let start_time = Instant::now();
        let mut results = Vec::with_capacity(operations.len());

        // Build Multiple Service Packet request
        let cip_request = self.build_multiple_service_packet(operations)?;

        // Send request and get response
        let response = self.send_cip_request(&cip_request).await?;

        // Parse response and create results
        let original_operations: Vec<BatchOperation> = operations
            .iter()
            .map(|operation| operation.operation.clone())
            .collect();
        let parsed_results =
            self.parse_multiple_service_response(&response, &original_operations)?;

        let execution_time = start_time.elapsed();

        // Create BatchResult objects
        for (i, operation) in operations.iter().enumerate() {
            let op_execution_time = execution_time.as_micros() as u64 / operations.len() as u64;

            let result = if i < parsed_results.len() {
                match &parsed_results[i] {
                    Ok(value) => Ok(value.clone()),
                    Err(e) => Err(e.clone()),
                }
            } else {
                Err(BatchError::Other(
                    "Missing result from response".to_string(),
                ))
            };

            results.push(BatchResult {
                operation: operation.operation.clone(),
                result,
                execution_time_us: op_execution_time,
            });
        }

        Ok(results)
    }

    /// Builds a CIP Multiple Service Packet request
    fn build_multiple_service_packet(
        &self,
        operations: &[PreparedBatchOperation],
    ) -> crate::error::Result<Vec<u8>> {
        let mut packet = Vec::with_capacity(8 + (operations.len() * 2));

        // Multiple Service Packet service code
        packet.push(0x0A);

        // Request path (2 bytes for class 0x02, instance 1)
        packet.push(0x02); // Path size in words
        packet.push(0x20); // Class segment
        packet.push(0x02); // Class 0x02 (Message Router)
        packet.push(0x24); // Instance segment
        packet.push(0x01); // Instance 1

        // Number of services
        packet.extend_from_slice(&(operations.len() as u16).to_le_bytes());

        // Calculate offset table
        let mut service_requests = Vec::with_capacity(operations.len());
        let mut current_offset = 2 + (operations.len() * 2); // Start after offset table

        for operation in operations {
            service_requests.push(operation.service_request.clone());
        }

        // Add offset table
        for service_request in &service_requests {
            packet.extend_from_slice(&(current_offset as u16).to_le_bytes());
            current_offset += service_request.len();
        }

        // Add service requests
        for service_request in service_requests {
            packet.extend_from_slice(&service_request);
        }

        tracing::trace!(
            "[BATCH] Built Multiple Service Packet ({} bytes, {} services)",
            packet.len(),
            operations.len()
        );

        Ok(packet)
    }

    async fn build_batch_service_request(
        &mut self,
        operation: &BatchOperation,
    ) -> crate::error::Result<Vec<u8>> {
        match operation {
            BatchOperation::Read { tag_name } => {
                if let Some((base_name, index)) = self.parse_array_element_access(tag_name)
                    && self.detect_bool_array_path(&base_name).await?
                {
                    return Ok(self.build_read_array_request(&base_name, index / 32, 1));
                }

                self.build_read_request(tag_name)
            }
            BatchOperation::Write { tag_name, value } => {
                if let PlcValue::Bool(bit_value) = value
                    && let Some((base_name, index)) = self.parse_array_element_access(tag_name)
                    && self.detect_bool_array_path(&base_name).await?
                {
                    let dword_index = index / 32;
                    let bit_index = index % 32;
                    let response = self
                        .send_cip_request(&self.build_read_array_request(
                            &base_name,
                            dword_index,
                            1,
                        ))
                        .await?;
                    let cip_data = self.extract_cip_from_response(&response)?;
                    let mut dword = self.parse_bool_array_dword_response(&cip_data)?;
                    if *bit_value {
                        dword |= 1u32 << bit_index;
                    } else {
                        dword &= !(1u32 << bit_index);
                    }

                    return self.build_write_array_request_with_index(
                        &base_name,
                        dword_index,
                        1,
                        values::BOOL_ARRAY_DWORD,
                        &dword.to_le_bytes(),
                    );
                }

                self.build_write_request(tag_name, value)
            }
        }
    }

    /// Parses a Multiple Service Packet response
    fn parse_multiple_service_response(
        &self,
        response: &[u8],
        operations: &[BatchOperation],
    ) -> crate::error::Result<Vec<std::result::Result<Option<PlcValue>, BatchError>>> {
        if response.len() < 6 {
            return Err(crate::error::EtherNetIpError::Protocol(
                "Response too short for Multiple Service Packet".to_string(),
            ));
        }

        let mut results = Vec::new();

        tracing::trace!(
            "Raw Multiple Service Response ({} bytes): {:02X?}",
            response.len(),
            response
        );

        // First, extract the CIP data from the EtherNet/IP response
        let cip_data = match self.extract_cip_from_response(response) {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Failed to extract CIP data: {}", e);
                return Err(e);
            }
        };

        tracing::trace!(
            "Extracted CIP data ({} bytes): {:02X?}",
            cip_data.len(),
            cip_data
        );

        if cip_data.len() < 6 {
            return Err(crate::error::EtherNetIpError::Protocol(
                "CIP data too short for Multiple Service Response".to_string(),
            ));
        }

        // Parse Multiple Service Response header from CIP data:
        // [0] = Service Code (0x8A)
        // [1] = Reserved (0x00)
        // [2] = General Status (0x00 for success)
        // [3] = Additional Status Size (0x00)
        // [4-5] = Number of replies (little endian)

        let service_code = cip_data[0];
        let general_status = cip_data[2];
        let num_replies = u16::from_le_bytes([cip_data[4], cip_data[5]]) as usize;

        tracing::debug!(
            "Multiple Service Response: service=0x{:02X}, status=0x{:02X}, replies={}",
            service_code,
            general_status,
            num_replies
        );

        if general_status != 0x00 {
            return Err(crate::error::EtherNetIpError::Protocol(
                self.describe_multiple_service_error(general_status, operations),
            ));
        }

        if num_replies != operations.len() {
            return Err(crate::error::EtherNetIpError::Protocol(format!(
                "Reply count mismatch: expected {}, got {}",
                operations.len(),
                num_replies
            )));
        }

        // Read reply offsets (each is 2 bytes, little endian)
        let mut reply_offsets = Vec::new();
        let mut offset = 6; // Skip header

        for _i in 0..num_replies {
            if offset + 2 > cip_data.len() {
                return Err(crate::error::EtherNetIpError::Protocol(
                    "CIP data too short for reply offsets".to_string(),
                ));
            }
            let reply_offset =
                u16::from_le_bytes([cip_data[offset], cip_data[offset + 1]]) as usize;
            reply_offsets.push(reply_offset);
            offset += 2;
        }

        tracing::trace!("Reply offsets: {:?}", reply_offsets);

        // The reply data starts after all the offsets
        let reply_base_offset = 6 + (num_replies * 2);

        tracing::trace!("Reply base offset: {}", reply_base_offset);

        // Parse each reply
        for (i, &reply_offset) in reply_offsets.iter().enumerate() {
            // Reply offset is relative to position 4 (after service code, reserved, status, additional status size)
            let reply_start = 4 + reply_offset;

            if reply_start >= cip_data.len() {
                results.push(Err(BatchError::Other(
                    "Reply offset beyond CIP data".to_string(),
                )));
                continue;
            }

            // Calculate reply end position
            let reply_end = if i + 1 < reply_offsets.len() {
                // Not the last reply - use next reply's offset as boundary
                4 + reply_offsets[i + 1]
            } else {
                // Last reply - goes to end of CIP data
                cip_data.len()
            };

            if reply_end > cip_data.len() || reply_start >= reply_end {
                results.push(Err(BatchError::Other(
                    "Invalid reply boundaries".to_string(),
                )));
                continue;
            }

            let reply_data = &cip_data[reply_start..reply_end];

            tracing::trace!(
                "Reply {} at offset {}: start={}, end={}, len={}",
                i,
                reply_offset,
                reply_start,
                reply_end,
                reply_data.len()
            );
            tracing::trace!("Reply {} data: {:02X?}", i, reply_data);

            let result = self.parse_individual_reply(reply_data, &operations[i]);
            results.push(result);
        }

        Ok(results)
    }

    /// Parses an individual service reply within a Multiple Service Packet response
    fn parse_individual_reply(
        &self,
        reply_data: &[u8],
        operation: &BatchOperation,
    ) -> std::result::Result<Option<PlcValue>, BatchError> {
        if reply_data.len() < 4 {
            return Err(BatchError::SerializationError(
                "Reply too short".to_string(),
            ));
        }

        tracing::trace!(
            "Parsing individual reply ({} bytes): {:02X?}",
            reply_data.len(),
            reply_data
        );

        // Each individual reply in Multiple Service Response has the same format as standalone CIP response:
        // [0] = Service Code (0xCC for read response, 0xCD for write response)
        // [1] = Reserved (0x00)
        // [2] = General Status (0x00 for success)
        // [3] = Additional Status Size (0x00)
        // [4..] = Response data (for reads) or empty (for writes)

        let service_code = reply_data[0];
        let general_status = reply_data[2];

        tracing::trace!(
            "Service code: 0x{:02X}, Status: 0x{:02X}",
            service_code,
            general_status
        );

        if general_status != 0x00 {
            let error_msg = self.get_cip_error_message(general_status);
            return Err(BatchError::CipError {
                status: general_status,
                message: error_msg,
            });
        }

        match operation {
            BatchOperation::Write { .. } => {
                // Write operations return no data on success
                Ok(None)
            }
            BatchOperation::Read { .. } => {
                // Read operations return data starting at offset 4
                if reply_data.len() < 6 {
                    return Err(BatchError::SerializationError(
                        "Read reply too short for data".to_string(),
                    ));
                }

                // Parse the data directly (skip the 4-byte header)
                // Data format: [type_low, type_high, value_bytes...]
                let data = &reply_data[4..];
                tracing::trace!("Parsing data ({} bytes): {:02X?}", data.len(), data);

                if data.len() < 2 {
                    return Err(BatchError::SerializationError(
                        "Data too short for type".to_string(),
                    ));
                }

                let data_type = u16::from_le_bytes([data[0], data[1]]);
                let value_data = &data[2..];

                tracing::trace!(
                    "Data type: 0x{:04X}, Value data ({} bytes): {:02X?}",
                    data_type,
                    value_data.len(),
                    value_data
                );

                if data_type == values::BOOL_ARRAY_DWORD {
                    if value_data.len() < 4 {
                        return Err(BatchError::SerializationError(
                            "Missing packed BOOL array DWORD value".to_string(),
                        ));
                    }

                    let packed_value = u32::from_le_bytes([
                        value_data[0],
                        value_data[1],
                        value_data[2],
                        value_data[3],
                    ]);

                    if let BatchOperation::Read { tag_name } = operation
                        && let Some((_base_name, index)) = self.parse_array_element_access(tag_name)
                    {
                        let bit_index = index % 32;
                        let value = (packed_value >> bit_index) & 1 != 0;
                        tracing::trace!(
                            "Parsed packed BOOL array element '{}' from DWORD 0x{:08X} using bit {} -> {}",
                            tag_name,
                            packed_value,
                            bit_index,
                            value
                        );
                        return Ok(Some(PlcValue::Bool(value)));
                    }
                }

                values::decode_payload(data_type, value_data)
                    .map(Some)
                    .map_err(|e| BatchError::SerializationError(e.to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EipClient, PreparedBatchOperation};
    use crate::batch::{BatchConfig, BatchOperation};

    fn prepared_read(name: &str, service_len: usize) -> PreparedBatchOperation {
        PreparedBatchOperation {
            operation: BatchOperation::Read {
                tag_name: name.to_string(),
            },
            service_request: vec![0x4C; service_len],
        }
    }

    #[test]
    fn batch_packing_respects_packet_size_and_operation_count() {
        let mut client = EipClient::new_unconnected_for_testing();
        client.configure_batch_operations(BatchConfig {
            max_operations_per_packet: 4,
            max_packet_size: 80,
            ..BatchConfig::default()
        });

        let operations: Vec<_> = (0..10)
            .map(|index| prepared_read(&format!("Tag{index}"), 20))
            .collect();
        let groups = client.pack_prepared_operations(operations);

        assert!(
            groups.len() > 1,
            "expected packet-size budget to split the batch"
        );
        for group in &groups {
            assert!(
                group.len() <= 4,
                "group exceeds max_operations_per_packet: {}",
                group.len()
            );
            assert!(
                client.group_wire_len(group) <= 80,
                "group exceeds max_packet_size: {}",
                client.group_wire_len(group)
            );
        }
    }

    #[test]
    fn batch_packing_keeps_single_oversized_operation() {
        let mut client = EipClient::new_unconnected_for_testing();
        client.configure_batch_operations(BatchConfig {
            max_operations_per_packet: 20,
            max_packet_size: 32,
            ..BatchConfig::default()
        });

        let groups = client.pack_prepared_operations(vec![
            prepared_read("TooLarge", 64),
            prepared_read("Small1", 4),
            prepared_read("Small2", 4),
        ]);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].len(), 1);
        assert!(
            client.group_wire_len(&groups[0]) > 32,
            "single oversized operation should be sent alone, not dropped"
        );
        assert!(
            client.group_wire_len(&groups[1]) <= 32,
            "small trailing operations should share a packet"
        );
    }
}
