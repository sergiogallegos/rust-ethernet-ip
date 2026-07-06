#![cfg(feature = "ffi")]

use std::ffi::CStr;

use rust_ethernet_ip::{ffi, version};

#[test]
fn ffi_abi_contract_exports_expected_values() {
    assert_eq!(ffi::eip_abi_version(), version::ABI_VERSION);
    assert_eq!(ffi::eip_abi_version(), 2);

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

#[test]
fn ffi_raw_pointer_exports_are_not_public_abi_symbols() {
    let ffi_source = include_str!("../src/ffi.rs");
    for symbol in [
        "eip_get_udt_definition",
        "eip_get_tag_attributes",
        "eip_discover_tags_detailed",
    ] {
        let exported_signature = format!("pub unsafe extern \"C\" fn {symbol}(");
        assert!(
            !ffi_source.contains(&exported_signature),
            "{symbol} must remain a private helper; callers use the _by_id export"
        );
    }

    for symbol in [
        "eip_get_udt_definition_by_id",
        "eip_get_tag_attributes_by_id",
        "eip_discover_tags_detailed_by_id",
    ] {
        let exported_signature = format!("pub unsafe extern \"C\" fn {symbol}(");
        assert!(
            ffi_source.contains(&exported_signature),
            "{symbol} should remain in the public ABI"
        );
    }
}
