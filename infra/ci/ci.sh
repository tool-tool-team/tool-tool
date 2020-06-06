#!/usr/bin/env bash

if [ -z "$1" ]
  then
    echo "Error: No CI command supplied"
    exit 1
fi

export RUST_BACKTRACE=1

set -e
set -u
set -x

fmt() {
  cargo fmt -- --check
}

clippy() {
  cargo clippy -- -D warnings
}

test() {
  cargo test
}

junit() {
  cargo junit --name target/JUnit.xml
}

coverage() {
  cargo tarpaulin -v
}

build_linux() {
  cargo build --release
  ls -lah target/release/tt
  strip target/release/tt
  ls -lah target/release/tt
}

build_windows() {
  cargo build --release --target x86_64-pc-windows-gnu
}

rustc --version
cargo --version

"$@"
