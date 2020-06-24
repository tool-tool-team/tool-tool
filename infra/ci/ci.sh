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
  cargo tarpaulin -v --exclude-files */windows.rs
}

build_linux() {
  TARGET=x86_64-unknown-linux-musl
  cargo build --release --target $TARGET
  ls -lah target/$TARGET/release/tt
  strip target/$TARGET/release/tt
  ls -lah target/$TARGET/release/tt
  # check that this is not dynamically linked
  ldd target/$TARGET/release/tt || true
}

build_windows() {
  TARGET=x86_64-pc-windows-gnu
  cargo build --release --target $TARGET
  ls -lah target/$TARGET/release/tt.exe
  strip target/$TARGET/release/tt.exe
  ls -lah target/$TARGET/release/tt.exe

}

rustc --version
cargo --version

"$@"
