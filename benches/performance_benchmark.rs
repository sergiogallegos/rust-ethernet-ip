use bytes::BytesMut;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
#[cfg(feature = "ffi")]
use rust_ethernet_ip::ffi;
use rust_ethernet_ip::{PlcValue, UdtData};
use rust_ethernet_ip_protocol::cip::{CipRequest, READ_TAG, WRITE_TAG};
use rust_ethernet_ip_protocol::encap::{EncapsulationHeader, REGISTER_SESSION};
use rust_ethernet_ip_protocol::values;
use rust_ethernet_ip_protocol::{Decode, Encode};

fn bench_plc_value_encode(c: &mut Criterion) {
    let values = [
        PlcValue::Bool(true),
        PlcValue::Dint(42),
        PlcValue::Real(42.5),
        PlcValue::String("Pump_A_Ready".to_string()),
        PlcValue::Udt(UdtData {
            symbol_id: 0x1234,
            data: vec![0x11; 128],
        }),
    ];

    let mut group = c.benchmark_group("plc_value_encode");
    for value in values {
        group.bench_with_input(value_label(&value), &value, |b, value| {
            b.iter(|| {
                let mut buf = BytesMut::with_capacity(160);
                black_box(value).encode(&mut buf);
                black_box(buf)
            })
        });
    }
    group.finish();
}

fn bench_plc_value_decode(c: &mut Criterion) {
    let fixtures: [(&str, u16, Vec<u8>); 5] = [
        ("bool", values::BOOL, vec![0xFF]),
        ("dint", values::DINT, 42i32.to_le_bytes().to_vec()),
        ("real", values::REAL, 42.5f32.to_le_bytes().to_vec()),
        ("string", values::STRING, string_payload("Pump_A_Ready")),
        ("udt", values::AB_UDT, vec![0x11; 128]),
    ];

    let mut group = c.benchmark_group("plc_value_decode");
    for (name, data_type, payload) in fixtures {
        group.bench_with_input(name, &(data_type, payload), |b, (data_type, payload)| {
            b.iter(|| black_box(values::decode_payload(*data_type, black_box(payload))).unwrap())
        });
    }
    group.finish();
}

fn bench_encapsulation_header_encode(c: &mut Criterion) {
    let header = EncapsulationHeader::new(REGISTER_SESSION, 4, 0);
    c.bench_function("encapsulation_header_encode", |b| {
        b.iter(|| {
            let mut buf = BytesMut::with_capacity(24);
            black_box(&header).encode(&mut buf);
            black_box(buf)
        })
    });
}

fn bench_cip_request_encode(c: &mut Criterion) {
    let tag_counts = vec![5usize, 10, 25, 50, 100];

    let mut group = c.benchmark_group("cip_batch_request_encode");
    for count in tag_counts {
        group.bench_with_input(BenchmarkId::new("tags", count), &count, |b, &count| {
            let requests = (0..count)
                .map(|i| {
                    CipRequest::new(
                        if i % 2 == 0 { READ_TAG } else { WRITE_TAG },
                        symbolic_path(&format!("Tag{i}")),
                        if i % 2 == 0 {
                            1u16.to_le_bytes().to_vec()
                        } else {
                            let value = PlcValue::Dint(i as i32);
                            let mut data = Vec::new();
                            data.extend_from_slice(&values::write_data_type(&value).to_le_bytes());
                            data.extend_from_slice(&value.to_bytes());
                            data
                        },
                    )
                })
                .collect::<Vec<_>>();

            b.iter(|| {
                let mut buf = BytesMut::with_capacity(count * 32);
                for request in black_box(&requests) {
                    request.encode(&mut buf).unwrap();
                }
                black_box(buf)
            })
        });
    }
    group.finish();
}

fn bench_cip_response_decode(c: &mut Criterion) {
    let mut encoded = BytesMut::new();
    encoded.extend_from_slice(&[0xCC, 0x00, 0x00, 0x00]);
    encoded.extend_from_slice(&values::DINT.to_le_bytes());
    encoded.extend_from_slice(&42i32.to_le_bytes());

    c.bench_function("cip_response_decode_dint", |b| {
        b.iter(|| {
            let mut buf = black_box(encoded.clone());
            let response = rust_ethernet_ip_protocol::cip::CipResponse::decode(&mut buf).unwrap();
            black_box(values::decode_payload(values::DINT, &response.data[2..]).unwrap())
        })
    });
}

fn symbolic_path(tag_name: &str) -> Vec<u8> {
    let mut path = Vec::with_capacity(tag_name.len() + 2);
    path.push(0x91);
    path.push(tag_name.len() as u8);
    path.extend_from_slice(tag_name.as_bytes());
    if !tag_name.len().is_multiple_of(2) {
        path.push(0);
    }
    path
}

fn string_payload(value: &str) -> Vec<u8> {
    let mut payload = Vec::with_capacity(4 + value.len());
    payload.extend_from_slice(&(value.len() as u32).to_le_bytes());
    payload.extend_from_slice(value.as_bytes());
    payload
}

fn value_label(value: &PlcValue) -> &'static str {
    match value {
        PlcValue::Bool(_) => "bool",
        PlcValue::Sint(_) => "sint",
        PlcValue::Int(_) => "int",
        PlcValue::Dint(_) => "dint",
        PlcValue::Lint(_) => "lint",
        PlcValue::Usint(_) => "usint",
        PlcValue::Uint(_) => "uint",
        PlcValue::Udint(_) => "udint",
        PlcValue::Ulint(_) => "ulint",
        PlcValue::Real(_) => "real",
        PlcValue::Lreal(_) => "lreal",
        PlcValue::String(_) => "string",
        PlcValue::Udt(_) => "udt",
    }
}

#[cfg(feature = "ffi")]
fn bench_ffi_state_mutation_overhead(c: &mut Criterion) {
    c.bench_function("ffi_set_max_packet_size_invalid_handle", |b| {
        b.iter(|| unsafe {
            black_box(ffi::eip_set_max_packet_size(black_box(-1), black_box(1500)))
        })
    });
}

#[cfg(not(feature = "ffi"))]
fn bench_ffi_state_mutation_overhead(_c: &mut Criterion) {}

criterion_group!(
    benches,
    bench_plc_value_encode,
    bench_plc_value_decode,
    bench_encapsulation_header_encode,
    bench_cip_request_encode,
    bench_cip_response_decode,
    bench_ffi_state_mutation_overhead
);
criterion_main!(benches);
