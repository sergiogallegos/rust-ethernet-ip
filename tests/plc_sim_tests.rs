mod plc_sim;

use plc_sim::{SimBehavior, SimulatedPlc};
use rust_ethernet_ip::error::EtherNetIpError;
use rust_ethernet_ip::{BatchError, EipClient, PlcValue};
use std::assert_matches;

#[tokio::test]
async fn simulated_plc_read_write_dint() {
    let sim = SimulatedPlc::start().await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");

    let initial = client.read_tag("DINT_TAG").await.expect("read");
    assert_eq!(initial, PlcValue::Dint(1234));

    client
        .write_tag("DINT_TAG", PlcValue::Dint(4321))
        .await
        .expect("write");

    let updated = client.read_tag("DINT_TAG").await.expect("read");
    assert_eq!(updated, PlcValue::Dint(4321));
}

#[tokio::test]
async fn simulated_plc_read_write_dint_bit_syntax() {
    let sim = SimulatedPlc::start().await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");

    client
        .write_tag("DINT_TAG", PlcValue::Dint(0))
        .await
        .expect("clear DINT");

    assert_eq!(
        client.read_tag("DINT_TAG.0").await.expect("read bit 0"),
        PlcValue::Bool(false)
    );
    assert_eq!(
        client.read_tag("DINT_TAG.15").await.expect("read bit 15"),
        PlcValue::Bool(false)
    );
    assert_eq!(
        client.read_tag("DINT_TAG.31").await.expect("read bit 31"),
        PlcValue::Bool(false)
    );

    client
        .write_tag("DINT_TAG.0", PlcValue::Bool(true))
        .await
        .expect("write bit 0");
    client
        .write_tag("DINT_TAG.15", PlcValue::Bool(true))
        .await
        .expect("write bit 15");
    client
        .write_tag("DINT_TAG.31", PlcValue::Bool(true))
        .await
        .expect("write bit 31");

    assert_eq!(
        client.read_tag("DINT_TAG.0").await.expect("read bit 0"),
        PlcValue::Bool(true)
    );
    assert_eq!(
        client.read_tag("DINT_TAG.15").await.expect("read bit 15"),
        PlcValue::Bool(true)
    );
    assert_eq!(
        client.read_tag("DINT_TAG.31").await.expect("read bit 31"),
        PlcValue::Bool(true)
    );
    assert_eq!(
        client.read_tag("DINT_TAG").await.expect("read DINT"),
        PlcValue::Dint(0x8000_8001u32 as i32)
    );

    client
        .write_tag("DINT_TAG.15", PlcValue::Bool(false))
        .await
        .expect("clear bit 15");

    assert_eq!(
        client.read_tag("DINT_TAG").await.expect("read DINT"),
        PlcValue::Dint(0x8000_0001u32 as i32)
    );
}

#[tokio::test]
async fn simulated_plc_read_write_bool_real_string() {
    let sim = SimulatedPlc::start().await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");

    let bool_val = client.read_tag("BOOL_TAG").await.expect("read");
    assert_eq!(bool_val, PlcValue::Bool(true));

    let real_val = client.read_tag("REAL_TAG").await.expect("read");
    assert_eq!(real_val, PlcValue::Real(3.0));

    let string_val = client.read_tag("STRING_TAG").await.expect("read");
    assert_eq!(string_val, PlcValue::String("Hello PLC".to_string()));

    client
        .write_tag("BOOL_TAG", PlcValue::Bool(false))
        .await
        .expect("write");
    client
        .write_tag("REAL_TAG", PlcValue::Real(std::f32::consts::TAU))
        .await
        .expect("write");
    client
        .write_tag("STRING_TAG", PlcValue::String("Updated".to_string()))
        .await
        .expect("write");

    let bool_updated = client.read_tag("BOOL_TAG").await.expect("read");
    assert_eq!(bool_updated, PlcValue::Bool(false));

    let real_updated = client.read_tag("REAL_TAG").await.expect("read");
    assert_eq!(real_updated, PlcValue::Real(std::f32::consts::TAU));

    let string_updated = client.read_tag("STRING_TAG").await.expect("read");
    assert_eq!(string_updated, PlcValue::String("Updated".to_string()));
}

#[tokio::test]
async fn simulated_plc_string_write_shorter_value_clears_residue() {
    let sim = SimulatedPlc::start().await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");

    client
        .write_tag("STRING_TAG", PlcValue::String("LongerValue".to_string()))
        .await
        .expect("write longer string");
    client
        .write_tag("STRING_TAG", PlcValue::String("Hi".to_string()))
        .await
        .expect("write shorter string");

    let updated = client.read_tag("STRING_TAG").await.expect("read");
    assert_eq!(updated, PlcValue::String("Hi".to_string()));
}

#[tokio::test]
async fn simulated_plc_batch_string_write_read_round_trip() {
    let sim = SimulatedPlc::start().await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");

    let writes = vec![
        ("STRING_TAG", PlcValue::String("Batch Updated".to_string())),
        ("DINT_TAG", PlcValue::Dint(2468)),
    ];
    let write_results = client.write_tags_batch(&writes).await.expect("batch write");
    assert_eq!(write_results.len(), 2);
    for (tag, result) in write_results {
        result.unwrap_or_else(|err| panic!("{tag} batch write failed: {err:?}"));
    }

    let reads = client
        .read_tags_batch(&["STRING_TAG", "DINT_TAG"])
        .await
        .expect("batch read");
    assert_eq!(reads.len(), 2);
    assert_eq!(
        reads[0].1.as_ref().expect("STRING read"),
        &PlcValue::String("Batch Updated".to_string())
    );
    assert_eq!(
        reads[1].1.as_ref().expect("DINT read"),
        &PlcValue::Dint(2468)
    );
}

#[tokio::test]
async fn simulated_plc_read_write_dint_array_element() {
    let sim = SimulatedPlc::start().await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");

    let initial = client.read_tag("DINT_ARRAY[1]").await.expect("read");
    assert_eq!(initial, PlcValue::Dint(20));

    client
        .write_tag("DINT_ARRAY[1]", PlcValue::Dint(55))
        .await
        .expect("write");

    let updated = client.read_tag("DINT_ARRAY[1]").await.expect("read");
    assert_eq!(updated, PlcValue::Dint(55));
}

#[tokio::test]
async fn simulated_plc_write_array_element_member_preserves_member_suffix() {
    let sim = SimulatedPlc::start().await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");

    assert_eq!(
        client
            .read_tag("UDT_ARRAY[3].DINT_MEMBER")
            .await
            .expect("initial read"),
        PlcValue::Dint(30)
    );

    client
        .write_tag("UDT_ARRAY[3].DINT_MEMBER", PlcValue::Dint(77))
        .await
        .expect("write member");

    assert_eq!(
        client
            .read_tag("UDT_ARRAY[3].DINT_MEMBER")
            .await
            .expect("updated read"),
        PlcValue::Dint(77)
    );
}

#[tokio::test]
async fn simulated_plc_bool_array_cross_dword_read_write() {
    let sim = SimulatedPlc::start().await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");

    for (index, expected) in [(0, true), (31, false), (32, false), (33, true), (63, true)] {
        let value = client
            .read_tag(&format!("BOOL_ARRAY[{index}]"))
            .await
            .expect("read bool array element");
        assert_eq!(value, PlcValue::Bool(expected), "index {index}");
    }

    client
        .write_tag("BOOL_ARRAY[5]", PlcValue::Bool(true))
        .await
        .expect("write BOOL_ARRAY[5]");
    client
        .write_tag("BOOL_ARRAY[35]", PlcValue::Bool(false))
        .await
        .expect("write BOOL_ARRAY[35]");
    client
        .write_tag("BOOL_ARRAY[63]", PlcValue::Bool(false))
        .await
        .expect("write BOOL_ARRAY[63]");

    assert_eq!(
        client.read_tag("BOOL_ARRAY[5]").await.expect("read [5]"),
        PlcValue::Bool(true)
    );
    assert_eq!(
        client.read_tag("BOOL_ARRAY[35]").await.expect("read [35]"),
        PlcValue::Bool(false)
    );
    assert_eq!(
        client.read_tag("BOOL_ARRAY[63]").await.expect("read [63]"),
        PlcValue::Bool(false)
    );
}

#[tokio::test]
async fn simulated_plc_batch_bool_array_cross_dword_read_write() {
    let sim = SimulatedPlc::start().await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");

    let writes = vec![
        ("BOOL_ARRAY[5]", PlcValue::Bool(true)),
        ("BOOL_ARRAY[35]", PlcValue::Bool(false)),
        ("BOOL_ARRAY[63]", PlcValue::Bool(false)),
    ];
    let write_results = client.write_tags_batch(&writes).await.expect("batch write");
    assert_eq!(write_results.len(), 3);
    for (tag, result) in write_results {
        result.unwrap_or_else(|err| panic!("{tag} batch write failed: {err:?}"));
    }

    // BOOL_ARRAY[35] shares bit position 3 with BOOL_ARRAY[3], but lives in
    // DWORD[1]. Writing [35] must not alias back onto DWORD[0].
    assert_eq!(
        client.read_tag("BOOL_ARRAY[3]").await.expect("read [3]"),
        PlcValue::Bool(true)
    );

    let reads = client
        .read_tags_batch(&["BOOL_ARRAY[5]", "BOOL_ARRAY[35]", "BOOL_ARRAY[63]"])
        .await
        .expect("batch read");

    assert_eq!(reads.len(), 3);
    assert_eq!(
        reads[0].1.as_ref().expect("BOOL_ARRAY[5] read"),
        &PlcValue::Bool(true)
    );
    assert_eq!(
        reads[1].1.as_ref().expect("BOOL_ARRAY[35] read"),
        &PlcValue::Bool(false)
    );
    assert_eq!(
        reads[2].1.as_ref().expect("BOOL_ARRAY[63] read"),
        &PlcValue::Bool(false)
    );
}

#[tokio::test]
async fn bool_array_dword_index_uses_element_segment() {
    let sim = SimulatedPlc::start().await;
    let addr = format!("{}", sim.address);
    let client = EipClient::connect(&addr).await.expect("connect");

    let request = client.build_read_array_request("BOOL_ARRAY", 1, 1);
    assert!(
        request.windows(2).any(|window| window == [0x28, 0x01]),
        "expected DWORD[1] 8-bit element segment in request: {request:02X?}"
    );
}

#[tokio::test]
async fn simulated_plc_nested_bool_array_element_read_write() {
    let sim = SimulatedPlc::start().await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");

    for (index, expected) in [
        (0, true),
        (1, false),
        (31, false),
        (32, false),
        (33, true),
        (63, true),
    ] {
        let value = client
            .read_tag(&format!("UDT_ARRAY[3].BOOL_NESTED[{index}]"))
            .await
            .expect("read nested bool array element");
        assert_eq!(value, PlcValue::Bool(expected), "index {index}");
    }

    assert_matches!(
        client
            .read_tag("UDT_ARRAY[3].BOOL_NESTED[0]")
            .await
            .expect("read nested bit 0"),
        PlcValue::Bool(_)
    );

    client
        .write_tag("UDT_ARRAY[3].BOOL_NESTED[5]", PlcValue::Bool(true))
        .await
        .expect("write nested [5]");
    client
        .write_tag("UDT_ARRAY[3].BOOL_NESTED[35]", PlcValue::Bool(false))
        .await
        .expect("write nested [35]");
    client
        .write_tag("UDT_ARRAY[3].BOOL_NESTED[63]", PlcValue::Bool(false))
        .await
        .expect("write nested [63]");

    assert_eq!(
        client
            .read_tag("UDT_ARRAY[3].BOOL_NESTED[5]")
            .await
            .expect("read nested [5]"),
        PlcValue::Bool(true)
    );
    assert_eq!(
        client
            .read_tag("UDT_ARRAY[3].BOOL_NESTED[35]")
            .await
            .expect("read nested [35]"),
        PlcValue::Bool(false)
    );
    assert_eq!(
        client
            .read_tag("UDT_ARRAY[3].BOOL_NESTED[63]")
            .await
            .expect("read nested [63]"),
        PlcValue::Bool(false)
    );
}

#[tokio::test]
async fn simulated_plc_read_array_range_dint() {
    let sim = SimulatedPlc::start().await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");
    let values = client
        .read_array_range("DINT_ARRAY", 0, 10)
        .await
        .expect("read range");

    assert_eq!(
        values,
        vec![
            PlcValue::Dint(10),
            PlcValue::Dint(20),
            PlcValue::Dint(30),
            PlcValue::Dint(40),
            PlcValue::Dint(50),
            PlcValue::Dint(60),
            PlcValue::Dint(70),
            PlcValue::Dint(80),
            PlcValue::Dint(90),
            PlcValue::Dint(100),
        ]
    );
}

#[tokio::test]
async fn simulated_plc_read_array_range_real() {
    let sim = SimulatedPlc::start().await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");
    let values = client
        .read_array_range("REAL_ARRAY", 0, 10)
        .await
        .expect("read range");

    assert_eq!(
        values,
        vec![
            PlcValue::Real(1.5),
            PlcValue::Real(2.5),
            PlcValue::Real(3.5),
            PlcValue::Real(4.5),
            PlcValue::Real(5.5),
            PlcValue::Real(6.5),
            PlcValue::Real(7.5),
            PlcValue::Real(8.5),
            PlcValue::Real(9.5),
            PlcValue::Real(10.5),
        ]
    );
}

#[tokio::test]
async fn simulated_plc_get_tag_attributes_known_tag() {
    let sim = SimulatedPlc::start().await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");
    let attrs = client
        .get_tag_attributes("DINT_ARRAY")
        .await
        .expect("attributes");

    assert_eq!(attrs.name, "DINT_ARRAY");
    assert_eq!(attrs.data_type, 0x00C4);
    assert_eq!(attrs.data_type_name, "DINT");
    assert_eq!(attrs.template_instance_id, Some(16));
}

#[tokio::test]
async fn simulated_plc_get_tag_attributes_unknown_tag_returns_error() {
    let sim = SimulatedPlc::start().await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");
    let err = client.get_tag_attributes("MISSING_TAG").await.unwrap_err();

    match err {
        EtherNetIpError::Protocol(message) => assert!(
            message.contains("Get Attribute List") && message.contains("Path segment"),
            "unexpected error message: {message}"
        ),
        other => panic!("expected Protocol error for unknown tag attributes, got: {other:?}"),
    }
}

#[tokio::test]
async fn read_bit_invalid_index_returns_error() {
    let sim = SimulatedPlc::start().await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");
    let err = client.read_bit("DINT_TAG", 32).await.unwrap_err();
    match &err {
        EtherNetIpError::Protocol(msg) => assert!(
            msg.contains("0..32"),
            "expected bit index message, got: {}",
            msg
        ),
        other => panic!("expected Protocol error for bit_index 32, got: {:?}", other),
    }
}

#[tokio::test]
async fn simulated_plc_read_bit_write_bit() {
    let sim = SimulatedPlc::start().await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");
    // DINT_TAG is 1234; bit 0 is 0, bit 1 is 1, bit 2 is 0, etc.
    let bit0 = client.read_bit("DINT_TAG", 0).await.expect("read bit 0");
    assert!(!bit0);
    let bit1 = client.read_bit("DINT_TAG", 1).await.expect("read bit 1");
    assert!(bit1);

    client
        .write_bit("DINT_TAG", 0, true)
        .await
        .expect("write bit 0");
    let bit0_after = client
        .read_bit("DINT_TAG", 0)
        .await
        .expect("read bit 0 after write");
    assert!(bit0_after);
}

#[tokio::test]
async fn simulated_plc_timeout_failure_mode() {
    let sim = SimulatedPlc::start_with_behavior(SimBehavior {
        drop_send_rr_response_after: Some(2),
        ..Default::default()
    })
    .await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");
    let err = client.read_tag("DINT_TAG").await.unwrap_err();

    match err {
        EtherNetIpError::Timeout(duration) => {
            assert_eq!(duration, std::time::Duration::from_secs(10));
        }
        other => panic!("expected timeout error, got: {:?}", other),
    }

    let next_err = client.read_tag("REAL_TAG").await.unwrap_err();
    assert!(
        matches!(next_err, EtherNetIpError::ConnectionLost(_)),
        "expected poisoned connection to fail fast, got: {next_err:?}"
    );
}

#[tokio::test]
async fn simulated_plc_sender_context_mismatch_poisons_connection() {
    let sim = SimulatedPlc::start_with_behavior(SimBehavior {
        corrupt_sender_context_after: Some(2),
        ..Default::default()
    })
    .await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");
    let err = client.read_tag("DINT_TAG").await.unwrap_err();

    match err {
        EtherNetIpError::Protocol(message) => assert!(
            message.contains("sender_context mismatch"),
            "unexpected protocol error: {message}"
        ),
        other => panic!("expected sender_context Protocol error, got: {other:?}"),
    }

    let next_err = client.read_tag("REAL_TAG").await.unwrap_err();
    assert!(
        matches!(next_err, EtherNetIpError::ConnectionLost(_)),
        "expected mismatched context to poison connection, got: {next_err:?}"
    );
}

#[tokio::test]
async fn simulated_plc_manual_reconnect_after_disconnect() {
    let sim = SimulatedPlc::start_with_behavior(SimBehavior {
        disconnect_on_send_rr_after: Some(3),
        ..Default::default()
    })
    .await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");

    let first = client.read_tag("DINT_TAG").await.expect("first read");
    assert_eq!(first, PlcValue::Dint(1234));

    let err = client.read_tag("DINT_TAG").await.unwrap_err();
    assert!(
        matches!(err, EtherNetIpError::Io(_) | EtherNetIpError::Timeout(_)),
        "expected transport-level failure after forced disconnect, got: {err:?}"
    );

    // Industrial clients commonly recover by reconnecting after transport loss.
    let mut reconnected = EipClient::connect(&addr).await.expect("reconnect");
    let value = reconnected
        .read_tag("DINT_TAG")
        .await
        .expect("read after reconnect");
    assert_eq!(value, PlcValue::Dint(1234));
}

#[tokio::test]
async fn simulated_plc_partial_batch_failures_are_isolated() {
    let sim = SimulatedPlc::start_with_behavior(SimBehavior {
        fail_read_tags: vec!["FAIL_TAG".to_string()],
        ..Default::default()
    })
    .await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");
    let results = client
        .read_tags_batch(&["DINT_TAG", "FAIL_TAG"])
        .await
        .expect("batch read should complete with per-op results");

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].0, "DINT_TAG");
    assert_eq!(results[1].0, "FAIL_TAG");

    match &results[0].1 {
        Ok(PlcValue::Dint(value)) => assert_eq!(*value, 1234),
        other => panic!("expected successful DINT read, got: {:?}", other),
    }

    match &results[1].1 {
        Err(BatchError::CipError { status, .. }) => {
            assert_eq!(*status, 0x04);
        }
        other => panic!("expected CIP error for failed tag, got: {:?}", other),
    }
}
