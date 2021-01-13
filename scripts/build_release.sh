#!/usr/bin/env bash

set -e
set -u
set -o pipefail

cargo test --target=x86_64-unknown-linux-musl

cargo build --release --target=x86_64-unknown-linux-musl
strip target/x86_64-unknown-linux-musl/release/tt

(cd integration-tests && cargo test)


cargo build --release --target=x86_64-pc-windows-gnu
#CARGO_BUILD_TARGET=x86_64-pc-windows-gnu CARGO_TARGET_DIR=target/x86_64-pc-windows-gnu cargo build --release
#cargo build --release

rm -rf target/package
mkdir -p target/package
cp -f target/x86_64-pc-windows-gnu/release/tt.exe target/x86_64-unknown-linux-musl/release/tt target/package
strip target/package/tt
strip target/package/tt.exe
tar -C target/package -czvf target/tt.tar.gz --transform s:'./*':: .
