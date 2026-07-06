use crate::EipClient;
use crate::PlcValue;
use crate::RUNTIME;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_short, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
use std::sync::{LazyLock, Mutex, MutexGuard, Once, OnceLock};
use tracing;

// FFI-specific client manager using synchronous mutex
const EIP_ERROR_RUNTIME_INIT: c_int = -2;

static FFI_CLIENTS: LazyLock<Mutex<HashMap<i32, EipClient>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static FFI_NEXT_ID: LazyLock<Mutex<i32>> = LazyLock::new(|| Mutex::new(1));
/// Per-client last error message, set when an FFI operation returns a failure
/// code so wrappers can retrieve a human-readable reason via `eip_get_last_error`.
static FFI_LAST_ERRORS: LazyLock<Mutex<HashMap<i32, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static RUNTIME_INIT_LOG: Once = Once::new();
#[cfg(test)]
static FORCE_RUNTIME_INIT_ERROR: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[unsafe(no_mangle)]
pub extern "C" fn eip_abi_version() -> u32 {
    crate::version::ABI_VERSION
}

/// Returns a static, null-terminated semver string.
///
/// The returned pointer is valid for the process lifetime. Callers must not free it.
#[unsafe(no_mangle)]
pub extern "C" fn eip_library_version() -> *const c_char {
    static VERSION_C: OnceLock<CString> = OnceLock::new();
    VERSION_C
        .get_or_init(|| CString::new(env!("CARGO_PKG_VERSION")).expect("static version"))
        .as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn eip_capabilities() -> u64 {
    crate::version::CAPABILITIES
}

fn runtime_init_error_code(error: &std::io::Error) -> c_int {
    RUNTIME_INIT_LOG.call_once(|| {
        tracing::error!("[FFI] Failed to initialize Tokio runtime: {}", error);
    });
    EIP_ERROR_RUNTIME_INIT
}

fn runtime() -> Result<&'static tokio::runtime::Runtime, c_int> {
    #[cfg(test)]
    if FORCE_RUNTIME_INIT_ERROR.load(std::sync::atomic::Ordering::SeqCst) {
        let error = std::io::Error::other("forced runtime initialization failure");
        return Err(runtime_init_error_code(&error));
    }

    match &*RUNTIME {
        Ok(runtime) => Ok(runtime),
        Err(error) => Err(runtime_init_error_code(error)),
    }
}

/// Awaits a future on the FFI Tokio runtime, early-returning
/// `EIP_ERROR_RUNTIME_INIT` if the runtime is unavailable. Only call
/// from inside an `unsafe extern "C" fn ... -> c_int` body.
macro_rules! ffi_block_on {
    ($client_id:expr, $future:expr) => {{
        let runtime = match runtime() {
            Ok(runtime) => runtime,
            Err(code) => {
                set_last_error(
                    $client_id,
                    format!("native runtime initialization failed with code {code}"),
                );
                return code;
            }
        };
        match catch_unwind(AssertUnwindSafe(|| runtime.block_on($future))) {
            Ok(value) => value,
            Err(payload) => {
                set_last_error($client_id, internal_panic_message(payload));
                return -1;
            }
        }
    }};
}

fn to_c_string_owned(value: &str) -> Result<*mut c_char, ()> {
    CString::new(value).map(|s| s.into_raw()).map_err(|_| ())
}

fn lock_clients() -> Result<MutexGuard<'static, HashMap<i32, EipClient>>, ()> {
    Ok(FFI_CLIENTS.lock().unwrap_or_else(|poisoned| {
        tracing::warn!("[FFI] Recovering poisoned client registry lock");
        poisoned.into_inner()
    }))
}

fn lock_next_id() -> Result<MutexGuard<'static, i32>, ()> {
    Ok(FFI_NEXT_ID.lock().unwrap_or_else(|poisoned| {
        tracing::warn!("[FFI] Recovering poisoned client-id allocator lock");
        poisoned.into_inner()
    }))
}

fn get_client(client_id: c_int) -> Result<EipClient, ()> {
    let clients = lock_clients()?;
    clients.get(&client_id).cloned().ok_or(())
}

fn allocate_client_id(clients: &HashMap<i32, EipClient>) -> Result<c_int, &'static str> {
    let mut next_id = lock_next_id().map_err(|_| "client-id allocator lock unavailable")?;
    let start = (*next_id).max(1);
    if *next_id < 1 {
        *next_id = 1;
    }

    loop {
        let candidate = *next_id;
        *next_id = if candidate == c_int::MAX {
            1
        } else {
            candidate + 1
        };

        if candidate > 0 && !clients.contains_key(&candidate) {
            return Ok(candidate);
        }

        if *next_id == start {
            return Err("FFI client id space exhausted");
        }
    }
}

#[doc(hidden)]
pub fn client_route_path_snapshot_for_testing(client_id: c_int) -> Option<crate::RoutePath> {
    get_client(client_id)
        .ok()
        .and_then(|client| client.get_route_path())
}

#[doc(hidden)]
pub fn client_max_packet_size_for_testing(client_id: c_int) -> Option<u32> {
    get_client(client_id)
        .ok()
        .map(|client| client.max_packet_size())
}

/// Records the last error message for a client so wrappers can surface a
/// human-readable reason after a failure code. Best-effort: a poisoned lock is
/// ignored rather than panicking across the FFI boundary.
fn set_last_error(client_id: c_int, message: impl Into<String>) {
    let mut errors = FFI_LAST_ERRORS.lock().unwrap_or_else(|poisoned| {
        tracing::warn!("[FFI] Recovering poisoned last-error lock");
        poisoned.into_inner()
    });
    errors.insert(client_id, message.into());
}

fn clear_last_error(client_id: c_int) {
    let mut errors = FFI_LAST_ERRORS.lock().unwrap_or_else(|poisoned| {
        tracing::warn!("[FFI] Recovering poisoned last-error lock");
        poisoned.into_inner()
    });
    errors.remove(&client_id);
}

fn remove_last_error(client_id: c_int) {
    clear_last_error(client_id);
}

fn fail_with_last_error(client_id: c_int, message: impl Into<String>) -> c_int {
    set_last_error(client_id, message);
    -1
}

fn internal_panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        format!("internal panic: {message}")
    } else if let Some(message) = payload.downcast_ref::<String>() {
        format!("internal panic: {message}")
    } else {
        "internal panic: unknown payload".to_string()
    }
}

/// Copies the most recent error message for `client_id` into `buffer` as a
/// NUL-terminated UTF-8 string. Returns the number of bytes written (excluding
/// the NUL), 0 if there is no recorded error, or -1 on a null buffer / capacity
/// overflow. Part of capability `CAP_LAST_ERROR`.
///
/// # Safety
///
/// `buffer` must be a valid, writable pointer to at least `max_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_get_last_error(
    client_id: c_int,
    buffer: *mut c_char,
    max_len: c_int,
) -> c_int {
    if buffer.is_null() || max_len <= 0 {
        return -1;
    }
    let message = match FFI_LAST_ERRORS.lock() {
        Ok(errors) => errors.get(&client_id).cloned(),
        Err(_) => return -1,
    };
    let Some(message) = message else {
        // No recorded error: write an empty string.
        // SAFETY: The output pointer was checked for null and the caller contract requires writable storage for this layout.
        unsafe { *buffer = 0 };
        return 0;
    };
    let Ok(c_message) = CString::new(message) else {
        return -1;
    };
    let bytes = c_message.as_bytes_with_nul();
    if bytes.len() > max_len as usize {
        return -1;
    }
    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), buffer, bytes.len());
    }
    // Bytes written excluding the trailing NUL.
    (bytes.len() - 1) as c_int
}

/// Generates an `eip_read_<type>` scalar FFI wrapper. The `|$v| $conv` argument
/// binds the inner `PlcValue` payload to `$v` and converts it to the C result
/// type; failures record a last-error message for `eip_get_last_error`.
macro_rules! ffi_read_scalar {
    ($name:ident, $ctype:ty, $variant:ident, |$v:ident| $conv:expr) => {
        /// Reads a scalar tag through the C FFI.
        ///
        /// # Safety
        ///
        /// `tag_name` must point to a valid NUL-terminated UTF-8 string and
        /// `result` must point to writable storage for the requested scalar.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            client_id: c_int,
            tag_name: *const c_char,
            result: *mut $ctype,
        ) -> c_int {
            if tag_name.is_null() || result.is_null() {
                return -1;
            }
            // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
            let Ok(tag_name_str) = (unsafe { CStr::from_ptr(tag_name) }).to_str() else {
                return -1;
            };
            let mut client = match get_client(client_id) {
                Ok(client) => client,
                Err(_) => return -1,
            };
            match ffi_block_on!(client_id, client.read_tag(tag_name_str)) {
                Ok(PlcValue::$variant($v)) => {
                    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
                    unsafe {
                        *result = $conv;
                    }
                    clear_last_error(client_id);
                    0
                }
                Ok(other) => {
                    set_last_error(
                        client_id,
                        format!(
                            "tag '{}': expected {} but got {:?}",
                            tag_name_str,
                            stringify!($variant),
                            other
                        ),
                    );
                    -1
                }
                Err(e) => {
                    set_last_error(client_id, e.to_string());
                    -1
                }
            }
        }
    };
}

/// Generates an `eip_write_<type>` scalar FFI wrapper. The `|$v| $conv` argument
/// binds the C value parameter to `$v` and converts it to the inner `PlcValue`
/// payload; failures record a last-error message for `eip_get_last_error`.
macro_rules! ffi_write_scalar {
    ($name:ident, $ctype:ty, $variant:ident, |$v:ident| $conv:expr) => {
        /// Writes a scalar tag through the C FFI.
        ///
        /// # Safety
        ///
        /// `tag_name` must point to a valid NUL-terminated UTF-8 string for
        /// the duration of the call.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            client_id: c_int,
            tag_name: *const c_char,
            $v: $ctype,
        ) -> c_int {
            if tag_name.is_null() {
                return -1;
            }
            // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
            let Ok(tag_name_str) = (unsafe { CStr::from_ptr(tag_name) }).to_str() else {
                return -1;
            };
            let mut client = match get_client(client_id) {
                Ok(client) => client,
                Err(_) => return -1,
            };
            match ffi_block_on!(
                client_id,
                client.write_tag(tag_name_str, PlcValue::$variant($conv))
            ) {
                Ok(_) => {
                    clear_last_error(client_id);
                    0
                }
                Err(e) => {
                    set_last_error(client_id, e.to_string());
                    -1
                }
            }
        }
    };
}

unsafe fn build_route_path_from_grouped_fields(
    slots: *const u8,
    slot_count: c_int,
    ports: *const u8,
    port_count: c_int,
    addresses: *mut *const c_char,
    address_count: c_int,
) -> crate::RoutePath {
    let mut route_path = crate::RoutePath::new();

    if !slots.is_null() && slot_count > 0 {
        // SAFETY: Caller-provided count and pointer arguments were validated before constructing this slice or offset pointer.
        let slots_slice = unsafe { std::slice::from_raw_parts(slots, slot_count as usize) };
        for &slot in slots_slice {
            route_path = route_path.add_backplane(1, slot);
        }
    }

    if !addresses.is_null() && address_count > 0 {
        let ports_slice = if !ports.is_null() && port_count > 0 {
            // SAFETY: Caller-provided count and pointer arguments were validated before constructing this slice or offset pointer.
            Some(unsafe { std::slice::from_raw_parts(ports, port_count as usize) })
        } else {
            None
        };
        let addresses_slice =
            // SAFETY: Caller-provided count and pointer arguments were validated before constructing this slice or offset pointer.
            unsafe { std::slice::from_raw_parts(addresses, address_count as usize) };
        for (index, &addr_ptr) in addresses_slice.iter().enumerate() {
            if !addr_ptr.is_null()
                // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
                && let Ok(addr_str) = unsafe { CStr::from_ptr(addr_ptr) }.to_str()
            {
                let port = ports_slice
                    .and_then(|slice| slice.get(index))
                    .copied()
                    .unwrap_or(2);
                route_path = route_path.add_ethernet_with_port(port, addr_str);
            }
        }
    }

    route_path
}

unsafe fn build_route_path_from_ordered_hops(
    hop_types: *const u8,
    ports: *const u8,
    slots: *const u8,
    addresses: *mut *const c_char,
    hop_count: c_int,
) -> Option<crate::RoutePath> {
    if hop_count < 0 {
        return None;
    }
    if hop_count == 0 {
        return Some(crate::RoutePath::new());
    }
    if hop_types.is_null() || ports.is_null() {
        return None;
    }

    // SAFETY: Caller-provided count and pointer arguments were validated before constructing this slice or offset pointer.
    let hop_types = unsafe { std::slice::from_raw_parts(hop_types, hop_count as usize) };
    // SAFETY: Caller-provided count and pointer arguments were validated before constructing this slice or offset pointer.
    let ports = unsafe { std::slice::from_raw_parts(ports, hop_count as usize) };
    let slots = if slots.is_null() {
        &[][..]
    } else {
        // SAFETY: Caller-provided count and pointer arguments were validated before constructing this slice or offset pointer.
        unsafe { std::slice::from_raw_parts(slots, hop_count as usize) }
    };
    let addresses = if addresses.is_null() {
        &[][..]
    } else {
        // SAFETY: Caller-provided count and pointer arguments were validated before constructing this slice or offset pointer.
        unsafe { std::slice::from_raw_parts(addresses, hop_count as usize) }
    };

    let mut route_path = crate::RoutePath::new();
    for index in 0..hop_count as usize {
        match hop_types[index] {
            1 => {
                let slot = *slots.get(index).unwrap_or(&0);
                route_path = route_path.add_backplane(ports[index], slot);
            }
            2 => {
                let addr_ptr = *addresses.get(index).unwrap_or(&ptr::null());
                if addr_ptr.is_null() {
                    return None;
                }
                // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
                let Ok(addr_str) = (unsafe { CStr::from_ptr(addr_ptr) }).to_str() else {
                    return None;
                };
                route_path = route_path.add_ethernet_with_port(ports[index], addr_str);
            }
            _ => return None,
        }
    }

    Some(route_path)
}

unsafe fn free_c_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        // SAFETY: The pointer being freed was allocated by this library and ownership has returned to Rust.
        let _ = unsafe { CString::from_raw(ptr) };
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

    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), output as *mut u8, bytes.len());
        *output.add(bytes.len()) = 0;
    }

    Ok(())
}

fn system_time_to_unix_millis(value: Option<std::time::SystemTime>) -> Option<u64> {
    value.and_then(|time| {
        time.duration_since(std::time::SystemTime::UNIX_EPOCH)
            .ok()
            .map(|duration| duration.as_millis() as u64)
    })
}

fn duration_to_seconds(value: std::time::Duration) -> f64 {
    value.as_secs_f64()
}

fn diagnostics_snapshot_json(snapshot: &crate::DiagnosticsSnapshot) -> Result<String, ()> {
    let payload = serde_json::json!({
        "captured_at_unix_ms": system_time_to_unix_millis(Some(snapshot.captured_at)),
        "system_metrics_are_placeholders": snapshot.system_metrics_are_placeholders,
        "connections": {
            "active_connections": snapshot.connections.active_connections,
            "total_connections": snapshot.connections.total_connections,
            "failed_connections": snapshot.connections.failed_connections,
            "connection_uptime_avg_seconds": duration_to_seconds(snapshot.connections.connection_uptime_avg),
            "last_connection_time_unix_ms": system_time_to_unix_millis(snapshot.connections.last_connection_time),
        },
        "operations": {
            "total_reads": snapshot.operations.total_reads,
            "total_writes": snapshot.operations.total_writes,
            "successful_reads": snapshot.operations.successful_reads,
            "successful_writes": snapshot.operations.successful_writes,
            "failed_reads": snapshot.operations.failed_reads,
            "failed_writes": snapshot.operations.failed_writes,
            "batch_operations": snapshot.operations.batch_operations,
            "subscription_updates": snapshot.operations.subscription_updates,
            "partial_batch_failures": snapshot.operations.partial_batch_failures,
            "last_successful_read_time_unix_ms": system_time_to_unix_millis(snapshot.operations.last_successful_read_time),
            "last_failed_read_time_unix_ms": system_time_to_unix_millis(snapshot.operations.last_failed_read_time),
            "last_successful_write_time_unix_ms": system_time_to_unix_millis(snapshot.operations.last_successful_write_time),
            "last_failed_write_time_unix_ms": system_time_to_unix_millis(snapshot.operations.last_failed_write_time),
        },
        "performance": {
            "avg_read_latency_ms": snapshot.performance.avg_read_latency_ms,
            "avg_write_latency_ms": snapshot.performance.avg_write_latency_ms,
            "max_read_latency_ms": snapshot.performance.max_read_latency_ms,
            "max_write_latency_ms": snapshot.performance.max_write_latency_ms,
            "reads_per_second": snapshot.performance.reads_per_second,
            "writes_per_second": snapshot.performance.writes_per_second,
            "memory_usage_mb": snapshot.performance.memory_usage_mb,
            "cpu_usage_percent": snapshot.performance.cpu_usage_percent,
        },
        "errors": {
            "network_errors": snapshot.errors.network_errors,
            "protocol_errors": snapshot.errors.protocol_errors,
            "timeout_errors": snapshot.errors.timeout_errors,
            "tag_not_found_errors": snapshot.errors.tag_not_found_errors,
            "data_type_errors": snapshot.errors.data_type_errors,
            "session_errors": snapshot.errors.session_errors,
            "route_path_errors": snapshot.errors.route_path_errors,
            "embedded_service_errors": snapshot.errors.embedded_service_errors,
            "known_controller_limitation_errors": snapshot.errors.known_controller_limitation_errors,
            "retriable_errors": snapshot.errors.retriable_errors,
            "non_retriable_errors": snapshot.errors.non_retriable_errors,
            "last_error_time_unix_ms": system_time_to_unix_millis(snapshot.errors.last_error_time),
            "last_error_message": snapshot.errors.last_error_message,
            "last_error_category": snapshot.errors.last_error_category.map(|value| format!("{value:?}")),
            "last_retriable_error_time_unix_ms": system_time_to_unix_millis(snapshot.errors.last_retriable_error_time),
        },
        "health": {
            "overall_health": format!("{:?}", snapshot.health.overall_health),
            "health_mode": format!("{:?}", snapshot.health.health_mode),
            "last_health_check_unix_ms": system_time_to_unix_millis(Some(snapshot.health.last_health_check)),
            "last_verified_health_check_unix_ms": system_time_to_unix_millis(snapshot.health.last_verified_health_check),
            "consecutive_failures": snapshot.health.consecutive_failures,
            "recovery_attempts": snapshot.health.recovery_attempts,
            "system_uptime_seconds": duration_to_seconds(snapshot.health.system_uptime),
            "last_success_time_unix_ms": system_time_to_unix_millis(snapshot.health.last_success_time),
            "last_failure_time_unix_ms": system_time_to_unix_millis(snapshot.health.last_failure_time),
        }
    });

    serde_json::to_string(&payload).map_err(|_| ())
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_connect(ip_address: *const c_char) -> c_int {
    if ip_address.is_null() {
        return -1;
    }

    // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
    let Ok(ip_str) = unsafe { CStr::from_ptr(ip_address) }.to_str() else {
        return -1;
    };

    let Ok(client) = ffi_block_on!(0, EipClient::new(ip_str)) else {
        return -1;
    };

    {
        let mut clients = match lock_clients() {
            Ok(guard) => guard,
            Err(_) => return -1,
        };
        let client_id = match allocate_client_id(&clients) {
            Ok(client_id) => client_id,
            Err(message) => return fail_with_last_error(0, message),
        };
        clients.insert(client_id, client);
        clear_last_error(client_id);
        client_id
    }
}

fn write_output_buffer_or_last_error(
    client_id: c_int,
    output: *mut c_char,
    capacity: c_int,
    payload: &str,
    context: &str,
) -> c_int {
    match write_output_buffer(output, capacity, payload) {
        Ok(()) => {
            clear_last_error(client_id);
            0
        }
        Err(()) => fail_with_last_error(
            client_id,
            format!("{context} output does not fit in caller buffer"),
        ),
    }
}

/// Connect to a PLC with route path (for ControlLogix)
///
/// # Safety
///
/// This function is unsafe because:
/// - `ip_address` must be a valid null-terminated C string pointer
/// - `slots` must be a valid pointer to an array of `slot_count` bytes
/// - The caller must ensure all pointers remain valid for the duration of the call
#[unsafe(no_mangle)]
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

    // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
    let Ok(ip_str) = unsafe { CStr::from_ptr(ip_address) }.to_str() else {
        return -1;
    };

    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    let route_path = unsafe {
        build_route_path_from_grouped_fields(
            slots,
            slot_count,
            ports,
            port_count,
            addresses,
            address_count,
        )
    };

    let Ok(client) = ffi_block_on!(0, crate::EipClient::with_route_path(ip_str, route_path)) else {
        return -1;
    };

    {
        let mut clients = match lock_clients() {
            Ok(guard) => guard,
            Err(_) => return -1,
        };
        let client_id = match allocate_client_id(&clients) {
            Ok(client_id) => client_id,
            Err(message) => return fail_with_last_error(0, message),
        };
        clients.insert(client_id, client);
        clear_last_error(client_id);
        client_id
    }
}

/// Connect to a PLC with an ordered route path.
///
/// # Safety
///
/// `ip_address` must be a valid NUL-terminated UTF-8 string. If `hop_count`
/// is non-zero, `hop_types`, `ports`, and any non-null `slots`/`addresses`
/// pointers must reference arrays with at least `hop_count` elements.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_connect_with_route_hops(
    ip_address: *const c_char,
    hop_types: *const u8,
    ports: *const u8,
    slots: *const u8,
    addresses: *mut *const c_char,
    hop_count: c_int,
) -> c_int {
    if ip_address.is_null() {
        return -1;
    }

    // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
    let Ok(ip_str) = unsafe { CStr::from_ptr(ip_address) }.to_str() else {
        return -1;
    };

    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    let Some(route_path) = (unsafe {
        build_route_path_from_ordered_hops(hop_types, ports, slots, addresses, hop_count)
    }) else {
        return -1;
    };

    let Ok(client) = ffi_block_on!(0, crate::EipClient::with_route_path(ip_str, route_path)) else {
        return -1;
    };

    {
        let mut clients = match lock_clients() {
            Ok(guard) => guard,
            Err(_) => return -1,
        };
        let client_id = match allocate_client_id(&clients) {
            Ok(client_id) => client_id,
            Err(message) => return fail_with_last_error(0, message),
        };
        clients.insert(client_id, client);
        clear_last_error(client_id);
        client_id
    }
}

/// Set route path for an existing client connection
///
/// # Safety
///
/// This function is unsafe because:
/// - `client_id` must be a valid client ID returned from `eip_connect`
/// - `slots` must be a valid pointer to an array of `slot_count` bytes
/// - The caller must ensure all pointers remain valid for the duration of the call
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_set_route_path(
    client_id: c_int,
    slots: *const u8,
    slot_count: c_int,
    ports: *const u8,
    port_count: c_int,
    addresses: *mut *const c_char,
    address_count: c_int,
) -> c_int {
    let mut client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => return -1,
    };

    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    let route_path = unsafe {
        build_route_path_from_grouped_fields(
            slots,
            slot_count,
            ports,
            port_count,
            addresses,
            address_count,
        )
    };

    client.set_route_path(route_path);
    0
}

/// Set an ordered route path on an existing client.
///
/// # Safety
///
/// If `hop_count` is non-zero, `hop_types`, `ports`, and any non-null
/// `slots`/`addresses` pointers must reference arrays with at least
/// `hop_count` elements. Address entries must be valid NUL-terminated UTF-8
/// strings when present.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_set_route_path_hops(
    client_id: c_int,
    hop_types: *const u8,
    ports: *const u8,
    slots: *const u8,
    addresses: *mut *const c_char,
    hop_count: c_int,
) -> c_int {
    let mut client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => return -1,
    };

    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    let Some(route_path) = (unsafe {
        build_route_path_from_ordered_hops(hop_types, ports, slots, addresses, hop_count)
    }) else {
        return -1;
    };

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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_disconnect(client_id: c_int) -> c_int {
    let mut clients = match lock_clients() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    let removed_client = clients.remove(&client_id);
    drop(clients);
    match removed_client {
        Some(mut client) => {
            if let Ok(runtime) = runtime() {
                let unregister_result = catch_unwind(AssertUnwindSafe(|| {
                    runtime.block_on(client.unregister_session())
                }));
                if let Err(payload) = unregister_result {
                    set_last_error(client_id, internal_panic_message(payload));
                }
            }
            remove_last_error(client_id);
            0
        }
        None => fail_with_last_error(client_id, "client id not found"),
    }
}

// Scalar read/write FFI wrappers. Generated by ffi_read_scalar! / ffi_write_scalar!,
// which record a last-error message on failure (see eip_get_last_error).
ffi_read_scalar!(eip_read_bool, c_int, Bool, |v| i32::from(v));
ffi_write_scalar!(eip_write_bool, c_int, Bool, |v| v != 0);
ffi_read_scalar!(eip_read_sint, i8, Sint, |v| v);
ffi_write_scalar!(eip_write_sint, i8, Sint, |v| v);
ffi_read_scalar!(eip_read_int, i16, Int, |v| v);
ffi_write_scalar!(eip_write_int, i16, Int, |v| v);
ffi_read_scalar!(eip_read_dint, c_int, Dint, |v| v);
ffi_write_scalar!(eip_write_dint, c_int, Dint, |v| v);
ffi_read_scalar!(eip_read_lint, i64, Lint, |v| v);
ffi_write_scalar!(eip_write_lint, i64, Lint, |v| v);
ffi_read_scalar!(eip_read_usint, u8, Usint, |v| v);
ffi_write_scalar!(eip_write_usint, u8, Usint, |v| v);
ffi_read_scalar!(eip_read_uint, u16, Uint, |v| v);
ffi_write_scalar!(eip_write_uint, u16, Uint, |v| v);
ffi_read_scalar!(eip_read_udint, u32, Udint, |v| v);
ffi_write_scalar!(eip_write_udint, u32, Udint, |v| v);
ffi_read_scalar!(eip_read_ulint, u64, Ulint, |v| v);
ffi_write_scalar!(eip_write_ulint, u64, Ulint, |v| v);
ffi_read_scalar!(eip_read_real, f64, Real, |v| f64::from(v));
ffi_write_scalar!(eip_write_real, f64, Real, |v| v as f32);
ffi_read_scalar!(eip_read_lreal, f64, Lreal, |v| v);
ffi_write_scalar!(eip_write_lreal, f64, Lreal, |v| v);

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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_read_string(
    client_id: c_int,
    tag_name: *const c_char,
    result: *mut c_char,
    max_length: c_int,
) -> c_int {
    if tag_name.is_null() || result.is_null() || max_length <= 0 {
        return -1;
    }

    // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => return -1,
    };

    let value = match ffi_block_on!(client_id, client.read_tag(tag_name_str)) {
        Ok(PlcValue::String(value)) => {
            tracing::info!(
                "[FFI] Read STRING tag '{}' succeeded: '{}'",
                tag_name_str,
                value
            );
            value
        }
        Ok(other) => {
            let message = format!(
                "tag '{}' is not a STRING; native read returned {:?}",
                tag_name_str,
                std::mem::discriminant(&other)
            );
            tracing::error!("[FFI] {message}");
            return fail_with_last_error(client_id, message);
        }
        Err(e) => {
            tracing::error!("[FFI] Read STRING tag '{}' failed: {}", tag_name_str, e);
            return fail_with_last_error(client_id, e.to_string());
        }
    };

    let Ok(c_string) = CString::new(value) else {
        return fail_with_last_error(client_id, "STRING value contains interior null byte");
    };

    let bytes = c_string.as_bytes_with_nul();
    if bytes.len() > max_length as usize {
        return fail_with_last_error(
            client_id,
            format!(
                "STRING result for '{tag_name_str}' too large for buffer ({} > {max_length} bytes)",
                bytes.len()
            ),
        );
    }

    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), result as *mut u8, bytes.len());
    }
    clear_last_error(client_id);
    0
}

/// Write a STRING tag.
///
/// # Safety
///
/// `tag_name` and `value` must point to valid NUL-terminated UTF-8 strings for
/// the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_write_string(
    client_id: c_int,
    tag_name: *const c_char,
    value: *const c_char,
) -> c_int {
    if tag_name.is_null() || value.is_null() {
        return fail_with_last_error(client_id, "tag name/value pointer is null");
    }

    // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return fail_with_last_error(client_id, "tag name is not valid UTF-8");
    };

    // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
    let Ok(value_str) = unsafe { CStr::from_ptr(value) }.to_str() else {
        return fail_with_last_error(client_id, "STRING value is not valid UTF-8");
    };

    let mut client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => return fail_with_last_error(client_id, "client id not found"),
    };

    match ffi_block_on!(
        client_id,
        client.write_tag(tag_name_str, PlcValue::String(value_str.to_string()))
    ) {
        Ok(_) => {
            clear_last_error(client_id);
            0
        }
        Err(e) => fail_with_last_error(client_id, e.to_string()),
    }
}

/// Read any tag and return as JSON (generic read function)
///
/// This function reads any tag type and returns it as JSON, allowing the caller
/// to determine the type dynamically. This is useful for complex paths like
/// UDT array element members (e.g., `"gTestUDT_Array[0].Member1_DINT"`).
///
/// # Safety
///
/// This function is unsafe because:
/// - `tag_name` must be a valid null-terminated C string pointer
/// - `result` must be a valid mutable pointer to a buffer of at least `max_size` bytes
/// - The caller must ensure both pointers remain valid for the duration of the call
/// - `client_id` must be a valid client ID returned from `eip_connect`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_read_tag(
    client_id: c_int,
    tag_name: *const c_char,
    result: *mut c_char,
    max_size: c_int,
) -> c_int {
    if tag_name.is_null() || result.is_null() || max_size <= 0 {
        return -1;
    }

    // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => {
            tracing::error!("[FFI] Client ID {} not found", client_id);
            return fail_with_last_error(client_id, "client id not found");
        }
    };
    let value = match ffi_block_on!(client_id, client.read_tag(tag_name_str)) {
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
            return fail_with_last_error(client_id, e.to_string());
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
            return fail_with_last_error(client_id, e.to_string());
        }
    };

    let Ok(c_string) = CString::new(json_result) else {
        tracing::error!("[FFI] Failed to create C string for tag '{}'", tag_name_str);
        return fail_with_last_error(client_id, "serialized tag JSON contains interior null byte");
    };

    let bytes = c_string.as_bytes_with_nul();
    if bytes.len() > max_size as usize {
        tracing::error!(
            "[FFI] JSON result too long for tag '{}': {} bytes (max: {})",
            tag_name_str,
            bytes.len(),
            max_size
        );
        return fail_with_last_error(
            client_id,
            format!(
                "read result for '{tag_name_str}' too large for buffer ({} > {max_size} bytes)",
                bytes.len()
            ),
        );
    }

    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), result as *mut u8, bytes.len());
    }
    clear_last_error(client_id);
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
#[unsafe(no_mangle)]
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
        return fail_with_last_error(client_id, "invalid array range arguments");
    }

    // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
    let Ok(base_array_name_str) = unsafe { CStr::from_ptr(base_array_name) }.to_str() else {
        return fail_with_last_error(client_id, "array name is not valid UTF-8");
    };

    let mut client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => return fail_with_last_error(client_id, "client id not found"),
    };

    let values = match ffi_block_on!(
        client_id,
        client.read_array_range(
            base_array_name_str,
            start_index as u32,
            element_count as u32,
        )
    ) {
        Ok(values) => values,
        Err(e) => return fail_with_last_error(client_id, e.to_string()),
    };

    let json_result = match serde_json::to_string(&values) {
        Ok(json) => json,
        Err(e) => return fail_with_last_error(client_id, e.to_string()),
    };

    let Ok(c_string) = CString::new(json_result) else {
        return fail_with_last_error(client_id, "array range JSON contains interior null byte");
    };

    let bytes = c_string.as_bytes_with_nul();
    if bytes.len() > max_size as usize {
        return fail_with_last_error(
            client_id,
            format!(
                "array range result for '{base_array_name_str}' too large for buffer ({} > {max_size} bytes)",
                bytes.len()
            ),
        );
    }

    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), result as *mut u8, bytes.len());
    }
    clear_last_error(client_id);
    0
}

/// Read a UDT tag into a caller-provided JSON buffer.
///
/// # Safety
///
/// `tag_name` must point to a valid NUL-terminated UTF-8 string and `result`
/// must be writable for `max_size` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_read_udt(
    client_id: c_int,
    tag_name: *const c_char,
    result: *mut c_char,
    max_size: c_int,
) -> c_int {
    if tag_name.is_null() || result.is_null() || max_size <= 0 {
        return fail_with_last_error(client_id, "invalid UDT read arguments");
    }

    // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return fail_with_last_error(client_id, "tag name is not valid UTF-8");
    };

    let mut client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => return fail_with_last_error(client_id, "client id not found"),
    };

    let value = match ffi_block_on!(client_id, client.read_udt_chunked(tag_name_str)) {
        Ok(PlcValue::Udt(udt_data)) => udt_data,
        Ok(other) => {
            return fail_with_last_error(
                client_id,
                format!("tag '{tag_name_str}' is not a UDT: {other:?}"),
            );
        }
        Err(e) => return fail_with_last_error(client_id, e.to_string()),
    };

    // Serialize UDT to JSON for C# consumption
    let json_result = match serde_json::to_string(&value) {
        Ok(json) => json,
        Err(e) => return fail_with_last_error(client_id, e.to_string()),
    };

    let Ok(c_string) = CString::new(json_result) else {
        return fail_with_last_error(client_id, "UDT JSON contains interior null byte");
    };

    let bytes = c_string.as_bytes_with_nul();
    if bytes.len() > max_size as usize {
        return fail_with_last_error(
            client_id,
            format!("UDT result for '{tag_name_str}' too large for buffer"),
        );
    }

    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), result as *mut u8, bytes.len());
    }
    clear_last_error(client_id);
    0
}

/// Write a UDT tag from a JSON payload.
///
/// # Safety
///
/// `tag_name` and `value` must point to valid NUL-terminated UTF-8 strings for
/// the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_write_udt(
    client_id: c_int,
    tag_name: *const c_char,
    value: *const c_char,
    size: c_int,
) -> c_int {
    if tag_name.is_null() || value.is_null() || size <= 0 {
        return fail_with_last_error(client_id, "tag name/value pointer is null or size <= 0");
    }

    // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return fail_with_last_error(client_id, "tag name is not valid UTF-8");
    };

    // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
    let Ok(value_str) = unsafe { CStr::from_ptr(value) }.to_str() else {
        return fail_with_last_error(client_id, "UDT JSON is not valid UTF-8");
    };

    // Generic UdtData format: {"symbol_id":N,"data":[..bytes..]}. This is the
    // shape the C#/Python wrappers send for raw UDT writes and what the batch
    // path uses. It is mutually exclusive with the member-dictionary shape
    // below (the dict has no symbol_id/data fields), so try it first and write
    // the raw bytes directly. A symbol_id of 0 triggers the read-before-write
    // path inside write_tag.
    if let Ok(udt_data) = serde_json::from_str::<crate::UdtData>(value_str) {
        let mut client = match get_client(client_id) {
            Ok(client) => client,
            Err(_) => return fail_with_last_error(client_id, "client id not found"),
        };
        return match ffi_block_on!(
            client_id,
            client.write_tag(tag_name_str, PlcValue::Udt(udt_data))
        ) {
            Ok(_) => {
                clear_last_error(client_id);
                0
            }
            Err(e) => fail_with_last_error(client_id, e.to_string()),
        };
    }

    // Deserialize JSON to UDT (HashMap format for backward compatibility)
    let udt_members: HashMap<String, PlcValue> = match serde_json::from_str(value_str) {
        Ok(data) => data,
        Err(e) => {
            return fail_with_last_error(
                client_id,
                format!("UDT JSON is neither raw UdtData nor member map: {e}"),
            );
        }
    };

    let mut client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => return fail_with_last_error(client_id, "client id not found"),
    };

    // Convert HashMap to UdtData format
    // First, read the tag to get symbol_id and UDT definition
    let udt_data = match ffi_block_on!(client_id, async {
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
        Ok(crate::UdtData::from_hash_map(
            &udt_members,
            &user_def,
            existing_udt.symbol_id,
        )?)
    }) {
        Ok(data) => data,
        Err(e) => return fail_with_last_error(client_id, e.to_string()),
    };

    match ffi_block_on!(
        client_id,
        client.write_tag(tag_name_str, PlcValue::Udt(udt_data))
    ) {
        Ok(_) => {
            clear_last_error(client_id);
            0
        }
        Err(e) => fail_with_last_error(client_id, e.to_string()),
    }
}

/// Legacy tag-discovery placeholder.
///
/// # Safety
///
/// This function does not dereference raw pointers. `client_id` should be a
/// handle returned by a connect function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_discover_tags(_client_id: c_int) -> c_int {
    // Return success for now - can implement tag discovery later
    0
}

/// Legacy tag-metadata placeholder.
///
/// # Safety
///
/// `tag_name` must be a valid NUL-terminated UTF-8 string when non-null, and
/// `metadata` must be writable for the layout expected by the caller. The
/// current implementation returns an unsupported error without dereferencing.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_get_tag_metadata(
    _client_id: c_int,
    _tag_name: *const c_char,
    _metadata: *mut u8,
) -> c_int {
    // For now, return error - metadata support can be added later
    -1
}

/// Set the maximum packet size for a connected client.
///
/// # Safety
///
/// This function does not dereference raw pointers. `client_id` should be a
/// handle returned by a connect function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_set_max_packet_size(client_id: c_int, size: c_int) -> c_int {
    if size <= 0 {
        return -1;
    }

    let mut client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => return -1,
    };
    client.set_max_packet_size(size as u32);
    0
}

/// Check client health.
///
/// # Safety
///
/// `is_healthy` must point to writable storage for one `c_int`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_check_health(client_id: c_int, is_healthy: *mut c_int) -> c_int {
    if is_healthy.is_null() {
        return -1;
    }

    let client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => {
            // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
            unsafe {
                *is_healthy = 0;
            }
            return -1;
        }
    };

    let is_ok = ffi_block_on!(client_id, client.check_health());
    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    unsafe {
        *is_healthy = if is_ok { 1 } else { 0 };
    }
    0
}

/// Check client health, including an active native health probe.
///
/// # Safety
///
/// `is_healthy` must point to writable storage for one `c_int`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_check_health_detailed(
    client_id: c_int,
    is_healthy: *mut c_int,
) -> c_int {
    if is_healthy.is_null() {
        return -1;
    }

    let mut client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => {
            // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
            unsafe {
                *is_healthy = 0;
            }
            return -1;
        }
    };

    let is_ok = ffi_block_on!(client_id, client.check_health_detailed()).unwrap_or_default();

    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    unsafe {
        *is_healthy = if is_ok { 1 } else { 0 };
    }
    0
}

/// Return a diagnostics snapshot as an allocated JSON string.
///
/// # Safety
///
/// `result_ptr` must point to writable storage for one `*mut c_char`. On
/// success the caller owns the returned string and must free it with
/// `eip_free_string`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_get_diagnostics_json(
    client_id: c_int,
    detailed: c_int,
    result_ptr: *mut *mut c_char,
) -> c_int {
    if result_ptr.is_null() {
        return -1;
    }

    let mut client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => return -1,
    };

    let snapshot = if detailed != 0 {
        match ffi_block_on!(client_id, client.get_diagnostics_snapshot_detailed()) {
            Ok(snapshot) => snapshot,
            Err(_) => return -1,
        }
    } else {
        ffi_block_on!(client_id, client.get_diagnostics_snapshot())
    };

    let json = match diagnostics_snapshot_json(&snapshot) {
        Ok(json) => json,
        Err(_) => return -1,
    };

    let Ok(owned) = to_c_string_owned(&json) else {
        return -1;
    };

    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    unsafe {
        *result_ptr = owned;
    }
    0
}

/// Read multiple tags and write a JSON result into a caller buffer.
///
/// # Safety
///
/// `tag_names` must reference `tag_count` valid C string pointers and
/// `results` must be writable for `results_capacity` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_read_tags_batch(
    client_id: c_int,
    tag_names: *mut *const c_char,
    tag_count: c_int,
    results: *mut c_char,
    results_capacity: c_int,
) -> c_int {
    if tag_names.is_null() || results.is_null() || tag_count <= 0 || results_capacity <= 0 {
        return fail_with_last_error(client_id, "invalid batch-read arguments");
    }

    let mut client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => return fail_with_last_error(client_id, "client id not found"),
    };

    // Convert C strings to Rust strings
    let mut tag_name_strs = Vec::new();
    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    unsafe {
        for i in 0..tag_count {
            let tag_name_ptr = *tag_names.offset(i as isize);
            if tag_name_ptr.is_null() {
                return fail_with_last_error(client_id, "batch-read tag pointer is null");
            }
            let Ok(tag_name) = CStr::from_ptr(tag_name_ptr).to_str() else {
                return fail_with_last_error(client_id, "batch-read tag name is not valid UTF-8");
            };
            tag_name_strs.push(tag_name);
        }
    }

    // Execute batch read
    let batch_results = ffi_block_on!(client_id, async {
        client.read_tags_batch(&tag_name_strs).await
    });

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
                Err(e) => return fail_with_last_error(client_id, e.to_string()),
            }
        }
        Err(e) => return fail_with_last_error(client_id, e.to_string()),
    };

    write_output_buffer_or_last_error(
        client_id,
        results,
        results_capacity,
        &results_data,
        "batch-read",
    )
}

/// Write multiple tags from a JSON request and write a JSON result buffer.
///
/// # Safety
///
/// `tag_values` must point to a valid NUL-terminated UTF-8 JSON string and
/// `results` must be writable for `results_capacity` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_write_tags_batch(
    client_id: c_int,
    tag_values: *const c_char,
    tag_count: c_int,
    results: *mut c_char,
    results_capacity: c_int,
) -> c_int {
    if tag_values.is_null() || results.is_null() || tag_count <= 0 || results_capacity <= 0 {
        return fail_with_last_error(client_id, "invalid batch-write arguments");
    }

    let mut client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => return fail_with_last_error(client_id, "client id not found"),
    };
    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    let input_str = unsafe {
        match CStr::from_ptr(tag_values).to_str() {
            Ok(s) => s,
            Err(_) => {
                return fail_with_last_error(client_id, "batch-write JSON is not valid UTF-8");
            }
        }
    };

    let request_items: Vec<FfiWriteRequestItem> = match serde_json::from_str(input_str) {
        Ok(items) => items,
        Err(e) => return fail_with_last_error(client_id, e.to_string()),
    };

    if request_items.len() != tag_count as usize {
        return fail_with_last_error(client_id, "batch-write count does not match payload length");
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

        match ffi_block_on!(client_id, client.write_tags_batch(&write_refs)) {
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
        Err(e) => return fail_with_last_error(client_id, e.to_string()),
    };

    write_output_buffer_or_last_error(
        client_id,
        results,
        results_capacity,
        &results_data,
        "batch-write",
    )
}

/// Execute a mixed read/write batch from a JSON request.
///
/// # Safety
///
/// `operations` must point to a valid NUL-terminated UTF-8 JSON string and
/// `results` must be writable for `results_capacity` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_execute_batch(
    client_id: c_int,
    operations: *const c_char,
    operation_count: c_int,
    results: *mut c_char,
    results_capacity: c_int,
) -> c_int {
    if operations.is_null() || results.is_null() || operation_count <= 0 || results_capacity <= 0 {
        return fail_with_last_error(client_id, "invalid batch-execute arguments");
    }

    let mut client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => return fail_with_last_error(client_id, "client id not found"),
    };
    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    let input_str = unsafe {
        match CStr::from_ptr(operations).to_str() {
            Ok(s) => s,
            Err(_) => {
                return fail_with_last_error(client_id, "batch-execute JSON is not valid UTF-8");
            }
        }
    };

    let request_items: Vec<FfiExecuteRequestItem> = match serde_json::from_str(input_str) {
        Ok(items) => items,
        Err(e) => return fail_with_last_error(client_id, e.to_string()),
    };

    if request_items.len() != operation_count as usize {
        return fail_with_last_error(
            client_id,
            "batch-execute count does not match payload length",
        );
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
        ffi_block_on!(client_id, client.execute_batch(&valid_operations))
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
                Err(err) => return fail_with_last_error(client_id, err.to_string()),
            };
            if write_output_buffer(results, results_capacity, &results_data).is_err() {
                return fail_with_last_error(
                    client_id,
                    "batch-execute error output does not fit in caller buffer",
                );
            }
            return fail_with_last_error(client_id, error_message);
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
        Err(e) => return fail_with_last_error(client_id, e.to_string()),
    };

    write_output_buffer_or_last_error(
        client_id,
        results,
        results_capacity,
        &results_data,
        "batch-execute",
    )
}

/// Legacy batch-configuration placeholder.
///
/// # Safety
///
/// `_config` must be valid for the layout expected by the caller if this
/// placeholder is ever implemented. The current implementation does not
/// dereference it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_configure_batch_operations(
    _client_id: c_int,
    _config: *const u8,
) -> c_int {
    -1 // Not implemented yet
}

/// Legacy batch-configuration query placeholder.
///
/// # Safety
///
/// `_config` must be writable for the layout expected by the caller if this
/// placeholder is ever implemented. The current implementation does not
/// dereference it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_get_batch_config(_client_id: c_int, _config: *mut u8) -> c_int {
    -1 // Not implemented yet
}

/// Read a UDT tag through the chunked UDT path into a JSON buffer.
///
/// # Safety
///
/// `tag_name` must point to a valid NUL-terminated UTF-8 string and `result`
/// must be writable for `max_size` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_read_udt_chunked(
    client_id: c_int,
    tag_name: *const c_char,
    result: *mut c_char,
    max_size: c_int,
) -> c_int {
    if tag_name.is_null() || result.is_null() || max_size <= 0 {
        return -1;
    }

    // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
    let Ok(tag_name_str) = unsafe { CStr::from_ptr(tag_name) }.to_str() else {
        return -1;
    };

    let mut client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => return -1,
    };

    let value = match ffi_block_on!(client_id, client.read_udt_chunked(tag_name_str)) {
        Ok(PlcValue::Udt(udt_data)) => udt_data,
        Ok(_) => return -1,
        Err(_) => return -1,
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

    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), result as *mut u8, bytes.len());
    }
    0
}

/// Read a UDT member by byte offset into a JSON buffer.
///
/// # Safety
///
/// `udt_name` and `data_type` must point to valid NUL-terminated UTF-8 strings
/// and `result` must be writable for `max_size` bytes.
#[unsafe(no_mangle)]
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

    // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
    let Ok(udt_name_str) = unsafe { CStr::from_ptr(udt_name) }.to_str() else {
        return -1;
    };

    let mut client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => return -1,
    };

    let value = match ffi_block_on!(
        client_id,
        client.read_udt_member_by_offset(
            udt_name_str,
            member_offset as usize,
            member_size as usize,
            data_type as u16,
        )
    ) {
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

    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), result as *mut u8, bytes.len());
    }
    0
}

/// Write a UDT member by byte offset from a string payload.
///
/// # Safety
///
/// `udt_name`, `data_type`, and `value` must point to valid NUL-terminated
/// UTF-8 strings for the duration of the call.
#[unsafe(no_mangle)]
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

    // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
    let Ok(udt_name_str) = unsafe { CStr::from_ptr(udt_name) }.to_str() else {
        return -1;
    };

    // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
    let Ok(value_str) = unsafe { CStr::from_ptr(value) }.to_str() else {
        return -1;
    };

    // Parse the value from JSON
    let plc_value: PlcValue = match serde_json::from_str(value_str) {
        Ok(value) => value,
        Err(_) => return -1,
    };

    let mut client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => return -1,
    };

    match ffi_block_on!(
        client_id,
        client.write_udt_member_by_offset(
            udt_name_str,
            member_offset as usize,
            member_size as usize,
            data_type as u16,
            plc_value,
        )
    ) {
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
///
/// # Safety
///
/// `ptr` must be null or a pointer previously returned by this library through
/// `CString::into_raw` and not already freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_free_string(ptr: *mut c_char) {
    // SAFETY: The pointer being freed was allocated by this library and ownership has returned to Rust.
    unsafe { free_c_string(ptr) };
}

/// Free a UDT definition result allocated by `eip_get_udt_definition`
///
/// # Safety
///
/// `result_ptr` must be null or point to a result structure previously
/// initialized by this library. Its owned pointer fields must not have already
/// been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_free_udt_definition(result_ptr: *mut UdtDefinitionResult) {
    if result_ptr.is_null() {
        return;
    }

    // SAFETY: The output pointer was checked for null and the caller contract requires writable storage for this layout.
    let result = unsafe { &mut *result_ptr };
    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    unsafe {
        free_c_string(result.error_message);
        free_c_string(result.name);
    }

    if !result.members.is_null() {
        if result.member_count > 0 {
            for i in 0..result.member_count as usize {
                // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
                unsafe {
                    let member = result.members.add(i);
                    free_c_string((*member).name);
                }
            }
        }
        // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
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
///
/// # Safety
///
/// `result_ptr` must be null or point to a result structure previously
/// initialized by this library. Its owned pointer fields must not have already
/// been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_free_tag_attributes_result(result_ptr: *mut TagAttributesResult) {
    if result_ptr.is_null() {
        return;
    }

    // SAFETY: The output pointer was checked for null and the caller contract requires writable storage for this layout.
    let result = unsafe { &mut *result_ptr };
    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
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
///
/// # Safety
///
/// `result_ptr` must be null or point to a result structure previously
/// initialized by this library. Its owned pointer fields must not have already
/// been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_free_tag_discovery_result(result_ptr: *mut TagDiscoveryResult) {
    if result_ptr.is_null() {
        return;
    }

    // SAFETY: The output pointer was checked for null and the caller contract requires writable storage for this layout.
    let result = unsafe { &mut *result_ptr };
    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    unsafe {
        free_c_string(result.error_message);
    }

    if !result.tags.is_null() {
        if result.tag_count > 0 {
            for i in 0..result.tag_count as usize {
                // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
                unsafe {
                    let tag = result.tags.add(i);
                    free_c_string((*tag).name);
                    free_c_string((*tag).data_type_name);
                }
            }
        }
        // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
        unsafe {
            libc::free(result.tags as *mut c_void);
        }
    }

    result.error_message = ptr::null_mut();
    result.tags = ptr::null_mut();
    result.tag_count = 0;
}

unsafe fn eip_get_udt_definition_impl(
    client: &mut EipClient,
    client_id: c_int,
    udt_name: *const c_char,
    result_ptr: *mut UdtDefinitionResult,
) -> c_int {
    if udt_name.is_null() || result_ptr.is_null() {
        return -1;
    }

    // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
    let udt_name_cstr = unsafe { CStr::from_ptr(udt_name) };
    let udt_name_str = match udt_name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match ffi_block_on!(client_id, client.get_udt_definition(udt_name_str)) {
        Ok(definition) => {
            // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
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

                // Allocate memory for members. A zero-member UDT uses a null
                // members pointer with member_count = 0.
                let members_ptr = if definition.members.is_empty() {
                    std::ptr::null_mut()
                } else {
                    libc::malloc(std::mem::size_of::<UdtMemberC>() * definition.members.len())
                        as *mut UdtMemberC
                };

                if !definition.members.is_empty() && members_ptr.is_null() {
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
            // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
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
///
/// # Safety
///
/// `udt_name` must point to a valid NUL-terminated UTF-8 string and
/// `result_ptr` must point to writable storage for one `UdtDefinitionResult`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_get_udt_definition_by_id(
    client_id: c_int,
    udt_name: *const c_char,
    result_ptr: *mut UdtDefinitionResult,
) -> c_int {
    if udt_name.is_null() || result_ptr.is_null() {
        return -1;
    }

    let mut client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => return -1,
    };

    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    unsafe { eip_get_udt_definition_impl(&mut client, client_id, udt_name, result_ptr) }
}

unsafe fn eip_get_tag_attributes_impl(
    client: &mut EipClient,
    client_id: c_int,
    tag_name: *const c_char,
    result_ptr: *mut TagAttributesResult,
) -> c_int {
    if tag_name.is_null() || result_ptr.is_null() {
        return -1;
    }

    // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
    let tag_name_cstr = unsafe { CStr::from_ptr(tag_name) };
    let tag_name_str = match tag_name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match ffi_block_on!(client_id, client.get_tag_attributes(tag_name_str)) {
        Ok(attributes) => {
            // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
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
            // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
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
///
/// # Safety
///
/// `tag_name` must point to a valid NUL-terminated UTF-8 string and
/// `result_ptr` must point to writable storage for one `TagAttributesResult`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_get_tag_attributes_by_id(
    client_id: c_int,
    tag_name: *const c_char,
    result_ptr: *mut TagAttributesResult,
) -> c_int {
    if tag_name.is_null() || result_ptr.is_null() {
        return -1;
    }

    let mut client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => return -1,
    };

    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    unsafe { eip_get_tag_attributes_impl(&mut client, client_id, tag_name, result_ptr) }
}

unsafe fn eip_discover_tags_detailed_impl(
    client: &mut EipClient,
    client_id: c_int,
    result_ptr: *mut TagDiscoveryResult,
) -> c_int {
    if result_ptr.is_null() {
        return -1;
    }

    match ffi_block_on!(client_id, client.discover_tags_detailed()) {
        Ok(tags) => {
            // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
            unsafe {
                (*result_ptr).success = true;
                (*result_ptr).error_message = std::ptr::null_mut();
                (*result_ptr).tag_count = tags.len() as c_int;

                // Allocate memory for tag attributes. An empty discovery result
                // uses a null tags pointer with tag_count = 0.
                let tags_ptr = if tags.is_empty() {
                    std::ptr::null_mut()
                } else {
                    libc::malloc(std::mem::size_of::<TagAttributesC>() * tags.len())
                        as *mut TagAttributesC
                };

                if !tags.is_empty() && tags_ptr.is_null() {
                    (*result_ptr).success = false;
                    (*result_ptr).error_message =
                        to_c_string_owned("Failed to allocate memory for tag attributes")
                            .unwrap_or(std::ptr::null_mut());
                    (*result_ptr).tags = std::ptr::null_mut();
                    (*result_ptr).tag_count = 0;
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
            // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
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
///
/// # Safety
///
/// `result_ptr` must point to writable storage for one `TagDiscoveryResult`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eip_discover_tags_detailed_by_id(
    client_id: c_int,
    result_ptr: *mut TagDiscoveryResult,
) -> c_int {
    if result_ptr.is_null() {
        return -1;
    }

    let mut client = match get_client(client_id) {
        Ok(client) => client,
        Err(_) => return -1,
    };

    // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
    unsafe { eip_discover_tags_detailed_impl(&mut client, client_id, result_ptr) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    struct ForceRuntimeInitErrorGuard;

    impl ForceRuntimeInitErrorGuard {
        fn enable() -> Self {
            FORCE_RUNTIME_INIT_ERROR.store(true, std::sync::atomic::Ordering::SeqCst);
            Self
        }
    }

    impl Drop for ForceRuntimeInitErrorGuard {
        fn drop(&mut self) {
            FORCE_RUNTIME_INIT_ERROR.store(false, std::sync::atomic::Ordering::SeqCst);
        }
    }

    #[test]
    fn forced_runtime_init_error_returns_documented_code() {
        let _guard = ForceRuntimeInitErrorGuard::enable();
        let address = CString::new("127.0.0.1:44818").expect("test address should be valid");
        // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
        let rc = unsafe { eip_connect(address.as_ptr()) };
        // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
        let rc_again = unsafe { eip_connect(address.as_ptr()) };

        assert_eq!(rc, EIP_ERROR_RUNTIME_INIT);
        assert_eq!(rc_again, EIP_ERROR_RUNTIME_INIT);
    }

    #[test]
    fn last_error_roundtrips_through_buffer() {
        let client_id = -987; // arbitrary id unused by real clients
        set_last_error(client_id, "boom: tag not found");

        let mut buf = [0_i8; 64];
        let written =
            // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
            unsafe { eip_get_last_error(client_id, buf.as_mut_ptr(), buf.len() as c_int) };
        assert_eq!(written, "boom: tag not found".len() as c_int);

        // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
        let msg = unsafe { CStr::from_ptr(buf.as_ptr()) }
            .to_str()
            .expect("utf-8");
        assert_eq!(msg, "boom: tag not found");
    }

    #[test]
    fn last_error_absent_writes_empty_string() {
        let mut buf = [0x7f_i8; 8];
        // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
        let written = unsafe { eip_get_last_error(-12_345, buf.as_mut_ptr(), buf.len() as c_int) };
        assert_eq!(written, 0);
        assert_eq!(buf[0], 0, "buffer should be an empty C string");
    }

    #[test]
    fn last_error_overflow_returns_error() {
        let client_id = -988;
        set_last_error(
            client_id,
            "this message is definitely longer than the buffer",
        );
        let mut buf = [0_i8; 4];
        // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
        let rc = unsafe { eip_get_last_error(client_id, buf.as_mut_ptr(), buf.len() as c_int) };
        assert_eq!(rc, -1);
    }

    fn panic_probe(client_id: c_int) -> c_int {
        let _: () = ffi_block_on!(client_id, async {
            panic!("ffi test panic");
        });
        0
    }

    #[test]
    fn ffi_block_on_converts_panic_to_last_error() {
        let client_id = -321;
        let rc = panic_probe(client_id);
        assert_eq!(rc, -1);

        let mut buf = [0_i8; 128];
        let written =
            // SAFETY: This raw-pointer operation is covered by the enclosing FFI function contract and preceding validation.
            unsafe { eip_get_last_error(client_id, buf.as_mut_ptr(), buf.len() as c_int) };
        assert!(written > 0);
        // SAFETY: The pointer was checked for null where applicable and the FFI caller contract requires a valid NUL-terminated string.
        let msg = unsafe { CStr::from_ptr(buf.as_ptr()) }
            .to_str()
            .expect("utf-8");
        assert!(msg.contains("internal panic: ffi test panic"));
    }
}
