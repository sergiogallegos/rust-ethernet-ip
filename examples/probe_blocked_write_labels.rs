use rust_ethernet_ip::{EipClient, PlcValue, RoutePath};
use serde::Deserialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

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
    all_blocked: bool,
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
        all_blocked: false,
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
                    .expect("--plc-slot must be an integer");
            }
            "--manifest" => {
                args.manifest_path =
                    PathBuf::from(iter.next().expect("--manifest requires a value"));
            }
            "--out-dir" => {
                args.out_dir = PathBuf::from(iter.next().expect("--out-dir requires a value"));
            }
            "--dry-run" => args.dry_run = true,
            "--all-blocked" => args.all_blocked = true,
            other => eprintln!("warning: ignoring unknown argument {other}"),
        }
    }

    args
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash)]
enum Kind {
    Dint,
    Int,
    Real,
    Bool,
    String,
    Udt,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
enum WriteMode {
    Writeable,
    ReadOnly,
    FirmwareBlockedString,
    EncodingBlockedUdtStringMember,
    ServiceLayerWriteable,
}

impl WriteMode {
    fn is_expected_blocked(self) -> bool {
        matches!(
            self,
            Self::FirmwareBlockedString | Self::EncodingBlockedUdtStringMember
        )
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Writeable => "writeable",
            Self::ReadOnly => "read_only",
            Self::FirmwareBlockedString => "firmware_blocked_string",
            Self::EncodingBlockedUdtStringMember => "encoding_blocked_udt_string_member",
            Self::ServiceLayerWriteable => "service_layer_writeable",
        }
    }
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
    scope: String,
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

#[derive(Clone)]
struct ProbeTarget {
    tag: String,
    category: String,
    scope: String,
    kind: Kind,
    mode: WriteMode,
    member_or_field: Option<String>,
}

impl ProbeTarget {
    fn class_key(&self) -> String {
        format!(
            "{}|{}|{}|{}|{:?}",
            self.scope,
            self.mode.as_str(),
            self.category,
            self.member_or_field.as_deref().unwrap_or("<tag>"),
            self.kind
        )
    }
}

fn build_probe_targets(
    manifest_path: &Path,
    all_blocked: bool,
) -> Result<Vec<ProbeTarget>, Box<dyn std::error::Error>> {
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

    let mut blocked = Vec::new();
    for category in manifest.categories {
        blocked.extend(expand_blocked_category(&category)?);
    }

    if all_blocked {
        return Ok(blocked);
    }

    let mut selected = Vec::new();
    let mut seen = BTreeSet::new();
    for target in blocked {
        if seen.insert(target.class_key()) {
            selected.push(target);
        }
    }
    Ok(selected)
}

fn expand_blocked_category(
    category: &ManifestCategory,
) -> Result<Vec<ProbeTarget>, Box<dyn std::error::Error>> {
    let mut targets = Vec::new();

    if let Some(members) = &category.members {
        for i in range_or_once(category.indices.as_ref()) {
            for (member, spec) in members {
                if spec.writeability.is_expected_blocked() {
                    targets.push(ProbeTarget {
                        tag: render_pattern(&category.pattern, Some(i), Some(member), None, None),
                        category: category.name.clone(),
                        scope: category.scope.clone(),
                        kind: spec.kind,
                        mode: spec.writeability,
                        member_or_field: Some(member.clone()),
                    });
                }
            }
        }
        return Ok(targets);
    }

    if let Some(inner) = &category.inner {
        for i in range_or_once(category.outer_indices.as_ref()) {
            for (field, spec) in inner {
                let range = spec
                    .range
                    .ok_or_else(|| format!("{}.{field} missing range", category.name))?;
                for j in range[0]..range[1] {
                    if spec.writeability.is_expected_blocked() {
                        targets.push(ProbeTarget {
                            tag: render_pattern(
                                &category.pattern,
                                Some(i),
                                None,
                                Some(field),
                                Some(j),
                            ),
                            category: category.name.clone(),
                            scope: category.scope.clone(),
                            kind: spec.kind,
                            mode: spec.writeability,
                            member_or_field: Some(field.clone()),
                        });
                    }
                }
            }
        }
        return Ok(targets);
    }

    let Some(mode) = category.writeability else {
        return Ok(targets);
    };
    if !mode.is_expected_blocked() {
        return Ok(targets);
    }
    let kind = category
        .kind
        .ok_or_else(|| format!("{} missing kind", category.name))?;
    for i in range_or_once(category.indices.as_ref()) {
        targets.push(ProbeTarget {
            tag: render_pattern(&category.pattern, Some(i), None, None, None),
            category: category.name.clone(),
            scope: category.scope.clone(),
            kind,
            mode,
            member_or_field: None,
        });
    }
    Ok(targets)
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

fn candidate_value(kind: Kind, original: &PlcValue, seq: usize) -> Option<PlcValue> {
    Some(match kind {
        Kind::Dint => match original {
            PlcValue::Dint(value) => PlcValue::Dint(value.saturating_add(777 + seq as i32)),
            _ => PlcValue::Dint(777 + seq as i32),
        },
        Kind::Int => match original {
            PlcValue::Int(value) => PlcValue::Int(value.saturating_add(37)),
            _ => PlcValue::Int(1234),
        },
        Kind::Real => match original {
            PlcValue::Real(value) => PlcValue::Real(value + 1.25),
            _ => PlcValue::Real(12.5),
        },
        Kind::Bool => match original {
            PlcValue::Bool(value) => PlcValue::Bool(!value),
            _ => PlcValue::Bool(true),
        },
        Kind::String => PlcValue::String(format!("AV_PROBE_{seq:02}")),
        Kind::Udt => return None,
    })
}

fn values_match(actual: &PlcValue, expected: &PlcValue) -> bool {
    match (actual, expected) {
        (PlcValue::Dint(a), PlcValue::Dint(b)) => a == b,
        (PlcValue::Int(a), PlcValue::Int(b)) => a == b,
        (PlcValue::Bool(a), PlcValue::Bool(b)) => a == b,
        (PlcValue::Real(a), PlcValue::Real(b)) => (a - b).abs() < 0.001,
        (PlcValue::String(a), PlcValue::String(b)) => a == b,
        _ => false,
    }
}

fn sibling_tag(target: &ProbeTarget) -> Option<String> {
    let (base, member) = target.tag.rsplit_once('.')?;
    let sibling_member = match member {
        "Member1_DINT" => "Member2_REAL",
        "Member2_REAL" | "Member3_BOOL" | "Member4_INT" | "Member5_String" => "Member1_DINT",
        _ => return None,
    };
    Some(format!("{base}.{sibling_member}"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args();
    let targets = build_probe_targets(&args.manifest_path, args.all_blocked)?;

    println!("Rust binding — blocked write-label probe");
    println!(
        "PLC: {} (slot {})  blocked probe targets: {}  mode: {}",
        args.address,
        args.slot,
        targets.len(),
        if args.all_blocked {
            "all blocked tags"
        } else {
            "one representative per blocked class"
        }
    );

    if args.dry_run {
        for target in &targets {
            println!(
                "  would-probe {:<32} {:<13} {:<44} {}",
                target.category,
                target.mode.as_str(),
                target.tag,
                target.member_or_field.as_deref().unwrap_or("<whole-tag>")
            );
        }
        println!(
            "would-probe binding=rust blocked_targets={} all_blocked={}",
            targets.len(),
            args.all_blocked
        );
        return Ok(());
    }

    let mut client =
        EipClient::with_route_path(&args.address, RoutePath::new().add_slot(args.slot)).await?;

    let mut results = Vec::new();
    let mut write_succeeded = 0u32;
    let mut write_failed = 0u32;
    let mut setup_failed = 0u32;
    let mut verify_failed = 0u32;
    let mut sibling_changed = 0u32;
    let mut restore_failed = 0u32;

    for (idx, target) in targets.iter().enumerate() {
        println!("probe {:02}/{} {}", idx + 1, targets.len(), target.tag);

        let sibling = sibling_tag(target);
        let sibling_before = match sibling.as_deref() {
            Some(tag) => match client.read_tag(tag).await {
                Ok(value) => Some((tag.to_string(), value)),
                Err(err) => {
                    setup_failed += 1;
                    results.push(json!({
                        "tag": target.tag,
                        "category": target.category,
                        "scope": target.scope,
                        "writeability": target.mode.as_str(),
                        "kind": format!("{:?}", target.kind),
                        "status": "setup_failed",
                        "error": format!("failed to read sibling {tag}: {err}")
                    }));
                    continue;
                }
            },
            None => None,
        };

        let original = match client.read_tag(&target.tag).await {
            Ok(value) => value,
            Err(err) => {
                setup_failed += 1;
                results.push(json!({
                    "tag": target.tag,
                    "category": target.category,
                    "scope": target.scope,
                    "writeability": target.mode.as_str(),
                    "kind": format!("{:?}", target.kind),
                    "status": "setup_failed",
                    "error": format!("failed to read original value: {err}")
                }));
                continue;
            }
        };

        let Some(candidate) = candidate_value(target.kind, &original, idx) else {
            setup_failed += 1;
            results.push(json!({
                "tag": target.tag,
                "category": target.category,
                "scope": target.scope,
                "writeability": target.mode.as_str(),
                "kind": format!("{:?}", target.kind),
                "status": "setup_failed",
                "original": format!("{:?}", original),
                "error": "no safe candidate value for this kind"
            }));
            continue;
        };

        match client.write_tag(&target.tag, candidate.clone()).await {
            Err(err) => {
                write_failed += 1;
                results.push(json!({
                    "tag": target.tag,
                    "category": target.category,
                    "scope": target.scope,
                    "writeability": target.mode.as_str(),
                    "kind": format!("{:?}", target.kind),
                    "status": "write_failed",
                    "original": format!("{:?}", original),
                    "candidate": format!("{:?}", candidate),
                    "error": err.to_string()
                }));
            }
            Ok(()) => {
                write_succeeded += 1;
                let readback = client.read_tag(&target.tag).await;
                let verified = matches!(&readback, Ok(value) if values_match(value, &candidate));
                if !verified {
                    verify_failed += 1;
                }

                let sibling_status = if let Some((sibling_tag, before)) = &sibling_before {
                    match client.read_tag(sibling_tag).await {
                        Ok(after) if values_match(&after, before) => {
                            json!({"tag": sibling_tag, "status": "unchanged"})
                        }
                        Ok(after) => {
                            sibling_changed += 1;
                            json!({
                                "tag": sibling_tag,
                                "status": "changed",
                                "before": format!("{:?}", before),
                                "after": format!("{:?}", after)
                            })
                        }
                        Err(err) => {
                            sibling_changed += 1;
                            json!({
                                "tag": sibling_tag,
                                "status": "read_failed",
                                "error": err.to_string()
                            })
                        }
                    }
                } else {
                    json!({"status": "not_applicable"})
                };

                let restore = match client.write_tag(&target.tag, original.clone()).await {
                    Ok(()) => match client.read_tag(&target.tag).await {
                        Ok(value) if values_match(&value, &original) => json!({"status": "ok"}),
                        Ok(value) => {
                            restore_failed += 1;
                            json!({
                                "status": "verify_failed",
                                "actual": format!("{:?}", value),
                                "expected": format!("{:?}", original)
                            })
                        }
                        Err(err) => {
                            restore_failed += 1;
                            json!({"status": "read_failed", "error": err.to_string()})
                        }
                    },
                    Err(err) => {
                        restore_failed += 1;
                        json!({"status": "write_failed", "error": err.to_string()})
                    }
                };

                results.push(json!({
                    "tag": target.tag,
                    "category": target.category,
                    "scope": target.scope,
                    "writeability": target.mode.as_str(),
                    "kind": format!("{:?}", target.kind),
                    "status": "write_succeeded",
                    "original": format!("{:?}", original),
                    "candidate": format!("{:?}", candidate),
                    "readback": readback.as_ref().map(|value| format!("{:?}", value)).unwrap_or_else(|err| format!("ERROR: {err}")),
                    "verified": verified,
                    "sibling_integrity": sibling_status,
                    "restore": restore
                }));
            }
        }
    }

    fs::create_dir_all(&args.out_dir)?;
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let out_path = args
        .out_dir
        .join(format!("blocked_write_label_probe_rust_{timestamp}.json"));

    let unexpected = setup_failed + verify_failed + sibling_changed + restore_failed;
    let report = json!({
        "schema_version": 1,
        "binding": "rust",
        "plc_address": args.address,
        "plc_slot": args.slot,
        "manifest": args.manifest_path,
        "all_blocked": args.all_blocked,
        "summary": {
            "targets": targets.len(),
            "write_succeeded": write_succeeded,
            "write_failed": write_failed,
            "setup_failed": setup_failed,
            "verify_failed": verify_failed,
            "sibling_changed": sibling_changed,
            "restore_failed": restore_failed,
            "unexpected": unexpected
        },
        "results": results
    });
    fs::write(&out_path, serde_json::to_string_pretty(&report)?)?;

    println!(
        "probe-result binding=rust targets={} write_succeeded={} write_failed={} setup_failed={} verify_failed={} sibling_changed={} restore_failed={} unexpected={} RESULT={}",
        targets.len(),
        write_succeeded,
        write_failed,
        setup_failed,
        verify_failed,
        sibling_changed,
        restore_failed,
        unexpected,
        if unexpected == 0 { "PASS" } else { "FAIL" }
    );
    println!("wrote {}", out_path.display());

    if unexpected == 0 {
        Ok(())
    } else {
        Err("blocked write-label probe had setup/verify/restore failures".into())
    }
}
