use rust_ethernet_ip::ffi;
use std::ffi::CString;
use std::ptr;

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
