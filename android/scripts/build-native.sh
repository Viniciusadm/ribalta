#!/usr/bin/env bash
set -euo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"
language="$here/.."
out="${1:-$here/app/build/generated/jniLibs}"
api=26

sdk="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-}}"
if [[ -z "$sdk" && -f "$here/local.properties" ]]; then
  sdk="$(sed -n 's/^sdk\.dir=//p' "$here/local.properties")"
fi
ndk="${ANDROID_NDK_HOME:-}"
if [[ -z "$ndk" ]]; then
  ndk="$(ls -d "$sdk"/ndk/* 2>/dev/null | sort -V | tail -1)"
fi
if [[ ! -d "$ndk" ]]; then
  echo "Android NDK not found. Install it with the SDK manager or set ANDROID_NDK_HOME." >&2
  exit 1
fi
toolchain="$(ls -d "$ndk"/toolchains/llvm/prebuilt/* | head -1)/bin"

build() {
  local target="$1" abi="$2" clang="$3"
  local variable
  variable="CARGO_TARGET_$(echo "$target" | tr 'a-z-' 'A-Z_')_LINKER"
  env "$variable=$toolchain/$clang" "CC_${target//-/_}=$toolchain/$clang" "AR_${target//-/_}=$toolchain/llvm-ar" \
    cargo build --release --target "$target" -p ribalta-jni --manifest-path "$language/Cargo.toml"
  mkdir -p "$out/$abi"
  cp "$language/target/$target/release/libribalta_jni.so" "$out/$abi/"
}

build aarch64-linux-android arm64-v8a "aarch64-linux-android$api-clang"
build x86_64-linux-android x86_64 "x86_64-linux-android$api-clang"
