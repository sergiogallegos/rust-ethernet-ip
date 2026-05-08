use bytes::BytesMut;

use crate::protocol::cip::{CipRequest, CipResponse, SendDataRequest};
use crate::protocol::encap::{
    EncapsulationHeader, REGISTER_SESSION, SEND_RR_DATA, SEND_UNIT_DATA, UNREGISTER_SESSION,
};
use crate::protocol::values;
use crate::protocol::{Decode, Encode};
use crate::{PlcValue, UdtData};

fn round_trip_value(value: PlcValue) {
    let mut buf = BytesMut::new();
    value.encode(&mut buf);
    let mut bytes = &buf[..];
    let decoded = PlcValue::decode(&mut bytes).expect("value should decode");
    assert_eq!(decoded, value);
}

#[test]
fn plc_value_bool_round_trips() {
    round_trip_value(PlcValue::Bool(true));
}

#[test]
fn plc_value_sint_round_trips() {
    round_trip_value(PlcValue::Sint(-12));
}

#[test]
fn plc_value_int_round_trips() {
    round_trip_value(PlcValue::Int(-1234));
}

#[test]
fn plc_value_dint_round_trips() {
    round_trip_value(PlcValue::Dint(-123456));
}

#[test]
fn plc_value_lint_round_trips() {
    round_trip_value(PlcValue::Lint(-123456789));
}

#[test]
fn plc_value_usint_round_trips() {
    round_trip_value(PlcValue::Usint(250));
}

#[test]
fn plc_value_uint_round_trips() {
    round_trip_value(PlcValue::Uint(65000));
}

#[test]
fn plc_value_udint_round_trips() {
    round_trip_value(PlcValue::Udint(4_000_000_000));
}

#[test]
fn plc_value_ulint_round_trips() {
    round_trip_value(PlcValue::Ulint(9_000_000_000));
}

#[test]
fn plc_value_real_round_trips() {
    round_trip_value(PlcValue::Real(123.5));
}

#[test]
fn plc_value_lreal_round_trips() {
    round_trip_value(PlcValue::Lreal(123.25));
}

#[test]
fn plc_value_string_round_trips() {
    round_trip_value(PlcValue::String("hello plc".to_string()));
}

#[test]
fn plc_value_udt_round_trips_as_container() {
    round_trip_value(PlcValue::Udt(UdtData {
        symbol_id: 0,
        data: vec![1, 2, 3, 4],
    }));
}

#[test]
fn udt_data_container_round_trips() {
    let data = UdtData {
        symbol_id: 17,
        data: vec![0xAA, 0xBB, 0xCC],
    };
    let mut buf = BytesMut::new();
    data.encode(&mut buf);
    let mut bytes = &buf[..];
    let decoded = UdtData::decode(&mut bytes).expect("udt data should decode");
    assert_eq!(decoded.symbol_id, 0);
    assert_eq!(decoded.data, data.data);
}

#[test]
fn decode_ab_string_payload_fixture() {
    let decoded =
        values::decode_payload(values::STRING, &[5, 0, 0, 0, b'H', b'e', b'l', b'l', b'o'])
            .expect("string should decode");
    assert_eq!(decoded, PlcValue::String("Hello".to_string()));
}

#[test]
fn decode_alt_string_payload_fixture() {
    let decoded = values::decode_payload(values::ALT_STRING, &[3, b'A', b'B', b'C'])
        .expect("alt string should decode");
    assert_eq!(decoded, PlcValue::String("ABC".to_string()));
}

#[test]
fn decode_bool_array_dword_fixture() {
    let decoded = values::decode_payload(values::BOOL_ARRAY_DWORD, &[0x05, 0x00, 0x00, 0x00])
        .expect("packed bool dword should decode");
    assert_eq!(decoded, PlcValue::Udint(5));
}

#[test]
fn write_data_type_uses_udt_symbol_id() {
    let value = PlcValue::Udt(UdtData {
        symbol_id: 0x1234,
        data: vec![1, 2],
    });
    assert_eq!(values::write_data_type(&value), 0x14D4);
}

fn round_trip_header(command: u16) {
    let header = EncapsulationHeader {
        command,
        length: 12,
        session_handle: 0x1122_3344,
        status: 0,
        sender_context: [1, 2, 3, 4, 5, 6, 7, 8],
        options: 0,
    };
    let mut buf = BytesMut::new();
    header.encode(&mut buf);
    let mut bytes = &buf[..];
    let decoded = EncapsulationHeader::decode(&mut bytes).expect("header should decode");
    assert_eq!(decoded, header);
}

#[test]
fn encap_register_session_round_trips() {
    round_trip_header(REGISTER_SESSION);
}

#[test]
fn encap_unregister_session_round_trips() {
    round_trip_header(UNREGISTER_SESSION);
}

#[test]
fn encap_send_rr_data_round_trips() {
    round_trip_header(SEND_RR_DATA);
}

#[test]
fn encap_send_unit_data_round_trips() {
    round_trip_header(SEND_UNIT_DATA);
}

#[test]
fn encap_register_session_pinned_bytes() {
    let mut buf = BytesMut::new();
    EncapsulationHeader::new(REGISTER_SESSION, 4, 0).encode(&mut buf);
    buf.extend_from_slice(&[1, 0, 0, 0]);
    assert_eq!(
        &buf[..],
        &[
            0x65, 0x00, 0x04, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0,
        ]
    );
}

#[test]
fn cip_read_tag_request_round_trips() {
    let request = CipRequest::new(0x4C, vec![0x91, 0x04, b'T', b'a', b'g', b'1'], vec![1, 0]);
    let mut buf = BytesMut::new();
    request.encode(&mut buf);
    assert_eq!(
        &buf[..],
        &[0x4C, 0x03, 0x91, 0x04, b'T', b'a', b'g', b'1', 1, 0]
    );
    let mut bytes = &buf[..];
    assert_eq!(CipRequest::decode(&mut bytes).unwrap(), request);
}

#[test]
fn cip_write_tag_request_round_trips() {
    let request = CipRequest::new(
        0x4D,
        vec![0x91, 0x04, b'T', b'a', b'g', b'1'],
        vec![0xC4, 0x00, 1, 0, 42, 0, 0, 0],
    );
    let mut buf = BytesMut::new();
    request.encode(&mut buf);
    let mut bytes = &buf[..];
    assert_eq!(CipRequest::decode(&mut bytes).unwrap(), request);
}

#[test]
fn cip_batch_request_round_trips() {
    let request = CipRequest::new(
        0x0A,
        vec![0x20, 0x02, 0x24, 0x01],
        vec![2, 0, 6, 0, 14, 0, 0x4C, 2, 0x91, 2, b'A', b'1'],
    );
    let mut buf = BytesMut::new();
    request.encode(&mut buf);
    let mut bytes = &buf[..];
    assert_eq!(CipRequest::decode(&mut bytes).unwrap(), request);
}

#[test]
fn cip_success_response_round_trips() {
    let response = CipResponse {
        service: 0xCC,
        status: 0,
        additional_status: Vec::new(),
        data: vec![0xC4, 0x00, 42, 0, 0, 0],
    };
    let mut buf = BytesMut::new();
    response.encode(&mut buf);
    let mut bytes = &buf[..];
    assert_eq!(CipResponse::decode(&mut bytes).unwrap(), response);
}

#[test]
fn cip_error_response_round_trips() {
    let response = CipResponse {
        service: 0xCC,
        status: 0xFF,
        additional_status: vec![0x2107],
        data: Vec::new(),
    };
    let mut buf = BytesMut::new();
    response.encode(&mut buf);
    assert_eq!(&buf[..], &[0xCC, 0x00, 0xFF, 0x01, 0x07, 0x21]);
    let mut bytes = &buf[..];
    assert_eq!(CipResponse::decode(&mut bytes).unwrap(), response);
}

#[test]
fn cpf_unconnected_send_round_trips() {
    let request = SendDataRequest::unconnected(&[0x4C, 0x02, 0x91, 0x02, b'A', b'1', 1, 0]);
    let mut buf = BytesMut::new();
    request.encode(&mut buf);
    let mut bytes = &buf[..];
    assert_eq!(SendDataRequest::decode(&mut bytes).unwrap(), request);
}

#[test]
fn send_rr_data_header_uses_expected_context() {
    let mut buf = BytesMut::new();
    EncapsulationHeader::send_rr_data(16, 0xAABB_CCDD).encode(&mut buf);
    assert_eq!(&buf[0..2], &[0x6F, 0x00]);
    assert_eq!(&buf[2..4], &[16, 0]);
    assert_eq!(&buf[4..8], &[0xDD, 0xCC, 0xBB, 0xAA]);
    assert_eq!(&buf[12..20], &[1, 2, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn value_payload_write_encoding_matches_pinned_dint() {
    let mut buf = BytesMut::new();
    values::encode_payload(&PlcValue::Dint(0x1234_5678), &mut buf);
    assert_eq!(&buf[..], &[0x78, 0x56, 0x34, 0x12]);
}

#[test]
fn value_type_prefixed_encoding_matches_pinned_string() {
    let mut buf = BytesMut::new();
    PlcValue::String("AB".to_string()).encode(&mut buf);
    assert_eq!(&buf[..8], &[0xCE, 0x00, 2, 0, 0, 0, b'A', b'B']);
    assert_eq!(buf.len(), 88);
}
