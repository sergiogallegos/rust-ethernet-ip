#![cfg(feature = "ffi")]

use std::ffi::CStr;

use rust_ethernet_ip::{ffi, version};

#[test]
fn ffi_abi_contract_exports_expected_values() {
    assert_eq!(ffi::eip_abi_version(), version::ABI_VERSION);
    assert_eq!(ffi::eip_abi_version(), 1);

    let version_ptr = ffi::eip_library_version();
    assert!(!version_ptr.is_null());
    let library_version = unsafe { CStr::from_ptr(version_ptr) }
        .to_str()
        .expect("library version is utf-8");
    assert_eq!(library_version, env!("CARGO_PKG_VERSION"));

    let capabilities = ffi::eip_capabilities();
    assert_ne!(capabilities & version::CAP_ROUTE_PATH_ORDERED_HOPS, 0);
    assert_ne!(capabilities & version::CAP_BATCH_EXECUTE_V1, 0);
    assert_ne!(capabilities & version::CAP_DIAGNOSTICS_JSON, 0);
    assert_ne!(capabilities & version::CAP_TAG_GROUP_SUBSCRIPTIONS, 0);
}
