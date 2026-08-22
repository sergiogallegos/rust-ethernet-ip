use rust_ethernet_ip::{EipClient, PlcValue, RoutePath};
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};

fn get_plc_address() -> String {
    env::var("TEST_PLC_ADDRESS").unwrap_or_else(|_| "192.168.0.1:44818".to_string())
}
fn get_cpu_slot() -> u8 {
    env::var("TEST_PLC_SLOT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

struct Args {
    address: String,
    slot: u8,
    manifest_path: PathBuf,
    out_dir: PathBuf,
    dry_run: bool,
    skip_preflight: bool,
    preflight_only: bool,
    benchmark_passes: usize,
    batch_benchmark: bool,
    batch_min_tag_operations: usize,
    batch_min_seconds: f64,
    allow_writes: bool,
}

fn parse_args() -> Args {
    let mut args = Args {
        address: get_plc_address(),
        slot: get_cpu_slot(),
        manifest_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("full_coverage_tags.json"),
        out_dir: PathBuf::from("examples/full_coverage_results"),
        dry_run: false,
        skip_preflight: false,
        preflight_only: false,
        benchmark_passes: 0,
        batch_benchmark: false,
        batch_min_tag_operations: 1_000,
        batch_min_seconds: 30.0,
        allow_writes: false,
    };
    let mut iter = env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--plc-address" => args.address = iter.next().expect("--plc-address requires a value"),
            "--plc-slot" => {
                args.slot = iter
                    .next()
                    .expect("--plc-slot requires a value")
                    .parse()
                    .expect("--plc-slot must be an integer")
            }
            "--manifest" => {
                args.manifest_path =
                    PathBuf::from(iter.next().expect("--manifest requires a value"));
            }
            "--out-dir" => {
                args.out_dir = PathBuf::from(iter.next().expect("--out-dir requires a value"))
            }
            "--dry-run" => args.dry_run = true,
            "--skip-preflight" => args.skip_preflight = true,
            "--preflight-only" => args.preflight_only = true,
            "--benchmark-passes" => {
                args.benchmark_passes = iter
                    .next()
                    .expect("--benchmark-passes requires a value")
                    .parse()
                    .expect("--benchmark-passes must be a positive integer")
            }
            "--allow-writes" => args.allow_writes = true,
            "--batch-benchmark" => args.batch_benchmark = true,
            "--batch-min-tag-operations" => {
                args.batch_min_tag_operations = iter
                    .next()
                    .expect("--batch-min-tag-operations requires a value")
                    .parse()
                    .expect("--batch-min-tag-operations must be a positive integer")
            }
            "--batch-min-seconds" => {
                args.batch_min_seconds = iter
                    .next()
                    .expect("--batch-min-seconds requires a value")
                    .parse()
                    .expect("--batch-min-seconds must be a non-negative number")
            }
            other => eprintln!("warning: ignoring unknown argument {other}"),
        }
    }
    args
}

struct Lcg(u64);
impl Lcg {
    fn seeded() -> Self {
        let n = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0xdead_beef_cafe_f00d);
        Self(n | 1)
    }
    fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((self.0 >> 32) & 0xffff_ffff) as u32
    }
    fn dint(&mut self) -> i32 {
        (self.next_u32() as i32).rem_euclid(900_000) + 1_000
    }
    fn int(&mut self) -> i16 {
        ((self.next_u32() & 0x7fff) as i16).rem_euclid(20_000) + 100
    }
    fn real(&mut self) -> f32 {
        (self.next_u32() % 9_000_000) as f32 / 1_000.0 + 1.0
    }
    fn boolean(&mut self) -> bool {
        self.next_u32() & 1 == 1
    }
}

#[derive(Clone, Copy, Deserialize, PartialEq, Eq, Hash)]
enum Kind {
    Dint,
    Int,
    Real,
    Bool,
    String,
    Udt,
}

#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum WriteMode {
    Writeable,
    ReadOnly,
    EncodingBlockedUdtStringMember,
    ServiceLayerWriteable,
}

impl WriteMode {
    fn is_writeable(self) -> bool {
        matches!(self, Self::Writeable | Self::ServiceLayerWriteable)
    }

    fn is_expected_blocked(self) -> bool {
        matches!(self, Self::EncodingBlockedUdtStringMember)
    }
}

struct Tag {
    name: String,
    category: String,
    kind: Kind,
    write: WriteMode,
}

#[derive(Deserialize)]
struct Manifest {
    #[serde(rename = "schema_version")]
    _schema_version: u32,
    categories: Vec<ManifestCategory>,
}

#[derive(Deserialize)]
struct ManifestCategory {
    name: String,
    pattern: String,
    kind: Option<Kind>,
    writeability: Option<WriteMode>,
    indices: Option<RangeSpec>,
    outer_indices: Option<RangeSpec>,
    members: Option<BTreeMap<String, ManifestSpec>>,
    inner: Option<BTreeMap<String, ManifestSpec>>,
}

#[derive(Deserialize)]
struct RangeSpec {
    range: [usize; 2],
}

#[derive(Deserialize)]
struct ManifestSpec {
    range: Option<[usize; 2]>,
    kind: Kind,
    writeability: WriteMode,
}

fn build_tags(manifest_path: &Path) -> Result<Vec<Tag>, Box<dyn std::error::Error>> {
    let manifest_text = fs::read_to_string(manifest_path).map_err(|err| {
        std::io::Error::new(
            err.kind(),
            format!(
                "manifest-error: failed to read {}: {}",
                manifest_path.display(),
                err
            ),
        )
    })?;
    let manifest: Manifest = serde_json::from_str(&manifest_text).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "manifest-error: failed to parse {}: {}",
                manifest_path.display(),
                err
            ),
        )
    })?;
    let mut tags = Vec::new();
    for category in manifest.categories {
        tags.extend(expand_category(&category)?);
    }
    Ok(tags)
}

fn expand_category(category: &ManifestCategory) -> Result<Vec<Tag>, Box<dyn std::error::Error>> {
    let mut tags = Vec::new();
    if let Some(members) = &category.members {
        for i in range_or_once(category.indices.as_ref()) {
            for (member, spec) in members {
                tags.push(mk(
                    render_pattern(&category.pattern, Some(i), Some(member), None, None),
                    category.name.clone(),
                    spec.kind,
                    spec.writeability,
                ));
            }
        }
        return Ok(tags);
    }
    if let Some(inner) = &category.inner {
        for i in range_or_once(category.outer_indices.as_ref()) {
            for (field, spec) in inner {
                let range = spec
                    .range
                    .ok_or_else(|| format!("{}.{field} missing range", category.name))?;
                for j in range[0]..range[1] {
                    tags.push(mk(
                        render_pattern(&category.pattern, Some(i), None, Some(field), Some(j)),
                        category.name.clone(),
                        spec.kind,
                        spec.writeability,
                    ));
                }
            }
        }
        return Ok(tags);
    }
    let kind = category
        .kind
        .ok_or_else(|| format!("{} missing kind", category.name))?;
    let write = category
        .writeability
        .ok_or_else(|| format!("{} missing writeability", category.name))?;
    for i in range_or_once(category.indices.as_ref()) {
        tags.push(mk(
            render_pattern(&category.pattern, Some(i), None, None, None),
            category.name.clone(),
            kind,
            write,
        ));
    }
    Ok(tags)
}

fn range_or_once(indices: Option<&RangeSpec>) -> Vec<usize> {
    match indices {
        Some(spec) => (spec.range[0]..spec.range[1]).collect(),
        None => vec![0],
    }
}

fn render_pattern(
    pattern: &str,
    i: Option<usize>,
    member: Option<&str>,
    field: Option<&str>,
    j: Option<usize>,
) -> String {
    let mut out = pattern.to_string();
    if let Some(i) = i {
        out = out.replace("{i}", &i.to_string());
    }
    if let Some(member) = member {
        out = out.replace("{member}", member);
    }
    if let Some(field) = field {
        out = out.replace("{field}", field);
    }
    if let Some(j) = j {
        out = out.replace("{j}", &j.to_string());
    }
    out
}

fn mk(name: String, category: String, kind: Kind, write: WriteMode) -> Tag {
    Tag {
        name,
        category,
        kind,
        write,
    }
}

#[derive(Default)]
struct CatStats {
    read_ok: u32,
    read_fail: u32,
    write_ok: u32,
    write_fail: u32,
    verify_ok: u32,
    verify_fail: u32,
    blocked_as_expected: u32,
    blocked_unexpected_pass: u32,
}

struct RunSummary<'a> {
    args: &'a Args,
    tag_count: usize,
    stats: &'a BTreeMap<String, CatStats>,
    preflight_ok: u32,
    preflight_fail: u32,
    totals: &'a CatStats,
    settle_ok: u32,
    settle_fail: u32,
    settle_verify_ok: u32,
    settle_verify_fail: u32,
    unexpected: u32,
}

fn rand_value(kind: Kind, rng: &mut Lcg) -> Option<PlcValue> {
    Some(match kind {
        Kind::Dint => PlcValue::Dint(rng.dint()),
        Kind::Int => PlcValue::Int(rng.int()),
        Kind::Real => PlcValue::Real(rng.real()),
        Kind::Bool => PlcValue::Bool(rng.boolean()),
        Kind::String => PlcValue::String(format!("FC{:08X}", rng.next_u32())),
        Kind::Udt => return None,
    })
}

fn nines(kind: Kind) -> Option<PlcValue> {
    Some(match kind {
        Kind::Dint => PlcValue::Dint(999_999),
        Kind::Int => PlcValue::Int(9_999),
        Kind::Real => PlcValue::Real(99.99),
        Kind::Bool => PlcValue::Bool(true),
        Kind::String => PlcValue::String("SETTLED".to_string()),
        Kind::Udt => return None,
    })
}

fn latency_summary(samples_ms: &[f64], failures: u64) -> serde_json::Value {
    let mut sorted = samples_ms.to_vec();
    sorted.sort_by(f64::total_cmp);
    let percentile = |fraction: f64| -> f64 {
        if sorted.is_empty() {
            return 0.0;
        }
        let index = ((sorted.len() - 1) as f64 * fraction).round() as usize;
        sorted[index]
    };
    let total_ms: f64 = sorted.iter().sum();
    let q1 = percentile(0.25);
    let q3 = percentile(0.75);
    let iqr = q3 - q1;
    let lower_fence = q1 - (1.5 * iqr);
    let upper_fence = q3 + (1.5 * iqr);
    let filtered: Vec<f64> = sorted
        .iter()
        .copied()
        .filter(|sample| *sample >= lower_fence && *sample <= upper_fence)
        .collect();
    let filtered_total: f64 = filtered.iter().sum();
    json!({
        "samples": sorted.len(),
        "failures": failures,
        "total_ms": total_ms,
        "avg_ms": if sorted.is_empty() { 0.0 } else { total_ms / sorted.len() as f64 },
        "min_ms": sorted.first().copied().unwrap_or(0.0),
        "p50_ms": percentile(0.50),
        "p95_ms": percentile(0.95),
        "p99_ms": percentile(0.99),
        "max_ms": sorted.last().copied().unwrap_or(0.0),
        "ops_per_sec": if total_ms > 0.0 { sorted.len() as f64 * 1000.0 / total_ms } else { 0.0 },
        "outlier_method": "Tukey 1.5*IQR",
        "outlier_count": sorted.len().saturating_sub(filtered.len()),
        "outlier_filtered_avg_ms": if filtered.is_empty() { 0.0 } else { filtered_total / filtered.len() as f64 }
    })
}

async fn run_batch_benchmark(
    client: &mut EipClient,
    tags: &[Tag],
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    const SIZES: &[usize] = &[1, 5, 10, 20, 50, 100];
    if !args.allow_writes {
        return Err(
            "batch benchmark writes terminal DINT values; rerun with --allow-writes".into(),
        );
    }
    let pool: Vec<&Tag> = tags
        .iter()
        .filter(|tag| {
            tag.category == "ctrl.DINT_array" && tag.kind == Kind::Dint && tag.write.is_writeable()
        })
        .take(100)
        .collect();
    if pool.len() != 100 {
        return Err(format!(
            "batch benchmark requires 100 controller DINT array tags, found {}",
            pool.len()
        )
        .into());
    }

    let mut rows = Vec::new();
    println!(
        "Batch benchmark — min {} tag operations and {:.1}s per size/direction",
        args.batch_min_tag_operations, args.batch_min_seconds
    );
    for &size in SIZES {
        let required_batches = args.batch_min_tag_operations.div_ceil(size);
        let selected = &pool[..size];
        let refs: Vec<&str> = selected.iter().map(|tag| tag.name.as_str()).collect();
        let writes: Vec<(&str, PlcValue)> = selected
            .iter()
            .map(|tag| (tag.name.as_str(), PlcValue::Dint(999_999)))
            .collect();

        for _ in 0..10 {
            let results = client.read_tags_batch(&refs).await?;
            if results.iter().any(|(_, result)| result.is_err()) {
                return Err(format!("batch read warm-up failed at size {size}").into());
            }
            let results = client.write_tags_batch(&writes).await?;
            if results.iter().any(|(_, result)| result.is_err()) {
                return Err(format!("batch write warm-up failed at size {size}").into());
            }
        }

        let mut read_samples = Vec::new();
        let mut read_failures = 0u64;
        let read_window = Instant::now();
        while read_samples.len() + (read_failures as usize) < required_batches
            || read_window.elapsed().as_secs_f64() < args.batch_min_seconds
        {
            let started = Instant::now();
            let result = client.read_tags_batch(&refs).await;
            let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
            match result {
                Ok(results)
                    if results.len() == size && results.iter().all(|(_, value)| value.is_ok()) =>
                {
                    read_samples.push(elapsed_ms)
                }
                _ => read_failures += 1,
            }
        }

        let mut write_samples = Vec::new();
        let mut write_failures = 0u64;
        let write_window = Instant::now();
        while write_samples.len() + (write_failures as usize) < required_batches
            || write_window.elapsed().as_secs_f64() < args.batch_min_seconds
        {
            let started = Instant::now();
            let result = client.write_tags_batch(&writes).await;
            let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
            match result {
                Ok(results)
                    if results.len() == size && results.iter().all(|(_, value)| value.is_ok()) =>
                {
                    write_samples.push(elapsed_ms)
                }
                _ => write_failures += 1,
            }
        }

        let mut reads = latency_summary(&read_samples, read_failures);
        let mut writes_json = latency_summary(&write_samples, write_failures);
        reads["tags_per_sec"] = json!(reads["ops_per_sec"].as_f64().unwrap_or(0.0) * size as f64);
        writes_json["tags_per_sec"] =
            json!(writes_json["ops_per_sec"].as_f64().unwrap_or(0.0) * size as f64);
        println!(
            "  size {:>3}: read avg={:>7.3}ms filtered={:>7.3}ms; write avg={:>7.3}ms filtered={:>7.3}ms",
            size,
            reads["avg_ms"].as_f64().unwrap_or(0.0),
            reads["outlier_filtered_avg_ms"].as_f64().unwrap_or(0.0),
            writes_json["avg_ms"].as_f64().unwrap_or(0.0),
            writes_json["outlier_filtered_avg_ms"]
                .as_f64()
                .unwrap_or(0.0)
        );
        rows.push(json!({"batch_size": size, "reads": reads, "writes": writes_json}));
    }

    let failures: u64 = rows
        .iter()
        .map(|row| {
            row["reads"]["failures"].as_u64().unwrap_or(0)
                + row["writes"]["failures"].as_u64().unwrap_or(0)
        })
        .sum();
    let mut terminal_verify_failures = 0u64;
    for tag in &pool {
        match read_value_for_kind(client, &tag.name, tag.kind).await {
            Ok(PlcValue::Dint(999_999)) => {}
            _ => terminal_verify_failures += 1,
        }
    }
    let result = json!({
        "schema_version": 1,
        "workload": "controller_dint_logical_batch_sizes",
        "binding": "rust",
        "binding_version": env!("CARGO_PKG_VERSION"),
        "plc_address": args.address,
        "plc_slot": args.slot,
        "batch_sizes": SIZES,
        "min_tag_operations_per_size_direction": args.batch_min_tag_operations,
        "min_seconds_per_size_direction": args.batch_min_seconds,
        "packet_policy": "default: max 20 operations and 504 bytes per CIP packet",
        "rows": rows,
        "terminal_verify": {"ok": pool.len() as u64 - terminal_verify_failures, "fail": terminal_verify_failures},
        "result": if failures == 0 && terminal_verify_failures == 0 { "PASS" } else { "FAIL" }
    });
    fs::create_dir_all(&args.out_dir)?;
    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    let path = args.out_dir.join(format!("rust_batch_benchmark_{ts}.json"));
    fs::write(&path, serde_json::to_string_pretty(&result)?)?;
    println!("wrote {}", path.display());
    if failures == 0 && terminal_verify_failures == 0 {
        Ok(())
    } else {
        Err("batch benchmark operations failed".into())
    }
}

async fn run_benchmark(
    client: &mut EipClient,
    tags: &[Tag],
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    if args.benchmark_passes == 0 {
        return Ok(());
    }
    if !args.allow_writes {
        return Err("benchmark mode writes terminal values; rerun with --allow-writes".into());
    }

    let writeable: Vec<&Tag> = tags.iter().filter(|tag| tag.write.is_writeable()).collect();
    let mut read_samples = Vec::with_capacity(tags.len() * args.benchmark_passes);
    let mut write_samples = Vec::with_capacity(writeable.len() * args.benchmark_passes);
    let mut read_failures = 0u64;
    let mut write_failures = 0u64;

    println!(
        "Benchmark — {} passes, {} reads/pass, {} writes/pass",
        args.benchmark_passes,
        tags.len(),
        writeable.len()
    );
    for pass in 0..args.benchmark_passes {
        let pass_start = Instant::now();
        for tag in tags {
            let started = Instant::now();
            let result = read_value_for_kind(client, &tag.name, tag.kind).await;
            let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
            if result.is_ok() {
                read_samples.push(elapsed_ms);
            } else {
                read_failures += 1;
            }
        }
        println!(
            "  read pass {}/{}: {:.1}s",
            pass + 1,
            args.benchmark_passes,
            pass_start.elapsed().as_secs_f64()
        );
    }
    for pass in 0..args.benchmark_passes {
        let pass_start = Instant::now();
        for tag in &writeable {
            let Some(value) = nines(tag.kind) else {
                continue;
            };
            let started = Instant::now();
            let result = client.write_tag(&tag.name, value).await;
            let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
            if result.is_ok() {
                write_samples.push(elapsed_ms);
            } else {
                write_failures += 1;
            }
        }
        println!(
            "  write pass {}/{}: {:.1}s",
            pass + 1,
            args.benchmark_passes,
            pass_start.elapsed().as_secs_f64()
        );
    }

    let reads = latency_summary(&read_samples, read_failures);
    let writes = latency_summary(&write_samples, write_failures);
    fs::create_dir_all(&args.out_dir)?;
    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    let path = args.out_dir.join(format!("rust_benchmark_{ts}.json"));
    let result = json!({
        "schema_version": 1,
        "workload": "full_coverage_manifest_sequential",
        "binding": "rust",
        "binding_version": env!("CARGO_PKG_VERSION"),
        "plc_address": args.address,
        "plc_slot": args.slot,
        "passes": args.benchmark_passes,
        "tag_count": tags.len(),
        "writeable_tag_count": writeable.len(),
        "warmup": "one full read-only preflight pass",
        "reads": reads,
        "writes": writes,
        "result": if read_failures == 0 && write_failures == 0 { "PASS" } else { "FAIL" }
    });
    fs::write(&path, serde_json::to_string_pretty(&result)?)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    println!("wrote {}", path.display());
    if read_failures == 0 && write_failures == 0 {
        Ok(())
    } else {
        Err("benchmark operations failed".into())
    }
}

fn values_match(a: &PlcValue, b: &PlcValue) -> bool {
    match (a, b) {
        (PlcValue::Dint(x), PlcValue::Dint(y)) => x == y,
        (PlcValue::Int(x), PlcValue::Int(y)) => x == y,
        (PlcValue::Bool(x), PlcValue::Bool(y)) => x == y,
        (PlcValue::Real(x), PlcValue::Real(y)) => (x - y).abs() < 0.001,
        (PlcValue::String(x), PlcValue::String(y)) => x == y,
        _ => false,
    }
}

async fn read_value_for_kind(
    client: &mut EipClient,
    tag_name: &str,
    kind: Kind,
) -> rust_ethernet_ip::Result<PlcValue> {
    if kind == Kind::String {
        client.read_string_tag(tag_name).await.map(PlcValue::String)
    } else {
        client.read_tag(tag_name).await
    }
}

fn settle_samples() -> Vec<(&'static str, &'static str, PlcValue)> {
    vec![
        (
            "ctrl.BOOL_array",
            "gTestArray_BOOL[5]",
            PlcValue::Bool(true),
        ),
        (
            "ctrl.DINT_array",
            "gTestArray_DINT[42]",
            PlcValue::Dint(999_999),
        ),
        (
            "ctrl.INT_array",
            "gTestArray_INT[100]",
            PlcValue::Int(9_999),
        ),
        (
            "ctrl.Large_DINT",
            "gTestArray_Large[500]",
            PlcValue::Dint(999_999),
        ),
        (
            "ctrl.REAL_array",
            "gTestArray_REAL[10]",
            PlcValue::Real(99.99),
        ),
        (
            "ctrl.UDT_members",
            "gTestUDT.Member1_DINT",
            PlcValue::Dint(999_999),
        ),
        (
            "ctrl.UDT_nested",
            "gTestUDT.Array_DINT[5]",
            PlcValue::Dint(999_999),
        ),
        (
            "ctrl.UDTarr_elem_nested",
            "gTestUDT_Array[2].Array_DINT[3]",
            PlcValue::Dint(999_999),
        ),
        (
            "ctrl.STRING",
            "gTest_STRING",
            PlcValue::String("SETTLED".to_string()),
        ),
        (
            "ctrl.UDT_members",
            "gTestUDT.Member5_String",
            PlcValue::String("SETTLED".to_string()),
        ),
        (
            "prog.BOOL_array",
            "Program:TestProgram.gTestArray_BOOL[5]",
            PlcValue::Bool(true),
        ),
        (
            "prog.DINT_array",
            "Program:TestProgram.gTestArray_DINT[42]",
            PlcValue::Dint(999_999),
        ),
        (
            "prog.REAL_array",
            "Program:TestProgram.gTestArray_REAL[10]",
            PlcValue::Real(99.99),
        ),
        (
            "prog.UDT_members",
            "Program:TestProgram.gTestUDT.Member1_DINT",
            PlcValue::Dint(999_999),
        ),
        (
            "prog.UDT_nested",
            "Program:TestProgram.gTestUDT.Array_DINT[5]",
            PlcValue::Dint(999_999),
        ),
        (
            "prog.UDTarr_elem_nested",
            "Program:TestProgram.gTestUDT_Array[2].Array_DINT[3]",
            PlcValue::Dint(999_999),
        ),
        (
            "prog.STRING",
            "Program:TestProgram.gTest_STRING",
            PlcValue::String("SETTLED".to_string()),
        ),
        (
            "prog.UDT_members",
            "Program:TestProgram.gTestUDT.Member5_String",
            PlcValue::String("SETTLED".to_string()),
        ),
    ]
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args();
    let tags = build_tags(&args.manifest_path)?;

    println!("Rust binding — full-coverage exerciser");
    println!(
        "PLC: {} (slot {})  total tags: {}",
        args.address,
        args.slot,
        tags.len()
    );
    let counts = tags.iter().fold((0, 0, 0), |(w, b, r), t| {
        if t.write.is_writeable() {
            (w + 1, b, r)
        } else if t.write.is_expected_blocked() {
            (w, b + 1, r)
        } else {
            (w, b, r + 1)
        }
    });
    println!(
        "  writeable: {}   expected-blocked: {}   read-only: {}",
        counts.0, counts.1, counts.2
    );
    println!();

    if args.dry_run {
        println!(
            "would-test binding=rust tags={} writeable={} blocked={} read_only={}",
            tags.len(),
            counts.0,
            counts.1,
            counts.2
        );
        return Ok(());
    }
    if args.preflight_only && args.skip_preflight {
        return Err("--preflight-only cannot be combined with --skip-preflight".into());
    }
    if args.benchmark_passes > 0 && args.skip_preflight {
        return Err("benchmark mode requires the warm-up/preflight pass".into());
    }
    if args.batch_benchmark && args.skip_preflight {
        return Err("batch benchmark requires the warm-up/preflight pass".into());
    }
    if args.batch_benchmark && args.benchmark_passes > 0 {
        return Err("choose either sequential or batch benchmark mode".into());
    }

    let mut client =
        EipClient::with_route_path(&args.address, RoutePath::new().add_slot(args.slot)).await?;
    let mut stats: BTreeMap<String, CatStats> = BTreeMap::new();
    let mut rng = Lcg::seeded();
    let mut random_values: Vec<(usize, PlcValue)> = Vec::new();
    let mut preflight_ok = 0u32;
    let mut preflight_fail = 0u32;

    if !args.skip_preflight {
        println!("Phase 0 — preflight tag inventory");
        let tp = Instant::now();
        for tag in &tags {
            match read_value_for_kind(&mut client, &tag.name, tag.kind).await {
                Ok(_) => preflight_ok += 1,
                Err(err) => {
                    preflight_fail += 1;
                    eprintln!(
                        "setup-error: tag {} failed preflight ({}) — verify the PLC project against docs/PLC_TEST_TAG_DEFINITIONS.md",
                        tag.name, err
                    );
                }
            }
        }
        println!(
            "  done in {:.1}s  preflight={}/{}",
            tp.elapsed().as_secs_f64(),
            preflight_ok,
            preflight_ok + preflight_fail
        );
        if preflight_fail > 0 {
            std::process::exit(2);
        }
        if args.preflight_only {
            println!("PASS: read-only preflight completed; no tags were written");
            return Ok(());
        }
    }

    if args.benchmark_passes > 0 {
        return run_benchmark(&mut client, &tags, &args).await;
    }
    if args.batch_benchmark {
        return run_batch_benchmark(&mut client, &tags, &args).await;
    }

    println!("Phase 1 — read every tag");
    let t0 = Instant::now();
    for tag in &tags {
        let entry = stats.entry(tag.category.clone()).or_default();
        match read_value_for_kind(&mut client, &tag.name, tag.kind).await {
            Ok(_) => entry.read_ok += 1,
            Err(_) => entry.read_fail += 1,
        }
    }
    println!("  done in {:.1}s", t0.elapsed().as_secs_f64());

    println!("Phase 2 — write random values to all writeable tags");
    let t1 = Instant::now();
    for (idx, tag) in tags.iter().enumerate() {
        if !tag.write.is_writeable() {
            continue;
        }
        let Some(v) = rand_value(tag.kind, &mut rng) else {
            continue;
        };
        let entry = stats.entry(tag.category.clone()).or_default();
        match client.write_tag(&tag.name, v.clone()).await {
            Ok(()) => {
                entry.write_ok += 1;
                random_values.push((idx, v));
            }
            Err(_) => {
                entry.write_fail += 1;
            }
        }
    }
    println!("  done in {:.1}s", t1.elapsed().as_secs_f64());

    println!("Phase 3 — verify writes via read-back");
    let t2 = Instant::now();
    for (idx, expected) in &random_values {
        let tag = &tags[*idx];
        let entry = stats.entry(tag.category.clone()).or_default();
        let read_result = read_value_for_kind(&mut client, &tag.name, tag.kind).await;
        match read_result {
            Ok(actual) if values_match(&actual, expected) => entry.verify_ok += 1,
            _ => entry.verify_fail += 1,
        }
    }
    println!("  done in {:.1}s", t2.elapsed().as_secs_f64());

    println!("Phase 4 — confirm expected-blocked writes are still rejected");
    let t3 = Instant::now();
    for tag in &tags {
        if !tag.write.is_expected_blocked() {
            continue;
        }
        let Some(v) = rand_value(tag.kind, &mut rng) else {
            continue;
        };
        let entry = stats.entry(tag.category.clone()).or_default();
        match client.write_tag(&tag.name, v).await {
            Err(_) => entry.blocked_as_expected += 1,
            Ok(()) => entry.blocked_unexpected_pass += 1,
        }
    }
    println!("  done in {:.1}s", t3.elapsed().as_secs_f64());

    println!("Phase 5 — settle writeable tags to terminal state (999999 / 9999 / 99.99 / true)");
    let t4 = Instant::now();
    let mut settle_ok = 0u32;
    let mut settle_fail = 0u32;
    for tag in &tags {
        if !tag.write.is_writeable() {
            continue;
        }
        let Some(v) = nines(tag.kind) else {
            continue;
        };
        match client.write_tag(&tag.name, v).await {
            Ok(()) => settle_ok += 1,
            Err(_) => settle_fail += 1,
        }
    }
    println!(
        "  done in {:.1}s  settle_ok={} settle_fail={}",
        t4.elapsed().as_secs_f64(),
        settle_ok,
        settle_fail
    );
    println!();

    println!("Phase 6 — verify settle (sample read-back)");
    let t5 = Instant::now();
    let mut settle_verify_ok = 0u32;
    let mut settle_verify_fail = 0u32;
    for (category, tag_name, expected) in settle_samples() {
        let read_result = if matches!(expected, PlcValue::String(_)) {
            client.read_string_tag(tag_name).await.map(PlcValue::String)
        } else {
            client.read_tag(tag_name).await
        };
        match read_result {
            Ok(actual) if values_match(&actual, &expected) => {
                settle_verify_ok += 1;
                println!("  verify-settle  {:<28} {:<48} OK", category, tag_name);
            }
            Ok(actual) => {
                settle_verify_fail += 1;
                println!(
                    "  verify-settle  {:<28} {:<48} FAIL MISMATCH: expected {:?}, got {:?}",
                    category, tag_name, expected, actual
                );
            }
            Err(err) => {
                settle_verify_fail += 1;
                println!(
                    "  verify-settle  {:<28} {:<48} FAIL READ ERROR: {}",
                    category, tag_name, err
                );
            }
        }
    }
    println!(
        "  done in {:.1}s  settle_verify={}/{}",
        t5.elapsed().as_secs_f64(),
        settle_verify_ok,
        settle_verify_ok + settle_verify_fail
    );
    println!();

    println!("Per-category results:");
    println!(
        "  {:<32} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9}",
        "category", "read+", "read-", "write+", "write-", "verify+", "blocked+"
    );
    let mut totals = CatStats::default();
    for (cat, s) in &stats {
        println!(
            "  {:<32} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9}",
            cat,
            s.read_ok,
            s.read_fail,
            s.write_ok,
            s.write_fail,
            s.verify_ok,
            s.blocked_as_expected
        );
        totals.read_ok += s.read_ok;
        totals.read_fail += s.read_fail;
        totals.write_ok += s.write_ok;
        totals.write_fail += s.write_fail;
        totals.verify_ok += s.verify_ok;
        totals.verify_fail += s.verify_fail;
        totals.blocked_as_expected += s.blocked_as_expected;
        totals.blocked_unexpected_pass += s.blocked_unexpected_pass;
    }
    println!(
        "  {:<32} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9}",
        "TOTAL",
        totals.read_ok,
        totals.read_fail,
        totals.write_ok,
        totals.write_fail,
        totals.verify_ok,
        totals.blocked_as_expected
    );
    println!();

    let unexpected = totals.read_fail
        + totals.write_fail
        + totals.verify_fail
        + totals.blocked_unexpected_pass
        + settle_fail
        + settle_verify_fail;
    println!(
        "Summary: reads={}/{}  writes={}/{}  verify={}/{}  blocked_as_expected={}  unexpected_anomalies={}",
        totals.read_ok,
        totals.read_ok + totals.read_fail,
        totals.write_ok,
        totals.write_ok + totals.write_fail,
        totals.verify_ok,
        totals.verify_ok + totals.verify_fail,
        totals.blocked_as_expected,
        unexpected
    );
    println!(
        "binding=rust tags={} reads={}/{} writes={}/{} verify={}/{} blocked={} anomalies={} RESULT={}",
        tags.len(),
        totals.read_ok,
        totals.read_ok + totals.read_fail,
        totals.write_ok,
        totals.write_ok + totals.write_fail,
        totals.verify_ok,
        totals.verify_ok + totals.verify_fail,
        totals.blocked_as_expected,
        unexpected,
        if unexpected == 0 { "PASS" } else { "FAIL" }
    );
    write_json_result(&RunSummary {
        args: &args,
        tag_count: tags.len(),
        stats: &stats,
        preflight_ok,
        preflight_fail,
        totals: &totals,
        settle_ok,
        settle_fail,
        settle_verify_ok,
        settle_verify_fail,
        unexpected,
    })?;

    if unexpected == 0 {
        println!("RESULT: PASS");
        Ok(())
    } else {
        println!("RESULT: FAIL ({} anomalies)", unexpected);
        std::process::exit(1);
    }
}

fn write_json_result(summary: &RunSummary<'_>) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(&summary.args.out_dir)?;
    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    let path = summary.args.out_dir.join(format!("rust_{ts}.json"));
    let mut categories = serde_json::Map::new();
    for (category, stats) in summary.stats {
        categories.insert(
            category.clone(),
            json!({
                "read_ok": stats.read_ok,
                "read_fail": stats.read_fail,
                "write_ok": stats.write_ok,
                "write_fail": stats.write_fail,
                "verify_ok": stats.verify_ok,
                "verify_fail": stats.verify_fail,
                "blocked_as_expected": stats.blocked_as_expected,
                "blocked_unexpected_pass": stats.blocked_unexpected_pass
            }),
        );
    }
    let result = json!({
        "schema_version": 1,
        "binding": "rust",
        "binding_version": env!("CARGO_PKG_VERSION"),
        "plc_address": summary.args.address,
        "plc_slot": summary.args.slot,
        "manifest_version": 1,
        "tag_count": summary.tag_count,
        "result": if summary.unexpected == 0 { "PASS" } else { "FAIL" },
        "anomalies": summary.unexpected,
        "phases": {
            "preflight": { "ok": summary.preflight_ok, "fail": summary.preflight_fail },
            "phase1_read": { "ok": summary.totals.read_ok, "fail": summary.totals.read_fail },
            "phase2_write": { "ok": summary.totals.write_ok, "fail": summary.totals.write_fail },
            "phase3_verify": { "ok": summary.totals.verify_ok, "fail": summary.totals.verify_fail },
            "phase4_blocked": { "ok": summary.totals.blocked_as_expected, "fail": summary.totals.blocked_unexpected_pass, "note": "expected current-encoding rejections" },
            "phase5_settle": { "ok": summary.settle_ok, "fail": summary.settle_fail },
            "phase6_verify_settle": { "ok": summary.settle_verify_ok, "fail": summary.settle_verify_fail }
        },
        "categories": categories
    });
    fs::write(path, serde_json::to_string_pretty(&result)?)?;
    Ok(())
}
