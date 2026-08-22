mod plc_sim;

use plc_sim::{SimBehavior, SimulatedPlc};
use rust_ethernet_ip::{EipClient, PlcValue};

fn bools(len: usize, true_index: usize) -> Vec<bool> {
    (0..len).map(|index| index == true_index).collect()
}

#[tokio::test]
async fn controller_dint_array_to_bool_array_recovers_below_and_above_32() {
    let sim = SimulatedPlc::start().await;
    sim.replace_with_dint_array("SCHEMA_CTL_LOW", 0..64);
    sim.replace_with_dint_array("SCHEMA_CTL_HIGH", 100..164);
    let mut client = EipClient::connect(&sim.address.to_string())
        .await
        .expect("connect");

    assert_eq!(
        client.read_tag("SCHEMA_CTL_LOW[5]").await.unwrap(),
        PlcValue::Dint(5)
    );
    assert_eq!(
        client.read_tag("SCHEMA_CTL_HIGH[40]").await.unwrap(),
        PlcValue::Dint(140)
    );

    sim.replace_with_bool_array("SCHEMA_CTL_LOW", bools(64, 5));
    sim.replace_with_bool_array("SCHEMA_CTL_HIGH", bools(64, 40));

    assert_eq!(
        client.read_tag("SCHEMA_CTL_LOW[5]").await.unwrap(),
        PlcValue::Bool(true)
    );
    assert_eq!(
        client.read_tag("SCHEMA_CTL_HIGH[40]").await.unwrap(),
        PlcValue::Bool(true)
    );
}

#[tokio::test]
async fn program_bool_array_to_dint_array_recovers_above_32() {
    const TAG: &str = "Program:MainProgram.SCHEMA_PROGRAM";
    let sim = SimulatedPlc::start().await;
    sim.replace_with_bool_array(TAG, bools(64, 40));
    let mut client = EipClient::connect(&sim.address.to_string())
        .await
        .expect("connect");

    assert_eq!(
        client.read_tag(&format!("{TAG}[40]")).await.unwrap(),
        PlcValue::Bool(true)
    );
    sim.replace_with_dint_array(TAG, 200..264);
    assert_eq!(
        client.read_tag(&format!("{TAG}[40]")).await.unwrap(),
        PlcValue::Dint(240)
    );
}

#[tokio::test]
async fn compatible_dint_array_to_real_array_needs_no_special_retry() {
    const TAG: &str = "SCHEMA_NUMERIC";
    let sim = SimulatedPlc::start().await;
    sim.replace_with_dint_array(TAG, 0..64);
    let mut client = EipClient::connect(&sim.address.to_string())
        .await
        .expect("connect");

    assert_eq!(
        client.read_tag(&format!("{TAG}[7]")).await.unwrap(),
        PlcValue::Dint(7)
    );
    let before = sim.read_count(TAG);
    sim.replace_with_real_array(TAG, (0..64).map(|index| index as f32 + 0.5));
    assert_eq!(
        client.read_tag(&format!("{TAG}[7]")).await.unwrap(),
        PlcValue::Real(7.5)
    );
    assert_eq!(sim.read_count(TAG) - before, 1);
}

#[tokio::test]
async fn deleted_tag_read_retries_once_then_recovers_after_recreation() {
    const TAG: &str = "SCHEMA_RECREATE";
    let sim = SimulatedPlc::start().await;
    sim.replace_with_dint_array(TAG, 0..64);
    let mut client = EipClient::connect(&sim.address.to_string())
        .await
        .expect("connect");

    client.read_tag(&format!("{TAG}[5]")).await.unwrap();
    sim.remove_tag(TAG);
    let before = sim.read_count(TAG);
    assert!(client.read_tag(&format!("{TAG}[5]")).await.is_err());
    assert_eq!(
        sim.read_count(TAG) - before,
        2,
        "initial read plus one retry"
    );

    sim.replace_with_bool_array(TAG, bools(64, 5));
    assert_eq!(
        client.read_tag(&format!("{TAG}[5]")).await.unwrap(),
        PlcValue::Bool(true)
    );
}

#[tokio::test]
async fn batch_recovers_only_changed_read_and_preserves_result_correlation() {
    const CHANGED: &str = "SCHEMA_BATCH_CHANGED";
    const STABLE: &str = "SCHEMA_BATCH_STABLE";
    let sim = SimulatedPlc::start().await;
    sim.replace_with_dint_array(CHANGED, 0..64);
    sim.replace_with_dint_array(STABLE, 100..164);
    let mut client = EipClient::connect(&sim.address.to_string())
        .await
        .expect("connect");

    client.read_tag(&format!("{CHANGED}[40]")).await.unwrap();
    client.read_tag(&format!("{STABLE}[5]")).await.unwrap();
    sim.replace_with_bool_array(CHANGED, bools(64, 40));

    let changed_path = format!("{CHANGED}[40]");
    let stable_path = format!("{STABLE}[5]");
    let results = client
        .read_tags_batch(&[&changed_path, &stable_path])
        .await
        .expect("batch transport");

    assert_eq!(results[0].0, changed_path);
    assert_eq!(results[0].1.as_ref().unwrap(), &PlcValue::Bool(true));
    assert_eq!(results[1].0, stable_path);
    assert_eq!(results[1].1.as_ref().unwrap(), &PlcValue::Dint(105));
}

#[tokio::test]
async fn failed_write_is_never_replayed_after_send() {
    let sim = SimulatedPlc::start_with_behavior(SimBehavior {
        fail_write_tags: vec!["BOOL_ARRAY".to_string()],
        ..SimBehavior::default()
    })
    .await;
    let mut client = EipClient::connect(&sim.address.to_string())
        .await
        .expect("connect");

    client.read_tag("BOOL_ARRAY[5]").await.unwrap();
    assert!(
        client
            .write_tag("BOOL_ARRAY[5]", PlcValue::Bool(true))
            .await
            .is_err()
    );
    assert_eq!(sim.write_count("BOOL_ARRAY"), 1);
}
