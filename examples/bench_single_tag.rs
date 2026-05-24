use rust_ethernet_ip::{EipClient, EtherNetIpError, PlcValue, RoutePath};
use std::env;
use std::time::Instant;

fn get_plc_address() -> String {
    env::var("TEST_PLC_ADDRESS").unwrap_or_else(|_| "192.168.0.1:44818".to_string())
}

fn get_cpu_slot() -> u8 {
    env::var("TEST_PLC_SLOT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

fn get_iterations() -> usize {
    env::var("BENCH_ITERATIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(500)
}

#[tokio::main]
async fn main() -> Result<(), EtherNetIpError> {
    let address = get_plc_address();
    let slot = get_cpu_slot();
    let iters = get_iterations();
    let tag = "gTestArray_DINT[0]";

    println!("Rust binding single-tag bench");
    println!(
        "PLC: {} (slot {})  tag: {}  iterations: {}",
        address, slot, tag, iters
    );

    let mut client = EipClient::with_route_path(&address, RoutePath::new().add_slot(slot)).await?;

    // Warm-up
    for _ in 0..10 {
        let _ = client.read_tag(tag).await?;
    }

    // Read bench
    let mut read_samples: Vec<u128> = Vec::with_capacity(iters);
    let read_start = Instant::now();
    for _ in 0..iters {
        let t = Instant::now();
        let _ = client.read_tag(tag).await?;
        read_samples.push(t.elapsed().as_micros());
    }
    let read_total = read_start.elapsed();

    // Write bench
    let mut write_samples: Vec<u128> = Vec::with_capacity(iters);
    let write_start = Instant::now();
    for i in 0..iters {
        let t = Instant::now();
        client
            .write_tag(tag, PlcValue::Dint(100_000 + i as i32))
            .await?;
        write_samples.push(t.elapsed().as_micros());
    }
    let write_total = write_start.elapsed();

    // Settle to 999999
    client.write_tag(tag, PlcValue::Dint(999_999)).await?;

    report(
        "read",
        iters,
        read_total.as_secs_f64() * 1000.0,
        &mut read_samples,
    );
    report(
        "write",
        iters,
        write_total.as_secs_f64() * 1000.0,
        &mut write_samples,
    );

    Ok(())
}

fn report(name: &str, iters: usize, total_ms: f64, samples: &mut [u128]) {
    samples.sort_unstable();
    let p50 = samples[samples.len() / 2] as f64 / 1000.0;
    let p95 = samples[(samples.len() * 95) / 100] as f64 / 1000.0;
    let p99 = samples[(samples.len() * 99) / 100] as f64 / 1000.0;
    let avg = total_ms / iters as f64;
    let ops = iters as f64 / (total_ms / 1000.0);
    println!(
        "{:<6} n={}  total={:.1}ms  avg={:.3}ms  p50={:.3}ms  p95={:.3}ms  p99={:.3}ms  ops/sec={:.1}",
        name, iters, total_ms, avg, p50, p95, p99, ops
    );
}
