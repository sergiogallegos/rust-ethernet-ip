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
    FirmwareBlockedString,
    FirmwareBlockedUdtStringMember,
    FirmwareBlockedUdtArrayElementMember,
    ServiceLayerWriteable,
}

impl WriteMode {
    fn is_writeable(self) -> bool {
        matches!(self, Self::Writeable | Self::ServiceLayerWriteable)
    }

    fn is_firmware_blocked(self) -> bool {
        matches!(
            self,
            Self::FirmwareBlockedString
                | Self::FirmwareBlockedUdtStringMember
                | Self::FirmwareBlockedUdtArrayElementMember
        )
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
        Kind::String | Kind::Udt => return None,
    })
}

fn nines(kind: Kind) -> Option<PlcValue> {
    Some(match kind {
        Kind::Dint => PlcValue::Dint(999_999),
        Kind::Int => PlcValue::Int(9_999),
        Kind::Real => PlcValue::Real(99.99),
        Kind::Bool => PlcValue::Bool(true),
        Kind::String | Kind::Udt => return None,
    })
}

fn values_match(a: &PlcValue, b: &PlcValue) -> bool {
    match (a, b) {
        (PlcValue::Dint(x), PlcValue::Dint(y)) => x == y,
        (PlcValue::Int(x), PlcValue::Int(y)) => x == y,
        (PlcValue::Bool(x), PlcValue::Bool(y)) => x == y,
        (PlcValue::Real(x), PlcValue::Real(y)) => (x - y).abs() < 0.001,
        _ => false,
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
        } else if t.write.is_firmware_blocked() {
            (w, b + 1, r)
        } else {
            (w, b, r + 1)
        }
    });
    println!(
        "  writeable: {}   firmware-blocked: {}   read-only: {}",
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
            match client.read_tag(&tag.name).await {
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
    }

    println!("Phase 1 — read every tag");
    let t0 = Instant::now();
    for tag in &tags {
        let entry = stats.entry(tag.category.clone()).or_default();
        match client.read_tag(&tag.name).await {
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
        match client.read_tag(&tag.name).await {
            Ok(actual) if values_match(&actual, expected) => entry.verify_ok += 1,
            _ => entry.verify_fail += 1,
        }
    }
    println!("  done in {:.1}s", t2.elapsed().as_secs_f64());

    println!("Phase 4 — confirm firmware-blocked writes are still blocked");
    let t3 = Instant::now();
    for tag in &tags {
        if !tag.write.is_firmware_blocked() {
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
        match client.read_tag(tag_name).await {
            Ok(actual) if values_match(&actual, &expected) => {
                settle_verify_ok += 1;
                println!("  verify-settle  {:<28} {:<48} ✓", category, tag_name);
            }
            Ok(actual) => {
                settle_verify_fail += 1;
                println!(
                    "  verify-settle  {:<28} {:<48} ✗ MISMATCH: expected {:?}, got {:?}",
                    category, tag_name, expected, actual
                );
            }
            Err(err) => {
                settle_verify_fail += 1;
                println!(
                    "  verify-settle  {:<28} {:<48} ✗ READ ERROR: {}",
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
            "phase4_blocked": { "ok": summary.totals.blocked_as_expected, "fail": summary.totals.blocked_unexpected_pass, "note": "expected firmware rejections" },
            "phase5_settle": { "ok": summary.settle_ok, "fail": summary.settle_fail },
            "phase6_verify_settle": { "ok": summary.settle_verify_ok, "fail": summary.settle_verify_fail }
        },
        "categories": categories
    });
    fs::write(path, serde_json::to_string_pretty(&result)?)?;
    Ok(())
}
