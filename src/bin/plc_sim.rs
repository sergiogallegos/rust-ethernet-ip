use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const CMD_REGISTER_SESSION: u16 = 0x0065;
const CMD_SEND_RR_DATA: u16 = 0x006F;

const CIP_READ_TAG: u8 = 0x4C;
const CIP_WRITE_TAG: u8 = 0x4D;

const CIP_REPLY_READ: u8 = 0xCC;
const CIP_REPLY_WRITE: u8 = 0xCD;

const CIP_TYPE_DINT: u16 = 0x00C4;
const CIP_TYPE_BOOL: u16 = 0x00C1;
const CIP_TYPE_REAL: u16 = 0x00CA;
const CIP_TYPE_STRING: u16 = 0x00CE;

#[derive(Clone, Debug)]
enum TagValue {
    Bool(bool),
    Dint(i32),
    Real(f32),
    String(String),
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
        ("REAL_TAG".to_string(), TagValue::Real(3.14)),
        (
            "STRING_TAG".to_string(),
            TagValue::String("Hello PLC".to_string()),
        ),
        (
            "DINT_ARRAY".to_string(),
            TagValue::Array(vec![TagValue::Dint(10), TagValue::Dint(20)]),
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

        let mut payload = vec![0u8; length];
        if length > 0 && stream.read_exact(&mut payload).await.is_err() {
            break;
        }

        match cmd {
            CMD_REGISTER_SESSION => {
                let response = build_register_session_response();
                if stream.write_all(&response).await.is_err() {
                    break;
                }
            }
            CMD_SEND_RR_DATA => {
                let cip_response = build_cip_response(&payload, &tags);
                let response = build_send_rr_response(session_handle, &cip_response);
                if stream.write_all(&response).await.is_err() {
                    break;
                }
            }
            _ => break,
        }
    }
}

fn build_register_session_response() -> Vec<u8> {
    let session_handle = 0x12345678_u32;
    let mut response = Vec::with_capacity(28);
    response.extend_from_slice(&CMD_REGISTER_SESSION.to_le_bytes());
    response.extend_from_slice(&4u16.to_le_bytes());
    response.extend_from_slice(&session_handle.to_le_bytes());
    response.extend_from_slice(&0u32.to_le_bytes());
    response.extend_from_slice(&[0u8; 8]);
    response.extend_from_slice(&0u32.to_le_bytes());
    response.extend_from_slice(&[0u8; 4]);
    response
}

fn build_send_rr_response(session_handle: u32, cip_response: &[u8]) -> Vec<u8> {
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
    response.extend_from_slice(&[0u8; 8]);
    response.extend_from_slice(&0u32.to_le_bytes());
    response.extend_from_slice(&data);
    response
}

fn build_cip_response(payload: &[u8], tags: &Arc<Mutex<HashMap<String, TagValue>>>) -> Vec<u8> {
    let service = extract_cip_service(payload).unwrap_or(0);
    match service {
        CIP_READ_TAG => build_read_response(payload, tags),
        CIP_WRITE_TAG => {
            handle_write(payload, tags);
            vec![CIP_REPLY_WRITE, 0x00, 0x00, 0x00]
        }
        _ => vec![CIP_REPLY_READ, 0x00, 0x01, 0x00],
    }
}

fn build_read_response(payload: &[u8], tags: &Arc<Mutex<HashMap<String, TagValue>>>) -> Vec<u8> {
    let cip_request = extract_cip_request(payload);
    let (tag_name, element_index) =
        parse_tag_and_path(&cip_request).unwrap_or(("DINT_TAG".to_string(), None));
    let tags_guard = tags.lock().expect("tag lock");
    let value = tags_guard.get(&tag_name).cloned().unwrap_or(TagValue::Dint(0));
    build_value_response(value, element_index)
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

fn handle_write(payload: &[u8], tags: &Arc<Mutex<HashMap<String, TagValue>>>) {
    let cip_request = extract_cip_request(payload);
    if cip_request.len() < 6 {
        return;
    }

    let (tag_name, element_index) = match parse_tag_and_path(&cip_request) {
        Some(value) => value,
        None => return,
    };

    let path_words = cip_request[1] as usize;
    let path_bytes = path_words * 2;
    let path_end = 2 + path_bytes;
    if cip_request.len() < path_end + 4 {
        return;
    }

    let data_type = u16::from_le_bytes([cip_request[path_end], cip_request[path_end + 1]]);
    let data_start = path_end + 4;

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
            if cip_request.len() < data_start + 4 {
                None
            } else {
                let length = u32::from_le_bytes([
                    cip_request[data_start],
                    cip_request[data_start + 1],
                    cip_request[data_start + 2],
                    cip_request[data_start + 3],
                ]) as usize;
                let string_start = data_start + 4;
                if cip_request.len() < string_start + length {
                    None
                } else {
                    let raw = &cip_request[string_start..string_start + length];
                    Some(TagValue::String(String::from_utf8_lossy(raw).to_string()))
                }
            }
        }
        _ => None,
    };

    let Some(value) = value else {
        return;
    };

    let mut tags = tags.lock().expect("tag lock");
    if let Some(index) = element_index {
        if let Some(TagValue::Array(items)) = tags.get_mut(&tag_name) {
            if index < items.len() {
                items[index] = value;
                return;
            }
        }
    }

    tags.insert(tag_name, value);
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

fn build_value_response(value: TagValue, element_index: Option<usize>) -> Vec<u8> {
    match value {
        TagValue::Array(items) => {
            if let Some(index) = element_index {
                if let Some(item) = items.get(index) {
                    return build_value_response(item.clone(), None);
                }
            }
            build_value_response(TagValue::Dint(0), None)
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
            response.extend_from_slice(&CIP_TYPE_STRING.to_le_bytes());
            response.extend_from_slice(&(v.len() as u32).to_le_bytes());
            response.extend_from_slice(v.as_bytes());
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
                if pos + 2 >= path.len() {
                    break;
                }
                element_index =
                    Some(u16::from_le_bytes([path[pos + 1], path[pos + 2]]) as usize);
                pos += 3;
            }
            0x2A => {
                if pos + 4 >= path.len() {
                    break;
                }
                element_index = Some(u32::from_le_bytes([
                    path[pos + 1],
                    path[pos + 2],
                    path[pos + 3],
                    path[pos + 4],
                ]) as usize);
                pos += 5;
            }
            _ => {
                pos += 1;
            }
        }
    }

    tag_name.map(|name| (name, element_index))
}
