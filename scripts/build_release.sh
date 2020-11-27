#!/usr/bin/env bash

set -e
set -u
set -o pipefail

cargo build --release
#CARGO_BUILD_TARGET=x86_64-pc-windows-gnu CARGO_TARGET_DIR=target/x86_64-pc-windows-gnu cargo build --release
CARGO_TARGET_DIR=target/x86_64-pc-windows-gnu cargo build --release --target=x86_64-pc-windows-gnu
--target=x86_64-pc-windows-gnu

(cd integration-tests && cargo test)

 tar -czvf target/tt.tar.gz /target/x86_64-pc-windows-gnu/release/tt.exe