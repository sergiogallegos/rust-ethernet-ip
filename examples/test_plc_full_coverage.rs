use rust_ethernet_ip::{EipClient, EtherNetIpError, PlcValue, RoutePath};
use std::collections::BTreeMap;
use std::env;
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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Kind {
    Dint,
    Int,
    Real,
    Bool,
    String,
    Udt,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WriteMode {
    Writeable,
    FirmwareBlocked,
    ReadOnly,
}

struct Tag {
    name: String,
    category: &'static str,
    kind: Kind,
    write: WriteMode,
}

fn build_tags() -> Vec<Tag> {
    let mut t = Vec::new();

    // Controller-scope arrays — full coverage
    for i in 0..100 {
        t.push(mk(
            format!("gTestArray_DINT[{}]", i),
            "ctrl.DINT_array",
            Kind::Dint,
            WriteMode::Writeable,
        ));
    }
    for i in 0..50 {
        t.push(mk(
            format!("gTestArray_REAL[{}]", i),
            "ctrl.REAL_array",
            Kind::Real,
            WriteMode::Writeable,
        ));
    }
    for i in 0..128 {
        t.push(mk(
            format!("gTestArray_BOOL[{}]", i),
            "ctrl.BOOL_array",
            Kind::Bool,
            WriteMode::Writeable,
        ));
    }
    for i in 0..200 {
        t.push(mk(
            format!("gTestArray_INT[{}]", i),
            "ctrl.INT_array",
            Kind::Int,
            WriteMode::Writeable,
        ));
    }
    for i in 0..1000 {
        t.push(mk(
            format!("gTestArray_Large[{}]", i),
            "ctrl.Large_DINT",
            Kind::Dint,
            WriteMode::Writeable,
        ));
    }

    // Controller STRING — read works, direct write blocked by firmware 0x2107
    t.push(mk(
        "gTest_STRING".into(),
        "ctrl.STRING",
        Kind::String,
        WriteMode::FirmwareBlocked,
    ));

    // Controller UDT
    t.push(mk(
        "gTestUDT".into(),
        "ctrl.UDT_whole",
        Kind::Udt,
        WriteMode::ReadOnly,
    ));
    t.push(mk(
        "gTestUDT.Member1_DINT".into(),
        "ctrl.UDT_members",
        Kind::Dint,
        WriteMode::Writeable,
    ));
    t.push(mk(
        "gTestUDT.Member2_REAL".into(),
        "ctrl.UDT_members",
        Kind::Real,
        WriteMode::Writeable,
    ));
    t.push(mk(
        "gTestUDT.Member3_BOOL".into(),
        "ctrl.UDT_members",
        Kind::Bool,
        WriteMode::Writeable,
    ));
    t.push(mk(
        "gTestUDT.Member4_INT".into(),
        "ctrl.UDT_members",
        Kind::Int,
        WriteMode::Writeable,
    ));
    t.push(mk(
        "gTestUDT.Member5_String".into(),
        "ctrl.UDT_members",
        Kind::String,
        WriteMode::FirmwareBlocked,
    ));
    for i in 0..10 {
        t.push(mk(
            format!("gTestUDT.Array_DINT[{}]", i),
            "ctrl.UDT_nested",
            Kind::Dint,
            WriteMode::Writeable,
        ));
    }
    for i in 0..5 {
        t.push(mk(
            format!("gTestUDT.Array_REAL[{}]", i),
            "ctrl.UDT_nested",
            Kind::Real,
            WriteMode::Writeable,
        ));
    }
    for i in 0..20 {
        t.push(mk(
            format!("gTestUDT.Array_BOOL[{}]", i),
            "ctrl.UDT_nested",
            Kind::Bool,
            WriteMode::Writeable,
        ));
    }

    // Controller UDT array — all element-member writes firmware-blocked
    t.push(mk(
        "gTestUDT_Array".into(),
        "ctrl.UDTarr_whole",
        Kind::Udt,
        WriteMode::ReadOnly,
    ));
    for i in 0..10 {
        t.push(mk(
            format!("gTestUDT_Array[{}]", i),
            "ctrl.UDTarr_element",
            Kind::Udt,
            WriteMode::ReadOnly,
        ));
        t.push(mk(
            format!("gTestUDT_Array[{}].Member1_DINT", i),
            "ctrl.UDTarr_elem_members",
            Kind::Dint,
            WriteMode::FirmwareBlocked,
        ));
        t.push(mk(
            format!("gTestUDT_Array[{}].Member2_REAL", i),
            "ctrl.UDTarr_elem_members",
            Kind::Real,
            WriteMode::FirmwareBlocked,
        ));
        t.push(mk(
            format!("gTestUDT_Array[{}].Member3_BOOL", i),
            "ctrl.UDTarr_elem_members",
            Kind::Bool,
            WriteMode::FirmwareBlocked,
        ));
        t.push(mk(
            format!("gTestUDT_Array[{}].Member4_INT", i),
            "ctrl.UDTarr_elem_members",
            Kind::Int,
            WriteMode::FirmwareBlocked,
        ));
        t.push(mk(
            format!("gTestUDT_Array[{}].Member5_String", i),
            "ctrl.UDTarr_elem_members",
            Kind::String,
            WriteMode::FirmwareBlocked,
        ));
        for j in 0..10 {
            t.push(mk(
                format!("gTestUDT_Array[{}].Array_DINT[{}]", i, j),
                "ctrl.UDTarr_elem_nested",
                Kind::Dint,
                WriteMode::FirmwareBlocked,
            ));
        }
        for j in 0..5 {
            t.push(mk(
                format!("gTestUDT_Array[{}].Array_REAL[{}]", i, j),
                "ctrl.UDTarr_elem_nested",
                Kind::Real,
                WriteMode::FirmwareBlocked,
            ));
        }
        for j in 0..20 {
            t.push(mk(
                format!("gTestUDT_Array[{}].Array_BOOL[{}]", i, j),
                "ctrl.UDTarr_elem_nested",
                Kind::Bool,
                WriteMode::FirmwareBlocked,
            ));
        }
    }

    // Program-scope arrays
    for i in 0..100 {
        t.push(mk(
            format!("Program:TestProgram.gTestArray_DINT[{}]", i),
            "prog.DINT_array",
            Kind::Dint,
            WriteMode::Writeable,
        ));
    }
    for i in 0..50 {
        t.push(mk(
            format!("Program:TestProgram.gTestArray_REAL[{}]", i),
            "prog.REAL_array",
            Kind::Real,
            WriteMode::Writeable,
        ));
    }
    for i in 0..100 {
        t.push(mk(
            format!("Program:TestProgram.gTestArray_BOOL[{}]", i),
            "prog.BOOL_array",
            Kind::Bool,
            WriteMode::Writeable,
        ));
    }
    t.push(mk(
        "Program:TestProgram.gTest_STRING".into(),
        "prog.STRING",
        Kind::String,
        WriteMode::FirmwareBlocked,
    ));

    // Program UDT
    t.push(mk(
        "Program:TestProgram.gTestUDT".into(),
        "prog.UDT_whole",
        Kind::Udt,
        WriteMode::ReadOnly,
    ));
    t.push(mk(
        "Program:TestProgram.gTestUDT.Member1_DINT".into(),
        "prog.UDT_members",
        Kind::Dint,
        WriteMode::Writeable,
    ));
    t.push(mk(
        "Program:TestProgram.gTestUDT.Member2_REAL".into(),
        "prog.UDT_members",
        Kind::Real,
        WriteMode::Writeable,
    ));
    t.push(mk(
        "Program:TestProgram.gTestUDT.Member3_BOOL".into(),
        "prog.UDT_members",
        Kind::Bool,
        WriteMode::Writeable,
    ));
    t.push(mk(
        "Program:TestProgram.gTestUDT.Member4_INT".into(),
        "prog.UDT_members",
        Kind::Int,
        WriteMode::Writeable,
    ));
    t.push(mk(
        "Program:TestProgram.gTestUDT.Member5_String".into(),
        "prog.UDT_members",
        Kind::String,
        WriteMode::FirmwareBlocked,
    ));
    for i in 0..10 {
        t.push(mk(
            format!("Program:TestProgram.gTestUDT.Array_DINT[{}]", i),
            "prog.UDT_nested",
            Kind::Dint,
            WriteMode::Writeable,
        ));
    }
    for i in 0..5 {
        t.push(mk(
            format!("Program:TestProgram.gTestUDT.Array_REAL[{}]", i),
            "prog.UDT_nested",
            Kind::Real,
            WriteMode::Writeable,
        ));
    }
    for i in 0..20 {
        t.push(mk(
            format!("Program:TestProgram.gTestUDT.Array_BOOL[{}]", i),
            "prog.UDT_nested",
            Kind::Bool,
            WriteMode::Writeable,
        ));
    }

    // Program UDT array — element members firmware-blocked too
    t.push(mk(
        "Program:TestProgram.gTestUDT_Array".into(),
        "prog.UDTarr_whole",
        Kind::Udt,
        WriteMode::ReadOnly,
    ));
    for i in 0..5 {
        t.push(mk(
            format!("Program:TestProgram.gTestUDT_Array[{}]", i),
            "prog.UDTarr_element",
            Kind::Udt,
            WriteMode::ReadOnly,
        ));
        t.push(mk(
            format!("Program:TestProgram.gTestUDT_Array[{}].Member1_DINT", i),
            "prog.UDTarr_elem_members",
            Kind::Dint,
            WriteMode::FirmwareBlocked,
        ));
        t.push(mk(
            format!("Program:TestProgram.gTestUDT_Array[{}].Member2_REAL", i),
            "prog.UDTarr_elem_members",
            Kind::Real,
            WriteMode::FirmwareBlocked,
        ));
        t.push(mk(
            format!("Program:TestProgram.gTestUDT_Array[{}].Member3_BOOL", i),
            "prog.UDTarr_elem_members",
            Kind::Bool,
            WriteMode::FirmwareBlocked,
        ));
        t.push(mk(
            format!("Program:TestProgram.gTestUDT_Array[{}].Member4_INT", i),
            "prog.UDTarr_elem_members",
            Kind::Int,
            WriteMode::FirmwareBlocked,
        ));
        for j in 0..10 {
            t.push(mk(
                format!(
                    "Program:TestProgram.gTestUDT_Array[{}].Array_DINT[{}]",
                    i, j
                ),
                "prog.UDTarr_elem_nested",
                Kind::Dint,
                WriteMode::FirmwareBlocked,
            ));
        }
    }

    t
}

fn mk(name: String, category: &'static str, kind: Kind, write: WriteMode) -> Tag {
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

#[tokio::main]
async fn main() -> Result<(), EtherNetIpError> {
    let address = get_plc_address();
    let slot = get_cpu_slot();
    let tags = build_tags();

    println!("Rust binding — full-coverage exerciser");
    println!(
        "PLC: {} (slot {})  total tags: {}",
        address,
        slot,
        tags.len()
    );
    let counts = tags.iter().fold((0, 0, 0), |(w, b, r), t| match t.write {
        WriteMode::Writeable => (w + 1, b, r),
        WriteMode::FirmwareBlocked => (w, b + 1, r),
        WriteMode::ReadOnly => (w, b, r + 1),
    });
    println!(
        "  writeable: {}   firmware-blocked: {}   read-only: {}",
        counts.0, counts.1, counts.2
    );
    println!();

    let mut client = EipClient::with_route_path(&address, RoutePath::new().add_slot(slot)).await?;
    let mut stats: BTreeMap<&'static str, CatStats> = BTreeMap::new();
    let mut rng = Lcg::seeded();
    let mut random_values: Vec<(usize, PlcValue)> = Vec::new();

    println!("Phase 1 — read every tag");
    let t0 = Instant::now();
    for tag in &tags {
        let entry = stats.entry(tag.category).or_default();
        match client.read_tag(&tag.name).await {
            Ok(_) => entry.read_ok += 1,
            Err(_) => entry.read_fail += 1,
        }
    }
    println!("  done in {:.1}s", t0.elapsed().as_secs_f64());

    println!("Phase 2 — write random values to all writeable tags");
    let t1 = Instant::now();
    for (idx, tag) in tags.iter().enumerate() {
        if tag.write != WriteMode::Writeable {
            continue;
        }
        let Some(v) = rand_value(tag.kind, &mut rng) else {
            continue;
        };
        let entry = stats.entry(tag.category).or_default();
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
        let entry = stats.entry(tag.category).or_default();
        match client.read_tag(&tag.name).await {
            Ok(actual) if values_match(&actual, expected) => entry.verify_ok += 1,
            _ => entry.verify_fail += 1,
        }
    }
    println!("  done in {:.1}s", t2.elapsed().as_secs_f64());

    println!("Phase 4 — confirm firmware-blocked writes are still blocked");
    let t3 = Instant::now();
    for tag in &tags {
        if tag.write != WriteMode::FirmwareBlocked {
            continue;
        }
        let Some(v) = rand_value(tag.kind, &mut rng) else {
            continue;
        };
        let entry = stats.entry(tag.category).or_default();
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
        if tag.write != WriteMode::Writeable {
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
        + settle_fail;
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

    if unexpected == 0 {
        println!("RESULT: PASS");
        Ok(())
    } else {
        println!("RESULT: FAIL ({} anomalies)", unexpected);
        std::process::exit(1);
    }
}
