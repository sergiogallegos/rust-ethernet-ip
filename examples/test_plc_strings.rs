//! Hardware STRING coverage — standalone STRING tags and STRING members inside UDTs / UDT array
//! elements, controller and program scope, exercising the handle-aware write path so both the
//! built-in `STRING` type and custom string types (own name/length, e.g. `Str82`, `Str400`) are
//! validated through the public API (`write_tag` + `read_string_tag`).
//!
//! Each target is read first, written with a probe value, verified, and restored. Targets that
//! aren't present on the connected controller (e.g. custom `Member6`/`Member7`) are skipped, so
//! the example is safe across test programs.
//!
//! Run: `TEST_PLC_ADDRESS=<ip>:44818 TEST_PLC_SLOT=0 cargo run --release --example test_plc_strings`

use rust_ethernet_ip::{EipClient, PlcValue, RoutePath};
use std::env;

fn plc_address() -> String {
    env::var("TEST_PLC_ADDRESS").unwrap_or_else(|_| "192.168.0.1:44818".to_string())
}

fn plc_slot() -> u8 {
    env::var("TEST_PLC_SLOT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

struct Totals {
    pass: u32,
    fail: u32,
    skip: u32,
}

async fn round_trip(client: &mut EipClient, tag: &str, totals: &mut Totals) {
    // Read the original; skip the target if it isn't present on this controller.
    let original = match client.read_string_tag(tag).await {
        Ok(value) => value,
        Err(_) => {
            println!("  {tag:52} SKIP (not present)");
            totals.skip += 1;
            return;
        }
    };

    let probe = "rust-ethernet-ip STRING coverage probe";
    let outcome = async {
        client
            .write_tag(tag, PlcValue::String(probe.to_string()))
            .await?;
        let read_back = client.read_string_tag(tag).await?;
        Ok::<bool, rust_ethernet_ip::EtherNetIpError>(read_back == probe)
    }
    .await;

    // Restore the original value regardless of the outcome.
    let _ = client
        .write_tag(tag, PlcValue::String(original.clone()))
        .await;

    match outcome {
        Ok(true) => {
            println!("  {tag:52} PASS");
            totals.pass += 1;
        }
        Ok(false) => {
            println!("  {tag:52} FAIL (read-back mismatch)");
            totals.fail += 1;
        }
        Err(error) => {
            println!("  {tag:52} FAIL ({error})");
            totals.fail += 1;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = plc_address();
    let slot = plc_slot();
    println!("STRING coverage against {address} (slot {slot})\n");

    let mut client = EipClient::with_route_path(&address, RoutePath::new().add_slot(slot)).await?;
    let mut totals = Totals {
        pass: 0,
        fail: 0,
        skip: 0,
    };

    for (prefix, label) in [("", "controller"), ("Program:TestProgram.", "program")] {
        println!("[{label} scope]");
        // Standalone STRING.
        round_trip(&mut client, &format!("{prefix}gTest_STRING"), &mut totals).await;
        // STRING members inside a UDT and a UDT array element. Member5 is the standard
        // test-program member; Member6/Member7 are optional custom-string members and are
        // skipped when absent.
        for member in ["Member5_String", "Member6_String", "Member7_String"] {
            round_trip(
                &mut client,
                &format!("{prefix}gTestUDT.{member}"),
                &mut totals,
            )
            .await;
            round_trip(
                &mut client,
                &format!("{prefix}gTestUDT_Array[0].{member}"),
                &mut totals,
            )
            .await;
        }
        println!();
    }

    println!(
        "Summary: pass={} fail={} skip={}  RESULT={}",
        totals.pass,
        totals.fail,
        totals.skip,
        if totals.fail == 0 { "PASS" } else { "FAIL" }
    );
    if totals.fail > 0 {
        std::process::exit(1);
    }
    Ok(())
}
