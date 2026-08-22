use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

const CMD_REGISTER_SESSION: u16 = 0x0065;
const CMD_SEND_RR_DATA: u16 = 0x006F;

const CIP_READ_TAG: u8 = 0x4C;
const CIP_WRITE_TAG: u8 = 0x4D;
const CIP_READ_TAG_FRAGMENTED: u8 = 0x52;
const CIP_WRITE_TAG_FRAGMENTED: u8 = 0x53;
const CIP_GET_ATTRIBUTE_LIST: u8 = 0x03;
const CIP_MULTIPLE_SERVICE_PACKET: u8 = 0x0A;

const CIP_REPLY_READ: u8 = 0xCC;
const CIP_REPLY_WRITE: u8 = 0xCD;
const CIP_REPLY_READ_TAG_FRAGMENTED: u8 = 0xD2;
const CIP_REPLY_WRITE_TAG_FRAGMENTED: u8 = 0xD3;
const CIP_REPLY_GET_ATTRIBUTE_LIST: u8 = 0x83;
const CIP_REPLY_MULTIPLE_SERVICE_PACKET: u8 = 0x8A;

const CIP_STATUS_SUCCESS: u8 = 0x00;
const CIP_STATUS_CONNECTION_FAILURE: u8 = 0x01;
const CIP_STATUS_PATH_SEGMENT_ERROR: u8 = 0x04;
const CIP_STATUS_PARTIAL_TRANSFER: u8 = 0x06;
const CIP_STATUS_EXTENDED_ERROR: u8 = 0xFF;
const CIP_EXT_STATUS_TYPE_MISMATCH: u16 = 0x2107;

const CIP_TYPE_BOOL: u16 = 0x00C1;
const CIP_TYPE_SINT: u16 = 0x00C2;
const CIP_TYPE_INT: u16 = 0x00C3;
const CIP_TYPE_DINT: u16 = 0x00C4;
const CIP_TYPE_LINT: u16 = 0x00C5;
const CIP_TYPE_USINT: u16 = 0x00C6;
const CIP_TYPE_UINT: u16 = 0x00C7;
const CIP_TYPE_UDINT: u16 = 0x00C8;
const CIP_TYPE_ULINT: u16 = 0x00C9;
const CIP_TYPE_REAL: u16 = 0x00CA;
const CIP_TYPE_LREAL: u16 = 0x00CB;
const CIP_TYPE_STRING: u16 = 0x00CE;
const CIP_TYPE_BOOL_ARRAY_DWORD: u16 = 0x00D3;
const CIP_TYPE_STRUCTURE: u16 = 0x02A0;
const CIP_STANDARD_STRING_HANDLE: u16 = 0x0FCE;
const CIP_LARGE_STRING_HANDLE: u16 = 0x3456;
const CIP_STANDARD_STRING_DATA_LEN: usize = 82;
const CIP_STANDARD_STRING_PAD_LEN: usize = 2;
const CIP_FRAGMENT_READ_CHUNK: usize = 240;

#[derive(Clone, Debug)]
enum TagValue {
    Bool(bool),
    Sint(i8),
    Int(i16),
    Dint(i32),
    Lint(i64),
    Usint(u8),
    Uint(u16),
    Udint(u32),
    Ulint(u64),
    Real(f32),
    Lreal(f64),
    String(String),
    CustomString {
        handle: u16,
        capacity: usize,
        value: String,
    },
    Array(Vec<TagValue>),
}

#[derive(Clone, Debug, Default)]
pub struct SimBehavior {
    /// When true, SendRRData responses are suppressed (client-side timeout path).
    pub drop_send_rr_response: bool,
    /// Suppress SendRRData responses once the global SendRRData count reaches this value.
    /// Useful to let initial connect/list-tags succeed, then inject timeout on later operations.
    pub drop_send_rr_response_after: Option<usize>,
    /// Close the current connection once the global SendRRData count reaches this value.
    pub disconnect_on_send_rr_after: Option<usize>,
    /// Corrupt the encapsulation sender_context once the global SendRRData count reaches this value.
    pub corrupt_sender_context_after: Option<usize>,
    /// Tags that should fail READ requests with CIP status 0x04.
    pub fail_read_tags: Vec<String>,
    /// Tags that should fail READ requests on specific per-tag read counts.
    pub fail_read_once_on_counts: Vec<(String, usize)>,
    /// Tags that should fail READ requests with CIP connection failure on specific read counts.
    pub fail_read_once_connection_failure_on_counts: Vec<(String, usize)>,
    /// Tags whose READ response should be the per-tag read count as a DINT.
    pub count_reads_as_dint_tags: Vec<String>,
    /// Tags that should fail WRITE requests with CIP status 0x04.
    pub fail_write_tags: Vec<String>,
}

struct SimRuntimeBehavior {
    config: SimBehavior,
    send_rr_count: AtomicUsize,
    read_counts: Mutex<HashMap<String, usize>>,
    write_counts: Mutex<HashMap<String, usize>>,
}

impl SimRuntimeBehavior {
    fn new(config: SimBehavior) -> Self {
        Self {
            config,
            send_rr_count: AtomicUsize::new(0),
            read_counts: Mutex::new(HashMap::new()),
            write_counts: Mutex::new(HashMap::new()),
        }
    }

    fn record_read(&self, tag_name: &str) -> usize {
        let mut read_counts = self.read_counts.lock().expect("read-count lock");
        let count = read_counts.entry(tag_name.to_string()).or_insert(0);
        *count += 1;
        *count
    }

    #[allow(dead_code)]
    fn read_count(&self, tag_name: &str) -> usize {
        self.read_counts
            .lock()
            .expect("read-count lock")
            .get(tag_name)
            .copied()
            .unwrap_or(0)
    }

    fn record_write(&self, tag_name: &str) -> usize {
        let mut write_counts = self.write_counts.lock().expect("write-count lock");
        let count = write_counts.entry(tag_name.to_string()).or_insert(0);
        *count += 1;
        *count
    }

    fn write_count(&self, tag_name: &str) -> usize {
        self.write_counts
            .lock()
            .expect("write-count lock")
            .get(tag_name)
            .copied()
            .unwrap_or(0)
    }

    fn read_failure_status(&self, tag_name: &str, read_count: usize) -> Option<u8> {
        if self
            .config
            .fail_read_tags
            .iter()
            .any(|configured| configured == tag_name)
            || self
                .config
                .fail_read_once_on_counts
                .iter()
                .any(|(configured, count)| configured == tag_name && *count == read_count)
        {
            return Some(CIP_STATUS_PATH_SEGMENT_ERROR);
        }

        if self
            .config
            .fail_read_once_connection_failure_on_counts
            .iter()
            .any(|(configured, count)| configured == tag_name && *count == read_count)
        {
            return Some(CIP_STATUS_CONNECTION_FAILURE);
        }

        None
    }

    fn count_reads_as_dint(&self, tag_name: &str) -> bool {
        self.config
            .count_reads_as_dint_tags
            .iter()
            .any(|configured| configured == tag_name)
    }

    fn should_fail_write(&self, tag_name: &str) -> bool {
        self.config
            .fail_write_tags
            .iter()
            .any(|configured| configured == tag_name)
    }
}

pub struct SimulatedPlc {
    pub address: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    #[allow(dead_code)]
    behavior: Arc<SimRuntimeBehavior>,
    tags: Arc<Mutex<HashMap<String, TagValue>>>,
}

impl SimulatedPlc {
    #[allow(dead_code)]
    pub async fn start() -> Self {
        Self::start_with_behavior(SimBehavior::default()).await
    }

    pub async fn start_with_behavior(behavior: SimBehavior) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind simulator");
        let address = listener.local_addr().expect("local addr");
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let behavior = Arc::new(SimRuntimeBehavior::new(behavior));
        let tags = Arc::new(Mutex::new(HashMap::from([
            ("DINT_TAG".to_string(), TagValue::Dint(1234)),
            ("BOOL_TAG".to_string(), TagValue::Bool(true)),
            ("SINT_TAG".to_string(), TagValue::Sint(-12)),
            ("INT_TAG".to_string(), TagValue::Int(-1234)),
            ("LINT_TAG".to_string(), TagValue::Lint(-123456789)),
            ("USINT_TAG".to_string(), TagValue::Usint(12)),
            ("UINT_TAG".to_string(), TagValue::Uint(1234)),
            ("UDINT_TAG".to_string(), TagValue::Udint(123456789)),
            ("ULINT_TAG".to_string(), TagValue::Ulint(123456789)),
            ("REAL_TAG".to_string(), TagValue::Real(3.0)),
            ("LREAL_TAG".to_string(), TagValue::Lreal(6.0)),
            (
                "STRING_TAG".to_string(),
                TagValue::String("Hello PLC".to_string()),
            ),
            (
                "LARGE_STRING".to_string(),
                TagValue::CustomString {
                    handle: CIP_LARGE_STRING_HANDLE,
                    capacity: 600,
                    value: "Large initial value".to_string(),
                },
            ),
            (
                "BOOL_ARRAY".to_string(),
                TagValue::Array(
                    (0..64)
                        .map(|index| TagValue::Bool(index % 3 == 0))
                        .collect(),
                ),
            ),
            (
                "UDT_ARRAY[3].BOOL_NESTED".to_string(),
                TagValue::Array(
                    (0..64)
                        .map(|index| TagValue::Bool(index % 3 == 0))
                        .collect(),
                ),
            ),
            ("UDT_ARRAY[3].DINT_MEMBER".to_string(), TagValue::Dint(30)),
            (
                "DINT_ARRAY".to_string(),
                TagValue::Array((1..=20).map(|index| TagValue::Dint(index * 10)).collect()),
            ),
            (
                "REAL_ARRAY".to_string(),
                TagValue::Array(
                    (0..20)
                        .map(|index| TagValue::Real(index as f32 + 1.5))
                        .collect(),
                ),
            ),
        ])));

        tokio::spawn(run_server(
            listener,
            Arc::clone(&tags),
            Arc::clone(&behavior),
            shutdown_rx,
        ));

        Self {
            address,
            shutdown: Some(shutdown_tx),
            behavior,
            tags,
        }
    }

    #[allow(dead_code)]
    pub fn read_count(&self, tag_name: &str) -> usize {
        self.behavior.read_count(tag_name)
    }

    #[allow(dead_code)]
    pub fn write_count(&self, tag_name: &str) -> usize {
        self.behavior.write_count(tag_name)
    }

    /// Replaces a tag with a DINT array while the simulator remains online.
    #[allow(dead_code)]
    pub fn replace_with_dint_array(&self, tag_name: &str, values: impl IntoIterator<Item = i32>) {
        self.tags.lock().expect("tag lock").insert(
            tag_name.to_string(),
            TagValue::Array(values.into_iter().map(TagValue::Dint).collect()),
        );
    }

    /// Replaces a tag with a packed BOOL array while the simulator remains online.
    #[allow(dead_code)]
    pub fn replace_with_bool_array(&self, tag_name: &str, values: impl IntoIterator<Item = bool>) {
        self.tags.lock().expect("tag lock").insert(
            tag_name.to_string(),
            TagValue::Array(values.into_iter().map(TagValue::Bool).collect()),
        );
    }

    /// Replaces a tag with a REAL array while the simulator remains online.
    #[allow(dead_code)]
    pub fn replace_with_real_array(&self, tag_name: &str, values: impl IntoIterator<Item = f32>) {
        self.tags.lock().expect("tag lock").insert(
            tag_name.to_string(),
            TagValue::Array(values.into_iter().map(TagValue::Real).collect()),
        );
    }

    /// Removes a tag so tests can model the temporary Symbol Not Found window.
    #[allow(dead_code)]
    pub fn remove_tag(&self, tag_name: &str) {
        self.tags.lock().expect("tag lock").remove(tag_name);
    }
}

impl Drop for SimulatedPlc {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
    }
}

async fn run_server(
    listener: TcpListener,
    tags: Arc<Mutex<HashMap<String, TagValue>>>,
    behavior: Arc<SimRuntimeBehavior>,
    mut shutdown_rx: oneshot::Receiver<()>,
) {
    loop {
        tokio::select! {
            _ = &mut shutdown_rx => break,
            accept_result = listener.accept() => {
                if let Ok((stream, _)) = accept_result {
                    let tags = Arc::clone(&tags);
                    let behavior = Arc::clone(&behavior);
                    tokio::spawn(async move {
                        handle_connection(stream, tags, behavior).await;
                    });
                }
            }
        }
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    tags: Arc<Mutex<HashMap<String, TagValue>>>,
    behavior: Arc<SimRuntimeBehavior>,
) {
    loop {
        let mut header = [0u8; 24];
        if stream.read_exact(&mut header).await.is_err() {
            break;
        }

        let cmd = u16::from_le_bytes([header[0], header[1]]);
        let length = u16::from_le_bytes([header[2], header[3]]) as usize;
        let session_handle = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        let mut sender_context = [0u8; 8];
        sender_context.copy_from_slice(&header[12..20]);

        let mut payload = vec![0u8; length];
        if length > 0 && stream.read_exact(&mut payload).await.is_err() {
            break;
        }

        match cmd {
            CMD_REGISTER_SESSION => {
                let response = build_register_session_response(sender_context);
                if stream.write_all(&response).await.is_err() {
                    break;
                }
            }
            CMD_SEND_RR_DATA => {
                let current = behavior.send_rr_count.fetch_add(1, Ordering::SeqCst) + 1;

                if let Some(disconnect_after) = behavior.config.disconnect_on_send_rr_after
                    && current == disconnect_after
                {
                    break;
                }

                if behavior.config.drop_send_rr_response {
                    continue;
                }
                if let Some(drop_after) = behavior.config.drop_send_rr_response_after
                    && current >= drop_after
                {
                    continue;
                }

                let cip_response = build_cip_response(&payload, &tags, &behavior);
                let response_context = if behavior
                    .config
                    .corrupt_sender_context_after
                    .is_some_and(|corrupt_after| current >= corrupt_after)
                {
                    [0xFF; 8]
                } else {
                    sender_context
                };
                let response =
                    build_send_rr_response(session_handle, response_context, &cip_response);
                if stream.write_all(&response).await.is_err() {
                    break;
                }
            }
            _ => break,
        }
    }
}

fn build_register_session_response(sender_context: [u8; 8]) -> Vec<u8> {
    let session_handle = 0x12345678_u32;
    let mut response = Vec::with_capacity(28);
    response.extend_from_slice(&CMD_REGISTER_SESSION.to_le_bytes());
    response.extend_from_slice(&4u16.to_le_bytes()); // length
    response.extend_from_slice(&session_handle.to_le_bytes());
    response.extend_from_slice(&0u32.to_le_bytes()); // status
    response.extend_from_slice(&sender_context); // context
    response.extend_from_slice(&0u32.to_le_bytes()); // options
    response.extend_from_slice(&[0u8; 4]); // protocol version + options
    response
}

fn build_send_rr_response(
    session_handle: u32,
    sender_context: [u8; 8],
    cip_response: &[u8],
) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&0u32.to_le_bytes()); // interface
    data.extend_from_slice(&0u16.to_le_bytes()); // timeout
    data.extend_from_slice(&2u16.to_le_bytes()); // item count

    data.extend_from_slice(&0u16.to_le_bytes()); // null address type
    data.extend_from_slice(&0u16.to_le_bytes()); // null address length

    data.extend_from_slice(&0x00B2u16.to_le_bytes()); // unconnected data
    data.extend_from_slice(&(cip_response.len() as u16).to_le_bytes());
    data.extend_from_slice(cip_response);

    let mut response = Vec::with_capacity(24 + data.len());
    response.extend_from_slice(&CMD_SEND_RR_DATA.to_le_bytes());
    response.extend_from_slice(&(data.len() as u16).to_le_bytes());
    response.extend_from_slice(&session_handle.to_le_bytes());
    response.extend_from_slice(&0u32.to_le_bytes()); // status
    response.extend_from_slice(&sender_context); // context
    response.extend_from_slice(&0u32.to_le_bytes()); // options
    response.extend_from_slice(&data);
    response
}

fn build_cip_response(
    payload: &[u8],
    tags: &Arc<Mutex<HashMap<String, TagValue>>>,
    behavior: &Arc<SimRuntimeBehavior>,
) -> Vec<u8> {
    let service = extract_cip_service(payload).unwrap_or(0);
    match service {
        CIP_READ_TAG => build_read_response(payload, tags, behavior),
        CIP_WRITE_TAG => handle_write(payload, tags, behavior),
        CIP_READ_TAG_FRAGMENTED => build_read_fragmented_response(payload, tags, behavior),
        CIP_WRITE_TAG_FRAGMENTED => handle_write_fragmented(payload, tags, behavior),
        CIP_GET_ATTRIBUTE_LIST => build_get_attribute_list_response(payload, tags),
        CIP_MULTIPLE_SERVICE_PACKET => build_multiple_service_response(payload, tags, behavior),
        _ => vec![CIP_REPLY_READ, 0x00, 0x01, 0x00],
    }
}

fn build_get_attribute_list_response(
    payload: &[u8],
    tags: &Arc<Mutex<HashMap<String, TagValue>>>,
) -> Vec<u8> {
    let cip_request = extract_cip_request(payload);
    if cip_request.len() < 4 {
        return build_cip_error_reply(CIP_REPLY_GET_ATTRIBUTE_LIST, CIP_STATUS_PATH_SEGMENT_ERROR);
    }

    let (tag_name, _) = match parse_tag_and_path(&cip_request) {
        Some(value) => value,
        None => {
            return build_cip_error_reply(
                CIP_REPLY_GET_ATTRIBUTE_LIST,
                CIP_STATUS_PATH_SEGMENT_ERROR,
            );
        }
    };

    let path_words = cip_request[1] as usize;
    let attrs_start = 2 + path_words * 2;
    if cip_request.len() < attrs_start + 2 {
        return build_cip_error_reply(CIP_REPLY_GET_ATTRIBUTE_LIST, CIP_STATUS_PATH_SEGMENT_ERROR);
    }

    let requested_attr_count =
        u16::from_le_bytes([cip_request[attrs_start], cip_request[attrs_start + 1]]) as usize;
    let attr_ids_start = attrs_start + 2;
    if cip_request.len() < attr_ids_start + requested_attr_count * 2 {
        return build_cip_error_reply(CIP_REPLY_GET_ATTRIBUTE_LIST, CIP_STATUS_PATH_SEGMENT_ERROR);
    }

    let tags_guard = tags.lock().expect("tag lock");
    let Some(value) = tags_guard.get(&tag_name) else {
        return build_cip_error_reply(CIP_REPLY_GET_ATTRIBUTE_LIST, CIP_STATUS_PATH_SEGMENT_ERROR);
    };

    // Get Attribute List reply format (1756-PM020 Symbol Object service 0x03):
    // CIP reply header, attribute count, then one [attribute id][status][value]
    // record per requested attribute. Attribute 1 is symbol type; attribute 2 is
    // the symbol instance id used by Logix metadata services.
    let mut response = vec![CIP_REPLY_GET_ATTRIBUTE_LIST, 0x00, CIP_STATUS_SUCCESS, 0x00];
    response.extend_from_slice(&(requested_attr_count as u16).to_le_bytes());

    for index in 0..requested_attr_count {
        let attr_pos = attr_ids_start + index * 2;
        let attr_id = u16::from_le_bytes([cip_request[attr_pos], cip_request[attr_pos + 1]]);
        response.extend_from_slice(&attr_id.to_le_bytes());
        match attr_id {
            0x0001 => {
                response.extend_from_slice(&0u16.to_le_bytes());
                response.extend_from_slice(&tag_data_type(value).to_le_bytes());
            }
            0x0002 => {
                response.extend_from_slice(&0u16.to_le_bytes());
                response.extend_from_slice(&tag_instance_id(&tag_name).to_le_bytes());
            }
            _ => {
                response.extend_from_slice(&(CIP_STATUS_PATH_SEGMENT_ERROR as u16).to_le_bytes());
            }
        }
    }

    response
}

fn build_read_response(
    payload: &[u8],
    tags: &Arc<Mutex<HashMap<String, TagValue>>>,
    behavior: &Arc<SimRuntimeBehavior>,
) -> Vec<u8> {
    let cip_request = extract_cip_request(payload);
    build_read_response_from_cip_request(&cip_request, tags, behavior)
}

fn build_read_response_from_cip_request(
    cip_request: &[u8],
    tags: &Arc<Mutex<HashMap<String, TagValue>>>,
    behavior: &Arc<SimRuntimeBehavior>,
) -> Vec<u8> {
    let (tag_name, element_index) =
        parse_tag_and_path(cip_request).unwrap_or(("DINT_TAG".to_string(), None));
    let read_count = behavior.record_read(&tag_name);

    if let Some(status) = behavior.read_failure_status(&tag_name, read_count) {
        return build_cip_error_reply(CIP_REPLY_READ, status);
    }

    let requested_count = parse_read_element_count(cip_request).unwrap_or(1) as usize;
    if behavior.count_reads_as_dint(&tag_name) {
        return build_value_response(
            TagValue::Dint(read_count as i32),
            element_index,
            requested_count,
        );
    }

    let tags_guard = tags.lock().expect("tag lock");
    let Some(value) = tags_guard.get(&tag_name).cloned() else {
        return build_cip_error_reply(CIP_REPLY_READ, CIP_STATUS_PATH_SEGMENT_ERROR);
    };
    build_value_response(value, element_index, requested_count)
}

fn extract_cip_service(payload: &[u8]) -> Option<u8> {
    if payload.len() < 8 {
        return None;
    }

    let item_count = u16::from_le_bytes([payload[6], payload[7]]);
    let mut pos = 8;
    for _ in 0..item_count {
        if pos + 4 > payload.len() {
            return None;
        }
        let item_type = u16::from_le_bytes([payload[pos], payload[pos + 1]]);
        let item_len = u16::from_le_bytes([payload[pos + 2], payload[pos + 3]]) as usize;
        pos += 4;
        if pos + item_len > payload.len() {
            return None;
        }
        if item_type == 0x00B2 {
            let ucmm = &payload[pos..pos + item_len];
            if ucmm.len() < 11 {
                return None;
            }
            return Some(ucmm[10]);
        }
        pos += item_len;
    }
    None
}

fn handle_write(
    payload: &[u8],
    tags: &Arc<Mutex<HashMap<String, TagValue>>>,
    behavior: &Arc<SimRuntimeBehavior>,
) -> Vec<u8> {
    let cip_request = extract_cip_request(payload);
    handle_write_cip_request(&cip_request, tags, behavior)
}

fn handle_write_cip_request(
    cip_request: &[u8],
    tags: &Arc<Mutex<HashMap<String, TagValue>>>,
    behavior: &Arc<SimRuntimeBehavior>,
) -> Vec<u8> {
    if cip_request.len() < 6 {
        return build_cip_error_reply(CIP_REPLY_WRITE, CIP_STATUS_PATH_SEGMENT_ERROR);
    }

    let (tag_name, element_index) = match parse_tag_and_path(cip_request) {
        Some(value) => value,
        None => return build_cip_error_reply(CIP_REPLY_WRITE, CIP_STATUS_PATH_SEGMENT_ERROR),
    };

    behavior.record_write(&tag_name);

    if behavior.should_fail_write(&tag_name) {
        return build_cip_error_reply(CIP_REPLY_WRITE, CIP_STATUS_PATH_SEGMENT_ERROR);
    }

    let path_words = cip_request[1] as usize;
    let path_bytes = path_words * 2;
    let path_end = 2 + path_bytes;
    if cip_request.len() < path_end + 4 {
        return build_cip_error_reply(CIP_REPLY_WRITE, CIP_STATUS_PATH_SEGMENT_ERROR);
    }

    let data_type = u16::from_le_bytes([cip_request[path_end], cip_request[path_end + 1]]);
    let mut data_start = path_end + 4;
    let mut structure_handle = None;
    if data_type == CIP_TYPE_STRUCTURE {
        if cip_request.len() < path_end + 6 {
            return build_cip_error_reply(CIP_REPLY_WRITE, CIP_STATUS_PATH_SEGMENT_ERROR);
        }

        structure_handle = Some(u16::from_le_bytes([
            cip_request[path_end + 2],
            cip_request[path_end + 3],
        ]));
        data_start = path_end + 6;
    }

    if data_type == CIP_TYPE_BOOL_ARRAY_DWORD {
        if cip_request.len() < data_start + 4 {
            return build_cip_error_reply(CIP_REPLY_WRITE, CIP_STATUS_PATH_SEGMENT_ERROR);
        }

        let dword_value = u32::from_le_bytes([
            cip_request[data_start],
            cip_request[data_start + 1],
            cip_request[data_start + 2],
            cip_request[data_start + 3],
        ]);
        let dword_index = element_index.unwrap_or(0);
        let mut tags = tags.lock().expect("tag lock");
        if let Some(TagValue::Array(items)) = tags.get_mut(&tag_name)
            && write_bool_array_dword(items, dword_index, dword_value)
        {
            return build_cip_ok_write_reply();
        }

        return build_cip_error_reply(CIP_REPLY_WRITE, CIP_STATUS_PATH_SEGMENT_ERROR);
    }

    let value = match data_type {
        CIP_TYPE_BOOL => cip_request.get(data_start).map(|b| TagValue::Bool(*b != 0)),
        CIP_TYPE_SINT => cip_request
            .get(data_start)
            .map(|b| TagValue::Sint(i8::from_le_bytes([*b]))),
        CIP_TYPE_INT => {
            if cip_request.len() < data_start + 2 {
                None
            } else {
                Some(TagValue::Int(i16::from_le_bytes([
                    cip_request[data_start],
                    cip_request[data_start + 1],
                ])))
            }
        }
        CIP_TYPE_DINT => {
            if cip_request.len() < data_start + 4 {
                None
            } else {
                Some(TagValue::Dint(i32::from_le_bytes([
                    cip_request[data_start],
                    cip_request[data_start + 1],
                    cip_request[data_start + 2],
                    cip_request[data_start + 3],
                ])))
            }
        }
        CIP_TYPE_LINT => {
            if cip_request.len() < data_start + 8 {
                None
            } else {
                Some(TagValue::Lint(i64::from_le_bytes([
                    cip_request[data_start],
                    cip_request[data_start + 1],
                    cip_request[data_start + 2],
                    cip_request[data_start + 3],
                    cip_request[data_start + 4],
                    cip_request[data_start + 5],
                    cip_request[data_start + 6],
                    cip_request[data_start + 7],
                ])))
            }
        }
        CIP_TYPE_USINT => cip_request.get(data_start).copied().map(TagValue::Usint),
        CIP_TYPE_UINT => {
            if cip_request.len() < data_start + 2 {
                None
            } else {
                Some(TagValue::Uint(u16::from_le_bytes([
                    cip_request[data_start],
                    cip_request[data_start + 1],
                ])))
            }
        }
        CIP_TYPE_UDINT => {
            if cip_request.len() < data_start + 4 {
                None
            } else {
                Some(TagValue::Udint(u32::from_le_bytes([
                    cip_request[data_start],
                    cip_request[data_start + 1],
                    cip_request[data_start + 2],
                    cip_request[data_start + 3],
                ])))
            }
        }
        CIP_TYPE_ULINT => {
            if cip_request.len() < data_start + 8 {
                None
            } else {
                Some(TagValue::Ulint(u64::from_le_bytes([
                    cip_request[data_start],
                    cip_request[data_start + 1],
                    cip_request[data_start + 2],
                    cip_request[data_start + 3],
                    cip_request[data_start + 4],
                    cip_request[data_start + 5],
                    cip_request[data_start + 6],
                    cip_request[data_start + 7],
                ])))
            }
        }
        CIP_TYPE_REAL => {
            if cip_request.len() < data_start + 4 {
                None
            } else {
                Some(TagValue::Real(f32::from_le_bytes([
                    cip_request[data_start],
                    cip_request[data_start + 1],
                    cip_request[data_start + 2],
                    cip_request[data_start + 3],
                ])))
            }
        }
        CIP_TYPE_LREAL => {
            if cip_request.len() < data_start + 8 {
                None
            } else {
                Some(TagValue::Lreal(f64::from_le_bytes([
                    cip_request[data_start],
                    cip_request[data_start + 1],
                    cip_request[data_start + 2],
                    cip_request[data_start + 3],
                    cip_request[data_start + 4],
                    cip_request[data_start + 5],
                    cip_request[data_start + 6],
                    cip_request[data_start + 7],
                ])))
            }
        }
        CIP_TYPE_STRING => {
            // Hardware validation 2026-07-02: atomic 0x00CE writes to Logix
            // STRING tags are rejected as tag type mismatch (0x2107). The
            // accepted form is the structure marker plus standard STRING handle.
            return build_cip_extended_error_reply(CIP_REPLY_WRITE, CIP_EXT_STATUS_TYPE_MISMATCH);
        }
        CIP_TYPE_STRUCTURE => {
            let Some(handle) = structure_handle else {
                return build_cip_error_reply(CIP_REPLY_WRITE, CIP_STATUS_PATH_SEGMENT_ERROR);
            };
            let mut tags = tags.lock().expect("tag lock");
            let capacity = match tags.get(&tag_name) {
                Some(TagValue::CustomString {
                    handle: tag_handle,
                    capacity,
                    ..
                }) if *tag_handle == handle => *capacity,
                _ if handle == CIP_STANDARD_STRING_HANDLE => CIP_STANDARD_STRING_DATA_LEN,
                _ => {
                    return build_cip_extended_error_reply(
                        CIP_REPLY_WRITE,
                        CIP_EXT_STATUS_TYPE_MISMATCH,
                    );
                }
            };
            let Some(parsed_value) = parse_string_payload(&cip_request[data_start..], capacity)
            else {
                return build_cip_error_reply(CIP_REPLY_WRITE, CIP_STATUS_PATH_SEGMENT_ERROR);
            };
            let TagValue::String(parsed_text) = parsed_value else {
                return build_cip_error_reply(CIP_REPLY_WRITE, CIP_STATUS_PATH_SEGMENT_ERROR);
            };
            match tags.get_mut(&tag_name) {
                Some(TagValue::CustomString { value, .. }) => *value = parsed_text,
                _ => {
                    tags.insert(tag_name, TagValue::String(parsed_text));
                }
            }
            return build_cip_ok_write_reply();
        }
        _ => None,
    };

    let Some(value) = value else {
        return build_cip_error_reply(CIP_REPLY_WRITE, CIP_STATUS_PATH_SEGMENT_ERROR);
    };

    let mut tags = tags.lock().expect("tag lock");
    if let Some(index) = element_index
        && let Some(TagValue::Array(items)) = tags.get_mut(&tag_name)
        && index < items.len()
    {
        items[index] = value;
        return build_cip_ok_write_reply();
    }

    tags.insert(tag_name, value);
    build_cip_ok_write_reply()
}

fn build_cip_ok_write_reply() -> Vec<u8> {
    vec![CIP_REPLY_WRITE, 0x00, CIP_STATUS_SUCCESS, 0x00]
}

fn build_cip_error_reply(reply_service: u8, status: u8) -> Vec<u8> {
    vec![reply_service, 0x00, status, 0x00]
}

fn build_cip_extended_error_reply(reply_service: u8, extended_status: u16) -> Vec<u8> {
    let mut reply = vec![reply_service, 0x00, CIP_STATUS_EXTENDED_ERROR, 0x01];
    reply.extend_from_slice(&extended_status.to_le_bytes());
    reply
}

fn parse_string_payload(data: &[u8], capacity: usize) -> Option<TagValue> {
    if data.len() < 4 + capacity {
        return None;
    }

    let length = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if length > capacity {
        return None;
    }

    let raw = &data[4..4 + length];
    Some(TagValue::String(String::from_utf8_lossy(raw).to_string()))
}

fn append_standard_string_payload(value: &str, response: &mut Vec<u8>) {
    append_string_payload(
        CIP_STANDARD_STRING_HANDLE,
        CIP_STANDARD_STRING_DATA_LEN,
        value,
        response,
    );
    response.resize(response.len() + CIP_STANDARD_STRING_PAD_LEN, 0);
}

fn append_string_payload(handle: u16, capacity: usize, value: &str, response: &mut Vec<u8>) {
    response.extend_from_slice(&handle.to_le_bytes());
    let string_bytes = value.as_bytes();
    let data_len = string_bytes.len().min(capacity);
    response.extend_from_slice(&(data_len as u32).to_le_bytes());
    response.extend_from_slice(&string_bytes[..data_len]);
    response.resize(response.len() + (capacity - data_len), 0);
}

fn custom_string_type_prefixed(handle: u16, capacity: usize, value: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(4 + capacity + 4);
    bytes.extend_from_slice(&CIP_TYPE_STRUCTURE.to_le_bytes());
    append_string_payload(handle, capacity, value, &mut bytes);
    bytes
}

fn build_read_fragmented_response(
    payload: &[u8],
    tags: &Arc<Mutex<HashMap<String, TagValue>>>,
    behavior: &Arc<SimRuntimeBehavior>,
) -> Vec<u8> {
    let cip_request = extract_cip_request(payload);
    let (tag_name, _) = match parse_tag_and_path(&cip_request) {
        Some(value) => value,
        None => {
            return build_cip_error_reply(
                CIP_REPLY_READ_TAG_FRAGMENTED,
                CIP_STATUS_PATH_SEGMENT_ERROR,
            );
        }
    };
    let read_count = behavior.record_read(&tag_name);
    if let Some(status) = behavior.read_failure_status(&tag_name, read_count) {
        return build_cip_error_reply(CIP_REPLY_READ_TAG_FRAGMENTED, status);
    }

    let path_words = cip_request[1] as usize;
    let data_start = 2 + (path_words * 2);
    if cip_request.len() < data_start + 6 {
        return build_cip_error_reply(CIP_REPLY_READ_TAG_FRAGMENTED, CIP_STATUS_PATH_SEGMENT_ERROR);
    }
    let byte_offset = u32::from_le_bytes([
        cip_request[data_start + 2],
        cip_request[data_start + 3],
        cip_request[data_start + 4],
        cip_request[data_start + 5],
    ]) as usize;

    let tags_guard = tags.lock().expect("tag lock");
    let Some(TagValue::CustomString {
        handle,
        capacity,
        value,
    }) = tags_guard.get(&tag_name)
    else {
        return build_cip_error_reply(CIP_REPLY_READ_TAG_FRAGMENTED, CIP_STATUS_PATH_SEGMENT_ERROR);
    };
    let full = custom_string_type_prefixed(*handle, *capacity, value);
    if byte_offset >= full.len() {
        return build_cip_error_reply(CIP_REPLY_READ_TAG_FRAGMENTED, CIP_STATUS_PATH_SEGMENT_ERROR);
    }

    let end = usize::min(byte_offset + CIP_FRAGMENT_READ_CHUNK, full.len());
    let status = if end < full.len() {
        CIP_STATUS_PARTIAL_TRANSFER
    } else {
        CIP_STATUS_SUCCESS
    };
    let mut response = vec![CIP_REPLY_READ_TAG_FRAGMENTED, 0x00, status, 0x00];
    response.extend_from_slice(&full[byte_offset..end]);
    response
}

fn handle_write_fragmented(
    payload: &[u8],
    tags: &Arc<Mutex<HashMap<String, TagValue>>>,
    behavior: &Arc<SimRuntimeBehavior>,
) -> Vec<u8> {
    let cip_request = extract_cip_request(payload);
    let (tag_name, _) = match parse_tag_and_path(&cip_request) {
        Some(value) => value,
        None => {
            return build_cip_error_reply(
                CIP_REPLY_WRITE_TAG_FRAGMENTED,
                CIP_STATUS_PATH_SEGMENT_ERROR,
            );
        }
    };

    behavior.record_write(&tag_name);

    if behavior.should_fail_write(&tag_name) {
        return build_cip_error_reply(
            CIP_REPLY_WRITE_TAG_FRAGMENTED,
            CIP_STATUS_PATH_SEGMENT_ERROR,
        );
    }

    let path_words = cip_request[1] as usize;
    let data_start = 2 + (path_words * 2);
    if cip_request.len() < data_start + 10 {
        return build_cip_error_reply(
            CIP_REPLY_WRITE_TAG_FRAGMENTED,
            CIP_STATUS_PATH_SEGMENT_ERROR,
        );
    }
    let data_type = u16::from_le_bytes([cip_request[data_start], cip_request[data_start + 1]]);
    let handle = u16::from_le_bytes([cip_request[data_start + 2], cip_request[data_start + 3]]);
    let byte_offset = u32::from_le_bytes([
        cip_request[data_start + 6],
        cip_request[data_start + 7],
        cip_request[data_start + 8],
        cip_request[data_start + 9],
    ]) as usize;
    let fragment = &cip_request[data_start + 10..];

    if data_type != CIP_TYPE_STRUCTURE {
        return build_cip_error_reply(
            CIP_REPLY_WRITE_TAG_FRAGMENTED,
            CIP_STATUS_PATH_SEGMENT_ERROR,
        );
    }

    let mut tags_guard = tags.lock().expect("tag lock");
    let Some(TagValue::CustomString {
        handle: tag_handle,
        capacity,
        value,
    }) = tags_guard.get_mut(&tag_name)
    else {
        return build_cip_error_reply(
            CIP_REPLY_WRITE_TAG_FRAGMENTED,
            CIP_STATUS_PATH_SEGMENT_ERROR,
        );
    };
    if *tag_handle != handle {
        return build_cip_extended_error_reply(
            CIP_REPLY_WRITE_TAG_FRAGMENTED,
            CIP_EXT_STATUS_TYPE_MISMATCH,
        );
    }

    let mut raw = vec![0u8; 4 + *capacity];
    let existing = value.as_bytes();
    let existing_len = existing.len().min(*capacity);
    raw[0..4].copy_from_slice(&(existing_len as u32).to_le_bytes());
    raw[4..4 + existing_len].copy_from_slice(&existing[..existing_len]);

    if byte_offset + fragment.len() > raw.len() {
        return build_cip_error_reply(
            CIP_REPLY_WRITE_TAG_FRAGMENTED,
            CIP_STATUS_PATH_SEGMENT_ERROR,
        );
    }
    raw[byte_offset..byte_offset + fragment.len()].copy_from_slice(fragment);

    let len = u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]) as usize;
    if len > *capacity {
        return build_cip_error_reply(
            CIP_REPLY_WRITE_TAG_FRAGMENTED,
            CIP_STATUS_PATH_SEGMENT_ERROR,
        );
    }
    *value = String::from_utf8_lossy(&raw[4..4 + len]).to_string();

    vec![
        CIP_REPLY_WRITE_TAG_FRAGMENTED,
        0x00,
        CIP_STATUS_SUCCESS,
        0x00,
    ]
}

fn build_multiple_service_response(
    payload: &[u8],
    tags: &Arc<Mutex<HashMap<String, TagValue>>>,
    behavior: &Arc<SimRuntimeBehavior>,
) -> Vec<u8> {
    let request = extract_cip_request(payload);
    if request.len() < 8 {
        return build_cip_error_reply(
            CIP_REPLY_MULTIPLE_SERVICE_PACKET,
            CIP_STATUS_PATH_SEGMENT_ERROR,
        );
    }

    let service_count = u16::from_le_bytes([request[6], request[7]]) as usize;
    if service_count == 0 {
        return build_cip_error_reply(CIP_REPLY_MULTIPLE_SERVICE_PACKET, CIP_STATUS_SUCCESS);
    }

    let offsets_start = 8;
    let offsets_end = offsets_start + (service_count * 2);
    if request.len() < offsets_end {
        return build_cip_error_reply(
            CIP_REPLY_MULTIPLE_SERVICE_PACKET,
            CIP_STATUS_PATH_SEGMENT_ERROR,
        );
    }

    let mut offsets = Vec::with_capacity(service_count);
    for i in 0..service_count {
        let pos = offsets_start + (i * 2);
        offsets.push(u16::from_le_bytes([request[pos], request[pos + 1]]) as usize);
    }

    let mut replies: Vec<Vec<u8>> = Vec::with_capacity(service_count);
    for i in 0..service_count {
        let start = 6 + offsets[i];
        let end = if i + 1 < service_count {
            6 + offsets[i + 1]
        } else {
            request.len()
        };

        if start >= request.len() || end > request.len() || start >= end {
            replies.push(build_cip_error_reply(
                CIP_REPLY_READ,
                CIP_STATUS_PATH_SEGMENT_ERROR,
            ));
            continue;
        }

        let service_request = &request[start..end];
        if service_request.is_empty() {
            replies.push(build_cip_error_reply(
                CIP_REPLY_READ,
                CIP_STATUS_PATH_SEGMENT_ERROR,
            ));
            continue;
        }

        let reply = match service_request[0] {
            CIP_READ_TAG => build_read_response_from_cip_request(service_request, tags, behavior),
            CIP_WRITE_TAG => handle_write_cip_request(service_request, tags, behavior),
            _ => build_cip_error_reply(CIP_REPLY_READ, CIP_STATUS_PATH_SEGMENT_ERROR),
        };
        replies.push(reply);
    }

    // Reply format expected by client parser:
    // [0]=0x8A [1]=0x00 [2]=status [3]=add_status_size [4..5]=reply_count [offset_table] [replies...]
    let mut response = vec![
        CIP_REPLY_MULTIPLE_SERVICE_PACKET,
        0x00,
        CIP_STATUS_SUCCESS,
        0x00,
    ];
    response.extend_from_slice(&(service_count as u16).to_le_bytes());

    let mut current_offset = 2 + (service_count * 2); // relative to byte 4
    for reply in &replies {
        response.extend_from_slice(&(current_offset as u16).to_le_bytes());
        current_offset += reply.len();
    }

    for reply in replies {
        response.extend_from_slice(&reply);
    }

    response
}

fn extract_cip_request(payload: &[u8]) -> Vec<u8> {
    if payload.len() < 8 {
        return Vec::new();
    }
    let item_count = u16::from_le_bytes([payload[6], payload[7]]);
    let mut pos = 8;
    for _ in 0..item_count {
        if pos + 4 > payload.len() {
            break;
        }
        let item_type = u16::from_le_bytes([payload[pos], payload[pos + 1]]);
        let item_len = u16::from_le_bytes([payload[pos + 2], payload[pos + 3]]) as usize;
        pos += 4;
        if pos + item_len > payload.len() {
            break;
        }
        if item_type == 0x00B2 {
            let ucmm = &payload[pos..pos + item_len];
            if ucmm.len() < 10 {
                return Vec::new();
            }
            let msg_len = u16::from_le_bytes([ucmm[8], ucmm[9]]) as usize;
            let start = 10;
            let end = usize::min(start + msg_len, ucmm.len());
            return ucmm[start..end].to_vec();
        }
        pos += item_len;
    }
    Vec::new()
}

fn build_value_response(
    value: TagValue,
    element_index: Option<usize>,
    requested_count: usize,
) -> Vec<u8> {
    match value {
        TagValue::Array(items) => {
            let start = element_index.unwrap_or(0);
            let count = requested_count.max(1);
            let subset: Vec<TagValue> = items.iter().skip(start).take(count).cloned().collect();
            if subset.is_empty() {
                return build_value_response(TagValue::Dint(0), None, 1);
            }

            match &subset[0] {
                TagValue::Bool(_) => {
                    // Read Tag reply format (1756-PM020 service 0x4C):
                    // [0xCC][reserved][status][additional-status-count][type u16][data...].
                    // Logix does not include an element-count word in multi-element replies.
                    let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
                    response.extend_from_slice(&CIP_TYPE_BOOL_ARRAY_DWORD.to_le_bytes());
                    let start_dword = element_index.unwrap_or(0);
                    for dword_offset in 0..count {
                        let dword = bool_array_dword(&items, start_dword + dword_offset);
                        response.extend_from_slice(&dword.to_le_bytes());
                    }
                    response
                }
                TagValue::Sint(_) => build_array_value_response(&subset, CIP_TYPE_SINT),
                TagValue::Int(_) => build_array_value_response(&subset, CIP_TYPE_INT),
                TagValue::Dint(_) => {
                    // Read Tag reply format (1756-PM020 service 0x4C):
                    // [0xCC][reserved][status][additional-status-count][type u16][data...].
                    // Logix does not include an element-count word in multi-element replies.
                    let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
                    response.extend_from_slice(&CIP_TYPE_DINT.to_le_bytes());
                    for item in subset {
                        if let TagValue::Dint(v) = item {
                            response.extend_from_slice(&v.to_le_bytes());
                        }
                    }
                    response
                }
                TagValue::Lint(_) => build_array_value_response(&subset, CIP_TYPE_LINT),
                TagValue::Usint(_) => build_array_value_response(&subset, CIP_TYPE_USINT),
                TagValue::Uint(_) => build_array_value_response(&subset, CIP_TYPE_UINT),
                TagValue::Udint(_) => build_array_value_response(&subset, CIP_TYPE_UDINT),
                TagValue::Ulint(_) => build_array_value_response(&subset, CIP_TYPE_ULINT),
                TagValue::Real(_) => {
                    // Read Tag reply format (1756-PM020 service 0x4C):
                    // [0xCC][reserved][status][additional-status-count][type u16][data...].
                    // Logix does not include an element-count word in multi-element replies.
                    let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
                    response.extend_from_slice(&CIP_TYPE_REAL.to_le_bytes());
                    for item in subset {
                        if let TagValue::Real(v) = item {
                            response.extend_from_slice(&v.to_le_bytes());
                        }
                    }
                    response
                }
                TagValue::Lreal(_) => build_array_value_response(&subset, CIP_TYPE_LREAL),
                TagValue::String(_) | TagValue::CustomString { .. } => {
                    // Simulator keeps string arrays simple: return first requested string.
                    build_value_response(subset[0].clone(), None, 1)
                }
                TagValue::Array(_) => {
                    // Nested arrays are not modeled in the simulator; return a safe default.
                    build_value_response(TagValue::Dint(0), None, 1)
                }
            }
        }
        TagValue::Bool(v) => {
            let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
            response.extend_from_slice(&CIP_TYPE_BOOL.to_le_bytes());
            response.push(if v { 0xFF } else { 0x00 });
            response
        }
        TagValue::Sint(v) => {
            let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
            response.extend_from_slice(&CIP_TYPE_SINT.to_le_bytes());
            response.extend_from_slice(&v.to_le_bytes());
            response
        }
        TagValue::Int(v) => {
            let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
            response.extend_from_slice(&CIP_TYPE_INT.to_le_bytes());
            response.extend_from_slice(&v.to_le_bytes());
            response
        }
        TagValue::Dint(v) => {
            let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
            response.extend_from_slice(&CIP_TYPE_DINT.to_le_bytes());
            response.extend_from_slice(&v.to_le_bytes());
            response
        }
        TagValue::Lint(v) => {
            let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
            response.extend_from_slice(&CIP_TYPE_LINT.to_le_bytes());
            response.extend_from_slice(&v.to_le_bytes());
            response
        }
        TagValue::Usint(v) => {
            let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
            response.extend_from_slice(&CIP_TYPE_USINT.to_le_bytes());
            response.push(v);
            response
        }
        TagValue::Uint(v) => {
            let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
            response.extend_from_slice(&CIP_TYPE_UINT.to_le_bytes());
            response.extend_from_slice(&v.to_le_bytes());
            response
        }
        TagValue::Udint(v) => {
            let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
            response.extend_from_slice(&CIP_TYPE_UDINT.to_le_bytes());
            response.extend_from_slice(&v.to_le_bytes());
            response
        }
        TagValue::Ulint(v) => {
            let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
            response.extend_from_slice(&CIP_TYPE_ULINT.to_le_bytes());
            response.extend_from_slice(&v.to_le_bytes());
            response
        }
        TagValue::Real(v) => {
            let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
            response.extend_from_slice(&CIP_TYPE_REAL.to_le_bytes());
            response.extend_from_slice(&v.to_le_bytes());
            response
        }
        TagValue::Lreal(v) => {
            let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
            response.extend_from_slice(&CIP_TYPE_LREAL.to_le_bytes());
            response.extend_from_slice(&v.to_le_bytes());
            response
        }
        TagValue::String(v) => {
            let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
            response.extend_from_slice(&CIP_TYPE_STRUCTURE.to_le_bytes());
            append_standard_string_payload(&v, &mut response);
            response
        }
        TagValue::CustomString {
            handle,
            capacity,
            value,
        } => {
            let full = custom_string_type_prefixed(handle, capacity, &value);
            if full.len() > 500 {
                let end = usize::min(CIP_FRAGMENT_READ_CHUNK, full.len());
                let mut response = vec![CIP_REPLY_READ, 0x00, CIP_STATUS_PARTIAL_TRANSFER, 0x00];
                response.extend_from_slice(&full[..end]);
                response
            } else {
                let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
                response.extend_from_slice(&full);
                response
            }
        }
    }
}

fn build_array_value_response(items: &[TagValue], data_type: u16) -> Vec<u8> {
    // Read Tag reply format (1756-PM020 service 0x4C):
    // [0xCC][reserved][status][additional-status-count][type u16][data...].
    // Logix does not include an element-count word in multi-element replies.
    let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
    response.extend_from_slice(&data_type.to_le_bytes());
    for item in items {
        match item {
            TagValue::Sint(v) => response.extend_from_slice(&v.to_le_bytes()),
            TagValue::Int(v) => response.extend_from_slice(&v.to_le_bytes()),
            TagValue::Lint(v) => response.extend_from_slice(&v.to_le_bytes()),
            TagValue::Usint(v) => response.push(*v),
            TagValue::Uint(v) => response.extend_from_slice(&v.to_le_bytes()),
            TagValue::Udint(v) => response.extend_from_slice(&v.to_le_bytes()),
            TagValue::Ulint(v) => response.extend_from_slice(&v.to_le_bytes()),
            TagValue::Lreal(v) => response.extend_from_slice(&v.to_le_bytes()),
            _ => {}
        }
    }
    response
}

fn tag_data_type(value: &TagValue) -> u16 {
    match value {
        TagValue::Bool(_) => CIP_TYPE_BOOL,
        TagValue::Sint(_) => CIP_TYPE_SINT,
        TagValue::Int(_) => CIP_TYPE_INT,
        TagValue::Dint(_) => CIP_TYPE_DINT,
        TagValue::Lint(_) => CIP_TYPE_LINT,
        TagValue::Usint(_) => CIP_TYPE_USINT,
        TagValue::Uint(_) => CIP_TYPE_UINT,
        TagValue::Udint(_) => CIP_TYPE_UDINT,
        TagValue::Ulint(_) => CIP_TYPE_ULINT,
        TagValue::Real(_) => CIP_TYPE_REAL,
        TagValue::Lreal(_) => CIP_TYPE_LREAL,
        TagValue::String(_) | TagValue::CustomString { .. } => CIP_TYPE_STRUCTURE,
        TagValue::Array(items) => items.first().map(tag_data_type).unwrap_or(CIP_TYPE_DINT),
    }
}

fn tag_instance_id(tag_name: &str) -> u32 {
    match tag_name {
        "DINT_TAG" => 1,
        "BOOL_TAG" => 2,
        "SINT_TAG" => 3,
        "INT_TAG" => 4,
        "LINT_TAG" => 5,
        "USINT_TAG" => 6,
        "UINT_TAG" => 7,
        "UDINT_TAG" => 8,
        "ULINT_TAG" => 9,
        "REAL_TAG" => 10,
        "LREAL_TAG" => 11,
        "STRING_TAG" => 12,
        "BOOL_ARRAY" => 13,
        "UDT_ARRAY[3].BOOL_NESTED" => 14,
        "UDT_ARRAY[3].DINT_MEMBER" => 15,
        "DINT_ARRAY" => 16,
        "REAL_ARRAY" => 17,
        _ => 0,
    }
}

fn bool_array_dword(items: &[TagValue], dword_index: usize) -> u32 {
    let start = dword_index * 32;
    let mut dword = 0u32;
    for bit in 0..32 {
        if let Some(TagValue::Bool(true)) = items.get(start + bit) {
            dword |= 1u32 << bit;
        }
    }
    dword
}

fn write_bool_array_dword(items: &mut [TagValue], dword_index: usize, dword: u32) -> bool {
    let start = dword_index * 32;
    if start >= items.len() {
        return false;
    }

    for bit in 0..32 {
        if let Some(item @ TagValue::Bool(_)) = items.get_mut(start + bit) {
            *item = TagValue::Bool((dword >> bit) & 1 != 0);
        }
    }
    true
}

fn parse_tag_and_path(cip_request: &[u8]) -> Option<(String, Option<usize>)> {
    if cip_request.len() < 2 {
        return None;
    }

    let path_words = cip_request[1] as usize;
    let path_bytes = path_words * 2;
    if cip_request.len() < 2 + path_bytes {
        return None;
    }

    let path = &cip_request[2..2 + path_bytes];
    let mut pos = 0;
    let mut path_parts: Vec<String> = Vec::new();
    let mut pending_element_index = None;

    while pos < path.len() {
        match path[pos] {
            0x91 => {
                if let Some(index) = pending_element_index.take()
                    && let Some(last_part) = path_parts.last_mut()
                {
                    last_part.push_str(&format!("[{}]", index));
                }

                if pos + 1 >= path.len() {
                    break;
                }
                let len = path[pos + 1] as usize;
                let start = pos + 2;
                let end = start + len;
                if end > path.len() {
                    break;
                }
                let name = String::from_utf8_lossy(&path[start..end]).to_string();
                path_parts.push(name);
                pos = end + (len % 2);
            }
            0x28 => {
                if pos + 1 >= path.len() {
                    break;
                }
                pending_element_index = Some(path[pos + 1] as usize);
                pos += 2;
            }
            0x29 => {
                if pos + 3 >= path.len() {
                    break;
                }
                pending_element_index =
                    Some(u16::from_le_bytes([path[pos + 2], path[pos + 3]]) as usize);
                pos += 4;
            }
            0x2A => {
                if pos + 5 >= path.len() {
                    break;
                }
                pending_element_index = Some(u32::from_le_bytes([
                    path[pos + 2],
                    path[pos + 3],
                    path[pos + 4],
                    path[pos + 5],
                ]) as usize);
                pos += 6;
            }
            _ => {
                pos += 1;
            }
        }
    }

    if path_parts.is_empty() {
        None
    } else {
        Some((path_parts.join("."), pending_element_index))
    }
}

fn parse_read_element_count(cip_request: &[u8]) -> Option<u16> {
    if cip_request.len() < 2 {
        return None;
    }
    let path_words = cip_request[1] as usize;
    let path_bytes = path_words * 2;
    let pos = 2 + path_bytes;
    if cip_request.len() < pos + 2 {
        return None;
    }
    Some(u16::from_le_bytes([cip_request[pos], cip_request[pos + 1]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_string_write_shape_is_rejected_with_type_mismatch() {
        let tags = Arc::new(Mutex::new(HashMap::new()));
        let behavior = Arc::new(SimRuntimeBehavior {
            config: SimBehavior::default(),
            send_rr_count: AtomicUsize::new(0),
            read_counts: Mutex::new(HashMap::new()),
            write_counts: Mutex::new(HashMap::new()),
        });
        let request = atomic_string_write_request("STRING_TAG", "OldShape");

        let reply = handle_write_cip_request(&request, &tags, &behavior);

        assert_eq!(
            reply,
            vec![
                CIP_REPLY_WRITE,
                0x00,
                CIP_STATUS_EXTENDED_ERROR,
                0x01,
                0x07,
                0x21
            ]
        );
    }

    fn atomic_string_write_request(tag: &str, value: &str) -> Vec<u8> {
        let mut path = Vec::new();
        path.push(0x91);
        path.push(tag.len() as u8);
        path.extend_from_slice(tag.as_bytes());
        if !tag.len().is_multiple_of(2) {
            path.push(0);
        }

        let mut request = vec![CIP_WRITE_TAG, (path.len() / 2) as u8];
        request.extend_from_slice(&path);
        request.extend_from_slice(&CIP_TYPE_STRING.to_le_bytes());
        request.extend_from_slice(&1u16.to_le_bytes());
        request.extend_from_slice(&(value.len() as u32).to_le_bytes());
        request.extend_from_slice(value.as_bytes());
        request
    }
}
