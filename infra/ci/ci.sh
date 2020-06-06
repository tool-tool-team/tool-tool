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

build() {
  cargo build --release
}

rustc --version
cargo --version

"$@"
