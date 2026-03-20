mod plc_sim;

use plc_sim::{SimBehavior, SimulatedPlc};
use rust_ethernet_ip::error::EtherNetIpError;
use rust_ethernet_ip::{BatchError, EipClient, PlcValue};

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
        .write_tag("REAL_TAG", PlcValue::Real(6.28))
        .await
        .expect("write");
    client
        .write_tag("STRING_TAG", PlcValue::String("Updated".to_string()))
        .await
        .expect("write");

    let bool_updated = client.read_tag("BOOL_TAG").await.expect("read");
    assert_eq!(bool_updated, PlcValue::Bool(false));

    let real_updated = client.read_tag("REAL_TAG").await.expect("read");
    assert_eq!(real_updated, PlcValue::Real(6.28));

    let string_updated = client.read_tag("STRING_TAG").await.expect("read");
    assert_eq!(string_updated, PlcValue::String("Updated".to_string()));
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
async fn simulated_plc_read_array_range_dint() {
    let sim = SimulatedPlc::start().await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");
    let values = client
        .read_array_range("DINT_ARRAY", 0, 2)
        .await
        .expect("read range");

    assert_eq!(values, vec![PlcValue::Dint(10), PlcValue::Dint(20)]);
}

#[tokio::test]
async fn simulated_plc_read_array_range_real() {
    let sim = SimulatedPlc::start().await;
    let addr = format!("{}", sim.address);

    let mut client = EipClient::connect(&addr).await.expect("connect");
    let values = client
        .read_array_range("REAL_ARRAY", 0, 2)
        .await
        .expect("read range");

    assert_eq!(values, vec![PlcValue::Real(1.5), PlcValue::Real(2.5)]);
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
    let value = reconnected.read_tag("DINT_TAG").await.expect("read after reconnect");
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
