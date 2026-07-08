mod plc_sim;

use plc_sim::SimulatedPlc;
use rust_ethernet_ip::{BatchOperation, Client, ConnectionEvent, PlcValue, RetryPolicy};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn actor_client_clone_handles_serialize_requests() {
    let sim = SimulatedPlc::start().await;
    let addr = sim.address.to_string();
    let client = Client::connect(&addr).await.expect("connect actor client");
    let clone = client.clone();

    client
        .write_tag("DINT_TAG", PlcValue::Dint(111))
        .await
        .expect("write through first handle");
    clone
        .write_tag("DINT_TAG", PlcValue::Dint(222))
        .await
        .expect("write through cloned handle");

    let value = client.read_tag("DINT_TAG").await.expect("read final value");
    assert_eq!(value, PlcValue::Dint(222));
}

#[tokio::test]
async fn actor_client_batch_passthrough_preserves_results() {
    let sim = SimulatedPlc::start().await;
    let addr = sim.address.to_string();
    let client = Client::connect(&addr).await.expect("connect actor client");

    let operations = vec![
        BatchOperation::Read {
            tag_name: "DINT_TAG".to_string(),
        },
        BatchOperation::Write {
            tag_name: "DINT_TAG".to_string(),
            value: PlcValue::Dint(333),
        },
        BatchOperation::Read {
            tag_name: "DINT_TAG".to_string(),
        },
    ];

    let results = client
        .execute_batch(&operations)
        .await
        .expect("execute batch");
    assert_eq!(results.len(), 3);

    let final_read = client.read_tag("DINT_TAG").await.expect("final read");
    assert_eq!(final_read, PlcValue::Dint(333));
}

#[tokio::test]
async fn actor_client_emits_connection_events() {
    let sim = SimulatedPlc::start().await;
    let addr = sim.address.to_string();
    let client = Client::connect(&addr).await.expect("connect actor client");
    let mut events = client.events();

    drop(client);
    let mut saw_terminal_event = false;
    for _ in 0..4 {
        match events.recv().await {
            Ok(ConnectionEvent::Disconnected | ConnectionEvent::WorkerStopped) => {
                saw_terminal_event = true;
                break;
            }
            Ok(ConnectionEvent::Connected) => {}
            Err(_) => break,
        }
    }
    assert!(saw_terminal_event, "expected actor terminal event");
}

#[tokio::test]
async fn subscribing_to_events_has_no_side_effect_on_existing_subscribers() {
    let sim = SimulatedPlc::start().await;
    let addr = sim.address.to_string();
    let client = Client::connect(&addr).await.expect("connect actor client");
    let mut first = client.events();

    let _ = timeout(Duration::from_millis(50), first.recv()).await;
    let _second = client.events();

    let result = timeout(Duration::from_millis(50), first.recv()).await;
    assert!(
        result.is_err(),
        "a second subscriber must not inject an event into existing subscribers"
    );
}

#[tokio::test]
async fn retry_client_reads_successful_values() {
    let sim = SimulatedPlc::start().await;
    let addr = sim.address.to_string();
    let client = Client::connect(&addr).await.expect("connect actor client");
    let retrying = client.with_retry(RetryPolicy::constant(3, Duration::from_millis(1)));

    let value = retrying.read_tag("DINT_TAG").await.expect("retry read");
    assert_eq!(value, PlcValue::Dint(1234));
}

#[tokio::test]
async fn retry_client_writes_only_when_enabled() {
    let sim = SimulatedPlc::start().await;
    let addr = sim.address.to_string();
    let client = Client::connect(&addr).await.expect("connect actor client");
    let retrying =
        client.with_retry(RetryPolicy::constant(2, Duration::from_millis(1)).retry_writes(true));

    retrying
        .write_tag("DINT_TAG", PlcValue::Dint(444))
        .await
        .expect("retry write");
    let value = client.read_tag("DINT_TAG").await.expect("read after write");
    assert_eq!(value, PlcValue::Dint(444));
}
