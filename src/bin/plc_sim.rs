use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const CMD_REGISTER_SESSION: u16 = 0x0065;
const CMD_SEND_RR_DATA: u16 = 0x006F;

const CIP_READ_TAG: u8 = 0x4C;
const CIP_WRITE_TAG: u8 = 0x4D;
const CIP_MULTIPLE_SERVICE_PACKET: u8 = 0x0A;

const CIP_REPLY_READ: u8 = 0xCC;
const CIP_REPLY_WRITE: u8 = 0xCD;
const CIP_REPLY_MULTIPLE_SERVICE_PACKET: u8 = 0x8A;
const CIP_STATUS_SUCCESS: u8 = 0x00;
const CIP_STATUS_EXTENDED_ERROR: u8 = 0xFF;
const CIP_STATUS_PATH_SEGMENT_ERROR: u8 = 0x04;
const CIP_EXT_STATUS_TYPE_MISMATCH: u16 = 0x2107;

const CIP_TYPE_DINT: u16 = 0x00C4;
const CIP_TYPE_BOOL: u16 = 0x00C1;
const CIP_TYPE_REAL: u16 = 0x00CA;
const CIP_TYPE_STRING: u16 = 0x00CE;
const CIP_TYPE_UDT: u16 = 0x02A0;
const CIP_TYPE_STRUCTURE: u16 = 0x02A0;
const CIP_STANDARD_STRING_HANDLE: u16 = 0x0FCE;
const CIP_STANDARD_STRING_DATA_LEN: usize = 82;
const CIP_STANDARD_STRING_PAD_LEN: usize = 2;
const CIP_STANDARD_STRING_PAYLOAD_LEN: usize =
    4 + CIP_STANDARD_STRING_DATA_LEN + CIP_STANDARD_STRING_PAD_LEN;

#[derive(Clone, Debug)]
enum TagValue {
    Bool(bool),
    Dint(i32),
    Real(f32),
    String(String),
    Udt(Vec<u8>),
    Array(Vec<TagValue>),
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let address = listener.local_addr().expect("addr");
    println!("PLC simulator listening on {}", address);
    println!("Set SIM_PLC_ADDRESS={} for C# tests.", address);

    let tags = Arc::new(Mutex::new(HashMap::from([
        ("DINT_TAG".to_string(), TagValue::Dint(1234)),
        ("BOOL_TAG".to_string(), TagValue::Bool(true)),
        ("REAL_TAG".to_string(), TagValue::Real(3.0)),
        (
            "STRING_TAG".to_string(),
            TagValue::String("Hello PLC".to_string()),
        ),
        (
            "UDT_TAG".to_string(),
            TagValue::Udt(vec![1, 0, 0, 0, 0x39, 0x30, 0, 0]),
        ),
        (
            "DINT_ARRAY".to_string(),
            TagValue::Array(vec![TagValue::Dint(10), TagValue::Dint(20)]),
        ),
        (
            "REAL_ARRAY".to_string(),
            TagValue::Array(vec![TagValue::Real(1.5), TagValue::Real(2.5)]),
        ),
    ])));

    loop {
        let (stream, _) = listener.accept().await.expect("accept");
        let tags = Arc::clone(&tags);
        tokio::spawn(async move {
            handle_connection(stream, tags).await;
        });
    }
}

async fn handle_connection(mut stream: TcpStream, tags: Arc<Mutex<HashMap<String, TagValue>>>) {
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
                let cip_response = build_cip_response(&payload, &tags);
                let response =
                    build_send_rr_response(session_handle, sender_context, &cip_response);
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
    response.extend_from_slice(&4u16.to_le_bytes());
    response.extend_from_slice(&session_handle.to_le_bytes());
    response.extend_from_slice(&0u32.to_le_bytes());
    response.extend_from_slice(&sender_context);
    response.extend_from_slice(&0u32.to_le_bytes());
    response.extend_from_slice(&[0u8; 4]);
    response
}

fn build_send_rr_response(
    session_handle: u32,
    sender_context: [u8; 8],
    cip_response: &[u8],
) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes());
    data.extend_from_slice(&2u16.to_le_bytes());

    data.extend_from_slice(&0u16.to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes());

    data.extend_from_slice(&0x00B2u16.to_le_bytes());
    data.extend_from_slice(&(cip_response.len() as u16).to_le_bytes());
    data.extend_from_slice(cip_response);

    let mut response = Vec::with_capacity(24 + data.len());
    response.extend_from_slice(&CMD_SEND_RR_DATA.to_le_bytes());
    response.extend_from_slice(&(data.len() as u16).to_le_bytes());
    response.extend_from_slice(&session_handle.to_le_bytes());
    response.extend_from_slice(&0u32.to_le_bytes());
    response.extend_from_slice(&sender_context);
    response.extend_from_slice(&0u32.to_le_bytes());
    response.extend_from_slice(&data);
    response
}

fn build_cip_response(payload: &[u8], tags: &Arc<Mutex<HashMap<String, TagValue>>>) -> Vec<u8> {
    let service = extract_cip_service(payload).unwrap_or(0);
    match service {
        CIP_READ_TAG => build_read_response(payload, tags),
        CIP_WRITE_TAG => handle_write(payload, tags),
        CIP_MULTIPLE_SERVICE_PACKET => build_multiple_service_response(payload, tags),
        _ => vec![CIP_REPLY_READ, 0x00, 0x01, 0x00],
    }
}

fn build_read_response(payload: &[u8], tags: &Arc<Mutex<HashMap<String, TagValue>>>) -> Vec<u8> {
    let cip_request = extract_cip_request(payload);
    let (tag_name, element_index) =
        parse_tag_and_path(&cip_request).unwrap_or(("DINT_TAG".to_string(), None));
    let requested_count = parse_read_element_count(&cip_request).unwrap_or(1) as usize;
    let tags_guard = tags.lock().expect("tag lock");
    let value = tags_guard
        .get(&tag_name)
        .cloned()
        .unwrap_or(TagValue::Dint(0));
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

fn handle_write(payload: &[u8], tags: &Arc<Mutex<HashMap<String, TagValue>>>) -> Vec<u8> {
    let cip_request = extract_cip_request(payload);
    handle_write_cip_request(&cip_request, tags)
}

fn handle_write_cip_request(
    cip_request: &[u8],
    tags: &Arc<Mutex<HashMap<String, TagValue>>>,
) -> Vec<u8> {
    if cip_request.len() < 6 {
        return build_cip_error_reply(CIP_REPLY_WRITE, CIP_STATUS_PATH_SEGMENT_ERROR);
    }

    let (tag_name, element_index) = match parse_tag_and_path(cip_request) {
        Some(value) => value,
        None => return build_cip_error_reply(CIP_REPLY_WRITE, CIP_STATUS_PATH_SEGMENT_ERROR),
    };

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

    let value = match data_type {
        CIP_TYPE_BOOL => cip_request.get(data_start).map(|b| TagValue::Bool(*b != 0)),
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
        CIP_TYPE_STRING => {
            // Hardware validation 2026-07-02: atomic 0x00CE writes to Logix
            // STRING tags are rejected as tag type mismatch (0x2107). The
            // accepted form is the structure marker plus standard STRING handle.
            return build_cip_extended_error_reply(CIP_REPLY_WRITE, CIP_EXT_STATUS_TYPE_MISMATCH);
        }
        CIP_TYPE_STRUCTURE if structure_handle == Some(CIP_STANDARD_STRING_HANDLE) => {
            parse_standard_string_payload(&cip_request[data_start..])
        }
        CIP_TYPE_STRUCTURE => {
            return build_cip_error_reply(CIP_REPLY_WRITE, CIP_STATUS_PATH_SEGMENT_ERROR);
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

fn parse_standard_string_payload(data: &[u8]) -> Option<TagValue> {
    if data.len() < CIP_STANDARD_STRING_PAYLOAD_LEN {
        return None;
    }

    let length = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if length > CIP_STANDARD_STRING_DATA_LEN {
        return None;
    }

    let raw = &data[4..4 + length];
    Some(TagValue::String(String::from_utf8_lossy(raw).to_string()))
}

fn append_standard_string_payload(value: &str, response: &mut Vec<u8>) {
    response.extend_from_slice(&CIP_STANDARD_STRING_HANDLE.to_le_bytes());
    let string_bytes = value.as_bytes();
    let data_len = string_bytes.len().min(CIP_STANDARD_STRING_DATA_LEN);
    response.extend_from_slice(&(data_len as u32).to_le_bytes());
    response.extend_from_slice(&string_bytes[..data_len]);
    response.resize(
        response.len() + (CIP_STANDARD_STRING_DATA_LEN - data_len),
        0,
    );
    response.resize(response.len() + CIP_STANDARD_STRING_PAD_LEN, 0);
}

fn build_multiple_service_response(
    payload: &[u8],
    tags: &Arc<Mutex<HashMap<String, TagValue>>>,
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

    let mut replies = Vec::with_capacity(service_count);
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
        let reply = match service_request.first().copied() {
            Some(CIP_READ_TAG) => build_read_response_from_cip_request(service_request, tags),
            Some(CIP_WRITE_TAG) => handle_write_cip_request(service_request, tags),
            _ => build_cip_error_reply(CIP_REPLY_READ, CIP_STATUS_PATH_SEGMENT_ERROR),
        };
        replies.push(reply);
    }

    let mut response = vec![
        CIP_REPLY_MULTIPLE_SERVICE_PACKET,
        0x00,
        CIP_STATUS_SUCCESS,
        0x00,
    ];
    response.extend_from_slice(&(service_count as u16).to_le_bytes());

    let mut current_offset = 2 + (service_count * 2);
    for reply in &replies {
        response.extend_from_slice(&(current_offset as u16).to_le_bytes());
        current_offset += reply.len();
    }

    for reply in replies {
        response.extend_from_slice(&reply);
    }

    response
}

fn build_read_response_from_cip_request(
    cip_request: &[u8],
    tags: &Arc<Mutex<HashMap<String, TagValue>>>,
) -> Vec<u8> {
    let (tag_name, element_index) =
        parse_tag_and_path(cip_request).unwrap_or(("DINT_TAG".to_string(), None));
    let requested_count = parse_read_element_count(cip_request).unwrap_or(1) as usize;
    let tags_guard = tags.lock().expect("tag lock");
    let value = tags_guard
        .get(&tag_name)
        .cloned()
        .unwrap_or(TagValue::Dint(0));
    build_value_response(value, element_index, requested_count)
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
                    let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
                    response.extend_from_slice(&CIP_TYPE_BOOL.to_le_bytes());
                    for item in subset {
                        if let TagValue::Bool(v) = item {
                            response.push(if v { 0xFF } else { 0x00 });
                        }
                    }
                    response
                }
                TagValue::Dint(_) => {
                    let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
                    response.extend_from_slice(&CIP_TYPE_DINT.to_le_bytes());
                    for item in subset {
                        if let TagValue::Dint(v) = item {
                            response.extend_from_slice(&v.to_le_bytes());
                        }
                    }
                    response
                }
                TagValue::Real(_) => {
                    let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
                    response.extend_from_slice(&CIP_TYPE_REAL.to_le_bytes());
                    for item in subset {
                        if let TagValue::Real(v) = item {
                            response.extend_from_slice(&v.to_le_bytes());
                        }
                    }
                    response
                }
                TagValue::String(_) => {
                    // Simulator keeps string arrays simple: return first requested string.
                    build_value_response(subset[0].clone(), None, 1)
                }
                TagValue::Udt(_) => {
                    // Simulator keeps UDT arrays simple: return first requested raw UDT.
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
        TagValue::Dint(v) => {
            let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
            response.extend_from_slice(&CIP_TYPE_DINT.to_le_bytes());
            response.extend_from_slice(&v.to_le_bytes());
            response
        }
        TagValue::Real(v) => {
            let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
            response.extend_from_slice(&CIP_TYPE_REAL.to_le_bytes());
            response.extend_from_slice(&v.to_le_bytes());
            response
        }
        TagValue::String(v) => {
            let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
            response.extend_from_slice(&CIP_TYPE_STRUCTURE.to_le_bytes());
            append_standard_string_payload(&v, &mut response);
            response
        }
        TagValue::Udt(data) => {
            let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
            response.extend_from_slice(&CIP_TYPE_UDT.to_le_bytes());
            response.extend_from_slice(&data);
            response
        }
    }
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
    let mut tag_name = None;
    let mut element_index = None;

    while pos < path.len() {
        match path[pos] {
            0x91 => {
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
                if tag_name.is_none() {
                    tag_name = Some(name);
                }
                pos = end + (len % 2);
            }
            0x28 => {
                if pos + 1 >= path.len() {
                    break;
                }
                element_index = Some(path[pos + 1] as usize);
                pos += 2;
            }
            0x29 => {
                if pos + 3 >= path.len() {
                    break;
                }
                element_index = Some(u16::from_le_bytes([path[pos + 2], path[pos + 3]]) as usize);
                pos += 4;
            }
            0x2A => {
                if pos + 5 >= path.len() {
                    break;
                }
                element_index = Some(u32::from_le_bytes([
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

    tag_name.map(|name| (name, element_index))
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
