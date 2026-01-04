#![allow(clippy::missing_safety_doc)]

use crate::EipClient;
use crate::PlcValue;
use crate::RUNTIME;
use lazy_static::lazy_static;
use serde_json;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_short};
use std::ptr;
use std::sync::Mutex;

// FFI-specific client manager using synchronous mutex
lazy_static! {
    static ref FFI_CLIENTS: Mutex<HashMap<i32, EipClient>> = Mutex::new(HashMap::new());
    static ref FFI_NEXT_ID: Mutex<i32> = Mutex::new(1);
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
        let mut next_id = FFI_NEXT_ID.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        id
    };

    {
        let mut clients = FFI_CLIENTS.lock().unwrap();
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
        let mut next_id = FFI_NEXT_ID.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        id
    };

    {
        let mut clients = FFI_CLIENTS.lock().unwrap();
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
    let mut clients = FFI_CLIENTS.lock().unwrap();
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
    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
    match clients.get_mut(&client_id) {
        Some(client) => match RUNTIME.block_on(client.read_tag(tag_name_str)) {
            Ok(PlcValue::Dint(value)) => {
                unsafe {
                    *result = value;
                }
                0
            }
            Ok(other_value) => {
                eprintln!("❌ [FFI] Expected DINT but got: {:?}", other_value);
                -1
            }
            Err(e) => {
                eprintln!("❌ [FFI] Read tag '{}' failed: {}", tag_name_str, e);
                -1
            }
        },
        None => {
            eprintln!("❌ [FFI] Client ID {} not found", client_id);
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
    let Some(client) = clients.get_mut(&client_id) else {
        return -1;
    };

    let value = match RUNTIME.block_on(client.read_tag(tag_name_str)) {
        Ok(PlcValue::String(value)) => value,
        Ok(_) => return -1,  // Wrong data type
        Err(_) => return -1, // Error reading tag
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

    let clients = FFI_CLIENTS.lock().unwrap();
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
    if tag_names.is_null() || results.is_null() || tag_count <= 0 {
        return -1;
    }

    let mut clients = FFI_CLIENTS.lock().unwrap();
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
            // Simple format: "tag1:value1;tag2:value2;..."
            let mut formatted = String::new();
            for (i, (tag_name, result)) in results.iter().enumerate() {
                if i > 0 {
                    formatted.push(';');
                }
                formatted.push_str(tag_name);
                formatted.push(':');
                match result {
                    Ok(value) => formatted.push_str(&format!("{value:?}")),
                    Err(e) => formatted.push_str(&format!("ERROR:{e}")),
                }
            }
            formatted
        }
        Err(_) => return -1,
    };

    // Copy results to output buffer
    let results_bytes = results_data.as_bytes();
    if results_bytes.len() >= results_capacity as usize {
        return -1;
    }

    unsafe {
        std::ptr::copy_nonoverlapping(
            results_bytes.as_ptr(),
            results as *mut u8,
            results_bytes.len(),
        );
        *results.add(results_bytes.len()) = 0; // Null terminate
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
    if tag_values.is_null() || results.is_null() || tag_count <= 0 {
        return -1;
    }

    let mut clients = FFI_CLIENTS.lock().unwrap();
    let Some(_client) = clients.get_mut(&client_id) else {
        return -1;
    };

    // Parse input (simplified implementation)
    let _input_str = unsafe {
        match CStr::from_ptr(tag_values).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    // For now, return not implemented
    // TODO: Parse input and execute batch write
    let results_data = "ERROR:Batch write not yet implemented in FFI";
    let results_bytes = results_data.as_bytes();

    if results_bytes.len() >= results_capacity as usize {
        return -1;
    }

    unsafe {
        std::ptr::copy_nonoverlapping(
            results_bytes.as_ptr(),
            results as *mut u8,
            results_bytes.len(),
        );
        *results.add(results_bytes.len()) = 0; // Null terminate
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
    if operations.is_null() || results.is_null() || operation_count <= 0 {
        return -1;
    }

    let mut clients = FFI_CLIENTS.lock().unwrap();
    let Some(_client) = clients.get_mut(&client_id) else {
        return -1;
    };

    // Parse input (simplified implementation)
    let _input_str = unsafe {
        match CStr::from_ptr(operations).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    // For now, return not implemented
    // TODO: Parse input and execute mixed batch operations
    let results_data = "ERROR:Mixed batch operations not yet implemented in FFI";
    let results_bytes = results_data.as_bytes();

    if results_bytes.len() >= results_capacity as usize {
        return -1;
    }

    unsafe {
        std::ptr::copy_nonoverlapping(
            results_bytes.as_ptr(),
            results as *mut u8,
            results_bytes.len(),
        );
        *results.add(results_bytes.len()) = 0; // Null terminate
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn eip_configure_batch_operations(
    _client_id: c_int,
    _config: *const u8,
) -> c_int {
    0 // Return success for now
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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
    if udt_name.is_null() || result.is_null() || max_size <= 0 {
        return -1;
    }

    let Ok(udt_name_str) = unsafe { CStr::from_ptr(udt_name) }.to_str() else {
        return -1;
    };

    let mut clients = FFI_CLIENTS.lock().unwrap();
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
    if udt_name.is_null() || value.is_null() || size <= 0 {
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

    let mut clients = FFI_CLIENTS.lock().unwrap();
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

/// FFI function to get UDT definition from PLC
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

                // Convert UdtDefinition to C struct
                let name_cstring = CString::new(definition.name).unwrap_or_default();
                (*result_ptr).name = name_cstring.into_raw();
                (*result_ptr).member_count = definition.members.len() as c_int;

                // Allocate memory for members
                let members_ptr =
                    libc::malloc(std::mem::size_of::<UdtMemberC>() * definition.members.len())
                        as *mut UdtMemberC;

                if members_ptr.is_null() {
                    (*result_ptr).success = false;
                    let error_msg =
                        CString::new("Failed to allocate memory for UDT members").unwrap();
                    (*result_ptr).error_message = error_msg.into_raw();
                    return -1;
                }

                (*result_ptr).members = members_ptr;

                // Copy members
                for (i, member) in definition.members.iter().enumerate() {
                    let member_c = UdtMemberC {
                        name: CString::new(member.name.clone()).unwrap().into_raw(),
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
                let error_msg = CString::new(format!("{}", e)).unwrap();
                (*result_ptr).error_message = error_msg.into_raw();
                (*result_ptr).name = std::ptr::null_mut();
                (*result_ptr).members = std::ptr::null_mut();
                (*result_ptr).member_count = 0;
            }
            -1
        }
    }
}

/// FFI function to get tag attributes from PLC
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

                let name_cstring = CString::new(attributes.name).unwrap_or_default();
                (*result_ptr).name = name_cstring.into_raw();

                let data_type_name_cstring =
                    CString::new(attributes.data_type_name).unwrap_or_default();
                (*result_ptr).data_type_name = data_type_name_cstring.into_raw();

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
                let error_msg = CString::new(format!("{}", e)).unwrap();
                (*result_ptr).error_message = error_msg.into_raw();
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

/// FFI function to discover tags with detailed attributes
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
                    let error_msg =
                        CString::new("Failed to allocate memory for tag attributes").unwrap();
                    (*result_ptr).error_message = error_msg.into_raw();
                    return -1;
                }

                (*result_ptr).tags = tags_ptr;

                // Copy tag attributes
                for (i, tag) in tags.iter().enumerate() {
                    let tag_c = TagAttributesC {
                        name: CString::new(tag.name.clone()).unwrap().into_raw(),
                        data_type_name: CString::new(tag.data_type_name.clone())
                            .unwrap()
                            .into_raw(),
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
                let error_msg = CString::new(format!("{}", e)).unwrap();
                (*result_ptr).error_message = error_msg.into_raw();
                (*result_ptr).tags = std::ptr::null_mut();
                (*result_ptr).tag_count = 0;
            }
            -1
        }
    }
}
