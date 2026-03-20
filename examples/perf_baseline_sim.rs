#[path = "../tests/plc_sim.rs"]
mod plc_sim;

use plc_sim::SimulatedPlc;
use rust_ethernet_ip::{BatchOperation, EipClient, PlcValue};
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
    iterations: usize,
    metrics: Vec<Metric>,
}

fn parse_iterations() -> usize {
    let mut iterations = 500usize;
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

#[tokio::main]
async fn main() {
    let iterations = parse_iterations();
    let sim = SimulatedPlc::start().await;
    let addr = sim.address.to_string();

    let mut client = EipClient::connect(&addr).await.expect("connect simulator");
    let _ = client.read_tag("DINT_TAG").await.expect("warmup read");

    let single_read_start = Instant::now();
    for _ in 0..iterations {
        let _ = client.read_tag("DINT_TAG").await.expect("single read");
    }
    let single_read = build_metric("single_read", iterations, iterations, single_read_start);

    let single_write_start = Instant::now();
    for i in 0..iterations {
        client
            .write_tag("DINT_TAG", PlcValue::Dint(i as i32))
            .await
            .expect("single write");
    }
    let single_write = build_metric("single_write", iterations, iterations, single_write_start);

    let batch_read_tags = ["DINT_TAG", "REAL_TAG", "BOOL_TAG", "STRING_TAG"];
    let batch_read_start = Instant::now();
    for _ in 0..iterations {
        let _ = client
            .read_tags_batch(&batch_read_tags)
            .await
            .expect("batch read");
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
            ("DINT_TAG", PlcValue::Dint((1000 + i) as i32)),
            ("REAL_TAG", PlcValue::Real((i as f32) * 1.5)),
            ("BOOL_TAG", PlcValue::Bool(i % 2 == 0)),
        ];
        let _ = client.write_tags_batch(&writes).await.expect("batch write");
    }
    let batch_write = build_metric("batch_write", iterations, iterations * 3, batch_write_start);

    let mixed_ops = vec![
        BatchOperation::Read {
            tag_name: "DINT_TAG".to_string(),
        },
        BatchOperation::Write {
            tag_name: "DINT_TAG".to_string(),
            value: PlcValue::Dint(777),
        },
        BatchOperation::Read {
            tag_name: "REAL_TAG".to_string(),
        },
        BatchOperation::Read {
            tag_name: "BOOL_TAG".to_string(),
        },
    ];
    let mixed_start = Instant::now();
    for _ in 0..iterations {
        let _ = client.execute_batch(&mixed_ops).await.expect("mixed execute");
    }
    let mixed_execute = build_metric(
        "mixed_execute",
        iterations,
        iterations * mixed_ops.len(),
        mixed_start,
    );

    let run = PerfRun {
        iterations,
        metrics: vec![
            single_read,
            single_write,
            batch_read,
            batch_write,
            mixed_execute,
        ],
    };

    println!(
        "{}",
        serde_json::to_string(&run).expect("serialize perf run")
    );
}
