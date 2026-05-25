mod plc_sim;

use plc_sim::SimulatedPlc;
use rust_ethernet_ip::{ConnectionEvent, Fleet, PlcValue};

#[tokio::test]
async fn fleet_connects_and_returns_cloneable_client() {
    let sim = SimulatedPlc::start().await;
    let addr = sim.address.to_string();
    let mut fleet = Fleet::new();

    fleet
        .connect("line-1".to_string(), &addr)
        .await
        .expect("connect fleet client");
    assert_eq!(fleet.len(), 1);

    let client = fleet.client(&"line-1".to_string()).expect("fleet client");
    client
        .write_tag("DINT_TAG", PlcValue::Dint(555))
        .await
        .expect("write through fleet client");
    let value = client.read_tag("DINT_TAG").await.expect("read back");
    assert_eq!(value, PlcValue::Dint(555));
}

#[tokio::test]
async fn fleet_forwards_connection_events_with_plc_id() {
    let sim = SimulatedPlc::start().await;
    let addr = sim.address.to_string();
    let mut fleet = Fleet::new();
    let mut events = fleet.events();

    let client = fleet
        .connect("plc-a".to_string(), &addr)
        .await
        .expect("connect fleet client");

    let connected = events.recv().await.expect("fleet connected event");
    assert_eq!(connected.plc_id, "plc-a");
    assert_eq!(connected.event, ConnectionEvent::Connected);

    drop(client);
    drop(fleet);
}

#[tokio::test]
async fn fleet_health_check_reports_all_clients() {
    let sim = SimulatedPlc::start().await;
    let addr = sim.address.to_string();
    let mut fleet = Fleet::new();

    fleet
        .connect("plc-a".to_string(), &addr)
        .await
        .expect("connect fleet client");

    let health = fleet.check_health().await;
    assert_eq!(health.len(), 1);
    assert!(
        health
            .get("plc-a")
            .expect("plc health result")
            .as_ref()
            .is_ok()
    );
}
