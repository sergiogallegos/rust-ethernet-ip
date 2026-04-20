#[path = "../tests/plc_sim.rs"]
mod plc_sim;

use plc_sim::SimulatedPlc;
use std::io::{self, Write};

#[tokio::main]
async fn main() {
    let sim = SimulatedPlc::start().await;
    println!("{}", sim.address);
    let _ = io::stdout().flush();

    let _ = tokio::signal::ctrl_c().await;
    drop(sim);
}
