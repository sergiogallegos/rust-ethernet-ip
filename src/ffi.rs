#![allow(clippy::missing_safety_doc)]

use crate::EipClient;
use crate::PlcValue;
use crate::RUNTIME;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_short, c_void};
use std::ptr;
use std::sync::{Mutex, MutexGuard};
use tracing;

// FFI-specific client manager using synchronous mutex
lazy_static! {
    static ref FFI_CLIENTS: Mutex<HashMap<i32, EipClient>> = Mutex::new(HashMap::new());
    static ref FFI_NEXT_ID: Mutex<i32> = Mutex::new(1);
}

fn to_c_string_owned(value: &str) -> Result<*mut c_char, ()> {
    CString::new(value).map(|s| s.into_raw()).map_err(|_| ())
}

fn lock_clients() -> Result<MutexGuard<'static, HashMap<i32, EipClient>>, ()> {
    FFI_CLIENTS.lock().map_err(|_| ())
}

fn lock_next_id() -> Result<MutexGuard<'static, i32>, ()> {
    FFI_NEXT_ID.lock().map_err(|_| ())
}

unsafe fn free_c_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}

fn write_output_buffer(output: *mut c_char, capacity: c_int, payload: &str) -> Result<(), ()> {
    if output.is_null() || capacity <= 0 {
        return Err(());
    }

    let bytes = payload.as_bytes();
    if bytes.len() + 1 > capacity as usize {
        return Err(());
    }

    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), output as *mut u8, bytes.len());
        *output.add(bytes.len()) = 0;
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct FfiWriteRequestItem {
    tag_name: String,
    value_type: String,
    value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct FfiExecuteRequestItem {
    tag_name: String,
    is_write: bool,
    value_type: Option<String>,
    value: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct FfiReadResultItem {
    tag_name: String,
    success: bool,
    value: Option<serde_json::Value>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct FfiWriteResultItem {
    tag_name: String,
    success: bool,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct FfiExecuteResultItem {
    index: usize,
    tag_name: String,
    is_write: bool,
    success: bool,
    value: Option<PlcValue>,
    error: Option<String>,
    execution_time_us: u64,
}

fn parse_plc_value(value_type: &str, value: serde_json::Value) -> Result<PlcValue, String> {
    let normalized = value_type.to_ascii_uppercase();

    match normalized.as_str() {
        "BOOL" => value
            .as_bool()
            .map(PlcValue::Bool)
            .ok_or_else(|| "Expected BOOL as JSON boolean".to_string()),
        "SINT" => value
            .as_i64()
            .and_then(|v| i8::try_from(v).ok())
            .map(PlcValue::Sint)
            .ok_or_else(|| "Expected SINT as JSON integer in [-128,127]".to_string()),
        "INT" => value
            .as_i64()
            .and_then(|v| i16::try_from(v).ok())
            .map(PlcValue::Int)
            .ok_or_else(|| "Expected INT as JSON integer in [-32768,32767]".to_string()),
        "DINT" => value
            .as_i64()
            .and_then(|v| i32::try_from(v).ok())
            .map(PlcValue::Dint)
            .ok_or_else(|| "Expected DINT as JSON integer in i32 range".to_string()),
        "LINT" => value
            .as_i64()
            .map(PlcValue::Lint)
            .ok_or_else(|| "Expected LINT as JSON integer in i64 range".to_string()),
        "USINT" => value
            .as_u64()
            .and_then(|v| u8::try_from(v).ok())
            .map(PlcValue::Usint)
            .ok_or_else(|| "Expected USINT as JSON integer in [0,255]".to_string()),
        "UINT" => value
            .as_u64()
            .and_then(|v| u16::try_from(v).ok())
            .map(PlcValue::Uint)
            .ok_or_else(|| "Expected UINT as JSON integer in [0,65535]".to_string()),
        "UDINT" => value
            .as_u64()
            .and_then(|v| u32::try_from(v).ok())
            .map(PlcValue::Udint)
            .ok_or_else(|| "Expected UDINT as JSON integer in u32 range".to_string()),
        "ULINT" => value
            .as_u64()
            .map(PlcValue::Ulint)
            .ok_or_else(|| "Expected ULINT as JSON integer in u64 range".to_string()),
        "REAL" => value
            .as_f64()
            .map(|v| PlcValue::Real(v as f32))
            .ok_or_else(|| "Expected REAL as JSON number".to_string()),
        "LREAL" => value
            .as_f64()
            .map(PlcValue::Lreal)
            .ok_or_else(|| "Expected LREAL as JSON number".to_string()),
        "STRING" => value
            .as_str()
            .map(|v| PlcValue::String(v.to_string()))
            .ok_or_else(|| "Expected STRING as JSON string".to_string()),
        "UDT" => serde_json::from_value::<crate::UdtData>(value)
            .map(PlcValue::Udt)
            .map_err(|e| format!("Expected UDT object {{symbol_id,data}}: {e}")),
        _ => Err(format!("Unsupported value_type: {value_type}")),
    }
}

/// Connect to a PLC and return a client ID
///
/// # Safety
///
/// This function is unsafe because:
/// - `ip_address` must be a valid null-terminated C string pointer
/// - The caller must ensure the pointer remains valid for the duration of the call
/// - The string must contain a valid IP address format
#[no_mangle]
pub unsafe extern "C" fn eip_connect(ip_address: *const c_char) -> c_int {
    if ip_address.is_null() {
        return -1;
    }

    let Ok(ip_str) = unsafe { CStr::from_ptr(ip_address) }.to_str() else {
        return -1;
    };

    let Ok(client) = RUNTIME.block_on(EipClient::new(ip_str)) else {
        return -1;
    };

    let client_id = {
        let mut next_id = match lock_next_id() {
            Ok(guard) => guard,
            Err(_) => return -1,
        };
        let id = *next_id;
        *next_id = next_id.wrapping_add(1);
        if *next_id < 1 {
            *next_id = 1;
        }
        id
    };

    {
        let mut clients = match lock_clients() {
            Ok(guard) => guard,
            Err(_) => return -1,
        };
        clients.insert(client_id, client);
    }

    client_id
}

/// Connect to a PLC with route path (for ControlLogix)
///
/// # Safety
///
/// This function is unsafe because:
/// - `ip_address` must be a valid null-terminated C string pointer
/// - `slots` must be a valid pointer to an array of `slot_count` bytes
/// - The caller must ensure all pointers remain valid for the duration of the call
#[no_mangle]
pub unsafe extern "C" fn eip_connect_with_route(
    ip_address: *const c_char,
    slots: *const u8,
    slot_count: c_int,
    ports: *const u8,
    port_count: c_int,
    addresses: *mut *const c_char,
    address_count: c_int,
) -> c_int {
    if ip_address.is_null() {
        return -1;
    }

    let Ok(ip_str) = unsafe { CStr::from_ptr(ip_address) }.to_str() else {
        return -1;
    };

    // Build route path
    let mut route_path = crate::RoutePath::new();

    // Add slots
    if !slots.is_null() && slot_count > 0 {
        let slots_slice = unsafe { std::slice::from_raw_parts(slots, slot_count as usize) };
        for &slot in slots_slice {
            route_path = route_path.add_slot(slot);
        }
    }

    // Add ports
    if !ports.is_null() && port_count > 0 {
        let ports_slice = unsafe { std::slice::from_raw_parts(ports, port_count as usize) };
        for &port in ports_slice {
            route_path = route_path.add_port(port);
        }
    }

    // Add addresses
    if !addresses.is_null() && address_count > 0 {
        let addresses_slice =
            unsafe { std::slice::from_raw_parts(addresses, address_count as usize) };
        for &addr_ptr in addresses_slice {
            if !addr_ptr.is_null() {
                if let Ok(addr_str) = unsafe { CStr::from_ptr(addr_ptr) }.to_str() {
                    route_path = route_path.add_address(addr_str.to_string());
                }
            }
        }
    }

    let Ok(client) = RUNTIME.block_on(crate::EipClient::with_route_path(ip_str, route_path)) else {
        return -1;
    };

    let client_id = {
        let mut next_id = match lock_next_id() {
            Ok(guard) => guard,
            Err(_) => return -1,
        };
        let id = *next_id;
        *next_id = next_id.wrapping_add(1);
        if *next_id < 1 {
            *next_id = 1;
        }
        id
    };

    {
        let mut clients = match lock_clients() {
            Ok(guard) => guard,
            Err(_) => return -1,
        };
        clients.insert(client_id, client);
    }

    client_id
}

/// Set route path for an existing client connection
///
/// # Safety
///
/// This function is unsafe because:
/// - `client_id` must be a valid client ID returned from `eip_connect`
/// - `slots` must be a valid pointer to an array of `slot_count` bytes
/// - The caller must ensure all pointers remain valid for the duration of the call
#[no_mangle]
pub unsafe extern "C" fn eip_set_route_path(
    client_id: c_int,
    slots: *const u8,
    slot_count: c_int,
    ports: *const u8,
    port_count: c_int,
    addresses: *mut *const c_char,
    address_count: c_int,
) -> c_int {
    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    let client = match clients.get_mut(&client_id) {
        Some(c) => c,
        None => return -1,
    };

    // Build route path
    let mut route_path = crate::RoutePath::new();

    // Add slots
    if !slots.is_null() && slot_count > 0 {
        let slots_slice = unsafe { std::slice::from_raw_parts(slots, slot_count as usize) };
        for &slot in slots_slice {
            route_path = route_path.add_slot(slot);
        }
    }

    // Add ports
    if !ports.is_null() && port_count > 0 {
        let ports_slice = unsafe { std::slice::from_raw_parts(ports, port_count as usize) };
        for &port in ports_slice {
            route_path = route_path.add_port(port);
        }
    }

    // Add addresses
    if !addresses.is_null() && address_count > 0 {
        let addresses_slice =
            unsafe { std::slice::from_raw_parts(addresses, address_count as usize) };
        for &addr_ptr in addresses_slice {
            if !addr_ptr.is_null() {
                if let Ok(addr_str) = unsafe { CStr::from_ptr(addr_ptr) }.to_str() {
                    route_path = route_path.add_address(addr_str.to_string());
                }
            }
        }
    }

    client.set_route_path(route_path);
    0
}

/// Disconnect from a PLC
///
/// # Safety
///
/// This function is unsafe because:
/// - `client_id` must be a valid client ID returned from `eip_connect`
/// - The caller must not use the `client_id` after this call
#[no_mangle]
pub unsafe extern "C" fn eip_disconnect(client_id: c_int) -> c_int {
    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.remove(&client_id) {
        Some(_) => 0,
        None => -1,
    }
}

/// Read a boolean tag
///
/// # Safety
///
/// This function is unsafe because:
/// - `tag_name` must be a valid null-terminated C string pointer
/// - `result` must be a valid mutable pointer to a `c_int`
/// - The caller must ensure both pointers remain valid for the duration of the call
/// - `client_id` must be a valid client ID returned from `eip_connect`
#[no_mangle]
pub unsafe extern "C" fn eip_read_bool(
    client_id: c_int,
    tag_name: *const c_char,
    result: *mut c_int,
) -> c_int {
    if tag_name.is_null() || result.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => match RUNTIME.block_on(client.read_tag(tag_name_str)) {
            Ok(PlcValue::Bool(value)) => {
                unsafe {
                    *result = i32::from(value);
                }
                0
            }
            _ => -1,
        },
        None => -1,
    }
}

/// Write a boolean tag
///
/// # Safety
///
/// This function is unsafe because:
/// - `tag_name` must be a valid null-terminated C string pointer
/// - The caller must ensure the pointer remains valid for the duration of the call
/// - `client_id` must be a valid client ID returned from `eip_connect`
/// - The tag name must be a valid PLC tag identifier
#[no_mangle]
pub unsafe extern "C" fn eip_write_bool(
    client_id: c_int,
    tag_name: *const c_char,
    value: c_int,
) -> c_int {
    if tag_name.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => {
            let bool_value = value != 0;
            if RUNTIME
                .block_on(client.write_tag(tag_name_str, PlcValue::Bool(bool_value)))
                .is_ok()
            {
                0
            } else {
                -1
            }
        }
        None => -1,
    }
}

// SINT (8-bit signed integer) operations
/// Read a signed 8-bit integer tag
///
/// # Safety
///
/// This function is unsafe because:
/// - `tag_name` must be a valid null-terminated C string pointer
/// - `result` must be a valid mutable pointer to an i8
/// - The caller must ensure both pointers remain valid for the duration of the call
/// - `client_id` must be a valid client ID returned from `eip_connect`
#[no_mangle]
pub unsafe extern "C" fn eip_read_sint(
    client_id: c_int,
    tag_name: *const c_char,
    result: *mut i8,
) -> c_int {
    if tag_name.is_null() || result.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => match RUNTIME.block_on(client.read_tag(tag_name_str)) {
            Ok(PlcValue::Sint(value)) => {
                unsafe {
                    *result = value;
                }
                0
            }
            _ => -1,
        },
        None => -1,
    }
}

/// Write a signed 8-bit integer tag
///
/// # Safety
///
/// This function is unsafe because:
/// - `tag_name` must be a valid null-terminated C string pointer
/// - The caller must ensure the pointer remains valid for the duration of the call
/// - `client_id` must be a valid client ID returned from `eip_connect`
#[no_mangle]
pub unsafe extern "C" fn eip_write_sint(
    client_id: c_int,
    tag_name: *const c_char,
    value: i8,
) -> c_int {
    if tag_name.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => {
            if RUNTIME
                .block_on(client.write_tag(tag_name_str, PlcValue::Sint(value)))
                .is_ok()
            {
                0
            } else {
                -1
            }
        }
        None => -1,
    }
}

// INT (16-bit signed integer) operations
#[no_mangle]
pub unsafe extern "C" fn eip_read_int(
    client_id: c_int,
    tag_name: *const c_char,
    result: *mut i16,
) -> c_int {
    if tag_name.is_null() || result.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => match RUNTIME.block_on(client.read_tag(tag_name_str)) {
            Ok(PlcValue::Int(value)) => {
                unsafe {
                    *result = value;
                }
                0
            }
            _ => -1,
        },
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn eip_write_int(
    client_id: c_int,
    tag_name: *const c_char,
    value: i16,
) -> c_int {
    if tag_name.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => {
            match RUNTIME.block_on(client.write_tag(tag_name_str, PlcValue::Int(value))) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        None => -1,
    }
}

/// Read a DINT tag
#[no_mangle]
pub unsafe extern "C" fn eip_read_dint(
    client_id: c_int,
    tag_name: *const c_char,
    result: *mut c_int,
) -> c_int {
    if tag_name.is_null() || result.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => match RUNTIME.block_on(client.read_tag(tag_name_str)) {
            Ok(PlcValue::Dint(value)) => {
                unsafe {
                    *result = value;
                }
                0
            }
            Ok(other_value) => {
                tracing::error!("[FFI] Expected DINT but got: {:?}", other_value);
                -1
            }
            Err(e) => {
                tracing::error!("[FFI] Read tag '{}' failed: {}", tag_name_str, e);
                -1
            }
        },
        None => {
            tracing::error!("[FFI] Client ID {} not found", client_id);
            -1
        }
    }
}

/// Write a DINT tag
#[no_mangle]
pub unsafe extern "C" fn eip_write_dint(
    client_id: c_int,
    tag_name: *const c_char,
    value: c_int,
) -> c_int {
    if tag_name.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => {
            match RUNTIME.block_on(client.write_tag(tag_name_str, PlcValue::Dint(value))) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        None => -1,
    }
}

// LINT (64-bit signed integer) operations
#[no_mangle]
pub unsafe extern "C" fn eip_read_lint(
    client_id: c_int,
    tag_name: *const c_char,
    result: *mut i64,
) -> c_int {
    if tag_name.is_null() || result.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => match RUNTIME.block_on(client.read_tag(tag_name_str)) {
            Ok(PlcValue::Lint(value)) => {
                unsafe {
                    *result = value;
                }
                0
            }
            _ => -1,
        },
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn eip_write_lint(
    client_id: c_int,
    tag_name: *const c_char,
    value: i64,
) -> c_int {
    if tag_name.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => {
            match RUNTIME.block_on(client.write_tag(tag_name_str, PlcValue::Lint(value))) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        None => -1,
    }
}

// USINT (8-bit unsigned integer) operations
#[no_mangle]
pub unsafe extern "C" fn eip_read_usint(
    client_id: c_int,
    tag_name: *const c_char,
    result: *mut u8,
) -> c_int {
    if tag_name.is_null() || result.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => match RUNTIME.block_on(client.read_tag(tag_name_str)) {
            Ok(PlcValue::Usint(value)) => {
                unsafe {
                    *result = value;
                }
                0
            }
            _ => -1,
        },
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn eip_write_usint(
    client_id: c_int,
    tag_name: *const c_char,
    value: u8,
) -> c_int {
    if tag_name.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => {
            match RUNTIME.block_on(client.write_tag(tag_name_str, PlcValue::Usint(value))) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        None => -1,
    }
}

// UINT (16-bit unsigned integer) operations
#[no_mangle]
pub unsafe extern "C" fn eip_read_uint(
    client_id: c_int,
    tag_name: *const c_char,
    result: *mut u16,
) -> c_int {
    if tag_name.is_null() || result.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => match RUNTIME.block_on(client.read_tag(tag_name_str)) {
            Ok(PlcValue::Uint(value)) => {
                unsafe {
                    *result = value;
                }
                0
            }
            _ => -1,
        },
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn eip_write_uint(
    client_id: c_int,
    tag_name: *const c_char,
    value: u16,
) -> c_int {
    if tag_name.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => {
            match RUNTIME.block_on(client.write_tag(tag_name_str, PlcValue::Uint(value))) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        None => -1,
    }
}

// UDINT (32-bit unsigned integer) operations
#[no_mangle]
pub unsafe extern "C" fn eip_read_udint(
    client_id: c_int,
    tag_name: *const c_char,
    result: *mut u32,
) -> c_int {
    if tag_name.is_null() || result.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => match RUNTIME.block_on(client.read_tag(tag_name_str)) {
            Ok(PlcValue::Udint(value)) => {
                unsafe {
                    *result = value;
                }
                0
            }
            _ => -1,
        },
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn eip_write_udint(
    client_id: c_int,
    tag_name: *const c_char,
    value: u32,
) -> c_int {
    if tag_name.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => {
            match RUNTIME.block_on(client.write_tag(tag_name_str, PlcValue::Udint(value))) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        None => -1,
    }
}

// ULINT (64-bit unsigned integer) operations
#[no_mangle]
pub unsafe extern "C" fn eip_read_ulint(
    client_id: c_int,
    tag_name: *const c_char,
    result: *mut u64,
) -> c_int {
    if tag_name.is_null() || result.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => match RUNTIME.block_on(client.read_tag(tag_name_str)) {
            Ok(PlcValue::Ulint(value)) => {
                unsafe {
                    *result = value;
                }
                0
            }
            _ => -1,
        },
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn eip_write_ulint(
    client_id: c_int,
    tag_name: *const c_char,
    value: u64,
) -> c_int {
    if tag_name.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => {
            if RUNTIME
                .block_on(client.write_tag(tag_name_str, PlcValue::Ulint(value)))
                .is_ok()
            {
                0
            } else {
                -1
            }
        }
        None => -1,
    }
}

/// Read a REAL tag
#[no_mangle]
pub unsafe extern "C" fn eip_read_real(
    client_id: c_int,
    tag_name: *const c_char,
    result: *mut f64,
) -> c_int {
    if tag_name.is_null() || result.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => match RUNTIME.block_on(client.read_tag(tag_name_str)) {
            Ok(PlcValue::Real(value)) => {
                unsafe {
                    *result = f64::from(value);
                }
                0
            }
            _ => -1,
        },
        None => -1,
    }
}

/// Write a REAL tag
#[no_mangle]
pub unsafe extern "C" fn eip_write_real(
    client_id: c_int,
    tag_name: *const c_char,
    value: f64,
) -> c_int {
    if tag_name.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => {
            match RUNTIME.block_on(client.write_tag(tag_name_str, PlcValue::Real(value as f32))) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        None => -1,
    }
}

// LREAL (64-bit double precision) operations
#[no_mangle]
pub unsafe extern "C" fn eip_read_lreal(
    client_id: c_int,
    tag_name: *const c_char,
    result: *mut f64,
) -> c_int {
    if tag_name.is_null() || result.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => match RUNTIME.block_on(client.read_tag(tag_name_str)) {
            Ok(PlcValue::Lreal(value)) => {
                unsafe {
                    *result = value;
                }
                0
            }
            _ => -1,
        },
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn eip_write_lreal(
    client_id: c_int,
    tag_name: *const c_char,
    value: f64,
) -> c_int {
    if tag_name.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    match clients.get_mut(&client_id) {
        Some(client) => {
            if RUNTIME
                .block_on(client.write_tag(tag_name_str, PlcValue::Lreal(value)))
                .is_ok()
            {
                0
            } else {
                -1
            }
        }
        None => -1,
    }
}

/// Read a STRING tag
///
/// # Safety
///
/// This function is unsafe because:
/// - `tag_name` must be a valid null-terminated C string pointer
/// - `result` must be a valid mutable pointer to a buffer of at least `max_length` bytes
/// - The caller must ensure both pointers remain valid for the duration of the call
/// - `client_id` must be a valid client ID returned from `eip_connect`
/// - `max_length` must be positive and represent the actual buffer size
#[no_mangle]
pub unsafe extern "C" fn eip_read_string(
    client_id: c_int,
    tag_name: *const c_char,
    result: *mut c_char,
    max_length: c_int,
) -> c_int {
    if tag_name.is_null() || result.is_null() || max_length <= 0 {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match FFI_CLIENTS.lock() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    let Some(client) = clients.get_mut(&client_id) else {
        return -1;
    };

    let value = match RUNTIME.block_on(client.read_tag(tag_name_str)) {
        Ok(PlcValue::String(value)) => {
            tracing::info!(
                "[FFI] Read STRING tag '{}' succeeded: '{}'",
                tag_name_str,
                value
            );
            value
        }
        Ok(PlcValue::Udt(udt_data)) => {
            // Allen-Bradley STRING tags are returned as UDT structures (0x02A0)
            // STRING UDT format: 4-byte length (DINT) followed by string data (up to 82 bytes)
            tracing::warn!("[FFI] STRING tag '{}' returned as UDT, attempting to extract string from UDT data ({} bytes): {:02X?}",
                tag_name_str, udt_data.data.len(),
                &udt_data.data[..std::cmp::min(20, udt_data.data.len())]);

            // Allen-Bradley STRING UDT format when returned as 0x02A0:
            // The UDT data might have a header before the actual STRING data
            // Try multiple formats:
            // Format 1: Direct STRING format - 4-byte length (DINT) at start
            // Format 2: UDT wrapper - might have type code (0x0FCE) + length (2 bytes) + data
            // Format 3: Just find the string data by looking for printable ASCII

            tracing::debug!(
                "[FFI] Attempting to parse STRING from UDT data ({} bytes)",
                udt_data.data.len()
            );

            // Try Format 1: Standard STRING format (4-byte DINT length at start)
            if udt_data.data.len() >= 4 {
                let length = u32::from_le_bytes([
                    udt_data.data[0],
                    udt_data.data[1],
                    udt_data.data[2],
                    udt_data.data[3],
                ]) as usize;

                // If length is reasonable (<= 82 for STRING), try this format
                if length <= 82 && udt_data.data.len() >= 4 + length {
                    let string_data = &udt_data.data[4..4 + length];
                    let trimmed_data: Vec<u8> = string_data
                        .iter()
                        .take_while(|&&b| b != 0)
                        .copied()
                        .collect();
                    if let Ok(s) = String::from_utf8(trimmed_data) {
                        tracing::info!("[FFI] Extracted STRING (Format 1): '{}'", s);
                        unsafe {
                            let Ok(c_string) = CString::new(s) else {
                                return -1;
                            };
                            let bytes = c_string.as_bytes_with_nul();
                            if bytes.len() > max_length as usize {
                                return -1;
                            }
                            ptr::copy_nonoverlapping(
                                bytes.as_ptr(),
                                result as *mut u8,
                                bytes.len(),
                            );
                            return 0;
                        }
                    }
                }

                // Try Format 2: UDT wrapper with type code (bytes 0-1) + length (bytes 2-3) + padding (bytes 4-5) + data (bytes 6+)
                if udt_data.data.len() >= 6 {
                    let length = u16::from_le_bytes([udt_data.data[2], udt_data.data[3]]) as usize;
                    // Check if length is reasonable and we have enough data
                    if length > 0 && length <= 82 && udt_data.data.len() >= 6 + length {
                        let string_data = &udt_data.data[6..6 + length];
                        let trimmed_data: Vec<u8> = string_data
                            .iter()
                            .take_while(|&&b| b != 0)
                            .copied()
                            .collect();
                        if let Ok(s) = String::from_utf8(trimmed_data) {
                            if !s.is_empty() {
                                tracing::info!(
                                    "[FFI] Extracted STRING (Format 2): '{}' (length={})",
                                    s,
                                    length
                                );
                                unsafe {
                                    let Ok(c_string) = CString::new(s) else {
                                        return -1;
                                    };
                                    let bytes = c_string.as_bytes_with_nul();
                                    if bytes.len() > max_length as usize {
                                        return -1;
                                    }
                                    ptr::copy_nonoverlapping(
                                        bytes.as_ptr(),
                                        result as *mut u8,
                                        bytes.len(),
                                    );
                                    return 0;
                                }
                            }
                        }
                    }
                }

                // Try Format 3: Find string data by scanning for printable ASCII
                // Look for the first sequence of printable ASCII characters
                for start in 0..std::cmp::min(udt_data.data.len() - 1, 20) {
                    if udt_data.data[start].is_ascii() && !udt_data.data[start].is_ascii_control() {
                        let mut end = start;
                        while end < udt_data.data.len()
                            && udt_data.data[end] != 0
                            && udt_data.data[end].is_ascii()
                        {
                            end += 1;
                        }
                        if end > start {
                            let string_data = &udt_data.data[start..end];
                            if let Ok(s) = String::from_utf8(string_data.to_vec()) {
                                tracing::info!(
                                    "[FFI] Extracted STRING (Format 3, scanned): '{}'",
                                    s
                                );
                                unsafe {
                                    let Ok(c_string) = CString::new(s) else {
                                        return -1;
                                    };
                                    let bytes = c_string.as_bytes_with_nul();
                                    if bytes.len() > max_length as usize {
                                        return -1;
                                    }
                                    ptr::copy_nonoverlapping(
                                        bytes.as_ptr(),
                                        result as *mut u8,
                                        bytes.len(),
                                    );
                                    return 0;
                                }
                            }
                        }
                    }
                }
            }

            tracing::error!(
                "[FFI] Could not extract STRING from UDT data for tag '{}'",
                tag_name_str
            );
            return -1;
        }
        Ok(other) => {
            tracing::error!(
                "[FFI] Expected STRING for tag '{}' but got: {:?}",
                tag_name_str,
                std::mem::discriminant(&other)
            );
            return -1; // Wrong data type
        }
        Err(e) => {
            tracing::error!("[FFI] Read STRING tag '{}' failed: {}", tag_name_str, e);
            return -1; // Error reading tag
        }
    };

    let Ok(c_string) = CString::new(value) else {
        return -1;
    };

    let bytes = c_string.as_bytes_with_nul();
    if bytes.len() > max_length as usize {
        return -1; // String too long
    }

    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), result as *mut u8, bytes.len());
    }
    0
}

/// Write a STRING tag
#[no_mangle]
pub unsafe extern "C" fn eip_write_string(
    client_id: c_int,
    tag_name: *const c_char,
    value: *const c_char,
) -> c_int {
    if tag_name.is_null() || value.is_null() {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let Ok(value_str) = unsafe { CStr::from_ptr(value) }.to_str() else {
        return -1;
    };

    let mut clients = match FFI_CLIENTS.lock() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    let Some(client) = clients.get_mut(&client_id) else {
        return -1;
    };

    if RUNTIME
        .block_on(client.write_tag(tag_name_str, PlcValue::String(value_str.to_string())))
        .is_ok()
    {
        0
    } else {
        -1
    }
}

/// Read any tag and return as JSON (generic read function)
///
/// This function reads any tag type and returns it as JSON, allowing the caller
/// to determine the type dynamically. This is useful for complex paths like
/// UDT array element members (e.g., "gTestUDT_Array[0].Member1_DINT").
///
/// # Safety
///
/// This function is unsafe because:
/// - `tag_name` must be a valid null-terminated C string pointer
/// - `result` must be a valid mutable pointer to a buffer of at least `max_size` bytes
/// - The caller must ensure both pointers remain valid for the duration of the call
/// - `client_id` must be a valid client ID returned from `eip_connect`
#[no_mangle]
pub unsafe extern "C" fn eip_read_tag(
    client_id: c_int,
    tag_name: *const c_char,
    result: *mut c_char,
    max_size: c_int,
) -> c_int {
    if tag_name.is_null() || result.is_null() || max_size <= 0 {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    let Some(client) = clients.get_mut(&client_id) else {
        tracing::error!("[FFI] Client ID {} not found", client_id);
        return -1;
    };

    let value = match RUNTIME.block_on(client.read_tag(tag_name_str)) {
        Ok(value) => {
            tracing::info!(
                "[FFI] Read tag '{}' succeeded, type: {:?}",
                tag_name_str,
                std::mem::discriminant(&value)
            );
            value
        }
        Err(e) => {
            tracing::error!("[FFI] Read tag '{}' failed: {}", tag_name_str, e);
            return -1;
        }
    };

    // Serialize PlcValue to JSON for C# consumption
    let json_result = match serde_json::to_string(&value) {
        Ok(json) => json,
        Err(e) => {
            tracing::error!(
                "[FFI] Failed to serialize tag '{}' to JSON: {}",
                tag_name_str,
                e
            );
            return -1;
        }
    };

    let Ok(c_string) = CString::new(json_result) else {
        tracing::error!("[FFI] Failed to create C string for tag '{}'", tag_name_str);
        return -1;
    };

    let bytes = c_string.as_bytes_with_nul();
    if bytes.len() > max_size as usize {
        tracing::error!(
            "[FFI] JSON result too long for tag '{}': {} bytes (max: {})",
            tag_name_str,
            bytes.len(),
            max_size
        );
        return -1; // JSON too long
    }

    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), result as *mut u8, bytes.len());
    }
    0
}

/// Read a range of array elements as JSON array of PlcValue.
///
/// The JSON output format is a Rust enum array, for example:
/// `[{"Dint":10},{"Dint":20}]`
///
/// # Safety
///
/// This function is unsafe because:
/// - `base_array_name` must be a valid null-terminated C string pointer
/// - `result` must be a valid mutable pointer to a buffer of at least `max_size` bytes
/// - The caller must ensure both pointers remain valid for the duration of the call
#[no_mangle]
pub unsafe extern "C" fn eip_read_array_range(
    client_id: c_int,
    base_array_name: *const c_char,
    start_index: c_int,
    element_count: c_int,
    result: *mut c_char,
    max_size: c_int,
) -> c_int {
    if base_array_name.is_null()
        || result.is_null()
        || max_size <= 0
        || start_index < 0
        || element_count <= 0
    {
        return -1;
    }

    let Ok(base_array_name_str) = unsafe { CStr::from_ptr(base_array_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    let Some(client) = clients.get_mut(&client_id) else {
        return -1;
    };

    let values = match RUNTIME.block_on(client.read_array_range(
        base_array_name_str,
        start_index as u32,
        element_count as u32,
    )) {
        Ok(values) => values,
        Err(_) => return -1,
    };

    let json_result = match serde_json::to_string(&values) {
        Ok(json) => json,
        Err(_) => return -1,
    };

    let Ok(c_string) = CString::new(json_result) else {
        return -1;
    };

    let bytes = c_string.as_bytes_with_nul();
    if bytes.len() > max_size as usize {
        return -1;
    }

    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), result as *mut u8, bytes.len());
    }
    0
}

// UDT operations
#[no_mangle]
pub unsafe extern "C" fn eip_read_udt(
    client_id: c_int,
    tag_name: *const c_char,
    result: *mut c_char,
    max_size: c_int,
) -> c_int {
    if tag_name.is_null() || result.is_null() || max_size <= 0 {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match FFI_CLIENTS.lock() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    let Some(client) = clients.get_mut(&client_id) else {
        return -1;
    };

    let value = match RUNTIME.block_on(client.read_udt_chunked(tag_name_str)) {
        Ok(PlcValue::Udt(udt_data)) => udt_data,
        Ok(_) => return -1,  // Wrong data type
        Err(_) => return -1, // Error reading tag
    };

    // Serialize UDT to JSON for C# consumption
    let json_result = match serde_json::to_string(&value) {
        Ok(json) => json,
        Err(_) => return -1,
    };

    let Ok(c_string) = CString::new(json_result) else {
        return -1;
    };

    let bytes = c_string.as_bytes_with_nul();
    if bytes.len() > max_size as usize {
        return -1; // JSON too long
    }

    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), result as *mut u8, bytes.len());
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn eip_write_udt(
    client_id: c_int,
    tag_name: *const c_char,
    value: *const c_char,
    size: c_int,
) -> c_int {
    if tag_name.is_null() || value.is_null() || size <= 0 {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let Ok(value_str) = unsafe { CStr::from_ptr(value) }.to_str() else {
        return -1;
    };

    // Deserialize JSON to UDT (HashMap format for backward compatibility)
    let udt_members: HashMap<String, PlcValue> = match serde_json::from_str(value_str) {
        Ok(data) => data,
        Err(_) => return -1,
    };

    let mut clients = match FFI_CLIENTS.lock() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    let Some(client) = clients.get_mut(&client_id) else {
        return -1;
    };

    // Convert HashMap to UdtData format
    // First, read the tag to get symbol_id and UDT definition
    let udt_data = match RUNTIME.block_on(async {
        // Read tag to get symbol_id
        let read_value = client.read_tag(tag_name_str).await?;
        let existing_udt = if let PlcValue::Udt(data) = read_value {
            data
        } else {
            return Err(crate::error::EtherNetIpError::Protocol(
                "Tag is not a UDT".to_string(),
            ));
        };

        // Get UDT definition to serialize HashMap to bytes
        let udt_def = client.get_udt_definition(tag_name_str).await?;

        // Convert UdtDefinition to UserDefinedType
        let mut user_def = crate::udt::UserDefinedType::new(udt_def.name.clone());
        for member in &udt_def.members {
            user_def.add_member(member.clone());
        }

        // Convert HashMap to UdtData using the definition
        crate::UdtData::from_hash_map(&udt_members, &user_def, existing_udt.symbol_id)
    }) {
        Ok(data) => data,
        Err(_) => {
            // Fallback: create UdtData with symbol_id=0 (will trigger auto-read in write_tag)
            // Note: This won't work perfectly without UDT definition, but write_tag will try
            crate::UdtData {
                symbol_id: 0,
                data: vec![], // Empty - write_tag will need to handle this
            }
        }
    };

    if RUNTIME
        .block_on(client.write_tag(tag_name_str, PlcValue::Udt(udt_data)))
        .is_ok()
    {
        0
    } else {
        -1
    }
}

// Tag discovery and metadata
#[no_mangle]
pub unsafe extern "C" fn eip_discover_tags(_client_id: c_int) -> c_int {
    // Return success for now - can implement tag discovery later
    0
}

#[no_mangle]
pub unsafe extern "C" fn eip_get_tag_metadata(
    _client_id: c_int,
    _tag_name: *const c_char,
    _metadata: *mut u8,
) -> c_int {
    // For now, return error - metadata support can be added later
    -1
}

// Configuration
#[no_mangle]
pub unsafe extern "C" fn eip_set_max_packet_size(_client_id: c_int, _size: c_int) -> c_int {
    // Return success for now - packet size configuration can be added later
    0
}

// Health checks
#[no_mangle]
pub unsafe extern "C" fn eip_check_health(client_id: c_int, is_healthy: *mut c_int) -> c_int {
    if is_healthy.is_null() {
        return -1;
    }

    let clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    if clients.get(&client_id).is_some() {
        unsafe {
            *is_healthy = 1;
        }
        0
    } else {
        unsafe {
            *is_healthy = 0;
        }
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn eip_check_health_detailed(
    client_id: c_int,
    is_healthy: *mut c_int,
) -> c_int {
    // Use the same logic as basic health check for now
    eip_check_health(client_id, is_healthy)
}

// Batch operations implementation
#[no_mangle]
pub unsafe extern "C" fn eip_read_tags_batch(
    client_id: c_int,
    tag_names: *mut *const c_char,
    tag_count: c_int,
    results: *mut c_char,
    results_capacity: c_int,
) -> c_int {
    if tag_names.is_null() || results.is_null() || tag_count <= 0 || results_capacity <= 0 {
        return -1;
    }

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    let Some(client) = clients.get_mut(&client_id) else {
        return -1;
    };

    // Convert C strings to Rust strings
    let mut tag_name_strs = Vec::new();
    unsafe {
        for i in 0..tag_count {
            let tag_name_ptr = *tag_names.offset(i as isize);
            if tag_name_ptr.is_null() {
                return -1;
            }
            let Ok(tag_name) = CStr::from_ptr(tag_name_ptr).to_str() else {
                return -1;
            };
            tag_name_strs.push(tag_name);
        }
    }

    // Execute batch read
    let batch_results = RUNTIME.block_on(async { client.read_tags_batch(&tag_name_strs).await });

    let results_data = match batch_results {
        Ok(results) => {
            let response_items: Vec<FfiReadResultItem> = results
                .into_iter()
                .map(|(tag_name, result)| match result {
                    Ok(value) => FfiReadResultItem {
                        tag_name,
                        success: true,
                        value: Some(serde_json::to_value(value).unwrap_or(serde_json::Value::Null)),
                        error: None,
                    },
                    Err(e) => FfiReadResultItem {
                        tag_name,
                        success: false,
                        value: None,
                        error: Some(e.to_string()),
                    },
                })
                .collect();

            match serde_json::to_string(&response_items) {
                Ok(json) => json,
                Err(_) => return -1,
            }
        }
        Err(_) => return -1,
    };

    if write_output_buffer(results, results_capacity, &results_data).is_err() {
        return -1;
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn eip_write_tags_batch(
    client_id: c_int,
    tag_values: *const c_char,
    tag_count: c_int,
    results: *mut c_char,
    results_capacity: c_int,
) -> c_int {
    if tag_values.is_null() || results.is_null() || tag_count <= 0 || results_capacity <= 0 {
        return -1;
    }

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    let Some(client) = clients.get_mut(&client_id) else {
        return -1;
    };

    let input_str = unsafe {
        match CStr::from_ptr(tag_values).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    let request_items: Vec<FfiWriteRequestItem> = match serde_json::from_str(input_str) {
        Ok(items) => items,
        Err(_) => return -1,
    };

    if request_items.len() != tag_count as usize {
        return -1;
    }

    let mut parse_errors: HashMap<String, String> = HashMap::new();
    let mut valid_writes: Vec<(String, PlcValue)> = Vec::new();
    for item in &request_items {
        match parse_plc_value(&item.value_type, item.value.clone()) {
            Ok(value) => valid_writes.push((item.tag_name.clone(), value)),
            Err(err) => {
                parse_errors.insert(item.tag_name.clone(), err);
            }
        }
    }

    let mut write_results: HashMap<String, Result<(), String>> = HashMap::new();
    if !valid_writes.is_empty() {
        let write_refs: Vec<(&str, PlcValue)> = valid_writes
            .iter()
            .map(|(name, value)| (name.as_str(), value.clone()))
            .collect();

        match RUNTIME.block_on(client.write_tags_batch(&write_refs)) {
            Ok(results_vec) => {
                for (tag_name, result) in results_vec {
                    match result {
                        Ok(()) => {
                            write_results.insert(tag_name, Ok(()));
                        }
                        Err(e) => {
                            write_results.insert(tag_name, Err(e.to_string()));
                        }
                    }
                }
            }
            Err(e) => {
                let err = e.to_string();
                for (tag_name, _) in &valid_writes {
                    write_results.insert(tag_name.clone(), Err(err.clone()));
                }
            }
        }
    }

    let response_items: Vec<FfiWriteResultItem> = request_items
        .into_iter()
        .map(|item| {
            if let Some(err) = parse_errors.get(&item.tag_name) {
                return FfiWriteResultItem {
                    tag_name: item.tag_name,
                    success: false,
                    error: Some(err.clone()),
                };
            }

            match write_results.get(&item.tag_name) {
                Some(Ok(())) => FfiWriteResultItem {
                    tag_name: item.tag_name,
                    success: true,
                    error: None,
                },
                Some(Err(err)) => FfiWriteResultItem {
                    tag_name: item.tag_name,
                    success: false,
                    error: Some(err.clone()),
                },
                None => FfiWriteResultItem {
                    tag_name: item.tag_name,
                    success: false,
                    error: Some("Missing result for write operation".to_string()),
                },
            }
        })
        .collect();

    let results_data = match serde_json::to_string(&response_items) {
        Ok(json) => json,
        Err(_) => return -1,
    };

    if write_output_buffer(results, results_capacity, &results_data).is_err() {
        return -1;
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn eip_execute_batch(
    client_id: c_int,
    operations: *const c_char,
    operation_count: c_int,
    results: *mut c_char,
    results_capacity: c_int,
) -> c_int {
    if operations.is_null() || results.is_null() || operation_count <= 0 || results_capacity <= 0 {
        return -1;
    }

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    let Some(client) = clients.get_mut(&client_id) else {
        return -1;
    };

    let input_str = unsafe {
        match CStr::from_ptr(operations).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    let request_items: Vec<FfiExecuteRequestItem> = match serde_json::from_str(input_str) {
        Ok(items) => items,
        Err(_) => return -1,
    };

    if request_items.len() != operation_count as usize {
        return -1;
    }

    let original_batch_cfg = client.get_batch_config().clone();
    let mut sequential_cfg = original_batch_cfg.clone();
    sequential_cfg.optimize_packet_packing = false;
    client.configure_batch_operations(sequential_cfg);

    let mut operation_parse_errors: HashMap<usize, String> = HashMap::new();
    let mut valid_operations: Vec<crate::BatchOperation> = Vec::new();

    for (idx, item) in request_items.iter().enumerate() {
        if item.is_write {
            let value_type = match &item.value_type {
                Some(v) => v,
                None => {
                    operation_parse_errors
                        .insert(idx, "Missing value_type for write operation".to_string());
                    continue;
                }
            };
            let value_json = match &item.value {
                Some(v) => v.clone(),
                None => {
                    operation_parse_errors
                        .insert(idx, "Missing value for write operation".to_string());
                    continue;
                }
            };

            match parse_plc_value(value_type, value_json) {
                Ok(value) => valid_operations.push(crate::BatchOperation::Write {
                    tag_name: item.tag_name.clone(),
                    value,
                }),
                Err(err) => {
                    operation_parse_errors.insert(idx, err);
                }
            }
        } else {
            valid_operations.push(crate::BatchOperation::Read {
                tag_name: item.tag_name.clone(),
            });
        }
    }

    let batch_exec_result = if valid_operations.is_empty() {
        Ok(Vec::new())
    } else {
        RUNTIME.block_on(client.execute_batch(&valid_operations))
    };

    // Restore caller's batch config to avoid side effects from this FFI call.
    client.configure_batch_operations(original_batch_cfg);

    let mut valid_iter = match batch_exec_result {
        Ok(vec) => vec.into_iter(),
        Err(e) => {
            let error_message = e.to_string();
            let response_items: Vec<FfiExecuteResultItem> = request_items
                .into_iter()
                .enumerate()
                .map(|(idx, item)| FfiExecuteResultItem {
                    index: idx,
                    tag_name: item.tag_name,
                    is_write: item.is_write,
                    success: false,
                    value: None,
                    error: Some(error_message.clone()),
                    execution_time_us: 0,
                })
                .collect();

            let results_data = match serde_json::to_string(&response_items) {
                Ok(json) => json,
                Err(_) => return -1,
            };
            if write_output_buffer(results, results_capacity, &results_data).is_err() {
                return -1;
            }
            return -1;
        }
    };

    let response_items: Vec<FfiExecuteResultItem> = request_items
        .into_iter()
        .enumerate()
        .map(|(idx, item)| {
            if let Some(err) = operation_parse_errors.get(&idx) {
                return FfiExecuteResultItem {
                    index: idx,
                    tag_name: item.tag_name,
                    is_write: item.is_write,
                    success: false,
                    value: None,
                    error: Some(err.clone()),
                    execution_time_us: 0,
                };
            }

            let Some(batch_result) = valid_iter.next() else {
                return FfiExecuteResultItem {
                    index: idx,
                    tag_name: item.tag_name,
                    is_write: item.is_write,
                    success: false,
                    value: None,
                    error: Some("Missing batch result for operation".to_string()),
                    execution_time_us: 0,
                };
            };

            match batch_result.result {
                Ok(value_opt) => FfiExecuteResultItem {
                    index: idx,
                    tag_name: item.tag_name,
                    is_write: item.is_write,
                    success: true,
                    value: value_opt,
                    error: None,
                    execution_time_us: batch_result.execution_time_us,
                },
                Err(e) => FfiExecuteResultItem {
                    index: idx,
                    tag_name: item.tag_name,
                    is_write: item.is_write,
                    success: false,
                    value: None,
                    error: Some(e.to_string()),
                    execution_time_us: batch_result.execution_time_us,
                },
            }
        })
        .collect();

    let results_data = match serde_json::to_string(&response_items) {
        Ok(json) => json,
        Err(_) => return -1,
    };

    if write_output_buffer(results, results_capacity, &results_data).is_err() {
        return -1;
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn eip_configure_batch_operations(
    _client_id: c_int,
    _config: *const u8,
) -> c_int {
    -1 // Not implemented yet
}

#[no_mangle]
pub unsafe extern "C" fn eip_get_batch_config(_client_id: c_int, _config: *mut u8) -> c_int {
    -1 // Not implemented yet
}

// Enhanced UDT operations
#[no_mangle]
pub unsafe extern "C" fn eip_read_udt_chunked(
    client_id: c_int,
    tag_name: *const c_char,
    result: *mut c_char,
    max_size: c_int,
) -> c_int {
    if tag_name.is_null() || result.is_null() || max_size <= 0 {
        return -1;
    }

    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    let Some(client) = clients.get_mut(&client_id) else {
        return -1;
    };

    let value = match RUNTIME.block_on(client.read_udt_chunked(tag_name_str)) {
        Ok(PlcValue::Udt(udt_data)) => udt_data,
        Ok(_) => return -1,  // Wrong data type
        Err(_) => return -1, // Error reading tag
    };

    // Serialize UDT to JSON for C# consumption
    let json_result = match serde_json::to_string(&value) {
        Ok(json) => json,
        Err(_) => return -1,
    };

    let Ok(c_string) = CString::new(json_result) else {
        return -1;
    };

    let bytes = c_string.as_bytes_with_nul();
    if bytes.len() > max_size as usize {
        return -1; // JSON too long
    }

    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), result as *mut u8, bytes.len());
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn eip_read_udt_member_by_offset(
    client_id: c_int,
    udt_name: *const c_char,
    member_offset: c_int,
    member_size: c_int,
    data_type: c_short,
    result: *mut c_char,
    max_size: c_int,
) -> c_int {
    if udt_name.is_null()
        || result.is_null()
        || max_size <= 0
        || member_offset < 0
        || member_size <= 0
    {
        return -1;
    }

    let Ok(udt_name_str) = unsafe { CStr::from_ptr(udt_name) }.to_str() else {
        return -1;
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    let Some(client) = clients.get_mut(&client_id) else {
        return -1;
    };

    let value = match RUNTIME.block_on(client.read_udt_member_by_offset(
        udt_name_str,
        member_offset as usize,
        member_size as usize,
        data_type as u16,
    )) {
        Ok(value) => value,
        Err(_) => return -1,
    };

    // Serialize value to JSON for C# consumption
    let json_result = match serde_json::to_string(&value) {
        Ok(json) => json,
        Err(_) => return -1,
    };

    let Ok(c_string) = CString::new(json_result) else {
        return -1;
    };

    let bytes = c_string.as_bytes_with_nul();
    if bytes.len() > max_size as usize {
        return -1; // JSON too long
    }

    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), result as *mut u8, bytes.len());
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn eip_write_udt_member_by_offset(
    client_id: c_int,
    udt_name: *const c_char,
    member_offset: c_int,
    member_size: c_int,
    data_type: c_short,
    value: *const c_char,
    size: c_int,
) -> c_int {
    if udt_name.is_null() || value.is_null() || size <= 0 || member_offset < 0 || member_size <= 0 {
        return -1;
    }

    let Ok(udt_name_str) = unsafe { CStr::from_ptr(udt_name) }.to_str() else {
        return -1;
    };

    let Ok(value_str) = unsafe { CStr::from_ptr(value) }.to_str() else {
        return -1;
    };

    // Parse the value from JSON
    let plc_value: PlcValue = match serde_json::from_str(value_str) {
        Ok(value) => value,
        Err(_) => return -1,
    };

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    let Some(client) = clients.get_mut(&client_id) else {
        return -1;
    };

    match RUNTIME.block_on(client.write_udt_member_by_offset(
        udt_name_str,
        member_offset as usize,
        member_size as usize,
        data_type as u16,
        plc_value,
    )) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// C struct for UDT member
#[repr(C)]
pub struct UdtMemberC {
    pub name: *mut c_char,
    pub data_type: c_short,
    pub offset: c_int,
    pub size: c_int,
}

/// C struct for UDT definition result
#[repr(C)]
pub struct UdtDefinitionResult {
    pub success: bool,
    pub error_message: *mut c_char,
    pub name: *mut c_char,
    pub members: *mut UdtMemberC,
    pub member_count: c_int,
}

/// C struct for tag attributes
#[repr(C)]
pub struct TagAttributesC {
    pub name: *mut c_char,
    pub data_type_name: *mut c_char,
    pub data_type: c_short,
    pub size: c_int,
    pub template_instance_id: c_int,
}

/// C struct for tag attributes result
#[repr(C)]
pub struct TagAttributesResult {
    pub success: bool,
    pub error_message: *mut c_char,
    pub name: *mut c_char,
    pub data_type_name: *mut c_char,
    pub data_type: c_short,
    pub size: c_int,
    pub template_instance_id: c_int,
}

/// C struct for tag discovery result
#[repr(C)]
pub struct TagDiscoveryResult {
    pub success: bool,
    pub error_message: *mut c_char,
    pub tags: *mut TagAttributesC,
    pub tag_count: c_int,
}

/// Free a C string allocated by this library
#[no_mangle]
pub unsafe extern "C" fn eip_free_string(ptr: *mut c_char) {
    free_c_string(ptr);
}

/// Free a UDT definition result allocated by `eip_get_udt_definition`
#[no_mangle]
pub unsafe extern "C" fn eip_free_udt_definition(result_ptr: *mut UdtDefinitionResult) {
    if result_ptr.is_null() {
        return;
    }

    let result = unsafe { &mut *result_ptr };
    unsafe {
        free_c_string(result.error_message);
        free_c_string(result.name);
    }

    if !result.members.is_null() {
        if result.member_count > 0 {
            for i in 0..result.member_count as usize {
                unsafe {
                    let member = result.members.add(i);
                    free_c_string((*member).name);
                }
            }
        }
        unsafe {
            libc::free(result.members as *mut c_void);
        }
    }

    result.error_message = ptr::null_mut();
    result.name = ptr::null_mut();
    result.members = ptr::null_mut();
    result.member_count = 0;
}

/// Free a tag attributes result allocated by `eip_get_tag_attributes`
#[no_mangle]
pub unsafe extern "C" fn eip_free_tag_attributes_result(result_ptr: *mut TagAttributesResult) {
    if result_ptr.is_null() {
        return;
    }

    let result = unsafe { &mut *result_ptr };
    unsafe {
        free_c_string(result.error_message);
        free_c_string(result.name);
        free_c_string(result.data_type_name);
    }

    result.error_message = ptr::null_mut();
    result.name = ptr::null_mut();
    result.data_type_name = ptr::null_mut();
}

/// Free a tag discovery result allocated by `eip_discover_tags_detailed`
#[no_mangle]
pub unsafe extern "C" fn eip_free_tag_discovery_result(result_ptr: *mut TagDiscoveryResult) {
    if result_ptr.is_null() {
        return;
    }

    let result = unsafe { &mut *result_ptr };
    unsafe {
        free_c_string(result.error_message);
    }

    if !result.tags.is_null() {
        if result.tag_count > 0 {
            for i in 0..result.tag_count as usize {
                unsafe {
                    let tag = result.tags.add(i);
                    free_c_string((*tag).name);
                    free_c_string((*tag).data_type_name);
                }
            }
        }
        unsafe {
            libc::free(result.tags as *mut c_void);
        }
    }

    result.error_message = ptr::null_mut();
    result.tags = ptr::null_mut();
    result.tag_count = 0;
}

/// FFI function to get UDT definition from PLC
///
/// The caller must free the returned fields using `eip_free_udt_definition`.
#[no_mangle]
pub unsafe extern "C" fn eip_get_udt_definition(
    client_ptr: *mut EipClient,
    udt_name: *const c_char,
    result_ptr: *mut UdtDefinitionResult,
) -> c_int {
    if client_ptr.is_null() || udt_name.is_null() || result_ptr.is_null() {
        return -1;
    }

    let udt_name_cstr = unsafe { CStr::from_ptr(udt_name) };
    let udt_name_str = match udt_name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let client = unsafe { &mut *client_ptr };
    let rt = RUNTIME.handle().clone();

    match rt.block_on(client.get_udt_definition(udt_name_str)) {
        Ok(definition) => {
            unsafe {
                (*result_ptr).success = true;
                (*result_ptr).error_message = std::ptr::null_mut();
                (*result_ptr).name = std::ptr::null_mut();
                (*result_ptr).members = std::ptr::null_mut();
                (*result_ptr).member_count = 0;

                let name_ptr = match to_c_string_owned(&definition.name) {
                    Ok(ptr) => ptr,
                    Err(_) => {
                        (*result_ptr).success = false;
                        (*result_ptr).error_message = to_c_string_owned(
                            "Failed to allocate UDT name (string contains null byte)",
                        )
                        .unwrap_or(std::ptr::null_mut());
                        return -1;
                    }
                };

                // Allocate memory for members
                let members_ptr =
                    libc::malloc(std::mem::size_of::<UdtMemberC>() * definition.members.len())
                        as *mut UdtMemberC;

                if members_ptr.is_null() {
                    free_c_string(name_ptr);
                    (*result_ptr).success = false;
                    (*result_ptr).error_message =
                        to_c_string_owned("Failed to allocate memory for UDT members")
                            .unwrap_or(std::ptr::null_mut());
                    return -1;
                }

                (*result_ptr).name = name_ptr;
                (*result_ptr).members = members_ptr;
                (*result_ptr).member_count = definition.members.len() as c_int;

                // Copy members
                for (i, member) in definition.members.iter().enumerate() {
                    let member_name_ptr = match to_c_string_owned(&member.name) {
                        Ok(ptr) => ptr,
                        Err(_) => {
                            for j in 0..i {
                                let prev = members_ptr.add(j);
                                free_c_string((*prev).name);
                            }
                            libc::free(members_ptr as *mut c_void);
                            free_c_string(name_ptr);
                            (*result_ptr).success = false;
                            (*result_ptr).error_message = to_c_string_owned(
                                "Failed to allocate UDT member name (string contains null byte)",
                            )
                            .unwrap_or(std::ptr::null_mut());
                            (*result_ptr).name = std::ptr::null_mut();
                            (*result_ptr).members = std::ptr::null_mut();
                            (*result_ptr).member_count = 0;
                            return -1;
                        }
                    };

                    let member_c = UdtMemberC {
                        name: member_name_ptr,
                        data_type: member.data_type as c_short,
                        offset: member.offset as c_int,
                        size: member.size as c_int,
                    };
                    std::ptr::write(members_ptr.add(i), member_c);
                }
            }
            0
        }
        Err(e) => {
            unsafe {
                (*result_ptr).success = false;
                (*result_ptr).error_message =
                    to_c_string_owned(&format!("{}", e)).unwrap_or(std::ptr::null_mut());
                (*result_ptr).name = std::ptr::null_mut();
                (*result_ptr).members = std::ptr::null_mut();
                (*result_ptr).member_count = 0;
            }
            -1
        }
    }
}

/// FFI function to get UDT definition from PLC using client ID
///
/// The caller must free the returned fields using `eip_free_udt_definition`.
#[no_mangle]
pub unsafe extern "C" fn eip_get_udt_definition_by_id(
    client_id: c_int,
    udt_name: *const c_char,
    result_ptr: *mut UdtDefinitionResult,
) -> c_int {
    if udt_name.is_null() || result_ptr.is_null() {
        return -1;
    }

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    let Some(client) = clients.get_mut(&client_id) else {
        return -1;
    };

    let client_ptr = client as *mut EipClient;
    eip_get_udt_definition(client_ptr, udt_name, result_ptr)
}

/// FFI function to get tag attributes from PLC
///
/// The caller must free the returned fields using `eip_free_tag_attributes_result`.
#[no_mangle]
pub unsafe extern "C" fn eip_get_tag_attributes(
    client_ptr: *mut EipClient,
    tag_name: *const c_char,
    result_ptr: *mut TagAttributesResult,
) -> c_int {
    if client_ptr.is_null() || tag_name.is_null() || result_ptr.is_null() {
        return -1;
    }

    let tag_name_cstr = unsafe { CStr::from_ptr(tag_name) };
    let tag_name_str = match tag_name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let client = unsafe { &mut *client_ptr };
    let rt = RUNTIME.handle().clone();

    match rt.block_on(client.get_tag_attributes(tag_name_str)) {
        Ok(attributes) => {
            unsafe {
                (*result_ptr).success = true;
                (*result_ptr).error_message = std::ptr::null_mut();
                (*result_ptr).name = std::ptr::null_mut();
                (*result_ptr).data_type_name = std::ptr::null_mut();

                let name_ptr = match to_c_string_owned(&attributes.name) {
                    Ok(ptr) => ptr,
                    Err(_) => {
                        (*result_ptr).success = false;
                        (*result_ptr).error_message = to_c_string_owned(
                            "Failed to allocate tag name (string contains null byte)",
                        )
                        .unwrap_or(std::ptr::null_mut());
                        return -1;
                    }
                };

                let data_type_name_ptr = match to_c_string_owned(&attributes.data_type_name) {
                    Ok(ptr) => ptr,
                    Err(_) => {
                        free_c_string(name_ptr);
                        (*result_ptr).success = false;
                        (*result_ptr).error_message = to_c_string_owned(
                            "Failed to allocate data type name (string contains null byte)",
                        )
                        .unwrap_or(std::ptr::null_mut());
                        return -1;
                    }
                };

                (*result_ptr).name = name_ptr;
                (*result_ptr).data_type_name = data_type_name_ptr;
                (*result_ptr).data_type = attributes.data_type as c_short;
                (*result_ptr).size = attributes.size as c_int;
                (*result_ptr).template_instance_id =
                    attributes.template_instance_id.unwrap_or(0) as c_int;
            }
            0
        }
        Err(e) => {
            unsafe {
                (*result_ptr).success = false;
                (*result_ptr).error_message =
                    to_c_string_owned(&format!("{}", e)).unwrap_or(std::ptr::null_mut());
                (*result_ptr).name = std::ptr::null_mut();
                (*result_ptr).data_type_name = std::ptr::null_mut();
                (*result_ptr).data_type = 0;
                (*result_ptr).size = 0;
                (*result_ptr).template_instance_id = 0;
            }
            -1
        }
    }
}

/// FFI function to get tag attributes from PLC using client ID
///
/// The caller must free the returned fields using `eip_free_tag_attributes_result`.
#[no_mangle]
pub unsafe extern "C" fn eip_get_tag_attributes_by_id(
    client_id: c_int,
    tag_name: *const c_char,
    result_ptr: *mut TagAttributesResult,
) -> c_int {
    if tag_name.is_null() || result_ptr.is_null() {
        return -1;
    }

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    let Some(client) = clients.get_mut(&client_id) else {
        return -1;
    };

    let client_ptr = client as *mut EipClient;
    eip_get_tag_attributes(client_ptr, tag_name, result_ptr)
}

/// FFI function to discover tags with detailed attributes
///
/// The caller must free the returned fields using `eip_free_tag_discovery_result`.
#[no_mangle]
pub unsafe extern "C" fn eip_discover_tags_detailed(
    client_ptr: *mut EipClient,
    result_ptr: *mut TagDiscoveryResult,
) -> c_int {
    if client_ptr.is_null() || result_ptr.is_null() {
        return -1;
    }

    let client = unsafe { &mut *client_ptr };
    let rt = RUNTIME.handle().clone();

    match rt.block_on(client.discover_tags_detailed()) {
        Ok(tags) => {
            unsafe {
                (*result_ptr).success = true;
                (*result_ptr).error_message = std::ptr::null_mut();
                (*result_ptr).tag_count = tags.len() as c_int;

                // Allocate memory for tag attributes
                let tags_ptr = libc::malloc(std::mem::size_of::<TagAttributesC>() * tags.len())
                    as *mut TagAttributesC;

                if tags_ptr.is_null() {
                    (*result_ptr).success = false;
                    (*result_ptr).error_message =
                        to_c_string_owned("Failed to allocate memory for tag attributes")
                            .unwrap_or(std::ptr::null_mut());
                    return -1;
                }

                (*result_ptr).tags = tags_ptr;

                // Copy tag attributes
                for (i, tag) in tags.iter().enumerate() {
                    let name_ptr = match to_c_string_owned(&tag.name) {
                        Ok(ptr) => ptr,
                        Err(_) => {
                            for j in 0..i {
                                let prev = tags_ptr.add(j);
                                free_c_string((*prev).name);
                                free_c_string((*prev).data_type_name);
                            }
                            libc::free(tags_ptr as *mut c_void);
                            (*result_ptr).success = false;
                            (*result_ptr).error_message = to_c_string_owned(
                                "Failed to allocate tag name (string contains null byte)",
                            )
                            .unwrap_or(std::ptr::null_mut());
                            (*result_ptr).tags = std::ptr::null_mut();
                            (*result_ptr).tag_count = 0;
                            return -1;
                        }
                    };

                    let data_type_name_ptr = match to_c_string_owned(&tag.data_type_name) {
                        Ok(ptr) => ptr,
                        Err(_) => {
                            free_c_string(name_ptr);
                            for j in 0..i {
                                let prev = tags_ptr.add(j);
                                free_c_string((*prev).name);
                                free_c_string((*prev).data_type_name);
                            }
                            libc::free(tags_ptr as *mut c_void);
                            (*result_ptr).success = false;
                            (*result_ptr).error_message = to_c_string_owned(
                                "Failed to allocate data type name (string contains null byte)",
                            )
                            .unwrap_or(std::ptr::null_mut());
                            (*result_ptr).tags = std::ptr::null_mut();
                            (*result_ptr).tag_count = 0;
                            return -1;
                        }
                    };

                    let tag_c = TagAttributesC {
                        name: name_ptr,
                        data_type_name: data_type_name_ptr,
                        data_type: tag.data_type as c_short,
                        size: tag.size as c_int,
                        template_instance_id: tag.template_instance_id.unwrap_or(0) as c_int,
                    };
                    std::ptr::write(tags_ptr.add(i), tag_c);
                }
            }
            0
        }
        Err(e) => {
            unsafe {
                (*result_ptr).success = false;
                (*result_ptr).error_message =
                    to_c_string_owned(&format!("{}", e)).unwrap_or(std::ptr::null_mut());
                (*result_ptr).tags = std::ptr::null_mut();
                (*result_ptr).tag_count = 0;
            }
            -1
        }
    }
}

/// FFI function to discover tags with detailed attributes using client ID
///
/// The caller must free the returned fields using `eip_free_tag_discovery_result`.
#[no_mangle]
pub unsafe extern "C" fn eip_discover_tags_detailed_by_id(
    client_id: c_int,
    result_ptr: *mut TagDiscoveryResult,
) -> c_int {
    if result_ptr.is_null() {
        return -1;
    }

    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    let Some(client) = clients.get_mut(&client_id) else {
        return -1;
    };

    let client_ptr = client as *mut EipClient;
    eip_discover_tags_detailed(client_ptr, result_ptr)
}
