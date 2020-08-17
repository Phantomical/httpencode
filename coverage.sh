#!/bin/bash

set -eo pipefail

if ! command -v grcov &> /dev/null; then
  echo "ERROR: grcov could not be found!"
  echo ""
  echo "Run 'cargo install grcov' to install it."
  exit 1
fi

export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
export RUSTDOCFLAGS="-Cpanic=abort"

if [ -Z ${CARGO_TARGET_DIR+x} ]; then
  TARGET=target
else
  TARGET="$CARGO_TARGET_DIR"
fi

rm -rf "$TARGET/coverage"

cargo clean
cargo test --features std

grcov "$TARGET/debug/" -s . -t html --llvm --branch \
  --ignore-not-existing -o "$TARGET/coverage" \
  --excl-line '#\[derive' \
  --commit-sha "$(git rev-parse HEAD)" \
  --vcs-branch "$(git rev-parse --abbrev-ref HEAD)"

