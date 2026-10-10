#!/usr/bin/env bash

# Use before committing. CI runs this same script.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

MSRV="1.94"

# Check that a toolchain is present.
require_toolchain() {
  local tc="$1"; shift
  if ! rustup toolchain list | grep -q "^$tc"; then
    echo "Toolchain '$tc' is missing. Install it with:" >&2
    echo "  rustup toolchain install $tc $*" >&2
    exit 1
  fi
}

require_toolchain stable --component rustfmt --component clippy
require_toolchain "$MSRV"

run() {
  echo "==> $*"
  "$@"
}

run cargo +stable fmt --check
run cargo +stable clippy --workspace --all-features --all-targets -- -D warnings
run cargo +stable test --workspace --all-features
run cargo +stable test -p valid8r_derive_tests --all-features --doc -- --include-ignored
run cargo +stable test -p valid8r --no-default-features --doc
run env RUSTDOCFLAGS="-D warnings" cargo +stable doc --workspace --all-features --no-deps

# The valid8r crate's feature combinations. The derive tests always build
# valid8r with the full feature set, so only `-p valid8r` is swept here.
for features in "" "email" "phone_number" "cards" "url" "indexmap" "full"; do
  run cargo +stable test -p valid8r --no-default-features ${features:+--features "$features"} --all-targets
done

echo "==> Checking MSRV ($MSRV)..."
run cargo +"$MSRV" check --workspace --all-features

echo "All checks passed."
