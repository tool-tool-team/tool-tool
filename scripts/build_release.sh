#!/usr/bin/env bash

set -e
set -u
set -o pipefail

cargo build --release
#CARGO_BUILD_TARGET=x86_64-pc-windows-gnu CARGO_TARGET_DIR=target/x86_64-pc-windows-gnu cargo build --release
cargo build --release --target=x86_64-pc-windows-gnu

(cd integration-tests && cargo test)


rm -rf target/package
mkdir -p target/package
mv target/x86_64-pc-windows-gnu/release/tt.exe target/release/tt target/package
tar -C target/package -czvf target/tt.tar.gz --transform s:'./*':: .
