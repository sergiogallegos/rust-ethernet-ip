//! Live UDT-layout companion for `docs/validation/SCHEMA_CHANGE_GATE.md`'s
//! "Offline UDT Edit and Download" section.
//!
//! Unlike the array-swap gate (`schema_change_gate_live.rs`), this section's
//! edit is inherently offline: adding/reordering a UDT member requires a
//! full project download, which may or may not survive the client's open
//! TCP/encapsulation session. This tool observes that directly (attempts a
//! reconnect if a post-edit read fails) rather than assuming either
//! outcome. It never edits controller schema itself — every Studio 5000
//! action stays manual, and the tool only pauses on stdin between phases.
//!
//! Uses `read_tag()` (whole-UDT byte payload) and `discover_tags_detailed()`
//! (`template_instance_id`) as the layout-change signal rather than
//! `get_udt_definition()`/`get_tag_attributes()`: on the 1756-L75 fw33 used
//! for this gate's live session, the latter's per-tag Get Attribute List
//! CIP request failed with a path-segment error on a freshly created
//! controller-scope UDT tag even though plain tag reads and the bulk
//! discovery sweep both succeeded — see the dated validation record and
//! CODEX-BJ for the follow-up investigation.
//!
//! ```text
//! cargo run --release --example schema_udt_gate_live -- --allow-writes
//! ```

use rust_ethernet_ip::{EipClient, PlcValue, RoutePath};
use std::env;
use std::error::Error;
use std::io::{self, Write};

struct Args {
    address: String,
    slot: u8,
    tag: String,
    dry_run: bool,
    allow_writes: bool,
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
        tag: "gSchemaUdt".to_string(),
        dry_run: false,
        allow_writes: false,
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
            "--tag" => {
                args.tag = input
                    .next()
                    .ok_or_else(|| invalid("--tag requires a value"))?;
            }
            "--dry-run" => args.dry_run = true,
            "--allow-writes" => args.allow_writes = true,
            unknown => return Err(invalid(format!("unknown argument: {unknown}"))),
        }
    }
    Ok(args)
}

struct UdtSnapshot {
    payload_len: usize,
    template_instance_id: Option<u32>,
    discovered_tag_count: usize,
}

fn describe_snapshot(snapshot: &UdtSnapshot) -> String {
    format!(
        "payload_bytes={} template_instance_id={:?} (out of {} discovered controller tags)",
        snapshot.payload_len, snapshot.template_instance_id, snapshot.discovered_tag_count
    )
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

/// Reads the whole UDT plus the discovery-sourced template instance id,
/// reconnecting once if the existing session no longer responds (the
/// expected failure shape after an offline download that dropped the
/// TCP/encapsulation session). Returns the snapshot and whether a
/// reconnect was required.
async fn snapshot_or_reconnect(
    client: &mut EipClient,
    address: &str,
    slot: u8,
    tag: &str,
) -> Result<(UdtSnapshot, bool), Box<dyn Error + Send + Sync>> {
    match take_snapshot(client, tag).await {
        Ok(snapshot) => Ok((snapshot, false)),
        Err(error) => {
            println!("  initial post-edit read failed ({error}); attempting one reconnect");
            *client = EipClient::with_route_path(address, RoutePath::new().add_slot(slot)).await?;
            let snapshot = take_snapshot(client, tag).await?;
            Ok((snapshot, true))
        }
    }
}

async fn take_snapshot(
    client: &mut EipClient,
    tag: &str,
) -> Result<UdtSnapshot, Box<dyn Error + Send + Sync>> {
    let payload_len = match client.read_tag(tag).await? {
        PlcValue::Udt(data) => data.data.len(),
        other => return Err(invalid(format!("{tag}: expected UDT, received {other:?}"))),
    };
    let discovered = client.discover_tags_detailed().await?;
    let template_instance_id = discovered
        .iter()
        .find(|attributes| attributes.name == tag)
        .and_then(|attributes| attributes.template_instance_id);
    Ok(UdtSnapshot {
        payload_len,
        template_instance_id,
        discovered_tag_count: discovered.len(),
    })
}

async fn run(args: Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Schema UDT-layout live gate companion (Rust)");
    println!(
        "target={} slot={} tag={} allow_writes={}",
        args.address, args.slot, args.tag, args.allow_writes
    );
    println!("This tool never edits controller schema; every Studio 5000 action stays manual.");

    if args.dry_run {
        println!("would-test udt={} phases=warm,edit,restore", args.tag);
        return Ok(());
    }
    if !args.allow_writes {
        return Err(invalid(
            "live mode requires --allow-writes (no PLC writes happen, but this mirrors the array gate's opt-in convention)",
        ));
    }

    let mut client =
        EipClient::with_route_path(&args.address, RoutePath::new().add_slot(args.slot)).await?;
    println!(
        "Phase 0 — connected; healthy={}",
        client.check_health().await
    );

    println!("Phase 1 — warm UDT read and discovery cache");
    let baseline = take_snapshot(&mut client, &args.tag).await?;
    println!("  baseline: {}", describe_snapshot(&baseline));

    let baseline_metrics = client.schema_cache_metrics();
    println!(
        "Phase 2 — baseline schema_cache_metrics: generation={} refreshes={}",
        baseline_metrics.generation, baseline_metrics.refreshes
    );

    pause_for_studio5000(&format!(
        "Pause application writes. Go OFFLINE in Studio 5000, add or reorder a dedicated \
         non-I/O member in the UDT backing '{}', then download the project and confirm online.",
        args.tag
    ))?;

    println!("Phase 3 — post-edit session check and refresh_schema()");
    let (pre_refresh_snapshot, reconnected) =
        snapshot_or_reconnect(&mut client, &args.address, args.slot, &args.tag).await?;
    println!("  session survived without reconnect: {}", !reconnected);
    println!(
        "  read immediately after edit (may still be cache-served pre-refresh): {}",
        describe_snapshot(&pre_refresh_snapshot)
    );

    let generation = client.refresh_schema().await;
    let post_refresh_snapshot = take_snapshot(&mut client, &args.tag).await?;
    println!("  generation now {generation}");
    println!(
        "  post-refresh snapshot: {}",
        describe_snapshot(&post_refresh_snapshot)
    );
    if post_refresh_snapshot.payload_len == baseline.payload_len
        && post_refresh_snapshot.template_instance_id == baseline.template_instance_id
    {
        println!(
            "  NOTE: payload size and template instance id both unchanged — confirm the Studio 5000 edit actually changed the layout"
        );
    } else {
        println!(
            "  layout change observed: payload_bytes {} -> {}, template_instance_id {:?} -> {:?}",
            baseline.payload_len,
            post_refresh_snapshot.payload_len,
            baseline.template_instance_id,
            post_refresh_snapshot.template_instance_id
        );
    }

    pause_for_studio5000(&format!(
        "Restore the UDT backing '{}' to its original member layout offline, download, and \
         confirm online — or, if you intend to keep the added member as the final test fixture, \
         press Enter without changing anything and say so when this tool finishes.",
        args.tag
    ))?;

    println!("Phase 4 — post-restore session check and refresh_schema()");
    let (pre_restore_refresh_snapshot, restore_reconnected) =
        snapshot_or_reconnect(&mut client, &args.address, args.slot, &args.tag).await?;
    println!(
        "  session survived without reconnect: {}",
        !restore_reconnected
    );
    println!(
        "  read immediately after restore (may still be cache-served pre-refresh): {}",
        describe_snapshot(&pre_restore_refresh_snapshot)
    );

    let final_generation = client.refresh_schema().await;
    let final_snapshot = take_snapshot(&mut client, &args.tag).await?;
    println!("  generation now {final_generation}");
    println!(
        "  post-restore-refresh snapshot: {}",
        describe_snapshot(&final_snapshot)
    );
    let restored_to_baseline = final_snapshot.payload_len == baseline.payload_len
        && final_snapshot.template_instance_id == baseline.template_instance_id;
    println!("  matches original baseline: {restored_to_baseline}");

    let final_metrics = client.schema_cache_metrics();
    println!();
    println!("=== Paste into the dated validation record ===");
    println!("edit session survived without reconnect: {}", !reconnected);
    println!(
        "restore session survived without reconnect: {}",
        !restore_reconnected
    );
    println!(
        "generation: {} -> {} (total across both refreshes)",
        baseline_metrics.generation, final_metrics.generation
    );
    println!(
        "refreshes: {} -> {}",
        baseline_metrics.refreshes, final_metrics.refreshes
    );
    println!("baseline:      {}", describe_snapshot(&baseline));
    println!(
        "post-edit:     {}",
        describe_snapshot(&post_refresh_snapshot)
    );
    println!("post-restore:  {}", describe_snapshot(&final_snapshot));
    println!("Rust UDT section: PASS (see notes above for restore/reconnect status)");

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
