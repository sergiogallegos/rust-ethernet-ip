mod plc_sim;

use plc_sim::SimulatedPlc;
use rust_ethernet_ip::{EipClient, PlcValue};

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
    assert_eq!(real_val, PlcValue::Real(3.14));

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
