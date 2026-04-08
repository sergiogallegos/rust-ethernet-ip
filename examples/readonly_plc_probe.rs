use rust_ethernet_ip::{EipClient, RoutePath};
use std::env;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "192.168.0.1:44818".to_string());
    let slot = env::var("TEST_PLC_SLOT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    println!("Connecting to {} via slot {}...", address, slot);
    let route = RoutePath::new().add_slot(slot);
    let mut client = match timeout(
        Duration::from_secs(10),
        EipClient::with_route_path(&address, route),
    )
    .await
    {
        Ok(Ok(client)) => client,
        Ok(Err(err)) => return Err(err.into()),
        Err(_) => return Err("Connection timeout".into()),
    };
    println!("Connected.\n");

    let checks = vec![
        "gTestArray_DINT",
        "gTestArray_DINT[0]",
        "gTestArray_DINT[5]",
        "gTestArray_REAL",
        "gTestArray_REAL[0]",
        "gTestArray_BOOL",
        "gTestArray_BOOL[0]",
        "gTestArray_Large",
        "gTestArray_Large[300]",
        "gTestUDT",
        "gTestUDT.Member1_DINT",
        "gTestUDT.Member2_REAL",
        "gTestUDT.Member3_BOOL",
        "gTestUDT.Member4_INT",
        "gTestUDT.Member5_String",
        "gTestUDT.Array_DINT[5]",
        "gTestUDT.Array_REAL[2]",
        "gTestUDT.Array_BOOL[10]",
        "gTestUDT_Array",
        "gTestUDT_Array[3]",
        "gTestUDT_Array[3].Member1_DINT",
        "gTestUDT_Array[3].Member2_REAL",
        "gTestUDT_Array[3].Member3_BOOL",
        "gTestUDT_Array[2].Array_DINT[4]",
        "Program:TestProgram.gTestArray_DINT",
        "Program:TestProgram.gTestArray_DINT[5]",
        "Program:TestProgram.gTestArray_REAL",
        "Program:TestProgram.gTestArray_REAL[0]",
        "Program:TestProgram.gTestArray_BOOL",
        "Program:TestProgram.gTestArray_BOOL[0]",
        "Program:TestProgram.gTestUDT",
        "Program:TestProgram.gTestUDT.Member1_DINT",
        "Program:TestProgram.gTestUDT.Member2_REAL",
        "Program:TestProgram.gTestUDT.Member3_BOOL",
        "Program:TestProgram.gTestUDT.Member4_INT",
        "Program:TestProgram.gTestUDT.Member5_String",
        "Program:TestProgram.gTestUDT.Array_DINT[5]",
        "Program:TestProgram.gTestUDT.Array_REAL[2]",
        "Program:TestProgram.gTestUDT_Array",
        "Program:TestProgram.gTestUDT_Array[2]",
        "Program:TestProgram.gTestUDT_Array[2].Member1_DINT",
        "Program:TestProgram.gTestUDT_Array[2].Member2_REAL",
        "Program:TestProgram.gTestUDT_Array[2].Member3_BOOL",
        "Program:TestProgram.gTestUDT_Array[2].Array_DINT[4]",
    ];

    let mut ok_count = 0usize;
    let mut fail_count = 0usize;

    for tag in checks {
        match timeout(Duration::from_secs(5), client.read_tag(tag)).await {
            Ok(Ok(value)) => {
                ok_count += 1;
                println!("OK   {:<58} {:?}", tag, value);
            }
            Ok(Err(err)) => {
                fail_count += 1;
                println!("MISS {:<58} {}", tag, err);
            }
            Err(_) => {
                fail_count += 1;
                println!("MISS {:<58} timeout", tag);
            }
        }
    }

    println!("\nSummary: {} ok, {} missing/failing", ok_count, fail_count);
    Ok(())
}
