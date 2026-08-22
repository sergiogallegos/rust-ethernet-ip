#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 4 ]]; then
    echo "usage: $0 PLC_ADDRESS[:PORT] [CPU_SLOT] [PROGRAM_NAME] [--allow-writes]" >&2
    exit 2
fi

plc_address="$1"
cpu_slot="${2:-0}"
program_name="${3:-TestProgram}"
write_opt_in="${4:-}"

if [[ "$write_opt_in" != "--allow-writes" ]]; then
    echo "refusing live run: pass --allow-writes as the final argument" >&2
    echo "four dedicated DINT test elements will be changed, verified, and restored" >&2
    exit 2
fi

case "$(uname -s)" in
    Darwin) native_library="$PWD/target/release/librust_ethernet_ip.dylib" ;;
    Linux) native_library="$PWD/target/release/librust_ethernet_ip.so" ;;
    *)
        echo "this orchestration script supports macOS and Linux; use the documented per-language commands on Windows" >&2
        exit 2
        ;;
esac

common_args=(
    --plc-address "$plc_address"
    --plc-slot "$cpu_slot"
    --program "$program_name"
    --allow-writes
)

echo "=== Rust: batch + UDT + controller/program discovery ==="
cargo run --release --features ffi --locked --example hardware_feature_gate -- "${common_args[@]}"

echo "=== Python: batch + UDT; discovery N/A ==="
PYTHONPATH=python python3 python/examples/hardware_feature_gate.py "${common_args[@]}"

echo "=== C#: batch + UDT + controller discovery ==="
dotnet run --project examples/CSharpHardwareFeatureGate/CSharpHardwareFeatureGate.csproj \
    -c Release -- "${common_args[@]}"

echo "=== C/C++: batch + UDT + controller discovery ==="
cmake -S examples/cpp -B target/cpp -DRUST_ETHERNET_IP_NATIVE_LIB="$native_library"
cmake --build target/cpp --target cpp_hardware_feature_gate --config Release
target/cpp/cpp_hardware_feature_gate "${common_args[@]}"

echo "=== PASS: all applicable cross-binding companion checks completed ==="
