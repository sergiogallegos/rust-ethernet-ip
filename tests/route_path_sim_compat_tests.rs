mod plc_sim;

use plc_sim::SimulatedPlc;
use rust_ethernet_ip::{BatchOperation, EipClient, PlcValue, RoutePath};

#[tokio::test]
async fn route_path_sim_connect_with_route_roundtrip() {
    let sim = SimulatedPlc::start().await;
    let addr = sim.address.to_string();

    let route = RoutePath::new().add_slot(0);
    let mut client = EipClient::with_route_path(&addr, route)
        .await
        .expect("connect with route");

    let value = client
        .read_tag("DINT_TAG")
        .await
        .expect("read after connect_with_route");
    assert_eq!(value, PlcValue::Dint(1234));

    client
        .write_tag("DINT_TAG", PlcValue::Dint(2468))
        .await
        .expect("write with route");
    let updated = client.read_tag("DINT_TAG").await.expect("read updated");
    assert_eq!(updated, PlcValue::Dint(2468));
}

#[tokio::test]
async fn route_path_sim_set_modify_clear_route_path_works() {
    let sim = SimulatedPlc::start().await;
    let addr = sim.address.to_string();

    let mut client = EipClient::connect(&addr).await.expect("connect");
    assert!(client.get_route_path().is_none());

    client.set_route_path(RoutePath::new().add_slot(0));
    let route = client.get_route_path().expect("route should be set");
    assert_eq!(route.slots, vec![0]);

    client.set_route_path(RoutePath::new().add_slot(0).add_slot(1));
    let updated_route = client.get_route_path().expect("route should be set");
    assert_eq!(updated_route.slots, vec![0, 1]);

    let read = client
        .read_tag("DINT_TAG")
        .await
        .expect("read with modified route");
    assert_eq!(read, PlcValue::Dint(1234));

    client.clear_route_path();
    assert!(client.get_route_path().is_none());
}

#[tokio::test]
async fn route_path_sim_batch_and_mixed_execute_compatibility() {
    let sim = SimulatedPlc::start().await;
    let addr = sim.address.to_string();

    let route = RoutePath::new().add_slot(0).add_slot(1);
    let mut client = EipClient::with_route_path(&addr, route)
        .await
        .expect("connect with route");

    let batch_read = client
        .read_tags_batch(&["DINT_TAG", "REAL_TAG", "BOOL_TAG"])
        .await
        .expect("batch read");
    assert_eq!(batch_read.len(), 3);

    let ops = vec![
        BatchOperation::Read {
            tag_name: "DINT_TAG".to_string(),
        },
        BatchOperation::Write {
            tag_name: "DINT_TAG".to_string(),
            value: PlcValue::Dint(9001),
        },
        BatchOperation::Read {
            tag_name: "DINT_TAG".to_string(),
        },
    ];
    let mixed = client.execute_batch(&ops).await.expect("mixed execute");
    assert_eq!(mixed.len(), 3);

    let final_read = client.read_tag("DINT_TAG").await.expect("final read");
    assert_eq!(final_read, PlcValue::Dint(9001));
}
