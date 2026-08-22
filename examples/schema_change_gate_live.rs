//! Live companion runner for `docs/validation/SCHEMA_CHANGE_GATE.md`.
//!
//! Automates the repeatable, non-editing steps of the schema-change
//! validation procedure against a real controller: baseline capture,
//! post-edit read/recovery observation, explicit refresh, rediscovery, and
//! restore-safe write verification. Every Studio 5000 action (deleting a
//! tag, renaming a replacement, downloading a project) stays manual and
//! maintainer-controlled — this tool only pauses on stdin between phases and
//! never issues a schema edit itself.
//!
//! ```text
//! cargo run --release --example schema_change_gate_live -- --allow-writes
//! ```

use rust_ethernet_ip::{EipClient, PlcValue, RoutePath, SchemaCacheMetrics};
use std::env;
use std::error::Error;
use std::io::{self, Write};

const INDICES: [usize; 2] = [5, 40];

struct Args {
    address: String,
    slot: u8,
    program: String,
    tag: String,
    allow_writes: bool,
    dry_run: bool,
}

fn invalid(message: impl Into<String>) -> Box<dyn Error + Send + Sync> {
    Box::new(io::Error::other(message.into()))
}

fn parse_args() -> Result<Args, Box<dyn Error + Send + Sync>> {
    let mut args = Args {
        address: env::var("TEST_PLC_ADDRESS").unwrap_or_else(|_| "192.168.0.1:44818".to_string()),
        slot: env::var("TEST_PLC_SLOT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(0),
        program: env::var("TEST_PLC_PROGRAM").unwrap_or_else(|_| "TestProgram".to_string()),
        tag: "gSchemaSwap".to_string(),
        allow_writes: false,
        dry_run: false,
    };

    let mut input = env::args().skip(1);
    while let Some(argument) = input.next() {
        match argument.as_str() {
            "--plc-address" => {
                args.address = input
                    .next()
                    .ok_or_else(|| invalid("--plc-address requires a value"))?;
            }
            "--plc-slot" => {
                args.slot = input
                    .next()
                    .ok_or_else(|| invalid("--plc-slot requires a value"))?
                    .parse()?;
            }
            "--program" => {
                args.program = input
                    .next()
                    .ok_or_else(|| invalid("--program requires a value"))?;
            }
            "--tag" => {
                args.tag = input
                    .next()
                    .ok_or_else(|| invalid("--tag requires a value"))?;
            }
            "--allow-writes" => args.allow_writes = true,
            "--dry-run" => args.dry_run = true,
            unknown => return Err(invalid(format!("unknown argument: {unknown}"))),
        }
    }
    Ok(args)
}

fn pause_for_studio5000(message: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!();
    println!("=== MAINTAINER ACTION REQUIRED ===");
    println!("{message}");
    println!("This tool never edits controller schema. Perform the Studio 5000 action now.");
    print!("Press Enter once the change is downloaded and online: ");
    io::stdout().flush().ok();
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|error| invalid(error.to_string()))?;
    Ok(())
}

fn describe(value: &PlcValue) -> String {
    match value {
        PlcValue::Dint(v) => format!("Dint({v})"),
        PlcValue::Bool(v) => format!("Bool({v})"),
        PlcValue::Real(v) => format!("Real({v})"),
        other => format!("{other:?}"),
    }
}

async fn read_element(
    client: &mut EipClient,
    path: &str,
) -> Result<PlcValue, Box<dyn Error + Send + Sync>> {
    Ok(client.read_tag(path).await?)
}

/// Produces a distinguishable probe value of the same type, for a
/// restore-safe write/read-back check. Only the two shapes this gate swaps
/// between (`DINT[]` and packed `BOOL[]`) are supported.
fn exercise(value: &PlcValue) -> Result<PlcValue, Box<dyn Error + Send + Sync>> {
    match value {
        PlcValue::Dint(v) => Ok(PlcValue::Dint(if *v == 123_456_789 {
            123_456_788
        } else {
            123_456_789
        })),
        PlcValue::Bool(v) => Ok(PlcValue::Bool(!v)),
        other => Err(invalid(format!(
            "unsupported schema-swap element type for a write probe: {other:?}"
        ))),
    }
}

async fn write_and_verify(
    client: &mut EipClient,
    path: &str,
    value: PlcValue,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    client.write_tag(path, value.clone()).await?;
    let read_back = read_element(client, path).await?;
    if read_back != value {
        return Err(invalid(format!(
            "{path}: wrote {value:?}, read back {read_back:?}"
        )));
    }
    Ok(())
}

fn metrics_delta(label: &str, before: &SchemaCacheMetrics, after: &SchemaCacheMetrics) {
    println!("  {label}:");
    println!(
        "    generation: {} -> {} ({:+})",
        before.generation,
        after.generation,
        after.generation as i64 - before.generation as i64
    );
    println!(
        "    refreshes: {} -> {} ({:+})",
        before.refreshes,
        after.refreshes,
        after.refreshes as i64 - before.refreshes as i64
    );
    println!(
        "    array classification hits/misses/evictions: {}/{}/{} -> {}/{}/{}",
        before.array_classification_hits,
        before.array_classification_misses,
        before.array_classification_evictions,
        after.array_classification_hits,
        after.array_classification_misses,
        after.array_classification_evictions
    );
    println!(
        "    datatype contradictions: {} -> {} ({:+})",
        before.datatype_contradictions,
        after.datatype_contradictions,
        after.datatype_contradictions as i64 - before.datatype_contradictions as i64
    );
    println!(
        "    read recoveries succeeded/failed: {}/{} -> {}/{}",
        before.successful_read_recoveries,
        before.failed_read_recoveries,
        after.successful_read_recoveries,
        after.failed_read_recoveries
    );
}

async fn run(args: Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Schema-change live gate companion (Rust)");
    println!(
        "target={} slot={} program={} tag={} allow_writes={}",
        args.address, args.slot, args.program, args.tag, args.allow_writes
    );
    println!("This tool never edits controller schema; every Studio 5000 action stays manual.");

    if args.dry_run {
        println!(
            "would-test scopes=controller,program indices={INDICES:?} allow_writes={}",
            args.allow_writes
        );
        return Ok(());
    }

    let mut client =
        EipClient::with_route_path(&args.address, RoutePath::new().add_slot(args.slot)).await?;
    println!(
        "Phase 0 — connected; healthy={}",
        client.check_health().await
    );

    let scopes: [(&str, String); 2] = [
        ("controller", args.tag.clone()),
        ("program", format!("Program:{}.{}", args.program, args.tag)),
    ];

    let baseline_metrics = client.schema_cache_metrics();
    println!(
        "Phase 1 — baseline schema_cache_metrics: generation={} refreshes={}",
        baseline_metrics.generation, baseline_metrics.refreshes
    );

    println!("Phase 2 — pre-edit reads (twice, to warm classification cache)");
    let mut pre_edit_values: Vec<(String, PlcValue)> = Vec::new();
    for (scope_name, base) in &scopes {
        for index in INDICES {
            let path = format!("{base}[{index}]");
            let first = read_element(&mut client, &path).await?;
            let second = read_element(&mut client, &path).await?;
            if first != second {
                return Err(invalid(format!(
                    "{path}: unstable read before any edit: {first:?} then {second:?}"
                )));
            }
            println!("  {scope_name} {path} = {}", describe(&second));
            pre_edit_values.push((path, second));
        }
    }

    if args.allow_writes {
        println!("Phase 3 — restore-safe pre-edit write smoke check");
        for (path, original) in &pre_edit_values {
            let probe = exercise(original)?;
            write_and_verify(&mut client, path, probe).await?;
            write_and_verify(&mut client, path, original.clone()).await?;
            println!("  {path}: exercised and restored to {}", describe(original));
        }
    } else {
        println!(
            "Phase 3 — skipped (pass --allow-writes to smoke-check writes before/after the edit)"
        );
    }

    pause_for_studio5000(&format!(
        "Move any test-only references off '{tag}', delete the unused original, and rename the \
         replacement to '{tag}' — for both controller and program scope.",
        tag = args.tag
    ))?;

    println!("Phase 4 — post-edit reads without calling refresh_schema() first");
    let pre_refresh_metrics = client.schema_cache_metrics();
    let mut post_edit_values: Vec<(String, PlcValue)> = Vec::new();
    for (scope_name, base) in &scopes {
        for index in INDICES {
            let path = format!("{base}[{index}]");
            match read_element(&mut client, &path).await {
                Ok(value) => {
                    println!(
                        "  {scope_name} {path} = {} (automatic recovery applies if the type changed)",
                        describe(&value)
                    );
                    post_edit_values.push((path, value));
                }
                Err(error) => {
                    println!("  {scope_name} {path}: read error before refresh: {error}");
                }
            }
        }
    }
    let post_read_metrics = client.schema_cache_metrics();
    metrics_delta(
        "automatic recovery (no explicit refresh yet)",
        &pre_refresh_metrics,
        &post_read_metrics,
    );

    println!("Phase 5 — explicit refresh_schema()");
    let generation = client.refresh_schema().await;
    let post_refresh_metrics = client.schema_cache_metrics();
    if post_refresh_metrics.generation != pre_refresh_metrics.generation + 1
        || post_refresh_metrics.refreshes != pre_refresh_metrics.refreshes + 1
    {
        return Err(invalid(format!(
            "refresh_schema() did not advance generation/refresh count by exactly one: before={pre_refresh_metrics:?} after={post_refresh_metrics:?}"
        )));
    }
    println!("  generation now {generation}");

    println!("Phase 6 — rediscovery");
    match client.discover_tags_detailed().await {
        Ok(tags) => println!(
            "  controller discovery: {} tags, {} match '{}'",
            tags.len(),
            tags.iter().filter(|tag| tag.name == args.tag).count(),
            args.tag
        ),
        Err(error) => println!("  controller discovery failed (non-fatal): {error}"),
    }
    match client.discover_program_tags(&args.program).await {
        Ok(tags) => println!(
            "  program discovery: {} tags, {} match '{}'",
            tags.len(),
            tags.iter().filter(|tag| tag.name == args.tag).count(),
            args.tag
        ),
        Err(error) => println!("  program discovery failed (non-fatal): {error}"),
    }

    println!("Phase 7 — post-refresh reads");
    let mut post_refresh_values: Vec<(String, PlcValue)> = Vec::new();
    for (scope_name, base) in &scopes {
        for index in INDICES {
            let path = format!("{base}[{index}]");
            let value = read_element(&mut client, &path).await?;
            println!("  {scope_name} {path} = {}", describe(&value));
            post_refresh_values.push((path, value));
        }
    }

    if args.allow_writes {
        println!("Phase 8 — restore-safe post-refresh write/verify");
        for (path, current) in &post_refresh_values {
            let probe = exercise(current)?;
            write_and_verify(&mut client, path, probe).await?;
            write_and_verify(&mut client, path, current.clone()).await?;
            println!(
                "  {path}: exercised the new addressing shape and restored to {}",
                describe(current)
            );
        }
    } else {
        println!(
            "Phase 8 — skipped (pass --allow-writes to verify the new addressing shape and restore)"
        );
    }

    let final_metrics = client.schema_cache_metrics();
    println!();
    println!("=== Paste into the dated validation record ===");
    println!(
        "session survived: yes (single connection held for the entire run; healthy={})",
        client.check_health().await
    );
    metrics_delta("baseline -> final", &baseline_metrics, &final_metrics);
    println!("Rust: PASS");

    Ok(())
}

#[tokio::main]
async fn main() {
    let result = match parse_args() {
        Ok(args) => run(args).await,
        Err(error) => Err(error),
    };
    if let Err(error) = result {
        eprintln!("FAIL: {error}");
        std::process::exit(1);
    }
}
