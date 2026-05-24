use rust_ethernet_ip::{EipClient, EtherNetIpError, PlcValue, RoutePath};
use std::env;
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

struct Lcg(u64);
impl Lcg {
    fn seeded() -> Self {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0xdead_beef_cafe_f00d);
        Self(nanos | 1)
    }
    fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((self.0 >> 32) & 0xffff_ffff) as u32
    }
    fn rand_dint(&mut self) -> i32 {
        (self.next_u32() as i32).rem_euclid(900_000) + 1_000
    }
    fn rand_int(&mut self) -> i16 {
        ((self.next_u32() & 0x7fff) as i16).rem_euclid(20_000) + 100
    }
    fn rand_real(&mut self) -> f32 {
        let scaled = (self.next_u32() % 9_000_000) as f32 / 1_000.0;
        scaled + 1.0
    }
    fn rand_bool(&mut self) -> bool {
        self.next_u32() & 1 == 1
    }
}

#[derive(Clone)]
enum Kind {
    Dint,
    Int,
    Real,
    Bool,
}

struct Spec {
    tag: &'static str,
    kind: Kind,
}

const TAGS: &[Spec] = &[
    Spec {
        tag: "gTestArray_DINT[0]",
        kind: Kind::Dint,
    },
    Spec {
        tag: "gTestArray_DINT[5]",
        kind: Kind::Dint,
    },
    Spec {
        tag: "gTestArray_DINT[9]",
        kind: Kind::Dint,
    },
    Spec {
        tag: "gTestArray_REAL[0]",
        kind: Kind::Real,
    },
    Spec {
        tag: "gTestArray_REAL[4]",
        kind: Kind::Real,
    },
    Spec {
        tag: "gTestArray_INT[0]",
        kind: Kind::Int,
    },
    Spec {
        tag: "gTestArray_INT[9]",
        kind: Kind::Int,
    },
    Spec {
        tag: "gTestArray_BOOL[0]",
        kind: Kind::Bool,
    },
    Spec {
        tag: "gTestArray_BOOL[5]",
        kind: Kind::Bool,
    },
    Spec {
        tag: "gTestArray_Large[300]",
        kind: Kind::Dint,
    },
    Spec {
        tag: "gTestArray_Large[999]",
        kind: Kind::Dint,
    },
    Spec {
        tag: "gTestUDT.Member1_DINT",
        kind: Kind::Dint,
    },
    Spec {
        tag: "gTestUDT.Member2_REAL",
        kind: Kind::Real,
    },
    Spec {
        tag: "gTestUDT.Member3_BOOL",
        kind: Kind::Bool,
    },
    Spec {
        tag: "gTestUDT.Member4_INT",
        kind: Kind::Int,
    },
    Spec {
        tag: "gTestUDT.Array_DINT[5]",
        kind: Kind::Dint,
    },
    Spec {
        tag: "gTestUDT.Array_REAL[2]",
        kind: Kind::Real,
    },
    Spec {
        tag: "gTestUDT.Array_BOOL[10]",
        kind: Kind::Bool,
    },
    Spec {
        tag: "Program:TestProgram.gTestArray_DINT[5]",
        kind: Kind::Dint,
    },
    Spec {
        tag: "Program:TestProgram.gTestArray_REAL[0]",
        kind: Kind::Real,
    },
    Spec {
        tag: "Program:TestProgram.gTestArray_BOOL[0]",
        kind: Kind::Bool,
    },
    Spec {
        tag: "Program:TestProgram.gTestUDT.Member1_DINT",
        kind: Kind::Dint,
    },
    Spec {
        tag: "Program:TestProgram.gTestUDT.Member2_REAL",
        kind: Kind::Real,
    },
    Spec {
        tag: "Program:TestProgram.gTestUDT.Member3_BOOL",
        kind: Kind::Bool,
    },
    Spec {
        tag: "Program:TestProgram.gTestUDT.Member4_INT",
        kind: Kind::Int,
    },
    Spec {
        tag: "Program:TestProgram.gTestUDT.Array_DINT[5]",
        kind: Kind::Dint,
    },
    Spec {
        tag: "Program:TestProgram.gTestUDT.Array_REAL[2]",
        kind: Kind::Real,
    },
];

fn nines_for(kind: &Kind) -> PlcValue {
    match kind {
        Kind::Dint => PlcValue::Dint(999_999),
        Kind::Int => PlcValue::Int(9_999),
        Kind::Real => PlcValue::Real(99.99),
        Kind::Bool => PlcValue::Bool(true),
    }
}

fn rand_for(kind: &Kind, rng: &mut Lcg) -> PlcValue {
    match kind {
        Kind::Dint => PlcValue::Dint(rng.rand_dint()),
        Kind::Int => PlcValue::Int(rng.rand_int()),
        Kind::Real => PlcValue::Real(rng.rand_real()),
        Kind::Bool => PlcValue::Bool(rng.rand_bool()),
    }
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
    println!("Rust binding random→verify→nines cycle");
    println!("PLC: {} (slot {})", address, slot);
    println!("Tags: {}", TAGS.len());
    println!();

    let route = RoutePath::new().add_slot(slot);
    let mut client = EipClient::with_route_path(&address, route).await?;

    let mut rng = Lcg::seeded();
    let mut random_values: Vec<(String, PlcValue)> = Vec::with_capacity(TAGS.len());

    println!("Phase 1 — write random values");
    let mut write_ok = 0usize;
    let mut write_fail = 0usize;
    for spec in TAGS {
        let v = rand_for(&spec.kind, &mut rng);
        match client.write_tag(spec.tag, v.clone()).await {
            Ok(()) => {
                println!("  WR  {:<55} {:?}", spec.tag, v);
                random_values.push((spec.tag.to_string(), v));
                write_ok += 1;
            }
            Err(e) => {
                println!("  ERR {:<55} {}", spec.tag, e);
                write_fail += 1;
            }
        }
    }
    println!("  -> {} ok, {} failed", write_ok, write_fail);
    println!();

    println!("Phase 2 — read back and verify");
    let mut verify_ok = 0usize;
    let mut verify_fail = 0usize;
    for (tag, expected) in &random_values {
        match client.read_tag(tag).await {
            Ok(actual) => {
                let ok = values_match(&actual, expected);
                println!(
                    "  {}  {:<55} expected={:?} actual={:?}",
                    if ok { "OK " } else { "MIS" },
                    tag,
                    expected,
                    actual
                );
                if ok {
                    verify_ok += 1;
                } else {
                    verify_fail += 1;
                }
            }
            Err(e) => {
                println!("  ERR {:<55} {}", tag, e);
                verify_fail += 1;
            }
        }
    }
    println!(
        "  -> {} matched, {} mismatched/failed",
        verify_ok, verify_fail
    );
    println!();

    println!("Phase 3 — settle to terminal state (DINT=999999, INT=9999, REAL=99.99, BOOL=true)");
    let mut final_ok = 0usize;
    let mut final_fail = 0usize;
    for spec in TAGS {
        let v = nines_for(&spec.kind);
        match client.write_tag(spec.tag, v.clone()).await {
            Ok(()) => {
                final_ok += 1;
            }
            Err(e) => {
                println!("  ERR {:<55} {}", spec.tag, e);
                final_fail += 1;
            }
        }
    }
    println!(
        "  -> {} settled to nines/true, {} failed",
        final_ok, final_fail
    );
    println!();

    println!(
        "Summary: random_writes={}/{}, verify={}/{}, terminal_writes={}/{}",
        write_ok,
        TAGS.len(),
        verify_ok,
        write_ok,
        final_ok,
        TAGS.len()
    );

    if write_fail == 0 && verify_fail == 0 && final_fail == 0 {
        println!("RESULT: PASS");
        Ok(())
    } else {
        println!("RESULT: FAIL");
        std::process::exit(1);
    }
}
