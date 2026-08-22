//! Restore-safe companion gate for batch, whole-UDT, and discovery coverage.

use rust_ethernet_ip::{EipClient, PlcValue, RoutePath};
use std::env;
use std::error::Error;
use std::io;

const BATCH_READ_TAGS: &[&str] = &[
    "gTestArray_DINT[0]",
    "gTestArray_INT[0]",
    "gTestArray_REAL[0]",
    "gTestArray_BOOL[0]",
    "gTest_STRING",
    "gTestUDT.Member1_DINT",
    "Program:TestProgram.gTestArray_DINT[0]",
    "Program:TestProgram.gTestUDT.Member1_DINT",
    "Program:TestProgram.gTestArray_REAL[0]",
    "Program:TestProgram.gTestArray_BOOL[0]",
];

const BATCH_WRITE_TAGS: &[&str] = &[
    "gTestArray_DINT[50]",
    "gTestArray_DINT[51]",
    "Program:TestProgram.gTestArray_DINT[50]",
    "Program:TestProgram.gTestArray_DINT[51]",
];

const WHOLE_UDT_TAGS: &[&str] = &[
    "gTestUDT",
    "gTestUDT_Array[0]",
    "Program:TestProgram.gTestUDT",
    "Program:TestProgram.gTestUDT_Array[0]",
];

struct Args {
    address: String,
    slot: u8,
    program: String,
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
        program: env::var("TEST_PLC_PROGRAM").unwrap_or_else(|_| "TestProgram".to_string()),
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
            "--program" => {
                args.program = input
                    .next()
                    .ok_or_else(|| invalid("--program requires a value"))?;
            }
            "--dry-run" => args.dry_run = true,
            "--allow-writes" => args.allow_writes = true,
            unknown => return Err(invalid(format!("unknown argument: {unknown}"))),
        }
    }
    Ok(args)
}

async fn read_dint(client: &mut EipClient, tag: &str) -> Result<i32, Box<dyn Error + Send + Sync>> {
    match client.read_tag(tag).await? {
        PlcValue::Dint(value) => Ok(value),
        other => Err(invalid(format!("{tag}: expected DINT, received {other:?}"))),
    }
}

fn exercise_value(original: i32) -> i32 {
    if original == 123_456_789 {
        123_456_788
    } else {
        123_456_789
    }
}

async fn write_dints(
    client: &mut EipClient,
    values: &[(String, i32)],
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let writes: Vec<(&str, PlcValue)> = values
        .iter()
        .map(|(tag, value)| (tag.as_str(), PlcValue::Dint(*value)))
        .collect();
    let results = client.write_tags_batch(&writes).await?;
    let failures: Vec<String> = results
        .into_iter()
        .filter_map(|(tag, result)| result.err().map(|error| format!("{tag}: {error}")))
        .collect();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(invalid(format!(
            "batch write failures: {}",
            failures.join("; ")
        )))
    }
}

async fn verify_dints(
    client: &mut EipClient,
    expected: &[(String, i32)],
) -> Result<(), Box<dyn Error + Send + Sync>> {
    for (tag, value) in expected {
        let actual = read_dint(client, tag).await?;
        if actual != *value {
            return Err(invalid(format!(
                "{tag}: expected {value} after write, received {actual}"
            )));
        }
    }
    Ok(())
}

async fn diagnose_batch_read_failure(client: &mut EipClient, tags: &[String]) {
    println!("  diagnostic — individual reads");
    for tag in tags {
        match client.read_tag(tag).await {
            Ok(_) => println!("    PASS single: {tag}"),
            Err(error) => println!("    FAIL single: {tag}: {error}"),
        }
    }

    let groups: &[(&str, &[usize])] = &[
        ("controller atomics", &[0, 1, 2, 3]),
        ("controller STRING", &[4]),
        ("controller UDT member", &[5]),
        ("program atomics", &[6, 7, 8, 9]),
        ("cross-scope DINT", &[0, 6]),
        ("all except STRING", &[0, 1, 2, 3, 5, 6, 7, 8, 9]),
    ];
    println!("  diagnostic — batch subsets");
    for (label, indices) in groups {
        let subset: Vec<&str> = indices.iter().map(|index| tags[*index].as_str()).collect();
        match client.read_tags_batch(&subset).await {
            Ok(results) => {
                let failures: Vec<String> = results
                    .into_iter()
                    .filter_map(|(tag, result)| result.err().map(|error| format!("{tag}: {error}")))
                    .collect();
                if failures.is_empty() {
                    println!("    PASS batch {label}: {}/{}", subset.len(), subset.len());
                } else {
                    println!("    FAIL batch {label}: {}", failures.join("; "));
                }
            }
            Err(error) => println!("    FAIL batch {label}: {error}"),
        }
    }
}

async fn run(args: Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Rust companion feature gate");
    println!(
        "target={} slot={} program={}",
        args.address, args.slot, args.program
    );
    println!("writes are limited to four DINT test elements and are restored");

    if args.dry_run {
        println!(
            "would-test binding=rust batch_read={} batch_write={} whole_udt={} controller_discovery=yes program_discovery=yes restore=yes",
            BATCH_READ_TAGS.len(),
            BATCH_WRITE_TAGS.len(),
            WHOLE_UDT_TAGS.len()
        );
        return Ok(());
    }
    if !args.allow_writes {
        return Err(invalid(
            "live mode requires --allow-writes; four dedicated DINT test elements will be changed and restored",
        ));
    }

    let mut client =
        EipClient::with_route_path(&args.address, RoutePath::new().add_slot(args.slot)).await?;

    println!("Phase 1 — controller discovery");
    let controller_tags = client.discover_tags_detailed().await?;
    if !controller_tags.iter().any(|tag| tag.name == "gTestUDT") {
        return Err(invalid("controller discovery did not return gTestUDT"));
    }
    println!("  PASS: {} controller tags", controller_tags.len());

    println!("Phase 2 — program discovery");
    let program_tags = client.discover_program_tags(&args.program).await?;
    if !program_tags.iter().any(|tag| tag.name == "gTestUDT") {
        return Err(invalid(format!(
            "program discovery for {} did not return gTestUDT",
            args.program
        )));
    }
    println!("  PASS: {} program tags", program_tags.len());
    for tag in &program_tags {
        println!("    {} ({})", tag.name, tag.data_type_name);
    }

    println!("Phase 3 — whole UDT reads");
    for tag in WHOLE_UDT_TAGS {
        let tag = tag.replace("TestProgram", &args.program);
        match client.read_tag(&tag).await? {
            PlcValue::Udt(value) if !value.data.is_empty() => {
                println!("  PASS: {tag} ({} payload bytes)", value.data.len());
            }
            PlcValue::Udt(_) => return Err(invalid(format!("{tag}: empty UDT payload"))),
            other => return Err(invalid(format!("{tag}: expected UDT, received {other:?}"))),
        }
    }

    println!("Phase 4 — mixed-type batch read");
    let batch_tags: Vec<String> = BATCH_READ_TAGS
        .iter()
        .map(|tag| tag.replace("TestProgram", &args.program))
        .collect();
    let refs: Vec<&str> = batch_tags.iter().map(String::as_str).collect();
    let results = client.read_tags_batch(&refs).await?;
    let failures: Vec<String> = results
        .into_iter()
        .filter_map(|(tag, result)| result.err().map(|error| format!("{tag}: {error}")))
        .collect();
    if !failures.is_empty() {
        diagnose_batch_read_failure(&mut client, &batch_tags).await;
        return Err(invalid(format!(
            "batch read failures: {}",
            failures.join("; ")
        )));
    }
    println!("  PASS: {}/{}", refs.len(), refs.len());

    println!("Phase 5 — restore-safe batch write");
    let write_tags: Vec<String> = BATCH_WRITE_TAGS
        .iter()
        .map(|tag| tag.replace("TestProgram", &args.program))
        .collect();
    let mut originals = Vec::with_capacity(write_tags.len());
    for tag in &write_tags {
        originals.push((tag.clone(), read_dint(&mut client, tag).await?));
    }
    let exercise: Vec<(String, i32)> = originals
        .iter()
        .map(|(tag, original)| (tag.clone(), exercise_value(*original)))
        .collect();

    let exercise_result = async {
        write_dints(&mut client, &exercise).await?;
        verify_dints(&mut client, &exercise).await
    }
    .await;
    let restore_result = async {
        write_dints(&mut client, &originals).await?;
        verify_dints(&mut client, &originals).await
    }
    .await;

    if let Err(error) = restore_result {
        return Err(invalid(format!("RESTORE FAILED: {error}")));
    }
    println!("  RESTORED: {}/{}", originals.len(), originals.len());
    exercise_result?;
    println!("  PASS: batch write and read-back");

    println!("PASS: Rust companion gate completed with original write values restored");
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
