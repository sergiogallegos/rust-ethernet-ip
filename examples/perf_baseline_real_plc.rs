use rust_ethernet_ip::{BatchOperation, EipClient, PlcValue, RoutePath};
use serde::Serialize;
use std::env;
use std::time::Instant;

#[derive(Debug, Serialize, Clone)]
struct Metric {
    name: String,
    iterations: usize,
    logical_ops: usize,
    elapsed_ms: f64,
    avg_call_ms: f64,
    ops_per_sec: f64,
}

#[derive(Debug, Serialize)]
struct PerfRun {
    address: String,
    iterations: usize,
    metrics: Vec<Metric>,
}

fn parse_iterations() -> usize {
    let mut iterations = 100usize;
    let args: Vec<String> = env::args().collect();
    let mut i = 0usize;
    while i < args.len() {
        if args[i] == "--iterations" && i + 1 < args.len() {
            if let Ok(parsed) = args[i + 1].parse::<usize>() {
                iterations = parsed.max(1);
            }
            i += 1;
        }
        i += 1;
    }
    iterations
}

fn build_metric(name: &str, iterations: usize, logical_ops: usize, start: Instant) -> Metric {
    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
    Metric {
        name: name.to_string(),
        iterations,
        logical_ops,
        elapsed_ms,
        avg_call_ms: elapsed_ms / iterations as f64,
        ops_per_sec: logical_ops as f64 / elapsed.as_secs_f64(),
    }
}

fn parse_slot() -> u8 {
    env::var("TEST_PLC_SLOT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let iterations = parse_iterations();
    let address = env::var("TEST_PLC_ADDRESS").unwrap_or_else(|_| "192.168.0.1:44818".to_string());
    let slot = parse_slot();
    let route = RoutePath::new().add_slot(slot);

    let mut client = EipClient::with_route_path(&address, route).await?;

    let single_read_tag = "gTestArray_DINT[0]";
    let batch_read_tags = [
        "gTestArray_DINT[0]",
        "gTestArray_DINT[1]",
        "gTestArray_DINT[2]",
        "gTestArray_DINT[3]",
        "gTestArray_DINT[4]",
        "gTestArray_REAL[0]",
        "gTestArray_REAL[1]",
        "gTestArray_BOOL[0]",
        "gTestArray_INT[0]",
        "gTestUDT.Member1_DINT",
    ];

    let batch_write_targets = [
        "gTestArray_DINT[5]",
        "gTestArray_DINT[6]",
        "gTestArray_DINT[7]",
    ];

    let original_single_write = client.read_tag(batch_write_targets[0]).await?;
    let original_batch_write: Vec<PlcValue> = {
        let mut values = Vec::with_capacity(batch_write_targets.len());
        for tag in &batch_write_targets {
            values.push(client.read_tag(tag).await?);
        }
        values
    };

    let _ = client.read_tag(single_read_tag).await?;
    let _ = client.read_tags_batch(&batch_read_tags).await?;

    let single_read_start = Instant::now();
    for _ in 0..iterations {
        let _ = client.read_tag(single_read_tag).await?;
    }
    let single_read = build_metric("single_read", iterations, iterations, single_read_start);

    let single_write_start = Instant::now();
    for i in 0..iterations {
        client
            .write_tag(batch_write_targets[0], PlcValue::Dint(10_000 + i as i32))
            .await?;
    }
    let single_write = build_metric("single_write", iterations, iterations, single_write_start);

    let batch_read_start = Instant::now();
    for _ in 0..iterations {
        let _ = client.read_tags_batch(&batch_read_tags).await?;
    }
    let batch_read = build_metric(
        "batch_read",
        iterations,
        iterations * batch_read_tags.len(),
        batch_read_start,
    );

    let batch_write_start = Instant::now();
    for i in 0..iterations {
        let writes = [
            (batch_write_targets[0], PlcValue::Dint(20_000 + i as i32)),
            (batch_write_targets[1], PlcValue::Dint(30_000 + i as i32)),
            (batch_write_targets[2], PlcValue::Dint(40_000 + i as i32)),
        ];
        let _ = client.write_tags_batch(&writes).await?;
    }
    let batch_write = build_metric("batch_write", iterations, iterations * 3, batch_write_start);

    let mixed_ops = vec![
        BatchOperation::Read {
            tag_name: "gTestArray_DINT[0]".to_string(),
        },
        BatchOperation::Write {
            tag_name: "gTestArray_DINT[5]".to_string(),
            value: PlcValue::Dint(50_000),
        },
        BatchOperation::Read {
            tag_name: "gTestArray_REAL[0]".to_string(),
        },
        BatchOperation::Read {
            tag_name: "gTestUDT.Member1_DINT".to_string(),
        },
    ];
    let mixed_start = Instant::now();
    for _ in 0..iterations {
        let _ = client.execute_batch(&mixed_ops).await?;
    }
    let mixed_execute = build_metric(
        "mixed_execute",
        iterations,
        iterations * mixed_ops.len(),
        mixed_start,
    );

    client
        .write_tag(batch_write_targets[0], original_single_write)
        .await?;
    let restore_writes: Vec<(&str, PlcValue)> = batch_write_targets
        .iter()
        .zip(original_batch_write.into_iter())
        .map(|(tag, value)| (*tag, value))
        .collect();
    let _ = client.write_tags_batch(&restore_writes).await?;

    let run = PerfRun {
        address,
        iterations,
        metrics: vec![
            single_read,
            single_write,
            batch_read,
            batch_write,
            mixed_execute,
        ],
    };

    println!("{}", serde_json::to_string(&run)?);
    Ok(())
}
