#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT="$ROOT_DIR/csharp/RustEtherNetIp/RustEtherNetIp.csproj"
OUTPUT_DIR="${1:-$ROOT_DIR/artifacts/nuget}"
CONFIGURATION="${CONFIGURATION:-Release}"
NATIVE_LIB="$ROOT_DIR/target/release/librust_ethernet_ip.dylib"
STAGE_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "$STAGE_DIR"
}
trap cleanup EXIT

if [[ ! -f "$NATIVE_LIB" ]]; then
  echo "Missing native library: $NATIVE_LIB" >&2
  echo "Build it first with: cargo build --release" >&2
  exit 1
fi

mkdir -p "$OUTPUT_DIR"

dotnet pack "$PROJECT" -c "$CONFIGURATION" -o "$OUTPUT_DIR"

PACKAGE_PATH="$(find "$OUTPUT_DIR" -maxdepth 1 -name 'RustEtherNetIp.*.nupkg' ! -name '*.symbols.nupkg' | head -n 1)"
if [[ -z "$PACKAGE_PATH" ]]; then
  echo "Could not find generated NuGet package in $OUTPUT_DIR" >&2
  exit 1
fi

mkdir -p "$STAGE_DIR/runtimes/osx-arm64/native"
cp "$NATIVE_LIB" "$STAGE_DIR/runtimes/osx-arm64/native/"

(
  cd "$STAGE_DIR"
  zip -q -u "$PACKAGE_PATH" "runtimes/osx-arm64/native/$(basename "$NATIVE_LIB")"
)

echo "NuGet package ready: $PACKAGE_PATH"
