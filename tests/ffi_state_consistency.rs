#![cfg(feature = "ffi")]

mod plc_sim;

use plc_sim::SimulatedPlc;
use rust_ethernet_ip::ffi;
use std::ffi::CString;
use std::os::raw::c_int;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

struct FfiClientGuard {
    id: c_int,
}

impl Drop for FfiClientGuard {
    fn drop(&mut self) {
        if self.id > 0 {
            unsafe {
                let _ = ffi::eip_disconnect(self.id);
            }
        }
    }
}

struct SimHarness {
    address: String,
    stop_tx: Option<mpsc::Sender<()>>,
    join_handle: Option<thread::JoinHandle<()>>,
}

impl SimHarness {
    fn start() -> Self {
        let (addr_tx, addr_rx) = mpsc::sync_channel::<String>(1);
        let (stop_tx, stop_rx) = mpsc::channel::<()>();

        let join_handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            rt.block_on(async move {
                let sim = SimulatedPlc::start().await;
                addr_tx
                    .send(sim.address.to_string())
                    .expect("send simulator addr");

                let _ = tokio::task::spawn_blocking(move || {
                    let _ = stop_rx.recv();
                })
                .await;

                drop(sim);
                tokio::time::sleep(Duration::from_millis(25)).await;
            });
        });

        let address = addr_rx.recv().expect("receive simulator addr");
        Self {
            address,
            stop_tx: Some(stop_tx),
            join_handle: Some(join_handle),
        }
    }
}

impl Drop for SimHarness {
    fn drop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

fn connect_ffi_client(addr: &str) -> FfiClientGuard {
    let addr_c = CString::new(addr).expect("address CString");
    let id = unsafe { ffi::eip_connect(addr_c.as_ptr()) };
    assert!(id > 0, "FFI connect failed for simulator address {addr}");
    FfiClientGuard { id }
}

#[test]
fn ffi_registry_mutations_survive_repeated_clone_lookups() {
    let sim = SimHarness::start();
    let client = connect_ffi_client(&sim.address);

    for iter in 0..1_000 {
        let slot = if iter % 2 == 0 { 0_u8 } else { 1_u8 };
        let slots = [slot];

        let rc = unsafe {
            ffi::eip_set_route_path(
                client.id,
                slots.as_ptr(),
                slots.len() as c_int,
                std::ptr::null(),
                0,
                std::ptr::null_mut(),
                0,
            )
        };
        assert_eq!(rc, 0, "route-path mutation failed at iter {iter}");

        let route = ffi::client_route_path_snapshot_for_testing(client.id)
            .expect("route path should remain visible through registry lookup");
        assert_eq!(route.slots(), vec![slot], "route-path drift at iter {iter}");

        let packet_size = 504 + (iter % 3) as u32;
        let rc = unsafe { ffi::eip_set_max_packet_size(client.id, packet_size as c_int) };
        assert_eq!(rc, 0, "max-packet mutation failed at iter {iter}");
        assert_eq!(
            ffi::client_max_packet_size_for_testing(client.id),
            Some(packet_size),
            "max-packet drift at iter {iter}"
        );
    }
}
