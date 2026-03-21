mod plc_sim;

use plc_sim::SimulatedPlc;
use rust_ethernet_ip::ffi;
use serde::Deserialize;
use serde_json::{json, Value};
use std::ffi::{CStr, CString};
use std::os::raw::c_int;
use std::ptr;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

struct FfiClientGuard {
    id: c_int,
}

impl FfiClientGuard {
    fn id(&self) -> c_int {
        self.id
    }
}

impl Drop for FfiClientGuard {
    fn drop(&mut self) {
        if self.id > 0 {
            unsafe {
                let _ = ffi::eip_disconnect(self.id);
            }
        }
    }
}

fn connect_ffi_client(addr: &str) -> FfiClientGuard {
    let addr_c = CString::new(addr).expect("address CString");
    let id = unsafe { ffi::eip_connect(addr_c.as_ptr()) };
    assert!(id > 0, "FFI connect failed for simulator address {addr}");
    FfiClientGuard { id }
}

fn read_c_string_buffer(buf: &[i8]) -> String {
    unsafe { CStr::from_ptr(buf.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

struct SimHarness {
    address: String,
    stop_tx: Option<mpsc::Sender<()>>,
    join_handle: Option<thread::JoinHandle<()>>,
}

impl SimHarness {
    fn start(behavior: plc_sim::SimBehavior) -> Self {
        let (addr_tx, addr_rx) = mpsc::sync_channel::<String>(1);
        let (stop_tx, stop_rx) = mpsc::channel::<()>();

        let join_handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            rt.block_on(async move {
                let sim = SimulatedPlc::start_with_behavior(behavior).await;
                addr_tx.send(sim.address.to_string()).expect("send simulator addr");

                let _ = tokio::task::spawn_blocking(move || {
                    let _ = stop_rx.recv();
                })
                .await;

                drop(sim);
                tokio::time::sleep(Duration::from_millis(25)).await;
            });
        });

        let address = addr_rx.recv().expect("receive simulator addr");
        Self {
            address,
            stop_tx: Some(stop_tx),
            join_handle: Some(join_handle),
        }
    }
}

impl Drop for SimHarness {
    fn drop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

#[derive(Debug, Deserialize)]
struct FfiWriteResultItem {
    tag_name: String,
    success: bool,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FfiExecuteResultItem {
    index: usize,
    tag_name: String,
    is_write: bool,
    success: bool,
    value: Option<Value>,
    error: Option<String>,
    #[serde(rename = "execution_time_us")]
    _execution_time_us: u64,
}

#[test]
fn ffi_null_inputs_return_error() {
    unsafe {
        assert_eq!(ffi::eip_connect(ptr::null()), -1);
        assert_eq!(
            ffi::eip_connect_with_route(
                ptr::null(),
                ptr::null(),
                0,
                ptr::null(),
                0,
                ptr::null_mut(),
                0
            ),
            -1
        );
        assert_eq!(
            ffi::eip_set_route_path(-1, ptr::null(), 0, ptr::null(), 0, ptr::null_mut(), 0),
            -1
        );

        let mut result_int = 0;
        assert_eq!(ffi::eip_read_bool(-1, ptr::null(), &mut result_int), -1);
        assert_eq!(ffi::eip_write_dint(-1, ptr::null(), 123), -1);
        assert_eq!(
            ffi::eip_read_string(-1, ptr::null(), ptr::null_mut(), 0),
            -1
        );
        assert_eq!(
            ffi::eip_read_array_range(-1, ptr::null(), 0, 1, ptr::null_mut(), 0),
            -1
        );
    }
}

#[test]
fn ffi_invalid_client_id_returns_error() {
    let name = CString::new("TestTag").expect("CString");
    unsafe {
        let mut udt_result = std::mem::MaybeUninit::<ffi::UdtDefinitionResult>::zeroed();
        assert_eq!(
            ffi::eip_get_udt_definition_by_id(-1, name.as_ptr(), udt_result.as_mut_ptr()),
            -1
        );

        let mut tag_attr_result = std::mem::MaybeUninit::<ffi::TagAttributesResult>::zeroed();
        assert_eq!(
            ffi::eip_get_tag_attributes_by_id(-1, name.as_ptr(), tag_attr_result.as_mut_ptr()),
            -1
        );

        let mut discovery_result = std::mem::MaybeUninit::<ffi::TagDiscoveryResult>::zeroed();
        assert_eq!(
            ffi::eip_discover_tags_detailed_by_id(-1, discovery_result.as_mut_ptr()),
            -1
        );
    }
}

#[test]
fn ffi_free_helpers_accept_null() {
    unsafe {
        ffi::eip_free_string(ptr::null_mut());
        ffi::eip_free_udt_definition(ptr::null_mut());
        ffi::eip_free_tag_attributes_result(ptr::null_mut());
        ffi::eip_free_tag_discovery_result(ptr::null_mut());
    }
}

#[test]
fn ffi_execute_batch_mixed_contract_keeps_parse_errors_isolated() {
    let sim = SimHarness::start(plc_sim::SimBehavior::default());
    let client = connect_ffi_client(&sim.address);

    let payload = json!([
        { "tag_name": "DINT_TAG", "is_write": false },
        { "tag_name": "DINT_TAG", "is_write": true, "value": 77 }
    ])
    .to_string();
    let payload_c = CString::new(payload).expect("payload CString");

    let mut result_buf = vec![0i8; 16_384];
    let rc = unsafe {
        ffi::eip_execute_batch(
            client.id(),
            payload_c.as_ptr(),
            2,
            result_buf.as_mut_ptr(),
            result_buf.len() as c_int,
        )
    };
    assert_eq!(rc, 0);

    let output_json = read_c_string_buffer(&result_buf);
    let output: Vec<FfiExecuteResultItem> =
        serde_json::from_str(&output_json).expect("execute batch json");

    assert_eq!(output.len(), 2);
    assert_eq!(output[0].index, 0);
    assert_eq!(output[0].tag_name, "DINT_TAG");
    assert!(!output[0].is_write);
    assert!(output[0].success);
    assert!(output[0].value.is_some());
    assert!(output[0].error.is_none());

    assert_eq!(output[1].index, 1);
    assert_eq!(output[1].tag_name, "DINT_TAG");
    assert!(output[1].is_write);
    assert!(!output[1].success);
    assert!(output[1].value.is_none());
    assert!(
        output[1]
            .error
            .as_deref()
            .unwrap_or("")
            .contains("Missing value_type for write operation")
    );
}

#[test]
fn ffi_execute_batch_rejects_malformed_payloads() {
    let sim = SimHarness::start(plc_sim::SimBehavior::default());
    let client = connect_ffi_client(&sim.address);

    let malformed = CString::new("{ this is not valid json ]").expect("malformed CString");
    let mut result_buf = vec![0i8; 4096];
    let malformed_rc = unsafe {
        ffi::eip_execute_batch(
            client.id(),
            malformed.as_ptr(),
            1,
            result_buf.as_mut_ptr(),
            result_buf.len() as c_int,
        )
    };
    assert_eq!(malformed_rc, -1);

    let count_mismatch = CString::new(
        json!([
            { "tag_name": "DINT_TAG", "is_write": false }
        ])
        .to_string(),
    )
    .expect("count_mismatch CString");
    let mismatch_rc = unsafe {
        ffi::eip_execute_batch(
            client.id(),
            count_mismatch.as_ptr(),
            2,
            result_buf.as_mut_ptr(),
            result_buf.len() as c_int,
        )
    };
    assert_eq!(mismatch_rc, -1);
}

#[test]
fn ffi_write_tags_batch_reports_per_item_parse_errors() {
    let sim = SimHarness::start(plc_sim::SimBehavior::default());
    let client = connect_ffi_client(&sim.address);

    let payload = CString::new(
        json!([
            { "tag_name": "DINT_TAG", "value_type": "BADTYPE", "value": 123 },
            { "tag_name": "REAL_TAG", "value_type": "REAL", "value": 6.5 }
        ])
        .to_string(),
    )
    .expect("payload CString");

    let mut result_buf = vec![0i8; 4096];
    let rc = unsafe {
        ffi::eip_write_tags_batch(
            client.id(),
            payload.as_ptr(),
            2,
            result_buf.as_mut_ptr(),
            result_buf.len() as c_int,
        )
    };
    assert_eq!(rc, 0);

    let output_json = read_c_string_buffer(&result_buf);
    let output: Vec<FfiWriteResultItem> =
        serde_json::from_str(&output_json).expect("write batch json");
    assert_eq!(output.len(), 2);

    let bad = output
        .iter()
        .find(|x| x.tag_name == "DINT_TAG" && x.error.is_some())
        .expect("bad parse result");
    assert!(!bad.success);
    assert!(
        bad.error
            .as_deref()
            .unwrap_or("")
            .contains("Unsupported value_type")
    );

    let ok = output
        .iter()
        .find(|x| x.success)
        .expect("successful write result");
    assert_eq!(ok.tag_name, "REAL_TAG");
    assert!(ok.error.is_none());
}

#[test]
fn ffi_write_tags_batch_rejects_malformed_payloads() {
    let sim = SimHarness::start(plc_sim::SimBehavior::default());
    let client = connect_ffi_client(&sim.address);

    let malformed = CString::new("[").expect("malformed CString");
    let mut result_buf = vec![0i8; 4096];
    let malformed_rc = unsafe {
        ffi::eip_write_tags_batch(
            client.id(),
            malformed.as_ptr(),
            1,
            result_buf.as_mut_ptr(),
            result_buf.len() as c_int,
        )
    };
    assert_eq!(malformed_rc, -1);

    let count_mismatch = CString::new(
        json!([
            { "tag_name": "DINT_TAG", "value_type": "DINT", "value": 1 }
        ])
        .to_string(),
    )
    .expect("count_mismatch CString");
    let mismatch_rc = unsafe {
        ffi::eip_write_tags_batch(
            client.id(),
            count_mismatch.as_ptr(),
            2,
            result_buf.as_mut_ptr(),
            result_buf.len() as c_int,
        )
    };
    assert_eq!(mismatch_rc, -1);
}

#[test]
fn ffi_batch_config_apis_are_explicitly_unsupported() {
    let sim = SimHarness::start(plc_sim::SimBehavior::default());
    let client = connect_ffi_client(&sim.address);

    let mut out_cfg: u8 = 0;
    let in_cfg: u8 = 1;

    let configure_rc = unsafe {
        ffi::eip_configure_batch_operations(client.id(), &in_cfg as *const u8)
    };
    assert_eq!(configure_rc, -1);

    let get_rc = unsafe { ffi::eip_get_batch_config(client.id(), &mut out_cfg as *mut u8) };
    assert_eq!(get_rc, -1);
}

#[test]
fn ffi_read_string_success_returns_zero_and_value() {
    let sim = SimHarness::start(plc_sim::SimBehavior::default());
    let client = connect_ffi_client(&sim.address);
    let tag = CString::new("STRING_TAG").expect("tag CString");
    let mut result_buf = vec![0i8; 512];

    let rc = unsafe {
        ffi::eip_read_string(
            client.id(),
            tag.as_ptr(),
            result_buf.as_mut_ptr(),
            result_buf.len() as c_int,
        )
    };

    assert_eq!(rc, 0);
    let output = read_c_string_buffer(&result_buf);
    assert_eq!(output, "Hello PLC");
}

#[test]
fn ffi_read_tag_success_returns_zero_and_json_value() {
    let sim = SimHarness::start(plc_sim::SimBehavior::default());
    let client = connect_ffi_client(&sim.address);
    let tag = CString::new("DINT_TAG").expect("tag CString");
    let mut result_buf = vec![0i8; 1024];

    let rc = unsafe {
        ffi::eip_read_tag(
            client.id(),
            tag.as_ptr(),
            result_buf.as_mut_ptr(),
            result_buf.len() as c_int,
        )
    };

    assert_eq!(rc, 0);
    let output = read_c_string_buffer(&result_buf);
    let parsed: Value = serde_json::from_str(&output).expect("read_tag JSON");
    assert_eq!(parsed["Dint"], json!(1234));
}

#[test]
fn ffi_read_array_range_success_returns_zero_and_values() {
    let sim = SimHarness::start(plc_sim::SimBehavior::default());
    let client = connect_ffi_client(&sim.address);
    let base_array = CString::new("DINT_ARRAY").expect("array CString");
    let mut result_buf = vec![0i8; 2048];

    let rc = unsafe {
        ffi::eip_read_array_range(
            client.id(),
            base_array.as_ptr(),
            0,
            2,
            result_buf.as_mut_ptr(),
            result_buf.len() as c_int,
        )
    };

    assert_eq!(rc, 0);
    let output = read_c_string_buffer(&result_buf);
    let parsed: Value = serde_json::from_str(&output).expect("read_array_range JSON");
    assert!(parsed.is_array());
    let arr = parsed.as_array().expect("array value");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["Dint"], json!(10));
    assert_eq!(arr[1]["Dint"], json!(20));
}
