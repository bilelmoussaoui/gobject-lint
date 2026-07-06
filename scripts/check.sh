#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

TOOLCHAIN="${TOOLCHAIN:-+stable}"

pass=0
fail=0

run() {
    echo "==> $*"
    if "$@"; then
        pass=$((pass + 1))
    else
        fail=$((fail + 1))
    fi
    echo
}

# rustfmt.toml uses nightly-only options
run cargo +nightly fmt --all -- --check

run cargo "$TOOLCHAIN" clippy --all-targets --all-features -- -D warnings
run cargo "$TOOLCHAIN" build --release --bin gobject-linter
run cargo "$TOOLCHAIN" build --release --bin gobject-linter-lsp --features lsp
run cargo "$TOOLCHAIN" test --all --all-features
run python3 tests/test_rule_consistency.py

if command -v meson &>/dev/null; then
    fixtures_build=$(mktemp -d)
    trap 'rm -rf "$fixtures_build"' EXIT
    run meson setup "$fixtures_build" tests/fixtures
    run ninja -C "$fixtures_build"
else
    echo "==> fixtures build (skipped, meson not installed)"
    echo
fi

if command -v typos &>/dev/null; then
    run typos
else
    echo "==> typos (skipped, not installed)"
    echo
fi

echo "--- Results: $pass passed, $fail failed ---"
exit "$fail"
