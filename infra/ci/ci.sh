#!/usr/bin/env bash

if [ -z "$1" ]
  then
    echo "Error: No CI command supplied"
    exit 1
fi

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

build_linux() {
  cargo build --release
}

build_windows() {
  cargo build --release --target x86_64-pc-windows-gnu
}

rustc --version
cargo --version

"$@"
